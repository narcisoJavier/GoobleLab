//! Optional Google Drive publishing. OAuth tokens stay in memory for this app session.
use crate::notebook::{NotebookConfig, Template, build_notebook, file_stem, validate};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use reqwest::{
    Method,
    blocking::{Client, RequestBuilder, Response},
    header,
};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};
use url::Url;

const AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const FILES_URL: &str = "https://www.googleapis.com/drive/v3/files";
const UPLOAD_URL: &str = "https://www.googleapis.com/upload/drive/v3/files";
const DRIVE_SCOPE: &str = "https://www.googleapis.com/auth/drive.file";
const CALLBACK_PATH: &str = "/oauth2/callback";
const MAX_IMAGE_BYTES: u64 = 25 * 1024 * 1024;
const MULTIPART_LIMIT: usize = 5 * 1024 * 1024;
const MAX_CALLBACK_BYTES: usize = 8192;

/// Deliberately has no Debug implementation: tokens must not appear in logs.
#[derive(Clone)]
pub struct Session {
    client: Client,
    client_id: String,
    client_secret: Option<String>,
    access_token: String,
    refresh_token: Option<String>,
    expires_at: Instant,
}

#[derive(Clone)]
pub struct Published {
    pub colab_url: String,
    pub drive_url: String,
    pub notebook_id: String,
}

#[derive(Deserialize)]
struct ClientFile {
    installed: DesktopClient,
}

#[derive(Deserialize)]
struct DesktopClient {
    client_id: String,
    client_secret: Option<String>,
}

#[derive(Deserialize)]
struct TokenReply {
    access_token: String,
    expires_in: u64,
    refresh_token: Option<String>,
    token_type: String,
}

fn http_client() -> Result<Client, String> {
    Client::builder()
        .timeout(Duration::from_secs(60))
        .connect_timeout(Duration::from_secs(15))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|_| "Could not initialize the Google HTTPS client.".into())
}

fn random_text() -> String {
    URL_SAFE_NO_PAD.encode(rand::random::<[u8; 32]>())
}

fn pkce_challenge(verifier: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()))
}

fn load_client(path: &Path) -> Result<DesktopClient, String> {
    let metadata = fs::metadata(path)
        .map_err(|_| "Select the downloaded Desktop app OAuth JSON file.".to_string())?;
    if !metadata.is_file() || metadata.len() > 64 * 1024 {
        return Err("OAuth credentials must be a Desktop app JSON file under 64 KB.".into());
    }
    let bytes = fs::read(path).map_err(|_| "Could not read the OAuth JSON file.".to_string())?;
    let credentials: ClientFile = serde_json::from_slice(&bytes).map_err(|_| {
        "Use a Google OAuth Desktop app JSON file containing an installed client.".to_string()
    })?;
    let client = credentials.installed;
    if client.client_id.len() > 512
        || !client.client_id.ends_with(".apps.googleusercontent.com")
        || !client
            .client_id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'.')
        || client
            .client_secret
            .as_ref()
            .is_some_and(|s| s.len() > 512 || s.chars().any(char::is_control))
    {
        return Err("The Desktop OAuth client configuration is invalid.".into());
    }
    Ok(client)
}

fn exchange_token(client: &Client, fields: &[(String, String)]) -> Result<TokenReply, String> {
    let response = client.post(TOKEN_URL).form(fields).send().map_err(|_| {
        "Could not reach Google's token service. Check your connection and sign in again."
            .to_string()
    })?;
    if !response.status().is_success() {
        return Err(format!(
            "Google sign-in could not finish (HTTP {}). Check the Desktop OAuth client and test-user settings.",
            response.status().as_u16()
        ));
    }
    let token: TokenReply = response
        .json()
        .map_err(|_| "Google returned an invalid token response. Sign in again.".to_string())?;
    if token.access_token.is_empty()
        || token.access_token.len() > 16_384
        || !token.token_type.eq_ignore_ascii_case("bearer")
        || token.expires_in == 0
    {
        return Err("Google returned an invalid token response. Sign in again.".into());
    }
    Ok(token)
}

