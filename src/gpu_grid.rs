use std::sync::Arc;

use crate::grid::{DrawMode, Grid};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuCell {
    concentration: f32,
    advection_x: f32,
    advection_y: f32,
    wall: u32,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    delta: f32,
    diffusion_coefficient: f32,
    width: u32,
    height: u32,
}

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

    project_divergence_pipeline: wgpu::ComputePipeline,
    project_jacobi_pipeline: wgpu::ComputePipeline,
    project_gradient_pipeline: wgpu::ComputePipeline,
    advect_diffusion_pipeline: wgpu::ComputePipeline,

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

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Compute Shaders"),
            source: wgpu::ShaderSource::Wgsl(include_str!("simulation.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Simulation Pipeline Layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let project_divergence_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Divergence Pipeline"),
                layout: Some(&pipeline_layout),
                module: &shader,
                entry_point: Some("compute_divergence"),
                compilation_options: Default::default(),
                cache: None,
            });
        let project_jacobi_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Jacobi Solver Pipeline"),
                layout: Some(&pipeline_layout),
                module: &shader,
                entry_point: Some("compute_pressures"),
                compilation_options: Default::default(),
                cache: None,
            });
        let project_gradient_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Gradient Subtraction Pipeline"),
                layout: Some(&pipeline_layout),
                module: &shader,
                entry_point: Some("compute_gradients"),
                compilation_options: Default::default(),
                cache: None,
            });
        let advect_diffusion_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Advection Pipeline"),
                layout: Some(&pipeline_layout),
                module: &shader,
                entry_point: Some("advect_diffusion"),
                compilation_options: Default::default(),
                cache: None,
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
            project_divergence_pipeline,
            project_jacobi_pipeline,
            project_gradient_pipeline,
            advect_diffusion_pipeline,
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

    fn clear(&mut self) {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Grid Clear Encoder"),
            });

        encoder.clear_buffer(&self.buffer_a, 0, None);
        encoder.clear_buffer(&self.buffer_b, 0, None);
        encoder.clear_buffer(&self.sources, 0, None);
        encoder.clear_buffer(&self.pressures, 0, None);
        encoder.clear_buffer(&self.divergences, 0, None);

        self.queue.submit(std::iter::once(encoder.finish()));
        self.iteration_toggle = true;
    }

    fn update(&mut self, diffusion_coefficient: f64, delta: f64) {
        let uniforms = Uniforms {
            delta: delta as f32,
            diffusion_coefficient: diffusion_coefficient as f32,
            width: self.width as u32,
            height: self.height as u32,
        };
        self.queue
            .write_buffer(&self.uniforms, 0, bytemuck::bytes_of(&uniforms));

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Compute Pass Encoder"),
            });

        let workgroups_x = (self.width + 15) / 16;
        let workgroups_y = (self.height + 15) / 16;

        let current_bind_group = if self.iteration_toggle {
            &self.bind_group_a
        } else {
            &self.bind_group_b
        };

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Divergence Compute Pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&self.project_divergence_pipeline);
            compute_pass.set_bind_group(0, current_bind_group, &[]);
            compute_pass.dispatch_workgroups(workgroups_x as u32, workgroups_y as u32, 1u32);
        }
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Jacobi Compute Pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&self.project_jacobi_pipeline);
            compute_pass.set_bind_group(0, current_bind_group, &[]);
            compute_pass.dispatch_workgroups(workgroups_x as u32, workgroups_y as u32, 1u32);
        }
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Gradient Compute Pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&self.project_gradient_pipeline);
            compute_pass.set_bind_group(0, current_bind_group, &[]);
            compute_pass.dispatch_workgroups(workgroups_x as u32, workgroups_y as u32, 1u32);
        }
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Advections Compute Pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&self.advect_diffusion_pipeline);
            compute_pass.set_bind_group(0, current_bind_group, &[]);
            compute_pass.dispatch_workgroups(workgroups_x as u32, workgroups_y as u32, 1u32);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        self.iteration_toggle = !self.iteration_toggle;
    }

    fn inject(
        &mut self,
        start_x: usize,
        start_y: usize,
        prev_cell_x: usize,
        prev_cell_y: usize,
        delta: f64,
    ) {
        if start_x >= self.width || start_y >= self.height {
            return;
        }

        let idx = start_y * self.width + start_x;

        let target_buffer = if self.iteration_toggle {
            &self.buffer_a
        } else {
            &self.buffer_b
        };

        match self.draw_mode {
            DrawMode::Gas => {
                let byte_offset = (idx * 16) as wgpu::BufferAddress;
                self.queue.write_buffer(
                    target_buffer,
                    byte_offset,
                    bytemuck::bytes_of(&(self.draw_intensity as f32)),
                );
            }
            DrawMode::Source | DrawMode::Sink => {
                let rate = if matches!(self.draw_mode, DrawMode::Source) {
                    self.draw_intensity.abs() / 100.0
                } else {
                    -self.draw_intensity.abs() / 100.0
                } as f32;
                let byte_offset = (idx * 4) as wgpu::BufferAddress;
                self.queue
                    .write_buffer(&self.sources, byte_offset, bytemuck::bytes_of(&rate));
            }
            DrawMode::Advection => {
                let dx = start_x as f64 - prev_cell_x as f64;
                let dy = start_y as f64 - prev_cell_y as f64;
                let strength = 5.0;
                let max_vel = self.cell_size as f64 / delta * 0.5;

                let byte_offset = (idx * 16 + 4) as wgpu::BufferAddress;
                let data = [
                    (dx * strength).clamp(-max_vel, max_vel) as f32,
                    (dy * strength).clamp(-max_vel, max_vel) as f32,
                ];
                self.queue
                    .write_buffer(target_buffer, byte_offset, bytemuck::cast_slice(&data));
            }
            DrawMode::Stopper => {
                let byte_offset = (idx * 16 + 12) as wgpu::BufferAddress;
                self.queue
                    .write_buffer(&self.buffer_a, byte_offset, bytemuck::bytes_of(&1u32));
                self.queue
                    .write_buffer(&self.buffer_b, byte_offset, bytemuck::bytes_of(&1u32));
            }
        }
    }
}
