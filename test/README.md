# Test

Keep executable automated tests and readable acceptance fixtures here once implementation starts.

Required coverage:
- E2B proposes candidates, but **Rust alone** confirms and writes valid parameters.
- `active matching concepts <= 8` per intent, including concurrent writes.
- Provenance, scope, confirmation, expiry, visibility, and revocation are enforced.
- At most 3 proactive AI questions per activation / matching attempt.
- Unknown intent/poor OCR does not become a guessed fact.
- User cannot see unauthorized other users' intents or chats.
- Real completion needs **both** users to confirm an actual 15-minute session.
- Controlled benchmarks against fixed-form and vector-only matching.

Use synthetic data only; no identifiable screenshots or live user exports.
