"""Offline checks: no GPU, server, installs, or model downloads."""

import ast
import json
from pathlib import Path
import struct
import subprocess
import sys
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import Mock, patch
import urllib.error

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))
import runner


PRESETS = {
    "fl2va-base": (20, "res_multistep", 12.0, 0),
    "fl2va-turbo-8": (8, "res_multistep", 12.0, 1),
    "fl2va-turbo-4": (4, "euler", 6.0, 1),
    "ref2va-base": (20, "res_multistep", 12.0, 0),
    "ref2va-turbo-4": (4, "euler", 12.0, 1),
}
OUTPUT_COUNTS = {
    "UNETLoader": 1, "CLIPLoader": 1, "VAELoader": 1,
    "LoraLoaderModelOnly": 1, "MiniMaxH3SigmaShift": 1,
    "MiniMaxH3ImageToVideo": 2, "MiniMaxH3ReferenceToVideo": 2,
    "BasicGuider": 1, "RandomNoise": 1, "KSamplerSelect": 1,
    "BasicScheduler": 1, "SamplerCustomAdvanced": 2,
    "VAEDecodeTiled": 1, "VAEDecodeAudio": 1, "CreateVideo": 1,
    "SaveVideo": 1, "LoadImage": 2,
}


class GraphTests(unittest.TestCase):
    def build(self, preset="fl2va-turbo-8", **kwargs):
        if preset.startswith("ref2va"):
            kwargs.setdefault("image_name", "reference.png")
        return runner.build_graph(preset, "A robot waves. Audio: soft wind.", **kwargs)

    def one(self, graph, class_type):
        found = [(key, node["inputs"]) for key, node in graph.items()
                 if node["class_type"] == class_type]
        self.assertEqual(len(found), 1, class_type)
        return found[0]

    def test_catalog_matches_manifest(self):
        manifest = json.loads((ROOT / "presets.json").read_text(encoding="utf-8"))
        self.assertEqual(runner.CATALOG, manifest)
        self.assertTrue(PRESETS.keys() <= runner.CATALOG["presets"].keys())

    def test_all_preset_graphs_have_valid_links_and_sampling(self):
        for preset, (steps, sampler, shift, lora_count) in PRESETS.items():
            with self.subTest(preset=preset):
                graph = self.build(preset)
                for node_id, node in graph.items():
                    self.assertIsInstance(node_id, str)
                    for value in node["inputs"].values():
                        if isinstance(value, list):
                            self.assertEqual(len(value), 2)
                            source, slot = value
                            self.assertIn(source, graph)
                            self.assertIsInstance(slot, int)
                            self.assertGreaterEqual(slot, 0)
                            self.assertLess(slot, OUTPUT_COUNTS[graph[source]["class_type"]])

                condition_type = ("MiniMaxH3ReferenceToVideo" if preset.startswith("ref2va")
                                  else "MiniMaxH3ImageToVideo")
                condition_id, condition = self.one(graph, condition_type)
                _, loaded = self.one(graph, "UNETLoader")
                asset = runner.CATALOG["assets"][runner.CATALOG["presets"][preset]["diffusion"]]
                self.assertEqual(loaded["unet_name"], Path(asset["path"]).name)
                self.assertEqual((condition["width"], condition["height"], condition["length"]),
                                 (1344, 768, 124))
                shift_id, shifted = self.one(graph, "MiniMaxH3SigmaShift")
                self.assertEqual((shifted["shift_video"], shifted["shift_audio"]), (shift, 3.0))
                guider_id, guider = self.one(graph, "BasicGuider")
                self.assertEqual(guider["conditioning"], [condition_id, 0])
                self.assertEqual(guider["model"], [shift_id, 0])
                scheduler_id, scheduler = self.one(graph, "BasicScheduler")
                self.assertEqual(scheduler["model"], [shift_id, 0])
                self.assertEqual((scheduler["scheduler"], scheduler["steps"], scheduler["denoise"]),
                                 ("simple", steps, 1.0))
                sampler_id, sampler_inputs = self.one(graph, "KSamplerSelect")
                self.assertEqual(sampler_inputs["sampler_name"], sampler)
                noise_id, noise = self.one(graph, "RandomNoise")
                self.assertEqual(noise["noise_seed"], 12345)
                sampled_id, sampled = self.one(graph, "SamplerCustomAdvanced")
                self.assertEqual(sampled, {
                    "noise": [noise_id, 0], "guider": [guider_id, 0],
                    "sampler": [sampler_id, 0], "sigmas": [scheduler_id, 0],
                    "latent_image": [condition_id, 1],
                })
                video_id, video = self.one(graph, "VAEDecodeTiled")
                audio_id, audio = self.one(graph, "VAEDecodeAudio")
                self.assertEqual(video["samples"], [sampled_id, 0])
                self.assertEqual(audio["samples"], [sampled_id, 0])
                create_id, create = self.one(graph, "CreateVideo")
                self.assertEqual((create["images"], create["audio"], create["fps"]),
                                 ([video_id, 0], [audio_id, 0], 24))
                _, save = self.one(graph, "SaveVideo")
                self.assertEqual(save["video"], [create_id, 0])
                self.assertEqual(save["format"], "mp4")
                self.assertEqual(save["format.codec"], "h264")
                self.assertEqual(save["filename_prefix"], "MiniMaxH3/clip")
                loras = [n for n in graph.values() if n["class_type"] == "LoraLoaderModelOnly"]
                self.assertEqual(len(loras), lora_count)
                if lora_count:
                    asset = runner.CATALOG["assets"][runner.CATALOG["presets"][preset]["lora"]]
                    self.assertEqual(loras[0]["inputs"]["lora_name"], Path(asset["path"]).name)
                upstream = graph[shifted["model"][0]]
                self.assertEqual(upstream["class_type"], "LoraLoaderModelOnly" if lora_count else "UNETLoader")

    def test_image_routes(self):
        for preset in PRESETS:
            with self.subTest(preset=preset):
                graph = self.build(preset, image_name="input.png")
                image_id, image = self.one(graph, "LoadImage")
                self.assertEqual(image["image"], "input.png")
                kind = "MiniMaxH3ReferenceToVideo" if preset.startswith("ref2va") else "MiniMaxH3ImageToVideo"
                _, condition = self.one(graph, kind)
                image_input = "ref_images.ref_image_0" if preset.startswith("ref2va") else "first_frame"
                self.assertEqual(condition[image_input], [image_id, 0])

    def test_reference_presets_require_image(self):
        for preset in ("ref2va-base", "ref2va-turbo-4"):
            with self.subTest(preset=preset), self.assertRaises((ValueError, TypeError)):
                runner.build_graph(preset, "A robot waves.")

    def test_custom_lora_is_after_turbo_and_before_shift(self):
        for preset, (_, _, _, builtin_count) in PRESETS.items():
            with self.subTest(preset=preset):
                graph = self.build(preset, lora_name="user/style.safetensors", lora_strength=0.75)
                _, shifted = self.one(graph, "MiniMaxH3SigmaShift")
                custom = graph[shifted["model"][0]]
                self.assertEqual(custom["class_type"], "LoraLoaderModelOnly")
                self.assertEqual(custom["inputs"]["lora_name"], "user/style.safetensors")
                self.assertEqual(custom["inputs"]["strength_model"], 0.75)
                upstream = graph[custom["inputs"]["model"][0]]
                self.assertEqual(upstream["class_type"], "LoraLoaderModelOnly" if builtin_count else "UNETLoader")

    def test_numeric_validation(self):
        invalid = [
            {"width": 0}, {"width": 1337}, {"height": -32},
            {"width": 1344, "height": 800},
            {"seconds": 3.99}, {"seconds": 15.01},
            {"seconds": float("nan")}, {"seconds": float("inf")},
            {"seed": -1}, {"seed": 2**64}, {"seed": 1.5},
            {"lora_strength": -0.01}, {"lora_strength": 2.01},
            {"lora_strength": float("nan")}, {"lora_strength": float("inf")},
        ]
        for kwargs in invalid:
            with self.subTest(kwargs=kwargs), self.assertRaises((ValueError, TypeError)):
                self.build(**kwargs)
        for prompt in ("", " \n\t"):
            with self.subTest(prompt=prompt), self.assertRaises((ValueError, TypeError)):
                runner.build_graph("fl2va-base", prompt)

    def test_valid_boundary_values_and_frame_grid(self):
        for seconds, frames in ((4, 107), (5, 124), (15, 362)):
            with self.subTest(seconds=seconds):
                graph = self.build(seconds=seconds, seed=2**64 - 1, lora_strength=2.0)
                _, condition = self.one(graph, "MiniMaxH3ImageToVideo")
                self.assertEqual(condition["length"], frames)


