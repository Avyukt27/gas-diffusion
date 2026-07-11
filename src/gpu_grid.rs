use std::sync::Arc;

use crate::grid::{DrawMode, Grid};

pub struct GpuGrid {
    width: usize,
    height: usize,
    cell_size: usize,
    draw_mode: DrawMode,
    draw_intensity: f64,

    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,

    buffer_a: wgpu::Buffer,
    buffer_b: wgpu::Buffer,
    sources: wgpu::Buffer,
    pressures: wgpu::Buffer,
    divergences: wgpu::Buffer,
    uniforms: wgpu::Buffer,

    // project_divergence_pipeline: wgpu::ComputePipeline,
    // project_jacobi_pipeline: wgpu::ComputePipeline,
    // project_gradient_pipeline: wgpu::ComputePipeline,
    // advect_diffusion_pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group_a: wgpu::BindGroup,
    bind_group_b: wgpu::BindGroup,

    iteration_toggle: bool,
}

impl GpuGrid {
    pub fn new(
        width: usize,
        height: usize,
        cell_size: usize,
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
    ) -> Self {
        let grid_width = width / cell_size;
        let grid_height = height / cell_size;
        let cell_count = grid_width * grid_height;
        let cell_byte_size = 16;

        let grid_buffer_bytes = (cell_count * cell_byte_size) as wgpu::BufferAddress;

        let buffer_a = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Grid Buffer A"),
            size: grid_buffer_bytes,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let buffer_b = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Grid Buffer B"),
            size: grid_buffer_bytes,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let sources = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Grid Buffer B"),
            size: grid_buffer_bytes,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let pressures = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Grid Buffer B"),
            size: grid_buffer_bytes,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let divergences = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Grid Buffer B"),
            size: grid_buffer_bytes,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let uniforms = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Grid Buffer B"),
            size: grid_buffer_bytes,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Grid Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let bind_group_a = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Grid Bind Group A"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer_a.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: buffer_b.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: sources.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: pressures.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: divergences.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: uniforms.as_entire_binding(),
                },
            ],
        });
        let bind_group_b = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Grid Bind Group B"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer_b.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: buffer_a.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: sources.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: pressures.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: divergences.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: uniforms.as_entire_binding(),
                },
            ],
        });

        Self {
            width: grid_width,
            height: grid_height,
            cell_size,
            draw_mode: DrawMode::Gas,
            draw_intensity: 1.0,
            device,
            queue,
            buffer_a,
            buffer_b,
            sources,
            pressures,
            divergences,
            uniforms,
            bind_group_layout,
            bind_group_a,
            bind_group_b,
            iteration_toggle: false,
        }
    }
}

impl Grid for GpuGrid {
    fn width(&self) -> usize {
        self.width
    }
    fn height(&self) -> usize {
        self.height
    }
    fn cell_size(&self) -> usize {
        self.cell_size
    }
    fn draw_mode(&self) -> &DrawMode {
        &self.draw_mode
    }
    fn draw_intensity(&self) -> f64 {
        self.draw_intensity
    }
    fn concentrations(&self) -> &[f64] {
        &[0.0]
    }
    fn walls(&self) -> &[u8] {
        &[0]
    }

    fn set_draw_mode(&mut self) {
        self.draw_mode = match self.draw_mode {
            DrawMode::Gas => DrawMode::Source,
            DrawMode::Source => DrawMode::Sink,
            DrawMode::Sink => DrawMode::Advection,
            DrawMode::Advection => DrawMode::Stopper,
            DrawMode::Stopper => DrawMode::Gas,
        };
    }
    fn set_draw_intensity(&mut self, intensity: f64) {
        self.draw_intensity = (self.draw_intensity + intensity).clamp(0.0, 1.0);
    }

    fn clear(&mut self) {}

    fn update(&mut self, diffusion_coefficient: f64, delta: f64) {}

    fn inject(
        &mut self,
        start_x: usize,
        start_y: usize,
        prev_cell_x: usize,
        prev_cell_y: usize,
        delta: f64,
    ) {
    }
}
