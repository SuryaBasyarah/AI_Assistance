"""Haruya RVC profile service.

Discovery and model lifecycle are engine-independent. Put an inference adapter in
`rvc_backend.py` exposing `Backend.load(profile)` and `Backend.convert(...)`.
Only the selected checkpoint is handed to the backend.
"""
from __future__ import annotations

import importlib.util
import json
import os
import re
import tempfile
import threading
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any

from fastapi import FastAPI, HTTPException, Query, Request
from fastapi.responses import Response

ROOT = Path(__file__).resolve().parent
VOICES_ROOT = Path(os.getenv("HARUYA_VOICES_ROOT", ROOT.parent / "voices")).resolve()
VOICES_ROOT.mkdir(parents=True, exist_ok=True)
app = FastAPI(title="Haruya RVC Service", version="0.1.0")
lock = threading.RLock()


@dataclass
class VoiceProfile:
    id: str
    display_name: str
    model_file: str | None
    index_file: str | None
    valid: bool
    loaded: bool = False
    language_hint: str = "ja"
    pitch: int = 0
    f0_method: str = "rmvpe"
    index_rate: float = 0.75
    protect: float = 0.33
    enabled: bool = True
    warning: str | None = None
    error: str | None = None
    _model_path: Path | None = None
    _index_path: Path | None = None

    def public(self) -> dict[str, Any]:
        value = asdict(self)
        value.pop("_model_path", None)
        value.pop("_index_path", None)
        return value


def stable_id(name: str) -> str:
    return re.sub(r"-+", "-", re.sub(r"[^a-z0-9]+", "-", name.lower())).strip("-")


def safe_named_file(folder: Path, value: object) -> Path | None:
    if not isinstance(value, str) or Path(value).name != value:
        return None
    candidate = (folder / value).resolve()
    return candidate if candidate.parent == folder.resolve() and candidate.is_file() else None


def scan_folder(folder: Path) -> VoiceProfile:
    profile_id = stable_id(folder.name)
    config: dict[str, Any] = {}
    config_path = folder / "profile.json"
    if config_path.is_file():
        try:
            config = json.loads(config_path.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError) as exc:
            return VoiceProfile(profile_id, folder.name, None, None, False, error=f"Invalid profile.json: {exc}")
    pths = sorted(p for p in folder.iterdir() if p.is_file() and p.suffix.lower() == ".pth" and not p.name.startswith("."))
    indexes = sorted(p for p in folder.iterdir() if p.is_file() and p.suffix.lower() == ".index" and not p.name.startswith("."))
    model = safe_named_file(folder, config.get("model")) if "model" in config else (pths[0] if len(pths) == 1 else None)
    index = safe_named_file(folder, config.get("index")) if "index" in config else (indexes[0] if len(indexes) == 1 else None)
    enabled = bool(config.get("enabled", True))
    error = None
    if not enabled:
        error = "Disabled in profile.json"
    elif not pths:
        error = "No .pth checkpoint found"
    elif len(pths) > 1 and "model" not in config:
        error = "Multiple .pth files; set model in profile.json"
    elif "model" in config and model is None:
        error = "Configured model file was not found or is unsafe"
    elif len(indexes) > 1 and "index" not in config:
        error = "Multiple .index files; set index in profile.json"
    elif "index" in config and index is None:
        error = "Configured index file was not found or is unsafe"
    warning = "No .index file; backend must support index-less conversion" if error is None and index is None else None
    display = config.get("display_name") or " ".join(part.capitalize() for part in re.split(r"[-_ ]+", folder.name))
    return VoiceProfile(
        id=profile_id, display_name=display, model_file=model.name if model else None,
        index_file=index.name if index else None, valid=error is None, enabled=enabled,
        language_hint=config.get("language_hint", "ja"), pitch=int(config.get("pitch", 0)),
        f0_method=config.get("f0_method", "rmvpe"), index_rate=float(config.get("index_rate", 0.75)),
        protect=float(config.get("protect", 0.33)), warning=warning, error=error,
        _model_path=model, _index_path=index,
    )


