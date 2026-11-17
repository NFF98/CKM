# Build — CAKAMI MVP

**Current state: planning, no implementation yet.**

**Start here:** [Build Backlog — B01–B21](BACKLOG.md). This is the single prioritized implementation queue. **There are no assigned Sprints yet.**

## Inputs (canonical design sources)

- [Business Proposal v1.2](../design/doc/CAKAMI_Business_Proposal_v1.2_MVP_Aligned.md)
- [System Design v1.7](../design/doc/CAKAMI_System_Design_v1.7_MVP_8_Parameters.docx)
- [Functional Design v1.8](../design/doc/CAKAMI_MVP_Functional_Design_v1.8_S01-S09_Sequences.docx)

Future code layout, when implementation starts:
- `app/` — Tauri v2 + TypeScript UI
- `core/` — Rust parameter validator, slot guard and question/matching logic
- `mobile/` — native OCR / on-device model bridge
- `cloud/` — API, schema/migrations, consent-based matching/chat

Only implement after checking the relevant screen mapping, privacy boundaries, safety gates and acceptance tests in `BACKLOG.md`. Lock dependency versions and never commit API secrets.
