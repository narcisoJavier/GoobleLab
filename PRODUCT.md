# GoobleLab

<!-- impeccable:product-schema 1 -->

## Platform

Browser application and Google Colab notebooks. The browser is the primary interface; editable notebooks remain the core output and can be used directly. The Windows v0.2.0 companion is preserved as an optional legacy interface.

## Stack

Dependency-free HTML, CSS and browser JavaScript, with a small Node.js static build script. Google Identity Services supports the optional Web OAuth upload preview. Python and native ComfyUI nodes execute workflows on Colab. The existing Rust egui/eframe desktop source remains available.

## Users

People who want to generate media with open-weight MiniMax H3 using a Colab GPU, without manually editing or uploading a raw notebook.

## Product Purpose

Guide users through template → settings → export/upload in a browser. Free prepares a basic MiniMax H3 notebook; the Pro direction sells advanced customization, curated workflows and reusable profiles. Downloads stay editable and open source. Users select their own Colab runtime and run the cells there.

## Operating Context

The existing repository has a working H3 FL2VA eight-step notebook and five curated presets. The user chose a browser-first product, with Free basic notebook creation and a paid advanced customization direction. The current Pro toggle is an evaluation preview; there are no accounts, subscriptions, prices or payment enforcement. Optional browser Drive upload uses a user-supplied Web application client ID and authorized JavaScript origin. The legacy desktop flow uses its separate Desktop OAuth configuration.

## Capabilities and Constraints

- First catalog: H3 text-to-video, image-to-video and reference-image-to-video, using the verified native ComfyUI graphs.
- Other model families and arbitrary ComfyUI graph imports are future work.
- The browser and legacy desktop application configure notebooks; inference runs on the user's Colab account.
- No Google tokens or local Windows image paths enter exported notebooks.
- Browser projects retain up to 20 configurations on the device, including image filenames; image files and tokens are not persisted. There is no cloud sync.
- Browser Google login uses Google Identity Services and the drive.file scope. Tokens stay in page memory. Upload creates a private Drive folder with the notebook and selected image.
- Free uses text-to-video, FL2VA Turbo 8, 1344×768, requested 5 seconds and seed 12345. Pro preview adds image/reference workflows and advanced controls without charging.
- Runtime control, arbitrary workflow imports, accounts and billing are outside the current browser preview.
- Runtime availability, model terms and GPU memory constraints apply. Paid launch requires review of Colab terms and the MiniMax H3 license's territory and commercial provisions; see docs/SAAS.md.

## Brand Commitments

The application and repository are named GoobleLab, spelled G-O-O-B-L-E-L-A-B. The notebook remains central in the product explanation; the browser provides the primary guided interface. Keep the warm yellow visual direction and visible MiniMax H3 attribution.

## Evidence on Hand

The user generated an H3 clip with the original Colab notebook. Browser exporter checks cover safe prompt quoting, exact unsigned 64-bit seeds, clean six-cell notebooks and input validation; exported Python cells compile. Drive requests have offline mocked checks. Live browser Google authorization, uploads and new GPU paths remain unverified. No production OAuth credentials or billing are supplied. Prior desktop code and its release are preserved.

## Product Principles

1. Preserve a usable, editable notebook without the browser or desktop app.
2. Guide configuration with supported templates and meaningful defaults.
3. Keep media and credentials private until the user chooses export/upload.
4. Make the transition into Colab explicit and recoverable.
