use winit::event_loop::EventLoop;

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    let mut app = core::app::App::new();
    event_loop.run_app(&mut app).unwrap();
}
