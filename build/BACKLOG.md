# CAKAMI — Phase 1 MVP Build Backlog

**Status:** PLANNING · no code committed · **no sprint allocation yet**  
**Updated:** 2026-11-17  
**Goal:** 18+ JLPT N2 Android Study Match, **S01–S09** and the S04a no-match branch.

## Source of truth

Use these approved source documents **together**; do not implement from deleted sprint plans:

1. [Business Proposal v1.2](../design/doc/CAKAMI_Business_Proposal_v1.2_MVP_Aligned.md) — market, scope, validation and business constraints.
2. [System Design v1.7](../design/doc/CAKAMI_System_Design_v1.7_MVP_8_Parameters.docx) — Local-first, Rust parameter integrity, 8-slot and algorithm specification.
3. [Functional Design v1.8](../design/doc/CAKAMI_MVP_Functional_Design_v1.8_S01-S09_Sequences.docx) — F01–F09 / S01–S09 screen map, outside and inside sequence diagrams.

**Technology intent:** Tauri v2 + TypeScript UI → Rust Controller → on-device OCR / Gemma E2B candidate → local SQLite; with *explicit consent*, minimal authorized Intent data to a cloud backend (planned: Supabase Auth/Postgres/pgvector + realtime chat). Validate the target-device inference runtime **before committing to its performance**.

### Product invariants — apply to every backlog item

- **Drop is not intent:** recognizing `パクる` or a restaurant ad must not automatically publish an invitation.
- **No fabricated facts:** OCR/model observations ≠ user-confirmed Goal, Need, Offer, time or permission.
- **8 active matching concepts** per Intent (slots 0–7), not 8 images, words, rows or global fields. Unknown values used for matching occupy a slot. **No silent replacement of hard constraints**; 9th necessary constraint blocks publish until resolved.
- **Question budget:** ≤3 proactive questions per Intent activation / matching attempt, including Goal clarification; pair-only ≤2; daily unsolicited follow-ups ≤3.
- **Rust is sole trusted local write gate**; SQLite constraints are the second gate; server validates authorized published data again.
- **Local by default:** raw images, OCR, private notes, full parameter histories and user profile stay local. Share only minimal user-approved matching information. Embeddings are not assumed anonymous.
- **MVP = real human session**: 15-minute text session completed and **confirmed by both people**; not just match/chat/click.
- **Safety:** 18+ pilot, consent before publishing, mutual acceptance before chat; block/report; expiry/revocation stop further matching.
- Map/Attraction is **Phase 2**; Travel is Phase 3. No Phase 1 location tracking, booking, creature game or marketplace.

## Build backlog (ordered by dependencies, not by calendar)

Status convention: `[ ]` not started · `[x]` verified complete. `P0` blocks the adult MVP; `P1` improves evaluation but must not displace P0. Items are **not** assigned to Sprint 1/2 yet.

