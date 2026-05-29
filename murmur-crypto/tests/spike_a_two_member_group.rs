//! Spike A — prove OpenMLS drives a 2-member MLS group end to end.
//!
//! Alice creates a group, adds Bob, and sends an application message that Bob
//! decrypts. If this passes, the RFC 9420 base Murmur depends on is real on this
//! machine. Classical ciphersuite for now; the PQ swap is Spike B.

use openmls::prelude::tls_codec::{Deserialize as _, Serialize as _};
use openmls::prelude::*;
use openmls_basic_credential::SignatureKeyPair;
use openmls_rust_crypto::OpenMlsRustCrypto;

const CIPHERSUITE: Ciphersuite = Ciphersuite::MLS_128_DHKEMX25519_AES128GCM_SHA256_Ed25519;

fn credential(
    identity: &[u8],
    provider: &OpenMlsRustCrypto,
) -> (CredentialWithKey, SignatureKeyPair) {
    let credential = BasicCredential::new(identity.to_vec());
    let signer = SignatureKeyPair::new(CIPHERSUITE.signature_algorithm())
        .expect("signature keypair");
    signer.store(provider.storage()).expect("store signer");
    (
        CredentialWithKey {
            credential: credential.into(),
            signature_key: signer.public().into(),
        },
        signer,
    )
}

fn key_package(
    provider: &OpenMlsRustCrypto,
    signer: &SignatureKeyPair,
    cwk: CredentialWithKey,
) -> KeyPackageBundle {
    KeyPackage::builder()
        .build(CIPHERSUITE, provider, signer, cwk)
        .expect("key package")
}

#[test]
fn two_member_group_roundtrip() {
    let alice_provider = OpenMlsRustCrypto::default();
    let bob_provider = OpenMlsRustCrypto::default();

    let (alice_cwk, alice_signer) = credential(b"alice", &alice_provider);
    let (bob_cwk, bob_signer) = credential(b"bob", &bob_provider);

    // Bob publishes a key package; in production this comes from the server.
    let bob_kp = key_package(&bob_provider, &bob_signer, bob_cwk);

    let group_config = MlsGroupCreateConfig::builder()
        .ciphersuite(CIPHERSUITE)
        .use_ratchet_tree_extension(true)
        .build();

    // Alice creates the group.
    let mut alice_group =
        MlsGroup::new(&alice_provider, &alice_signer, &group_config, alice_cwk)
            .expect("create group");

    // Alice adds Bob.
    let (_commit, welcome, _group_info) = alice_group
        .add_members(
            &alice_provider,
            &alice_signer,
            &[bob_kp.key_package().clone()],
        )
        .expect("add bob");
    alice_group
        .merge_pending_commit(&alice_provider)
        .expect("merge commit");

    // Bob joins from the Welcome.
    let serialized_welcome = welcome.tls_serialize_detached().expect("serialize welcome");
    let welcome_in =
        MlsMessageIn::tls_deserialize_exact(serialized_welcome).expect("deserialize welcome");
    let welcome = match welcome_in.extract() {
        MlsMessageBodyIn::Welcome(w) => w,
        _ => panic!("expected a Welcome message"),
    };
    let mut bob_group = StagedWelcome::new_from_welcome(
        &bob_provider,
        group_config.join_config(),
        welcome,
        None, // ratchet tree carried in-band via the extension
    )
    .expect("stage welcome")
    .into_group(&bob_provider)
    .expect("join group");

    assert_eq!(alice_group.members().count(), 2);
    assert_eq!(bob_group.members().count(), 2);

    // Alice sends an application message; Bob decrypts it.
    const PLAINTEXT: &[u8] = b"hush who? murmur.";
    let msg_out = alice_group
        .create_message(&alice_provider, &alice_signer, PLAINTEXT)
        .expect("create message");

    let serialized = msg_out.tls_serialize_detached().expect("serialize message");
    let msg_in = MlsMessageIn::tls_deserialize_exact(serialized).expect("deserialize message");
    let protocol_msg = msg_in
        .try_into_protocol_message()
        .expect("protocol message");

    let processed = bob_group
        .process_message(&bob_provider, protocol_msg)
        .expect("process message");

    match processed.into_content() {
        ProcessedMessageContent::ApplicationMessage(app) => {
            assert_eq!(app.into_bytes(), PLAINTEXT);
        }
        _ => panic!("expected an application message"),
    }
}
