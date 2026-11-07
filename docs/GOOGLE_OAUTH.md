# Google Drive publishing setup

The GoobleLab desktop app can create a private project folder in your Google Drive, upload a configured notebook and its optional reference image, and open the notebook in Google Colab. Local notebook export works without Google sign-in.

## One-time Google Cloud setup

1. Open the [Google Cloud console](https://console.cloud.google.com/) and create or select a project.
2. In **APIs & Services → Library**, enable **Google Drive API**.
3. Open **Google Auth Platform** and configure the app name, support email, audience and contact email. Use your own accurate app details.
4. Under **Data Access**, add only `https://www.googleapis.com/auth/drive.file`. This is Google's recommended non-sensitive per-file scope. GoobleLab does not request access to all Drive files.
5. If the app audience is External and publishing status is Testing, add your Google account under **Audience → Test users**. Test-mode refresh tokens can expire; sign in again when needed.
6. Under **Clients → Create client**, choose **Desktop app**. Download the OAuth client JSON to a private local folder. Select that JSON file in GoobleLab; use the Desktop client download containing an `installed` object, rather than a Web application client or service-account key.

This repository does not include a production OAuth client. For personal use, use your own client. A distributor offering shared sign-in must configure and publish their own OAuth consent screen and meet Google's verification requirements. Selecting a client JSON file does not automatically publish or verify an OAuth app.

## Publish a configured notebook

1. Open GoobleLab. Select a template and MiniMax H3 preset, then configure the prompt, duration, resolution and optional compatible LoRA.
2. For image/reference-to-video, select a **PNG, JPG or WebP** reference image, up to **25 MiB**.
3. Select your downloaded Desktop OAuth JSON, then click the Google sign-in control. Complete consent in your normal system browser. The browser returns to a short-lived local callback on `127.0.0.1`; return to GoobleLab afterward. Sign-in times out after five minutes and can be cancelled in the app.
4. Use the Drive upload control. Every upload creates a new **GoobleLab - project - unique suffix** folder in **My Drive**, containing the notebook and optional `reference.png`, `reference.jpg` or `reference.webp`. Existing files are not overwritten, and the app does not make files public.
5. Open the returned **Colab** link. Choose the available GPU under **Runtime → Change runtime type**, then run the notebook's setup and generation cells. Google Drive mounting or Hugging Face access may ask for separate authorization inside Colab.
6. Keep the uploaded project folder and reference filename in place: the generated notebook points to that image's mounted Drive path. Outputs use the notebook's configured output folder. Disconnect and delete the Colab runtime when finished to release its GPU.

## Storage and authorization

- Authorization uses Google's fixed endpoints, a system-browser authorization-code flow, PKCE S256, a random state value and a loopback callback. OAuth URLs in a downloaded JSON file cannot override the app's Google endpoints.
- The app requests **`drive.file` only**. It can create its project folders and files; it does not search your existing Drive or overwrite notebooks from other applications.
- Access and refresh tokens live **only in app memory**. This first version does not save a login or write tokens to disk. Closing the app removes its session; sign in again next time. The user-selected client JSON remains where you downloaded it. Do not commit it to Git or paste it into notebooks or issue reports.
- Notebook uploads use Drive's `multipart/related` format. Reference images above 5 MiB use a resumable upload session; an interrupted upload currently needs another publish attempt. A failed attempt may leave its new folder or an uploaded image behind in Drive.
- LoRA URLs and prompts are configuration saved inside the notebook. Put Hugging Face credentials in **Colab Secrets → HF_TOKEN**, not in URLs or prompts.
- Uploading does not start GPU compute or execute cells. Premium GPUs and high-memory runtimes depend on your Colab subscription, compute-unit balance and current availability. This version uses manual Colab execution. Google's newer beta runtime API could support a separate future execution feature.

## Troubleshooting

- **Access blocked / unverified app:** check your OAuth audience and Test users; personal clients in Testing are intended for explicitly listed accounts. Public distribution needs the appropriate publishing and verification setup.
- **Drive HTTP 403:** check that Google Drive API is enabled in the project that owns your Desktop OAuth client, that the account is permitted to use the OAuth app, and that the account has available Drive storage.
- **Sign-in callback timeout:** allow the system browser to reach the local loopback callback; a firewall, proxy or closed consent tab can interrupt it. Cancel and sign in again.
- **Expired session / HTTP 401:** sign in again. Refreshing an access token is automatic while the app retains a valid in-memory refresh token.
- **Wrong Google account in Colab:** switch Colab to the account used for the upload before opening the notebook or mounting Drive.

The code has offline checks for PKCE, callback validation, upload URL validation, MIME selection and multipart formatting. Live Google authorization, upload and Colab GPU execution require user credentials and have not been validated by those offline checks.

Sources: [Desktop OAuth](https://developers.google.com/identity/protocols/oauth2/native-app), [Drive scopes](https://developers.google.com/workspace/drive/api/guides/api-specific-auth), [Drive uploads](https://developers.google.com/workspace/drive/api/guides/manage-uploads), [OAuth verification](https://support.google.com/cloud/answer/13464321), [Colab runtime API](https://developers.google.com/colab/api/reference/rest/v1beta/runtimes), [Colab resources](https://research.google.com/colaboratory/faq.html).
