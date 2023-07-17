#![deny(clippy::all)]
#![forbid(unsafe_code)]

use std::time::Duration;

use async_winit::dpi::{LogicalSize, PhysicalSize};
use async_winit::event::{ElementState, VirtualKeyCode};
use async_winit::event_loop::EventLoop;
use async_winit::window::WindowBuilder;
use error_iter::ErrorIter as _;
// use futures_lite::{StreamExt as _};
use async_winit::Timer;
use futures_lite::prelude::*;
use log::error;
use pixels::{Pixels, SurfaceTexture};
use winit::event::KeyboardInput;

const WIDTH: u32 = 320;
const HEIGHT: u32 = 240;
const BOX_SIZE: i16 = 64;

/// Representation of the application state. In this example, a box will bounce around the screen.
struct World {
    box_x: i16,
    box_y: i16,
    velocity_x: i16,
    velocity_y: i16,
}

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window_target = event_loop.window_target().clone();

    event_loop.block_on(async move {
        // Wait for event loop to resume
        window_target.resumed().await;

        let window = {
            let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
            WindowBuilder::new()
                .with_title("Hello Pixels")
                .with_inner_size(size)
                .with_min_inner_size(size)
                .build()
                .await
                .unwrap()
        };

        let mut pixels = {
            let window_size = window.inner_size().await;
            let surface_texture =
                SurfaceTexture::new(window_size.width, window_size.height, &window);

            match Pixels::new(WIDTH, HEIGHT, surface_texture) {
                Ok(pixels) => pixels,
                Err(err) => {
                    log_error("pixels.render", err);
                    window_target.exit().await
                }
            }
        };

        // Sigh! We need to aggregate events from multiple streams into a single type
        enum Events {
            Close,
            Resize(PhysicalSize<u32>),
            Redraw,
            Continue,
            Timer,
        }

        // Handle input events
        let close = window.close_requested().wait_many().map(|_| Events::Close);
        let input = window.keyboard_input().wait_many().map(|key| {
            if let KeyboardInput {
                state: ElementState::Pressed,
                virtual_keycode: Some(VirtualKeyCode::Escape),
                ..
            } = key.input
            {
                Events::Close
            } else {
                Events::Continue
            }
        });

        // Resize the window
        let resize = window.resized().wait_many().map(Events::Resize);

        // Update internal state and request a redraw
        let redraw = window
            .redraw_requested()
            .wait_many()
            .map(|_| Events::Redraw);

        // Periodic timer for 60 fps World updates
        let timer = Timer::interval(Duration::from_micros(16_666)).map(|_| Events::Timer);

        let mut events = redraw.or(timer).or(input).or(resize).or(close);
        let mut world = World::new();

        // This is the actual event loop. Just process events as they come!
        while let Some(event) = events.next().await {
            match event {
                Events::Close => break,
                Events::Continue => (),
                Events::Timer => {
                    world.update();
                    window.request_redraw();
                }
                Events::Redraw => {
                    world.draw(pixels.frame_mut());
                    if let Err(err) = pixels.render() {
                        log_error("pixels.render", err);
                        break;
                    }
                }
                Events::Resize(size) => {
                    if let Err(err) = pixels.resize_surface(size.width, size.height) {
                        log_error("pixels.resize_surface", err);
                        break;
                    }
                }
            }
        }

        window_target.exit().await
    });
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
