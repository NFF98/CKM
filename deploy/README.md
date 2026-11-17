# Deploy

**Phase 1 target:** Android first through Tauri v2. iOS and desktop are later targets.

Release gate:
1. All P0 parameter integrity + privacy + auth tests pass.
2. Local E2B model compatibility, battery, memory and latency checked on target phones.
3. Backend access policies and rate limits verified; no service-role key in client.
4. Test mutual acceptance, chat, revocation, expiry and complete-session flow end-to-end.
5. Version/tag, signed artifact and rollback notes recorded.

No production project or deployment has been created yet.
