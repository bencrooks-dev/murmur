//! PQ-HPKE — single-shot hybrid public-key encryption built on the hybrid KEM.
//!
//! This is the layer an MLS ciphersuite actually calls (RFC 9180 HPKE base mode,
//! shape): KEM → KDF → AEAD.
//!   - **KEM:** [`crate`] hybrid X25519 + ML-KEM-768.
//!   - **KDF:** HKDF-SHA-256 over the KEM shared secret + `info`.
//!   - **AEAD:** AES-256-GCM.
//! Seal produces the KEM ciphertext plus an AEAD ciphertext; open recovers the
//! plaintext only with the right secret key, `info`, and `aad`.
//!
//! Status: tested primitive. Wiring this as the HPKE of a registered MLS
//! ciphersuite (forked OpenMLS) and an audit are the remaining steps — see
//! `docs/PQ-INTEGRATION.md`.

use aes_gcm::aead::{Aead, KeyInit, Payload};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use hkdf::Hkdf;
use sha2::Sha256;

use crate::{decapsulate, encapsulate, Ciphertext, PublicKey, SecretKey};

/// Errors from the open (decrypt) path.
#[derive(Debug, PartialEq, Eq)]
pub enum HpkeError {
    /// AEAD authentication failed (wrong key, info, aad, or tampered ciphertext).
    OpenFailed,
}

// A fixed, single-use nonce is correct here because each seal derives a fresh key
// from a fresh KEM encapsulation (the shared secret is never reused).
const NONCE: [u8; 12] = [0u8; 12];
const KDF_LABEL: &[u8] = b"murmur-pq-hpke/aes256gcm/v1";

fn derive_key(shared_secret: &[u8; 32], info: &[u8]) -> [u8; 32] {
    let hk = Hkdf::<Sha256>::new(Some(KDF_LABEL), shared_secret);
    let mut key = [0u8; 32];
    hk.expand(info, &mut key).expect("32 bytes is a valid HKDF length");
    key
}

/// Encrypt `plaintext` to `recipient`. Returns the KEM ciphertext and the AEAD
/// ciphertext; `aad` is authenticated but not encrypted.
pub fn seal(
    recipient: &PublicKey,
    info: &[u8],
    aad: &[u8],
    plaintext: &[u8],
) -> (Ciphertext, Vec<u8>) {
    let (kem_ct, shared) = encapsulate(recipient);
    let key = derive_key(&shared.0, info);
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));
    let aead_ct = cipher
        .encrypt(Nonce::from_slice(&NONCE), Payload { msg: plaintext, aad })
        .expect("aes-256-gcm encrypt");
    (kem_ct, aead_ct)
}

/// Decrypt an AEAD ciphertext produced by [`seal`] for this recipient.
pub fn open(
    recipient_sk: &SecretKey,
    kem_ct: &Ciphertext,
    info: &[u8],
    aad: &[u8],
    aead_ct: &[u8],
) -> Result<Vec<u8>, HpkeError> {
    let shared = decapsulate(recipient_sk, kem_ct);
    let key = derive_key(&shared.0, info);
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));
    cipher
        .decrypt(Nonce::from_slice(&NONCE), Payload { msg: aead_ct, aad })
        .map_err(|_| HpkeError::OpenFailed)
}
