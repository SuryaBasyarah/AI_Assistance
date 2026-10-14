# Voice profiles

Create one direct subfolder per character. A valid profile needs exactly one `.pth` checkpoint. One `.index` is strongly recommended.

```text
voices/
  mizuki/
    mizuki_800e_13600s.pth
    mizuki.index
```

If a folder has multiple model or index files, add `profile.json`:

```json
{
  "display_name": "Mizuki",
  "model": "Mizuki_Rvc_Lite_300e_17100s.pth",
  "index": "mizuki_Rvc_Lite.index",
  "language_hint": "ja",
  "pitch": 0,
  "f0_method": "rmvpe",
  "index_rate": 0.75,
  "protect": 0.33,
  "enabled": true
}
```
