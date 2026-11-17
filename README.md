# CAKAMI (CKM) — MVP Workspace

CAKAMI = **Drop Anything → Discover People → Accomplish Together.**

**Phase 1 (MVP):** adult JLPT N2 Study Match on Android first. A user drops a note/image, confirms intent, authorizes minimal sharing, finds a real study partner, and completes a 15-minute text study session.

> Status: **design / planning**, not a released product. The existing business design is a hypothesis requiring validation.
> Repository is **public**: never commit real user images, personal data, API keys, service-role keys, or private research.

## Repository map

| Folder | Purpose |
|---|---|
| [design/](design/) | Approved product/system specifications. [doc/](design/doc/) contains the latest business proposal plus two user-uploaded system/functional designs. |
| [build/](build/) | Implementation workspace and a dependency-ordered **Build Backlog**; sprint planning is deferred. |
| [test/](test/) | Automated tests, acceptance scenarios, benchmark cases. |
| [debug/](debug/) | Defect reports, reproduction guides, root-cause notes (sanitize sensitive data). |
| [deploy/](deploy/) | Android packaging, CI/CD, release checklist, rollback guidance. |
| [communication/](communication/) | Decisions, release notes, product updates, handover notes. |

## Architecture to implement

Tauri v2 + TypeScript UI → Rust Core → local Gemma E2B / OCR → SQLite. With explicit consent, publish minimal searchable intent to a cloud service (planned: Supabase/Postgres + pgvector) for matching, invitations, and chat. AI proposes data; Rust validates it. No raw private photos or notes sent to the cloud for normal local processing.

**Hard MVP rules:** at most **8 active matching parameter concepts per intent**; at most **3 proactive AI questions per intent activation / matching attempt**; known source/scope/confirmation/lifetime; no unapproved cloud publication; no silent deletion of hard constraints.

## Plan build first; assign sprints later

**Current single build queue:** [Phase 1 MVP Build Backlog — B01–B21](build/BACKLOG.md).

**Current design sources:** [Business v1.2](design/doc/CAKAMI_Business_Proposal_v1.2_MVP_Aligned.md), [System v1.7](design/doc/CAKAMI_System_Design_v1.7_MVP_8_Parameters.docx), [Functional v1.8](design/doc/CAKAMI_MVP_Functional_Design_v1.8_S01-S09_Sequences.docx).

Earlier Sprint 01/02 plans and Issues #1/#2 have been retired; scope will be grouped into sprints only after backlog review.
