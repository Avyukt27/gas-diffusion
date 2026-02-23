use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn run() {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
}
