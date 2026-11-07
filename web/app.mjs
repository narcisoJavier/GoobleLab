import { buildNotebook, fileStem, validate } from './notebook.mjs';
import { loadGoogle, authorize, uploadProject } from './drive.mjs';

const $ = (selector) => document.querySelector(selector);
const escape = (value) => String(value ?? '').replace(/[&<>"']/g, (c) => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
const COLAB = 'https://colab.research.google.com/github/narcisoJavier/GoobleLab/blob/v0.2.0/MiniMax_H3_Colab.ipynb';
const REPO = 'https://github.com/narcisoJavier/GoobleLab';
const STORAGE = 'gooblelab.projects.v1';
const defaults = {
  name: 'My MiniMax video', template: 'text', presetId: 'fl2va-turbo-8',
  prompt: 'Cinematic live-action wildlife shot, one continuous take. A red panda sits on a moss-covered branch in a misty bamboo forest at sunrise. It turns toward the camera and tilts its head while a breeze moves its fur. A slow, steady push-in, shallow depth of field, warm golden rim light and natural movement. Audio: bamboo rustling, distant birds and quiet forest ambience. No dialogue, captions, text or logos.',
  width: 1344, height: 768, seconds: 5, seed: '12345', loraSource: '', loraStrength: 1, imageName: '',
};
const names = { text: 'Text to video', image: 'Image to video', reference: 'Reference to video' };
let catalog, notebookTemplate, imageFile = null, imageUrl = null, session = null, busy = false, googleReady = false;
let config = { ...defaults }, plan = 'free', step = 0, projectId = null, published = null, pendingDelete = null;
let toastTimer;

function notify(message, error = false) {
  const status = $('#status'); status.textContent = message; status.dataset.error = error; status.hidden = false;
  clearTimeout(toastTimer); toastTimer = setTimeout(() => { status.hidden = true; }, error ? 10000 : 5500);
}
function activeConfig() {
  return plan === 'pro' ? { ...config } : { ...config, template: 'text', presetId: defaults.presetId,
    width: defaults.width, height: defaults.height, seconds: defaults.seconds, seed: defaults.seed,
    loraSource: '', loraStrength: 1, imageName: '' };
}
function readProjects() {
  try { const data = JSON.parse(localStorage.getItem(STORAGE) || '[]'); return Array.isArray(data) ? data.slice(0, 20) : []; }
  catch { return []; }
}
function page() { return ['projects', 'plans'].includes(location.hash.slice(1)) ? location.hash.slice(1) : 'builder'; }
function previewPro() { plan = 'pro'; published = null; render(); notify('Pro preview enabled. No account or payment is required for this preview.'); }
function useFree() { plan = 'free'; step = 0; published = null; location.hash = 'builder'; render(); notify('Free builder selected. Your advanced settings are kept for the Pro preview.'); }
function frameCount(seconds) {
  const raw = seconds * 24, floor = Math.floor(raw);
  let frames = raw - floor === 0.5 ? floor + floor % 2 : Math.round(raw);
  frames += (5 + 17 - frames % 17) % 17; return frames;
}
function summary() {
  const c = activeConfig();
  return `<h2>Notebook plan</h2><dl>${[
    ['Workflow', names[c.template]], ['Model', 'MiniMax H3'], ['Preset', catalog.presets[c.presetId]?.label.replace('H3 ', '') || 'Choose a preset'],
    ['Video output', `${c.width} × ${c.height}<br><small>${(frameCount(c.seconds) / 24).toFixed(2)} seconds · 24 fps</small>`],
    ['Your Colab runtime', 'A100 · High RAM'], ['Input image', escape(c.imageName || 'None')],
    ['Custom LoRA', c.loraSource ? `Strength ${escape(c.loraStrength)}` : 'None'],
  ].map(([label, value]) => `<div><dt>${label}</dt><dd>${value}</dd></div>`).join('')}</dl>
  <div class="handoff">Template → notebook → Colab</div><div class="plan-actions"><button class="button small" data-action="save">Save project</button><a class="button small" href="${COLAB}" target="_blank" rel="noopener noreferrer">Open core notebook</a></div>
  <p class="plan-note">Projects save in this browser. Generation uses your own Colab account and compute.</p><span class="badge ${plan === 'pro' ? 'pro' : ''}">${plan === 'pro' ? 'Pro customization preview' : 'Free notebook builder'}</span>`;
}
function templateArt(kind) {
  const source = kind === 'text'
    ? '<path d="M21 33h39M21 43h30M21 53h35"/>'
    : '<rect x="17" y="26" width="48" height="34" rx="4"/><circle cx="51" cy="36" r="3"/><path d="m21 55 13-13 10 10 7-7 10 10"/>';
  return `<svg class="template-art" viewBox="0 0 180 86" aria-hidden="true">${source}<path d="M77 43h24m-6-5 6 5-6 5"/><rect x="116" y="22" width="46" height="42" rx="5"/><path d="m135 35 12 8-12 8z"/>${kind === 'reference' ? '<path d="M113 28h-4v42h45v-3"/>' : ''}</svg>`;
}
function templateView() {
  return `<h1>What would you like to create?</h1><p class="intro">Choose a starting point for your MiniMax H3 notebook.</p>
  ${plan === 'pro' ? '<p class="preview-note"><strong>Pro preview.</strong> Explore advanced customization. Billing is not enabled.</p>' : ''}
  <fieldset class="workflow-list"><legend>MiniMax H3 workflow</legend>${[
    ['text', 'Describe a scene, motion and sound. Start with the basic MiniMax H3 notebook.'],
    ['image', 'Bring a first frame and describe what happens next.'],
    ['reference', 'Guide a character or style with a reference image.'],
  ].map(([value, description]) => `<label class="workflow"><span class="workflow-top"><input type="radio" name="template" value="${value}" ${activeConfig().template === value ? 'checked' : ''} ${plan === 'free' && value !== 'text' ? 'disabled' : ''}>${value !== 'text' ? '<span class="badge pro">Pro</span>' : '<span class="badge">Free</span>'}</span>${templateArt(value)}<span class="workflow-copy"><strong>${names[value]}</strong><span class="description">${description}</span></span></label>`).join('')}</fieldset>
  <div class="after-choices">${plan === 'free' ? '<button class="text-button" data-action="pro">Preview Pro workflows and customization</button>' : '<p class="quiet">Curated workflows use the same native ComfyUI nodes as the core notebook.</p>'}
  <p class="quiet">Your notebook runs in Colab. This browser app prepares its settings and files.</p></div>`;
}
function settingsView() {
  const c = activeConfig(), disabled = plan === 'free' ? 'disabled' : '';
  const family = c.template === 'reference' ? 'ref2va' : 'fl2va';
  return `<h1>Make it yours</h1><p class="intro">Describe the video. We’ll put your choices into the notebook.</p>
  <div class="field"><label for="name">Project name</label><input id="name" name="name" value="${escape(c.name)}" maxlength="80" autocomplete="off"></div>
  <div class="field"><label for="prompt">Prompt</label><textarea id="prompt" name="prompt" maxlength="12000">${escape(c.prompt)}</textarea><small>Describe the subject, movement, camera and sound.${c.template === 'reference' ? ' Refer to your image as &lt;Picture 1&gt;.' : ''}</small></div>
  ${c.template !== 'text' ? `<div class="field"><label for="image">${c.template === 'reference' ? 'Reference image' : 'First frame'}</label><input id="image" type="file" accept="image/png,image/jpeg,image/webp,.png,.jpg,.jpeg,.webp"><small>PNG, JPEG or WebP, up to 25 MiB. The image stays on your device until you choose Drive upload.</small></div><div id="image-preview">${imagePreview()}</div>` : ''}
  <section class="advanced"><div class="section-heading"><h2>Video customization <span class="badge pro">Pro</span></h2>${plan === 'free' ? '<button class="text-button" data-action="pro">Preview controls</button>' : ''}</div>
  ${plan === 'free' ? '<p class="quiet">The Free builder uses the 8-step preset, 1344 × 768 and about 5 seconds. Pro adds these browser controls; all exported notebooks remain editable.</p>' : '<p class="quiet">Pro preview is unlocked for evaluation. No subscription or payment is active.</p>'}
  <fieldset ${disabled}><legend class="quiet">Generation settings</legend>
  <div class="field"><label for="presetId">Model preset</label><select id="presetId" name="presetId">${Object.entries(catalog.presets).filter(([, p]) => p.family === family).map(([id, p]) => `<option value="${id}" ${id === c.presetId ? 'selected' : ''}>${escape(p.label)}</option>`).join('')}</select></div>
  ${c.presetId === 'ref2va-turbo-4' ? '<p class="quiet">Experimental reference Turbo was trained at 544p. Try 960 × 544.</p>' : ''}
  <div class="two-fields"><div class="field"><label for="resolution">Resolution</label><select id="resolution" name="resolution">${[[1344,768,'Landscape'],[768,1344,'Portrait'],[960,544,'Reference draft'],[864,480,'Draft'],[768,768,'Square']].map(([w,h,label]) => `<option value="${w}x${h}" ${w === c.width && h === c.height ? 'selected' : ''}>${label} · ${w} × ${h}</option>`).join('')}</select></div><div class="field"><label for="seconds">Duration, seconds</label><input id="seconds" name="seconds" type="number" min="4" max="15" step="0.5" value="${c.seconds}"></div></div>
  <div class="field"><label for="seed">Seed</label><input id="seed" name="seed" inputmode="numeric" value="${escape(c.seed)}" pattern="[0-9]+"><small>Use the same seed and settings to help reproduce a result.</small></div>
  <details class="setup"><summary>Custom LoRA</summary><div class="field"><label for="loraSource">Hugging Face file URL or mounted Drive path</label><input id="loraSource" name="loraSource" value="${escape(c.loraSource)}" placeholder="https://huggingface.co/owner/repo/resolve/main/adapter.safetensors"><small>Use an adapter trained for the selected H3 family. Put private tokens in Colab Secrets, never here.</small></div><div class="field"><label for="loraStrength">LoRA strength</label><input id="loraStrength" name="loraStrength" type="number" min="0" max="2" step="0.05" value="${c.loraStrength}"></div></details>
  </fieldset></section>`;
}
function imagePreview() {
  if (!config.imageName) return '';
  return `<div class="image-selection">${imageUrl ? `<img src="${escape(imageUrl)}" alt="Selected input image">` : ''}<div><strong>${escape(config.imageName)}</strong><p class="quiet">${imageFile ? 'Ready for notebook and Drive export.' : 'Reselect this file to upload it to Drive. A downloaded notebook asks for it inside Colab.'}</p><button class="text-button" data-action="remove-image">Remove image</button></div></div>`;
}
function exportView() {
  const c = activeConfig();
  return `<h1>Your notebook, ready to run</h1><p class="intro">Take your configuration to Colab. You choose the runtime and run the cells.</p>
  <section class="export-path"><h2>Download notebook</h2><p>An editable .ipynb with your prompt and settings. No Google connection is needed here.${c.template !== 'text' ? ' Step 5 will ask you to upload your input image.' : ''}</p><button class="button primary" data-action="download" ${busy ? 'disabled' : ''}>Download .ipynb</button><p class="quiet">In Colab, choose File → Upload notebook. Select A100 + high RAM, then run Steps 1–5.</p></section>
  <section class="export-path"><h2>Send to Google Drive</h2><p>Upload the notebook${c.template !== 'text' ? ' and your selected image' : ''} to a new private folder, then open it in Colab.</p>
  <button class="button" data-action="${session ? 'upload' : 'google'}" ${busy || !googleReady ? 'disabled' : ''}>${session ? 'Upload to Drive & open Colab' : googleReady ? 'Connect Google Drive' : 'Set up Google Drive below'}</button>
  ${session ? '<button class="text-button" data-action="disconnect">Disconnect</button>' : ''}
  <details class="setup" id="google-setup"><summary>Google Drive setup for this preview</summary><p>Use a Google <strong>Web application</strong> OAuth client, with Drive API and <code>drive.file</code> enabled. Add <code>${escape(location.origin)}</code> to its authorized JavaScript origins. A Desktop OAuth JSON is for the Windows app.</p><div class="field"><label for="google-client">Web OAuth client ID</label><input id="google-client" autocomplete="off" value="${escape(stateClientId)}" placeholder="12345-example.apps.googleusercontent.com"><small>This public client ID is kept only for this page. Never paste a client secret.</small></div><button class="button small" data-action="prepare-google" ${busy ? 'disabled' : ''}>Prepare Google connection</button><p><a href="https://developers.google.com/identity/oauth2/web/guides/get-google-api-clientid" target="_blank" rel="noopener noreferrer">Google’s setup instructions</a></p></details>
  <p class="quiet">Drive access is limited to files created by this app. Access tokens stay in page memory. Live Google upload is not yet verified for this preview.</p>
  ${published ? `<div class="success-links"><a href="${escape(published.colabUrl)}" target="_blank" rel="noopener noreferrer">Open uploaded notebook in Colab</a><a href="${escape(published.driveUrl)}" target="_blank" rel="noopener noreferrer">View project folder</a></div>` : ''}</section>
  <section class="export-path"><h2>When you finish in Colab</h2><p>Enable STOP_BACKEND in Step 6 and run it. Then disconnect and delete the runtime to release your GPU. Videos save to My Drive → ComfyUI_Outputs when Drive is enabled.</p></section>`;
}
function builder() {
  return `<div class="shell${step > 0 ? ' is-editing' : ''}"><div class="builder-heading"><p class="builder-title">Your next video.<br><span>Ready for Colab.</span></p><div class="builder-context"><p>From an idea to an editable notebook.<br>Configure MiniMax H3 here. Create on your Colab GPU.</p><a href="${REPO}" target="_blank" rel="noopener noreferrer">Open source. Yours to build on.<svg viewBox="0 0 20 20" aria-hidden="true"><path d="M4 15 15 4M5 4h10v10"/></svg></a></div></div><aside class="rail"><nav class="steps" aria-label="Builder steps">${['Template','Settings','Export / upload'].map((label, i) => `<button class="step" data-step="${i}" ${i === step ? 'aria-current="step"' : ''} ${busy ? 'disabled' : ''}><span class="step-number">${i + 1}</span>${label}</button>`).join('')}</nav><div class="rail-note">Your notebook. Your compute.</div></aside>
  <main class="workspace" id="main" tabindex="-1"><div id="work">${[templateView,settingsView,exportView][step]()}</div><p id="form-error" class="error" role="alert" hidden></p><p id="busy-text" class="busy-text" role="status" ${busy ? '' : 'hidden'}>Working with Google Drive. Please keep this tab open…</p><div class="actions">${step > 0 ? '<button class="button" data-action="back">Back</button>' : ''}${step < 2 ? `<button class="button primary" data-action="next">${step === 0 ? 'Continue to settings' : 'Review notebook'}</button>` : '<button class="text-button" data-action="restart">Start another notebook</button>'}</div></main><aside class="plan" aria-label="Current notebook plan" id="plan">${summary()}</aside></div>`;
}
function projectsView() {
  const projects = readProjects();
  return `<main id="main" class="page" tabindex="-1"><div class="page-head"><div><h1>Your saved projects</h1><p class="intro">Keep a configuration and come back to it.</p></div><button class="button primary" data-action="new">New notebook</button></div><p class="quiet">Saved in this browser on this device. Images and Google tokens are never stored with a project. Cloud accounts and sync are not enabled.</p>${projects.length ? `<div class="table-wrap"><table class="project-table"><thead><tr><th>Project</th><th>Workflow</th><th>Saved</th><th><span class="quiet">Actions</span></th></tr></thead><tbody>${projects.map((p) => `<tr><td><strong>${escape(p.config?.name || 'Untitled project')}</strong><br><span class="quiet">MiniMax H3</span></td><td>${escape(names[p.config?.template] || 'Notebook')}</td><td>${escape(new Date(p.updated).toLocaleDateString(undefined,{month:'short',day:'numeric',year:'numeric'}))}</td><td>${pendingDelete === p.id ? `<button class="text-button" data-delete-confirm="${escape(p.id)}">Delete project</button><button class="text-button" data-action="keep">Keep</button>` : `<button class="button small" data-open="${escape(p.id)}">Open</button><button class="text-button" data-delete="${escape(p.id)}">Remove</button>`}</td></tr>`).join('')}</tbody></table></div>` : '<div class="empty"><h2>A good prompt is worth keeping.</h2><p class="intro">Save a project from the notebook plan. It will appear here with its settings, ready to reopen.</p><a class="button" href="#builder">Go to builder</a></div>'}</main>`;
}
function plansView() {
  return `<main id="main" class="page" tabindex="-1"><h1>Start simple. Customize further.</h1><p class="intro">A free MiniMax H3 notebook builder, with a paid product direction focused on better browser customization.</p><div class="comparison"><section class="offering"><h2>Free</h2><p>The basic notebook, without the setup friction.</p><p class="price">$0</p><ul><li>Text-to-video with MiniMax H3 Turbo 8</li><li>Your own prompt and project name</li><li>1344 × 768 · about 5 seconds · audio</li><li>Download an editable Colab notebook</li><li>Save projects in this browser</li></ul><button class="button" data-action="free">Use Free builder</button></section><section class="offering"><h2>Pro customization</h2><p>More control before you open Colab.</p><p class="price">Preview available</p><ul><li>Image and reference workflows</li><li>Five compatible H3 model presets</li><li>Resolution, duration and seed controls</li><li>Compatible custom LoRA configuration</li><li>Reusable, editable project settings</li></ul><button class="button primary" data-action="pro-builder">Try Pro preview</button><p class="quiet">No charges. Subscription pricing and billing are not enabled.</p></section></div><section class="roadmap"><h2>What a paid service would provide</h2><p>GoobleLab would charge for the convenience of advanced browser configuration, curated workflows and reusable settings. The open-source notebook stays editable; experienced users can still configure it themselves. Colab compute is purchased and used separately through each user’s own account.</p><p>Subscriptions, cloud accounts and cross-device sync are planned. This preview does not accept payments. MiniMax H3 has commercial and territory conditions that must be reviewed before a paid launch.</p><a href="https://huggingface.co/MiniMaxAI/MiniMax-H3/blob/main/LICENSE" target="_blank" rel="noopener noreferrer">Review the MiniMax H3 license</a> · <a href="https://research.google.com/colaboratory/tos_v6.html" target="_blank" rel="noopener noreferrer">Review Colab terms</a></section></main>`;
}
function render() {
  const route = page();
  $('#app').innerHTML = route === 'projects' ? projectsView() : route === 'plans' ? plansView() : builder();
  document.querySelectorAll('.topbar nav a').forEach((a) => a.setAttribute('aria-current', a.hash === `#${route}` ? 'page' : 'false'));
  $('#plan-badge').textContent = plan === 'pro' ? 'Pro preview' : 'Free'; $('#plan-badge').classList.toggle('pro', plan === 'pro');
  $('#preview-pro').textContent = plan === 'pro' ? 'View plans' : 'Preview Pro';
  $('#preview-pro').disabled=busy;
  if(busy) document.querySelectorAll('#app button,#app input,#app textarea,#app select').forEach((el)=>{el.disabled=true;});
}
function error(message) { const el = $('#form-error'); if (el) { el.textContent = message; el.hidden = false; el.scrollIntoView({block:'nearest',behavior:'auto'}); } else notify(message,true); }
function goStep(next) {
  if (next === 2) { try { validate(activeConfig(), catalog); } catch (e) { error(e.message); return; } }
  step = next; render(); $('#main')?.focus({preventScroll:true}); window.scrollTo({top:0,behavior:'auto'});
}
function saveProject() {
  const c = activeConfig();
  if (!c.name.trim()) { notify('Enter a project name before saving.',true); return; }
  const projects = readProjects(), id = projectId || crypto.randomUUID();
  if (!projectId && projects.length >= 20) { notify('This browser holds 20 projects. Remove one before saving another.',true); return; }
  const record = {id,config:c,plan,updated:new Date().toISOString()};
  try { localStorage.setItem(STORAGE,JSON.stringify([record,...projects.filter((p) => p.id !== id)])); projectId=id; notify('Project saved in this browser. Image files stay on your device.'); }
  catch { notify('Browser storage is unavailable or full. Download the notebook to keep your settings.',true); }
}
function download() {
  const notebook = buildNotebook(activeConfig(),notebookTemplate,catalog);
  const url = URL.createObjectURL(new Blob([JSON.stringify(notebook,null,2)],{type:'application/x-ipynb+json'}));
  const link = document.createElement('a'); link.href=url; link.download=`${fileStem(config.name)}.ipynb`; link.click();
  setTimeout(() => URL.revokeObjectURL(url),10000); notify('Notebook downloaded. Upload it in Colab, select A100 + high RAM, then run Steps 1–5.');
}
async function prepareGoogle() {
  const id = $('#google-client').value.trim();
  if (!/^\d+-[A-Za-z0-9_-]+\.apps\.googleusercontent\.com$/.test(id)) throw new Error('Enter a Google Web application client ID, not a client secret or JSON file.');
  busy=true; stateClientId=id; render();
  try { await loadGoogle(); stateClientId=id; googleReady=true; render(); notify('Google is ready. Click Connect Google Drive to authorize in a popup.'); }
  finally { busy=false; render(); }
}
let stateClientId = '';
async function connectDrive() {
  if (!googleReady || !stateClientId) throw new Error('Prepare a Google Web OAuth client first.');
  const promise = authorize(stateClientId); // Keep the popup call inside the user gesture.
  busy=true; render();
  try { session=await promise; notify('Google Drive connected for this page session.'); }
  finally { busy=false; render(); }
}
async function upload() {
  if (!session || Date.now() >= session.expiresAt) { session=null; render(); throw new Error('Your Google session expired. Connect Google Drive again.'); }
  const c = validate(activeConfig(),catalog);
  if (c.template !== 'text' && !imageFile) throw new Error('Reselect your input image before uploading it to Drive.');
  busy=true; render();
  try { published=await uploadProject({accessToken:session.accessToken,config:c,notebookTemplate,catalog,imageFile}); notify('Notebook uploaded. Open the Colab link below and choose your runtime.'); }
  finally { busy=false; render(); }
}
function clearImage() { if(imageUrl) URL.revokeObjectURL(imageUrl); imageUrl=null; imageFile=null; config.imageName=''; }
function reset() { clearImage(); config={...defaults}; projectId=null; published=null; step=0; location.hash='builder'; render(); }

document.addEventListener('input',(event) => {
  const el=event.target; if(!el.name || el.name === 'template' || busy) return;
  if(el.name === 'resolution') [config.width,config.height]=el.value.split('x').map(Number);
  else config[el.name]=['seconds','loraStrength'].includes(el.name) ? Number(el.value) : el.value;
  published=null; const summaryEl=$('#plan'); if(summaryEl)summaryEl.innerHTML=summary(); const message=$('#form-error'); if(message)message.hidden=true;
});
document.addEventListener('change',(event) => {
  const el=event.target;
  if(el.name === 'template') { config.template=el.value; config.presetId=el.value === 'reference' ? 'ref2va-base' : defaults.presetId; if(el.value === 'text')clearImage(); published=null; render(); }
  else if(el.id === 'image') {
    const file=el.files?.[0]; if(!file)return;
    if(!/\.(png|jpe?g|webp)$/i.test(file.name) || file.size===0 || file.size>25*1024*1024) {el.value=''; error('Choose a PNG, JPEG or WebP image up to 25 MiB.'); return;}
    clearImage(); imageFile=file;config.imageName=file.name;imageUrl=URL.createObjectURL(file);published=null;
    $('#image-preview').innerHTML=imagePreview(); $('#plan').innerHTML=summary();
  } else if(el.name === 'presetId')render();
});
document.addEventListener('click',async(event) => {
  if(event.target.closest('.skip')) {event.preventDefault();$('#main')?.focus();return;}
  if(busy && event.target.closest('.topbar nav a')) {event.preventDefault();notify('Please wait for the Google operation to finish.');return;}
  const button=event.target.closest('button'); if(!button)return;
  if(button.id === 'preview-pro') {if(plan === 'pro'){location.hash='plans';}else previewPro();return;}
  if(busy)return;
  try {
    if(button.dataset.step !== undefined) {goStep(Number(button.dataset.step));return;}
    if(button.dataset.open) {
      const record=readProjects().find((p)=>p.id===button.dataset.open); if(!record)return;
      clearImage(); config={...defaults,...record.config};plan=record.plan==='pro'?'pro':'free';projectId=record.id;step=1;published=null;location.hash='builder';render();return;
    }
    if(button.dataset.delete) {pendingDelete=button.dataset.delete;render();return;}
    if(button.dataset.deleteConfirm) {localStorage.setItem(STORAGE,JSON.stringify(readProjects().filter((p)=>p.id!==button.dataset.deleteConfirm)));pendingDelete=null;render();notify('Project removed from this browser.');return;}
    switch(button.dataset.action) {
      case 'pro':previewPro();break;
      case 'pro-builder':plan='pro';step=0;location.hash='builder';render();notify('Pro preview enabled. No charges or subscription.');break;
      case 'free':useFree();break;
      case 'next':goStep(step+1);break;
      case 'back':goStep(step-1);break;
      case 'save':saveProject();break;
      case 'download':download();break;
      case 'prepare-google':await prepareGoogle();break;
      case 'google':await connectDrive();break;
      case 'upload':await upload();break;
      case 'disconnect':session=null;published=null;render();notify('Google session removed from this page.');break;
      case 'remove-image':clearImage();render();break;
      case 'restart':case 'new':reset();break;
      case 'keep':pendingDelete=null;render();break;
    }
  } catch(e) {error(e.message || 'This action could not finish. Please try again.');}
});
window.addEventListener('hashchange',() => {render();$('#main')?.focus({preventScroll:true});window.scrollTo({top:0,behavior:'auto'});});
window.addEventListener('pagehide',() => {session=null;if(imageUrl)URL.revokeObjectURL(imageUrl);});
try {
  const responses=await Promise.all([fetch('./presets.json'),fetch('./template.json')]);
  if(responses.some((r)=>!r.ok))throw new Error('Notebook files could not be loaded. Refresh this page or open the core notebook from GitHub.');
  [catalog,notebookTemplate]=await Promise.all(responses.map((r)=>r.json())); render();
} catch(e) {$('#app').innerHTML=`<main id="main" class="boot"><h1>The builder could not load</h1><p class="error">${escape(e.message)}</p><a href="${COLAB}">Open the core notebook in Colab</a></main>`;}
