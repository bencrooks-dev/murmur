# Murmur — Architecture (v0 draft)

> Working codename **Murmur**. A self-hostable, end-to-end-encrypted messaging /
> voice / video platform — a "better Hush." This document is the design
> contract; nothing is built yet.

---

## 0. Honest premise

Hush (github.com/hushhq/hush) is **already** built on RFC 9420 (MLS) and
**already** uses AES-256-class ciphersuites (MLS ciphersuites *are* the
encryption layer). Therefore:

- "Add RFC 9420" → already the foundation. Not a differentiator.
- "Add AES-256" → already available as an MLS ciphersuite selection. Not a feature.

To be genuinely **better** than Hush, Murmur differentiates on three axes Hush
is weak on, plus matches everything else:

1. **Post-quantum crypto** — hybrid `X25519 + ML-KEM-768` (X-Wing HPKE) inside
   the MLS ciphersuite. Hush is classical-only today.
2. **Metadata resistance** — sealed sender + traffic padding so the relay learns
   as little as possible about who-talks-to-whom. Hush's server still sees the
   social graph.
3. **Federation, standards-first** — target the IETF **MIMI** effort
   (`draft-ietf-mimi-protocol`), which standardizes federated MLS messaging,
   instead of a bespoke protocol. Hush has federation only on its roadmap.

Everything else (E2EE groups, voice/video via SFU, device linking, key
transparency, self-hosting) we match.

---

## 1. Component topology

Mirrors Hush's 6-repo split (clean ownership boundaries) but with PQ + metadata
hardening baked in from the crypto core up.

```
            ┌─────────────────────────────────────────────┐
            │  CLIENTS                                      │
            │   murmur-web      React + Vite + WASM (PWA)   │
            │   murmur-desktop  Tauri shell (not Electron)  │
            │   murmur-mobile   React Native + WASM/uniffi  │
            └───────────────────────┬─────────────────────-┘
                                     │
                  MLS ciphertext + sealed-sender envelope over WSS
                                     │
            ┌───────────────────────▼─────────────────────-┐
            │  murmur-server    Rust (axum) · Postgres ·    │
            │  relay + storage  Redis · LiveKit SFU adapter │
            │                   + MIMI federation endpoint  │
            └───────────────────────┬─────────────────────-┘
                                     │
            ┌───────────────────────▼─────────────────────-┐
            │  murmur-crypto    Rust · OpenMLS + PQ HPKE    │
            │  (MLS group state)  provider · WASM + uniffi  │
            └─────────────────────────────────────────────-┘

            ┌─────────────────────────────────────────────┐
            │  murmur-directory  Federated discovery +      │
            │                    key-transparency log       │
            └─────────────────────────────────────────────┘
```

### Deltas vs. Hush
- **Server in Rust (axum), not Go.** One language across server + crypto core =
  shared types, no FFI boundary on the server side, easier auditing. (Trade-off:
  smaller hiring pool than Go. Revisit in Phase 0.)
- **Desktop in Tauri, not Electron.** ~10x smaller binary, Rust-native, can link
  the crypto core directly instead of going through WASM. (Trade-off: less mature
  ecosystem than Electron; webview differences across OSes.)
- **Crypto core exposes both WASM (web) and uniffi (mobile/desktop)** bindings
  from one Rust codebase.

---

## 2. The crypto core (`murmur-crypto`) — the heart

### 2.1 MLS base
- Built on **OpenMLS** (Rust). Same proven base as Hush's `hush-crypto`.
- One MLS group per channel. Membership change = MLS Commit. Forward secrecy +
  post-compromise security inherited from RFC 9420.

### 2.2 Post-quantum ciphersuite (the differentiator)
- OpenMLS lets you supply a custom **crypto provider**. We implement a provider
  whose HPKE KEM is **X-Wing** = hybrid `X25519 + ML-KEM-768` (FIPS 203).
  - Hybrid = secure as long as *either* primitive holds → safe against
    "harvest now, decrypt later" without betting everything on PQ maturity.
  - AEAD stays **AES-256-GCM** (or ChaCha20-Poly1305 on platforms without AES-NI).
  - Signatures: keep Ed25519 now; leave a seam for ML-DSA (Dilithium) later.
