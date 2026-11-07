import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { authorize, connectGoogle, loadGoogle, uploadProject } from './drive.mjs';

const notebookTemplate = JSON.parse(await readFile(new URL('../MiniMax_H3_Colab.ipynb', import.meta.url), 'utf8'));
const catalog = JSON.parse(await readFile(new URL('../presets.json', import.meta.url), 'utf8'));
const config = {
  name: 'Private test', template: 'image', presetId: 'fl2va-turbo-8', prompt: 'A red panda.',
  width: 1344, height: 768, seconds: 5, seed: '12345', loraSource: '', imageName: 'photo.png',
};
const png = Uint8Array.from([137, 80, 78, 71, 13, 10, 26, 10, 0, 255, 128, 1, 13, 10]);
const args = { accessToken: 'test-memory-token', config, notebookTemplate, catalog };
const id = value => new Response(JSON.stringify({ id: value }), { headers: { 'Content-Type': 'application/json' } });
const originalFetch = globalThis.fetch;
function mockHttp(t, responses) {
  const calls = [];
  globalThis.fetch = async (url, options) => {
    calls.push({ url: String(url), options });
    const response = responses.shift();
    if (response instanceof Error) throw response;
    return response;
  };
  t.after(() => { globalThis.fetch = originalFetch; });
  return calls;
}
async function multipart(call) {
  const boundary = call.options.headers['Content-Type'].split('boundary=')[1];
  const parts = Buffer.from(await call.options.body.arrayBuffer()).toString('latin1').split(`--${boundary}`).slice(1, -1);
  const content = part => part.slice(part.indexOf('\r\n\r\n') + 4, -2);
  return { metadata: JSON.parse(content(parts[0])), bytes: Buffer.from(content(parts[1]), 'latin1'), headers: parts[1] };
}

test('private folder and multipart uploads preserve image bytes and write its safe Drive path into notebook', async t => {
  const calls = mockHttp(t, [id('folder_123'), id('image_123'), id('notebook_123')]);
  const imageFile = new File([png], 'personal-name.png', { type: 'text/plain' });
  const result = await uploadProject({ ...args, imageFile });
  assert.deepEqual(result, {
    colabUrl: 'https://colab.research.google.com/drive/notebook_123',
    driveUrl: 'https://drive.google.com/drive/folders/folder_123',
  });
  assert.equal(calls.length, 3);
  for (const call of calls) {
    assert.equal(call.options.method, 'POST');
    assert.equal(call.options.redirect, 'error');
    assert.equal(call.options.credentials, 'omit');
    assert.equal(call.options.referrerPolicy, 'no-referrer');
    assert.equal(call.options.headers.Authorization, 'Bearer test-memory-token');
    assert.equal(new URL(call.url).origin, 'https://www.googleapis.com');
  }
  const folder = JSON.parse(calls[0].options.body);
  assert.match(folder.name, /^gooblelab-private-test-[0-9a-f-]{36}$/u);
  assert.equal(folder.mimeType, 'application/vnd.google-apps.folder');
  assert.deepEqual(folder.parents, ['root']);
  assert.ok(!('permissions' in folder));
  const image = await multipart(calls[1]);
  assert.deepEqual(image.bytes, Buffer.from(png));
  assert.equal(image.metadata.name, 'input.png');
  assert.equal(image.metadata.mimeType, 'image/png');
  assert.deepEqual(image.metadata.parents, ['folder_123']);
  const notebook = await multipart(calls[2]);
  assert.equal(notebook.metadata.mimeType, 'application/x-ipynb+json');
  const text = notebook.bytes.toString('utf8');
  assert.ok(text.includes(`/content/drive/MyDrive/${folder.name}/input.png`));
  assert.ok(!text.includes('personal-name'));
  assert.ok(!text.includes('test-memory-token'));
  assert.doesNotThrow(() => JSON.parse(text));
});

