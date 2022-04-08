use std::mem;

use bytemuck::{Pod, Zeroable};
use futures_lite::future;
use glam::{Mat4, Vec3};
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle, WebHandle};
use web_sys::HtmlCanvasElement;
use yew::prelude::*;

use crate::{
    mesh::{SharedMesh, Vertex},
    println,
    texture::SharedTexture,
};

pub enum Message {
    Rotate(f32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shader {
    Shaded,
    Unshaded,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Uniforms {
    view_proj: [[f32; 4]; 4],
    transform: [[f32; 4]; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct LightUniforms {
    proj: [[f32; 4]; 4],
    direction: [f32; 3],
}

#[derive(PartialEq, Properties)]
pub struct Properties {
    pub mesh: SharedMesh,
    pub texture: SharedTexture,
    #[prop_or_else(|| Shader::Shaded)]
    pub shader: Shader,
    pub rotation: Option<f32>,
    #[prop_or_default]
    pub on_rotate: Callback<f32>,
}

struct Resources {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface,
    uniforms: wgpu::Buffer,
    uniforms_group: wgpu::BindGroup,
    sampler: wgpu::Sampler,
    texture_layout: wgpu::BindGroupLayout,
    texture_group: wgpu::BindGroup,
    mesh: SharedMesh,
    mesh_width: f32,
    mesh_height: f32,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    texture: SharedTexture,
    depth_width: u32,
    depth_height: u32,
    depth_texture: wgpu::Texture,
    shaded: wgpu::RenderPipeline,
    unshaded: wgpu::RenderPipeline,
}

impl Resources {
    pub fn new(
        device: wgpu::Device,
        queue: wgpu::Queue,
        surface: wgpu::Surface,
        mesh: &SharedMesh,
        texture: &SharedTexture,
    ) -> Self {
        let uniforms_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("basil-uniforms-group-layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                count: None,
            }],
        });

        let texture_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("basil-texture-group-layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("basil-pipeline-descriptor"),
            bind_group_layouts: &[&uniforms_layout, &texture_layout],
            push_constant_ranges: &[],
        });

        let uniforms = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("basil-uniforms-buffer"),
            size: 1024,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            mapped_at_creation: false,
        });

        let shaded = Self::shader(
            &device,
            &pipeline_layout,
            wgpu::include_wgsl!("shaders/shaded.wgsl"),
        );

        let unshaded = Self::shader(
            &device,
            &pipeline_layout,
            wgpu::include_wgsl!("shaders/unshaded.wgsl"),
        );

        let uniforms_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("basil-uniforms-group"),
            layout: &uniforms_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniforms.as_entire_binding(),
            }],
        });

        let view = texture
            .create_texture(&device, &queue)
            .create_view(&Default::default());
        let sampler = device.create_sampler(&Default::default());

        let texture_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("basil-texture-group"),
            layout: &texture_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("basil-depth-texture"),
            format: wgpu::TextureFormat::Depth24Plus,
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        });

        let (vertex_buffer, index_buffer) = mesh.buffers(&device);

        Self {
            device,
            queue,
            surface,
            uniforms,
            uniforms_group,
            sampler,
            texture_layout,
            texture_group,
            texture: texture.clone(),
            mesh: mesh.clone(),
            mesh_width: mesh.width(),
            mesh_height: mesh.height(),
            vertex_buffer,
            index_buffer,
            depth_width: 1,
            depth_height: 1,
            depth_texture,
            shaded,
            unshaded,
        }
    }

    fn shader(
        device: &wgpu::Device,
        layout: &wgpu::PipelineLayout,
        shader: wgpu::ShaderModuleDescriptor,
    ) -> wgpu::RenderPipeline {
        let module = &device.create_shader_module(&shader);

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("basil-render-pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module,
                entry_point: "vert",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: mem::size_of::<Vertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x3,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x3,
                            offset: 12,
                            shader_location: 1,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 24,
                            shader_location: 2,
                        },
                    ],
                }],
            },
            primitive: Default::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth24Plus,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module,
                entry_point: "frag",
                targets: &[wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                }],
            }),
            multiview: None,
        })
    }

    pub fn get_shader(&self, shader: &Shader) -> &wgpu::RenderPipeline {
        match shader {
            Shader::Shaded => &self.shaded,
            Shader::Unshaded => &self.unshaded,
        }
    }
}

pub struct MeshViewer {
    id: u32,
    angle: f32,
    canvas: NodeRef,
    resources: Option<Resources>,
}

struct MeshViewerWindow(u32);

unsafe impl HasRawWindowHandle for MeshViewerWindow {
    fn raw_window_handle(&self) -> RawWindowHandle {
        let mut handle = WebHandle::empty();
        handle.id = self.0;

        RawWindowHandle::Web(handle)
    }
}

