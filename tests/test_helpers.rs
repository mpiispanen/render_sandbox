use render_sandbox::{
    graphics_api::{GraphicsApi, WgpuGraphicsApi},
    resource_manager::ResourceManager,
};

/// Test helper for creating a minimal GPU context for testing
pub struct TestGpuContext {
    pub graphics_api: WgpuGraphicsApi,
    #[allow(dead_code)]
    pub resource_manager: ResourceManager,
}

impl TestGpuContext {
    /// Create a new test GPU context in headless mode
    #[allow(dead_code)]
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Create headless graphics API
        let graphics_api = WgpuGraphicsApi::new(None).await?;

        // Create resource manager
        let resource_manager = ResourceManager::new();

        Ok(TestGpuContext {
            graphics_api,
            resource_manager,
        })
    }

    /// Get reference to the wgpu device
    #[allow(dead_code)]
    pub fn device(&self) -> &wgpu::Device {
        self.graphics_api.device()
    }

    /// Split into device and resource manager to avoid borrow checker issues
    #[allow(dead_code)]
    pub fn split(&mut self) -> (&wgpu::Device, &mut ResourceManager) {
        (self.graphics_api.device(), &mut self.resource_manager)
    }
}

/// Test helper macro to skip GPU-dependent tests in CI environments
#[macro_export]
macro_rules! skip_if_no_gpu {
    ($test_name:expr) => {
        // Try to create a test GPU context
        match futures::executor::block_on(TestGpuContext::new()) {
            Ok(_) => {
                // GPU available, test can run
            }
            Err(e) => {
                eprintln!("Skipping {} - GPU not available: {}", $test_name, e);
                return;
            }
        }
    };
}