fn expiry(seconds: u64) -> Instant {
    // Renew early, while leaving short-lived test responses usable.
    Instant::now() + Duration::from_secs(seconds.saturating_sub(60).clamp(1, 24 * 60 * 60))
}

impl Session {
    fn ensure_access(&mut self) -> Result<(), String> {
        if Instant::now() < self.expires_at {
            return Ok(());
        }
        let refresh = self
            .refresh_token
            .as_ref()
            .ok_or("Your Google session expired. Sign in again.")?;
        let mut fields = vec![
            ("client_id".into(), self.client_id.clone()),
            ("refresh_token".into(), refresh.clone()),
            ("grant_type".into(), "refresh_token".into()),
        ];
        if let Some(secret) = &self.client_secret {
            fields.push(("client_secret".into(), secret.clone()));
        }
        let token = exchange_token(&self.client, &fields)?;
        self.access_token = token.access_token;
        self.expires_at = expiry(token.expires_in);
        if token.refresh_token.is_some() {
            self.refresh_token = token.refresh_token;
        }
        Ok(())
    }

    fn request(&mut self, method: Method, url: &str) -> Result<RequestBuilder, String> {
        self.ensure_access()?;
        Ok(self
            .client
            .request(method, url)
            .bearer_auth(&self.access_token))
    }
}

/// Opens the system browser; waits at most five minutes for the loopback callback.
pub fn sign_in(client_json_path: &Path, cancel: Arc<AtomicBool>) -> Result<Session, String> {
    let credentials = load_client(client_json_path)?;
    if cancel.load(Ordering::Relaxed) {
        return Err("Google sign-in was cancelled.".into());
    }
    let client = http_client()?;
    let listener = TcpListener::bind(("127.0.0.1", 0)).map_err(|_| {
        "Could not open the local Google sign-in callback. Check your firewall.".to_string()
    })?;
    listener
        .set_nonblocking(true)
        .map_err(|_| "Could not configure the local sign-in callback.".to_string())?;
    let port = listener
        .local_addr()
        .map_err(|_| "Could not read the local callback address.".to_string())?
        .port();
    let redirect = format!("http://127.0.0.1:{port}{CALLBACK_PATH}");
    let verifier = random_text();
    let state = random_text();
    let mut authorization = Url::parse(AUTH_URL).expect("fixed Google authorization URL");
    authorization.query_pairs_mut().extend_pairs([
        ("client_id", credentials.client_id.as_str()),
        ("redirect_uri", redirect.as_str()),
        ("response_type", "code"),
        ("scope", DRIVE_SCOPE),
        ("code_challenge", pkce_challenge(&verifier).as_str()),
        ("code_challenge_method", "S256"),
        ("state", state.as_str()),
        ("access_type", "offline"),
        ("prompt", "consent"),
    ]);
    webbrowser::open(authorization.as_str())
        .map_err(|_| "Could not open your system browser for Google sign-in.".to_string())?;
    let code = wait_for_callback(listener, port, &state, &cancel)?;
    if cancel.load(Ordering::Relaxed) {
        return Err("Google sign-in was cancelled.".into());
    }
    let mut fields = vec![
        ("client_id".into(), credentials.client_id.clone()),
        ("code".into(), code),
        ("code_verifier".into(), verifier),
        ("redirect_uri".into(), redirect),
        ("grant_type".into(), "authorization_code".into()),
    ];
    if let Some(secret) = &credentials.client_secret {
        fields.push(("client_secret".into(), secret.clone()));
    }
    let token = exchange_token(&client, &fields)?;
    if cancel.load(Ordering::Relaxed) {
        return Err("Google sign-in was cancelled.".into());
    }
    Ok(Session {
        client,
        client_id: credentials.client_id,
        client_secret: credentials.client_secret,
        access_token: token.access_token,
        refresh_token: token.refresh_token,
        expires_at: expiry(token.expires_in),
    })
}

