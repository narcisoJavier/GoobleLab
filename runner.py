"""Native MiniMax H3 inference for the companion Google Colab notebook."""
import hashlib
import json
import math
import re
import shutil
import socket
import struct
import subprocess
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
import uuid
from datetime import datetime
from pathlib import Path, PurePosixPath

CATALOG = json.loads(Path(__file__).with_name('presets.json').read_text(encoding='utf-8'))
COMFY_VERSION = 'v0.44.0'
COMFY_COMMIT = '2c75dc1b379a59e0880619d029607af5169a62ef'
BASE_URL = 'http://127.0.0.1:8188'
LOG_PATH = Path('/content/minimax_h3_comfy.log')


def _preset(preset_id):
    if preset_id not in CATALOG['presets']:
        raise ValueError(f'Unknown preset: {preset_id}')
    return CATALOG['presets'][preset_id]


def _relative_file(filename):
    path = PurePosixPath(filename)
    if (not filename or '\\' in filename or ':' in filename or path.is_absolute()
            or '..' in path.parts or path.suffix.lower() != '.safetensors'):
        raise ValueError('Use a relative .safetensors filename without traversal.')
    return filename


def parse_lora_source(source):
    """Accept an absolute local path or a normal Hugging Face file URL."""
    source = source.strip()
    if source.startswith(('https://', 'http://')):
        url = urllib.parse.urlsplit(source)
        if (url.scheme != 'https' or url.netloc != 'huggingface.co'
                or url.username or url.password or url.fragment):
            raise ValueError('Use an HTTPS huggingface.co file URL, without credentials.')
        if urllib.parse.parse_qs(url.query, keep_blank_values=True) not in ({}, {'download': ['true']}, {'download': ['1']}):
            raise ValueError('Remove URL tokens/query parameters. Use the HF_TOKEN Colab secret.')
        parts = urllib.parse.unquote(url.path).strip('/').split('/')
        if (len(parts) < 5 or parts[2] not in ('resolve', 'blob')
                or not all(re.fullmatch(r'[A-Za-z0-9_.-]+', p) and p not in ('.', '..')
                           for p in (parts[0], parts[1], parts[3]))):
            raise ValueError('Expected https://huggingface.co/OWNER/REPO/resolve/REVISION/FILE.safetensors')
        filename = _relative_file('/'.join(parts[4:]))
        return 'hf', '/'.join(parts[:2]), parts[3], filename
    path = Path(source).expanduser()
    if not path.is_absolute() or path.suffix.lower() != '.safetensors':
        raise ValueError('Use an absolute local .safetensors path or a Hugging Face file URL.')
    if '..' in path.parts:
        raise ValueError('Local LoRA paths must not contain traversal.')
    return 'local', '', '', str(path)


def _check_safetensors(path):
    """Reject a truncated download or an HTML error page before model loading."""
    size = path.stat().st_size
    with path.open('rb') as stream:
        raw = stream.read(8)
        if len(raw) != 8:
            raise ValueError(f'Invalid safetensors file: {path.name}')
        length = struct.unpack('<Q', raw)[0]
        if not 2 <= length <= min(100_000_000, size - 8):
            raise ValueError(f'Invalid safetensors header: {path.name}')
        header = json.loads(stream.read(length))
    if not isinstance(header, dict):
        raise ValueError(f'Invalid safetensors header: {path.name}')
    tensors = [value for key, value in header.items() if key != '__metadata__']
    payload_size = size - 8 - length
    for tensor in tensors:
        offsets = tensor.get('data_offsets') if isinstance(tensor, dict) else None
        if (not isinstance(offsets, list) or len(offsets) != 2
                or any(type(offset) is not int for offset in offsets)
                or not 0 <= offsets[0] <= offsets[1] <= payload_size):
            raise ValueError(f'Truncated or invalid tensor data: {path.name}')
    if not tensors:
        raise ValueError(f'No tensors in {path.name}')


