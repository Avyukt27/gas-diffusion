use std::sync::Arc;

#[cfg(target_arch = "wasm32")]
use winit::event_loop::{EventLoop, EventLoopProxy};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{KeyEvent, WindowEvent},
    keyboard::PhysicalKey,
    window::{CursorIcon, Icon, Window},
};

use crate::{
    renderer::Renderer,
    state::{HEIGHT, State, WIDTH},
};

pub struct App {
    #[cfg(target_arch = "wasm32")]
    proxy: Option<EventLoopProxy<State>>,
    state: Option<State>,
}

impl App {
    pub fn new(#[cfg(target_arch = "wasm32")] event_loop: &EventLoop<State>) -> Self {
        #[cfg(target_arch = "wasm32")]
        let proxy = Some(event_loop.create_proxy());
        Self {
            #[cfg(target_arch = "wasm32")]
            proxy,
            state: None,
        }
    }
}

impl ApplicationHandler<State> for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let icon = image::load_from_memory(include_bytes!("../favicon.png"))
            .unwrap()
            .into_rgba8();
        let (width, height) = icon.dimensions();
        let window_icon = Icon::from_rgba(icon.into_raw(), width, height).unwrap();

        #[allow(unused_mut)]
        let mut window_attributes = Window::default_attributes()
            .with_title("Diffusion Simulation Window")
            .with_window_icon(Some(window_icon))
            .with_cursor(CursorIcon::Crosshair)
            .with_inner_size(LogicalSize::new(WIDTH as f64, HEIGHT as f64))
            .with_resizable(false);

        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::{JsCast, UnwrapThrowExt};
            use winit::platform::web::WindowAttributesExtWebSys;

            let window = web_sys::window().unwrap_throw();
            let document = window.document().unwrap_throw();
            let canvas = document.get_element_by_id("canvas").unwrap_throw();
            let html_canvas_element = canvas.unchecked_into();
            window_attributes = window_attributes.with_canvas(Some(html_canvas_element));
        }

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        #[cfg(not(target_arch = "wasm32"))]
        {
            let renderer = pollster::block_on(Renderer::new(&window)).unwrap();
            let mut state = pollster::block_on(State::new(window.clone(), renderer)).unwrap();

            let initial_size = window.inner_size();
            state.resize(initial_size.width, initial_size.height);

            self.state = Some(state);
        }

        #[cfg(target_arch = "wasm32")]
        {
            if let Some(proxy) = self.proxy.take() {
                wasm_bindgen_futures::spawn_local(async move {
                    let renderer = Renderer::new(&window)
                        .await
                        .expect("Unable to create renderer");
                    let state = State::new(window, renderer)
                        .await
                        .expect("Unable to create canvas");
                    assert!(proxy.send_event(state).is_ok())
                });
            }
        }
    }

    #[allow(unused_mut)]
    fn user_event(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop, mut event: State) {
        #[cfg(target_arch = "wasm32")]
        {
            event.window().request_redraw();
            event.resize(
                event.window().inner_size().width,
                event.window().inner_size().height,
            );
        }
        self.state = Some(event);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let state = match &mut self.state {
            Some(canvas) => canvas,
            None => return,
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => state.resize(size.width, size.height),
            WindowEvent::RedrawRequested => {
                let _ = state.render();
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                ..
            } => state.handle_key(event_loop, code, key_state.is_pressed()),
            WindowEvent::CursorMoved { position, .. } => state.handle_mouse_move(position),
            WindowEvent::MouseInput {
                state: click_state,
                button,
                ..
            } => state.handle_mouse_click(click_state, button),
            _ => {}
        }
    }
}
