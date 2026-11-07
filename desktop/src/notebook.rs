use std::path::PathBuf;

use percent_encoding::percent_decode_str;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

const NOTEBOOK: &str = include_str!("../../MiniMax_H3_Colab.ipynb");
const PRESETS: &str = include_str!("../../presets.json");

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Template {
    TextToVideo,
    ImageToVideo,
    ReferenceToVideo,
}

impl Template {
    pub fn label(self) -> &'static str {
        match self {
            Self::TextToVideo => "Text to video",
            Self::ImageToVideo => "Image to video",
            Self::ReferenceToVideo => "Reference to video",
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct NotebookConfig {
    pub name: String,
    pub template: Template,
    pub preset_id: String,
    pub prompt: String,
    pub width: u32,
    pub height: u32,
    pub seconds: f64,
    pub seed: u64,
    pub lora_source: String,
    pub lora_strength: f64,
    pub image_path: Option<PathBuf>,
}

impl Default for NotebookConfig {
    fn default() -> Self {
        Self {
            name: "My MiniMax video".into(),
            template: Template::TextToVideo,
            preset_id: "fl2va-turbo-8".into(),
            prompt: "Cinematic live-action wildlife shot, one continuous take. A small red panda sits on a moss-covered branch in a misty bamboo forest at sunrise. It gently turns toward the camera, blinks, and tilts its head while a light breeze moves its fur and the bamboo leaves. Camera: a slow, steady push-in from a medium shot to a close-up, shallow depth of field, warm golden rim light, realistic fur and natural movement. Audio: soft wind rustling bamboo, distant birds, and quiet forest ambience; no dialogue or music. No cuts, captions, text, or logos.".into(),
            width: 1344,
            height: 768,
            seconds: 5.0,
            seed: 12345,
            lora_source: String::new(),
            lora_strength: 1.0,
            image_path: None,
        }
    }
}

fn drive_path(path: &str) -> bool {
    path.starts_with("/content/drive/MyDrive/")
        && !path.contains(['\\', ':', '\0'])
        && !path.split('/').any(|part| matches!(part, "." | ".."))
}

fn safetensors_file(path: &str) -> bool {
    path.to_ascii_lowercase().ends_with(".safetensors")
        && !path.contains(['\\', ':', '\0'])
        && !path.split('/').any(|part| matches!(part, "." | ".."))
}

fn validate_lora(source: &str) -> Result<(), String> {
    if source.is_empty() {
        return Ok(());
    }
    if drive_path(source) && safetensors_file(source) {
        return Ok(());
    }
    let error = "Use an HTTPS Hugging Face .safetensors file URL or /content/drive/MyDrive/... path. Windows paths cannot be read by Colab.";
    if !source.starts_with("https://huggingface.co/") {
        return Err(error.into());
    }
    let url = url::Url::parse(source).map_err(|_| error.to_owned())?;
    if url.host_str() != Some("huggingface.co")
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
        || !matches!(
            url.query(),
            None | Some("") | Some("download=true") | Some("download=1")
        )
    {
        return Err(
            "Remove credentials and URL tokens; use the HF_TOKEN Colab secret for private LoRAs."
                .into(),
        );
    }
    // Inspect the original path too: Url normalizes /../ before exposing path().
    let original_path = source["https://huggingface.co".len()..]
        .split(['?', '#'])
        .next()
        .unwrap_or("");
    let decoded = percent_decode_str(original_path)
        .decode_utf8()
        .map_err(|_| error.to_owned())?;
    let parts: Vec<_> = decoded.trim_matches('/').split('/').collect();
    let identifier = |part: &str| {
        !part.is_empty()
            && !matches!(part, "." | "..")
            && part
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || b"_.-".contains(&c))
    };
    if parts.len() < 5
        || !matches!(parts[2], "resolve" | "blob")
        || ![parts[0], parts[1], parts[3]].into_iter().all(identifier)
        || !safetensors_file(&parts[4..].join("/"))
    {
        return Err(error.into());
    }
    Ok(())
}

pub fn validate(config: &NotebookConfig) -> Result<(), String> {
    if config.name.trim().is_empty() || config.name.chars().any(char::is_control) {
        return Err("Enter a project name without control characters.".into());
    }
    if config.prompt.trim().is_empty() {
        return Err("Enter a prompt.".into());
    }
    if !(256..=1344).contains(&config.width)
        || !(256..=1344).contains(&config.height)
        || !config.width.is_multiple_of(32)
        || !config.height.is_multiple_of(32)
        || config.width * config.height > 1344 * 768
    {
        return Err("Use dimensions divisible by 32, within 1344×768 total pixels.".into());
    }
    if !config.seconds.is_finite() || !(4.0..=15.0).contains(&config.seconds) {
        return Err("Choose a duration from 4 to 15 seconds.".into());
    }
    if !config.lora_strength.is_finite() || !(0.0..=2.0).contains(&config.lora_strength) {
        return Err("LoRA strength must be between 0 and 2.".into());
    }
    let catalog: Value = serde_json::from_str(PRESETS).map_err(|e| e.to_string())?;
    let family = catalog["presets"][&config.preset_id]["family"]
        .as_str()
        .ok_or("Choose a supported MiniMax H3 preset.")?;
    if (config.template == Template::ReferenceToVideo) != (family == "ref2va") {
        return Err(
            "Reference to video requires a Ref2VA preset; text/image to video requires FL2VA."
                .into(),
        );
    }
    if config.template != Template::TextToVideo {
        let image = config.image_path.as_ref().ok_or("Choose an input image.")?;
        let extension = image
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if !image.is_file() || !matches!(extension.as_str(), "png" | "jpg" | "jpeg" | "webp") {
            return Err("Choose an existing PNG, JPEG, or WebP image.".into());
        }
    }
    validate_lora(config.lora_source.trim())
}

pub fn file_stem(name: &str) -> String {
    let mut stem = String::new();
    for c in name.chars() {
        if c.is_ascii_alphanumeric() {
            stem.push(c.to_ascii_lowercase());
        } else if !stem.is_empty() && !stem.ends_with('-') {
            stem.push('-');
        }
        if stem.len() >= 80 {
            break;
        }
    }
    let stem = stem.trim_end_matches('-');
    if stem.is_empty() {
        "gooblelab".into()
    } else {
        format!("gooblelab-{stem}")
    }
}

fn python_string(value: &str) -> Result<String, String> {
    serde_json::to_string(value).map_err(|e| e.to_string())
}

fn set_parameter(notebook: &mut Value, name: &str, value: &str) -> Result<(), String> {
    let prefix = format!("{name} = ");
    let mut replaced = 0;
    for cell in notebook["cells"]
        .as_array_mut()
        .ok_or("Notebook cells missing.")?
    {
        if cell["cell_type"] != "code" {
            continue;
        }
        for line in cell["source"]
            .as_array_mut()
            .ok_or("Notebook source missing.")?
        {
            let source = line.as_str().ok_or("Invalid notebook source.")?;
            if source.starts_with(&prefix) {
                let annotation = source
                    .find("  # @param")
                    .map(|i| &source[i..])
                    .unwrap_or("\n");
                *line = Value::String(format!("{prefix}{value}{annotation}"));
                replaced += 1;
            }
        }
    }
    if replaced != 1 {
        return Err(format!(
            "Notebook template must contain one {name} assignment; found {replaced}."
        ));
    }
    Ok(())
}

pub fn build_notebook(
    config: &NotebookConfig,
    image_drive_path: Option<&str>,
) -> Result<Vec<u8>, String> {
    validate(config)?;
    if image_drive_path.is_some_and(|path| !drive_path(path)) {
        return Err("Uploaded image path must be inside /content/drive/MyDrive/.".into());
    }
    let mut notebook: Value = serde_json::from_str(NOTEBOOK).map_err(|e| e.to_string())?;
    let has_image = config.template != Template::TextToVideo;
    for (name, value) in [
        ("PRESET", python_string(&config.preset_id)?),
        ("CUSTOM_LORA", python_string(config.lora_source.trim())?),
        ("PROMPT", python_string(&config.prompt)?),
        ("WIDTH", config.width.to_string()),
        ("HEIGHT", config.height.to_string()),
        ("SECONDS", config.seconds.to_string()),
        ("SEED", config.seed.to_string()),
        ("CUSTOM_LORA_STRENGTH", config.lora_strength.to_string()),
        (
            "INPUT_IMAGE",
            python_string(if has_image {
                image_drive_path.unwrap_or("")
            } else {
                ""
            })?,
        ),
    ] {
        set_parameter(&mut notebook, name, &value)?;
    }
    let cells = notebook["cells"]
        .as_array_mut()
        .ok_or("Notebook cells missing.")?;
    let first = cells.first_mut().ok_or("Notebook title missing.")?;
    let original = first["source"]
        .as_array()
        .ok_or("Notebook title source missing.")?;
    let mut title = vec![Value::String(format!(
        "# GoobleLab — {}\n",
        config.name.trim()
    ))];
    title.extend(original.iter().skip(1).cloned());
    first["source"] = Value::Array(title);
    if has_image && image_drive_path.is_none() {
        let fallback = "# Upload the image selected in GoobleLab; it stays in this Colab runtime.\nif not INPUT_IMAGE.strip():\n    from google.colab import files\n    uploaded = files.upload()\n    if len(uploaded) != 1:\n        raise ValueError('Upload exactly one PNG, JPEG, or WebP image.')\n    INPUT_IMAGE = str(Path(next(iter(uploaded))).resolve())\n\n";
        let mut inserted = 0;
        for cell in cells.iter_mut().filter(|cell| cell["cell_type"] == "code") {
            let source = cell["source"]
                .as_array_mut()
                .ok_or("Notebook source missing.")?;
            if let Some(index) = source.iter().position(|line| {
                line.as_str()
                    .is_some_and(|line| line.starts_with("last_video = runner.generate("))
            }) {
                source.insert(index, Value::String(fallback.into()));
                inserted += 1;
            }
        }
        if inserted != 1 {
            return Err("Notebook template must contain one generation call.".into());
        }
    }
    for cell in cells.iter_mut().filter(|cell| cell["cell_type"] == "code") {
        cell["execution_count"] = Value::Null;
        cell["outputs"] = json!([]);
    }
    notebook["metadata"]["colab"]["name"] = json!(format!("{}.ipynb", file_stem(&config.name)));
    notebook["metadata"]["gooblelab"] = json!({
        "version": env!("CARGO_PKG_VERSION"), "project": config.name.trim(),
        "template": config.template, "preset": config.preset_id,
    });
    serde_json::to_vec_pretty(&notebook).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn code(notebook: &Value) -> String {
        notebook["cells"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|cell| cell["cell_type"] == "code")
            .flat_map(|cell| cell["source"].as_array().unwrap())
            .map(|line| line.as_str().unwrap())
            .collect()
    }

    #[test]
    fn export_is_clean_and_prompt_is_quoted() {
        let config = NotebookConfig {
            prompt: "竹林 \"quoted\" \\ and newline\n'); import os; #".into(),
            ..Default::default()
        };
        let notebook: Value =
            serde_json::from_slice(&build_notebook(&config, None).unwrap()).unwrap();
        let cells = notebook["cells"].as_array().unwrap();
        assert_eq!(cells.iter().filter(|c| c["cell_type"] == "code").count(), 6);
        assert!(
            cells
                .iter()
                .filter(|c| c["cell_type"] == "code")
                .all(|c| c["execution_count"].is_null() && c["outputs"] == json!([]))
        );
        let source = code(&notebook);
        let prompt = source
            .lines()
            .find(|line| line.starts_with("PROMPT = "))
            .unwrap();
        let value = prompt
            .strip_prefix("PROMPT = ")
            .unwrap()
            .split("  # @param")
            .next()
            .unwrap();
        assert_eq!(
            serde_json::from_str::<String>(value).unwrap(),
            config.prompt
        );
        assert!(!source.contains("files.upload()"));
        assert_eq!(file_stem("../../ Cool: Video?"), "gooblelab-cool-video");
        assert_eq!(file_stem("竹林"), "gooblelab");
    }

    #[test]
    fn image_export_uses_drive_or_upload_without_local_path() {
        let path = std::env::temp_dir().join(format!("gooblelab-test-{}.png", std::process::id()));
        std::fs::write(&path, b"test-image").unwrap();
        let config = NotebookConfig {
            template: Template::ImageToVideo,
            image_path: Some(path.clone()),
            ..Default::default()
        };
        let upload = String::from_utf8(build_notebook(&config, None).unwrap()).unwrap();
        assert!(upload.contains("files.upload()"));
        assert!(!upload.contains(&path.to_string_lossy().replace('\\', "\\\\")));
        let drive = String::from_utf8(
            build_notebook(&config, Some("/content/drive/MyDrive/GoobleLab/image.png")).unwrap(),
        )
        .unwrap();
        assert!(drive.contains("/content/drive/MyDrive/GoobleLab/image.png"));
        assert!(!drive.contains("files.upload()"));
        assert!(!drive.contains("access_token"));
        assert!(build_notebook(&config, Some("C:\\private\\image.png")).is_err());
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn invalid_settings_and_unsafe_lora_sources_are_rejected() {
        let mut config = NotebookConfig::default();
        assert!(validate(&config).is_ok());
        config.width = 1343;
        assert!(validate(&config).is_err());
        config.width = 1344;
        config.seconds = f64::NAN;
        assert!(validate(&config).is_err());
        config.seconds = 5.0;
        config.preset_id = "ref2va-base".into();
        assert!(validate(&config).is_err());
        for source in [
            "C:\\models\\adapter.safetensors",
            "/content/drive/MyDrive/../adapter.safetensors",
            "https://huggingface.co/u/r/resolve/main/../a.safetensors",
            "https://huggingface.co/u/r/resolve/main/%2e%2e/a.safetensors",
            "https://huggingface.co/u/r/resolve/main/a.safetensors?token=secret",
            "https://huggingface.co/u/r/resolve/main/a.safetensors#secret",
            "https://huggingface.co@evil.example/u/r/resolve/main/a.safetensors",
        ] {
            assert!(validate_lora(source).is_err(), "accepted {source}");
        }
        for source in [
            "",
            "/content/drive/MyDrive/adapter.safetensors",
            "https://huggingface.co/user/repo/blob/main/sub/adapter.safetensors?download=true",
        ] {
            assert!(validate_lora(source).is_ok(), "rejected {source}");
        }
    }
}