def selected_assets(preset_id):
    preset = _preset(preset_id)
    names = CATALOG['common'] + [preset['diffusion']]
    if preset['lora']:
        names.append(preset['lora'])
    return {name: CATALOG['assets'][name] for name in names}


def install_comfy(comfy_dir):
    comfy_dir = Path(comfy_dir)
    if not comfy_dir.exists():
        subprocess.run([
            'git', 'clone', '--depth', '1', '--branch', COMFY_VERSION,
            'https://github.com/Comfy-Org/ComfyUI.git', str(comfy_dir)
        ], check=True)
    revision = subprocess.check_output(
        ['git', '-C', str(comfy_dir), 'rev-parse', 'HEAD'], text=True
    ).strip()
    if revision != COMFY_COMMIT:
        raise RuntimeError('Different ComfyUI version found. Use a fresh runtime or a new install directory.')
    subprocess.run([
        sys.executable, '-m', 'pip', 'install', '-q', '-r',
        str(comfy_dir / 'requirements.txt'), 'huggingface_hub'
    ], check=True)
    subprocess.run([
        sys.executable, '-c',
        "import torch; print('PyTorch:', torch.__version__, 'CUDA:', torch.version.cuda); "
        "assert torch.cuda.is_available(), 'PyTorch cannot access the GPU'"
    ], check=True)


def prepare_models(preset_id, model_dir, custom_lora='', token=None):
    lora_spec = parse_lora_source(custom_lora) if custom_lora.strip() else None
    if lora_spec and lora_spec[0] == 'local' and not Path(lora_spec[3]).is_file():
        raise ValueError('The custom LoRA path must point to an existing file.')
    from huggingface_hub import get_hf_file_metadata, hf_hub_download, hf_hub_url

    model_dir = Path(model_dir).expanduser().resolve()
    assets = selected_assets(preset_id)
    print(f'Selected model files: {sum(a["size"] for a in assets.values()) / 1e9:.1f} GB')
    for asset in assets.values():
        filename = _relative_file(asset['path'])
        target = model_dir / filename
        if not target.exists() or target.stat().st_size != asset['size']:
            print('Downloading:', filename)
            hf_hub_download(
                repo_id=CATALOG['repo_id'], revision=CATALOG['revision'], filename=filename,
                local_dir=str(model_dir), token=token, force_download=target.exists()
            )
        if target.stat().st_size != asset['size']:
            raise RuntimeError(f'Incomplete download: {filename}')
        _check_safetensors(target)
        print('Ready:', filename)
    if lora_spec is None:
        return None
    kind, repo, revision, filename = lora_spec
    lora_dir = model_dir / 'loras'
    lora_dir.mkdir(parents=True, exist_ok=True)
    if kind == 'hf':
        metadata = get_hf_file_metadata(hf_hub_url(repo, filename, revision=revision), token=token)
        # Resolve branch names to a commit so this run uses a fixed adapter.
        if not metadata.commit_hash or metadata.size is None:
            raise RuntimeError('Hugging Face did not return adapter revision/size metadata.')
        cache_id = hashlib.sha256(f'{repo}/{metadata.commit_hash}/{filename}'.encode()).hexdigest()[:16]
        cache_dir = lora_dir / ('custom_' + cache_id)
        cached = cache_dir / filename
        target = Path(hf_hub_download(
            repo_id=repo, revision=metadata.commit_hash, filename=filename,
            local_dir=str(cache_dir), token=token,
            force_download=cached.exists() and cached.stat().st_size != metadata.size
        ))
        if target.stat().st_size != metadata.size:
            raise RuntimeError('Incomplete custom LoRA download.')
    else:
        source = Path(filename).resolve(strict=True)
        if not source.is_file():
            raise ValueError('The custom LoRA must be a file.')
        if source.is_relative_to(lora_dir.resolve()):
            target = source
        else:
            identity = f'{source}:{source.stat().st_size}:{source.stat().st_mtime_ns}'
            cache_id = hashlib.sha256(identity.encode()).hexdigest()[:16]
            target = lora_dir / f'custom_{cache_id}_{source.name}'
            if not target.exists() or target.stat().st_size != source.stat().st_size:
                shutil.copy2(source, target)
    _check_safetensors(target)
    result = target.relative_to(lora_dir).as_posix()
    print('Custom model-only LoRA:', result)
    return result


