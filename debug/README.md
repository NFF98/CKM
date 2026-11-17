# Debug

Log reproducible bugs without personal data.

Issue template:
- Device / Android version / build commit
- Screen ID (S01–S09), steps to reproduce, expected vs actual
- Sanitized sample input and relevant error code
- Severity, suspected layer (Tauri / Rust / local model / SQLite / cloud)
- Fix commit, regression test, verification status

Do not commit raw OCR, private photos, auth tokens, private chats or GPS coordinates.
