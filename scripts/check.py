"""Core offline release checks; no installs, model downloads, credentials, or GPU."""
import ast
import json
from pathlib import Path
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]


def run(*command, capture=False):
    return subprocess.run(command, cwd=ROOT, check=True, text=True,
                          encoding="utf-8", stdout=subprocess.PIPE if capture else None)


def main():
    node = shutil.which("node")
    if not node:
        raise RuntimeError("Install Node.js 22 or newer, then rerun this command.")
    if int(run(node, "--version", capture=True).stdout.lstrip("v").split(".")[0]) < 22:
        raise RuntimeError("The offline checks require Node.js 22 or newer.")
    run(sys.executable, "-m", "unittest", "discover", "-s", "tests", "-v")
    for module in ("app.mjs", "notebook.mjs", "drive.mjs", "build.mjs"):
        run(node, "--check", f"web/{module}")
    run(node, "--test", "web/test-notebook.mjs", "web/test-drive.mjs")
    run(node, "web/build.mjs")

    # Exercise the built assets, then parse the exported Python with Python itself.
    exports = json.loads(run(node, "--input-type=module", "-e", r'''
import { readFileSync } from 'node:fs';
import { buildNotebook } from './web/dist/notebook.mjs';
const template = JSON.parse(readFileSync('web/dist/template.json', 'utf8'));
const catalog = JSON.parse(readFileSync('web/dist/presets.json', 'utf8'));
const results = [];
for (const [presetId, preset] of Object.entries(catalog.presets)) {
  for (const workflow of preset.family === 'ref2va' ? ['reference'] : ['text', 'image']) {
    const config = {name:'Release check', template:workflow, presetId,
      prompt:'A bird turns toward the camera. Audio: forest ambience.',
      width:1344, height:768, seconds:5, seed:'18446744073709551615',
      loraSource:'', loraStrength:1, imageName:workflow === 'text' ? '' : 'input.png'};
    results.push({presetId, workflow, notebook:buildNotebook(config, template, catalog)});
  }
}
process.stdout.write(JSON.stringify(results));
''', capture=True).stdout)
    for result in exports:
        cells = [c for c in result["notebook"]["cells"] if c["cell_type"] == "code"]
        if len(cells) != 6:
            raise RuntimeError(f"Unexpected cell count: {result['presetId']}")
        for index, cell in enumerate(cells, 1):
            ast.parse("".join(cell["source"]),
                      filename=f"{result['presetId']}/{result['workflow']}/step-{index}")
            if cell.get("execution_count") is not None or cell.get("outputs"):
                raise RuntimeError("Export contains execution state or outputs.")
    for font in ("InstrumentSerif-Regular.ttf", "Manrope-Variable.ttf",
                 "InstrumentSerif-OFL.txt", "Manrope-OFL.txt"):
        if (ROOT / "web/fonts" / font).read_bytes() != (ROOT / "web/dist/fonts" / font).read_bytes():
            raise RuntimeError(f"Built font asset differs: {font}")
    print(f"PASS: core tests, browser build, and {len(exports)} clean notebook exports.")
    print("Offline checks do not establish live OAuth, GPU success, or paid-launch readiness.")


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, RuntimeError, subprocess.CalledProcessError) as exc:
        print(f"CHECK FAILED: {exc}", file=sys.stderr)
        sys.exit(1)
