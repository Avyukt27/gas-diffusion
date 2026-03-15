use winit::window::Window;

pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,

    texture: wgpu::Texture,
    texture_view: wgpu::TextureView,
    sampler: wgpu::Sampler,

    bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
}

impl Renderer {
    pub fn new(window: &Window, sim_width: u32, sim_height: u32) {}
}
