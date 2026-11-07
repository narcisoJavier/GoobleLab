import { buildNotebook, fileStem, validate } from './notebook.mjs';

const SCOPE = 'https://www.googleapis.com/auth/drive.file';
const FILES = 'https://www.googleapis.com/drive/v3/files';
const UPLOAD = 'https://www.googleapis.com/upload/drive/v3/files';
let googleLoading;

/** Preload before enabling the sign-in button, so authorize() keeps its user gesture. */
export function loadGoogle() {
  if (globalThis.google?.accounts?.oauth2?.initTokenClient) return Promise.resolve();
  if (googleLoading) return googleLoading;
  googleLoading = new Promise((resolve, reject) => {
    const script = document.createElement('script');
    script.src = 'https://accounts.google.com/gsi/client';
    script.async = true;
    const finish = success => {
      clearTimeout(timer);
      script.onload = script.onerror = null;
      if (success && globalThis.google?.accounts?.oauth2?.initTokenClient) resolve();
      else {
        script.remove();
        googleLoading = undefined;
        reject(new Error('Google sign-in could not load. Check your connection and try again.'));
      }
    };
    const timer = setTimeout(() => finish(false), 15000);
    script.onload = () => finish(true);
    script.onerror = () => finish(false);
    document.head.append(script);
  });
  return googleLoading;
}

/** Call directly from a click, after loadGoogle(). Tokens belong only in session memory. */
export function authorize(clientId) {
  if (!globalThis.google?.accounts?.oauth2?.initTokenClient) {
    return Promise.reject(new Error('Load Google sign-in before connecting.'));
  }
  if (typeof clientId !== 'string' || !/^\d+-[A-Za-z0-9_-]+\.apps\.googleusercontent\.com$/u.test(clientId.trim())) {
    return Promise.reject(new Error('Enter a Google Web application client ID.'));
  }
  return new Promise((resolve, reject) => {
    const fail = () => reject(new Error('Google sign-in was cancelled or blocked. Try connecting again.'));
    try {
      const client = google.accounts.oauth2.initTokenClient({
        client_id: clientId.trim(), scope: SCOPE, include_granted_scopes: false,
        callback: response => {
          const expires = Number(response?.expires_in);
          if (response?.error || typeof response?.access_token !== 'string' || !response.access_token
              || /\s/u.test(response.access_token) || !Number.isFinite(expires) || expires <= 0
              || typeof response?.scope !== 'string' || !response.scope.split(/\s+/u).includes(SCOPE)) return fail();
          resolve({ accessToken: response.access_token, expiresAt: Date.now() + expires * 1000 });
        },
        error_callback: fail,
      });
      client.requestAccessToken({ prompt: 'consent' });
    } catch { fail(); }
  });
}

export const connectGoogle = authorize;

async function request(url, token, options) {
  let response;
  try {
    response = await fetch(url, {
      ...options, headers: { ...options.headers, Authorization: `Bearer ${token}` },
      credentials: 'omit', referrerPolicy: 'no-referrer', redirect: 'error',
    });
  } catch { throw new Error('Drive request failed. Check your connection and try again.'); }
  if (!response.ok) {
    throw new Error(response.status === 401 ? 'Google sign-in expired. Connect again.'
      : 'Drive upload failed. Check your Drive access and storage, then try again.');
  }
  return response;
}

async function fileId(response) {
  let data;
  try { data = await response.json(); } catch { /* Report only a fixed, safe message. */ }
  if (typeof data?.id !== 'string' || !/^[A-Za-z0-9_-]{1,200}$/u.test(data.id)) {
    throw new Error('Drive returned an invalid file response. Try again.');
  }
  return data.id;
}