#[derive(Debug, PartialEq)]
enum Callback {
    Code(String),
    Denied,
}

fn same_state(left: &str, right: &str) -> bool {
    left.len() == right.len()
        && left
            .bytes()
            .zip(right.bytes())
            .fold(0u8, |a, (l, r)| a | (l ^ r))
            == 0
}

fn parse_callback(request: &str, port: u16, expected_state: &str) -> Result<Callback, ()> {
    if request.len() > MAX_CALLBACK_BYTES || !request.ends_with("\r\n\r\n") {
        return Err(());
    }
    let mut lines = request.split("\r\n");
    let first: Vec<_> = lines.next().ok_or(())?.split_whitespace().collect();
    if first.len() != 3
        || first[0] != "GET"
        || first[2] != "HTTP/1.1"
        || !first[1].starts_with('/')
        || first[1].starts_with("//")
        || first[1].contains('#')
    {
        return Err(());
    }
    let host = format!("127.0.0.1:{port}");
    let mut host_count = 0;
    for line in lines.take_while(|line| !line.is_empty()) {
        let (name, value) = line.split_once(':').ok_or(())?;
        let value = value.trim();
        if name.eq_ignore_ascii_case("host") {
            host_count += 1;
            if value != host {
                return Err(());
            }
        }
        // Normal top-level browser redirects do not send Origin. Reject scripted cross-origin requests.
        if name.eq_ignore_ascii_case("origin")
            || name.eq_ignore_ascii_case("transfer-encoding")
            || (name.eq_ignore_ascii_case("content-length") && value != "0")
        {
            return Err(());
        }
    }
    if host_count != 1 {
        return Err(());
    }
    let url = Url::parse(&format!("http://{host}{}", first[1])).map_err(|_| ())?;
    if url.path() != CALLBACK_PATH
        || url.host_str() != Some("127.0.0.1")
        || url.port() != Some(port)
    {
        return Err(());
    }
    let mut state = None;
    let mut code = None;
    let mut error = None;
    for (key, value) in url.query_pairs() {
        let slot = match key.as_ref() {
            "state" => &mut state,
            "code" => &mut code,
            "error" => &mut error,
            _ => continue,
        };
        if slot.is_some() {
            return Err(());
        }
        *slot = Some(value.into_owned());
    }
    if !state
        .as_deref()
        .is_some_and(|state| same_state(state, expected_state))
    {
        return Err(());
    }
    match (code, error) {
        (Some(code), None)
            if !code.is_empty() && code.len() <= 4096 && !code.chars().any(char::is_control) =>
        {
            Ok(Callback::Code(code))
        }
        (None, Some(_)) => Ok(Callback::Denied),
        _ => Err(()),
    }
}

fn read_callback(
    stream: &mut TcpStream,
    cancel: &AtomicBool,
    deadline: Instant,
) -> Result<String, ()> {
    stream
        .set_read_timeout(Some(Duration::from_millis(200)))
        .map_err(|_| ())?;
    let request_deadline = (Instant::now() + Duration::from_secs(5)).min(deadline);
    let mut bytes = Vec::new();
    let mut buffer = [0u8; 1024];
    while Instant::now() < request_deadline && !cancel.load(Ordering::Relaxed) {
        match stream.read(&mut buffer) {
            Ok(0) => return Err(()),
            Ok(count) => {
                bytes.extend_from_slice(&buffer[..count]);
                if bytes.len() > MAX_CALLBACK_BYTES {
                    return Err(());
                }
                if let Some(end) = bytes.windows(4).position(|w| w == b"\r\n\r\n") {
                    bytes.truncate(end + 4);
                    return String::from_utf8(bytes).map_err(|_| ());
                }
            }
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) => {}
            Err(_) => return Err(()),
        }
    }
    Err(())
}

