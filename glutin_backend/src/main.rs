extern crate glutin;
extern crate peglrs;

use std::time::Instant;

use glutin::{
    event::{ElementState, Event, MouseButton, VirtualKeyCode, WindowEvent},
    event_loop::ControlFlow,
};

fn main() {
    let events_loop = glutin::event_loop::EventLoop::new();
    let window = glutin::window::WindowBuilder::new()
        .with_title("Stuffy (ESC)")
        .with_inner_size(glutin::dpi::LogicalSize::new(600.0, 600.0))
        .with_decorations(true);
    let window_context = glutin::ContextBuilder::new()
        .build_windowed(window, &events_loop)
        .unwrap();

    let window_context = unsafe { window_context.make_current().unwrap() };

    peglrs::load_gl_symbol();
    peglrs::print_gl_info();

    let dpi_ratio = window_context.window().scale_factor();
    let size = window_context.window().inner_size();
    peglrs::init_gl(size.width as f64, size.height as f64, dpi_ratio);
    peglrs::init_scene(size.width as f64, size.height as f64, dpi_ratio);

    let mut mouse_init = false;
    let mut mouse_prev: (f64, f64) = (0.0, 0.0);
    let mut mouse_pressed = false;
    let mut pause = false;

    let counter = Instant::now();
    events_loop.run(move |event, _, control_flow| {
        let mut stop = false;

        if !pause {
            peglrs::display_loop(counter.elapsed().as_millis() as f64 / 1000.0, 0, true);
            window_context.swap_buffers().unwrap();
        }

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Focused(focused) => {
                    pause = !focused;
                    if pause {
                        println!("pause");
                        window_context.window().set_title("Stuffy *PAUSED* (ESC)");
                    } else {
                        println!("unpause");
                        window_context.window().set_title("Stuffy (ESC)");
                    }
                }
                WindowEvent::CloseRequested => {
                    peglrs::quit();
                    stop = true;
                }
                WindowEvent::KeyboardInput { input, .. } => {
                    if let Some(vkey) = input.virtual_keycode {
                        match vkey {
                            VirtualKeyCode::Escape => {
                                stop = true;
                            }
                            VirtualKeyCode::P => {
                                peglrs::reset(0);
                                window_context.swap_buffers().unwrap();
                            }
                            _ => {}
                        }
                    }
                }
                WindowEvent::Resized(size) => {
                    let dpi = window_context.window().scale_factor();
                    peglrs::resize_window(size.width as f64, size.height as f64, dpi);
                }
                WindowEvent::ScaleFactorChanged {
                    scale_factor,
                    new_inner_size,
                } => {
                    peglrs::resize_window(
                        new_inner_size.width as f64,
                        new_inner_size.height as f64,
                        scale_factor,
                    );
                }
                WindowEvent::CursorMoved { position, .. } => {
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
                WindowEvent::MouseInput { state, button, .. } => {
                    if state == ElementState::Pressed && button == MouseButton::Left {
                        mouse_pressed = true;
                    }

                    if state == ElementState::Released && button == MouseButton::Left {
                        mouse_pressed = false;
                        mouse_init = false;
                    }
                }
                _ => (),
            },
            _ => (),
        };

        if stop {
            *control_flow = ControlFlow::Exit;
        }
    });
}
