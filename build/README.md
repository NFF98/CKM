# Build

Production code goes here after the architecture is scaffolded.

Planned subfolders when implementation starts:
- `app/`: Tauri v2 + TypeScript UI and mobile UI
- `core/`: Rust parameter controller and matching/quality logic
- `mobile/`: Android native bridges for local OCR / E2B (only as needed)
- `cloud/`: API, migrations and allowed intent sync

**Sprint plans**:
- [Sprint 01 — Local Intent Core](sprints/SPRINT-01.md)
- [Sprint 02 — Matching to Completed Session](sprints/SPRINT-02.md)

Use lockfiles and pin dependencies when code is added. No secrets in commits.