fn browser_reply(stream: &mut TcpStream, accepted: bool) {
    let (status, message) = if accepted {
        (
            "200 OK",
            "Sign-in response received. You can close this tab and return to GoobleLab.",
        )
    } else {
        (
            "400 Bad Request",
            "Sign-in was not completed. Return to GoobleLab to continue or try again.",
        )
    };
    let html = format!(
        "<!doctype html><html lang=\"en\"><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width\"><title>GoobleLab sign-in</title><body><p>{message}</p></body></html>"
    );
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nCache-Control: no-store\r\nContent-Security-Policy: default-src 'none'; frame-ancestors 'none'\r\nX-Content-Type-Options: nosniff\r\nConnection: close\r\n\r\n{html}",
        html.len()
    );
    let _ = stream.set_write_timeout(Some(Duration::from_secs(1)));
    let _ = stream.write_all(response.as_bytes());
}

fn wait_for_callback(
    listener: TcpListener,
    port: u16,
    state: &str,
    cancel: &AtomicBool,
) -> Result<String, String> {
    let deadline = Instant::now() + Duration::from_secs(300);
    while Instant::now() < deadline {
        if cancel.load(Ordering::Relaxed) {
            return Err("Google sign-in was cancelled.".into());
        }
        match listener.accept() {
            Ok((mut stream, peer)) => {
                if !peer.ip().is_loopback() {
                    continue;
                }
                let parsed = read_callback(&mut stream, cancel, deadline)
                    .and_then(|request| parse_callback(&request, port, state));
                browser_reply(&mut stream, matches!(parsed, Ok(Callback::Code(_))));
                match parsed {
                    Ok(Callback::Code(code)) => return Ok(code),
                    Ok(Callback::Denied) => {
                        return Err("Google access was not granted. You can sign in again.".into());
                    }
                    Err(()) => continue,
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(50))
            }
            Err(_) => {
                return Err(
                    "The local Google sign-in callback failed. Try signing in again.".into(),
                );
            }
        }
    }
    Err("Google sign-in timed out after five minutes. Try again.".into())
}

fn drive_response(response: Response) -> Result<Response, String> {
    if response.status().is_success() {
        return Ok(response);
    }
    let status = response.status().as_u16();
    let help = match status {
        401 => "Sign in again; Google no longer accepts this session.",
        403 => "Check that Drive API is enabled and your OAuth test user is allowed.",
        429 => "Google is rate limiting uploads. Wait briefly and try again.",
        _ => "Check your connection and Drive storage, then try again.",
    };
    // Do not echo response bodies: account details and credentials must stay out of UI errors.
    Err(format!(
        "Google Drive request failed (HTTP {status}). {help}"
    ))
}

fn file_id(response: Response) -> Result<String, String> {
    let data: Value = drive_response(response)?
        .json()
        .map_err(|_| "Google Drive returned an invalid file response.".to_string())?;
    let id = data
        .get("id")
        .and_then(Value::as_str)
        .filter(|id| {
            !id.is_empty()
                && id.len() <= 200
                && id
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        })
        .ok_or("Google Drive did not return a valid file ID.")?;
    Ok(id.to_string())
}

fn upload_error(_: reqwest::Error) -> String {
    "Could not reach Google Drive. Check your connection and try again.".into()
}

fn multipart_body(metadata: &Value, mime: &str, data: &[u8], boundary: &str) -> Vec<u8> {
    let mut body = format!("--{boundary}\r\nContent-Type: application/json; charset=UTF-8\r\n\r\n{metadata}\r\n--{boundary}\r\nContent-Type: {mime}\r\n\r\n").into_bytes();
    body.extend_from_slice(data);
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
    body
}

