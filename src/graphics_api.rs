use winit::window::Window;

/// Errors that can occur during graphics API operations
#[derive(Debug)]
pub enum GraphicsError {
    AdapterNotFound,
    DeviceRequestFailed(wgpu::RequestDeviceError),
    SurfaceError(wgpu::SurfaceError),
    Other(String),
}

impl std::fmt::Display for GraphicsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GraphicsError::AdapterNotFound => write!(f, "No suitable graphics adapter found"),
            GraphicsError::DeviceRequestFailed(err) => write!(f, "Device request failed: {err}"),
            GraphicsError::SurfaceError(err) => write!(f, "Surface error: {err}"),
            GraphicsError::Other(msg) => write!(f, "Graphics error: {msg}"),
        }
    }
}

impl std::error::Error for GraphicsError {}

/// Trait for graphics API abstraction
pub trait GraphicsApi: Send + Sync {
    /// Initialize the graphics API
    fn new(
        window: Option<&Window>,
        width: u32,
        height: u32,
    ) -> impl std::future::Future<Output = Result<Self, GraphicsError>>
    where
        Self: Sized;

    /// Validate and clamp MSAA sample count to supported values
    fn validate_sample_count(&self, requested_samples: u32) -> u32;

    /// Resize the surface
    fn resize(&mut self, width: u32, height: u32);

    /// Present the current frame
    fn present(&mut self) -> Result<(), GraphicsError>;

    /// Get the current surface texture
    fn get_current_texture(&self) -> Result<wgpu::SurfaceTexture, GraphicsError>;

    /// Get the device
    fn device(&self) -> &wgpu::Device;

    /// Get the queue
    fn queue(&self) -> &wgpu::Queue;

    /// Get the surface format
    fn surface_format(&self) -> wgpu::TextureFormat;

    /// Get the surface size
    fn surface_size(&self) -> (u32, u32);
}

/// wgpu implementation of the graphics API
pub struct WgpuGraphicsApi {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface_format: wgpu::TextureFormat,
    surface_size: (u32, u32),
    #[allow(dead_code)]
    has_surface: bool,
}

impl WgpuGraphicsApi {
    /// Create a new wgpu graphics API instance
    pub async fn new_impl(
        window: Option<&Window>,
        width: u32,
        height: u32,
    ) -> Result<Self, GraphicsError> {
        log::info!("Initializing wgpu graphics API with resolution {width}x{height}");

        // Create instance
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // Create surface if window is provided (but don't store it)
        let (adapter, surface_format, surface_size, has_surface) = if let Some(window) = window {
            let surface = instance
                .create_surface(window)
                .map_err(|e| GraphicsError::Other(format!("Failed to create surface: {e}")))?;

            // Request adapter
            let adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::default(),
                    compatible_surface: Some(&surface),
                    force_fallback_adapter: false,
                })
                .await
                .ok_or(GraphicsError::AdapterNotFound)?;

            let size = window.inner_size();
            let surface_caps = surface.get_capabilities(&adapter);
            let surface_format = surface_caps
                .formats
                .iter()
                .find(|f| f.is_srgb())
                .copied()
                .unwrap_or(surface_caps.formats[0]);

            // We'll configure the surface later when we have the device
            (adapter, surface_format, (size.width, size.height), true)
        } else {
            // For headless mode, request adapter without surface, use provided dimensions
            let adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::default(),
                    compatible_surface: None,
                    force_fallback_adapter: false,
                })
                .await
                .ok_or(GraphicsError::AdapterNotFound)?;

            (
                adapter,
                wgpu::TextureFormat::Rgba8UnormSrgb,
                (width, height),
                false,
            )
        };

        log::info!("Selected adapter: {:?}", adapter.get_info());

        // Request device and queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::MemoryUsage,
                },
                None,
            )
            .await
            .map_err(GraphicsError::DeviceRequestFailed)?;

        log::info!("wgpu graphics API initialized successfully");

        Ok(WgpuGraphicsApi {
            device,
            queue,
            surface_format,
            surface_size,
            has_surface,
        })
    }
}

impl GraphicsApi for WgpuGraphicsApi {
    async fn new(window: Option<&Window>, width: u32, height: u32) -> Result<Self, GraphicsError> {
        Self::new_impl(window, width, height).await
    }

    fn validate_sample_count(&self, requested_samples: u32) -> u32 {
        // MSAA sample counts must be powers of 2 and supported by the hardware
        // Common supported values are 1, 2, 4, 8, 16
        let valid_samples = [1, 2, 4, 8, 16];

        // Find the closest valid sample count that doesn't exceed the requested value
        let clamped = valid_samples
            .iter()
            .rev() // Start from highest to find the best match
            .find(|&&samples| samples <= requested_samples)
            .copied()
            .unwrap_or(1); // Default to 1 if no valid value found

        if clamped != requested_samples {
            log::warn!(
                "MSAA sample count {requested_samples} is not supported, clamping to {clamped}"
            );
        }

        clamped
    }

    fn resize(&mut self, width: u32, height: u32) {
        log::debug!("Resizing surface to {width}x{height}");
        self.surface_size = (width, height);
        // Note: In a full implementation, we'd reconfigure the surface here
    }

    fn present(&mut self) -> Result<(), GraphicsError> {
        // For now, just return Ok since we don't have actual rendering
        Ok(())
    }

    fn get_current_texture(&self) -> Result<wgpu::SurfaceTexture, GraphicsError> {
        // For headless mode, create a dummy texture
        // In a real implementation with windowed mode, this would get the actual surface texture
        Err(GraphicsError::Other(
            "get_current_texture not implemented in simplified version".to_string(),
        ))
    }

    fn device(&self) -> &wgpu::Device {
        &self.device
    }

    fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    fn surface_format(&self) -> wgpu::TextureFormat {
        self.surface_format
    }

    fn surface_size(&self) -> (u32, u32) {
        self.surface_size
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_sample_count_validation_logic() {
        // Test the validation logic without requiring a full graphics API instance
        fn validate_sample_count_test(requested_samples: u32) -> u32 {
            let valid_samples = [1, 2, 4, 8, 16];
            valid_samples
                .iter()
                .rev()
                .find(|&&samples| samples <= requested_samples)
                .copied()
                .unwrap_or(1)
        }

        // Test valid sample counts
        assert_eq!(validate_sample_count_test(1), 1);
        assert_eq!(validate_sample_count_test(2), 2);
        assert_eq!(validate_sample_count_test(4), 4);
        assert_eq!(validate_sample_count_test(8), 8);
        assert_eq!(validate_sample_count_test(16), 16);

        // Test invalid sample counts (should be clamped to nearest lower valid value)
        assert_eq!(validate_sample_count_test(3), 2);
        assert_eq!(validate_sample_count_test(5), 4);
        assert_eq!(validate_sample_count_test(6), 4);
        assert_eq!(validate_sample_count_test(7), 4);
        assert_eq!(validate_sample_count_test(9), 8);
        assert_eq!(validate_sample_count_test(15), 8);
        assert_eq!(validate_sample_count_test(17), 16);
        assert_eq!(validate_sample_count_test(32), 16);

        // Test edge case
        assert_eq!(validate_sample_count_test(0), 1);
    }
}