- Ciphersuite registered in MLS's private-use range until a standard PQ suite
  lands. **Document this as a forward-compat risk** (see CORE-INVARIANTS).

### 2.3 Exporter-derived media keys
- Voice/video SRTP keys derived from the MLS **exporter secret** (same pattern as
  Hush). The SFU only ever sees opaque SRTP. PQ protection flows through for free
  because the exporter secret is PQ-protected upstream.

### 2.4 Device linking
- Per-account device-linking ceremony hands a new device a **sealed bundle**
  (history snapshot + transcript + key material) over a chunked encrypted relay.
  No plaintext at the server. (Matches Hush; this is table stakes.)

---

## 3. Metadata resistance (differentiator #2)

Hush encrypts payloads but the relay still observes sender→recipient routing.
Murmur reduces that:

- **Sealed sender:** outer envelope addressed to the *group*, not a named sender;
  sender identity is inside the MLS ciphertext. Relay routes on group ID only.
- **Length padding:** pad ciphertext to power-of-two buckets so message size
  leaks nothing about content type (text vs. media negotiation).
- **Decoupled delivery receipts** so timing correlation is weaker.
- **Explicitly NOT solved:** full traffic-analysis resistance (timing, volume) —
  that needs mixnet-grade infra. We document the threat boundary, not pretend.

---

## 4. Federation (differentiator #3) — standards-first

- Target IETF **MIMI** (`draft-ietf-mimi-protocol`): it layers federated transport
  + room policy on top of MLS, which is exactly our base.
- Each instance owns its users; cross-instance rooms are MLS groups whose members
  span servers. Servers exchange MLS messages + MIMI control over an
  authenticated S2S channel.
- `murmur-directory` holds the **key-transparency log** (append-only, verifiable)
  so a returning user can confirm their own device-key history wasn't tampered
  with — and, federated, that a remote server isn't lying about a user's keys.
- **Phase it late.** Federation multiplies the threat model; ship single-instance
  first, design the data model to not preclude it.

---

## 5. Server (`murmur-server`)

- **axum** (Rust) HTTP/WSS relay. Stateless where possible; group/membership
  state in Postgres, ephemeral fanout state in Redis.
- Stores **only ciphertext + routing metadata** (group ID, ordering, blob refs).
- LiveKit SFU adapter for voice/video; server holds no media keys.
- Admin dashboard for instance operators (mirrors Hush's `/admin/`).
- Self-host path: Docker Compose (axum + Postgres + Redis + LiveKit + Caddy),
  one-command `setup.sh` like Hush.

---

## 6. Clients

| Client | Stack | Notes |
|---|---|---|
| `murmur-web` | React + Vite, crypto via **WASM** | PWA, the reference client |
| `murmur-desktop` | **Tauri** wrapping the web bundle | links crypto core natively |
| `murmur-mobile` | React Native, crypto via **uniffi** | ship it (Hush only plans to) |

All clients share `murmur-crypto`. The crypto core is the single source of MLS
truth; clients never reimplement protocol logic.

---

## 7. Core invariants (must hold across all repos)

1. **Server never holds key material.** No exceptions, including device-linking
   relay and media keys.
2. **All protocol logic lives in `murmur-crypto`.** Clients are dumb shells over it.
3. **PQ is hybrid, never PQ-only** (until standards + maturity justify otherwise).
4. **Federation changes are gated** against the metadata + key-transparency model.
5. **Ciphertext + minimal routing metadata only** in server storage.

See `CORE-INVARIANTS.md` (to be written in Phase 0) for the enforceable checklist.

---

## 8. Known risks / open questions (resolve in Phase 0)

- **Non-standard PQ ciphersuite** → interop + audit risk. Mitigate: isolate in
  provider, track the IETF PQ-MLS draft, be ready to swap.
- **Rust server vs. Go** → hiring + ecosystem. Decision needed before Phase 1.
- **Tauri webview fragmentation** across OSes → desktop QA cost.
- **Audit budget.** New crypto = mandatory third-party audit before any "secure"
  marketing claim. Non-negotiable, costs real money.
- **AGPL-3.0** (Hush's license). Inherit it? Affects commercial strategy.
- **This is a multi-month, multi-person platform.** Solo MVP must ruthlessly cut
  to: web client + single-instance server + PQ crypto core. Everything else later.
