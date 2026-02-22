use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn run() {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
}
