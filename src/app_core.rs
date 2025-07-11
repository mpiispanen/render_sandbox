use crate::engine::{Engine, EngineError, RealTimeEngine};
use log::{debug, error, info, warn};
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

/// Main application struct that manages windowed and headless modes
pub struct Application {
    window: Option<Window>,
    event_loop: Option<EventLoop<()>>,
    engine: Option<Box<dyn Engine>>,
    should_exit: bool,
    is_headless: bool,
    gltf_path: String,
}

impl Application {
    /// Creates a new Application instance
    /// `headless`: If true, no window is created
    /// `gltf_path`: Path to the GLTF file to load
    pub fn new(headless: bool, gltf_path: String) -> Result<Self, EngineError> {
        info!("Creating application (headless: {headless})");

        if headless {
            // Headless mode - no window or event loop
            Ok(Application {
                window: None,
                event_loop: None,
                engine: None,
                should_exit: false,
                is_headless: true,
                gltf_path,
            })
        } else {
            // Windowed mode - create window and event loop
            let event_loop = EventLoop::new().map_err(|e| {
                EngineError::InitializationError(format!("Failed to create event loop: {e}"))
            })?;

            let window = WindowBuilder::new()
                .with_title("Render Sandbox")
                .with_inner_size(PhysicalSize::new(800, 600))
                .build(&event_loop)
                .map_err(|e| {
                    EngineError::InitializationError(format!("Failed to create window: {e}"))
                })?;

            Ok(Application {
                window: Some(window),
                event_loop: Some(event_loop),
                engine: None,
                should_exit: false,
                is_headless: false,
                gltf_path,
            })
        }
    }

    /// Initialize the engine asynchronously
    async fn initialize_engine(&mut self) -> Result<(), EngineError> {
        info!("Initializing engine");

        let engine = if self.is_headless {
            Box::new(RealTimeEngine::new(None, &self.gltf_path).await?) as Box<dyn Engine>
        } else {
            Box::new(RealTimeEngine::new(self.window.as_ref(), &self.gltf_path).await?)
                as Box<dyn Engine>
        };

        self.engine = Some(engine);
        info!("Engine initialized successfully");
        Ok(())
    }

    /// Runs the application loop
    /// Delegates to `run_windowed` or `run_headless` based on `is_headless`
    pub fn run(mut self) -> Result<(), EngineError> {
        // Initialize engine in a blocking way
        futures::executor::block_on(self.initialize_engine())?;

        if self.is_headless {
            info!("Starting headless mode");
            let frame_data = self.run_headless(Some(10))?; // Render 10 frames by default
            if let Some(data) = frame_data {
                info!("Rendered frame data: {} bytes", data.len());
            }
            Ok(())
        } else {
            info!("Starting windowed mode");
            if let Some(event_loop) = self.event_loop.take() {
                self.run_windowed(event_loop);
                Ok(())
            } else {
                Err(EngineError::InitializationError(
                    "Event loop not available for windowed mode".to_string(),
                ))
            }
        }
    }

    /// Runs the main event loop for windowed mode
    fn run_windowed(mut self, event_loop: EventLoop<()>) {
        debug!("Starting windowed event loop");

        let _ = event_loop.run(move |event, elwt| {
            elwt.set_control_flow(ControlFlow::Poll);

            match event {
                Event::WindowEvent { event, window_id } => {
                    if let Some(window) = &self.window {
                        if window.id() == window_id {
                            match event {
                                WindowEvent::CloseRequested => {
                                    info!("Close requested");
                                    self.should_exit = true;
                                    elwt.exit();
                                }
                                WindowEvent::Resized(physical_size) => {
                                    debug!("Window resized to: {physical_size:?}");
                                    if let Some(engine) = &mut self.engine {
                                        engine.resize(physical_size);
                                    }
                                }
                                WindowEvent::KeyboardInput { event, .. } => {
                                    // Handle escape key to exit
                                    if event.physical_key
                                        == winit::keyboard::PhysicalKey::Code(
                                            winit::keyboard::KeyCode::Escape,
                                        )
                                    {
                                        info!("Escape key pressed, exiting");
                                        self.should_exit = true;
                                        elwt.exit();
                                    } else if let Some(engine) = &mut self.engine {
                                        // Pass the original event directly to the engine
                                        engine.handle_input(&WindowEvent::KeyboardInput {
                                            event,
                                            device_id: unsafe { winit::event::DeviceId::dummy() },
                                            is_synthetic: false,
                                        });
                                    }
                                }
                                WindowEvent::RedrawRequested => {
                                    debug!("Redraw requested");
                                    if let Some(engine) = &mut self.engine {
                                        if let Err(e) = engine.render() {
                                            error!("Render error: {e}");
                                        }
                                    }
                                }
                                _ => {
                                    if let Some(engine) = &mut self.engine {
                                        engine.handle_input(&event);
                                    }
                                }
                            }
                        }
                    }
                }
                Event::AboutToWait => {
                    // Update game logic
                    if let Some(engine) = &mut self.engine {
                        engine.update();
                    }

                    // Request redraw
                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }

                    if self.should_exit {
                        elwt.exit();
                    }
                }
                Event::LoopExiting => {
                    info!("Event loop exiting");
                }
                _ => {}
            }
        });
    }

    /// Runs the application in headless mode for a fixed number of frames
    /// Returns the last rendered frame's data if available
    fn run_headless(mut self, max_frames: Option<u32>) -> Result<Option<Vec<u8>>, EngineError> {
        debug!("Starting headless loop");

        let max_frames = max_frames.unwrap_or(100);
        let mut frame_count = 0;

        while frame_count < max_frames && !self.should_exit {
            // Update engine
            if let Some(engine) = &mut self.engine {
                engine.update();

                // Render frame
                if let Err(e) = engine.render() {
                    warn!("Render error in headless mode: {e}");
                }
            }

            frame_count += 1;

            // Simple exit condition for demo purposes
            if frame_count >= max_frames {
                self.should_exit = true;
            }
        }

        info!("Completed {frame_count} frames in headless mode");

        // Return the last rendered frame data
        if let Some(engine) = &self.engine {
            Ok(engine.get_rendered_frame_data())
        } else {
            Ok(None)
        }
    }

    /// Returns whether the application is in headless mode
    pub fn is_headless(&self) -> bool {
        self.is_headless
    }

    /// Returns whether the application has a window
    pub fn has_window(&self) -> bool {
        self.window.is_some()
    }

    /// Returns whether the application has an event loop
    pub fn has_event_loop(&self) -> bool {
        self.event_loop.is_some()
    }
}
