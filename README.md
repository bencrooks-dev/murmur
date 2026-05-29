<div align="center">

# Murmur

**Self-hostable, end-to-end-encrypted messaging — engineered to beat Hush on every axis.**

Post-quantum-ready MLS · metadata-resistant relay · standards-first federation · mobile from day one

[![License: AGPL-3.0](https://img.shields.io/badge/license-AGPL--3.0-3dd4b8)](./LICENSE)
[![Crypto core](https://img.shields.io/badge/crypto-OpenMLS%20(RFC%209420)-3dd4b8)](https://github.com/openmls/openmls)
[![Status](https://img.shields.io/badge/status-early%20%E2%80%94%20not%20audited-e0a33e)](#security--status)

</div>

---

> ### ⚠ Status: early, and **not yet security-audited**
> Murmur is under active development. The cryptographic core works and is tested,
> but **no third-party audit has been performed**. Do **not** use Murmur to protect
> real secrets until an independent cryptographic audit has passed. This is research
> and engineering in the open — not a finished product.

Murmur is a "better Hush": the same RFC 9420 (MLS) foundation, deliberately
differentiated on the four axes Hush is weak on.

| Axis | Hush | Murmur |
|------|------|--------|
| E2EE group messaging (MLS) | ✅ | ✅ working, networked |
| **Metadata resistance** | server sees the social graph | relay routes on group id only, never reads payloads |
| **Post-quantum crypto** | classical X25519/AES | hybrid **X25519 + ML-KEM** ciphersuite *(architecture set; implementation is the next phase)* |
| **Federation** | roadmap only | standards-first via IETF **MIMI** *(planned)* |
| **Mobile** | planned only | shared Rust core with **uniffi** bindings, day one *(seam proven)* |

See [`docs/ROADMAP.md`](./docs/ROADMAP.md) for exactly what is built vs. planned.

## What works today (verified)

| Component | What it owns | Verification |
|-----------|--------------|--------------|
| [`murmur-crypto`](./murmur-crypto) | MLS core over OpenMLS — `Identity`, `Group`, `Account` (create/add/join/send/receive/remove, exporter-derived media keys) | `cargo test` — 5 passing |
| [`murmur-wasm`](./murmur-wasm) | The core compiled to WebAssembly for the browser | runs a full E2EE round-trip under Node |
| [`murmur-server`](./murmur-server) | axum WebSocket relay — key-package directory, Welcome inbox, opaque ciphertext routing (sealed sender) | `cargo test` — 2 passing |
| [`murmur-web`](./murmur-web) | React + Vite client on a strict dark design system, wired to the WASM core | type-checks + production-builds; live two-browser E2EE chat |
| [`murmur-uniffi-spike`](./murmur-uniffi-spike) | Proof that one Rust core generates Kotlin + Swift bindings | generates both |

A Node end-to-end test ([`murmur-wasm/test_e2e_relay.js`](./murmur-wasm/test_e2e_relay.js))
drives two independent clients through the **live relay**: they register, exchange
key packages, deliver a Welcome, and send encrypted messages both ways.

## Architecture

```
            ┌──────────────────────────────────────────────┐
            │  CLIENTS                                       │
            │   murmur-web      React + Vite + WASM          │
            │   murmur-desktop  Tauri      (planned)         │
            │   murmur-mobile   React Native + uniffi (planned)
            └───────────────────────┬──────────────────────┘
                                     │  MLS ciphertext + sealed-sender over WSS
            ┌───────────────────────▼──────────────────────┐
            │  murmur-server   Rust · axum                   │
            │  relay + directory + Welcome inbox             │
            │  (Postgres + Redis + LiveKit planned)          │
            └───────────────────────┬──────────────────────┘
            ┌───────────────────────▼──────────────────────┐
            │  murmur-crypto   Rust · OpenMLS                │
            │  WASM (web) + uniffi (mobile/desktop)          │
            └───────────────────────────────────────────────┘
```

Design docs: [`ARCHITECTURE`](./docs/ARCHITECTURE.md) ·
[`ROADMAP`](./docs/ROADMAP.md) · [`DESIGN-LANGUAGE`](./docs/DESIGN-LANGUAGE.md) ·
[`PHASE-0-FINDINGS`](./docs/PHASE-0-FINDINGS.md)

## Quick start (local)

Prerequisites: **Rust** (MSVC toolchain on Windows), the `wasm32-unknown-unknown`
target, `wasm-bindgen-cli`, and **Node 20+**.

```bash
# 1. crypto core tests
cd murmur-crypto && cargo test

# 2. build the WASM core + regenerate web bindings
cd ../murmur-wasm
cargo build --target wasm32-unknown-unknown --release
wasm-bindgen --target web --out-dir ../murmur-web/src/crypto \
  target/wasm32-unknown-unknown/release/murmur_wasm.wasm

# 3. run the relay (terminal A)
cd ../murmur-server && cargo run            # ws://0.0.0.0:8787/ws

# 4. run the web client (terminal B)
cd ../murmur-web && npm install && npm run dev
```

Open the printed URL, connect as `alice`, open a second tab as `bob`, and start a
chat. Messages encrypt in the browser; the relay only ever sees ciphertext.

### Use it from your phone (same Wi-Fi)
Run the dev server bound to your LAN and allow the ports through your firewall:

```bash
cd murmur-web && npm run dev -- --host 0.0.0.0
```
Then browse to `http://<your-computer-LAN-ip>:5173/` on your phone. (There is no
native app yet — mobile clients are a planned phase.)

## Security & status

- **One MLS group per channel.** Membership changes are MLS commits; forward
  secrecy and post-compromise security follow from RFC 9420.
- **The server never holds keys** and never inspects ciphertext — it routes on
  group id only (sealed sender).
- **Not audited.** See the banner at the top. Report security issues privately;
  do not open public issues for vulnerabilities.

## Roadmap (high level)

`P0 de-risk ✅ → P1 crypto core ✅ → P3 relay ✅(MVP) → P4 web ✅(MVP) →`
**`P2 post-quantum ciphersuite`** `→ P5 voice/video → P6 desktop (Tauri) →`
`P7 federation (MIMI) → P8 third-party audit + launch`

Full detail and per-task status in [`docs/ROADMAP.md`](./docs/ROADMAP.md).

## Contributing

Early-stage. Cross-cutting proposals (protocol, federation, architecture) and
implementation PRs are welcome. Keep the core invariants intact: the server holds
no keys, all protocol logic lives in `murmur-crypto`, and PQ is always hybrid.

## License

[AGPL-3.0](./LICENSE) — matching the project it sets out to improve on.

---

<sub>Built in the open. Not affiliated with Hush.</sub>
