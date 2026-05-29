//! Spike C — prove the uniffi seam (Rust → Kotlin/Swift) works on this machine.
//!
//! Mobile-day-one requires the crypto core to expose uniffi bindings. This spike
//! exports a trivial function + object via uniffi proc-macros and is paired with
//! a bindgen step that emits Kotlin (Android) and Swift (iOS) bindings. Proving
//! codegen here is platform-independent; cross-compiling the device library
//! needs the Android NDK (Android) or macOS (iOS) and is a later step.

uniffi::setup_scaffolding!();

/// Smoke export — the simplest possible function crossing the FFI boundary.
#[uniffi::export]
pub fn murmur_ffi_smoke() -> String {
    "murmur uniffi seam ok".to_string()
}

/// A stand-in for the future MLS session handle that mobile clients will hold.
#[derive(uniffi::Object)]
pub struct SessionHandle {
    label: String,
}

#[uniffi::export]
impl SessionHandle {
    #[uniffi::constructor]
    pub fn new(label: String) -> Self {
        Self { label }
    }

    pub fn label(&self) -> String {
        self.label.clone()
    }
}
