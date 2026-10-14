# Haruya Virtual Companion

Tauri v2 desktop companion for Indonesian input, structured Japanese speech, Indonesian subtitles, SQLite semantic memory, Style-Bert-VITS2, and plug-and-play RVC profiles.

## Implemented

- Companion, Voice Lab, Voice Browser, Latency, and Settings views.
- OpenAI Responses structured contract: `ja`, `id`, `emotion`, and `intensity`.
- Indonesian recording transcription; microphone audio and keys stay behind Rust IPC.
- SQLite chat, durable memory, embeddings, weighted retrieval, voice settings, and latency samples.
- Separate RAG stores: `memories` contains facts about Haruya, while `ai_knowledge` contains categorized Mizuki character knowledge with canonical-source labels.
- Windows Credential Manager storage for the OpenAI API key.
- Japanese SBV2 synthesis and optional raw-SBV2 fallback.
- Stable RVC discovery with strict one-folder/one-model rules and optional `profile.json`.
- RVC endpoints: `/health`, `/voices`, `/voices/refresh`, `/voices/load`, and `/convert`.

## First run

```powershell
npm install
npm run setup:rvc
npm run services
npm run tauri dev
```

Open **Settings** and save the OpenAI API key. It is stored by Windows Credential Manager, not SQLite or the frontend.

The supplied SBV2 checkout runs on CPU from `TTS/sbv2/Style-Bert-VITS2`. To run on CUDA, remove `--cpu` from `scripts/start-local-services.ps1`.

## Connect an RVC engine

The project includes the profile service and keeps inference engine code replaceable. The original workspace did not contain an RVC v2 runtime, so conversion needs an adapter:

1. Install an RVC v2 runtime once.
2. Copy `rvc-runtime/rvc_backend.example.py` to `rvc-runtime/rvc_backend.py`.
3. Implement `Backend.load` and `Backend.convert` with that runtime's Python API.
4. Restart `npm run services`.

Load shared ContentVec/Hubert and RMVPE assets in `Backend.__init__`. `Backend.load` receives only the selected profile so all checkpoints are never loaded into VRAM.

## Add a voice

```text
voices/
  mizuki/
    mizuki_800e_13600s.pth
    mizuki.index
```

Press **Refresh voices**. No code changes or absolute paths are required. See `voices/README.md` for `profile.json` configuration.

## Environment overrides

- Chat and memory extraction use the hardcoded `gpt-5.6-sol` model.
- `HARUYA_SBV2_URL` (default `http://127.0.0.1:5000`)
- `HARUYA_RVC_URL` (default `http://127.0.0.1:5200`)
- `HARUYA_VOICES_ROOT` (default project `voices/`)
- `OPENAI_API_KEY` (takes precedence over Credential Manager)

Application data is stored in the Tauri app-data directory as `haruya.sqlite3`. Migrations are additive and preserve chat and memory data.

Database schema and seeds live as standalone files under `src-tauri/migrations/`. The Mizuki profile supplied for the project is seeded by `012_seed_mizuki_knowledge.sql`; it keeps official facts, story summaries, and interpretive analysis distinguishable.

## Verify

```powershell
npm run build
cargo check --manifest-path src-tauri/Cargo.toml
npm run tauri build
```

RVC health: `http://127.0.0.1:5200/health`. SBV2 docs: `http://127.0.0.1:5000/docs`.
