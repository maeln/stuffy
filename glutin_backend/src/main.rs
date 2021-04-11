extern crate glutin;
extern crate peglrs;

use glutin::ContextTrait;
use std::time::{Duration, Instant};

fn main() {
    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new()
        .with_title("Stuffy (ESC)")
        .with_dimensions(glutin::dpi::LogicalSize::new(600.0, 600.0))
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
    let mut mouse_init = false;
    let mut mouse_prev: (f64, f64) = (0.0, 0.0);
    let mut mouse_next: (f64, f64) = (0.0, 0.0);
    let mut mouse_pressed = false;
    let mut loop_render = false;
    let mut pause = false;

    events_loop.run_forever(|event| {
        let mut stop = false;

        let elapsed = time.elapsed();
        if !pause {
            peglrs::display_loop(elapsed.as_millis() as f64 / 1000.0, 0, true);
            window_context.swap_buffers().unwrap();
        }
        match event {
            glutin::Event::WindowEvent { event, .. } => match event {
                glutin::WindowEvent::Focused(focused) => {
                    pause = !focused;
                    if pause {
                        window_context.set_title("Stuffy *PAUSED* (ESC)");
                    } else {
                        window_context.set_title("Stuffy (ESC)");
                    }
                }
                glutin::WindowEvent::CloseRequested => {
                    peglrs::quit();
                    stop = true;
                }
                glutin::WindowEvent::KeyboardInput { input, .. } => {
                    if let Some(vkey) = input.virtual_keycode {
                        match vkey {
                            glutin::VirtualKeyCode::Escape => {
                                stop = true;
                            }
                            glutin::VirtualKeyCode::P => {
                                peglrs::reset(0);
                                time = Instant::now();
                                window_context.swap_buffers().unwrap();
                            }
                            _ => {}
                        }
                    }
                }
                glutin::WindowEvent::Resized(size) => {
                    let dpi = window_context.get_hidpi_factor();
                    peglrs::resize_window(size.width, size.height, dpi);
                    window_context.resize(size.to_physical(dpi));
                }
                glutin::WindowEvent::HiDpiFactorChanged(dpi) => {
                    let size = window_context.get_inner_size().unwrap();
                    peglrs::resize_window(size.width, size.height, dpi);
                    window_context.resize(size.to_physical(dpi));
                }
                glutin::WindowEvent::CursorMoved { position, .. } => {
                    if mouse_pressed {
                        if !mouse_init {
                            mouse_prev = (position.x, position.y);
                            mouse_init = true;
                        } else {
                            let mouse_delta =
                                (mouse_prev.0 - position.x, mouse_prev.1 - position.y);
                            peglrs::handle_mouse(mouse_delta.0 as f32, mouse_delta.1 as f32, 0.001);
                            mouse_prev = (position.x, position.y);
                        }
                    }
                }
                glutin::WindowEvent::MouseInput { state, button, .. } => {
                    if state == glutin::ElementState::Pressed && button == glutin::MouseButton::Left
                    {
                        mouse_pressed = true;
                    }

                    if state == glutin::ElementState::Released
                        && button == glutin::MouseButton::Left
                    {
                        mouse_pressed = false;
                        mouse_init = false;
                    }
                }
                _ => (),
            },
            _ => (),
        };

        if stop {
            return glutin::ControlFlow::Break;
        }
        return glutin::ControlFlow::Continue;
    });
}