fn resumable_location(location: &str) -> Result<Url, String> {
    let url = Url::parse(location)
        .map_err(|_| "Google Drive returned an invalid upload location.".to_string())?;
    if url.scheme() != "https"
        || url.host_str() != Some("www.googleapis.com")
        || !url.username().is_empty()
        || url.password().is_some()
        || url.port_or_known_default() != Some(443)
        || url.path() != "/upload/drive/v3/files"
        || url.fragment().is_some()
    {
        return Err(
            "Google Drive returned an unexpected upload location. Upload was stopped.".into(),
        );
    }
    Ok(url)
}

fn upload_file(
    session: &mut Session,
    metadata: &Value,
    mime: &str,
    bytes: Vec<u8>,
) -> Result<String, String> {
    if bytes.len() <= MULTIPART_LIMIT {
        let boundary = format!("gooblelab_{}", random_text());
        let response = session
            .request(Method::POST, UPLOAD_URL)?
            .query(&[("uploadType", "multipart"), ("fields", "id")])
            .header(
                header::CONTENT_TYPE,
                format!("multipart/related; boundary={boundary}"),
            )
            .body(multipart_body(metadata, mime, &bytes, &boundary))
            .send()
            .map_err(upload_error)?;
        return file_id(response);
    }
    let response = session
        .request(Method::POST, UPLOAD_URL)?
        .query(&[("uploadType", "resumable"), ("fields", "id")])
        .header("X-Upload-Content-Type", mime)
        .header("X-Upload-Content-Length", bytes.len())
        .json(metadata)
        .send()
        .map_err(upload_error)?;
    let response = drive_response(response)?;
    let location = response
        .headers()
        .get(header::LOCATION)
        .and_then(|v| v.to_str().ok())
        .ok_or("Google Drive did not return an upload location.")?;
    let location = resumable_location(location)?;
    let response = session
        .request(Method::PUT, location.as_str())?
        .header(header::CONTENT_TYPE, mime)
        .header(header::CONTENT_LENGTH, bytes.len())
        .body(bytes)
        .send()
        .map_err(upload_error)?;
    file_id(response)
}

fn image_type(path: &Path, bytes: &[u8]) -> Result<(&'static str, &'static str), String> {
    let extension = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    match extension.as_str() {
        "png" if bytes.starts_with(b"\x89PNG\r\n\x1a\n") => Ok(("image/png", "reference.png")),
        "jpg" | "jpeg" if bytes.starts_with(&[0xff, 0xd8, 0xff]) => {
            Ok(("image/jpeg", "reference.jpg"))
        }
        "webp" if bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP" => {
            Ok(("image/webp", "reference.webp"))
        }
        _ => Err("Reference image must be a PNG, JPG, or WebP file matching its extension.".into()),
    }
}

