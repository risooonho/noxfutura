use crate::modes::playgame::CameraUniform;
use crate::utils::Frustrum;
use crate::{components::*, modes::playgame::GBuffer};
use bengine::*;
use legion::*;

// The idea here is to build geometry for grass layers above
// the terrain in a cross-hatch pattern.

pub struct GrassPass {
    grass_buffer: FloatBuffer<f32>,
    pipeline: gpu::RenderPipeline,
    bind_group: gpu::BindGroup,
    pub models_changed: bool,
}

impl GrassPass {
    pub fn new(uniforms: &CameraUniform) -> Self {
        // Shader
        let (grass_vert, grass_frag) = helpers::shader_from_bytes(
            bengine::gpu::include_spirv!("grass.vert.spv"),
            bengine::gpu::include_spirv!("grass.frag.spv"),
        );

        // Texture
        let mut tlock = TEXTURES.write();
        let tex_id = tlock.load_texture_from_bytes(include_bytes!("grass.png"), "Grass");

        // Make template grass buffer
        let mut grass_buffer = FloatBuffer::new(&[3, 2], 1000, gpu::BufferUsage::VERTEX);
        grass_buffer.add3(0.0, 0.0, 0.0);
        grass_buffer.add2(0.0, 0.0);
        grass_buffer.build();

        let dl = RENDER_CONTEXT.read();
        let ctx = dl.as_ref().unwrap();

        let bind_group_layout =
            ctx.device
                .create_bind_group_layout(&gpu::BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[
                        gpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: gpu::ShaderStage::VERTEX,
                            ty: gpu::BindingType::UniformBuffer {
                                dynamic: false,
                                min_binding_size: gpu::BufferSize::new(64),
                            },
                            count: None,
                        },
                        gpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: gpu::ShaderStage::FRAGMENT,
                            ty: gpu::BindingType::SampledTexture {
                                multisampled: false,
                                dimension: gpu::TextureViewDimension::D2,
                                component_type: gpu::TextureComponentType::Uint,
                            },
                            count: None,
                        },
                        gpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: gpu::ShaderStage::FRAGMENT,
                            ty: gpu::BindingType::Sampler { comparison: false },
                            count: None,
                        },
                    ],
                });

        let bind_group = ctx.device.create_bind_group(&gpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                gpu::BindGroupEntry {
                    binding: 0,
                    resource: gpu::BindingResource::Buffer(uniforms.uniform_buffer.slice(..)),
                },
                gpu::BindGroupEntry {
                    binding: 1,
                    resource: gpu::BindingResource::TextureView(tlock.get_view(tex_id)),
                },
                gpu::BindGroupEntry {
                    binding: 2,
                    resource: gpu::BindingResource::Sampler(tlock.get_sampler(tex_id)),
                },
            ],
        });

        let pipeline_layout = ctx
            .device
            .create_pipeline_layout(&gpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });
        let pipeline = ctx
            .device
            .create_render_pipeline(&gpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                vertex_stage: gpu::ProgrammableStageDescriptor {
                    module: SHADERS.read().get_module(grass_vert),
                    entry_point: "main",
                },
                fragment_stage: Some(gpu::ProgrammableStageDescriptor {
                    module: SHADERS.read().get_module(grass_frag),
                    entry_point: "main",
                }),
                rasterization_state: Some(gpu::RasterizationStateDescriptor {
                    front_face: gpu::FrontFace::Ccw,
                    cull_mode: gpu::CullMode::None,
                    ..Default::default()
                }),
                primitive_topology: gpu::PrimitiveTopology::TriangleList,
                color_states: &[
                    gpu::ColorStateDescriptor {
                        format: ctx.swapchain_format,
                        color_blend: gpu::BlendDescriptor::REPLACE,
                        alpha_blend: gpu::BlendDescriptor::REPLACE,
                        write_mask: gpu::ColorWrite::ALL,
                    },
                    gpu::ColorStateDescriptor {
                        format: ctx.swapchain_format,
                        color_blend: gpu::BlendDescriptor::REPLACE,
                        alpha_blend: gpu::BlendDescriptor::REPLACE,
                        write_mask: gpu::ColorWrite::ALL,
                    },
                ],
                depth_stencil_state: Some(gpu::DepthStencilStateDescriptor {
                    format: gpu::TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: gpu::CompareFunction::Less,
                    stencil: gpu::StencilStateDescriptor::default(),
                }),
                vertex_state: gpu::VertexStateDescriptor {
                    index_format: gpu::IndexFormat::Uint16,
                    vertex_buffers: &[grass_buffer.descriptor()],
                },
                sample_count: 1,
                sample_mask: !0,
                alpha_to_coverage_enabled: false,
            });

        Self {
            grass_buffer,
            bind_group,
            pipeline,
            models_changed: true,
        }
    }

    pub fn render(&mut self, core: &Core, ecs: &mut World, frustrum: &Frustrum, gbuffer: &GBuffer) {
        if self.models_changed {
            let camera_z = <(&Position, &CameraOptions)>::query()
                .iter(ecs)
                .map(|(pos, _)| pos.as_point3())
                .nth(0)
                .unwrap()
                .z;

            self.grass_buffer.clear();
            <(&Vegetation, &Position)>::query()
                .iter(ecs)
                .for_each(|(_, pos)| {
                    if let Some(pt) = pos.as_point3_only_tile() {
                        if pt.z <= camera_z
                            && pt.z > camera_z - 50
                            && frustrum.check_sphere(&pos.as_vec3(), 2.0)
                        {
                            // Insert geometry
                            let bx = pt.x as f32 + 0.5;
                            let bz = pt.z as f32;
                            const HEIGHT: f32 = 0.5;
                            const GRASS_SPACING: f32 = 0.2;

                            let mut by = pt.y as f32;
                            while by < pt.y as f32 + 1.01 {
                                self.grass_buffer.add3(bx - 0.5, bz, by);
                                self.grass_buffer.add2(0.0, 0.0);
                                self.grass_buffer.add3(bx + 0.5, bz, by);
                                self.grass_buffer.add2(1.0, 0.0);
                                self.grass_buffer.add3(bx + 0.5, bz + HEIGHT, by);
                                self.grass_buffer.add2(1.0, 1.0);

                                self.grass_buffer.add3(bx - 0.5, bz, by);
                                self.grass_buffer.add2(0.0, 0.0);
                                self.grass_buffer.add3(bx - 0.5, bz + HEIGHT, by);
                                self.grass_buffer.add2(0.0, 1.0);
                                self.grass_buffer.add3(bx + 0.5, bz + HEIGHT, by);
                                self.grass_buffer.add2(1.0, 1.0);
                                by += GRASS_SPACING;
                            }

                            let by = pt.y as f32 + 0.5;
                            let mut bx = pt.x as f32;
                            while bx < pt.x as f32 + 1.01 {
                                self.grass_buffer.add3(bx, bz, by - 0.5);
                                self.grass_buffer.add2(0.0, 0.0);
                                self.grass_buffer.add3(bx, bz, by + 0.5);
                                self.grass_buffer.add2(1.0, 0.0);
                                self.grass_buffer.add3(bx, bz + HEIGHT, by + 0.5);
                                self.grass_buffer.add2(1.0, 1.0);

                                self.grass_buffer.add3(bx, bz, by - 0.5);
                                self.grass_buffer.add2(0.0, 0.0);
                                self.grass_buffer.add3(bx, bz + HEIGHT, by - 0.5);
                                self.grass_buffer.add2(0.0, 1.0);
                                self.grass_buffer.add3(bx, bz + HEIGHT, by + 0.5);
                                self.grass_buffer.add2(1.0, 1.0);
                                bx += GRASS_SPACING;
                            }
                        }
                    }
                });
            self.grass_buffer.build();
            self.models_changed = false;
        }

        // Draw the grass
        let dl = RENDER_CONTEXT.read();
        let ctx = dl.as_ref().unwrap();
        let tlock = TEXTURES.read();

        let mut encoder = ctx
            .device
            .create_command_encoder(&gpu::CommandEncoderDescriptor { label: None });

        {
            let mut rpass = encoder.begin_render_pass(&gpu::RenderPassDescriptor {
                color_attachments: &[
                    gpu::RenderPassColorAttachmentDescriptor {
                        attachment: &gbuffer.albedo.view,
                        resolve_target: None,
                        ops: gpu::Operations {
                            load: gpu::LoadOp::Load,
                            store: true,
                        },
                    },
                    gpu::RenderPassColorAttachmentDescriptor {
                        attachment: &gbuffer.normal.view,
                        resolve_target: None,
                        ops: gpu::Operations {
                            load: gpu::LoadOp::Load,
                            store: true,
                        },
                    },
                ],
                depth_stencil_attachment: Some(gpu::RenderPassDepthStencilAttachmentDescriptor {
                    attachment: tlock.get_view(0),
                    depth_ops: Some(gpu::Operations {
                        load: gpu::LoadOp::Load,
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            rpass.set_pipeline(&self.pipeline);
            rpass.set_bind_group(0, &self.bind_group, &[]);

            rpass.set_vertex_buffer(0, self.grass_buffer.slice());
            rpass.draw(0..self.grass_buffer.len(), 0..1);
        }
        ctx.queue.submit(Some(encoder.finish()));
    }
}
