use crate::render_graph::{
    PassId, RenderGraphError, RenderPass, ResourceDeclaration, ResourceId, ResourceUsage,
};
use crate::resource_manager::ResourceManager;

/// A simple forward renderer pass that renders meshes
pub struct ForwardRenderPass {
    id: PassId,
    resources: Vec<ResourceDeclaration>,
    clear_color: [f64; 4],
    resolution: (u32, u32),
    surface_format: wgpu::TextureFormat,
    render_pipeline: Option<wgpu::RenderPipeline>,
    initialized: bool,
}

impl ForwardRenderPass {
    pub fn new(name: &str) -> Self {
        Self {
            id: PassId::new(name),
            resources: vec![],
            clear_color: [0.0, 0.0, 0.0, 1.0],
            resolution: (800, 600), // Default resolution
            surface_format: wgpu::TextureFormat::Bgra8UnormSrgb, // Default format, should be overridden
            render_pipeline: None,
            initialized: false,
        }
    }

    pub fn with_resource(mut self, resource_id: &str, usage: ResourceUsage) -> Self {
        self.resources.push(ResourceDeclaration {
            id: ResourceId::new(resource_id),
            usage,
        });
        self
    }

    pub fn with_clear_color(mut self, color: [f64; 4]) -> Self {
        self.clear_color = color;
        self
    }

    pub fn with_resolution(mut self, width: u32, height: u32) -> Self {
        self.resolution = (width, height);
        self
    }

    pub fn with_surface_format(mut self, format: wgpu::TextureFormat) -> Self {
        self.surface_format = format;
        self
    }
}

impl RenderPass for ForwardRenderPass {
    fn id(&self) -> PassId {
        self.id.clone()
    }

    fn resources(&self) -> Vec<ResourceDeclaration> {
        self.resources.clone()
    }

    fn initialize(
        &mut self,
        device: &wgpu::Device,
        _resource_manager: &ResourceManager,
    ) -> Result<(), RenderGraphError> {
        if self.initialized {
            return Ok(());
        }

        log::debug!("Initializing forward render pass: {}", self.id);

        // Create a basic shader for forward rendering
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Forward Render Shader"),
            source: wgpu::ShaderSource::Wgsl(
                r#"
                @vertex
                fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4<f32> {
                    // Simple fullscreen triangle
                    var pos = array<vec2<f32>, 3>(
                        vec2<f32>(-1.0, -1.0),
                        vec2<f32>(-1.0,  3.0),
                        vec2<f32>( 3.0, -1.0),
                    );
                    return vec4<f32>(pos[vertex_index], 0.0, 1.0);
                }

                @fragment
                fn fs_main() -> @location(0) vec4<f32> {
                    return vec4<f32>(0.1, 0.2, 0.3, 1.0);
                }
                "#.into(),
            ),
        });

        // Create render pipeline
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Forward Render Pipeline"),
            layout: Some(
                &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Forward Pipeline Layout"),
                    bind_group_layouts: &[],
                    push_constant_ranges: &[],
                }),
            ),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: self.surface_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        self.render_pipeline = Some(render_pipeline);
        self.initialized = true;

        log::debug!("Forward render pass initialized successfully");
        Ok(())
    }

    fn execute(
        &self,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        resource_manager: &ResourceManager,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Result<(), RenderGraphError> {
        log::debug!("Executing forward render pass: {}", self.id);

        // Get the actual render targets from the resource manager
        let back_buffer_handle: crate::resource_manager::Handle<wgpu::Texture> = resource_manager
            .get_named_resource("BackBuffer")
            .ok_or_else(|| {
                RenderGraphError::ResourceNotFound(crate::render_graph::ResourceId::new(
                    "BackBuffer",
                ))
            })?;

        let back_buffer = resource_manager
            .get_texture(back_buffer_handle)
            .map_err(|e| {
                RenderGraphError::ExecutionFailed(format!("Failed to get BackBuffer: {e}"))
            })?;

        let depth_buffer_handle: crate::resource_manager::Handle<wgpu::Texture> = resource_manager
            .get_named_resource("DepthBuffer")
            .ok_or_else(|| {
                RenderGraphError::ResourceNotFound(crate::render_graph::ResourceId::new(
                    "DepthBuffer",
                ))
            })?;

        let depth_buffer = resource_manager
            .get_texture(depth_buffer_handle)
            .map_err(|e| {
                RenderGraphError::ExecutionFailed(format!("Failed to get DepthBuffer: {e}"))
            })?;

        // Create texture views for rendering
        let color_view = back_buffer.create_view(&wgpu::TextureViewDescriptor::default());
        let depth_view = depth_buffer.create_view(&wgpu::TextureViewDescriptor::default());

        // Create the render pass with proper render targets
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Forward Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &color_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: self.clear_color[0],
                        g: self.clear_color[1],
                        b: self.clear_color[2],
                        a: self.clear_color[3],
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        // If we have a render pipeline, use it to render a simple triangle
        if let Some(ref pipeline) = self.render_pipeline {
            render_pass.set_pipeline(pipeline);
            render_pass.draw(0..3, 0..1); // Draw a single triangle
        }

        log::debug!("Forward render pass executed with proper render targets");

        Ok(())
    }
}
