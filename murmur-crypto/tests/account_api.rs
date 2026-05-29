//! Phase 1 — the stateful `Account` surface the FFI bindings wrap.

use murmur_crypto::Account;

#[test]
fn account_lifecycle_with_removal() {
    let mut alice = Account::new("alice").unwrap();
    let mut bob = Account::new("bob").unwrap();

    let bob_kp = bob.key_package().unwrap();
    let gid = alice.create_group().unwrap();
    let welcome = alice.add_member(&gid, &bob_kp).unwrap();
    let bob_gid = bob.join_group(&welcome).unwrap();
    assert_eq!(gid, bob_gid, "both sides share the same group id");

    assert_eq!(alice.member_count(&gid).unwrap(), 2);
    assert_eq!(bob.member_count(&bob_gid).unwrap(), 2);

    // Message round-trip via the account API.
    const MSG: &[u8] = b"enterprise-grade and quiet";
    let ct = alice.send(&gid, MSG).unwrap();
    assert_eq!(bob.receive(&bob_gid, &ct).unwrap().as_deref(), Some(MSG));

    // Remove Bob (leaf index 1). Alice's group shrinks to 1; forward secrecy holds.
    alice.remove_member(&gid, 1).unwrap();
    assert_eq!(alice.member_count(&gid).unwrap(), 1);
}
