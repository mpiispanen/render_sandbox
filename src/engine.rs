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
            EngineError::InitializationError(msg) => write!(f, "Initialization error: {}", msg),
            EngineError::RenderingError(msg) => write!(f, "Rendering error: {}", msg),
            EngineError::Other(msg) => write!(f, "Engine error: {}", msg),
        }
    }
}

impl std::error::Error for EngineError {}

/// Engine trait for rendering abstraction
pub trait Engine: Send + 'static {
    /// Create a new engine instance
    fn new(window_handle: Option<&Window>) -> Result<Self, EngineError>
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
    fn new(window_handle: Option<&Window>) -> Result<Self, EngineError> {
        log::info!("Creating placeholder engine (headless: {})", window_handle.is_none());
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
        log::debug!("Handling input event: {:?}", event);
    }

    fn get_rendered_frame_data(&self) -> Option<Vec<u8>> {
        if self.is_headless {
            // Return placeholder RGBA data (tiny 2x2 image)
            Some(vec![255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255])
        } else {
            None
        }
    }
}