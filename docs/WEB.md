# GoobleLab browser builder

The browser is GoobleLab's primary interface. It prepares an editable MiniMax H3 notebook; generation runs in your own Google Colab account. The [core notebook](../MiniMax_H3_Colab.ipynb) also works directly. The [Windows v0.2.0 companion](DESKTOP.md) remains available as an optional legacy interface.

The [hosted preview](https://gooblelab-notebook-builder.helpfulaisle2.chatgpt.site) is owner-private and uses the hosting platform's ChatGPT sign-in gate. This does not implement GoobleLab subscription accounts. Local use below needs no hosting-platform sign-in.

## Run locally

From the repository root, with Node.js and Python installed:

```sh
node web/build.mjs
python -m http.server 8765 --bind 127.0.0.1 --directory web/dist
```

Open [the local builder](http://127.0.0.1:8765/). There are no npm dependencies. Rebuild after changing the browser files, notebook or preset catalog. Serve the generated directory over HTTP; opening `index.html` as a local file does not load its notebook resources reliably.

## Build and download a notebook

1. Choose **Template → Settings → Export / upload**. Free lets you edit the name and prompt for text-to-video using FL2VA Turbo 8, 1344×768, a requested 5 seconds and seed `12345`.
2. **Preview Pro** enables image/reference workflows, five H3 presets, resolution, 4–15 second duration, exact unsigned 64-bit seeds and compatible custom LoRA controls. This is an evaluation toggle: no account, subscription, payment or paid entitlement is active.
3. Click **Download .ipynb**. In Colab choose **File → Upload notebook**, select **A100 + high RAM**, and run Steps 1–5 in order. The plan shows the actual duration after H3 frame-count rounding; a requested 5 seconds produces about 5.17 seconds.
4. Finish with Step 6's `STOP_BACKEND`, then disconnect and delete the Colab runtime. See the [main guide](../README.md#quick-start) for compute, storage and output locations.

All exports retain the six editable code cells and Colab form annotations, with execution counts and outputs cleared. Settings remain editable in the notebook regardless of the browser plan. Downloading requires no Google connection in GoobleLab.

Image and reference workflows accept PNG, JPEG or WebP selections up to 25 MiB. A downloaded notebook includes neither the image nor a device path: Step 5 asks you to upload exactly one image into the Colab runtime. For Ref2VA, refer to it as `<Picture 1>` in your prompt. The experimental Ref2VA Turbo 4 preset was trained at 544p; try 960×544.

Custom LoRAs must match the selected H3 family. Use an HTTPS Hugging Face `.safetensors` file URL or `/content/drive/MyDrive/...` path, with strength from 0 to 2. Put private download credentials in Colab's `HF_TOKEN` secret. URLs with credentials, tokens or traversal are rejected.

## Saved projects and privacy

**Save project** keeps up to 20 configurations in this browser's local storage. Records contain names, prompts, settings and the selected image's filename. Image files, OAuth client IDs and Google access tokens are not saved with projects. Reselect an image before a Drive upload after reopening a project; downloaded notebooks can still ask for it inside Colab.

Projects do not sync across devices or browser profiles. Clearing site data removes them. Download a notebook to keep a portable copy. Google tokens stay in page memory and are discarded on disconnect or when the page closes. There are no GoobleLab accounts, cloud project storage or billing in this preview.

## Optional Google Drive upload preview

Live Google authorization and upload have **not yet been verified**. Download remains available without this setup.

1. Follow [Google's web OAuth setup](https://developers.google.com/identity/oauth2/web/guides/get-google-api-clientid): create a Google Cloud project, enable Drive API, configure consent and the `https://www.googleapis.com/auth/drive.file` scope, and create a **Web application** client.
2. Add the exact origin shown in the builder's Google setup panel to **Authorized JavaScript origins**, including the local port when testing. Register a hosted preview's origin separately. Configure test users if your OAuth consent configuration requires them.
3. Paste the public **Web OAuth client ID** in **Google Drive setup for this preview**, then choose **Prepare Google connection → Connect Google Drive**. Desktop OAuth JSON files and client secrets do not belong in this browser flow.
4. After consent, click **Upload to Drive & open Colab**. The app creates a fresh private My Drive folder, uploads the notebook and any selected image, and presents Colab and folder links. The notebook uses the mounted Drive image path. Uploading does not start a GPU runtime or share files.

Expired sessions require reconnecting. If an upload fails after creating files, a partial private folder may remain; check Drive before retrying. Choosing **Disconnect** clears the page session; manage any persistent OAuth grant through your Google account.

## Check the exporter

```sh
node --test web/test-notebook.mjs web/test-drive.mjs
```

Offline checks cover clean notebook export, safe prompt quoting, exact maximum seeds, preset/dimension/LoRA validation and mocked Drive requests. They do not establish live OAuth or GPU inference behavior. The commercial direction and launch requirements are in [SAAS.md](SAAS.md).
