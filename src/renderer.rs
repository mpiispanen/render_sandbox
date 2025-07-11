use crate::graphics_api::{GraphicsApi, GraphicsError};
use crate::render_graph::{
    ForwardRenderPass, PlaceholderPass, RenderGraph, RenderGraphError, ResourceUsage,
};
use crate::resource_manager::ResourceManager;
use crate::scene::{NodeContent, Scene};

/// Errors that can occur during rendering operations
#[derive(Debug)]
pub enum RendererError {
    GraphicsError(GraphicsError),
    RenderGraphError(RenderGraphError),
    ResourceError(String),
    Other(String),
}

impl std::fmt::Display for RendererError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RendererError::GraphicsError(err) => write!(f, "Graphics error: {err}"),
            RendererError::RenderGraphError(err) => write!(f, "Render graph error: {err}"),
            RendererError::ResourceError(msg) => write!(f, "Resource error: {msg}"),
            RendererError::Other(msg) => write!(f, "Renderer error: {msg}"),
        }
    }
}

impl std::error::Error for RendererError {}

impl From<GraphicsError> for RendererError {
    fn from(err: GraphicsError) -> Self {
        RendererError::GraphicsError(err)
    }
}

impl From<RenderGraphError> for RendererError {
    fn from(err: RenderGraphError) -> Self {
        RendererError::RenderGraphError(err)
    }
}

/// Configuration for the renderer
#[derive(Debug, Clone)]
pub struct RendererConfig {
    pub enable_depth_testing: bool,
    pub enable_culling: bool,
    pub clear_color: [f32; 4],
    pub msaa_samples: u32,
}

impl Default for RendererConfig {
    fn default() -> Self {
        Self {
            enable_depth_testing: true,
            enable_culling: true,
            clear_color: [0.0, 0.0, 0.0, 1.0],
            msaa_samples: 1,
        }
    }
}

/// Statistics about the rendering process
#[derive(Debug, Default, Clone)]
pub struct RenderStats {
    pub frame_count: u64,
    pub draw_calls: u32,
    pub vertices_rendered: u32,
    pub triangles_rendered: u32,
    pub render_passes: u32,
}

/// Enhanced renderer that coordinates all rendering components
pub struct Renderer {
    graphics_api: Box<dyn GraphicsApi>,
    resource_manager: ResourceManager,
    render_graph: RenderGraph,
    config: RendererConfig,
    stats: RenderStats,
    initialized: bool,
}

impl Renderer {
    /// Create a new renderer
    pub fn new(graphics_api: Box<dyn GraphicsApi>) -> Self {
        log::info!("Creating renderer");

        Self {
            graphics_api,
            resource_manager: ResourceManager::new(),
            render_graph: RenderGraph::new(),
            config: RendererConfig::default(),
            stats: RenderStats::default(),
            initialized: false,
        }
    }

    /// Initialize the renderer with default resources and render passes
    pub fn initialize(&mut self) -> Result<(), RendererError> {
        if self.initialized {
            log::warn!("Renderer already initialized");
            return Ok(());
        }

        log::info!("Initializing renderer");

        // Create basic render passes for a simple forward rendering pipeline
        self.setup_default_render_graph()?;

        // Create default resources
        self.create_default_resources()?;

        self.initialized = true;
        log::info!("Renderer initialized successfully");
        Ok(())
    }

    /// Setup the default render graph with basic passes
    fn setup_default_render_graph(&mut self) -> Result<(), RendererError> {
        log::debug!("Setting up default render graph");

        // Clear pass
        let clear_pass =
            PlaceholderPass::new("ClearPass").with_resource("BackBuffer", ResourceUsage::Write);
        self.render_graph.add_pass(Box::new(clear_pass));

        // Forward render pass - use actual ForwardRenderPass instead of placeholder
        let (width, height) = self.graphics_api.surface_size();
        let forward_pass = ForwardRenderPass::new("ForwardPass")
            .with_resource("BackBuffer", ResourceUsage::ReadWrite)
            .with_resource("DepthBuffer", ResourceUsage::ReadWrite)
            .with_clear_color([0.1, 0.2, 0.3, 1.0]) // Dark blue background
            .with_resolution(width, height);
        self.render_graph.add_pass(Box::new(forward_pass));

        // Compile the render graph
        self.render_graph.compile()?;

        log::debug!("Default render graph setup complete");
        Ok(())
    }

