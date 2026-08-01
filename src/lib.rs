#[cfg(target_arch = "wasm32")]
use wasm_bindgen::UnwrapThrowExt;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;
use winit::event_loop::EventLoop;

pub mod app;
pub mod cpu_grid;
pub mod gpu_grid;
pub mod grid;
pub mod renderer;
pub mod state;

use crate::app::App;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn run_web() -> Result<(), wasm_bindgen::JsValue> {
    console_error_panic_hook::set_once();
    run().unwrap_throw();

    Ok(())
}

pub fn run() -> anyhow::Result<()> {
    #[cfg(not(target_arch = "wasm32"))]
    env_logger::init();
    #[cfg(target_arch = "wasm32")]
    console_log::init_with_level(log::Level::Info).unwrap_throw();

    let event_loop = EventLoop::with_user_event().build()?;
    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut app = App::new();
        event_loop.run_app(&mut app)?;
    }
    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::EventLoopExtWebSys;

        let app = App::new(&event_loop);
        event_loop.spawn_app(app);
    }

    Ok(())
}
