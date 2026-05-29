//! Murmur MLS crypto core.
//!
//! A small, clean façade over OpenMLS (RFC 9420) that the clients drive through
//! WASM (web) and uniffi (mobile/desktop) bindings. Clients never touch OpenMLS
//! directly — all protocol logic lives here, so every platform behaves
//! identically.
//!
//! Phase 1 scope: classical ciphersuite, in-memory provider, the group lifecycle
//! (identity → create → add → join → send → receive → exporter secret). The
//! post-quantum ciphersuite (Phase 2) swaps the crypto provider behind this same
//! API; the public surface below does not change.

use openmls::prelude::tls_codec::{Deserialize as _, Serialize as _};
use openmls::prelude::*;
use openmls_basic_credential::SignatureKeyPair;
use openmls_rust_crypto::OpenMlsRustCrypto;

/// The crypto/storage backend. In-memory for now; persistent storage is a later
/// phase. Kept as one type alias so the rest of the code never names it directly.
pub type Provider = OpenMlsRustCrypto;

/// Classical MLS ciphersuite for Phase 1. Phase 2 introduces a hybrid PQ suite.
pub const CIPHERSUITE: Ciphersuite =
    Ciphersuite::MLS_128_DHKEMX25519_AES128GCM_SHA256_Ed25519;

/// Errors surfaced across the binding boundary. OpenMLS' many error types are
/// flattened to stable, string-carrying variants so the FFI surface stays small.
#[derive(Debug, thiserror::Error)]
pub enum MurmurError {
    #[error("mls protocol error: {0}")]
    Mls(String),
    #[error("message encoding error: {0}")]
    Codec(String),
    #[error("unexpected message type")]
    UnexpectedMessage,
}

type Result<T> = std::result::Result<T, MurmurError>;

fn mls<E: std::fmt::Display>(e: E) -> MurmurError {
    MurmurError::Mls(e.to_string())
}
fn codec<E: std::fmt::Display>(e: E) -> MurmurError {
    MurmurError::Codec(e.to_string())
}

/// A local account identity: a signing keypair plus its MLS credential. In
/// production the signer is persisted in the provider's storage; here it lives
/// for the session.
pub struct Identity {
    credential_with_key: CredentialWithKey,
    signer: SignatureKeyPair,
}

impl Identity {
    /// Create a fresh identity for `name` and store its signing key in `provider`.
    pub fn generate(name: &str, provider: &Provider) -> Result<Self> {
        let credential = BasicCredential::new(name.as_bytes().to_vec());
        let signer = SignatureKeyPair::new(CIPHERSUITE.signature_algorithm())
            .map_err(mls)?;
        signer.store(provider.storage()).map_err(mls)?;
        Ok(Self {
            credential_with_key: CredentialWithKey {
                credential: credential.into(),
                signature_key: signer.public().into(),
            },
            signer,
        })
    }

    /// Publish a key package others use to add this identity to a group. In
    /// production this is uploaded to the relay; here the bytes are returned.
    pub fn key_package(&self, provider: &Provider) -> Result<Vec<u8>> {
        let bundle = KeyPackage::builder()
            .build(
                CIPHERSUITE,
                provider,
                &self.signer,
                self.credential_with_key.clone(),
            )
            .map_err(mls)?;
        bundle
            .key_package()
            .tls_serialize_detached()
            .map_err(codec)
    }
}

/// One MLS group == one Murmur channel.
pub struct Group {
    inner: MlsGroup,
}

impl Group {
    fn create_config() -> MlsGroupCreateConfig {
        MlsGroupCreateConfig::builder()
            .ciphersuite(CIPHERSUITE)
            .use_ratchet_tree_extension(true)
            .build()
    }

    fn join_config() -> MlsGroupJoinConfig {
        MlsGroupJoinConfig::builder()
            .use_ratchet_tree_extension(true)
            .build()
    }

    /// Create a new channel owned by `identity`.
    pub fn create(provider: &Provider, identity: &Identity) -> Result<Self> {
        let inner = MlsGroup::new(
            provider,
            &identity.signer,
            &Self::create_config(),
            identity.credential_with_key.clone(),
        )
        .map_err(mls)?;
        Ok(Self { inner })
    }

