extern crate glutin;
extern crate peglrs;

use std::time::Instant;

use cgmath::{InnerSpace, Vector2, Vector3};
use glutin::{event::{ElementState, Event, KeyboardInput, MouseButton, VirtualKeyCode, WindowEvent}, event_loop::ControlFlow};

use glutin::event::MouseScrollDelta;

fn main() {
    let events_loop = glutin::event_loop::EventLoop::new();
    let window = glutin::window::WindowBuilder::new()
        .with_title("Stuffy (ESC)")
        .with_inner_size(glutin::dpi::LogicalSize::new(800.0, 600.0))
        .with_decorations(true);
    let window_context = glutin::ContextBuilder::new()
        .build_windowed(window, &events_loop)
        .unwrap();

    let window_context = unsafe { window_context.make_current().unwrap() };

    peglrs::load_gl_symbol();
    peglrs::print_gl_info();

    let size = window_context.window().inner_size();
    // We put the dpi at 1.0 because the size is already scaled.
    peglrs::init_gl(size.width as f64, size.height as f64, 1.0);
    peglrs::init_scene(size.width as f64, size.height as f64, 1.0);

    let mut mouse_init = false;
    let mut mouse_prev: (f64, f64) = (0.0, 0.0);
    let mut mouse_pressed = false;
    let mut pause = false;
    let mut iter: usize = 0;

    let mut hangle: f32 = -2.2;
    let mut vangle: f32 = 0.03;
    let mut dt: f32 = 0.0;

    let mut cam_eye: Vector3<f32> = Vector3::new(3.1415, 0.906, 2.308);
    let mut focus_pos = Vector2::new(0.57, 0.44833332);
    let mut aperture: f32 = 0.4;
    let mut cam_direction = Vector3::new(
        vangle.cos() * hangle.sin(), 
        vangle.sin(), 
        vangle.cos() * hangle.cos());
    let right = Vector3::new(
        (hangle - 3.1415 / 2.0).sin(), 
        0.0, 
        (hangle - 3.1415 / 2.0).cos());
    let mut cam_up = right.cross(cam_direction);
    
    peglrs::update_camera(
        cam_eye,
        cam_eye + cam_direction,
        cam_up,
        focus_pos,
        aperture,
    );


    let mut forward = false;
    let mut backward = false;
    let mut leftward = false;
    let mut rightward = false;
    let mut upward = false;
    let mut downward = false;
    let mouse_speed: f32 = 0.01;
    let keyboard_speed: f32 = 10.0;
    let mut zero_aperture = false;
    let mut mouse_pos = Vector2::new(0.0, 0.0);

    let counter = Instant::now();
    events_loop.run(move |event, _, control_flow| {
        let loop_start = Instant::now();
        let mut stop = false;

        let mut cam_moved = false;
        let mut mouse_moved = false;
        let mut mouse_dx: f32 = 0.0;
        let mut mouse_dy: f32 = 0.0;
        let mut mouse_ds: f32 = 0.0;

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Focused(focused) => {
                    pause = !focused;
                    if pause {
                        window_context.window().set_title("Stuffy *PAUSED* (ESC)");
                    } else {
                        window_context.window().set_title("Stuffy (ESC)");
                    }
                }
                WindowEvent::CloseRequested => {
                    peglrs::quit();
                    stop = true;
                }
                WindowEvent::KeyboardInput { input: KeyboardInput { virtual_keycode: Some(virtual_keycode), state, .. }, .. } => {
                    match (virtual_keycode, state) {
                        (VirtualKeyCode::Escape, ElementState::Pressed) => {
                            stop = true;
                        }
                        (VirtualKeyCode::P, ElementState::Pressed) => {
                            iter = 0;
                            peglrs::reset(0);
                            window_context.swap_buffers().unwrap();
                        }
                        (VirtualKeyCode::O, ElementState::Pressed) => {
                            println!("samples: {}", iter);
                        }
                        (VirtualKeyCode::I, ElementState::Pressed) => {
                            println!("Cam:\neye: {:?}\ndirection: {:?}\n, up: {:?}\n focus pos: {:?}\n, aperture: {}\n hangle: {}\n vangle: {}", 
                                    cam_eye, cam_direction, cam_up, focus_pos, aperture, hangle, vangle);
                        }
                        (VirtualKeyCode::V, ElementState::Pressed) => {
                            zero_aperture = true;
                            cam_moved = true;
                        }
                        (VirtualKeyCode::V, ElementState::Released) => {
                            zero_aperture = false;
                            cam_moved = true;
                        }
                        (VirtualKeyCode::A, ElementState::Pressed) => {
                            leftward = true;
                        }
                        (VirtualKeyCode::A, ElementState::Released) => {
                            leftward = false;
                        }
                        (VirtualKeyCode::D, ElementState::Pressed) => {
                            rightward = true;
                        }
                        (VirtualKeyCode::D, ElementState::Released) => {
                            rightward = false;
                        }
                        (VirtualKeyCode::W, ElementState::Pressed) => {
                            forward = true;
                        }
                        (VirtualKeyCode::W, ElementState::Released) => {
                            forward = false;
                        }
                        (VirtualKeyCode::S, ElementState::Pressed) => {
                            backward = true;
                        }
                        (VirtualKeyCode::S, ElementState::Released) => {
                            backward = false;
                        }
                        (VirtualKeyCode::Space, ElementState::Pressed) => {
                            upward = true;
                        }
                        (VirtualKeyCode::Space, ElementState::Released) => {
                            upward = false;
                        }
                        (VirtualKeyCode::LShift, ElementState::Pressed) => {
                            downward = true;
                        }
                        (VirtualKeyCode::LShift, ElementState::Released) => {
                            downward = false;
                        }
                        _ => {}
                    }
                }
                WindowEvent::Resized(size) => {
                    // We put the dpi at 1.0 because the size is already scaled.
                    peglrs::resize_window(size.width as f64, size.height as f64, 1.0);
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
                    mouse_pos = Vector2::new(position.x as f32, position.y as f32);
                    if mouse_pressed {
                        if !mouse_init {
                            mouse_prev = (position.x, position.y);
                            mouse_init = true;
                        } else {
                            let mouse_delta =
                                (mouse_prev.0 - position.x, mouse_prev.1 - position.y);
                            mouse_dx = mouse_delta.0 as f32;
                            mouse_dy = mouse_delta.1 as f32;
                            cam_moved = true;
                            peglrs::handle_mouse(mouse_delta.0 as f32, mouse_delta.1 as f32, mouse_speed);
                            mouse_prev = (position.x, position.y);
                            mouse_moved = true;
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

                    if state == ElementState::Released && button == MouseButton::Right {
                        focus_pos = Vector2::new(
                            mouse_pos.x / window_context.window().inner_size().width as f32, 
                            mouse_pos.y / window_context.window().inner_size().height as f32
                        );
                        cam_moved = true;
                    }
                }
                WindowEvent::MouseWheel { delta, .. } => {
                    if let MouseScrollDelta::LineDelta(_, y) = delta {
                        mouse_ds = y;
                        mouse_moved = true;
                    }
                }
                _ => (),
            },
            _ => (),
        };

        if leftward {
            let mut left = cam_up.cross(cam_direction);
            let len = left.dot(left).sqrt();
            left = left / len;
            cam_eye += left * keyboard_speed * dt;
            cam_moved = true;
        }

        if rightward {
            let mut right = cam_direction.cross(cam_up);
            let len = right.dot(right).sqrt();
            right = right / len;
            cam_eye += right * keyboard_speed * dt;
            cam_moved = true;
        }

        if forward {
            cam_eye += cam_direction * keyboard_speed * dt;
            cam_moved = true;
        }

        if backward {
            cam_eye -= cam_direction * keyboard_speed * dt;
            cam_moved = true;
        }

        if upward {
            cam_eye += Vector3::new(0.0, 1.0, 0.0) * keyboard_speed * dt;
            cam_moved = true;
        }

        if downward {
            cam_eye -= Vector3::new(0.0, 1.0, 0.0) * keyboard_speed * dt;
            cam_moved = true;
        }

        if mouse_moved {
            aperture += mouse_ds / 50.0;
            
            hangle += mouse_dx * mouse_speed;
            vangle += mouse_dy * mouse_speed;

            cam_direction = Vector3::new(
                vangle.cos() * hangle.sin(), 
                vangle.sin(), 
                vangle.cos() * hangle.cos());
            let right = Vector3::new(
                (hangle - 3.1415 / 2.0).sin(), 
                0.0, 
                (hangle - 3.1415 / 2.0).cos());
            cam_up = right.cross(cam_direction);
            cam_moved = true;
        }

        
        if cam_moved {
            iter = 0;
            peglrs::update_camera(
                cam_eye,
                cam_eye + cam_direction,
                cam_up,
                focus_pos,
                if zero_aperture { 0.0 } else { aperture },
            );
            peglrs::reset(0);
            // window_context.swap_buffers().unwrap();
        }

        if stop {
            *control_flow = ControlFlow::Exit;
        }

        if !pause {
            peglrs::display_loop(counter.elapsed().as_millis() as f64 / 1000.0, 0, true);
            window_context.swap_buffers().unwrap();
            iter += 1;
        }
        let ms = Instant::now().duration_since(loop_start).as_millis();
        dt = ms as f32 / 1000.0;
    });
}
