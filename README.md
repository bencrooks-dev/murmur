<div align="center">

# Murmur

**Self-hostable, end-to-end-encrypted messaging, voice, and video — built to beat Hush on every axis.**

Post-quantum MLS · metadata resistance · standards-first federation · mobile day one

</div>

---

Murmur is a "better Hush": same RFC 9420 (MLS) foundation, but differentiated on
the four axes Hush is weak on — **post-quantum crypto** (hybrid X25519 + ML-KEM),
**metadata resistance** (sealed sender + padding), **federation** (IETF MIMI), and
**mobile from day one**. Dark, professional, audited.

> Status: **early.** Phase 0 (de-risk) complete; Phase 1 (crypto core) in progress.
> Not yet audited — do not use for real secrets until a third-party crypto audit
> has passed.

## Repository layout (this workspace)
| Path | What it is | Status |
|-|-|-|
| `docs/` | Architecture, roadmap, design language, phase findings | Living |
| `murmur-crypto/` | Rust MLS core over OpenMLS — `Identity`, `Group`, `Account` | ✅ API + tests green |
| `murmur-wasm/` | wasm-bindgen wrapper (web client core) | ✅ compiles + runs in Node |
| `murmur-uniffi-spike/` | uniffi seam (Android/iOS bindings) proof | ✅ generates Kotlin + Swift |

Planned (per `docs/ROADMAP.md`): `murmur-server` (Rust/axum relay + storage),
`murmur-web` (React/Vite), `murmur-mobile` (React Native), `murmur-desktop`
(Tauri), `murmur-directory` (federation + key transparency).

## What works today
- A real MLS group lifecycle (create → add → join → send → receive → remove) with
  exporter-derived media keys, behind one clean Rust API.
- That same core compiled to **WebAssembly** and verified end-to-end under a JS
  runtime (`murmur-wasm/test_node.js`).
- The mobile binding seam proven (one Rust core → Kotlin + Swift).

## Develop
```bash
# crypto core (native)
cd murmur-crypto && cargo test

# wasm core, run under node
cd murmur-wasm
cargo build --target wasm32-unknown-unknown --release
wasm-bindgen --target nodejs --out-dir pkg-node target/wasm32-unknown-unknown/release/murmur_wasm.wasm
node test_node.js
```
Toolchain: Rust (MSVC on Windows), `wasm32-unknown-unknown` target, `wasm-bindgen-cli`.

## Docs
- `docs/ARCHITECTURE.md` — components, crypto core, metadata/federation models
- `docs/ROADMAP.md` — phased plan (P0–P8), critical path
- `docs/DESIGN-LANGUAGE.md` — binding visual/UX contract (dark, anti-template)
- `docs/PHASE-0-FINDINGS.md` — de-risk results + locked decisions

## License
AGPL-3.0 (matching the project it improves on).