async function checkImage(file) {
  if (!(file instanceof Blob) || typeof file.name !== 'string' || file.size < 12 || file.size > 25 * 1024 * 1024) {
    throw new Error('Choose a PNG, JPEG, or WebP image up to 25 MiB.');
  }
  const bytes = new Uint8Array(await file.slice(0, 12).arrayBuffer());
  const matches = (offset, signature) => signature.every((byte, index) => bytes[offset + index] === byte);
  const extension = file.name.toLowerCase().match(/\.([a-z0-9]+)$/u)?.[1];
  const mime = extension === 'png' && matches(0, [137, 80, 78, 71, 13, 10, 26, 10]) ? 'image/png'
    : ['jpg', 'jpeg'].includes(extension) && matches(0, [255, 216, 255]) ? 'image/jpeg'
      : extension === 'webp' && matches(0, [82, 73, 70, 70]) && matches(8, [87, 69, 66, 80]) ? 'image/webp' : '';
  if (!mime) throw new Error('The image contents must match its PNG, JPEG, or WebP extension.');
  return { extension, mime };
}

async function uploadFile(token, parent, name, blob, mime) {
  const metadata = JSON.stringify({ name, parents: [parent], mimeType: mime });
  if (blob.size <= 5 * 1024 * 1024) {
    const boundary = `gooblelab_${crypto.randomUUID()}`;
    const body = new Blob([
      `--${boundary}\r\nContent-Type: application/json; charset=UTF-8\r\n\r\n${metadata}\r\n`,
      `--${boundary}\r\nContent-Type: ${mime}\r\n\r\n`, blob, `\r\n--${boundary}--\r\n`,
    ]);
    return fileId(await request(`${UPLOAD}?uploadType=multipart&fields=id`, token, {
      method: 'POST', headers: { 'Content-Type': `multipart/related; boundary=${boundary}` }, body,
    }));
  }
  const response = await request(`${UPLOAD}?uploadType=resumable&fields=id`, token, {
    method: 'POST', headers: {
      'Content-Type': 'application/json; charset=UTF-8', 'X-Upload-Content-Type': mime,
      'X-Upload-Content-Length': String(blob.size),
    }, body: metadata,
  });
  let location;
  try { location = new URL(response.headers.get('Location')); } catch { /* Reject absent or invalid URLs below. */ }
  if (!location || location.origin !== 'https://www.googleapis.com' || location.pathname !== '/upload/drive/v3/files'
      || location.username || location.password || location.hash
      || location.searchParams.get('uploadType') !== 'resumable' || !location.searchParams.get('upload_id')) {
    throw new Error('Drive returned an invalid upload destination. Try again.');
  }
  return fileId(await request(location.href, token, { method: 'PUT', headers: { 'Content-Type': mime }, body: blob }));
}

/** Creates a fresh private My Drive folder; this never shares files or starts Colab compute. */
export async function uploadProject({ accessToken, config, notebookTemplate, catalog, imageFile }) {
  if (typeof accessToken !== 'string' || !accessToken || /\s/u.test(accessToken)) {
    throw new Error('Connect Google before uploading.');
  }
  const selectedImage = config?.template === 'text' ? null : imageFile;
  const settings = validate(selectedImage ? { ...config, imageName: selectedImage.name } : config, catalog);
  const image = settings.template === 'text' ? null : await checkImage(selectedImage);
  const folderName = `${fileStem(settings.name)}-${crypto.randomUUID()}`;
  const imageName = image ? `input.${image.extension}` : null;
  const imagePath = image ? `/content/drive/MyDrive/${folderName}/${imageName}` : null;
  const notebook = buildNotebook(settings, notebookTemplate, catalog, imagePath);
  const notebookMime = 'application/x-ipynb+json';
  const notebookBlob = new Blob([JSON.stringify(notebook, null, 2)], { type: notebookMime });
  const folderId = await fileId(await request(`${FILES}?fields=id`, accessToken, {
    method: 'POST', headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ name: folderName, mimeType: 'application/vnd.google-apps.folder', parents: ['root'] }),
  }));
  if (image) await uploadFile(accessToken, folderId, imageName, selectedImage, image.mime);
  const notebookId = await uploadFile(accessToken, folderId, `${fileStem(settings.name)}.ipynb`, notebookBlob, notebookMime);
  return {
    colabUrl: `https://colab.research.google.com/drive/${notebookId}`,
    driveUrl: `https://drive.google.com/drive/folders/${folderId}`,
  };
}
