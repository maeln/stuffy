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
use shaders::shader_loader::ShaderManager;
use shaders::{Program, Shader};

use cgmath::prelude::*;
use cgmath::{perspective, Deg, Matrix4, Point3, Vector3};

use std::ffi::CStr;

#[derive(Debug)]
pub struct Scene {
    pub model_mat: Matrix4<f32>,
    pub projection_mat: Matrix4<f32>,
    pub camera: Camera,
    pub shader_manager: ShaderManager,
    pub program: u32,
    pub mesh: mesh::Mesh,
}

static mut m_scene: Option<Scene> = None;

#[no_mangle]
pub fn resize_window(width: f64, height: f64, dpi_ratio: f64) {
    let real_width = dpi_ratio * width;
    let real_height = dpi_ratio * height;

    unsafe {
        gl::Viewport(0, 0, real_width as i32, real_height as i32);

        if let Some(scene) = &mut m_scene {
            scene.projection_mat =
                perspective(Deg(75.0), (real_width / real_height) as f32, 0.1, 10.0);
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
        Path::new("data/shaders/basic/projection.vs"),
        Path::new("data/shaders/basic/phong/phong.fs"),
    ]);

    let mut cube = mesh::Mesh::cube();
    cube.ready_up();

    let mut model = Matrix4::<f32>::identity();
    let mut projection: Matrix4<f32> =
        perspective(Deg(75.0), (true_width / true_height) as f32, 0.1, 10.0);

    let mut cam = Camera::new(
        Point3::new(0.0, 0.0, -2.0),
        Vector3::new(0.0, 0.0, 1.0),
        Vector3::new(0.0, 1.0, 0.0),
    );

    unsafe {
        m_scene = Some(Scene {
            model_mat: model,
            projection_mat: projection,
            camera: cam,
            shader_manager,
            program: program,
            mesh: cube,
        })
    }
}

#[no_mangle]
pub fn handle_mouse(dx: f32, dy: f32, speed: f32) {
    unsafe {
        if let Some(scene) = &mut m_scene {
            scene.camera.move_target(dx, dy, speed);
        }
    }
}

#[no_mangle]
pub fn quit() {
    unsafe {
        m_scene = None;
    }
}

#[no_mangle]
pub fn display_loop(time: f64) {
    unsafe {
        if let Some(scene) = &mut m_scene {
            scene.shader_manager.handle_reload();

            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT);
            gl::ClearColor(0.0, 0.0, 0.0, 0.0);

            if let Some(program) = scene.shader_manager.get_program(scene.program) {
                let prog = program.lock().unwrap();
                prog.bind();
                prog.set_mat4("projection", &scene.projection_mat);
                prog.set_mat4("view", &scene.camera.view());
                prog.set_mat4("model", &scene.model_mat);
                prog.set_vec4("eye_pos", &scene.camera.position.to_homogeneous());
                prog.set_vec4("light_pos", &Point3::new(3.0, 1.0, 1.0).to_homogeneous());
            }

            scene.mesh.draw();
        }
    }
}
