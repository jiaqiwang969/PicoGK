# api-gap

Heuristic C# -> Rust API gap reporter for PicoGK.

Prereqs:
- `picogk-rs/CSHARP_PUBLIC_API.md` exists (generate via `tools/api-dump`)

Run:
```bash
python3 tools/api-gap/api_gap.py
```

Output:
- `picogk-rs/API_GAP_REPORT.md`

