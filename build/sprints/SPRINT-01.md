# Sprint 01 — Local Intent Core (S01–S03)

**Goal:** A real Android user can drop a picture/text, have on-device AI interpret it, correct the goal, manage an 8-slot Intent, and save it offline without sharing private data.

**Prerequisite:** target Android test phone; confirm Gemma E2B runtime feasibility. Do not substitute cloud LLM without documenting the privacy and cost implications.

## User journey delivered

S01 Drop image / text → S02 AI understanding / correction → S03 Confirm Intent & opt-in publish permissions (**local only in this sprint**).

## Work items

- [ ] **S1-01** Scaffold Tauri v2 + TypeScript Android application and Rust commands; basic navigation S01–S03.
- [ ] **S1-02** Implement local file picker/camera and text input; store raw content only on the device; deduplicate where practical.
- [ ] **S1-03** Wire native OCR and on-device E2B (or documented device-compatible local inference route). Show extracted content with failure and manual-correction UI.
- [ ] **S1-04** Build **Dynamic Parameter Controller**: source/provenance, scope (profile/intent/pair), candidate vs confirmed state, typed value, visibility, expiry, version, and concept registry.
- [ ] **S1-05** SQLite migrations and repository methods; allow writes only through Rust validation; atomically enforce **8 active matching concepts per intent** and do not drop hard constraints silently.
- [ ] **S1-06** Intent save/edit/cancel/revoke; expired or unconfirmed values cannot act as confirmed matching facts; reuse profile knowledge only when scope/validity permits.
- [ ] **S1-07** Question budget counter: no more than **3 proactive AI questions** per activation/matching attempt, including goal clarification.
- [ ] **S1-08** Synthetic test fixtures: Japanese word `パクる`, JLPT notes, full restaurant ad with unknown intent, blurry OCR, expired deal, duplicate/contradictory time claims.
- [ ] **S1-09** Android test-build + performance measurements (speed, memory, battery, compatibility). Capture a short screen recording using **synthetic** input.

## Key acceptance tests

1. `パクる`: OCR saves the word as content; **0 matching parameters** until the user confirms they want to match.
2. Restaurant ad: store restaurant/ad text locally, do not assume a dine-together Intent; expired coupon is not an active matching fact.
3. E2B cannot create confirmed values or bypass Rust; source, scope, status, lifecycle and consent are checked.
4. 8 validated active concepts accepted; 9th is rejected or explicitly user-replaced; transaction and concurrent-write tests pass.
5. Input that is unclear remains `UNKNOWN` and is editable, never silently made specific.
6. Original private content never leaves device during these flows.
7. Offline save/read/edit works after app restart.

## Definition of Done

- [ ] App launches on target Android device and follows S01–S03.
- [ ] Automated Rust + SQLite validation tests pass.
- [ ] Device performance and extraction quality documented (do not call unmeasured values targets achieved).
- [ ] No real PII or secrets in repo; no unauthorized cloud requests.

**Not in Sprint 1:** online search, real user matching, chat, map, iOS or production release.
