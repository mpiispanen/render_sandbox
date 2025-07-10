use crate::engine::{Engine, EngineError, PlaceholderEngine};
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
    engine: Box<dyn Engine>,
    should_exit: bool,
    is_headless: bool,
}

impl Application {
    /// Creates a new Application instance
    /// `headless`: If true, no window is created
    pub fn new(headless: bool) -> Result<Self, EngineError> {
        info!("Creating application (headless: {})", headless);

        if headless {
            // Headless mode - no window or event loop
            let engine = Box::new(PlaceholderEngine::new(None)?);
            Ok(Application {
                window: None,
                event_loop: None,
                engine,
                should_exit: false,
                is_headless: true,
            })
        } else {
            // Windowed mode - create window and event loop
            let event_loop = EventLoop::new().map_err(|e| {
                EngineError::InitializationError(format!("Failed to create event loop: {}", e))
            })?;

            let window = WindowBuilder::new()
                .with_title("Render Sandbox")
                .with_inner_size(PhysicalSize::new(800, 600))
                .build(&event_loop)
                .map_err(|e| {
                    EngineError::InitializationError(format!("Failed to create window: {}", e))
                })?;

            let engine = Box::new(PlaceholderEngine::new(Some(&window))?);

            Ok(Application {
                window: Some(window),
                event_loop: Some(event_loop),
                engine,
                should_exit: false,
                is_headless: false,
            })
        }
    }

    /// Runs the application loop
    /// Delegates to `run_windowed` or `run_headless` based on `is_headless`
    pub fn run(mut self) -> Result<(), EngineError> {
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
                                    debug!("Window resized to: {:?}", physical_size);
                                    self.engine.resize(physical_size);
                                }
                                WindowEvent::KeyboardInput { event, .. } => {
                                    // Handle escape key to exit
                                    if event.physical_key == winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Escape) {
                                        info!("Escape key pressed, exiting");
                                        self.should_exit = true;
                                        elwt.exit();
                                    } else {
                                        // Pass the original event directly to the engine
                                        self.engine.handle_input(&WindowEvent::KeyboardInput { event, device_id: unsafe { winit::event::DeviceId::dummy() }, is_synthetic: false });
                                    }
                                }
                                WindowEvent::RedrawRequested => {
                                    debug!("Redraw requested");
                                    if let Err(e) = self.engine.render() {
                                        error!("Render error: {}", e);
                                    }
                                }
                                _ => {
                                    self.engine.handle_input(&event);
                                }
                            }
                        }
                    }
                }
                Event::AboutToWait => {
                    // Update game logic
                    self.engine.update();

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
            self.engine.update();

            // Render frame
            if let Err(e) = self.engine.render() {
                warn!("Render error in headless mode: {}", e);
            }

            frame_count += 1;

            // Simple exit condition for demo purposes
            if frame_count >= max_frames {
                self.should_exit = true;
            }
        }

        info!("Completed {} frames in headless mode", frame_count);

        // Return the last rendered frame data
        Ok(self.engine.get_rendered_frame_data())
    }
}