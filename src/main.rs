#[cfg(not(target_arch = "wasm32"))]
fn main() {
    diffusion::native::run();
}

#[cfg(target_arch = "wasm32")]
fn main() {
    diffusion::wasm::run();
}
