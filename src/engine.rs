use crate::graphics_api::{GraphicsApi, WgpuGraphicsApi};
use crate::renderer::Renderer;
use crate::scene::Scene;
use winit::{dpi::PhysicalSize, event::WindowEvent, window::Window};

/// Error types for engine operations
#[derive(Debug)]
pub enum EngineError {
    InitializationError(String),
    RenderingError(String),
    #[allow(dead_code)]
    Other(String),
}

impl std::fmt::Display for EngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EngineError::InitializationError(msg) => write!(f, "Initialization error: {msg}"),
            EngineError::RenderingError(msg) => write!(f, "Rendering error: {msg}"),
            EngineError::Other(msg) => write!(f, "Engine error: {msg}"),
        }
    }
}

impl std::error::Error for EngineError {}

/// Engine trait for rendering abstraction
pub trait Engine: Send + 'static {
    /// Create a new engine instance
    fn new(
        window_handle: Option<&Window>,
    ) -> impl std::future::Future<Output = Result<Self, EngineError>>
    where
        Self: Sized;

    /// Handle window resize
    fn resize(&mut self, new_size: PhysicalSize<u32>);

    /// Update game logic/scene state
    fn update(&mut self);

    /// Render a frame
    fn render(&mut self) -> Result<(), EngineError>;

    /// Handle window input events
    fn handle_input(&mut self, event: &WindowEvent);

    /// Get rendered frame data for headless mode
    fn get_rendered_frame_data(&self) -> Option<Vec<u8>>;
}

/// Simple placeholder engine implementation for testing
pub struct PlaceholderEngine {
    frame_count: u32,
    is_headless: bool,
}

impl Engine for PlaceholderEngine {
    async fn new(window_handle: Option<&Window>) -> Result<Self, EngineError> {
        log::info!(
            "Creating placeholder engine (headless: {})",
            window_handle.is_none()
        );
        Ok(PlaceholderEngine {
            frame_count: 0,
            is_headless: window_handle.is_none(),
        })
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if !self.is_headless {
            log::debug!("Engine resize to {}x{}", new_size.width, new_size.height);
        }
    }

    fn update(&mut self) {
        // Simple update logic
        self.frame_count += 1;
    }

    fn render(&mut self) -> Result<(), EngineError> {
        log::debug!("Rendering frame {}", self.frame_count);
        // Placeholder rendering logic
        Ok(())
    }

    fn handle_input(&mut self, event: &WindowEvent) {
        log::debug!("Handling input event: {event:?}");
    }

    fn get_rendered_frame_data(&self) -> Option<Vec<u8>> {
        if self.is_headless {
            // Return placeholder RGBA data (tiny 2x2 image)
            Some(vec![
                255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255,
            ])
        } else {
            None
        }
    }
}

/// Real-time rendering engine implementation
pub struct RealTimeEngine {
    renderer: Renderer,
    scene: Scene,
    is_headless: bool,
    frame_count: u32,
}

impl RealTimeEngine {
    async fn new_impl(window_handle: Option<&Window>) -> Result<Self, EngineError> {
        log::info!(
            "Creating real-time engine (headless: {})",
            window_handle.is_none()
        );

        // Initialize graphics API
        let graphics_api = WgpuGraphicsApi::new(window_handle).await.map_err(|e| {
            EngineError::InitializationError(format!("Failed to create graphics API: {e}"))
        })?;

        // Create renderer
        let mut renderer = Renderer::new(Box::new(graphics_api));

        // Initialize renderer
        renderer.initialize().map_err(|e| {
            EngineError::InitializationError(format!("Failed to initialize renderer: {e}"))
        })?;

        // Create scene
        let mut scene = Scene::new();

        // Try to load a GLTF file if available, otherwise create a GLTF-style test triangle
        let gltf_path = "test_assets/triangle.gltf";
        let triangle_created = if std::path::Path::new(gltf_path).exists() {
            log::info!("Loading triangle from GLTF file: {}", gltf_path);
            match renderer.load_gltf_to_scene(gltf_path, &mut scene) {
                Ok(()) => {
                    log::info!("Successfully loaded GLTF triangle");
                    true
                }
                Err(e) => {
                    log::warn!("Failed to load GLTF file ({}), falling back to test triangle", e);
                    false
                }
            }
        } else {
            log::info!("GLTF file not found, using GLTF-style test triangle");
            false
        };

        // If GLTF loading failed or file not found, create a GLTF-style test triangle
        if !triangle_created {
            renderer.create_gltf_test_triangle(&mut scene).map_err(|e| {
                EngineError::InitializationError(format!("Failed to create GLTF test triangle: {e}"))
            })?;
        }

        Ok(RealTimeEngine {
            renderer,
            scene,
            is_headless: window_handle.is_none(),
            frame_count: 0,
        })
    }
}

impl Engine for RealTimeEngine {
    async fn new(window_handle: Option<&Window>) -> Result<Self, EngineError> {
        Self::new_impl(window_handle).await
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        log::debug!(
            "Real-time engine resize to {}x{}",
            new_size.width,
            new_size.height
        );
        if let Err(e) = self.renderer.resize(new_size.width, new_size.height) {
            log::error!("Failed to resize renderer: {e}");
        }
    }

    fn update(&mut self) {
        // Update scene
        self.scene.update(0.016); // Assuming 60 FPS
        self.frame_count += 1;
    }

    fn render(&mut self) -> Result<(), EngineError> {
        self.renderer
            .render(&self.scene)
            .map_err(|e| EngineError::RenderingError(format!("Render failed: {e}")))?;

        if self.frame_count % 60 == 0 {
            let stats = self.renderer.get_stats();
            log::debug!(
                "Rendered {} frames, {} passes",
                stats.frame_count,
                stats.render_passes
            );
        }

        Ok(())
    }

    fn handle_input(&mut self, event: &WindowEvent) {
        log::debug!("Real-time engine handling input: {event:?}");
        // Handle input events for camera movement, object interaction, etc.
    }

    fn get_rendered_frame_data(&self) -> Option<Vec<u8>> {
        if self.is_headless {
            // In a real implementation, this would capture the rendered frame
            // For now, return placeholder data
            Some(vec![128; 800 * 600 * 4]) // Gray image
        } else {
            None
        }
    }
}