def api(path, payload=None):
    request = urllib.request.Request(
        BASE_URL + path, data=None if payload is None else json.dumps(payload).encode(),
        headers={'Content-Type': 'application/json'}
    )
    try:
        with urllib.request.urlopen(request, timeout=30) as response:
            return json.load(response)
    except urllib.error.HTTPError as exc:
        raise RuntimeError(exc.read().decode()) from exc


def stop_backend(proc):
    if proc is not None and proc.poll() is None:
        proc.terminate()
        try:
            proc.wait(timeout=30)
        except subprocess.TimeoutExpired:
            proc.kill()
            proc.wait()


def start_backend(comfy_dir, model_dir, output_dir, previous=None):
    stop_backend(previous)
    comfy_dir, model_dir, output_dir = map(Path, (comfy_dir, model_dir, output_dir))
    output_dir.mkdir(parents=True, exist_ok=True)
    with socket.socket() as sock:
        sock.settimeout(1)
        if sock.connect_ex(('127.0.0.1', 8188)) == 0:
            raise RuntimeError('Port 8188 is already used. Stop the other backend or reset the runtime.')
    model_paths = comfy_dir / 'colab_model_paths.yaml'
    # JSON is valid YAML; no separate YAML dependency is needed here.
    model_paths.write_text(json.dumps({'colab': {
        'base_path': str(model_dir),
        **{key: key for key in ('diffusion_models', 'text_encoders', 'vae', 'loras')}
    }}), encoding='utf-8')
    with LOG_PATH.open('w') as log:
        proc = subprocess.Popen([
            sys.executable, '-u', 'main.py', '--listen', '127.0.0.1', '--port', '8188',
            '--output-directory', str(output_dir), '--extra-model-paths-config', str(model_paths),
            '--reserve-vram', '2', '--disable-all-custom-nodes', '--disable-partner-nodes'
        ], cwd=str(comfy_dir), stdout=log, stderr=subprocess.STDOUT)
    try:
        for _ in range(180):
            if proc.poll() is not None:
                raise RuntimeError(LOG_PATH.read_text(errors='replace')[-12000:])
            try:
                nodes = api('/object_info')
                if 'MiniMaxH3SigmaShift' not in nodes or 'SaveVideo' not in nodes:
                    raise RuntimeError('Native H3/video nodes failed to load. Read the backend log.')
                print('Ready for Step 5. Offloading is enabled.')
                return proc
            except (urllib.error.URLError, TimeoutError):
                time.sleep(2)
        raise TimeoutError(f'Backend startup timed out. Read {LOG_PATH}.')
    except BaseException:
        stop_backend(proc)
        raise


