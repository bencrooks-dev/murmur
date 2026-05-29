# OpenMLS fork plan — landing the PQ ciphersuite (Phase 2, the hard part)

**Fork:** https://github.com/bencrooks-dev/openmls (forked from `openmls/openmls`).
This is a **multi-week, audit-gated effort**, tracked here. It is deliberately
kept on the fork / a branch and NOT merged into Murmur's working build until it
compiles, passes the full test stack, and clears an independent review.

## Why a fork is required
OpenMLS's `Ciphersuite` is a closed Rust enum (Spike B). A custom crypto provider
can change *how* an existing suite computes crypto, but cannot add a new
ciphersuite id. So we add the suite inside the fork.

## Already done (in `murmur-pq`, tested)
- Hybrid KEM: X25519 + ML-KEM-768 (4 tests).
- PQ-HPKE: KEM → HKDF-SHA256 → AES-256-GCM seal/open (5 tests).
These are the exact primitives the new ciphersuite consumes.

## Work items (in the fork)
1. **Add the ciphersuite id** — a private-use value, e.g. `0xF768`
   `MLS_256_HYBRID_X25519MLKEM768_AES256GCM_SHA384`, to the `Ciphersuite` enum and
   every exhaustive `match` it touches (KDF=SHA-384, AEAD=AES-256-GCM, sig=Ed25519
   to start). Expect the compiler to enumerate ~all the sites.
2. **HPKE wiring** — route the suite's `hpke_*` ops through `murmur-pq::hpke`
   (KEM = hybrid). Implement the OpenMLS crypto-provider methods for the new id.
3. **Key schedule + capabilities** — ensure the new suite negotiates in
   KeyPackages and group config; sizes (ML-KEM keys are kB) flow through.
4. **Point `murmur-crypto`** at the fork (path/git dep) and flip `CIPHERSUITE` to
   the PQ suite; re-run the full stack (group lifecycle, WASM, relay e2e).
5. **Benchmark** commit/welcome cost on desktop AND mobile.
6. **Third-party cryptographic audit** — the hard gate before any "secure" claim.

## Done criteria
A PQ-protected MLS group completes create → add → message → remove on web and a
phone, with the hybrid suite, all tests green, perf documented, and an audit
passed. Until then: **do not market Murmur as post-quantum-secure.**

## Status
Step 0 (fork created) ✅. Steps 1–6 are the dedicated build ahead.