/// Creates a new project folder and uploads its notebook and optional reference image.
pub fn upload_project(session: &mut Session, config: &NotebookConfig) -> Result<Published, String> {
    validate(config)?;
    let image = match selected_image_path(config) {
        Some(path) => {
            let size = fs::metadata(path)
                .map_err(|_| "Could not read the reference image.".to_string())?
                .len();
            if size == 0 || size > MAX_IMAGE_BYTES {
                return Err("Reference images must be no larger than 25 MiB.".into());
            }
            // Bound the read too, in case another program changes the file after metadata inspection.
            let mut bytes = Vec::new();
            fs::File::open(path)
                .map_err(|_| "Could not open the reference image.".to_string())?
                .take(MAX_IMAGE_BYTES + 1)
                .read_to_end(&mut bytes)
                .map_err(|_| "Could not read the reference image.".to_string())?;
            if bytes.len() as u64 > MAX_IMAGE_BYTES {
                return Err("Reference images must be no larger than 25 MiB.".into());
            }
            let (mime, name) = image_type(path, &bytes)?;
            Some((mime, name, bytes))
        }
        None => None,
    };
    let suffix = rand::random::<u32>();
    let folder_name = format!("GoobleLab - {} - {suffix:08x}", file_stem(&config.name));
    if folder_name.contains(['/', '\\']) {
        return Err("Use a project name without path separators.".into());
    }
    let image_drive_path = image
        .as_ref()
        .map(|(_, name, _)| format!("/content/drive/MyDrive/{folder_name}/{name}"));
    let notebook = build_notebook(config, image_drive_path.as_deref())?;
    let folder_id = file_id(
        session
            .request(Method::POST, FILES_URL)?
            .query(&[("fields", "id")])
            .json(&json!({"name": folder_name, "mimeType": "application/vnd.google-apps.folder"}))
            .send()
            .map_err(upload_error)?,
    )?;
    if let Some((mime, name, bytes)) = image {
        upload_file(
            session,
            &json!({"name": name, "parents": [folder_id]}),
            mime,
            bytes,
        )?;
    }
    let notebook_id = upload_file(
        session,
        &json!({"name": format!("{}.ipynb", file_stem(&config.name)), "parents": [folder_id], "mimeType": "application/x-ipynb+json"}),
        "application/x-ipynb+json",
        notebook,
    )?;
    Ok(Published {
        colab_url: format!("https://colab.research.google.com/drive/{notebook_id}"),
        drive_url: format!("https://drive.google.com/drive/folders/{folder_id}"),
        notebook_id,
    })
}

