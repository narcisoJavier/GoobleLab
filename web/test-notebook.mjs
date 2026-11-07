import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { buildNotebook, fileStem, validate } from './notebook.mjs';

const template = JSON.parse(await readFile(new URL('../MiniMax_H3_Colab.ipynb', import.meta.url), 'utf8'));
const catalog = JSON.parse(await readFile(new URL('../presets.json', import.meta.url), 'utf8'));
const config = {
  name: 'My MiniMax video', template: 'text', presetId: 'fl2va-turbo-8',
  prompt: 'A red panda in a misty bamboo forest.', width: 1344, height: 768,
  seconds: 5, seed: '12345', loraSource: '', loraStrength: 1, imageName: '',
};
const code = notebook => notebook.cells.filter(cell => cell.cell_type === 'code').flatMap(cell => cell.source).join('');

test('export quotes prompts, preserves exact u64 seeds and Colab forms, and clears execution state', () => {
  const prompt = '竹林 "quoted" \\ and newline\n\'); import os; #\u2028next';
  const dirtyTemplate = structuredClone(template);
  for (const cell of dirtyTemplate.cells.filter(cell => cell.cell_type === 'code')) {
    cell.execution_count = 7;
    cell.outputs = [{ output_type: 'stream', text: 'old output' }];
  }
  const result = buildNotebook({ ...config, prompt, seed: '18446744073709551615' }, dirtyTemplate, catalog);
  const codeCells = result.cells.filter(cell => cell.cell_type === 'code');
  assert.equal(codeCells.length, 6);
  assert.ok(codeCells.every(cell => cell.execution_count === null && cell.outputs.length === 0));
  const promptLine = code(result).split('\n').find(line => line.startsWith('PROMPT = '));
  assert.equal(JSON.parse(promptLine.slice('PROMPT = '.length).split('  # @param')[0]), prompt);
  assert.match(code(result), /^SEED = 18446744073709551615  # @param \{type:"integer"\}$/m);
  assert.doesNotMatch(code(result), /files\.upload\(\)/u);
  assert.equal(dirtyTemplate.cells[1].execution_count, 7);
  assert.equal(result.metadata.colab.name, 'gooblelab-my-minimax-video.ipynb');
  assert.equal(fileStem('../../ Cool: Video?'), 'gooblelab-cool-video');
  assert.equal(fileStem('竹林'), 'gooblelab');
  assert.deepEqual(result, buildNotebook({ ...config, prompt, seed: '18446744073709551615' }, dirtyTemplate, catalog));
  assert.equal(validate({ ...config, seed: '0000' }, catalog).seed, '0');
});

test('image and reference exports ask for one image in Colab without leaking local filenames', () => {
  for (const [templateName, presetId] of [['image', 'fl2va-turbo-8'], ['reference', 'ref2va-base']]) {
    const result = buildNotebook({ ...config, template: templateName, presetId, imageName: 'private-reference.webp' }, template, catalog);
    const source = code(result);
    assert.equal(source.match(/files\.upload\(\)/gu)?.length, 1);
    assert.match(source, /if len\(uploaded\) != 1:/u);
    assert.match(source, /^INPUT_IMAGE = ""  # @param/m);
    assert.ok(source.indexOf('files.upload()') < source.indexOf('last_video = runner.generate('));
    assert.ok(!JSON.stringify(result).includes('private-reference.webp'));
    const uploaded = code(buildNotebook(
      { ...config, template: templateName, presetId, imageName: 'private-reference.webp' },
      template, catalog, '/content/drive/MyDrive/GoobleLab/reference.webp',
    ));
    assert.match(uploaded, /^INPUT_IMAGE = "\/content\/drive\/MyDrive\/GoobleLab\/reference.webp"  # @param/m);
    assert.doesNotMatch(uploaded, /files\.upload\(\)/u);
  }
  for (const path of ['C:\\private\\image.png', '/content/drive/MyDrive/../image.png',
    '/content/drive/MyDrive/./image.png', '/content/drive/MyDrive/image\n.png']) {
    assert.throws(() => buildNotebook(config, template, catalog, path), /inside/u);
  }
});

test('validation rejects invalid settings, mismatched presets, and unsafe LoRA sources', () => {
  for (const invalid of [
    { name: 'bad\nname' }, { prompt: ' ' }, { template: 'other' },
    { width: 1343 }, { width: 1344, height: 1344 }, { height: 224 },
    { seconds: NaN }, { seconds: 3.9 }, { seconds: 15.1 },
    { seed: '-1' }, { seed: '1.5' }, { seed: '18446744073709551616' }, { seed: 12345 },
    { loraStrength: Infinity }, { loraStrength: 2.01 },
    { presetId: 'missing' }, { presetId: '__proto__' }, { presetId: ['fl2va-turbo-8'] }, { presetId: 'ref2va-base' },
    { template: 'reference', imageName: 'ref.png' },
    { template: 'image' }, { template: 'image', imageName: 'clip.mp4' },
  ]) assert.throws(() => validate({ ...config, ...invalid }, catalog), undefined, JSON.stringify(invalid));

  for (const loraSource of [
    'C:\\models\\adapter.safetensors', '/content/drive/MyDrive/../adapter.safetensors',
    'https://huggingface.co/u/r/resolve/main/../a.safetensors',
    'https://huggingface.co/u/r/resolve/main/%2e%2e/a.safetensors',
    'https://huggingface.co/u/r/resolve/main/a.safetensors?token=secret',
    'https://huggingface.co/u/r/resolve/main/a.safetensors#secret',
    'https://huggingface.co/u/r/resolve/main/a.safetensors#',
    'https://huggingface.co/u/r/resolve/main/a.safetensors?download=true&token=secret',
    'https://huggingface.co@evil.example/u/r/resolve/main/a.safetensors',
    'https://huggingface.co/u/r/resolve/main/%5c..%5ca.safetensors',
  ]) assert.throws(() => validate({ ...config, loraSource }, catalog), undefined, loraSource);

  for (const loraSource of [
    '', '/content/drive/MyDrive/adapter.safetensors',
    'https://huggingface.co/user/repo/blob/main/sub/adapter.safetensors?download=true',
    'https://huggingface.co/user/repo/resolve/main/adapter.safetensors?download=1',
  ]) assert.doesNotThrow(() => validate({ ...config, loraSource }, catalog), loraSource);
});

test('export rejects missing or duplicate parameter assignments', () => {
  for (const duplicate of [false, true]) {
    const changed = structuredClone(template);
    const cell = changed.cells.find(cell => cell.source.some(line => line.startsWith('SEED = ')));
    if (duplicate) cell.source.push('SEED = 1\n');
    else cell.source = cell.source.filter(line => !line.startsWith('SEED = '));
    assert.throws(() => buildNotebook(config, changed, catalog), /one SEED assignment/u);
  }
});