| ID | Priority | Functional deliverable | Screen | Depends on | Acceptance (user-visible or testable) |
|---|---|---|---|---|---|
| **B01** | P0 | Tauri v2 Android + TypeScript + Rust scaffold; S01–S09 navigation placeholders | All | — | App runs on target Android phone; commands bridge UI↔Rust; no cloud dependency for local screens |
| **B02** | P0 | Drop photo, note or text; private asset storage | S01 | B01 | Local image/text accessible after restart; no raw input uploaded |
| **B03** | P0 | On-device OCR and E2B inference adapter; manual correction/failure handling | S01–S02 | B01–B02 | `パクる` recognized as content; unreadable photos stay UNKNOWN rather than hallucinating |
| **B04** | P0 | Separate observed content from explicit goal; editable Intent Draft | S02 | B03 | Word lookup/restaurant ad generates **0 publishable matching parameters** until user confirms purpose |
| **B05** | P0 | Concept Registry + Rust validation (source, scope, type, user-confirmed state, lifecycle, version, privacy) | S02–S03 / S06 | B04 | Unknown names are private proposals; synonyms reuse concepts; model cannot forge CONFIRMED |
| **B06** | P0 | SQLite schema + transactional 8-slot guard + evidence and confirmation events | S03 / S06 | B05 | Slot 0–7 enforced; duplicate concept blocked; concurrent 9th rejected; safety constraint cannot be silently dropped |
| **B07** | P0 | Local Intent list/edit/revoke/expiry and scope reuse | S03 | B06 | One-off availability not elevated to permanent profile; expired intent not matchable; offline changes persist |
| **B08** | P0 | Consent UI + audit events + share preview | S03 | B07 | User sees which facts will be shared; default private; no publish without explicit consent |
| **B09** | P0 | Safe question proposal/selection with question budgets | S02 / S06 | B05–B07 | Ask only for decision-relevant UNKNOWNs, ≤3 questions per attempt, pair-only ≤2, can skip when sufficient |
| **B10** | P0 | Adult-user authentication, abuse controls, account and access model | Cloud | B01 | Authenticated users isolated; 18+ eligibility step and block/report policy |
| **B11** | P0 | Cloud intent, candidate, invitation, chat and outcome schema; server validation + RLS | S03–S09 | B06, B10 | No access to unrelated private data; server rejects expired/revoked/invalid 8-slot payloads |
| **B12** | P0 | Consent-gated publish/sync, revoke and expiry retraction | S03 / S04 | B08, B11 | Publish only minimal authorized data; revoke disables matching immediately locally, cloud retract status observable |
| **B13** | P0 | Multi-path candidate search: same Goal and complementary Need↔Offer; constraint filter | S04 | B11–B12 | Find real eligible users; hard mismatches excluded even when vector similarity is high |
| **B14** | P0 | Candidate ranking + explanation + S04a honest empty state | S04–S05 | B13 | Show only actual candidates and reasons; no fake matches, no invented success probability |
| **B15** | P0 | Pair-specific clarifying answers, re-check and rerank | S06 | B09, B14 | Missing decisive condition → one relevant question → rerank; answer kept pair-scoped with expiry |
| **B16** | P0 | Invite / decline / mutual acceptance; block and report | S07 | B14–B15 | Chat impossible until **both** accept; refusal/time-out respected |
| **B17** | P0 | Consent-based text chat and 15-minute study session | S08 | B16 | Two accepted users can chat and run a 15-minute study flow; basic abuse/rate limiting |
| **B18** | P0 | Two-sided completion/failed/no-show outcome | S09 | B17 | Completion recorded only from valid two-sided confirmation; canceled/no-show not counted as success |
| **B19** | P0 | Automated validation, authorization, end-to-end and device tests | S01–S09 | B01–B18 | All 13 Parameter Integrity cases from v1.7 plus auth/chat/revoke/expiry/E2E cases pass |
| **B20** | P1 | Matching benchmark: fixed form vs vector-only vs CAKAMI under the same pool | S04–S09 | B13–B18 | Report completed sessions per 100 valid intents, question count, constraint violations, and cost/session |
| **B21** | P0 | Android pilot build, security/privacy review, deploy/rollback runbook | Release | B19; B20 report | Signed internal test build; documented E2B latency/memory/battery and privacy checks; P0 release gate passed |

## Work order — what should be built first?

**Foundation (offline, no social backend):** B01 → B02 → B03 → B04 → B05 → B06 → B07 → B08.  
**Question capability:** B09 after parameter validation; it must not bypass Rust.  
**Secure matchmaking:** B10–B15 after local integrity and privacy are working.  
**Real outcome:** B16–B18.  
**Quality and pilot:** B19 → B20 → B21.

A build item moves from `[ ]` to `[x]` **only when acceptance evidence exists** (test, screenshot from synthetic input, PR/commit, or device result). Do not mark scaffolding as a working MVP.

## Baseline acceptance scenarios

| Scenario | Expected |
|---|---|
| Japanese word `パクる` dropped alone | OCR/content saved locally; no invented matching goal |
| Restaurant advertisement without an explicit goal | Content saved; no restaurant partner match; expiry applies to coupon, not automatically to the owner's future restaurant interest |
| E2B outputs `status=CONFIRMED` | Rust rejects forged confirmation without UI event |
| Eight concepts already allocated; 9th proposed | Validation stops publication unless user explicitly resolves slot budget |
| 100 vocabulary screenshots dropped | Vocabulary data grow, but matching concepts and SQL columns do not explode |
| Candidate has great semantic similarity but conflicting time/permission | Hard constraint excludes the pairing |
| A declines or blocks B | No chat; never count a completed match |
| Both complete a 15-minute practice | Count exactly one verified completed session |
| Device offline or user retracts consent | Local usable offline; revoked intent unavailable locally; cloud retraction queued and status visible |

## Scope and governance

- **Backlog first, Sprints later.** Prior `build/sprints/SPRINT-01.md` and `SPRINT-02.md` are removed. GitHub Issues #1/#2 were closed as superseded.
- Maintain this file as the single **build prioritization list**. Do not create a new sprint plan until the user agrees on task grouping and effort.
- Use GitHub issues for individual work items only when beginning implementation; reference `Bxx` in PRs/test logs.
- Source documents in `design/doc/` are not to be changed or deleted as part of backlog planning.
