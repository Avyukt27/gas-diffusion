use std::cell::RefCell;
use wasm_bindgen::prelude::*;
use winit::event_loop::EventLoop;
use winit::platform::web::EventLoopExtWebSys;

thread_local! {
    static EVENT_LOOP: RefCell<Option<EventLoop<()>>> = const { RefCell::new(None) };
}

#[wasm_bindgen]
pub fn start() {
    console_error_panic_hook::set_once();

    let mut el_opt = EVENT_LOOP.with(|el| el.borrow_mut().take());

    if el_opt.is_none() {
        if let Ok(event_loop) = EventLoop::builder().build() {
            event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
            el_opt = Some(event_loop);
        }
    }

    if let Some(event_loop) = el_opt {
        let app = core::app::App::new();
        event_loop.spawn_app(app);
    }
}