class LoraSourceTests(unittest.TestCase):
    def test_hugging_face_sources(self):
        for route, revision, filename, query in (
            ("blob", "main", "style.safetensors", ""),
            ("resolve", "abc123", "nested/style.safetensors", ""),
            ("blob", "main", "nested/style.safetensors", "?download=true"),
        ):
            source = f"https://huggingface.co/author/repository/{route}/{revision}/{filename}{query}"
            with self.subTest(source=source):
                self.assertEqual(runner.parse_lora_source(source),
                                 ("hf", "author/repository", revision, filename))

    def test_absolute_local_path_does_not_need_to_exist(self):
        source = str(Path(tempfile.gettempdir()) / "not-created-minimax-lora.safetensors")
        self.assertEqual(runner.parse_lora_source(source), ("local", "", "", source))

    def test_rejects_unsafe_or_unsupported_sources(self):
        base = "https://huggingface.co/author/repository/resolve/main/"
        invalid = [
            "http://huggingface.co/author/repository/resolve/main/a.safetensors",
            "https://example.com/author/repository/resolve/main/a.safetensors",
            "https://huggingface.co.evil.example/author/repository/resolve/main/a.safetensors",
            "https://user:password@huggingface.co/author/repository/resolve/main/a.safetensors",
            "https://huggingface.co/author/repository/tree/main/a.safetensors",
            base + "../a.safetensors", base + "%2e%2e/a.safetensors",
            base + "folder\\a.safetensors", base + "folder%5ca.safetensors",
            base + "a.ckpt", base + "a.safetensors?token=not-a-secret",
            base + "a.safetensors?download=true&token=not-a-secret",
            base + "a.safetensors?token=", base + "a.safetensors?download=true&token=",
            "relative/a.safetensors", "", "/content/a.ckpt",
        ]
        for source in invalid:
            with self.subTest(source=source), self.assertRaises((ValueError, TypeError)):
                runner.parse_lora_source(source)


