#![deny(clippy::all)]
#![forbid(unsafe_code)]

use std::sync::Arc;

use crate::gui::Framework;
use error_iter::ErrorIter as _;
use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::window::Window;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};

mod gui;

const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;
const BOX_SIZE: i16 = 64;

/// Representation of the application state. In this example, a box will bounce around the screen.
struct World {
    box_x: i16,
    box_y: i16,
    velocity_x: i16,
    velocity_y: i16,
}

struct App {
    window: Option<Arc<Window>>,
    world: World,
    pixels: Option<Pixels<'static>>,
    framework: Option<Framework>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        let attributes = Window::default_attributes()
            .with_title("Hello Pixels + egui")
            .with_inner_size(size)
            .with_min_inner_size(size);
        let window = Arc::new(event_loop.create_window(attributes).unwrap());
        let scale_factor = window.as_ref().scale_factor() as f32;
        self.window = Some(window.clone());
        let window_size = self.window.as_mut().unwrap().inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, window.clone());
        self.pixels = Some(Pixels::new(WIDTH, HEIGHT, surface_texture).unwrap());
        self.framework = Some(Framework::new(
            event_loop,
            window_size.width,
            window_size.height,
            scale_factor,
            self.pixels.as_ref().unwrap(),
        ));
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                println!("Closing!");
                event_loop.exit();
            },
            WindowEvent::ScaleFactorChanged { scale_factor, inner_size_writer: _ } => {
                self.framework.as_mut().unwrap().scale_factor(scale_factor);
            },
            WindowEvent::KeyboardInput { device_id: _, event, is_synthetic: _ } => {
                if event.state == winit::event::ElementState::Pressed {
                    if let Key::Named(NamedKey::Escape) = event.logical_key {
                        println!("Escape pressed!");
                        event_loop.exit();
                    }
                }
            },
            WindowEvent::Resized(size) => {
                self.pixels.as_mut().unwrap().resize_surface(size.width, size.height).ok();
                self.framework.as_mut().unwrap().resize(size.width, size.height);
            },
            WindowEvent::RedrawRequested => {
                self.world.update();
                self.world.draw(self.pixels.as_mut().unwrap().frame_mut());
                self.framework.as_mut().unwrap().prepare(self.window.as_ref().unwrap());

                let render_result = self.pixels.as_mut().unwrap().render_with(|encoder, render_target, context| {
                    context.scaling_renderer.render(encoder, render_target);
                    self.framework.as_mut().unwrap().render(encoder, render_target, context);
                    Ok(())
                });

                if let Err(err) = render_result {
                    log_error("pixels.render", err);
                    event_loop.exit();
                }
            },
            _ => {
                self.framework.as_mut().unwrap().handle_event(self.window.as_ref().unwrap(), &event);
            },
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
        framework: None,    
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
    fn new() -> Self {
        Self {
            box_x: 24,
            box_y: 16,
            velocity_x: 1,
            velocity_y: 1,
        }
    }

    /// Update the `World` internal state; bounce the box around the screen.
    fn update(&mut self) {
        if self.box_x <= 0 || self.box_x + BOX_SIZE > WIDTH as i16 {
            self.velocity_x *= -1;
        }
        if self.box_y <= 0 || self.box_y + BOX_SIZE > HEIGHT as i16 {
            self.velocity_y *= -1;
        }

        self.box_x += self.velocity_x;
        self.box_y += self.velocity_y;
    }

    /// Draw the `World` state to the frame buffer.
    ///
    /// Assumes the default texture format: `wgpu::TextureFormat::Rgba8UnormSrgb`
    fn draw(&self, frame: &mut [u8]) {
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let x = (i % WIDTH as usize) as i16;
            let y = (i / WIDTH as usize) as i16;

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
