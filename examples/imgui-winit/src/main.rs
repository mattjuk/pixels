#![deny(clippy::all)]
#![forbid(unsafe_code)]

use crate::gui::Gui;
use error_iter::ErrorIter as _;
use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::Window;

mod gui;

const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;
const BOX_SIZE: i16 = 64;

/// Representation of the application state. In this example, a box will bounce around the screen.
///
/// The world is resizable, meaning the backing pixel buffer can be resized without creating a
/// border around the screen.
struct World {
    width: i16,
    height: i16,
    box_x: i16,
    box_y: i16,
    velocity_x: i16,
    velocity_y: i16,
}

struct App {
    window: Option<Arc<Window>>,
    world: World,
    pixels: Option<Pixels<'static>>,
    gui: Option<Gui>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {        
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        let attributes = Window::default_attributes()
            .with_title("Hello Pixels + Dear ImGui")
            .with_inner_size(size)
            .with_min_inner_size(size);

        let window = Arc::new(event_loop.create_window(attributes).unwrap());
        self.window = Some(window.clone());
        let window_size = self.window.as_mut().unwrap().inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, window.clone());
        self.pixels = Some(Pixels::new(WIDTH, HEIGHT, surface_texture).unwrap());

        // Set up Dear ImGui
        self.gui = Some(Gui::new(&window, self.pixels.as_ref().unwrap()));
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                // Handle window close widget
                event_loop.exit();
            },
            WindowEvent::KeyboardInput { device_id: _, event, is_synthetic: _ } => {
                // Handle keyboard input for exit
                if let Key::Named(NamedKey::Escape) = event.logical_key {
                    if event.state.is_pressed() {
                        event_loop.exit();
                    }
                }
            },
            WindowEvent::Resized(size) => {
                // Resize the window
                self.pixels.as_mut().unwrap().resize_surface(size.width, size.height).ok();
            },
            WindowEvent::RedrawRequested => {
                // Update internal state and request a redraw
                self.world.update();
                self.world.draw(self.pixels.as_mut().unwrap().frame_mut());

                // Prepare Dear ImGui
                self.gui.as_mut().unwrap().prepare(self.window.as_ref().unwrap()).expect("gui.prepare() failed");

                // Render everything together
                let render_result = self.pixels.as_mut().unwrap().render_with(|encoder, render_target, context| {
                    // Render the world texture
                    context.scaling_renderer.render(encoder, render_target);

                    // Render Dear ImGui
                    self.gui.as_mut().unwrap().render(self.window.as_ref().unwrap(), encoder, render_target, context)?;

                    Ok(())
                });

                if let Err(err) = self.pixels.as_mut().unwrap().render() {
                    log_error("pixels.render", err);
                    event_loop.exit();
                }
            },
            _ => {},
        }

        self.window.as_ref().unwrap().request_redraw();
    }
}

fn main() -> Result<(), Error> {
    env_logger::init();

    let event_loop = EventLoop::new().expect("Couldn't create event loop!");

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App {
        window: None,
        world: World::new(),
        pixels: None,
        gui: None,
    };

    let _ = event_loop.run_app(&mut app);
    Ok(())
}

fn log_error<E: std::error::Error + 'static>(method_name: &str, err: E) {
    error!("{method_name}() failed: {err}");
    for source in err.sources().skip(1) {
        error!("  Caused by: {source}");
    }
}

impl World {
    /// Create a new `World` instance that can draw a moving box.
    fn new(width: u32, height: u32) -> Self {
        Self {
            width: width as i16,
            height: height as i16,
            box_x: 24,
            box_y: 16,
            velocity_x: 1,
            velocity_y: 1,
        }
    }

    /// Update the `World` internal state; bounce the box around the screen.
    fn update(&mut self) {
        if self.box_x <= 0 {
            self.velocity_x = 1;
        }
        if self.box_x + BOX_SIZE > self.width {
            self.velocity_x = -1;
        }
        if self.box_y <= 0 {
            self.velocity_y = 1;
        }
        if self.box_y + BOX_SIZE > self.height {
            self.velocity_y = -1;
        }

        self.box_x += self.velocity_x;
        self.box_y += self.velocity_y;
    }

    /// Resize the world
    fn resize(&mut self, width: u32, height: u32) {
        self.width = width as i16;
        self.height = height as i16;
    }

    /// Draw the `World` state to the frame buffer.
    ///
    /// Assumes the default texture format: `wgpu::TextureFormat::Rgba8UnormSrgb`
    fn draw(&self, frame: &mut [u8]) {
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let x = (i % self.width as usize) as i16;
            let y = (i / self.width as usize) as i16;

            let inside_the_box = x >= self.box_x
                && x < self.box_x + BOX_SIZE
                && y >= self.box_y
                && y < self.box_y + BOX_SIZE;

            let rgba = if inside_the_box {
                [0x5e, 0x48, 0xe8, 0xff]
            } else {
                [0x48, 0xb2, 0xe8, 0xff]
            };

            pixel.copy_from_slice(&rgba);
        }
    }
}
