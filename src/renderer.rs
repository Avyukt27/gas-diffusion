use std::sync::Arc;
use winit::window::Window;

pub struct Renderer {
    // pub width: u32,
    // pub height: u32,
    // pub cell_size: u32,
    //
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    //
    // pub bind_group_layout: wgpu::BindGroupLayout,
    // pub bind_group: wgpu::BindGroup,
    // pub texture: wgpu::Texture,
    //
    // pub pipeline: wgpu::RenderPipeline,
}

impl Renderer {
    pub async fn new(window: &Arc<Window>) -> anyhow::Result<Self> {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::GL,
            flags: Default::default(),
            memory_budget_thresholds: Default::default(),
            backend_options: Default::default(),
        });

        let surface = instance.create_surface(window.clone()).unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await?;

        let info = adapter.get_info();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Simulation Device"),
                required_features: wgpu::Features::empty(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                required_limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        // surface.configure(&device, &config);
        //
        // let grid_width = width / cell_size;
        // let grid_height = height / cell_size;
        //
        // let texture_descriptor = wgpu::TextureDescriptor {
        //     size: wgpu::Extent3d {
        //         width: grid_width.max(1),
        //         height: grid_height.max(1),
        //         depth_or_array_layers: 1,
        //     },
        //     mip_level_count: 1,
        //     sample_count: 1,
        //     dimension: wgpu::TextureDimension::D2,
        //     format: wgpu::TextureFormat::R32Float,
        //     usage: wgpu::TextureUsages::TEXTURE_BINDING
        //         | wgpu::TextureUsages::COPY_DST
        //         | wgpu::TextureUsages::RENDER_ATTACHMENT,
        //     label: Some("Simulation Texture"),
        //     view_formats: &[],
        // };
        //
        // let texture = device.create_texture(&texture_descriptor);
        // let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        //
        // let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        //     label: Some("Shader"),
        //     source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        // });
        //
        // let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        //     entries: &[
        //         wgpu::BindGroupLayoutEntry {
        //             binding: 0,
        //             visibility: wgpu::ShaderStages::FRAGMENT,
        //             ty: wgpu::BindingType::Texture {
        //                 multisampled: false,
        //                 view_dimension: wgpu::TextureViewDimension::D2,
        //                 sample_type: wgpu::TextureSampleType::Float { filterable: false },
        //             },
        //             count: None,
        //         },
        //         wgpu::BindGroupLayoutEntry {
        //             binding: 1,
        //             visibility: wgpu::ShaderStages::FRAGMENT,
        //             ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
        //             count: None,
        //         },
        //     ],
        //     label: Some("Simulation Bind Group Layout"),
        // });
        //
        // let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        //     label: Some("Render Pipeline Layout"),
        //     bind_group_layouts: &[&bind_group_layout],
        //     push_constant_ranges: &[],
        // });
        //
        // let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        //     label: Some("Render Pipeline"),
        //     layout: Some(&pipeline_layout),
        //     vertex: wgpu::VertexState {
        //         module: &shader,
        //         entry_point: Some("vtx_main"),
        //         buffers: &[],
        //         compilation_options: Default::default(),
        //     },
        //     fragment: Some(wgpu::FragmentState {
        //         module: &shader,
        //         entry_point: Some("frag_main"),
        //         targets: &[Some(wgpu::ColorTargetState {
        //             format: surface_format,
        //             blend: Some(wgpu::BlendState::REPLACE),
        //             write_mask: wgpu::ColorWrites::ALL,
        //         })],
        //         compilation_options: Default::default(),
        //     }),
        //     primitive: wgpu::PrimitiveState {
        //         topology: wgpu::PrimitiveTopology::TriangleList,
        //         strip_index_format: None,
        //         front_face: wgpu::FrontFace::Ccw,
        //         cull_mode: Some(wgpu::Face::Back),
        //         polygon_mode: wgpu::PolygonMode::Fill,
        //         unclipped_depth: false,
        //         conservative: false,
        //     },
        //     depth_stencil: None,
        //     multisample: wgpu::MultisampleState {
        //         count: 1,
        //         mask: !0,
        //         alpha_to_coverage_enabled: false,
        //     },
        //     multiview: None,
        //     cache: None,
        // });
        //
        // let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        //     address_mode_u: wgpu::AddressMode::ClampToEdge,
        //     address_mode_v: wgpu::AddressMode::ClampToEdge,
        //     mag_filter: wgpu::FilterMode::Nearest,
        //     min_filter: wgpu::FilterMode::Nearest,
        //     mipmap_filter: wgpu::FilterMode::Nearest,
        //     ..Default::default()
        // });
        //
        // let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        //     layout: &bind_group_layout,
        //     entries: &[
        //         wgpu::BindGroupEntry {
        //             binding: 0,
        //             resource: wgpu::BindingResource::TextureView(&texture_view),
        //         },
        //         wgpu::BindGroupEntry {
        //             binding: 1,
        //             resource: wgpu::BindingResource::Sampler(&sampler),
        //         },
        //     ],
        //     label: Some("Simulation Bind Group"),
        // });
        //
        // #[cfg(target_arch = "wasm32")]
        // web_sys::console::log_1(
        //     &format!(
        //         "wgpu Backend Active: {:?} | Driver: {} | Device: {}\nWidth: {}\nHeight: {}",
        //         info.backend, info.driver, info.name, width, height
        //     )
        //     .into(),
        // );

        Ok(Self {
            // width,
            // height,
            // cell_size,
            surface,
            device,
            queue,
            config,
            // bind_group_layout,
            // bind_group,
            // texture,
            // pipeline,
        })
    }

    // pub fn render(&self) {
    //     let output = self.surface.get_current_texture().unwrap();
    //     let view = output
    //         .texture
    //         .create_view(&wgpu::TextureViewDescriptor::default());
    //     let mut encoder = self
    //         .device
    //         .create_command_encoder(&wgpu::CommandEncoderDescriptor {
    //             label: Some("Render Encoder"),
    //         });
    //
    //     {
    //         let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
    //             label: Some("Render Pass"),
    //             color_attachments: &[Some(wgpu::RenderPassColorAttachment {
    //                 view: &view,
    //                 resolve_target: None,
    //                 ops: wgpu::Operations {
    //                     load: wgpu::LoadOp::Clear(wgpu::Color {
    //                         r: 0.0,
    //                         g: 0.5,
    //                         b: 1.0,
    //                         a: 1.0,
    //                     }),
    //                     store: wgpu::StoreOp::Store,
    //                 },
    //                 depth_slice: None,
    //             })],
    //             depth_stencil_attachment: None,
    //             occlusion_query_set: None,
    //             timestamp_writes: None,
    //         });
    //
    //         render_pass.set_pipeline(&self.pipeline);
    //         render_pass.set_bind_group(0, &self.bind_group, &[]);
    //         render_pass.draw(0..6, 0..1);
    //     }
    //
    //     self.queue.submit(std::iter::once(encoder.finish()));
    //     output.present();
    // }
    //
    // pub fn upload_texture(
    //     &self,
    //     concentrations: &[f64],
    //     walls: &[bool],
    //     sim_width: u32,
    //     sim_height: u32,
    // ) {
    //     let bytes_per_pixel = 4;
    //     let mut packed_buffer = vec![0.0f32; (sim_width * sim_height) as usize];
    //
    //     for y in 0..sim_height as usize {
    //         let src_start = y * sim_width as usize;
    //         for x in 0..sim_width as usize {
    //             let sim_idx = src_start + x;
    //             if sim_idx < concentrations.len() {
    //                 if walls[sim_idx] {
    //                     packed_buffer[sim_idx] = -1.0;
    //                 } else {
    //                     packed_buffer[sim_idx] = concentrations[sim_idx] as f32;
    //                 }
    //             }
    //         }
    //     }
    //
    //     self.queue.write_texture(
    //         wgpu::TexelCopyTextureInfo {
    //             texture: &self.texture,
    //             mip_level: 0,
    //             origin: wgpu::Origin3d::ZERO,
    //             aspect: wgpu::TextureAspect::All,
    //         },
    //         bytemuck::cast_slice(&packed_buffer),
    //         wgpu::TexelCopyBufferLayout {
    //             offset: 0,
    //             bytes_per_row: Some(sim_width * bytes_per_pixel),
    //             rows_per_image: Some(sim_height),
    //         },
    //         wgpu::Extent3d {
    //             width: sim_width,
    //             height: sim_height,
    //             depth_or_array_layers: 1,
    //         },
    //     );
    // }
    //
    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            // self.width = width;
            //         self.height = height;
            //
            self.config.width = width.min(2048);
            self.config.height = height.min(2048);
            self.surface.configure(&self.device, &self.config);
            //
            //         let grid_width = (width / self.cell_size).max(1);
            //         let grid_height = (height / self.cell_size).max(1);
            //
            //         let texture_descriptor = wgpu::TextureDescriptor {
            //             size: wgpu::Extent3d {
            //                 width: grid_width,
            //                 height: grid_height,
            //                 depth_or_array_layers: 1,
            //             },
            //             mip_level_count: 1,
            //             sample_count: 1,
            //             dimension: wgpu::TextureDimension::D2,
            //             format: wgpu::TextureFormat::R32Float,
            //             usage: wgpu::TextureUsages::TEXTURE_BINDING
            //                 | wgpu::TextureUsages::COPY_DST
            //                 | wgpu::TextureUsages::RENDER_ATTACHMENT,
            //             label: Some("Simulation Texture (Resized)"),
            //             view_formats: &[],
            //         };
            //
            //         self.texture = self.device.create_texture(&texture_descriptor);
            //         let texture_view = self
            //             .texture
            //             .create_view(&wgpu::TextureViewDescriptor::default());
            //
            //         let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            //             address_mode_u: wgpu::AddressMode::ClampToEdge,
            //             address_mode_v: wgpu::AddressMode::ClampToEdge,
            //             mag_filter: wgpu::FilterMode::Nearest,
            //             min_filter: wgpu::FilterMode::Nearest,
            //             mipmap_filter: wgpu::FilterMode::Nearest,
            //             ..Default::default()
            //         });
            //
            //         self.bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            //             layout: &self.bind_group_layout,
            //             entries: &[
            //                 wgpu::BindGroupEntry {
            //                     binding: 0,
            //                     resource: wgpu::BindingResource::TextureView(&texture_view),
            //                 },
            //                 wgpu::BindGroupEntry {
            //                     binding: 1,
            //                     resource: wgpu::BindingResource::Sampler(&sampler),
            //                 },
            //             ],
            //             label: Some("Simulation Bind Group (Resized)"),
            //         });
        }
    }
}