impl Component for MeshViewer {
    type Message = Message;
    type Properties = Properties;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            id: rand::random(),
            angle: ctx.props().rotation.unwrap_or(0.0),
            canvas: NodeRef::default(),
            resources: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::Rotate(delta) => {
                self.angle += delta * 0.0005;
                ctx.props().on_rotate.emit(self.angle);
            }
        }

        true
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        if let Some(rotation) = ctx.props().rotation {
            self.angle = rotation;
        }

        if let Some(ref mut resources) = self.resources {
            if resources.mesh != ctx.props().mesh {
                let (vertex, index) = ctx.props().mesh.buffers(&resources.device);

                resources.mesh_width = ctx.props().mesh.width();
                resources.mesh_height = ctx.props().mesh.height();

                resources.vertex_buffer = vertex;
                resources.index_buffer = index;

                resources.mesh = ctx.props().mesh.clone();
            }

            if resources.texture != ctx.props().texture {
                let view = ctx
                    .props()
                    .texture
                    .create_texture(&resources.device, &resources.queue)
                    .create_view(&Default::default());

                let texture_group =
                    resources
                        .device
                        .create_bind_group(&wgpu::BindGroupDescriptor {
                            label: Some("basil-texture-group"),
                            layout: &resources.texture_layout,
                            entries: &[
                                wgpu::BindGroupEntry {
                                    binding: 0,
                                    resource: wgpu::BindingResource::TextureView(&view),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 1,
                                    resource: wgpu::BindingResource::Sampler(&resources.sampler),
                                },
                            ],
                        });

                resources.texture_group = texture_group;

                resources.texture = ctx.props().texture.clone();
            }
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <canvas
                class="mesh-viewer-canvas"
                ref={ &self.canvas }
                data-raw-handle={ self.id.to_string() }
                onwheel={ ctx.link().callback(|event: WheelEvent| Message::Rotate(event.delta_y() as f32)) }
            />
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, _first_render: bool) {
        if self.resources.is_none() {
            let window = MeshViewerWindow(self.id);

            let instance = wgpu::Instance::new(wgpu::Backends::GL);

            let surface = unsafe { instance.create_surface(&window) };

            let adapter_fut = instance.request_adapter(&wgpu::RequestAdapterOptionsBase {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            });

            let adapter = future::block_on(adapter_fut).unwrap();

            let device_fut = adapter.request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("basil-device"),
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits {
                        max_texture_dimension_2d: 8192,
                        ..wgpu::Limits::downlevel_webgl2_defaults()
                    },
                },
                None,
            );

            let (device, queue) = future::block_on(device_fut).unwrap();

            let resources = Resources::new(
                device,
                queue,
                surface,
                &ctx.props().mesh,
                &ctx.props().texture,
            );

            self.resources = Some(resources);
        }

        let canvas = self.canvas.cast::<HtmlCanvasElement>().unwrap();
        let resources = self.resources.as_mut().unwrap();

        let width = canvas.client_width() as u32 * 2;
        let height = canvas.client_height() as u32 * 2;

        canvas.set_width(width);
        canvas.set_height(height);

        let aspect = width as f32 / height as f32;

        let (sin, cos) = self.angle.sin_cos();
        let mesh_height = resources.mesh_height / 2.0;
        let mesh_radius = (resources.mesh_width * 0.75).max(0.1);
        let position = Vec3::new(cos * mesh_radius, mesh_height, sin * mesh_radius);
        let world = Mat4::from_translation(position);
        let view = Mat4::look_at_rh(position, Vec3::new(0.0, mesh_height / 2.0, 0.0), Vec3::Y);
        let proj = Mat4::perspective_infinite_rh(std::f32::consts::PI / 2.0, aspect, 0.1);
        let view_proj = proj * view * world.inverse();

        let uniforms = Uniforms {
            view_proj: view_proj.to_cols_array_2d(),
            transform: Mat4::IDENTITY.to_cols_array_2d(),
        };

        resources
            .queue
            .write_buffer(&resources.uniforms, 0, bytemuck::bytes_of(&uniforms));

        resources.surface.configure(
            &resources.device,
            &wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                width: canvas.width(),
                height: canvas.height(),
                present_mode: wgpu::PresentMode::Fifo,
            },
        );

        let target = resources.surface.get_current_texture().unwrap();
        let view = target.texture.create_view(&Default::default());

        if resources.depth_width != width || resources.depth_height != height {
            let depth = resources.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("basil-depth-texture"),
                format: wgpu::TextureFormat::Depth24Plus,
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            });

            resources.depth_texture = depth;
            resources.depth_width = width;
            resources.depth_height = height;
        }
        let depth_view = resources.depth_texture.create_view(&Default::default());

        let mut encoder = resources.device.create_command_encoder(&Default::default());

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("basil-mesh-viewer-render-pass"),
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: true,
                },
            }],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        });

        let shader = resources.get_shader(&ctx.props().shader);

        render_pass.set_pipeline(shader);
        render_pass.set_bind_group(0, &resources.uniforms_group, &[]);
        render_pass.set_bind_group(1, &resources.texture_group, &[]);

        render_pass.set_vertex_buffer(0, resources.vertex_buffer.slice(..));
        render_pass.set_index_buffer(resources.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

        render_pass.draw_indexed(0..resources.mesh.indices.len() as u32, 0, 0..1);

        drop(render_pass);

        resources.queue.submit(std::iter::once(encoder.finish()));

        target.present();
    }
}