    /// Add a member from their published key-package bytes. Returns the serialized
    /// Welcome to hand to the new member out of band (via the relay).
    pub fn add_member(
        &mut self,
        provider: &Provider,
        identity: &Identity,
        key_package_bytes: &[u8],
    ) -> Result<Vec<u8>> {
        let key_package = KeyPackageIn::tls_deserialize_exact(key_package_bytes)
            .map_err(codec)?
            .validate(provider.crypto(), ProtocolVersion::Mls10)
            .map_err(mls)?;

        let (_commit, welcome, _group_info) = self
            .inner
            .add_members(provider, &identity.signer, &[key_package])
            .map_err(mls)?;
        self.inner.merge_pending_commit(provider).map_err(mls)?;

        welcome.tls_serialize_detached().map_err(codec)
    }

    /// Join a group from a serialized Welcome.
    pub fn join(provider: &Provider, welcome_bytes: &[u8]) -> Result<Self> {
        let message =
            MlsMessageIn::tls_deserialize_exact(welcome_bytes).map_err(codec)?;
        let welcome = match message.extract() {
            MlsMessageBodyIn::Welcome(w) => w,
            _ => return Err(MurmurError::UnexpectedMessage),
        };
        let inner = StagedWelcome::new_from_welcome(
            provider,
            &Self::join_config(),
            welcome,
            None, // ratchet tree carried in-band via the extension
        )
        .map_err(mls)?
        .into_group(provider)
        .map_err(mls)?;
        Ok(Self { inner })
    }

    /// Encrypt an application message. Returns ciphertext to relay to the group.
    pub fn send(
        &mut self,
        provider: &Provider,
        identity: &Identity,
        plaintext: &[u8],
    ) -> Result<Vec<u8>> {
        let out = self
            .inner
            .create_message(provider, &identity.signer, plaintext)
            .map_err(mls)?;
        out.tls_serialize_detached().map_err(codec)
    }