def discover() -> tuple[dict[str, VoiceProfile], list[VoiceProfile]]:
    valid: dict[str, VoiceProfile] = {}
    invalid: list[VoiceProfile] = []
    for folder in sorted(VOICES_ROOT.iterdir(), key=lambda p: p.name.lower()):
        if not folder.is_dir() or folder.name.startswith("."):
            continue
        profile = scan_folder(folder)
        if profile.valid:
            valid[profile.id] = profile
        else:
            invalid.append(profile)
    return valid, invalid


class MissingBackend:
    ready = False
    device = "unconfigured"

    def load(self, profile: VoiceProfile) -> None:
        raise RuntimeError("Install/configure rvc-runtime/rvc_backend.py; see README.md")

    def convert(self, input_wav: Path, output_wav: Path, profile: VoiceProfile, settings: dict[str, Any]) -> None:
        raise RuntimeError("RVC inference backend is not configured")


def load_backend() -> Any:
    path = ROOT / "rvc_backend.py"
    if not path.is_file():
        return MissingBackend()
    spec = importlib.util.spec_from_file_location("haruya_rvc_backend", path)
    if spec is None or spec.loader is None:
        return MissingBackend()
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module.Backend()


registry, invalid_profiles = discover()
current_profile: str | None = None
backend = load_backend()


@app.get("/health")
def health() -> dict[str, Any]:
    return {"ready": bool(getattr(backend, "ready", False)), "current_profile": current_profile,
            "device": getattr(backend, "device", "unknown"), "model_loaded": current_profile is not None,
            "index_loaded": bool(current_profile and registry.get(current_profile) and registry[current_profile].index_file),
            "voices_root": str(VOICES_ROOT)}


@app.get("/voices")
def voices() -> dict[str, Any]:
    return {"voices": [p.public() for p in registry.values()], "invalid": [p.public() for p in invalid_profiles]}


@app.post("/voices/refresh")
def refresh() -> dict[str, Any]:
    global registry, invalid_profiles, current_profile
    with lock:
        registry, invalid_profiles = discover()
        if current_profile not in registry:
            current_profile = None
        for key, profile in registry.items():
            profile.loaded = key == current_profile
    return voices()


@app.post("/voices/load")
def load_voice(profile: str = Query(...)) -> dict[str, Any]:
    global current_profile
    with lock:
        target = registry.get(profile)
        if target is None:
            raise HTTPException(404, f"Unknown or invalid voice profile: {profile}")
        previous = current_profile
        try:
            backend.load(target)
        except Exception as exc:
            current_profile = previous
            raise HTTPException(503, f"Failed to load {profile}: {exc}") from exc
        current_profile = profile
        for key, value in registry.items():
            value.loaded = key == profile
    return {"ready": True, "profile": profile}


@app.post("/convert")
async def convert(request: Request, profile: str | None = None, pitch: int | None = None,
                  f0_method: str | None = None, index_rate: float | None = None,
                  resample_sr: int = 0, rms_mix_rate: float = 1.0,
                  protect: float | None = None) -> Response:
    target_id = profile or current_profile
    if not target_id or target_id not in registry:
        raise HTTPException(409, "No valid RVC profile is selected")
    target = registry[target_id]
    if current_profile != target_id:
        load_voice(target_id)
    raw = await request.body()
    if not raw:
        raise HTTPException(400, "Request body must contain WAV audio")
    settings = {"pitch": target.pitch if pitch is None else pitch, "f0_method": f0_method or target.f0_method,
                "index_rate": target.index_rate if index_rate is None else index_rate, "protect": target.protect if protect is None else protect,
                "resample_sr": resample_sr, "rms_mix_rate": rms_mix_rate}
    with lock, tempfile.TemporaryDirectory(prefix="haruya-rvc-") as temp:
        source, output = Path(temp) / "input.wav", Path(temp) / "output.wav"
        source.write_bytes(raw)
        try:
            backend.convert(source, output, target, settings)
        except Exception as exc:
            raise HTTPException(503, f"RVC conversion failed for {target_id}: {exc}") from exc
        if not output.is_file():
            raise HTTPException(500, "RVC backend did not produce output.wav")
        return Response(output.read_bytes(), media_type="audio/wav")


if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="127.0.0.1", port=5200)
