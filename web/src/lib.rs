use wasm_bindgen::prelude::*;
use winit::platform::web::EventLoopExtWebSys;

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    let event_loop = winit::event_loop::EventLoop::new().unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    let app = core::app::App::new();
    event_loop.spawn_app(app);
}
