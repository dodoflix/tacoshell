use wasm_bindgen::prelude::*;

// Re-export core engine functions as WASM bindings.
// Each function mirrors the Tauri IPC command surface so the UI layer
// can use the same TypeScript types on both desktop and web.
//
// Implementations added incrementally alongside the corresponding
// core feature (Phase 1.3 → crypto, Phase 1.4 → storage, etc.)

#[wasm_bindgen(start)]
pub fn init() {
    // Set up panic hook so Rust panics show readable messages in the browser console
    console_error_panic_hook::set_once();
}
