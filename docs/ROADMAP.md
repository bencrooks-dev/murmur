# Murmur — Roadmap (v1 — "win every metric, mobile day one")

Mandate (set 2026-05-29): beat Hush on **all four** axes, not some:

| Axis | Hush | Murmur target |
|---|---|---|
| Post-quantum crypto | Classical X25519/AES | **Hybrid X25519 + ML-KEM-768** (X-Wing HPKE) |
| Metadata privacy | Server sees the social graph | **Sealed sender + padding** from MVP |
| Federation | Roadmap only | **MIMI** (`draft-ietf-mimi-protocol`) — must ship |
| Mobile | Planned only | **RN client in the MVP slice — day one** |

Consequence of "mobile day one": the crypto core exposes **WASM (web) + uniffi
(iOS/Android/desktop) from Phase 1**. No client ever reimplements protocol logic.

Principle unchanged: every phase ships something verifiable; the spine is a thin
vertical slice carried all the way to a phone.

---

## Phase 0 — Decide & de-risk (no product code) ← IN PROGRESS
**Goal:** kill the unknowns that would invalidate later work.
See `docs/PHASE-0-FINDINGS.md` for full results.
- [x] Install toolchain: Rust (rustc/cargo 1.96). **Switched to GNU toolchain** —
      no MSVC linker on this box; gnu bundles its own. wasm-pack + Docker still TODO.
- [x] **Spike A:** OpenMLS 2-member group, send/receive a message. ✅ PASS
      (`murmur-crypto/tests/spike_a_two_member_group.rs`).
- [x] **Spike B:** custom OpenMLS crypto provider seam. ✅ PASS
      (`murmur-crypto/tests/spike_b_custom_provider.rs`). **Caveat:** OpenMLS 0.6
      `Ciphersuite` is a CLOSED enum → a new PQ ciphersuite needs a fork or
      upstream support. This is the #1 PQ risk; decide before Phase 2.
- [x] **Spike C:** uniffi binding seam. ✅ PASS on MSVC — one Rust core generated
      real Kotlin (Android) + Swift (iOS) bindings (`murmur-uniffi-spike/bindings/`).
      Device cross-compile still needs Android NDK / macOS (deferred).
- [x] Lock decisions: PQ → **fork OpenMLS**; iOS → **macOS CI**; Windows toolchain
      → **MSVC**. (Server lang / desktop shell / license / audit budget deferred to P1.)
- [ ] Write `CORE-INVARIANTS.md` + `THREAT-MODEL.md`.  ← only remaining P0 item
- **Exit:** ✅ all three spikes green, core decisions recorded. (Invariants doc pending.)

## Phase 1 — Crypto core, multi-target (`murmur-crypto`) ← IN PROGRESS
**Goal:** the protocol heart, classical first, **bound for both web and mobile**.
Sequencing note: OpenMLS fork deferred to the P1→P2 boundary (only needed for the
PQ ciphersuite); P1 builds the real API against upstream OpenMLS.
- [x] Real public API over OpenMLS: `Identity` (generate, key_package) + `Group`
      (create / add_member / join / send / receive / exporter_secret / member_count).
      `src/lib.rs`; flattened `MurmurError`; integration test `tests/group_api.rs`
      drives the full lifecycle incl. matching exporter media keys. ✅ 4/4 tests pass.
- [x] `Account` stateful surface (owns provider/identity/groups) + **member removal**.
      Tests `tests/account_api.rs`. ✅
- [x] **WASM bindings** (`murmur-wasm`, wasm-bindgen) — built to wasm32 and
      **verified running under Node** (`test_node.js`): E2EE round-trip, matching
      exporter media keys, removal. ✅ (Needed `openmls`/`getrandom` `js` features.)
- [ ] Handle proposal/commit fan-out beyond the happy path (multi-device ordering).
- [ ] **uniffi bindings** of the real `Account` API (seam proven in Spike C; wrap like WASM).
- [ ] RFC 9420 test-vector conformance suite.
- [ ] Persistent storage provider (replace in-memory).
- **Exit:** the same core drives a browser tab AND a RN dev build exchanging
  classical E2EE messages. Mobile parity proven before PQ.

## Phase 2 — PQ ciphersuite (differentiator #1)
- [ ] X-Wing HPKE provider (`X25519 + ML-KEM-768`), AES-256-GCM AEAD.
- [ ] Register as private-use MLS ciphersuite; negotiate at group creation.
- [ ] Full PQ cycle: create → add → message → remove.
- [ ] Benchmark on **mobile** too (PQ keys are big; phones are the constraint).
- **Exit:** PQ-protected group works end-to-end on web AND phone; perf documented.

