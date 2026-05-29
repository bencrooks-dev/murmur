//! Phase 1 — exercise the real `murmur-crypto` public API end to end, the way a
//! client binding will: two identities, create channel, add member via Welcome,
//! exchange an application message, and derive matching media keys.

use murmur_crypto::{Group, Identity, Provider};

#[test]
fn full_channel_lifecycle() {
    // Each device has its own provider (its own key store).
    let alice_p = Provider::default();
    let bob_p = Provider::default();

    let alice = Identity::generate("alice", &alice_p).unwrap();
    let bob = Identity::generate("bob", &bob_p).unwrap();

    // Bob publishes a key package; Alice creates the channel and adds him.
    let bob_kp = bob.key_package(&bob_p).unwrap();
    let mut alice_group = Group::create(&alice_p, &alice).unwrap();
    let welcome = alice_group.add_member(&alice_p, &alice, &bob_kp).unwrap();

    let mut bob_group = Group::join(&bob_p, &welcome).unwrap();

    assert_eq!(alice_group.member_count(), 2);
    assert_eq!(bob_group.member_count(), 2);

    // Alice → Bob application message.
    const MSG: &[u8] = b"the walls have no ears here";
    let ct = alice_group.send(&alice_p, &alice, MSG).unwrap();
    let pt = bob_group.receive(&bob_p, &ct).unwrap();
    assert_eq!(pt.as_deref(), Some(MSG));

    // Exporter-derived media keys must match on both sides (voice/video basis).
    let ka = alice_group.exporter_secret(&alice_p, "murmur/media", 32).unwrap();
    let kb = bob_group.exporter_secret(&bob_p, "murmur/media", 32).unwrap();
    assert_eq!(ka.len(), 32);
    assert_eq!(ka, kb, "both members derive the same media key");
}