test('large images use resumable binary PUT only to the expected HTTPS Google upload endpoint', async t => {
  const location = 'https://www.googleapis.com/upload/drive/v3/files?uploadType=resumable&upload_id=example123';
  const calls = mockHttp(t, [id('folder_123'), new Response('', { headers: { Location: location } }), id('image_123'), id('notebook_123')]);
  const imageFile = new File([png, new Uint8Array(5 * 1024 * 1024)], 'photo.png');
  await uploadProject({ ...args, imageFile });
  assert.equal(calls.length, 4);
  assert.equal(new URL(calls[1].url).searchParams.get('uploadType'), 'resumable');
  assert.equal(calls[1].options.headers['X-Upload-Content-Length'], String(imageFile.size));
  assert.equal(calls[2].url, location);
  assert.equal(calls[2].options.method, 'PUT');
  assert.equal(calls[2].options.redirect, 'error');
  assert.equal(calls[2].options.body, imageFile);
  assert.deepEqual(Buffer.from(await calls[2].options.body.arrayBuffer()), Buffer.from(await imageFile.arrayBuffer()));
});

test('invalid files fail before network; text template ignores stale image; HTTP bodies and unsafe locations stay private', async t => {
  const calls = mockHttp(t, [id('folder_123'), id('notebook_123')]);
  for (const imageFile of [new File([png], 'photo.jpg'), new File([new Uint8Array(12)], 'photo.png'),
    new File([png, new Uint8Array(25 * 1024 * 1024)], 'photo.png'), null]) {
    await assert.rejects(uploadProject({ ...args, imageFile }), /image|extension/u);
  }
  assert.equal(calls.length, 0);
  await uploadProject({ ...args, config: { ...config, template: 'text' }, imageFile: new File(['bad'], 'stale.exe') });
  assert.equal(calls.length, 2);
  assert.ok(!(await multipart(calls[1])).bytes.toString('utf8').includes('stale.exe'));
  for (const location of ['http://www.googleapis.com/upload/drive/v3/files?uploadType=resumable&upload_id=x',
    'https://www.googleapis.com.evil.example/upload/drive/v3/files?uploadType=resumable&upload_id=x',
    'https://www.googleapis.com/other?uploadType=resumable&upload_id=x']) {
    const attempts = mockHttp(t, [id('folder_123'), new Response('', { headers: { Location: location } })]);
    await assert.rejects(uploadProject({ ...args, imageFile: new File([png, new Uint8Array(5 * 1024 * 1024)], 'photo.png') }), /invalid upload destination/u);
    assert.equal(attempts.length, 2);
  }
  mockHttp(t, [new Response('private-user@example.com secret-token', { status: 403 })]);
  await assert.rejects(uploadProject({ ...args, config: { ...config, template: 'text' } }), error => !/private-user|secret-token/u.test(error.message));
  mockHttp(t, [id('folder/unsafe')]);
  await assert.rejects(uploadProject({ ...args, config: { ...config, template: 'text' } }), /invalid file response/u);
});

test('GIS authorization invokes popup synchronously with narrow scope and returns only an in-memory token', async t => {
  const previous = globalThis.google;
  let requested = false;
  let options;
  globalThis.google = { accounts: { oauth2: { initTokenClient(value) {
    options = value;
    return { requestAccessToken(request) {
      requested = true;
      assert.deepEqual(request, { prompt: 'consent' });
      queueMicrotask(() => options.callback({ access_token: 'token', expires_in: 3600, scope: options.scope }));
    } };
  } } } };
  t.after(() => { globalThis.google = previous; });
  await loadGoogle();
  assert.equal(connectGoogle, authorize);
  const connected = authorize('123-example.apps.googleusercontent.com');
  assert.equal(requested, true);
  assert.equal(options.scope, 'https://www.googleapis.com/auth/drive.file');
  assert.equal(options.include_granted_scopes, false);
  const result = await connected;
  assert.equal(result.accessToken, 'token');
  assert.ok(result.expiresAt > Date.now() + 3590 * 1000);
  await assert.rejects(authorize('desktop-client.json'), /Web application client ID/u);
  globalThis.google.accounts.oauth2.initTokenClient = value => ({ requestAccessToken() {
    queueMicrotask(() => value.callback({ error: 'private-information' }));
  } });
  await assert.rejects(authorize('123-example.apps.googleusercontent.com'), error => !error.message.includes('private-information'));
});
