use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;
use wgpu_launchpad::{wgpu, PhysicalSize, Scene};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Vertex {
    pub pos: [f32; 2],
    pub color: [f32; 3],
}
unsafe impl Zeroable for Vertex {}
unsafe impl Pod for Vertex {}

pub struct Renderer {
    triangle_pipeline: wgpu::RenderPipeline,
    triangle_vertex_buf: wgpu::Buffer,
    triangle_index_buf: wgpu::Buffer,
    n_triangle_indices: u32,
    vertex_bind_group: wgpu::BindGroup,
    fragment_bind_group: wgpu::BindGroup,
    camera_ubo: wgpu::Buffer,
    animation_ubo: wgpu::Buffer,
}

pub struct Args {
    pub triangle_vertices: Vec<Vertex>,
    pub triangle_indices: Vec<u16>,
}

impl Scene for Renderer {
    type Args = Args;

    fn new(device: &wgpu::Device, args: Self::Args) -> Renderer {
        // Create buffers
        let triangle_vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Triangle vertex Buffer"),
            contents: bytemuck::cast_slice(&args.triangle_vertices),
            usage: wgpu::BufferUsage::VERTEX,
        });

        let triangle_index_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Triangle index Buffer"),
            contents: bytemuck::cast_slice(&args.triangle_indices),
            usage: wgpu::BufferUsage::INDEX,
        });

        // Uniform buffers
        let camera_ubo = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera UBO"),
            size: std::mem::size_of::<[f32; 16]>() as _,
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            mapped_at_creation: false,
        });

        let animation_ubo = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Animation UBO"),
            size: std::mem::size_of::<f32>() as _,
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            mapped_at_creation: false,
        });

        // Bind group layout (basically a descriptorset layout)
        let vertex_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::UniformBuffer {
                        dynamic: false,
                        min_binding_size: wgpu::BufferSize::new(
                            std::mem::size_of::<[f32; 16]>() as u64
                        ),
                    },
                    count: None,
                }],
            });

        let index_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[wgpu::BindGroupLayoutEntry {
                    // This might need its own bind group entry...
                    binding: 0,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::UniformBuffer {
                        dynamic: false,
                        min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<f32>() as u64),
                    },
                    count: None,
                }],
            });

        // Vertex descriptor
        let vertex_state = wgpu::VertexStateDescriptor {
            index_format: wgpu::IndexFormat::Uint16,
            vertex_buffers: &[wgpu::VertexBufferDescriptor {
                stride: std::mem::size_of::<Vertex>() as _,
                step_mode: wgpu::InputStepMode::Vertex,
                attributes: &[
                    wgpu::VertexAttributeDescriptor {
                        format: wgpu::VertexFormat::Float2,
                        offset: 0,
                        shader_location: 0,
                    },
                    wgpu::VertexAttributeDescriptor {
                        format: wgpu::VertexFormat::Float3,
                        offset: 2 * std::mem::size_of::<f32>() as u64,
                        shader_location: 1,
                    },
                ],
            }],
        };

        // Shader modules
        let vs_module =
            device.create_shader_module(wgpu::include_spirv!("shaders/shader.vert.spv"));
        let fs_module =
            device.create_shader_module(wgpu::include_spirv!("shaders/shader.frag.spv"));

        // Pipeline
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            push_constant_ranges: &[],
            bind_group_layouts: &[&vertex_bind_group_layout, &index_bind_group_layout],
        });

        // Create bind group for the uniform
        let vertex_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &vertex_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(camera_ubo.slice(..)),
            }],
            label: None,
        });

        let fragment_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &index_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(animation_ubo.slice(..)),
            }],
            label: None,
        });

        let triangle_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &fs_module,
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::None,
                ..Default::default()
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &[wgpu::ColorStateDescriptor {
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],
            depth_stencil_state: None,
            vertex_state: vertex_state.clone(),
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        Self {
            triangle_pipeline,
            triangle_vertex_buf,
            triangle_index_buf,
            vertex_bind_group,
            fragment_bind_group,
            n_triangle_indices: args.triangle_indices.len() as _,
            camera_ubo,
            animation_ubo,
        }
    }

    fn draw(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        size: PhysicalSize<u32>,
        queue: &wgpu::Queue,
    ) {
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });
        rpass.set_pipeline(&self.triangle_pipeline);
        rpass.set_vertex_buffer(0, self.triangle_vertex_buf.slice(..));
        rpass.set_index_buffer(self.triangle_index_buf.slice(..));
        rpass.set_bind_group(0, &self.vertex_bind_group, &[]);
        rpass.set_bind_group(1, &self.fragment_bind_group, &[]);
        rpass.draw_indexed(0..self.n_triangle_indices, 0, 0..1);
        queue.write_buffer(
            &self.camera_ubo,
            0,
            bytemuck::cast_slice(&ortho_camera(size)),
        );
        std::thread::sleep(std::time::Duration::from_micros(16_667));
    }
}

// TODO: Make this DPI-aware...
fn ortho_camera(size: PhysicalSize<u32>) -> [f32; 16] {
    let w = 2.0 / size.width as f32;
    let h = 2.0 / size.height as f32;
    [
        w, 0.0, 0.0, 0.0, //
        0.0, h, 0.0, 0.0, //
        0.0, 0.0, 1.0, 0.0, //
        -1.0, -1.0, 0.0, 1.0, //
    ]
    /*
    glLoadIdentity();
    float min = std::min(input.tl().x, input.tl().y);
    float max = std::max(input.br().x, input.br().y);
    glOrtho(min, max, min, max, max_z, -max_z);
    */
}
