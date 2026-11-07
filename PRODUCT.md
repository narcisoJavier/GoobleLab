# GoobleLab

<!-- impeccable:product-schema 1 -->

## Platform

Windows desktop companion and Google Colab notebooks. Windows is the first supported desktop platform; the notebooks remain the primary product.

## Stack

Rust desktop application, as requested. Native egui/eframe is the implementation choice for a small Rust-only GUI. Python and native ComfyUI nodes continue to execute the notebook workflows on Colab.

## Users

People who want to generate media with open-weight MiniMax H3 using a Colab GPU, without manually editing or uploading a raw notebook.

## Product Purpose

The notebook is the core deliverable. An optional guided companion chooses a template, configures generation, builds the notebook, uploads it and the selected image to Drive, and opens Colab. Users select their Colab runtime and run the notebook there.

## Operating Context

The existing repository has a working H3 FL2VA eight-step notebook and five curated presets. The user selected template → settings → export/upload as the desktop flow and automatic upload of the selected image alongside the notebook. Google Drive upload needs a user-supplied Desktop OAuth client configuration; the first version includes setup instructions and local export.

## Capabilities and Constraints

- First catalog: H3 text-to-video, image-to-video and reference-image-to-video, using the verified native ComfyUI graphs.
- Other model families and arbitrary ComfyUI graph imports are future work.
- The desktop application configures notebooks; inference runs on Colab.
- No Google tokens or local Windows image paths enter exported notebooks.
- Google login uses the system browser, PKCE, a localhost callback and the drive.file scope. Tokens stay in memory in this version.
- Colab now documents a beta runtime API; direct runtime management is outside this first companion's scope.
- Runtime availability, model terms and GPU memory constraints still apply.

## Brand Commitments

The application and repository are named GoobleLab, spelled G-O-O-B-L-E-L-A-B. The notebooks remain central in the README and product explanation; the GUI is an optional feature.

## Evidence on Hand

The user generated an H3 clip with the original Colab notebook. The refactored runner has 18 passing offline checks and the desktop libraries have 11 passing tests. Native UI captures cover both supported window sizes; local exports produce valid Python cells. Live Google authorization, uploads and new GPU paths remain unverified. No production OAuth credentials are supplied.

## Product Principles

1. Preserve a usable notebook without the desktop app.
2. Guide configuration with supported templates and meaningful defaults.
3. Keep media and credentials private until the user chooses export/upload.
4. Make the transition into Colab explicit and recoverable.
