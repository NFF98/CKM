# CAKAMI (CKM) — MVP Workspace

CAKAMI = **Drop Anything → Discover People → Accomplish Together.**

**Phase 1 (MVP):** adult JLPT N2 Study Match on Android first. A user drops a note/image, confirms intent, authorizes minimal sharing, finds a real study partner, and completes a 15-minute text study session.

> Status: **design / planning**, not a released product. The existing business design is a hypothesis requiring validation.
> Repository is **public**: never commit real user images, personal data, API keys, service-role keys, or private research.

## Repository map

| Folder | Purpose |
|---|---|
| [design/](design/) | Approved product/system specifications. [doc/](design/doc/) contains **only the latest Business Proposal**. |
| [build/](build/) | Implementation, source code, Sprint 1 and Sprint 2 plans. |
| [test/](test/) | Automated tests, acceptance scenarios, benchmark cases. |
| [debug/](debug/) | Defect reports, reproduction guides, root-cause notes (sanitize sensitive data). |
| [deploy/](deploy/) | Android packaging, CI/CD, release checklist, rollback guidance. |
| [communication/](communication/) | Decisions, release notes, product updates, handover notes. |

## Architecture to implement

Tauri v2 + TypeScript UI → Rust Core → local Gemma E2B / OCR → SQLite. With explicit consent, publish minimal searchable intent to a cloud service (planned: Supabase/Postgres + pgvector) for matching, invitations, and chat. AI proposes data; Rust validates it. No raw private photos or notes sent to the cloud for normal local processing.

**Hard MVP rules:** at most **8 active matching parameter concepts per intent**; at most **3 proactive AI questions per intent activation / matching attempt**; known source/scope/confirmation/lifetime; no unapproved cloud publication; no silent deletion of hard constraints.

## Build order

1. [Sprint 01 — Local Intent Core](build/sprints/SPRINT-01.md)
2. [Sprint 02 — Mutual Matching & Completion](build/sprints/SPRINT-02.md)

**Canonical business proposal:** [v1.2 MVP Aligned](design/doc/CAKAMI_Business_Proposal_v1.2_MVP_Aligned.md). Previous versions deliberately not committed.