## Phase 3 — Server MVP (`murmur-server`) + metadata resistance (differentiator #2) ← STARTED
- [x] axum **WebSocket relay**, in-memory group→subscriber fanout. Integration test
      proves opaque payload routing between two WS clients, no sender echo. (`cargo run`
      → ws://0.0.0.0:8787/ws.) Docker/Postgres/Redis deferred.
- [x] **Sealed sender** (basic) — relay routes on group id only, never inspects body.
- [ ] Key-package **directory** + per-user **Welcome inbox** (needed for real 2-client
      handshake over the wire — the next increment that connects murmur-web to the relay).
- [ ] Auth, registration, device-key registration.
- [ ] Length padding to power-of-two buckets.
- [ ] Redis fanout; Postgres for accounts/groups/membership/refs (durable history).
- [ ] **Length padding** to power-of-two buckets.
- [ ] Invariant check: ciphertext + minimal routing metadata ONLY in storage.
- **Exit:** two clients (one of them a phone) exchange PQ-E2EE messages through
  the server, relay cannot see who sent what.

## Phase 4 — Clients in parallel: web + mobile (mobile day one) ← STARTED
**Goal:** ship `murmur-web` AND `murmur-mobile` together.
- [~] `murmur-web` SCAFFOLDED: React + Vite + TS over the real WASM core, on the
      locked dark design system (tokens, app shell, secure DM, live in-browser MLS
      encryption, ciphertext inspector, exporter-derived channel fingerprint).
      Type-checks + production-builds clean (`npm run build`). Run: `npm run dev`.
      TODO: server-backed channels, membership UI, device linking, key-transparency,
      real multi-user (not the local 2-account demo).
- [ ] `murmur-mobile`: React Native over uniffi core; same feature set; encrypted
      push payloads; background key handling.
- [ ] Shared UI logic / design system across both where practical.
- **Exit:** usable 1:1 + group text chat, PQ by default, on web **and** iOS+Android.

## Phase 5 — Voice / video
- [ ] LiveKit SFU; SRTP keys from MLS exporter secret; SFU sees opaque SRTP only.
- [ ] Calls work on web + mobile.
- **Exit:** E2EE group call on a phone; SFU never holds media keys.

## Phase 6 — Desktop (`murmur-desktop`)
- [ ] Tauri shell over the web bundle; crypto core linked natively (uniffi, no WASM).
- [ ] Signed Mac / Windows / Linux builds + auto-update.
- **Exit:** desktop parity with web.

## Phase 7 — Federation (differentiator #3) — MUST ship (`murmur-directory` + S2S)
- [ ] Implement MIMI (`draft-ietf-mimi-protocol`) transport + room policy.
- [ ] Cross-instance MLS groups over an authenticated S2S channel.
- [ ] Federated key-transparency log in `murmur-directory`.
- **Exit:** two independent instances share one E2EE room, verifiable keys.

## Phase 8 — Harden & launch
- [ ] **Third-party crypto audit** — hard gate before any "secure" claim.
- [ ] One-command self-host (`setup.sh`, Docker Compose, Caddy TLS).
- [ ] Admin dashboard, runbook, transparency-log docs.
- [ ] License decision applied (AGPL-3.0 or alternative).
- **Exit:** publicly self-hostable, audited, documented — better than Hush on all 4 axes.

---

## Critical path
`P0 → P1 (web+mobile bindings) → P2 (PQ) → P3 (server+metadata) → P4 (web+mobile clients)`
is the spine and carries mobile the whole way. Voice (P5), desktop (P6),
federation (P7) hang off a working P4 and parallelize across people.

## Reality flags (unchanged + sharpened)
- **Mobile day one roughly doubles client effort** and forces uniffi correctness
  early. It's the right call for "beat Hush," but it is the biggest cost driver.
- **PQ on phones:** ML-KEM keys/ciphertexts are kilobytes; measure commit cost on
  real devices in P2, not just desktop.
- **Federation is genuinely hard** and multiplies the threat model — but it's a
  required win-metric, so it's a committed phase, not "someday."
- **Audit is the real gate** to calling this secure. Budget for it early.
- **This is a multi-person, multi-month platform.** Solo → P0–P4 dominate.

## Next action
Finish Phase 0: run the scaffolded OpenMLS spike once Rust finishes installing.
