const MAX_SEED = 18446744073709551615n;
const LORA_ERROR = 'Use an HTTPS Hugging Face .safetensors file URL or /content/drive/MyDrive/... path. Windows paths cannot be read by Colab.';

function drivePath(path) {
  return typeof path === 'string' && path.startsWith('/content/drive/MyDrive/')
    && path.length > '/content/drive/MyDrive/'.length
    && !/[\\:\u0000-\u001f\u007f-\u009f]/u.test(path)
    && !path.split('/').some(part => part === '.' || part === '..');
}

function safetensorsFile(path) {
  return /\.safetensors$/i.test(path)
    && !/[\\:\0]/u.test(path)
    && !path.split('/').some(part => part === '.' || part === '..');
}

function validateLora(source) {
  if (!source) return;
  if (drivePath(source) && safetensorsFile(source)) return;
  if (!source.startsWith('https://huggingface.co/')) throw new Error(LORA_ERROR);

  let url;
  try {
    url = new URL(source);
  } catch {
    throw new Error(LORA_ERROR);
  }
  if (url.hostname !== 'huggingface.co' || url.username || url.password
      || source.includes('#') || !['', '?', '?download=true', '?download=1'].includes(url.search)) {
    throw new Error('Remove credentials and URL tokens; use the HF_TOKEN Colab secret for private LoRAs.');
  }

  // URL normalizes dot segments, so validate the original, decoded path too.
  let path;
  try {
    path = decodeURIComponent(source.slice('https://huggingface.co'.length).split(/[?#]/u)[0]);
  } catch {
    throw new Error(LORA_ERROR);
  }
  const parts = path.replace(/^\/+|\/+$/gu, '').split('/');
  const identifier = part => /^[A-Za-z0-9_.-]+$/u.test(part) && part !== '.' && part !== '..';
  if (parts.length < 5 || !['resolve', 'blob'].includes(parts[2])
      || ![parts[0], parts[1], parts[3]].every(identifier)
      || !safetensorsFile(parts.slice(4).join('/'))) {
    throw new Error(LORA_ERROR);
  }
}

/** Validate form values and return a normalized copy. Keep seeds as decimal strings. */
export function validate(config, catalog) {
  if (!config || typeof config.name !== 'string' || !config.name.trim()
      || /[\u0000-\u001f\u007f-\u009f]/u.test(config.name)) {
    throw new Error('Enter a project name without control characters.');
  }
  if (!['text', 'image', 'reference'].includes(config.template)) {
    throw new Error('Choose a supported notebook template.');
  }
  if (typeof config.prompt !== 'string' || !config.prompt.trim()) throw new Error('Enter a prompt.');
  if (!Number.isInteger(config.width) || !Number.isInteger(config.height)
      || config.width < 256 || config.width > 1344 || config.height < 256 || config.height > 1344
      || config.width % 32 || config.height % 32 || config.width * config.height > 1344 * 768) {
    throw new Error('Use dimensions divisible by 32, within 1344×768 total pixels.');
  }
  if (!Number.isFinite(config.seconds) || config.seconds < 4 || config.seconds > 15) {
    throw new Error('Choose a duration from 4 to 15 seconds.');
  }
  const loraStrength = config.loraStrength ?? 1;
  if (!Number.isFinite(loraStrength) || loraStrength < 0 || loraStrength > 2) {
    throw new Error('LoRA strength must be between 0 and 2.');
  }
  if (typeof config.seed !== 'string' || !/^\d+$/u.test(config.seed.trim())) {
    throw new Error('Use a whole-number seed from 0 to 18446744073709551615.');
  }
  const seed = config.seed.trim().replace(/^0+(?=\d)/u, '');
  if (seed.length > 20 || BigInt(seed) > MAX_SEED) {
    throw new Error('Use a whole-number seed from 0 to 18446744073709551615.');
  }
  const family = typeof config.presetId === 'string' && Object.hasOwn(catalog?.presets ?? {}, config.presetId)
    ? catalog.presets[config.presetId]?.family : undefined;
  if (!['fl2va', 'ref2va'].includes(family)) throw new Error('Choose a supported MiniMax H3 preset.');
  if ((config.template === 'reference') !== (family === 'ref2va')) {
    throw new Error('Reference to video requires a Ref2VA preset; text/image to video requires FL2VA.');
  }
  const imageName = config.imageName ?? '';
  if (config.template !== 'text' && (typeof imageName !== 'string' || !/\.(png|jpe?g|webp)$/iu.test(imageName))) {
    throw new Error('Choose a PNG, JPEG, or WebP input image.');
  }
  if (config.loraSource != null && typeof config.loraSource !== 'string') throw new Error(LORA_ERROR);
  const loraSource = (config.loraSource ?? '').trim();
  validateLora(loraSource);
  return { ...config, name: config.name.trim(), seed, imageName, loraSource, loraStrength };
}

export function fileStem(name) {
  let stem = '';
  for (const character of name) {
    if (/^[A-Za-z0-9]$/u.test(character)) stem += character.toLowerCase();
    else if (stem && !stem.endsWith('-')) stem += '-';
    if (stem.length >= 80) break;
  }
  stem = stem.replace(/-+$/u, '');
  return stem ? `gooblelab-${stem}` : 'gooblelab';
}

function pythonString(value) {
  return JSON.stringify(value).replace(/\u2028/gu, '\\u2028').replace(/\u2029/gu, '\\u2029');
}

function setParameter(cells, name, value) {
  const prefix = `${name} = `;
  let replaced = 0;
  for (const cell of cells.filter(cell => cell.cell_type === 'code')) {
    cell.source = cell.source.map(source => {
      if (!source.startsWith(prefix)) return source;
      const annotation = source.indexOf('  # @param');
      replaced += 1;
      return `${prefix}${value}${annotation < 0 ? '\n' : source.slice(annotation)}`;
    });
  }
  if (replaced !== 1) throw new Error(`Notebook template must contain one ${name} assignment; found ${replaced}.`);
}

/** Prepare an editable notebook without embedding the selected image or a local path. */
export function buildNotebook(config, notebookTemplate, catalog, imageDrivePath = null) {
  const settings = validate(config, catalog);
  if (imageDrivePath !== null && !drivePath(imageDrivePath)) {
    throw new Error('Uploaded image path must be inside /content/drive/MyDrive/.');
  }
  const notebook = JSON.parse(JSON.stringify(notebookTemplate));
  const cells = notebook.cells;
  if (!Array.isArray(cells) || !cells.length) throw new Error('Notebook cells missing.');
  if (cells.some(cell => !Array.isArray(cell.source) || cell.source.some(line => typeof line !== 'string'))) {
    throw new Error('Invalid notebook source.');
  }
  for (const [name, value] of Object.entries({
    PRESET: pythonString(settings.presetId),
    CUSTOM_LORA: pythonString(settings.loraSource),
    PROMPT: pythonString(settings.prompt),
    WIDTH: String(settings.width),
    HEIGHT: String(settings.height),
    SECONDS: String(settings.seconds),
    SEED: settings.seed,
    CUSTOM_LORA_STRENGTH: String(settings.loraStrength),
    INPUT_IMAGE: pythonString(settings.template === 'text' ? '' : imageDrivePath ?? ''),
  })) setParameter(cells, name, value);

  cells[0].source = [`# GoobleLab — ${settings.name}\n`, ...cells[0].source.slice(1)];
  if (settings.template !== 'text' && imageDrivePath === null) {
    const fallback = '# Upload the image selected in GoobleLab; it stays in this Colab runtime.\n'
      + 'if not INPUT_IMAGE.strip():\n'
      + '    from google.colab import files\n'
      + '    uploaded = files.upload()\n'
      + '    if len(uploaded) != 1:\n'
      + "        raise ValueError('Upload exactly one PNG, JPEG, or WebP image.')\n"
      + '    INPUT_IMAGE = str(Path(next(iter(uploaded))).resolve())\n\n';
    let inserted = 0;
    for (const cell of cells.filter(cell => cell.cell_type === 'code')) {
      const index = cell.source.findIndex(line => line.startsWith('last_video = runner.generate('));
      if (index >= 0) {
        cell.source.splice(index, 0, fallback);
        inserted += 1;
      }
    }
    if (inserted !== 1) throw new Error('Notebook template must contain one generation call.');
  }
  for (const cell of cells.filter(cell => cell.cell_type === 'code')) {
    cell.execution_count = null;
    cell.outputs = [];
  }
  notebook.metadata ??= {};
  notebook.metadata.colab ??= {};
  notebook.metadata.colab.name = `${fileStem(settings.name)}.ipynb`;
  notebook.metadata.gooblelab = {
    version: '0.2.0', project: settings.name, template: settings.template, preset: settings.presetId,
  };
  return notebook;
}
