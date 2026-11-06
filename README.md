# MiniMax H3 Google Colab — AI Video Generation & LoRA

[![Open in Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/narcisoJavier/minimax-h3-colab/blob/main/MiniMax_H3_Colab.ipynb)

Generate AI videos with sound in **Google Colab** using open-weight **MiniMax H3** and ComfyUI. Choose a model preset, write a prompt, optionally load a compatible LoRA, and save the MP4 to Google Drive. Configuration stays in notebook forms; ComfyUI runs locally inside the runtime.

## Quick start

1. Open the notebook above and save a copy to your Drive.
2. Choose **Runtime → Change runtime type → A100 GPU**, with high RAM recommended. GPU availability and paid compute depend on your Colab account.
3. Choose your preset before setup, then run Steps 1–4 in order. Allow **50 GB of free storage** for one model set and headroom. A Turbo set downloads approximately **44.4 GB**. Keeping both model families adds a **20.96 GB diffusion checkpoint**, plus any additional Turbo adapters; the text encoder and VAEs are shared.
4. In Step 5, edit the prompt and settings, then run the cell. The default is **1344×768**, approximately **5.17 seconds**, at 24 fps. The cell shows the saved path and a video preview.
5. Generate more clips with Step 5. Run Step 6 only when finished, then use **Runtime → Disconnect and delete runtime** to release the Colab GPU.

Changing the preset or LoRA source requires rerunning Steps 3 and 4. With Drive enabled, the model cache is **My Drive → ComfyUI_Models** and MP4s appear under **My Drive → ComfyUI_Outputs → MiniMaxH3_[timestamp]**. Without Drive, files live under `/content/H3_models` and `/content/H3_outputs` and are temporary.

## MiniMax H3 presets

| Preset ID | Steps | Sampler | Video/audio sigma shift | Image |
| --- | ---: | --- | --- | --- |
| `fl2va-turbo-8` (default) | 8 | res_multistep | 12 / 3 | Optional first frame |
| `fl2va-base` | 20 | res_multistep | 12 / 3 | Optional first frame |
| `fl2va-turbo-4` | 4 | euler | 6 / 3 | Optional first frame |
| `ref2va-base` | 20 | res_multistep | 12 / 3 | Required single reference |
| `ref2va-turbo-4` (experimental) | 4 | euler | 12 / 3 | Required single reference |

FL2VA is the starting choice for text-to-video generation. For image input, set `INPUT_IMAGE` to a local PNG, JPEG, or WebP path in Step 5. Ref2VA requires this image; refer to it as **`<Picture 1>`** in the prompt. Turbo presets include the matching acceleration LoRA. The experimental Ref2VA 4-step adapter was trained at 544p; try **960×544** for that preset. Results and runtime depend on the prompt, settings, and allocated GPU.

The notebook uses FP8-pruned diffusion weights, an NVFP4 text encoder, and an FP16 video VAE. It targets an A100 Colab runtime and does not promise free-tier compatibility. **1344×768 is the default native landscape preset**, not an upscaler.

## Use a custom LoRA

In Step 1, set the optional `CUSTOM_LORA` field to either:

- A Hugging Face file URL: `https://huggingface.co/OWNER/REPO/resolve/REVISION/FILE.safetensors`.
- An absolute local path, such as `/content/drive/MyDrive/my_h3_lora.safetensors`.

Run Steps 3 and 4 after changing the source. Set `CUSTOM_LORA_STRENGTH` in Step 5. Leave the source blank to use only the chosen preset's built-in adapter. Public downloads need no token. For a private or gated custom adapter, add `HF_TOKEN` in **Colab Secrets** and grant notebook access; never put tokens in URLs or notebook fields. Hugging Face branch revisions are resolved to a commit for the current download.

Choose a **ComfyUI-compatible model-only LoRA trained for the selected H3 family**. FL2VA and Ref2VA adapters are not interchangeable; LoRAs for other architectures such as Wan, Flux, or Stable Diffusion do not work here. Custom LoRA loading applies an existing adapter; this notebook does not train one. Check the adapter's model card for its recommended strength, trigger words, and license.

## Output and recovery

- **Video saved, preview failed:** the MP4 may already be complete. Use the printed path. In a new cell, run `display(Video(str(last_video), embed=True))` after importing `Video` and `display` from `IPython.display`.
- **Connection refused / backend stopped:** rerun Step 4, wait for readiness, then run Step 5. Step 6 stops the backend.
- **Out of memory:** reduce resolution or duration, restart the backend, and retry. Maximum resolution is more demanding than the smaller test settings.
- **Runtime disconnected:** reconnect and rerun setup/startup. Cached Drive files and saved Drive outputs remain; files stored only under `/content` can be lost.

## Project status

The earlier FL2VA 8-step notebook generated a video at 864×480 in the author's Colab session. **This repository's refactored runner, all preset paths, custom-LoRA loading, and 1344×768 default still need GPU validation.** Local code checks do not establish generation quality or GPU compatibility. Please include the selected preset, GPU, settings, and relevant log excerpt when reporting an issue. Remove tokens and private paths before posting logs.

The implementation is intentionally small: `MiniMax_H3_Colab.ipynb` provides the forms, `runner.py` handles setup and generation, and `presets.json` lists the model presets. Support for other open-weight video models is a future goal after the MiniMax H3 workflow is stable.

## Licenses and credits

Repository code is [MIT licensed](LICENSE). Downloaded model weights, LoRAs, ComfyUI, and other dependencies retain their own upstream licenses and conditions; the MIT license does not relicense them. Review the model and adapter licenses before commercial use or redistribution.

Built on [ComfyUI](https://github.com/Comfy-Org/ComfyUI) and its [MiniMax H3 guide](https://docs.comfy.org/tutorials/video/minimax/minimax-h3). See the [Colab FAQ](https://research.google.com/colaboratory/faq.html) for current resource limits. This is a community notebook, unaffiliated with MiniMax, Google, or ComfyUI.
