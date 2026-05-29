//! WASM bindings for the Murmur crypto core.
//!
//! Thin wrapper over `murmur_crypto::Account` exposed to JavaScript. The web
//! client (`murmur-web`) drives this; all protocol logic stays in Rust, so web
//! and mobile behave identically. Messages cross the boundary as `Uint8Array`.

use murmur_crypto::{Account, MurmurError};
use wasm_bindgen::prelude::*;

fn js_err(e: MurmurError) -> JsError {
    JsError::new(&e.to_string())
}

/// A local account, owned by the browser tab. Holds the key store, identity, and
/// joined groups in memory (persistent storage is a later phase).
#[wasm_bindgen]
pub struct WasmAccount {
    inner: Account,
}

#[wasm_bindgen]
impl WasmAccount {
    #[wasm_bindgen(constructor)]
    pub fn new(name: &str) -> Result<WasmAccount, JsError> {
        Account::new(name)
            .map(|inner| WasmAccount { inner })
            .map_err(js_err)
    }

    #[wasm_bindgen(js_name = keyPackage)]
    pub fn key_package(&self) -> Result<Vec<u8>, JsError> {
        self.inner.key_package().map_err(js_err)
    }

    #[wasm_bindgen(js_name = createGroup)]
    pub fn create_group(&mut self) -> Result<Vec<u8>, JsError> {
        self.inner.create_group().map_err(js_err)
    }

    #[wasm_bindgen(js_name = addMember)]
    pub fn add_member(&mut self, group_id: &[u8], key_package: &[u8]) -> Result<Vec<u8>, JsError> {
        self.inner.add_member(group_id, key_package).map_err(js_err)
    }

    #[wasm_bindgen(js_name = joinGroup)]
    pub fn join_group(&mut self, welcome: &[u8]) -> Result<Vec<u8>, JsError> {
        self.inner.join_group(welcome).map_err(js_err)
    }

    #[wasm_bindgen(js_name = removeMember)]
    pub fn remove_member(&mut self, group_id: &[u8], leaf_index: u32) -> Result<Vec<u8>, JsError> {
        self.inner.remove_member(group_id, leaf_index).map_err(js_err)
    }

    pub fn send(&mut self, group_id: &[u8], plaintext: &[u8]) -> Result<Vec<u8>, JsError> {
        self.inner.send(group_id, plaintext).map_err(js_err)
    }

    /// Returns the decrypted bytes for an application message, or `undefined` for
    /// handshake traffic.
    pub fn receive(
        &mut self,
        group_id: &[u8],
        message: &[u8],
    ) -> Result<Option<Vec<u8>>, JsError> {
        self.inner.receive(group_id, message).map_err(js_err)
    }

    #[wasm_bindgen(js_name = exporterSecret)]
    pub fn exporter_secret(
        &self,
        group_id: &[u8],
        label: &str,
        length: usize,
    ) -> Result<Vec<u8>, JsError> {
        self.inner
            .exporter_secret(group_id, label, length)
            .map_err(js_err)
    }

    #[wasm_bindgen(js_name = memberCount)]
    pub fn member_count(&self, group_id: &[u8]) -> Result<usize, JsError> {
        self.inner.member_count(group_id).map_err(js_err)
    }
}
