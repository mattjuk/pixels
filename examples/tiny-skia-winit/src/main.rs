#![deny(clippy::all)]
#![forbid(unsafe_code)]

use error_iter::ErrorIter as _;
use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use std::time::Instant;
use tiny_skia::Pixmap;
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::KeyCode;
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

mod shape;

const WIDTH: u32 = 500;
const HEIGHT: u32 = 500;

fn main() -> Result<(), Error> {
    env_logger::init();
    let event_loop = EventLoop::new().expect("Couldn't create event loop!");
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Hello tiny-skia")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);

        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };

    let mut drawing = Pixmap::new(WIDTH, HEIGHT).unwrap();
    let now = Instant::now();

    event_loop.run(move |event, window_target| {
        // Handle input events
        if input.update(&event) {
            // Close events
            if input.key_pressed(KeyCode::Escape) || input.close_requested() {
                window_target.exit();
                return;
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                if let Err(err) = pixels.resize_surface(size.width, size.height) {
                    log_error("pixels.resize_surface", err);
                    window_target.exit();
                    return;
                }
            }

            // Update internal state and request a redraw
            shape::draw(&mut drawing, now.elapsed().as_secs_f32());
            window.request_redraw();
        }

        if let Event::WindowEvent { event: WindowEvent::RedrawRequested, .. } = event {
            // Draw the current frame
            pixels.frame_mut().copy_from_slice(drawing.data());

            if let Err(err) = pixels.render() {
                log_error("pixels.render", err);
                window_target.exit();
            }           
        }
    }).unwrap();

    Ok(())
}

fn log_error<E: std::error::Error + 'static>(method_name: &str, err: E) {
    error!("{method_name}() failed: {err}");
    for source in err.sources().skip(1) {
        error!("  Caused by: {source}");
    }
}
