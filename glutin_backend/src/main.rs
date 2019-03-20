extern crate glutin;
extern crate peglrs;

use glutin::ContextTrait;
use std::time::{Duration, Instant};

fn main() {
    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new()
        .with_title("Hello world!")
        .with_dimensions(glutin::dpi::LogicalSize::new(1024.0, 768.0))
        .with_decorations(true)
        .with_transparency(false);
    let window_context = glutin::ContextBuilder::new()
        .build_windowed(window, &events_loop)
        .unwrap();
    unsafe {
        window_context.make_current().unwrap();
    }

    peglrs::load_gl_symbol();
    peglrs::print_gl_info();

    let dpi_ratio = window_context.get_hidpi_factor();
    let size = window_context.get_inner_size().unwrap();
    peglrs::init_gl(size.width, size.height, dpi_ratio);
    peglrs::init_scene(size.width, size.height, dpi_ratio);

    let mut running = true;
    let mut time = Instant::now();
    while running {
        events_loop.poll_events(|event| match event {
            _ => (),
        });

        let elapsed = time.elapsed();
        peglrs::display_loop(elapsed.as_millis() as f64);
        window_context.swap_buffers().unwrap();
    }
}
