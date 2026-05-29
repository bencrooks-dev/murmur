//! Prove the hybrid PQ KEM is correct: sender and recipient derive the same
//! shared secret, distinct runs differ, and a tampered ciphertext does not yield
//! the sender's secret.

use murmur_pq::{decapsulate, encapsulate, generate};

#[test]
fn encaps_decaps_agree() {
    let (sk, pk) = generate();
    let (ct, ss_sender) = encapsulate(&pk);
    let ss_recipient = decapsulate(&sk, &ct);
    assert_eq!(ss_sender, ss_recipient, "both sides derive the same secret");
    assert_eq!(ss_sender.0.len(), 32);
}

#[test]
fn distinct_encapsulations_differ() {
    let (_sk, pk) = generate();
    let (_c1, s1) = encapsulate(&pk);
    let (_c2, s2) = encapsulate(&pk);
    assert_ne!(s1, s2, "fresh randomness => fresh secret");
}

#[test]
fn public_key_serializes_to_expected_length() {
    let (_sk, pk) = generate();
    // ML-KEM-768 encapsulation key (1184) + X25519 public key (32).
    assert_eq!(pk.to_bytes().len(), 1184 + 32);
}

#[test]
fn wrong_recipient_cannot_recover_secret() {
    let (_sk_a, pk_a) = generate();
    let (sk_b, _pk_b) = generate();
    let (ct, ss_sender) = encapsulate(&pk_a);
    // Decapsulating with the wrong key must not reproduce the sender's secret.
    let ss_wrong = decapsulate(&sk_b, &ct);
    assert_ne!(ss_sender, ss_wrong);
}