def build_graph(preset_id, prompt, *, width=1344, height=768, seconds=5, seed=12345,
                image_name=None, lora_name=None, lora_strength=1.0, filename_prefix='MiniMaxH3/clip'):
    preset = _preset(preset_id)
    if not isinstance(prompt, str) or not prompt.strip():
        raise ValueError('Enter a nonempty prompt.')
    if (type(width) is not int or type(height) is not int
            or not 256 <= width <= 1344 or not 256 <= height <= 1344
            or width % 32 or height % 32 or width * height > 1344 * 768):
        raise ValueError('Use dimensions divisible by 32, within 1344x768 total pixels.')
    if not math.isfinite(seconds) or not 4 <= seconds <= 15:
        raise ValueError('Choose 4-15 seconds.')
    if type(seed) is not int or not 0 <= seed < 2**64:
        raise ValueError('Seed must be an unsigned 64-bit integer.')
    if not math.isfinite(lora_strength) or not 0 <= lora_strength <= 2:
        raise ValueError('LoRA strength must be between 0 and 2.')
    if preset['family'] == 'ref2va' and not image_name:
        raise ValueError('Reference presets require INPUT_IMAGE. Refer to it as <Picture 1> in the prompt.')
    if lora_name:
        _relative_file(lora_name)
    prefix = PurePosixPath(filename_prefix)
    if prefix.is_absolute() or '..' in prefix.parts or '\\' in filename_prefix or ':' in filename_prefix:
        raise ValueError('Output prefix must stay within the output directory.')
    if image_name and (Path(image_name).name != image_name or '/' in image_name or '\\' in image_name):
        raise ValueError('Image name must be a staged input filename.')
    frames = max(5, round(seconds * 24))
    frames += (5 - frames % 17) % 17
    assets = CATALOG['assets']
    graph = {
        '1': {'class_type': 'UNETLoader', 'inputs': {
            'unet_name': Path(assets[preset['diffusion']]['path']).name, 'weight_dtype': 'default'}},
        '2': {'class_type': 'CLIPLoader', 'inputs': {
            'clip_name': Path(assets['text']['path']).name, 'type': 'minimax', 'device': 'default'}},
        '3': {'class_type': 'VAELoader', 'inputs': {'vae_name': Path(assets['video_vae']['path']).name}},
        '4': {'class_type': 'VAELoader', 'inputs': {'vae_name': Path(assets['audio_vae']['path']).name}},
        '5': {'class_type': 'MiniMaxH3ImageToVideo', 'inputs': {
            'clip': ['2', 0], 'vae': ['3', 0], 'prompt': prompt,
            'width': width, 'height': height, 'length': frames}},
        '7': {'class_type': 'BasicGuider', 'inputs': {'model': ['shift', 0], 'conditioning': ['5', 0]}},
        '8': {'class_type': 'RandomNoise', 'inputs': {'noise_seed': seed}},
        '9': {'class_type': 'KSamplerSelect', 'inputs': {'sampler_name': preset['sampler']}},
        '10': {'class_type': 'BasicScheduler', 'inputs': {
            'model': ['shift', 0], 'scheduler': 'simple', 'steps': preset['steps'], 'denoise': 1.0}},
        '11': {'class_type': 'SamplerCustomAdvanced', 'inputs': {
            'noise': ['8', 0], 'guider': ['7', 0], 'sampler': ['9', 0],
            'sigmas': ['10', 0], 'latent_image': ['5', 1]}},
        '12': {'class_type': 'VAEDecodeTiled', 'inputs': {
            'samples': ['11', 0], 'vae': ['3', 0], 'tile_size': 512,
            'overlap': 64, 'temporal_size': 64, 'temporal_overlap': 8}},
        '13': {'class_type': 'VAEDecodeAudio', 'inputs': {'samples': ['11', 0], 'vae': ['4', 0]}},
        '14': {'class_type': 'CreateVideo', 'inputs': {'images': ['12', 0], 'audio': ['13', 0], 'fps': 24}},
        '15': {'class_type': 'SaveVideo', 'inputs': {
            'video': ['14', 0], 'filename_prefix': filename_prefix, 'format': 'mp4',
            'format.codec': 'h264', 'format.codec.encoding': 'auto'}},
    }
    model = ['1', 0]
    if preset['lora']:
        graph['6'] = {'class_type': 'LoraLoaderModelOnly', 'inputs': {
            'model': model, 'lora_name': Path(assets[preset['lora']]['path']).name, 'strength_model': 1.0}}
        model = ['6', 0]
    if lora_name:
        graph['custom_lora'] = {'class_type': 'LoraLoaderModelOnly', 'inputs': {
            'model': model, 'lora_name': lora_name, 'strength_model': lora_strength}}
        model = ['custom_lora', 0]
    graph['shift'] = {'class_type': 'MiniMaxH3SigmaShift', 'inputs': {
        'model': model, 'shift_video': preset['shift_video'], 'shift_audio': preset['shift_audio']}}
    if image_name:
        graph['image'] = {'class_type': 'LoadImage', 'inputs': {'image': image_name}}
        if preset['family'] == 'fl2va':
            graph['5']['inputs']['first_frame'] = ['image', 0]
        else:
            graph['5']['class_type'] = 'MiniMaxH3ReferenceToVideo'
            graph['5']['inputs'].update({
                'audio_vae': ['4', 0], 'ref_image_size': 'match',
                'ref_images.ref_image_0': ['image', 0]
            })
    return graph


