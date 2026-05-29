//! PQ-HPKE round-trip + negative tests.

use murmur_pq::generate;
use murmur_pq::hpke::{open, seal, HpkeError};

const INFO: &[u8] = b"murmur/channel/42";
const AAD: &[u8] = b"epoch=7";
const MSG: &[u8] = b"post-quantum sealed payload";

#[test]
fn seal_open_roundtrip() {
    let (sk, pk) = generate();
    let (kem_ct, ct) = seal(&pk, INFO, AAD, MSG);
    let pt = open(&sk, &kem_ct, INFO, AAD, &ct).unwrap();
    assert_eq!(pt, MSG);
}

#[test]
fn wrong_aad_fails() {
    let (sk, pk) = generate();
    let (kem_ct, ct) = seal(&pk, INFO, AAD, MSG);
    assert_eq!(open(&sk, &kem_ct, INFO, b"epoch=8", &ct), Err(HpkeError::OpenFailed));
}

#[test]
fn wrong_info_fails() {
    let (sk, pk) = generate();
    let (kem_ct, ct) = seal(&pk, INFO, AAD, MSG);
    assert_eq!(open(&sk, &kem_ct, b"other", AAD, &ct), Err(HpkeError::OpenFailed));
}

#[test]
fn tampered_ciphertext_fails() {
    let (sk, pk) = generate();
    let (kem_ct, mut ct) = seal(&pk, INFO, AAD, MSG);
    ct[0] ^= 0xff;
    assert_eq!(open(&sk, &kem_ct, INFO, AAD, &ct), Err(HpkeError::OpenFailed));
}

#[test]
fn wrong_recipient_fails() {
    let (_sk_a, pk_a) = generate();
    let (sk_b, _pk_b) = generate();
    let (kem_ct, ct) = seal(&pk_a, INFO, AAD, MSG);
    assert_eq!(open(&sk_b, &kem_ct, INFO, AAD, &ct), Err(HpkeError::OpenFailed));
}
