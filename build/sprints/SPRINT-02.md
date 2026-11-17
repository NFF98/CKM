# Sprint 02 — Matching → Mutual Consent → Completed Study (S04–S09)

**Goal:** Two authenticated adult users can discover a compatible JLPT N2 study partner using minimal opt-in data, clarify up to three AI questions per attempt, mutually accept, chat, complete a 15-minute session, and confirm the outcome.

**Dependency:** Sprint 1 local parameter integrity is complete. The cloud is used for matchmaking and communications, not raw image OCR/interpretation.

## User journey delivered

S03 explicit share permission → S04 publish/search → S05 candidates → S06 critical clarification → S07 mutual invite → S08 timed text study/chat → S09 both confirm completed session.  
**S04a:** no real candidates → honest empty state; no fake users or AI pretending a human exists.

## Work items

- [ ] **S2-01** Authentication with **18+ eligibility gate** and privacy/abuse controls; opt-in consent audit trail.
- [ ] **S2-02** Cloud schema and backend API: minimum publishable intents, typed parameters, embeddings, candidate link, invitations, session, messages and outcomes. Enforce server-side validation; configure strict row-level access policies.
- [ ] **S2-03** Local-to-cloud sync only after explicit permission; no original photos/notes; revoke/expire must retract active cloud searchability.
- [ ] **S2-04** Candidate retrieval: goal↔goal and need↔offer searches, filter hard constraints, dedupe, rank with feasible reciprocal match; track baseline comparison.
- [ ] **S2-05** Question selector: ask the highest-value missing question rather than a fixed long form; enforce 3-question budget per activation / matching attempt.
- [ ] **S2-06** S04/S04a/S05/S06 screens: honest empty state, safe candidate cards and explicit explanation of why candidate could work.
- [ ] **S2-07** S07 invite + accept on both sides, decline, block/report and no chat before bilateral consent.
- [ ] **S2-08** S08 text chat + timer for 15-minute study; abuse reporting, rate limiting and reasonable safety defaults.
- [ ] **S2-09** S09 two-sided completion confirmation + cancellation/no-show statuses; completion metric cannot be inferred from just messages.
- [ ] **S2-10** E2E tests and benchmark: fixed form vs vector only vs multi-vector+hard constraints vs dynamic 8-slot; same candidate pool. Record completions/100 active intents, question count, latency, cloud cost per completed session.
- [ ] **S2-11** Android QA release candidate, deployment/rollback checklist, privacy/security signoff, pilot with real consenting adults.

## Key acceptance tests

1. No publish without consent; permission revocation stops matching and retracts searchable content.
2. Server independently rechecks 8-slot/metadata/expiration, mutual acceptance and access rights.
3. Never leak private note/photos or unrelated parameter history in cloud results.
4. Not-matchable due to hard constraints/expiry is excluded even if vector similarity is high.
5. At most three AI questions, and pair-specific answers never silently become lifetime preferences.
6. No chat before bilateral acceptance; report/block functions work.
7. A successful Session requires **both participants' completion confirmation** of an actual 15-minute activity.
8. Empty candidate pool shown honestly; test user density/cold-start handling.
9. Sprint 1 regression suite still passes.

## Definition of Done

- [ ] S01–S09 and S04a work end-to-end with two real test accounts.
- [ ] P0 privacy, access, parameter and lifecycle tests pass with real backend policies.
- [ ] Baseline benchmark results, device performance and cost assumptions are recorded.
- [ ] No claim of achieved network effects until real user evidence exists.

**Out of scope:** Map/Attraction (Phase 2), Travel (Phase 3), general-purpose marketplace, dating, iOS initial release.

**Scope caution:** This is a full feature sprint for planning, not a guaranteed two-week solo-founder deadline. If needed, subdivide tasks without changing the Phase 1 acceptance criteria.