def generate(preset_id, prompt, proc, comfy_dir, output_dir, *, image_path='', timeout_minutes=60, **settings):
    if proc is None or proc.poll() is not None:
        raise RuntimeError('ComfyUI is stopped. Run Step 4, wait for Ready, then Step 5.')
    if not math.isfinite(timeout_minutes) or not 1 <= timeout_minutes <= 180:
        raise ValueError('Timeout must be 1-180 minutes.')
    try:
        node_info = api('/object_info')
    except (urllib.error.URLError, TimeoutError) as exc:
        raise RuntimeError('Backend unreachable. Rerun Step 4, then Step 5.') from exc
    image_name = None
    if image_path.strip():
        source = Path(image_path).expanduser().resolve(strict=True)
        if not source.is_file() or source.suffix.lower() not in ('.png', '.jpg', '.jpeg', '.webp'):
            raise ValueError('INPUT_IMAGE must be a local PNG, JPEG, or WebP file.')
        image_name = f'colab_{uuid.uuid4().hex}{source.suffix.lower()}'
        destination = Path(comfy_dir) / 'input' / image_name
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source, destination)
    run_name = 'MiniMaxH3_' + datetime.now().strftime('%Y%m%d_%H%M%S_%f')
    graph = build_graph(preset_id, prompt, image_name=image_name, filename_prefix=run_name + '/clip', **settings)
    for node in graph.values():
        node_type = node['class_type']
        if node_type not in node_info:
            raise RuntimeError(f'Missing native node {node_type}. Read {LOG_PATH}.')
        missing = set(node_info[node_type]['input'].get('required', {})) - node['inputs'].keys()
        if missing:
            raise RuntimeError(f'Missing {node_type} inputs: {missing}')
    run_dir = Path(output_dir) / run_name
    run_dir.mkdir(parents=True, exist_ok=False)
    (run_dir / 'workflow_api.json').write_text(json.dumps(graph, indent=2), encoding='utf-8')
    try:
        reply = api('/prompt', {'prompt': graph})
        if not reply.get('prompt_id') or reply.get('node_errors'):
            raise RuntimeError(str(reply))
        prompt_id = reply['prompt_id']
        frames = graph['5']['inputs']['length']
        print(f'Generating {frames} frames ({frames/24:.2f}s). Prompt ID: {prompt_id}')
        deadline = time.monotonic() + timeout_minutes * 60
        last_log = ''
        while time.monotonic() < deadline:
            if proc.poll() is not None:
                raise RuntimeError(LOG_PATH.read_text(errors='replace')[-12000:])
            result = api('/history/' + prompt_id).get(prompt_id)
            if result:
                status = result.get('status', {})
                if status.get('status_str') == 'error':
                    raise RuntimeError(json.dumps(status.get('messages', []), indent=2)[-12000:])
                if status.get('completed'):
                    break
            tail = LOG_PATH.read_text(errors='replace')[-1000:].strip().splitlines()
            if tail and tail[-1] != last_log:
                last_log = tail[-1]
                print(last_log[-240:])
            time.sleep(10)
        else:
            raise TimeoutError('Generation timed out. Reduce the settings and retry.')
    except (KeyboardInterrupt, TimeoutError):
        try:
            api('/interrupt', {})
        except (urllib.error.URLError, TimeoutError):
            pass
        raise
    videos = sorted(run_dir.glob('*.mp4'))
    if not videos:
        raise RuntimeError(f'No MP4 saved. Read {LOG_PATH}.')
    print('Saved:', videos[-1])
    return videos[-1]
