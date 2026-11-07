# GoobleLab Windows companion

The optional Rust desktop app builds configured MiniMax H3 notebooks. The [Colab notebook](https://colab.research.google.com/github/narcisoJavier/GoobleLab/blob/main/MiniMax_H3_Colab.ipynb) remains the core product and can be used directly.

## Download and run

Download [GoobleLab-v0.2.0-windows-x64.zip](https://github.com/narcisoJavier/GoobleLab/releases/download/v0.2.0/GoobleLab-v0.2.0-windows-x64.zip) from the [v0.2.0 release](https://github.com/narcisoJavier/GoobleLab/releases/tag/v0.2.0), extract it, and run `gooblelab.exe`. The portable app prepares notebooks without a local GPU or Python installation. Generation runs on your Colab GPU.

## Template → Settings → Export & upload

1. **Template:** choose text to video, image to video using a first frame, or reference to video. Text/image templates offer the three FL2VA presets; reference templates offer the two Ref2VA presets. The external ComfyUI gallery link is for browsing; the app does not import arbitrary workflows.
2. **Settings:** enter a project name and prompt, select a resolution and duration, and set a seed. Requested duration is adjusted to H3's supported frame count at 24 fps; the plan shows the resulting duration. Reference prompts should refer to the image as `<Picture 1>`. Select a PNG, JPEG or WebP image for either image template; Drive uploads accept images up to 25 MiB. The experimental Ref2VA Turbo 4 preset was trained at 544p; try 960×544.
3. **Optional custom LoRA:** use a compatible H3 model-only `.safetensors` file at a Hugging Face HTTPS URL or `/content/drive/MyDrive/...` path, and choose its strength. The file must match the selected FL2VA or Ref2VA family. Turbo acceleration adapters are already included. This loads an adapter; it does not train one. Put private Hugging Face credentials in the `HF_TOKEN` Colab secret.
4. **Export & upload:** review the plan and choose a local export or Google Drive upload below. Settings remain editable in the generated notebook.

## Save a notebook without Google credentials

Click **Save notebook…** and save the `.ipynb`, then open [Colab](https://colab.research.google.com/) and choose **File → Upload notebook**. No Google OAuth client or desktop sign-in is needed for this export; using Colab and mounting Drive use your Google account separately.

For an image template, the export does not bundle the selected image or copy its Windows path into the notebook. When you run Step 5, Colab asks you to upload exactly one image. Upload the image you selected in GoobleLab. That copy lives in the temporary Colab runtime; upload it again after starting a fresh runtime, or set `INPUT_IMAGE` to an existing mounted Drive path.

## Upload the notebook and image to Drive

Follow the [Google OAuth setup guide](GOOGLE_OAUTH.md) to create your own Desktop OAuth client with Drive API enabled. Select the downloaded client JSON, click **Sign in with Google**, and complete consent in your normal system browser. The app requests only `drive.file` and keeps access and refresh tokens in memory. Closing the app or choosing **Forget session** removes its session.

Click **Upload to Drive & open Colab**. Each upload creates a fresh private folder in My Drive and uploads the notebook plus the selected image. Existing project files are not overwritten. The image receives a stable `reference.png`, `reference.jpg` or `reference.webp` filename, and the notebook points to its mounted Drive path. Keep that folder and filename in place and mount Drive with the same Google account in Colab. A failed upload can leave its folder or image behind; see the OAuth guide for recovery details.

The app opens the uploaded notebook in Colab and shows links to it and the project folder. It does not execute cells. Select **A100 GPU + high RAM**, then run Steps 1–5. The notebook requires at least 38,000 MiB GPU memory and 48 GiB system RAM, with roughly 50 GB storage for one model set and headroom. GPU availability and compute costs depend on your Colab account. The desktop app does not change generation speed.

When finished, enable `STOP_BACKEND` and run Step 6, then choose **Runtime → Disconnect and delete runtime** to release the GPU. Download runtime-only outputs before deleting the runtime.

## Build from source

On Windows, install Rust **1.95 or newer** with the MSVC toolchain and Visual Studio C++ build tools. From the repository root:

```powershell
cargo run --release --manifest-path desktop/Cargo.toml
```

The executable is written to `desktop/target/release/gooblelab.exe`. Run the desktop offline checks with:

```powershell
cargo test --manifest-path desktop/Cargo.toml
```

For a default notebook export without opening the GUI:

```powershell
cargo run --release --manifest-path desktop/Cargo.toml -- --export-default gooblelab-default.ipynb
```

The app embeds the notebook and preset catalog at build time, so rebuild it after changing either source file.

## Validation status

The original FL2VA 8-step Colab notebook produced one GPU-tested clip at 864×480. The refactored runner, remaining presets, custom LoRAs, image/reference generation and native 1344×768 default still need GPU validation. Desktop export and OAuth/upload helpers have offline tests; live Google authorization and upload have not yet been tested. See the [main README](../README.md) for preset details, recovery steps and licenses.
