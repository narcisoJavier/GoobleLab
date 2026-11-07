# GoobleLab — MiniMax H3 Google Colab notebooks

[![Open in Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/narcisoJavier/GoobleLab/blob/main/MiniMax_H3_Colab.ipynb)

Generate AI videos with sound in **Google Colab** using open-weight **MiniMax H3** and ComfyUI. GoobleLab supports text-to-video, image-to-video from a first frame, and reference-image-to-video, with five model presets and optional compatible LoRAs. Save the MP4 to Google Drive or the Colab runtime.

**The notebook is the core product.** Open it above to get started; configuration stays in notebook forms and ComfyUI runs inside the Colab runtime. The optional [Windows desktop companion](#optional-windows-desktop-companion) prepares a configured notebook for you.

## Quick start

1. Open the notebook above and save a copy to your Drive.
2. Choose **Runtime → Change runtime type → A100 GPU** with **high RAM**. The notebook checks for at least **38,000 MiB GPU memory** and **48 GiB system RAM**. GPU availability and paid compute depend on your Colab account.
3. Choose your preset before setup, then run Steps 1–4 in order. Allow **50 GB of free storage** for one model set and headroom. A Turbo set downloads approximately **44.4 GB**. Keeping both model families adds a **20.96 GB diffusion checkpoint**, plus any additional Turbo adapters; the text encoder and VAEs are shared.
4. In Step 5, edit the prompt and settings, then run the cell. The default is **1344×768**, approximately **5.17 seconds**, at 24 fps. The cell shows the saved path and a video preview.
5. Generate more clips with Step 5. When finished, enable `STOP_BACKEND` and run Step 6, then use **Runtime → Disconnect and delete runtime** to release the Colab GPU. Step 6 leaves the backend running while its switch is off.

After changing the preset or LoRA source in Step 1, rerun Step 1 and then Steps 3 and 4 before generating. With Drive enabled, the model cache is **My Drive → ComfyUI_Models** and MP4s appear under **My Drive → ComfyUI_Outputs → MiniMaxH3_[timestamp]**. Without Drive, files live under `/content/H3_models` and `/content/H3_outputs` and are temporary.

## Optional Windows desktop companion

[Download GoobleLab v0.2.0 for Windows x64](https://github.com/narcisoJavier/GoobleLab/releases/download/v0.2.0/GoobleLab-v0.2.0-windows-x64.zip) · [Release notes](https://github.com/narcisoJavier/GoobleLab/releases/tag/v0.2.0) · [Desktop guide](docs/DESKTOP.md)

Extract the portable ZIP and run `gooblelab.exe`. The Rust app guides you through **Template → Settings → Export & upload**:

1. Choose one of three curated H3 templates: text to video, image to video, or reference to video.
2. Set the matching preset, prompt, resolution, duration, seed and optional custom LoRA. Image templates require a PNG, JPEG or WebP input.
3. **Save notebook…** exports an editable `.ipynb` without Google credentials. Upload it in Colab; image templates ask you to upload the image in Step 5, without embedding a Windows path.
4. Alternatively, follow the [Google OAuth setup guide](docs/GOOGLE_OAUTH.md), select your own **Desktop OAuth client JSON**, and sign in through your system browser. **Upload to Drive & open Colab** creates a fresh private project folder containing the notebook and selected image, then opens Colab. The app requests only `drive.file`; tokens stay in memory for the current app session.

You still choose the GPU and run the cells in Colab. The companion needs no local GPU and does not accelerate GPU generation or guarantee Colab resources. It supports these curated templates; arbitrary ComfyUI workflows and other model families are not imported.

## MiniMax H3 presets

| Preset ID | Steps | Sampler | Video/audio sigma shift | Image |
| --- | ---: | --- | --- | --- |
| `fl2va-turbo-8` (default) | 8 | res_multistep | 12 / 3 | Optional first frame |
| `fl2va-base` | 20 | res_multistep | 12 / 3 | Optional first frame |
| `fl2va-turbo-4` | 4 | euler | 6 / 3 | Optional first frame |
| `ref2va-base` | 20 | res_multistep | 12 / 3 | Required single reference |
| `ref2va-turbo-4` (experimental) | 4 | euler | 12 / 3 | Required single reference |

FL2VA is the starting choice for text-to-video generation. For image input in the direct notebook, set `INPUT_IMAGE` to a PNG, JPEG, or WebP path accessible inside Colab, such as `/content/drive/MyDrive/reference.png`. A Windows path cannot be read by Colab. Ref2VA requires this image; refer to it as **`<Picture 1>`** in the prompt. Turbo presets include the matching acceleration LoRA. The experimental Ref2VA 4-step adapter was trained at 544p; try **960×544** for that preset. Results and runtime depend on the prompt, settings, and allocated GPU.

The notebook uses FP8-pruned diffusion weights, an NVFP4 text encoder, and an FP16 video VAE. It targets an A100 Colab runtime and does not promise free-tier compatibility. **1344×768 is the default native landscape preset**, not an upscaler.

## Use a custom LoRA

In Step 1, set the optional `CUSTOM_LORA` field to either:

- A Hugging Face file URL: `https://huggingface.co/OWNER/REPO/resolve/REVISION/FILE.safetensors`.
- An absolute local path, such as `/content/drive/MyDrive/my_h3_lora.safetensors`.

After editing the source, rerun Step 1 and then Steps 3 and 4. Set `CUSTOM_LORA_STRENGTH` in Step 5. Leave the source blank to use only the chosen preset's built-in adapter. Public downloads need no token. For a private or gated custom adapter, add `HF_TOKEN` in **Colab Secrets** and grant notebook access; never put tokens in URLs or notebook fields. Hugging Face branch revisions are resolved to a commit for the current download.

Choose a **ComfyUI-compatible model-only LoRA trained for the selected H3 family**. FL2VA and Ref2VA adapters are not interchangeable; LoRAs for other architectures such as Wan, Flux, or Stable Diffusion do not work here. Custom LoRA loading applies an existing adapter; this notebook does not train one. Check the adapter's model card for its recommended strength, trigger words, and license.

## Output and recovery

- **Video saved, preview failed:** the MP4 may already be complete. Use the printed path. In a new cell, run `display(Video(str(last_video), embed=True))` after importing `Video` and `display` from `IPython.display`.
- **Connection refused / backend stopped:** rerun Step 4, wait for readiness, then run Step 5. Step 6 stops the backend when `STOP_BACKEND` is enabled.
- **Out of memory:** reduce resolution or duration, restart the backend, and retry. Maximum resolution is more demanding than the smaller test settings.
- **Runtime disconnected:** reconnect and rerun setup/startup. Cached Drive files and saved Drive outputs remain; files stored only under `/content` can be lost.

## Project status

The original FL2VA 8-step notebook generated a video at 864×480 in the author's Colab session. **That clip is the only GPU-validated path so far.** The refactored runner, other presets, custom-LoRA loading, image/reference paths and 1344×768 default have offline checks but still need GPU validation. The desktop exporter and OAuth helpers also have offline checks; live Google sign-in and Drive upload have not yet been tested. Local checks do not establish generation quality, GPU compatibility or end-to-end Google integration.

Please include the selected preset, GPU, settings and relevant log excerpt when reporting an [issue](https://github.com/narcisoJavier/GoobleLab/issues). Remove tokens and private paths before posting logs.

## Development

[`MiniMax_H3_Colab.ipynb`](MiniMax_H3_Colab.ipynb) provides the forms, [`runner.py`](runner.py) handles setup and generation, [`presets.json`](presets.json) lists the model presets, and [`desktop/`](desktop/) contains the optional Rust companion. The v0.2.0 notebook loads its runner from the matching release tag. Support for other open-weight video models is a future goal after the MiniMax H3 workflow is stable.

Run the offline checks with `python -m unittest discover -s tests -v`. They do not install packages, download model weights, or execute GPU inference.

To build and run the desktop app on Windows, install Rust **1.95 or newer** with the Windows MSVC build tools, then run from the repository root:

```powershell
cargo run --release --manifest-path desktop/Cargo.toml
```

Run its offline checks with `cargo test --manifest-path desktop/Cargo.toml`. See the [desktop guide](docs/DESKTOP.md) for export and build details.

## Licenses and credits

Repository code is [MIT licensed](LICENSE). Downloaded model weights, LoRAs, ComfyUI, and other dependencies retain their own upstream licenses and conditions; the MIT license does not relicense them. Review the model and adapter licenses before commercial use or redistribution.

Built on [ComfyUI](https://github.com/Comfy-Org/ComfyUI), its [MiniMax H3 guide](https://docs.comfy.org/tutorials/video/minimax/minimax-h3), and the [Turbo LoRA publisher's settings](https://github.com/ModelTC/Minimax-H3-Turbo/blob/main/COMFYUI_SETUP_AND_INFERENCE.md). See the [Colab FAQ](https://research.google.com/colaboratory/faq.html) for current resource limits. This is a community notebook, unaffiliated with MiniMax, Google, or ComfyUI.
