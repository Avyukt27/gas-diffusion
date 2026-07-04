use std::sync::Arc;

#[cfg(target_arch = "wasm32")]
use winit::event_loop::{EventLoop, EventLoopProxy};
use winit::{
    application::ApplicationHandler,
    event::{KeyEvent, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

use crate::renderer::Renderer;
use crate::state::State;

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
        #[allow(unused_mut)]
        let mut window_attributes =
            Window::default_attributes().with_title("Diffusion Simulation Window");

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
            self.state = Some(pollster::block_on(State::new(window, renderer)).unwrap());
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
            event.window.request_redraw();
            event.resize(
                event.window.inner_size().width,
                event.window.inner_size().height,
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
                state.render();
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
            //         WindowEvent::CursorMoved { position, .. } => {
            //             self.mouse_position = position;
            //             if self.mouse_down {
            //                 let cell_x = self.mouse_position.x as usize / self.grid.cell_size;
            //                 let cell_y = self.mouse_position.y as usize / self.grid.cell_size;
            //
            //                 let prev_cell_x = self.prev_mouse_position.x as usize / self.grid.cell_size;
            //                 let prev_cell_y = self.prev_mouse_position.y as usize / self.grid.cell_size;
            //
            //                 if cell_x < self.grid.width
            //                     && cell_y < self.grid.height
            //                     && prev_cell_x < self.grid.width
            //                     && prev_cell_y < self.grid.height
            //                 {
            //                     self.apply_brush(
            //                         cell_x,
            //                         cell_y,
            //                         prev_cell_x,
            //                         prev_cell_y,
            //                         self.draw_size,
            //                         self.draw_size,
            //                     );
            //                 }
            //             }
            //             self.prev_mouse_position = position;
            //         }
            //         WindowEvent::MouseInput { state, button, .. } => {
            //             if button == MouseButton::Left {
            //                 self.mouse_down = state.is_pressed();
            //                 if self.mouse_down {
            //                     let cell_x = self.mouse_position.x as usize / self.grid.cell_size;
            //                     let cell_y = self.mouse_position.y as usize / self.grid.cell_size;
            //
            //                     if cell_x < self.grid.width && cell_y < self.grid.height {
            //                         self.apply_brush(
            //                             cell_x,
            //                             cell_y,
            //                             cell_x,
            //                             cell_y,
            //                             self.draw_size,
            //                             self.draw_size,
            //                         );
            //                     }
            //                 }
            //             }
            //         }
            //         WindowEvent::MouseWheel { delta, .. } => {
            //             let scroll_y = match delta {
            //                 MouseScrollDelta::LineDelta(_, y) => y as f64,
            //                 MouseScrollDelta::PixelDelta(pos) => pos.y / 50.0,
            //             };
            //
            //             if scroll_y > 0.0 {
            //                 self.draw_size += 1;
            //             } else if scroll_y < 0.0 && self.draw_size > 1 {
            //                 self.draw_size -= 1;
            //             }
            //         }
            _ => {}
        }
    }
}
