//! Bindgen entry point. Run with the cdylib already built:
//!   cargo run --bin uniffi-bindgen -- generate \
//!     --library target/debug/murmur_uniffi_spike.dll \
//!     --language kotlin --out-dir bindings
fn main() {
    uniffi::uniffi_bindgen_main()
}
