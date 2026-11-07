# Contributing

GoobleLab is a notebook-builder preview, not a hosted generation service. Keep the editable notebook usable without the browser. Changes must preserve the distinction between offline checks and actual Colab results.

## Check a change

Use Python 3.10 or newer and Node.js 22 or newer from the repository root:

```sh
python scripts/check.py
```

This runs the Python and browser tests, builds the browser, and parses eight exported notebooks covering the five presets and their supported text/image/reference modes. It needs no third-party Python packages, OAuth credentials, GPU or model downloads. The optional Windows companion has separate checks in [the desktop guide](docs/DESKTOP.md).

For browser changes, also inspect Template, Settings, Export, Saved projects and Free/Pro on desktop and mobile. Exercise keyboard navigation, disabled controls and an invalid input. Follow [DESIGN.md](DESIGN.md); do not change the product's facts to fit a visual treatment.

## Runtime changes and evidence

The notebook currently loads the `v0.2.0` runtime. Editing `runner.py` on `main` alone does not update exported notebooks. Test the intended runtime on Colab, publish a new immutable release, and update the notebook bootstrap, browser core-notebook URL and exporter metadata together. Keep existing tags intact. Test both fresh setup and cached setup, and record the runtime versions and preset in the [validation matrix](docs/SAAS.md#validation-and-release-gates).

ComfyUI and model revisions are pinned, but Python dependencies are not fully locked. Capture the environment from a successful GPU run before introducing a tested constraint set; do not label the current setup fully reproducible.

## Reports and pull requests

Use the bug report form for ordinary failures. Include the commit/release, preset, runtime and minimal redacted error. Never attach access tokens, OAuth JSON, private media, a full private prompt or an unredacted notebook/log. Exported notebooks and `workflow_api.json` contain prompt text.

Describe the user-visible change, why it is needed, checks run, and any unverified Google/GPU behavior. For a vulnerability, use [private reporting](SECURITY.md), not a public issue.
