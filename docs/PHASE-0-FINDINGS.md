# Murmur — Phase 0 findings (de-risk spikes)

Date: 2026-05-29. Machine: Windows 11, working dir `C:\Users\mtodd\Murmur`.

## Toolchain
- Rust installed via winget (rustup 1.29, rustc/cargo 1.96).
- **MSVC linker absent** (no Visual Studio C++ tools). Resolved by switching to
  the **GNU toolchain** (`stable-x86_64-pc-windows-gnu`), which bundles its own
  MinGW linker. No multi-GB VS install needed. ✅
- Node 24 + npm present (for web client later). Docker NOT installed (needed for
  server self-host path in Phase 3 — install before then).

## Spike A — OpenMLS 2-member group ✅ PASS
- `tests/spike_a_two_member_group.rs`: Alice creates a group, adds Bob via key
  package + Welcome, sends an application message, Bob decrypts it. Passes.
- Stack resolved: `openmls 0.6.0`, `openmls_rust_crypto 0.3.0`,
  `openmls_basic_credential 0.3.0`. Ciphersuite
  `MLS_128_DHKEMX25519_AES128GCM_SHA256_Ed25519`.
- Dep tree includes `aes-gcm`, `x25519-dalek`, `hpke-rs`, `p256/p384`,
  `chacha20poly1305` — the crypto primitives we need are all present.
- **Conclusion:** the RFC 9420 base Murmur depends on works on this machine. The
  protocol foundation is real, not aspirational.

## Spike B — custom crypto-provider seam ✅ PASS (with a caveat)
- `tests/spike_b_custom_provider.rs`: a `MurmurProvider` wrapping the stock
  crypto/rand/storage drives a full group creation. OpenMLS accepts our provider
  type. The `crypto()` return is the single seam we replace in Phase 2.
- **⚠ CRITICAL CONSTRAINT — the PQ plan's biggest risk:** OpenMLS 0.6's
  `Ciphersuite` is a **closed Rust enum**. A custom provider can change *how*
  crypto is computed for an EXISTING ciphersuite ID, but it **cannot register a
  brand-new PQ ciphersuite number** (e.g. an X-Wing / ML-KEM suite) without one
  of:
  1. **Forking OpenMLS** to add the ciphersuite to the enum + key schedule.
  2. **Upstream pluggable-ciphersuite support** (track the OpenMLS roadmap / the
     IETF PQ-MLS draft) and wait/contribute.
  3. **A different MLS library** with open ciphersuite registration.
- This does NOT block the differentiator, but it makes "hybrid PQ ciphersuite" a
  **fork-or-upstream decision**, not a drop-in provider swap. Must be decided
  before Phase 2.

## Spike C — uniffi / mobile-day-one ⏳ NOT YET RUN + a hard platform finding
- The uniffi binding *seam* (Rust → Swift/Kotlin) is provable on Windows.
- **⚠ HARD CONSTRAINT for "mobile day one":** **iOS builds require macOS.** You
  cannot compile or sign an iOS app on this Windows box — full stop. Android
  builds work on Windows but need the **Android NDK** installed.
- Implication: "mobile day one" is achievable for **Android** here; **iOS needs a
  Mac** (CI runner or local machine). Plan the mobile track around that split.

## Spike C — uniffi seam ✅ PASS (on MSVC)
- Spike crate `murmur-uniffi-spike/`: exports a function + a `SessionHandle`
  object via uniffi 0.28 proc-macros, plus a `uniffi-bindgen` binary.
- On GNU the `cdylib` build failed (incomplete MinGW binutils — `dlltool` can't
  spawn helpers). **Switched to MSVC (decision #3)** and it built cleanly.
- `uniffi-bindgen` generated real bindings for BOTH targets:
  - **Kotlin (Android):** `bindings/uniffi/murmur_uniffi_spike/murmur_uniffi_spike.kt`
    — contains `class SessionHandle` + `murmur_ffi_smoke`.
  - **Swift (iOS):** `bindings/murmur_uniffi_spike.swift` (+ `.h`, `.modulemap`)
    — contains `open class SessionHandle` + `func label()`.
  - (ktlint/swiftformat auto-format warnings are cosmetic — formatters not installed.)
- **Conclusion:** one Rust core → working Android + iOS bindings. Mobile-day-one
  is technically real. Device cross-compile still needs the Android NDK (Android)
  and macOS (iOS) — expected, deferred to the mobile track.

## DECISIONS (locked 2026-05-29)
1. **PQ approach → FORK OpenMLS.** Rationale: Spikes A+B prove OpenMLS is the
   right base; a provider-only swap is impossible (closed ciphersuite enum); the
   PQ change is contained to the ciphersuite table + key schedule. Track the IETF
   PQ-MLS draft and rebase onto upstream pluggable ciphersuites if/when they land.
2. **iOS build host → macOS CI runner.** Develop Android-first locally; build/sign
   iOS on a cloud macOS runner. No Mac purchase required to start.
3. **Canonical Windows toolchain → MSVC.** GNU was a stopgap that cleared Spikes
   A+B but is incomplete for cdylib/FFI (Spike C) and not what Tauri/native server
   builds expect on Windows. MSVC is the official recommended Windows Rust
   toolchain. Installing VS 2022 Build Tools (VCTools workload) now.

## Remaining decisions (defer to Phase 1 kickoff)
- Server language (Rust/axum vs Go), desktop shell (Tauri vs Electron — leaning
  Tauri), license (AGPL-3.0?), audit budget — unchanged from ARCHITECTURE §8.

## Status — PHASE 0 COMPLETE
- Spikes A + B + C all green. MLS base, PQ provider seam, and mobile binding seam
  are all proven on this machine. Toolchain settled on MSVC.
- All three decisions locked (fork OpenMLS / macOS CI for iOS / MSVC).
- Two constraints recorded that shape later work: closed ciphersuite enum → PQ is
  a fork; iOS device builds need macOS.
- **Next (Phase 1):** install Docker (Phase 3 server) + Android NDK (mobile device
  builds); fork OpenMLS into a `murmur-openmls` vendor dir; build the real
  `murmur-crypto` group API with WASM + uniffi bindings + RFC 9420 test vectors.
