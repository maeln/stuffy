extern crate cgmath;
extern crate gl;
extern crate gl_loader;

mod camera;
mod frame;
mod mesh;
mod scene;
mod shaders;
mod utils;

use std::sync::Arc;
use std::{collections::HashMap, path::Path};

use camera::Camera;
use frame::fbo::Framebuffer;
use shaders::shader_loader::ShaderManager;
use shaders::{Program, Shader};

use cgmath::prelude::*;
use cgmath::{perspective, Deg, Matrix4, Point3, Vector2, Vector3};

use std::ffi::CStr;

#[derive(Debug)]
pub struct Scene {
    pub shader_manager: ShaderManager,
    pub programs: Vec<u32>,
    pub framebuffers: Vec<Framebuffer>,
    pub binding: HashMap<u32, Option<usize>>,
    pub mesh: mesh::Mesh,
    pub size: Vector2<f32>,
    pub frame_nb: u32,
}

static mut m_scene: Option<Scene> = None;

#[no_mangle]
pub fn resize_window(width: f64, height: f64, dpi_ratio: f64) {
    let real_width = dpi_ratio * width;
    let real_height = dpi_ratio * height;

    unsafe {
        gl::Viewport(0, 0, real_width as i32, real_height as i32);

        if let Some(scene) = &mut m_scene {
            scene.size = Vector2 {
                x: real_width as f32,
                y: real_height as f32,
            };

            let mut new_fbs: Vec<Framebuffer> = Vec::new();
            for _ in &scene.framebuffers {
                new_fbs.push(Framebuffer::new_xhdr(real_width as i32, real_height as i32));
            }
            drop(&scene.framebuffers);
            scene.framebuffers = new_fbs;

            scene.frame_nb = 0;
        }
    }
}

#[no_mangle]
pub fn load_gl_symbol() {
    gl_loader::init_gl();
    gl::load_with(|symbol| gl_loader::get_proc_address(symbol) as *const _);
}

#[no_mangle]
pub fn print_gl_info() {
    unsafe {
        let gl_version = gl::GetString(gl::VERSION);
        let glsl_version = gl::GetString(gl::SHADING_LANGUAGE_VERSION);

        if gl_version != std::ptr::null() && glsl_version != std::ptr::null() {
            let gl_version = CStr::from_ptr(gl_version as *const i8);
            let glsl_version = CStr::from_ptr(glsl_version as *const i8);

            println!("OGL v.{:?}", gl_version);
            println!("GLSL v.{:?}", glsl_version);
        }
    }
}

#[no_mangle]
pub fn init_gl(width: f64, height: f64, dpi_ratio: f64) {
    let true_width = width * dpi_ratio;
    let true_height = height * dpi_ratio;

    unsafe {
        gl::Enable(gl::DEPTH_TEST);
        gl::DepthMask(gl::TRUE);
        gl::DepthFunc(gl::LEQUAL);
        gl::DepthRange(0.0, 1.0);
        gl::Enable(gl::DEPTH_CLAMP);
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::ONE, gl::ONE);
        gl::Viewport(0, 0, true_width as i32, true_height as i32);
    }
}

#[no_mangle]
pub fn init_scene(width: f64, height: f64, dpi_ratio: f64) {
    let true_width = width * dpi_ratio;
    let true_height = height * dpi_ratio;

    let mut programs: Vec<u32> = Vec::new();
    let mut framebuffers: Vec<Framebuffer> = Vec::new();
    let mut binding: HashMap<u32, Option<usize>> = HashMap::new();

    framebuffers.push(Framebuffer::new_xhdr(true_width as i32, true_height as i32));
    // framebuffers.push(Framebuffer::new_ldr(true_width as i32, true_height as i32));

    let mut shader_manager = ShaderManager::new();
    let path_tracer = shader_manager.load_program(&vec![
        Path::new("data/shaders/post/post.vs"),
        Path::new("data/shaders/post/post.fs"),
    ]);
    programs.push(path_tracer);
    binding.insert(path_tracer, Some(0));

    let grading_program = shader_manager.load_program(&vec![
        Path::new("data/shaders/grading/grading.vs"),
        Path::new("data/shaders/grading/grading.fs"),
    ]);
    programs.push(grading_program);
    binding.insert(grading_program, None);
    /*
        let swap = shader_manager.load_program(&vec![
            Path::new("data/shaders/tex/tex.vs"),
            Path::new("data/shaders/tex/tex.fs"),
        ]);
        programs.push(swap);
        binding.insert(swap, Some(0));
    */
    let mut fs_plane = mesh::Mesh::fs_quad();
    fs_plane.ready_up();

    unsafe {
        m_scene = Some(Scene {
            shader_manager,
            programs,
            framebuffers,
            binding,
            mesh: fs_plane,
            size: Vector2 {
                x: true_width as f32,
                y: true_height as f32,
            },
            frame_nb: 0,
        })
    }
}

#[no_mangle]
pub fn handle_mouse(dx: f32, dy: f32, speed: f32) {}

#[no_mangle]
pub fn quit() {
    unsafe {
        m_scene = None;
    }
}

#[no_mangle]
pub fn reset(fbo: u32) {
    unsafe {
        if let Some(scene) = &mut m_scene {
            scene.frame_nb = 0;
            for fb in &scene.framebuffers {
                gl::BindFramebuffer(gl::FRAMEBUFFER, fb.addr);
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT);
                gl::ClearColor(0.0, 0.0, 0.0, 0.0);
            }

            gl::BindFramebuffer(gl::FRAMEBUFFER, fbo);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT);
            gl::ClearColor(0.0, 0.0, 0.0, 0.0);
        }
    }
}

#[no_mangle]
pub fn display_loop(time: f64, fbo: u32, reset_on_reload: bool) {
    unsafe {
        if let Some(scene) = &mut m_scene {
            let should_clear = scene.shader_manager.handle_reload();
            if should_clear && reset_on_reload {
                reset(fbo);
            }

            let mut p = 0;
            for program in &scene.programs {
                let bind = scene.binding.get(program).clone().unwrap();
                if bind.is_some() {
                    gl::BindFramebuffer(
                        gl::FRAMEBUFFER,
                        scene
                            .framebuffers
                            .get_unchecked(bind.unwrap() as usize)
                            .addr,
                    );
                } else {
                    gl::BindFramebuffer(gl::FRAMEBUFFER, fbo);
                }
                if p > 0 {
                    gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT);
                    gl::ClearColor(0.0, 0.0, 0.0, 0.0);
                    gl::Disable(gl::BLEND);
                } else {
                    gl::Clear(gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT);
                    gl::Enable(gl::BLEND);
                    gl::BlendFunc(gl::ONE, gl::ONE);
                }
                if let Some(program) = scene.shader_manager.get_program(program.clone()) {
                    let prog = program.lock().unwrap();
                    prog.bind();
                    prog.set_vec2("resolution", &scene.size);
                    prog.set_float("frame_nb", scene.frame_nb as f32);
                    prog.set_float("time", time as f32);
                }
                let mut i: u32 = 0;
                for tex in &scene.framebuffers {
                    gl::ActiveTexture(gl::TEXTURE0 + i);
                    gl::BindTexture(gl::TEXTURE_2D, tex.color_attachment.unwrap());
                    i += 1;
                }
                scene.mesh.draw();
                p += 1;
            }

            // Show scene
            gl::BindFramebuffer(gl::FRAMEBUFFER, fbo);
            print!("\rsamples: {}", scene.frame_nb);
            scene.frame_nb += 1;
        }
    }
}
