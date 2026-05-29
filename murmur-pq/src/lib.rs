//! Hybrid post-quantum KEM for Murmur — **X25519 + ML-KEM-768**.
//!
//! This is the cryptographic heart of the post-quantum differentiator. A hybrid
//! KEM stays secure as long as *either* primitive holds, defeating "harvest now,
//! decrypt later" without betting everything on PQ maturity:
//!   - **ML-KEM-768** (FIPS 203) — the post-quantum KEM.
//!   - **X25519** — the classical ECDH fallback.
//! The two shared secrets are bound together with a SHA3-256 combiner that also
//! commits to the X25519 ciphertext and recipient key (X-Wing style).
//!
//! ## Status
//! Real, tested cryptography. NOT yet wired into MLS and NOT audited. The next
//! step is a forked OpenMLS that registers a ciphersuite whose HPKE KEM calls
//! this module (see `docs/PQ-INTEGRATION.md`). Do not ship for real secrets
//! until that integration lands and an independent audit passes.

use ml_kem::kem::{Decapsulate, Encapsulate};
use ml_kem::{EncodedSizeUser, KemCore, MlKem768};
use rand_core::OsRng;
use sha3::{Digest, Sha3_256};
use x25519_dalek::{PublicKey as XPublic, StaticSecret as XSecret};

type MlEk = <MlKem768 as KemCore>::EncapsulationKey;
type MlDk = <MlKem768 as KemCore>::DecapsulationKey;

/// Domain separation for the hybrid combiner. Bump the version on any change.
const COMBINER_LABEL: &[u8] = b"murmur-pq/x25519+ml-kem-768/v1";

/// A 32-byte hybrid shared secret. Feeds the MLS HPKE key schedule downstream.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SharedSecret(pub [u8; 32]);

/// Recipient secret key (both halves).
pub struct SecretKey {
    ml_dk: MlDk,
    x_sk: XSecret,
}

/// Recipient public key (both halves).
pub struct PublicKey {
    ml_ek: MlEk,
    x_pk: XPublic,
}

/// KEM ciphertext (both halves).
pub struct Ciphertext {
    ml_ct: ml_kem::Ciphertext<MlKem768>,
    x_ct: XPublic,
}

impl PublicKey {
    /// Serialize as `ml-kem-ek (1184) || x25519-pk (32)`.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = self.ml_ek.as_bytes().to_vec();
        out.extend_from_slice(self.x_pk.as_bytes());
        out
    }
}

fn combine(ml_ss: &[u8], x_ss: &[u8], x_ct: &[u8], x_pk: &[u8]) -> SharedSecret {
    let mut h = Sha3_256::new();
    h.update(COMBINER_LABEL);
    h.update(ml_ss);
    h.update(x_ss);
    h.update(x_ct);
    h.update(x_pk);
    let digest = h.finalize();
    let mut ss = [0u8; 32];
    ss.copy_from_slice(&digest);
    SharedSecret(ss)
}

/// Generate a fresh hybrid keypair.
pub fn generate() -> (SecretKey, PublicKey) {
    let (ml_dk, ml_ek) = MlKem768::generate(&mut OsRng);
    let x_sk = XSecret::random_from_rng(OsRng);
    let x_pk = XPublic::from(&x_sk);
    (SecretKey { ml_dk, x_sk }, PublicKey { ml_ek, x_pk })
}

/// Encapsulate to a recipient public key, producing a ciphertext and the
/// sender's copy of the shared secret.
pub fn encapsulate(pk: &PublicKey) -> (Ciphertext, SharedSecret) {
    let (ml_ct, ml_ss) = pk.ml_ek.encapsulate(&mut OsRng).expect("ml-kem encapsulate");
    let eph = XSecret::random_from_rng(OsRng);
    let x_ct = XPublic::from(&eph);
    let x_ss = eph.diffie_hellman(&pk.x_pk);
    let ss = combine(ml_ss.as_slice(), x_ss.as_bytes(), x_ct.as_bytes(), pk.x_pk.as_bytes());
    (Ciphertext { ml_ct, x_ct }, ss)
}

/// Decapsulate with the recipient secret key, recovering the same shared secret.
pub fn decapsulate(sk: &SecretKey, ct: &Ciphertext) -> SharedSecret {
    let ml_ss = sk.ml_dk.decapsulate(&ct.ml_ct).expect("ml-kem decapsulate");
    let x_ss = sk.x_sk.diffie_hellman(&ct.x_ct);
    let x_pk = XPublic::from(&sk.x_sk);
    combine(ml_ss.as_slice(), x_ss.as_bytes(), ct.x_ct.as_bytes(), x_pk.as_bytes())
}
