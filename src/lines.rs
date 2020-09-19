use wgpu_launchpad::{Scene, wgpu};
use wgpu::util::DeviceExt;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Vertex {
    pub pos: [f32; 2],
    pub color: [f32; 3],
}
unsafe impl Zeroable for Vertex {}
unsafe impl Pod for Vertex {}

pub struct Lines {
    triangle_pipeline: wgpu::RenderPipeline,
    triangle_vertex_buf: wgpu::Buffer,
    triangle_index_buf: wgpu::Buffer,
    n_triangle_indices: u32,
}

pub struct Args {
    pub triangle_vertices: Vec<Vertex>,
    pub triangle_indices: Vec<u16>,
}

impl Scene for Lines {
    type Args = Args;

    fn new(device: &wgpu::Device, args: Self::Args) -> Lines {
        // Create buffers
        let triangle_vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Triangle vertex Buffer"),
            contents: bytemuck::cast_slice(&args.triangle_vertices),
            usage: wgpu::BufferUsage::VERTEX,
        });

        let triangle_index_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Triangle index Buffer"),
            contents: bytemuck::cast_slice(&args.triangle_indices),
            usage: wgpu::BufferUsage::VERTEX,
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
        let vs_module = device.create_shader_module(wgpu::include_spirv!("shaders/shader.vert.spv"));
        let fs_module = device.create_shader_module(wgpu::include_spirv!("shaders/shader.frag.spv"));

        // Pipeline
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            push_constant_ranges: &[],
            bind_group_layouts: &[],
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

        Lines { 
            triangle_pipeline,
            triangle_vertex_buf,
            triangle_index_buf,
            n_triangle_indices: args.triangle_indices.len() as _,
        }
    }

    fn draw(
        &mut self, 
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
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
        rpass.draw_indexed(0..self.n_triangle_indices, 0, 0..1);
    }
}