fn selected_image_path(config: &NotebookConfig) -> Option<&Path> {
    config
        .image_path
        .as_deref()
        .filter(|_| config.template != Template::TextToVideo)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn callback(target: &str, headers: &str) -> String {
        format!("GET {target} HTTP/1.1\r\nHost: 127.0.0.1:3456\r\n{headers}\r\n")
    }

    #[test]
    fn pkce_matches_rfc7636_vector() {
        assert_eq!(
            pkce_challenge("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"),
            "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
        );
        let verifier = random_text();
        assert_eq!(verifier.len(), 43);
        assert!(
            verifier
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        );
    }

    #[test]
    fn callback_checks_state_path_origin_and_duplicate_parameters() {
        let good = callback("/oauth2/callback?code=test%2Bcode&state=expected", "");
        assert_eq!(
            parse_callback(&good, 3456, "expected"),
            Ok(Callback::Code("test+code".into()))
        );
        let denied = callback("/oauth2/callback?error=access_denied&state=expected", "");
        assert_eq!(
            parse_callback(&denied, 3456, "expected"),
            Ok(Callback::Denied)
        );
        for bad in [
            good.replace("state=expected", "state=wrong"),
            good.replace("GET ", "POST "),
            good.replace("/oauth2/callback?", "/other?"),
            good.replace("127.0.0.1:3456", "attacker.example:3456"),
            callback("/oauth2/callback?code=x&state=expected&state=expected", ""),
            callback("/oauth2/callback?code=x&code=y&state=expected", ""),
            callback(
                "/oauth2/callback?code=x&state=expected",
                "Origin: https://attacker.example\r\n",
            ),
            callback(
                "/oauth2/callback?code=x&state=expected",
                "Content-Length: 12\r\n",
            ),
            callback(
                "//attacker.example/oauth2/callback?code=x&state=expected",
                "",
            ),
            format!("{good}{}", "x".repeat(MAX_CALLBACK_BYTES)),
        ] {
            assert!(parse_callback(&bad, 3456, "expected").is_err());
        }
    }

    #[test]
    fn multipart_is_related_metadata_then_binary_and_final_boundary() {
        let body = multipart_body(
            &json!({"name": "a.ipynb"}),
            "application/x-ipynb+json",
            b"{\"cells\":[]}",
            "test_boundary",
        );
        let body = String::from_utf8(body).unwrap();
        assert!(body.starts_with(
            "--test_boundary\r\nContent-Type: application/json; charset=UTF-8\r\n\r\n"
        ));
        assert!(body.contains("{\"name\":\"a.ipynb\"}\r\n--test_boundary\r\nContent-Type: application/x-ipynb+json\r\n\r\n{\"cells\":[]}"));
        assert!(body.ends_with("\r\n--test_boundary--\r\n"));
        assert!(!body.contains("form-data"));
    }

    #[test]
    fn resumable_url_cannot_redirect_bearer_credentials() {
        assert!(
            resumable_location("https://www.googleapis.com/upload/drive/v3/files?upload_id=fake")
                .is_ok()
        );
        for bad in [
            "http://www.googleapis.com/upload/drive/v3/files",
            "https://www.googleapis.com.attacker.example/upload/drive/v3/files",
            "https://attacker.example/upload/drive/v3/files",
            "https://attacker@www.googleapis.com/upload/drive/v3/files",
            "https://www.googleapis.com:444/upload/drive/v3/files",
            "https://www.googleapis.com/other",
            "https://www.googleapis.com/upload/drive/v3/files#fragment",
        ] {
            assert!(resumable_location(bad).is_err());
        }
    }

    #[test]
    fn image_mime_requires_matching_signature() {
        assert_eq!(
            image_type(Path::new("x.PNG"), b"\x89PNG\r\n\x1a\n"),
            Ok(("image/png", "reference.png"))
        );
        assert_eq!(
            image_type(Path::new("x.jpeg"), &[0xff, 0xd8, 0xff]),
            Ok(("image/jpeg", "reference.jpg"))
        );
        assert_eq!(
            image_type(Path::new("x.webp"), b"RIFF1234WEBP"),
            Ok(("image/webp", "reference.webp"))
        );
        assert!(image_type(Path::new("x.png"), b"<html>not an image</html>").is_err());
    }

    #[test]
    fn text_to_video_never_selects_a_stale_image_for_upload() {
        let mut config = NotebookConfig {
            image_path: Some(
                std::env::temp_dir().join(format!("missing-gooblelab-{}.png", random_text())),
            ),
            ..Default::default()
        };
        assert!(validate(&config).is_ok());
        assert!(selected_image_path(&config).is_none());
        config.template = Template::ImageToVideo;
        assert_eq!(selected_image_path(&config), config.image_path.as_deref());
    }

    #[test]
    fn invalid_client_json_errors_do_not_echo_credentials() {
        let path =
            std::env::temp_dir().join(format!("gooblelab-oauth-test-{}.json", random_text()));
        let secret = "fake-secret-must-not-appear-in-errors";
        for bytes in [
            format!("{{\"web\":{{\"client_id\":\"fake\",\"client_secret\":\"{secret}\"}}}}"),
            format!(
                "{{\"installed\":{{\"client_id\":\"attacker.example\",\"client_secret\":\"{secret}\"}}}}"
            ),
        ] {
            fs::write(&path, bytes).unwrap();
            let error = match load_client(&path) {
                Err(error) => error,
                Ok(_) => panic!("invalid Desktop OAuth JSON was accepted"),
            };
            assert!(!error.contains(secret));
            assert!(!error.contains("attacker.example"));
        }
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn loopback_callback_rejects_wrong_state_then_returns_valid_code() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        listener.set_nonblocking(true).unwrap();
        let port = listener.local_addr().unwrap().port();
        let worker = thread::spawn(move || {
            wait_for_callback(listener, port, "expected", &AtomicBool::new(false))
        });
        for state in ["wrong", "expected"] {
            let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(3)))
                .unwrap();
            write!(stream, "GET /oauth2/callback?code=fake-code&state={state} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\n\r\n").unwrap();
            let mut reply = String::new();
            stream.read_to_string(&mut reply).unwrap();
            assert!(reply.starts_with(if state == "wrong" {
                "HTTP/1.1 400"
            } else {
                "HTTP/1.1 200"
            }));
            assert!(!reply.contains("fake-code"));
        }
        assert_eq!(worker.join().unwrap(), Ok("fake-code".into()));
    }
}