    /// Process an inbound message. Returns `Some(plaintext)` for an application
    /// message, `None` for handshake traffic (commits are merged automatically).
    pub fn receive(
        &mut self,
        provider: &Provider,
        message_bytes: &[u8],
    ) -> Result<Option<Vec<u8>>> {
        let message =
            MlsMessageIn::tls_deserialize_exact(message_bytes).map_err(codec)?;
        let protocol = message
            .try_into_protocol_message()
            .map_err(|_| MurmurError::UnexpectedMessage)?;
        let processed = self.inner.process_message(provider, protocol).map_err(mls)?;

        match processed.into_content() {
            ProcessedMessageContent::ApplicationMessage(app) => {
                Ok(Some(app.into_bytes()))
            }
            ProcessedMessageContent::StagedCommitMessage(staged) => {
                self.inner
                    .merge_staged_commit(provider, *staged)
                    .map_err(mls)?;
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    /// Derive a media key from the group's exporter secret. Voice/video SRTP keys
    /// come from here (the SFU never sees them).
    pub fn exporter_secret(
        &self,
        provider: &Provider,
        label: &str,
        length: usize,
    ) -> Result<Vec<u8>> {
        self.inner
            .export_secret(provider, label, &[], length)
            .map_err(mls)
    }

    /// Number of members currently in the group.
    pub fn member_count(&self) -> usize {
        self.inner.members().count()
    }

    /// The group's stable identifier (used to route messages and key storage).
    pub fn id(&self) -> Vec<u8> {
        self.inner.group_id().as_slice().to_vec()
    }

    /// Remove the member at `leaf_index`. Returns the commit to broadcast to the
    /// remaining members so they advance the group (forward secrecy: the removed
    /// member cannot read anything after this commit).
    pub fn remove_member(
        &mut self,
        provider: &Provider,
        identity: &Identity,
        leaf_index: u32,
    ) -> Result<Vec<u8>> {
        let (commit, _welcome, _group_info) = self
            .inner
            .remove_members(
                provider,
                &identity.signer,
                &[LeafNodeIndex::new(leaf_index)],
            )
            .map_err(mls)?;
        self.inner.merge_pending_commit(provider).map_err(mls)?;
        commit.tls_serialize_detached().map_err(codec)
    }
}

/// A stateful local account: owns its key store (`Provider`), its `Identity`, and
/// the groups it belongs to, keyed by group id. This is the surface the WASM
/// (web) and uniffi (mobile/desktop) bindings wrap, so every client behaves
/// identically. Messages cross the boundary as opaque bytes.
pub struct Account {
    provider: Provider,
    identity: Identity,
    groups: std::collections::HashMap<Vec<u8>, Group>,
}

impl Account {
    /// Create a new account for `name` with a fresh identity.
    pub fn new(name: &str) -> Result<Self> {
        let provider = Provider::default();
        let identity = Identity::generate(name, &provider)?;
        Ok(Self {
            provider,
            identity,
            groups: std::collections::HashMap::new(),
        })
    }

    /// Publish this account's key package (others use it to add this account).
    pub fn key_package(&self) -> Result<Vec<u8>> {
        self.identity.key_package(&self.provider)
    }

    /// Create a new channel; returns its group id.
    pub fn create_group(&mut self) -> Result<Vec<u8>> {
        let group = Group::create(&self.provider, &self.identity)?;
        let id = group.id();
        self.groups.insert(id.clone(), group);
        Ok(id)
    }

    /// Add a member to a group from their key-package bytes; returns the Welcome.
    /// (Fields are borrowed disjointly so the group's `&mut` coexists with the
    /// shared borrows of `provider`/`identity`.)
    pub fn add_member(&mut self, group_id: &[u8], key_package: &[u8]) -> Result<Vec<u8>> {
        let group = self
            .groups
            .get_mut(group_id)
            .ok_or_else(|| MurmurError::Mls("unknown group".into()))?;
        group.add_member(&self.provider, &self.identity, key_package)
    }

    /// Remove the member at `leaf_index`; returns the commit to broadcast.
    pub fn remove_member(&mut self, group_id: &[u8], leaf_index: u32) -> Result<Vec<u8>> {
        let group = self
            .groups
            .get_mut(group_id)
            .ok_or_else(|| MurmurError::Mls("unknown group".into()))?;
        group.remove_member(&self.provider, &self.identity, leaf_index)
    }

    /// Join a channel from a Welcome; returns the joined group id.
    pub fn join_group(&mut self, welcome: &[u8]) -> Result<Vec<u8>> {
        let group = Group::join(&self.provider, welcome)?;
        let id = group.id();
        self.groups.insert(id.clone(), group);
        Ok(id)
    }

    /// Encrypt a message for a group.
    pub fn send(&mut self, group_id: &[u8], plaintext: &[u8]) -> Result<Vec<u8>> {
        let group = self
            .groups
            .get_mut(group_id)
            .ok_or_else(|| MurmurError::Mls("unknown group".into()))?;
        group.send(&self.provider, &self.identity, plaintext)
    }

    /// Decrypt/process an inbound message. `Some(bytes)` for an application
    /// message, `None` for handshake traffic.
    pub fn receive(&mut self, group_id: &[u8], message: &[u8]) -> Result<Option<Vec<u8>>> {
        let group = self
            .groups
            .get_mut(group_id)
            .ok_or_else(|| MurmurError::Mls("unknown group".into()))?;
        group.receive(&self.provider, message)
    }

    /// Derive a media key (voice/video) from a group's exporter secret.
    pub fn exporter_secret(&self, group_id: &[u8], label: &str, length: usize) -> Result<Vec<u8>> {
        let group = self
            .groups
            .get(group_id)
            .ok_or_else(|| MurmurError::Mls("unknown group".into()))?;
        group.exporter_secret(&self.provider, label, length)
    }

    /// Member count of a group.
    pub fn member_count(&self, group_id: &[u8]) -> Result<usize> {
        Ok(self
            .groups
            .get(group_id)
            .ok_or_else(|| MurmurError::Mls("unknown group".into()))?
            .member_count())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ciphersuite_is_classical_for_phase_1() {
        assert_eq!(
            CIPHERSUITE,
            Ciphersuite::MLS_128_DHKEMX25519_AES128GCM_SHA256_Ed25519
        );
    }
}