    /// Create default resources needed for rendering
    fn create_default_resources(&mut self) -> Result<(), RendererError> {
        log::debug!("Creating default resources");

        let device = self.graphics_api.device();
        let (width, height) = self.graphics_api.surface_size();

        // Create depth buffer
        let _depth_texture = self.resource_manager.create_texture(
            device,
            &wgpu::TextureDescriptor {
                label: Some("Depth Texture"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: self.config.msaa_samples,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            },
        );

        log::debug!("Default resources created");
        Ok(())
    }

    /// Update the renderer configuration
    pub fn update_config(&mut self, config: RendererConfig) {
        log::debug!("Updating renderer configuration");
        self.config = config;
    }

    /// Resize the renderer's resources
    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), RendererError> {
        log::info!("Resizing renderer to {width}x{height}");

        self.graphics_api.resize(width, height);

        // Recreate size-dependent resources
        if self.initialized {
            self.create_default_resources()?;
        }

        Ok(())
    }

    /// Perform visibility culling on the scene
    fn cull_scene<'a>(&self, scene: &'a Scene) -> Vec<&'a crate::scene::SceneNode> {
        log::trace!("Performing scene culling");

        // For now, just return all visible mesh nodes
        // In a real implementation, this would perform frustum culling,
        // occlusion culling, distance culling, etc.
        let visible_nodes = scene.get_mesh_nodes();

        log::trace!("Culled scene: {} visible nodes", visible_nodes.len());
        visible_nodes
    }

    /// Build the render graph for the current frame
    fn build_frame_render_graph(&mut self, _scene: &Scene) -> Result<(), RendererError> {
        log::trace!("Building frame render graph");

        // For now, use the default render graph
        // In a real implementation, this would dynamically build the graph
        // based on scene content, lighting, post-processing effects, etc.

        if !self.render_graph.is_compiled() {
            self.render_graph.compile()?;
        }

        Ok(())
    }

    /// Render a frame
    pub fn render(&mut self, scene: &Scene) -> Result<(), RendererError> {
        if !self.initialized {
            return Err(RendererError::Other("Renderer not initialized".to_string()));
        }

        log::trace!("Rendering frame {}", self.stats.frame_count);

        // Step 1: Visibility & Culling
        let _visible_nodes = self.cull_scene(scene);

        // Step 2: Build render graph for this frame
        self.build_frame_render_graph(scene)?;

        // Step 3: Execute render graph
        self.render_graph.execute(
            self.graphics_api.device(),
            self.graphics_api.queue(),
            &self.resource_manager,
        )?;

        // Step 4: Present
        self.graphics_api.present()?;

        // Update statistics
        self.stats.frame_count += 1;
        self.stats.render_passes = self.render_graph.pass_count() as u32;

        log::trace!("Frame {} rendered successfully", self.stats.frame_count);
        Ok(())
    }

    /// Get the current render statistics
    pub fn get_stats(&self) -> &RenderStats {
        &self.stats
    }

    /// Reset render statistics
    pub fn reset_stats(&mut self) {
        self.stats = RenderStats::default();
        log::debug!("Render statistics reset");
    }

    /// Get the resource manager
    pub fn resource_manager(&self) -> &ResourceManager {
        &self.resource_manager
    }

    /// Get a mutable reference to the resource manager
    pub fn resource_manager_mut(&mut self) -> &mut ResourceManager {
        &mut self.resource_manager
    }

    /// Execute a function with access to both device and resource manager
    /// This allows safe access to both without borrow checker issues
    pub fn with_device_and_resource_manager<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&wgpu::Device, &mut ResourceManager) -> R,
    {
        let device = self.graphics_api.device();
        f(device, &mut self.resource_manager)
    }

    /// Get the render graph
    pub fn render_graph(&self) -> &RenderGraph {
        &self.render_graph
    }

    /// Get a mutable reference to the render graph
    pub fn render_graph_mut(&mut self) -> &mut RenderGraph {
        &mut self.render_graph
    }

    /// Get the graphics API
    pub fn graphics_api(&self) -> &dyn GraphicsApi {
        self.graphics_api.as_ref()
    }

    /// Check if the renderer is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Get the current configuration
    pub fn config(&self) -> &RendererConfig {
        &self.config
    }

    /// Create a simple mesh from vertex data
    pub fn create_simple_mesh(
        &mut self,
        vertices: &[f32],
        indices: Option<&[u16]>,
    ) -> Result<crate::scene::Mesh, RendererError> {
        let device = self.graphics_api.device();

        // Create vertex buffer
        let vertex_buffer = self.resource_manager.create_buffer_init(
            device,
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(vertices),
                usage: wgpu::BufferUsages::VERTEX,
            },
        );

        // Create index buffer if indices are provided
        let (index_buffer, index_count) = if let Some(indices) = indices {
            let buffer = self.resource_manager.create_buffer_init(
                device,
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: bytemuck::cast_slice(indices),
                    usage: wgpu::BufferUsages::INDEX,
                },
            );
            (Some(buffer), Some(indices.len() as u32))
        } else {
            (None, None)
        };

        Ok(crate::scene::Mesh {
            vertex_buffer,
            index_buffer,
            vertex_count: (vertices.len() / 3) as u32, // Assuming 3 components per vertex
            index_count,
        })
    }

    /// Add a simple triangle to the scene for testing
    pub fn create_test_triangle(&mut self, scene: &mut Scene) -> Result<(), RendererError> {
        log::info!("Creating test triangle");

        // Simple triangle vertices (position only)
        #[rustfmt::skip]
        let vertices = [
            0.0, 0.5, 0.0,   // Top
            -0.5, -0.5, 0.0, // Bottom left
            0.5, -0.5, 0.0,  // Bottom right
        ];

        let mesh = self.create_simple_mesh(&vertices, None)?;

        // Create a mesh node
        let mesh_node = crate::scene::SceneNode::with_name("TestTriangle")
            .with_content(NodeContent::Mesh(mesh));

        scene.add_node(mesh_node);

        log::info!("Test triangle created successfully");
        Ok(())
    }

    /// Load a GLTF file and add its contents to the scene
    pub fn load_gltf_to_scene<P: AsRef<std::path::Path>>(
        &mut self,
        path: P,
        scene: &mut Scene,
    ) -> Result<(), RendererError> {
        log::info!("Loading GLTF file to scene: {}", path.as_ref().display());

        self.with_device_and_resource_manager(|device, resource_manager| {
            crate::gltf_loader::GltfLoader::load_gltf(device, resource_manager, path, scene)
        })
        .map_err(|e| RendererError::Other(format!("GLTF loading failed: {e}")))?;

        log::info!("GLTF file loaded successfully");
        Ok(())
    }

    /// Create a test triangle from GLTF-style data and add it to the scene
    pub fn create_gltf_test_triangle(&mut self, scene: &mut Scene) -> Result<(), RendererError> {
        log::info!("Creating GLTF-style test triangle");

        let mesh = self
            .with_device_and_resource_manager(|device, resource_manager| {
                crate::gltf_loader::GltfLoader::create_test_triangle(device, resource_manager)
            })
            .map_err(|e| {
                RendererError::Other(format!("GLTF test triangle creation failed: {e}"))
            })?;

        // Create a mesh node
        let mesh_node = crate::scene::SceneNode::with_name("GltfTestTriangle")
            .with_content(NodeContent::Mesh(mesh));

        scene.add_node(mesh_node);

        log::info!("GLTF-style test triangle created successfully");
        Ok(())
    }
}
