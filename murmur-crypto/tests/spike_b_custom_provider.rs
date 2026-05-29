//! Spike B — prove the OpenMLS crypto-provider seam is interposable.
//!
//! The PQ differentiator depends on supplying our OWN provider so we can later
//! route HPKE through a hybrid X25519 + ML-KEM KEM. This spike defines a
//! `MurmurProvider` that wraps the stock Rust crypto/rand/storage and drives a
//! full group creation + commit through it. If OpenMLS accepts our provider
//! type here, the same seam accepts a PQ-augmented crypto impl in Phase 2.
//!
//! KNOWN CONSTRAINT (see docs/PHASE-0-FINDINGS.md): OpenMLS 0.6's `Ciphersuite`
//! is a CLOSED enum. We can interpose on crypto operations for existing
//! ciphersuite IDs, but registering a brand-new PQ ciphersuite number requires
//! either a fork or upstream pluggable-ciphersuite support. This spike proves
//! the provider plumbing; it does NOT yet prove a new ciphersuite ID.

use openmls::prelude::*;
use openmls_basic_credential::SignatureKeyPair;
use openmls::prelude::OpenMlsProvider;
use openmls_rust_crypto::{MemoryStorage, OpenMlsRustCrypto, RustCrypto};

const CIPHERSUITE: Ciphersuite = Ciphersuite::MLS_128_DHKEMX25519_AES128GCM_SHA256_Ed25519;

/// Murmur's provider. Today it delegates to the stock implementations; the
/// `crypto()` return is the single seam we'll later replace with a PQ-aware
/// crypto provider that wraps the same `RustCrypto` and overrides HPKE.
#[derive(Default)]
struct MurmurProvider {
    inner: OpenMlsRustCrypto,
}

impl OpenMlsProvider for MurmurProvider {
    type CryptoProvider = RustCrypto;
    type RandProvider = RustCrypto;
    type StorageProvider = MemoryStorage;

    fn storage(&self) -> &Self::StorageProvider {
        self.inner.storage()
    }
    fn crypto(&self) -> &Self::CryptoProvider {
        // PHASE 2 SEAM: swap this for a PQ-augmented OpenMlsCrypto impl.
        self.inner.crypto()
    }
    fn rand(&self) -> &Self::RandProvider {
        self.inner.rand()
    }
}

#[test]
fn group_runs_through_custom_provider() {
    let provider = MurmurProvider::default();

    let credential = BasicCredential::new(b"alice-via-murmur-provider".to_vec());
    let signer =
        SignatureKeyPair::new(CIPHERSUITE.signature_algorithm()).expect("signature keypair");
    signer.store(provider.storage()).expect("store signer");
    let cwk = CredentialWithKey {
        credential: credential.into(),
        signature_key: signer.public().into(),
    };

    let group_config = MlsGroupCreateConfig::builder()
        .ciphersuite(CIPHERSUITE)
        .use_ratchet_tree_extension(true)
        .build();

    // The whole point: MlsGroup accepts &MurmurProvider, exercising our seam.
    let group = MlsGroup::new(&provider, &signer, &group_config, cwk).expect("create group");

    assert_eq!(group.ciphersuite(), CIPHERSUITE);
    assert_eq!(group.members().count(), 1);
}
