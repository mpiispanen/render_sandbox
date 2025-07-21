use crate::engine::{Engine, EngineError, RealTimeEngine};
use crate::Args;
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
    args: Args,
}

impl Application {
    /// Creates a new Application instance
    pub fn new(args: Args) -> Result<Self, EngineError> {
        info!("Creating application (headless: {})", args.headless);

        if args.headless {
            // Headless mode - no window or event loop
            Ok(Application {
                window: None,
                event_loop: None,
                engine: None,
                should_exit: false,
                args,
            })
        } else {
            // Windowed mode - create window and event loop
            let event_loop = EventLoop::new().map_err(|e| {
                EngineError::InitializationError(format!("Failed to create event loop: {e}"))
            })?;

            let window = WindowBuilder::new()
                .with_title("Render Sandbox")
                .with_inner_size(PhysicalSize::new(args.width, args.height))
                .build(&event_loop)
                .map_err(|e| {
                    EngineError::InitializationError(format!("Failed to create window: {e}"))
                })?;

            Ok(Application {
                window: Some(window),
                event_loop: Some(event_loop),
                engine: None,
                should_exit: false,
                args,
            })
        }
    }

    /// Initialize the engine asynchronously
    async fn initialize_engine(&mut self) -> Result<(), EngineError> {
        info!("Initializing engine");

        let engine = if self.args.headless {
            Box::new(
                RealTimeEngine::new(
                    None,
                    &self.args.gltf_path,
                    self.args.width,
                    self.args.height,
                )
                .await?,
            ) as Box<dyn Engine>
        } else {
            Box::new(
                RealTimeEngine::new(
                    self.window.as_ref(),
                    &self.args.gltf_path,
                    self.args.width,
                    self.args.height,
                )
                .await?,
            ) as Box<dyn Engine>
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

        if self.args.headless {
            info!("Starting headless mode");
            self.run_headless()?;
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

    /// Runs the application in headless mode and saves the rendered image
    fn run_headless(mut self) -> Result<(), EngineError> {
        debug!("Starting headless loop");

        // Render a few frames to ensure everything is properly initialized
        let warmup_frames = 5;
        let mut frame_count = 0;

        while frame_count < warmup_frames {
            // Update engine
            if let Some(engine) = &mut self.engine {
                engine.update();

                // Render frame
                if let Err(e) = engine.render() {
                    warn!("Render error in headless mode: {e}");
                }
            }

            frame_count += 1;
        }

        info!("Completed {frame_count} warmup frames in headless mode");

        // Get the final rendered frame data and save it
        if let Some(engine) = &self.engine {
            if let Some(frame_data) = engine.get_rendered_frame_data() {
                info!("Rendered frame data: {} bytes", frame_data.len());
                self.save_image_data(&frame_data)?;
            } else {
                return Err(EngineError::RenderingError(
                    "No frame data available from engine".to_string(),
                ));
            }
        } else {
            return Err(EngineError::RenderingError(
                "Engine not available".to_string(),
            ));
        }

        Ok(())
    }

    /// Save frame data as an image file
    fn save_image_data(&self, data: &[u8]) -> Result<(), EngineError> {
        info!("Saving image to: {}", self.args.output);

        // Convert RGBA data to image format
        let img = image::RgbaImage::from_raw(self.args.width, self.args.height, data.to_vec())
            .ok_or_else(|| {
                EngineError::RenderingError("Failed to create image from frame data".to_string())
            })?;

        // Determine format based on file extension or args.format
        let format = match self.args.format.to_lowercase().as_str() {
            "png" => image::ImageFormat::Png,
            "jpg" | "jpeg" => image::ImageFormat::Jpeg,
            "bmp" => image::ImageFormat::Bmp,
            _ => {
                warn!("Unknown format '{}', defaulting to PNG", self.args.format);
                image::ImageFormat::Png
            }
        };

        // Save the image
        img.save_with_format(&self.args.output, format)
            .map_err(|e| EngineError::RenderingError(format!("Failed to save image: {}", e)))?;

        info!("Successfully saved image: {}", self.args.output);
        Ok(())
    }

    /// Returns whether the application is in headless mode
    pub fn is_headless(&self) -> bool {
        self.args.headless
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
