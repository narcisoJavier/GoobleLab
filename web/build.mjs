import { mkdir, copyFile, cp } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
const root = new URL('./', import.meta.url);
await mkdir(new URL('dist/', root), { recursive: true });
for (const name of ['index.html', 'style.css', 'app.mjs', 'notebook.mjs', 'drive.mjs', 'favicon.svg']) {
  await copyFile(new URL(name, root), new URL(`dist/${name}`, root));
}
await cp(new URL('fonts/', root), new URL('dist/fonts/', root), { recursive: true });
for (const [source, target] of [['../MiniMax_H3_Colab.ipynb', 'template.json'], ['../presets.json', 'presets.json']]) {
  await copyFile(new URL(source, root), new URL(`dist/${target}`, root));
}
console.log(`Built browser app: ${fileURLToPath(new URL('dist/', root))}`);
