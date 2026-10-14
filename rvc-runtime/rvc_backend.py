from pathlib import Path
from typing import Any
import subprocess


class Backend:
    ready = True
    device = "cuda"

    def __init__(self) -> None:
        root = Path(__file__).resolve().parent.parent

        self.rvc_root = root / "rvc-engine"
        self.python = (
            self.rvc_root
            / ".venv"
            / "Scripts"
            / "python.exe"
        )

        self.current_profile = None

        if not self.rvc_root.is_dir():
            self.ready = False
            raise RuntimeError(
                f"RVC engine not found: {self.rvc_root}"
            )

        if not self.python.is_file():
            self.ready = False
            raise RuntimeError(
                f"RVC Python not found: {self.python}"
            )

    def load(self, profile: Any) -> None:
        model = profile._model_path

        if model is None or not model.is_file():
            raise RuntimeError(
                f"RVC model not found: {model}"
            )

        index = profile._index_path

        if index is not None and not index.is_file():
            raise RuntimeError(
                f"RVC index not found: {index}"
            )

        # Validate checkpoint
        command = [
            str(self.python),
            "-m",
            "infer.cli",
            "--model",
            str(model),
            "--list-speakers",
        ]

        result = subprocess.run(
            command,
            cwd=str(self.rvc_root),
            capture_output=True,
            text=True,
            timeout=120,
        )

        if result.returncode != 0:
            raise RuntimeError(
                "Failed to load RVC checkpoint.\n"
                f"STDOUT:\n{result.stdout}\n"
                f"STDERR:\n{result.stderr}"
            )

        self.current_profile = profile

    def convert(
        self,
        input_wav: Path,
        output_wav: Path,
        profile: Any,
        settings: dict[str, Any],
    ) -> None:

        if not input_wav.is_file():
            raise RuntimeError(
                f"Input WAV not found: {input_wav}"
            )

        model = profile._model_path

        if model is None:
            raise RuntimeError("RVC model path is missing")

        command = [
            str(self.python),
            "-m",
            "infer.cli",

            "--model",
            str(model),

            "--speaker-id",
            "0",

            "--input",
            str(input_wav),

            "--output",
            str(output_wav),

            "--pitch",
            str(settings.get("pitch", 0)),

            "--f0-method",
            str(settings.get("f0_method", "rmvpe")),

            "--index-rate",
            str(settings.get("index_rate", 0.75)),

            "--resample-sr",
            str(settings.get("resample_sr", 0)),

            "--rms-mix-rate",
            str(settings.get("rms_mix_rate", 1.0)),

            "--protect",
            str(settings.get("protect", 0.33)),

            "--overwrite",
        ]

        index = profile._index_path

        if index is not None:
            command.extend([
                "--index",
                str(index),
            ])

        result = subprocess.run(
            command,
            cwd=str(self.rvc_root),
            capture_output=True,
            text=True,
            timeout=180,
        )

        if result.returncode != 0:
            raise RuntimeError(
                "RVC conversion failed.\n"
                f"STDOUT:\n{result.stdout}\n"
                f"STDERR:\n{result.stderr}"
            )

        if not output_wav.is_file():
            raise RuntimeError(
                "RVC returned successfully but no output WAV was created"
            )