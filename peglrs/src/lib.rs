// MUST REMOVE WHEN REALLY DOING STUFF
#![allow(dead_code)]
#![allow(non_upper_case_globals)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(unreachable_patterns)]
#![allow(non_camel_case_types)]

extern crate cgmath;
extern crate gl;
extern crate gl_loader;

mod camera;
mod frame;
mod mesh;
mod scene;
mod shaders;
mod utils;

use std::path::Path;
use std::sync::Arc;

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
    pub program: u32,
    pub fb_program: u32,
    pub split_program: u32,
    pub mesh: mesh::Mesh,
    pub size: Vector2<f32>,
    pub scenebuffer: Framebuffer,
    pub backbuffer: Framebuffer,
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
            drop(&scene.backbuffer);
            scene.backbuffer = Framebuffer::new_ldr(real_width as i32, real_height as i32);

            drop(&scene.scenebuffer);
            scene.scenebuffer = Framebuffer::new_ldr(real_width as i32, real_height as i32);

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
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        gl::Viewport(0, 0, true_width as i32, true_height as i32);
    }
}

#[no_mangle]
pub fn init_scene(width: f64, height: f64, dpi_ratio: f64) {
    let true_width = width * dpi_ratio;
    let true_height = height * dpi_ratio;

    let mut shader_manager = ShaderManager::new();
    let program = shader_manager.load_program(&vec![
        Path::new("data/shaders/post/post.vs"),
        Path::new("data/shaders/post/post.fs"),
    ]);

    let fb_program = shader_manager.load_program(&vec![
        Path::new("data/shaders/tex/tex.vs"),
        Path::new("data/shaders/tex/tex.fs"),
    ]);

    let split_program = shader_manager.load_program(&vec![
        Path::new("data/shaders/split/split.vs"),
        Path::new("data/shaders/split/split.fs"),
    ]);

    let mut fs_plane = mesh::Mesh::fs_quad();
    fs_plane.ready_up();

    unsafe {
        m_scene = Some(Scene {
            shader_manager,
            program: program,
            fb_program: fb_program,
            split_program: split_program,
            mesh: fs_plane,
            size: Vector2 {
                x: true_width as f32,
                y: true_height as f32,
            },
            backbuffer: Framebuffer::new_ldr(true_width as i32, true_height as i32),
            scenebuffer: Framebuffer::new_ldr(true_width as i32, true_height as i32),
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
            gl::BindFramebuffer(gl::FRAMEBUFFER, scene.backbuffer.addr);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT);
            gl::ClearColor(0.0, 0.0, 0.0, 0.0);
            gl::BindFramebuffer(gl::FRAMEBUFFER, scene.scenebuffer.addr);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT);
            gl::ClearColor(0.0, 0.0, 0.0, 0.0);
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT);
            gl::ClearColor(0.0, 0.0, 0.0, 0.0);
        }
    }
}

#[no_mangle]
pub fn display_loop(time: f64, fbo: u32) {
    unsafe {
        if let Some(scene) = &mut m_scene {
            scene.shader_manager.handle_reload();

            // Draw the main frame
            gl::BindFramebuffer(gl::FRAMEBUFFER, scene.scenebuffer.addr);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT);
            gl::ClearColor(0.0, 0.0, 0.0, 0.0);
            if let Some(program) = scene.shader_manager.get_program(scene.program) {
                let prog = program.lock().unwrap();
                prog.bind();
                prog.set_float("time", time as f32);
                prog.set_vec2("resolution", &scene.size);
                prog.set_float("frame_nb", scene.frame_nb as f32);
            }

            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, scene.backbuffer.color_attachment.unwrap());
            scene.mesh.draw();

            // Draw the split view on the default framebuffer
            gl::BindFramebuffer(gl::FRAMEBUFFER, fbo);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT);
            gl::ClearColor(0.0, 0.0, 0.0, 0.0);
            if let Some(program) = scene.shader_manager.get_program(scene.split_program) {
                let prog = program.lock().unwrap();
                prog.bind();
                prog.set_vec2("resolution", &scene.size);
                prog.set_i32("scenebuffer", 0);
                prog.set_i32("backbuffer", 1);
            }

            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, scene.scenebuffer.color_attachment.unwrap());
            gl::ActiveTexture(gl::TEXTURE0 + 1);
            gl::BindTexture(gl::TEXTURE_2D, scene.backbuffer.color_attachment.unwrap());
            scene.mesh.draw();

            // Update backbuffer
            gl::BindFramebuffer(gl::FRAMEBUFFER, scene.backbuffer.addr);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT);
            gl::ClearColor(0.0, 0.0, 0.0, 0.0);
            if let Some(program) = scene.shader_manager.get_program(scene.fb_program) {
                let prog = program.lock().unwrap();
                prog.bind();
                prog.set_vec2("resolution", &scene.size);
            }

            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, scene.scenebuffer.color_attachment.unwrap());
            scene.mesh.draw();

            // Show scene
            gl::BindFramebuffer(gl::FRAMEBUFFER, fbo);

            scene.frame_nb += 1;
        }
    }
}
