# Post-quantum integration plan (Phase 2)

## What is done ✅
`murmur-pq` implements a real, tested **hybrid X25519 + ML-KEM-768 KEM** (the
X-Wing construction): `generate` / `encapsulate` / `decapsulate`, a SHA3-256
combiner binding both shared secrets + the X25519 ciphertext and recipient key.
Tests prove correctness, freshness, key length, and wrong-recipient failure.

This is the cryptographic core of the differentiator. The shared secret it
produces is exactly what an MLS HPKE KEM must yield.

## What remains (and why it's not a flip of a switch) ⛔
OpenMLS 0.6's `Ciphersuite` is a **closed enum** (proven in Spike B). To make MLS
itself post-quantum we must:

1. **Fork OpenMLS** into `vendor/murmur-openmls`.
2. **Register a private-use ciphersuite id** (e.g. `0xF768`)
   `MLS_256_HYBRID_X25519MLKEM768_AES256GCM_SHA384` in the ciphersuite table,
   key schedule, and capability negotiation.
3. **Provide a custom `OpenMlsCrypto`** whose `hpke_*` operations use `murmur-pq`
   for the KEM step, with AES-256-GCM AEAD and SHA-384 KDF (the seam proven in
   Spike B). HPKE single-shot seal/open wraps the KEM secret per RFC 9180.
4. **Wire `murmur-crypto`** to select the new ciphersuite in `CIPHERSUITE`, and
   re-run the full stack (group lifecycle, WASM, relay e2e) on it.
5. **Benchmark on mobile** — ML-KEM keys/ciphertexts are kilobytes; measure
   commit cost on real devices.
6. **Third-party cryptographic audit** — the hard gate before any "secure" claim.

## Why hybrid, not PQ-only
Security holds if *either* X25519 or ML-KEM holds. This defeats harvest-now-
decrypt-later without betting on PQ implementation maturity. Stay hybrid until
standards + audits justify otherwise; track the IETF PQ-MLS draft and rebase onto
a standardized suite when it lands.

## Honest status line
PQ **primitive**: built + tested. PQ **MLS ciphersuite**: specified, not yet
implemented. PQ **audited**: no. Do not market Murmur as post-quantum-secure until
steps 1–6 are complete.