class RuntimeTests(unittest.TestCase):
    def setUp(self):
        self.enterContext(patch("builtins.print"))

    @staticmethod
    def tensor_bytes(end=4, payload=b"1234"):
        header = json.dumps({"weight": {
            "dtype": "F32", "shape": [1], "data_offsets": [0, end]
        }}).encode()
        return struct.pack("<Q", len(header)) + header + payload

    def test_safetensors_rejects_truncated_payload(self):
        with tempfile.TemporaryDirectory() as directory:
            target = Path(directory) / "adapter.safetensors"
            target.write_bytes(self.tensor_bytes())
            runner._check_safetensors(target)
            for invalid in (b"<html>error</html>", self.tensor_bytes(end=100), self.tensor_bytes(payload=b"")):
                target.write_bytes(invalid)
                with self.subTest(size=len(invalid)), self.assertRaises(ValueError):
                    runner._check_safetensors(target)

    def test_model_cache_reused_and_incomplete_file_replaced(self):
        data = self.tensor_bytes()
        asset = {"path": "diffusion_models/sample.safetensors", "size": len(data)}
        download = Mock()
        hub = SimpleNamespace(hf_hub_download=download, get_hf_file_metadata=Mock(), hf_hub_url=Mock())
        with tempfile.TemporaryDirectory() as directory:
            target = Path(directory) / asset["path"]
            target.parent.mkdir()
            target.write_bytes(data)
            def download_file(**kwargs):
                target.write_bytes(data)
                return str(target)
            download.side_effect = download_file
            with patch.dict(sys.modules, {"huggingface_hub": hub}), patch.object(runner, "selected_assets", return_value={"sample": asset}):
                self.assertIsNone(runner.prepare_models("fl2va-base", directory))
                download.assert_not_called()
                target.write_bytes(data[:-1])
                runner.prepare_models("fl2va-base", directory)
                download.assert_called_once()
                self.assertTrue(download.call_args.kwargs["force_download"])
                self.assertEqual(target.read_bytes(), data)

    def test_custom_hf_cache_resolves_revision_and_repairs_truncation(self):
        data = self.tensor_bytes()
        revision = "1" * 40
        metadata = SimpleNamespace(commit_hash=revision, size=len(data))
        download = Mock()
        hub = SimpleNamespace(hf_hub_download=download, get_hf_file_metadata=Mock(return_value=metadata), hf_hub_url=Mock())
        def download_file(**kwargs):
            target = Path(kwargs["local_dir"]) / kwargs["filename"]
            if not target.exists() or kwargs.get("force_download"):
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_bytes(data)
            return str(target)
        download.side_effect = download_file
        with tempfile.TemporaryDirectory() as directory, patch.dict(sys.modules, {"huggingface_hub": hub}), patch.object(runner, "selected_assets", return_value={}):
            source = "https://huggingface.co/author/repository/blob/main/nested/style.safetensors"
            result = runner.prepare_models("fl2va-base", directory, source)
            self.assertEqual(download.call_args.kwargs["revision"], revision)
            target = Path(directory) / "loras" / result
            target.write_bytes(data[:-1])
            self.assertEqual(runner.prepare_models("fl2va-base", directory, source), result)
            self.assertTrue(download.call_args.kwargs["force_download"])
            self.assertEqual(target.read_bytes(), data)

    def test_stop_backend_escalates_and_waits(self):
        runner.stop_backend(None)
        stopped = Mock()
        stopped.poll.return_value = 0
        runner.stop_backend(stopped)
        stopped.terminate.assert_not_called()
        active = Mock()
        active.poll.return_value = None
        active.wait.side_effect = [subprocess.TimeoutExpired("comfy", 30), None]
        runner.stop_backend(active)
        self.assertEqual([call[0] for call in active.method_calls],
                         ["poll", "terminate", "wait", "kill", "wait"])

    def test_generate_stopped_backend_fails_before_api(self):
        with patch.object(runner, "api") as api:
            with self.assertRaisesRegex(RuntimeError, "Step 4"):
                runner.generate("fl2va-base", "A robot waves.", None, "/unused", "/unused")
            api.assert_not_called()
        proc = Mock()
        proc.poll.return_value = None
        with patch.object(runner, "api", side_effect=urllib.error.URLError("unreachable")):
            with self.assertRaisesRegex(RuntimeError, "Step 4"):
                runner.generate("fl2va-base", "A robot waves.", proc, "/unused", "/unused")

    def test_failed_startup_stops_backend(self):
        proc = Mock()
        proc.poll.return_value = None
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            comfy = root / "comfy"
            comfy.mkdir()
            with patch.object(runner.socket, "socket") as socket, patch.object(runner.subprocess, "Popen", return_value=proc), patch.object(runner, "api", side_effect=urllib.error.URLError("not ready")), patch.object(runner.time, "sleep"), patch.object(runner, "LOG_PATH", root / "comfy.log"):
                socket.return_value.__enter__.return_value.connect_ex.return_value = 1
                with self.assertRaises(TimeoutError):
                    runner.start_backend(comfy, root / "models", root / "output")
                proc.terminate.assert_called_once()
                proc.wait.assert_called_once_with(timeout=30)

    def test_generate_success_and_cancel_without_server(self):
        graph = runner.build_graph("fl2va-base", "A robot waves.")
        node_info = {node["class_type"]: {"input": {"required": {}}} for node in graph.values()}
        proc = Mock()
        proc.poll.return_value = None
        for outcome in ("success", "interrupt", "timeout"):
            with self.subTest(outcome=outcome), tempfile.TemporaryDirectory() as directory:
                output = Path(directory)
                def api_response(path, payload=None):
                    if path == "/object_info":
                        return node_info
                    if path == "/prompt":
                        if outcome == "interrupt":
                            raise KeyboardInterrupt()
                        save = next(n["inputs"] for n in payload["prompt"].values() if n["class_type"] == "SaveVideo")
                        (output / (save["filename_prefix"] + "_00001_.mp4")).write_bytes(b"mp4")
                        return {"prompt_id": "test"}
                    if path == "/history/test":
                        return {"test": {"status": {"status_str": "success", "completed": True}}}
                    if path == "/interrupt":
                        return {}
                    self.fail(f"Unexpected request: {path}")
                with patch.object(runner, "api", side_effect=api_response) as api, patch.object(runner.time, "monotonic", side_effect=[0, 61] if outcome == "timeout" else [0, 1]):
                    if outcome == "success":
                        video = runner.generate("fl2va-base", "A robot waves.", proc, directory, output)
                        self.assertEqual(video.read_bytes(), b"mp4")
                        self.assertTrue((video.parent / "workflow_api.json").is_file())
                    else:
                        with self.assertRaises(KeyboardInterrupt if outcome == "interrupt" else TimeoutError):
                            runner.generate("fl2va-base", "A robot waves.", proc, directory, output, timeout_minutes=1)
                        self.assertIn(unittest.mock.call("/interrupt", {}), api.call_args_list)


class NotebookTests(unittest.TestCase):
    def test_notebook_code_cells_parse(self):
        notebooks = list(ROOT.glob("*.ipynb"))
        self.assertTrue(notebooks, "Expected the Colab notebook in the repository root")
        for notebook in notebooks:
            data = json.loads(notebook.read_text(encoding="utf-8"))
            for index, cell in enumerate(data["cells"]):
                if cell["cell_type"] == "code":
                    with self.subTest(notebook=notebook.name, cell=index):
                        ast.parse("".join(cell["source"]), filename=f"{notebook.name}:cell{index}")


if __name__ == "__main__":
    unittest.main()
