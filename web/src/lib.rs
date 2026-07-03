use wasm_bindgen::prelude::*;
use winit::event_loop::EventLoop;
use winit::platform::web::EventLoopExtWebSys;

#[wasm_bindgen]
pub fn start() {
    console_error_panic_hook::set_once();
    let event_loop = EventLoop::builder()
        .build()
        .expect("Failed to build event loop");
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    let app = core::app::App::new();
    event_loop.spawn_app(app);
}
