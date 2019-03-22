use crate::utils;
use gl;
use std::path::Path;

use std::ffi::CString;
use std::fs;
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use cgmath::{Array, Matrix, Matrix4, Vector2, Vector3, Vector4};

use super::*;

#[derive(Debug)]
pub struct ShaderManager {
    pub programs: Arc<Vec<Arc<Program>>>,
    pub flagged_for_reload: Arc<Mutex<Vec<Arc<Program>>>>,
    pub watcher: thread::JoinHandle<()>,
    watcher_running: Arc<AtomicBool>,
}

fn should_reload_shader(shader: &Arc<Shader>) -> bool {
    let last_modifed = shader.last_modified;
    let path = Path::new(&shader.path);
    let stat = fs::metadata(path).unwrap();
    let new_modified = stat.modified().unwrap();
    new_modified > last_modifed
}

fn check_program_for_reload(program: &Arc<Program>) -> bool {
    let shaders = &program.shaders;
    for shader in shaders {
        if should_reload_shader(shader) {
            return true;
        }
    }
    false
}

fn flag_program_for_reload(
    programs: &Arc<Vec<Arc<Program>>>,
    flagged_for_reload: &Arc<Mutex<Vec<Arc<Program>>>>,
) {
    for program in programs.iter() {
        if check_program_for_reload(program) {
            let mut flag_borrow = flagged_for_reload.lock().unwrap();
            flag_borrow.push(program.clone());
        }
    }
}

impl ShaderManager {
    fn new() -> ShaderManager {
        let programs: Arc<Vec<Arc<Program>>> = Arc::new(Vec::new());
        let flagged_for_reload: Arc<Mutex<Vec<Arc<Program>>>> = Arc::new(Mutex::new(Vec::new()));
        let watcher_running: Arc<AtomicBool> = Arc::new(AtomicBool::new(true));

        let programs_clone = programs.clone();
        let flag_clone = flagged_for_reload.clone();
        let running = watcher_running.clone();
        let watcher = thread::spawn(move || {
            while running.load(Ordering::Relaxed) {
                flag_program_for_reload(&programs_clone, &flag_clone);
                thread::sleep(Duration::from_millis(1000));
            }
        });

        ShaderManager {
            programs,
            flagged_for_reload,
            watcher,
            watcher_running,
        }
    }

    fn kill_watcher(self) {
        self.watcher_running.store(false, Ordering::Relaxed);
    }

    fn spawn_new_watcher(&mut self) {
        self.watcher_running.store(true, Ordering::Relaxed);

        let running = self.watcher_running.clone();
        let programs_clone = self.programs.clone();
        let flag_clone = self.flagged_for_reload.clone();
        self.watcher = thread::spawn(move || {
            while running.load(Ordering::Relaxed) {
                flag_program_for_reload(&programs_clone, &flag_clone);
                thread::sleep(Duration::from_millis(1000));
            }
        });
    }
}

pub fn parse_uniforms(src: &str) -> Vec<String> {
    let mut uniforms: Vec<String> = Vec::new();
    for line in src.lines() {
        if line.starts_with("uniform ") {
            let attrb: Vec<&str> = line.split(' ').collect();
            if attrb.len() < 3 {
                continue;
            }

            let uniform_name = String::from(attrb[2]).trim_end_matches(';').to_string();
            uniforms.push(uniform_name);
        }
    }

    uniforms
}

impl Shader {
    pub fn load_shader(path: &Path) -> Option<Shader> {
        #[cfg(feature = "debug")]
        println!("[NFO] Loading shader {}", path.display());

        let src = utils::load_file(path).and_then(|shd_src| CString::new(shd_src.as_bytes()).ok());
        if src.is_none() {
            #[cfg(feature = "debug")]
            eprintln!("[ERR] Couldn't load source for shader {}", path.display());

            return None;
        }
        let src = src.unwrap();

        let shader_type = get_shader_type(path);
        if shader_type.is_none() {
            #[cfg(feature = "debug")]
            eprintln!("[ERR] Couldn't detect shader type for {}", path.display());

            return None;
        }

        let gl_type = get_gl_shader_type(&shader_type);
        unsafe {
            let addr = gl::CreateShader(gl_type.unwrap());
            gl::ShaderSource(addr, 1, &src.as_ptr(), ptr::null());
            // println!("src: {:?}", src);
            gl::CompileShader(addr);

            let mut status: i32 = 0;
            gl::GetShaderiv(addr, gl::COMPILE_STATUS, &mut status);
            if status == i32::from(gl::FALSE) {
                #[cfg(feature = "debug")]
                {
                    let mut log_len: i32 = 0;
                    gl::GetShaderiv(addr, gl::INFO_LOG_LENGTH, &mut log_len);
                    // println!("log len: {}", log_len);
                    let mut log: Vec<u8> = Vec::with_capacity(log_len as usize);
                    gl::GetShaderInfoLog(
                        addr,
                        log_len,
                        ptr::null_mut(),
                        log.as_mut_ptr() as *mut i8,
                    );
                    log.set_len(log_len as usize);
                    println!("log len: {:?}", log);
                    eprintln!(
                        "[ERR] Couldn't compile shader {}, log:\n{}",
                        path.display(),
                        String::from_utf8_lossy(&log[..])
                    );
                }

                return None;
            }

            let stat = fs::metadata(path).unwrap();
            Some(Shader {
                addr,
                path: String::from(path.to_str().unwrap()),
                uniforms: parse_uniforms(&src.to_string_lossy()),
                shader_type: shader_type.unwrap(),
                last_modified: stat.modified().unwrap(),
            })
        }
    }
}

impl Program {
    pub fn bind(&self) {
        unsafe {
            gl::UseProgram(self.addr);
        }
    }

    pub fn unbind() {
        unsafe {
            gl::UseProgram(0);
        }
    }

    pub fn set_float(&self, name: &str, value: f32) {
        unsafe {
            gl::Uniform1f(self.uniforms_location[name], value);
        }
    }

    pub fn set_vec2(&self, name: &str, value: &Vector2<f32>) {
        unsafe {
            gl::Uniform2fv(self.uniforms_location[name], 1, value.as_ptr());
        }
    }

    pub fn set_vec3(&self, name: &str, value: &Vector3<f32>) {
        unsafe {
            gl::Uniform3fv(self.uniforms_location[name], 1, value.as_ptr());
        }
    }

    pub fn set_vec4(&self, name: &str, value: &Vector4<f32>) {
        unsafe {
            gl::Uniform4fv(self.uniforms_location[name], 1, value.as_ptr());
        }
    }

    pub fn set_mat4(&self, name: &str, value: &Matrix4<f32>) {
        unsafe {
            gl::UniformMatrix4fv(self.uniforms_location[name], 1, gl::FALSE, value.as_ptr());
        }
    }

    pub fn load_program(shaders: &Vec<Arc<Shader>>) -> Option<Program> {
        unsafe {
            let addr = gl::CreateProgram();
            for shader in shaders {
                gl::AttachShader(addr, shader.addr);
            }
            gl::LinkProgram(addr);

            let mut status: i32 = 0;
            gl::GetProgramiv(addr, gl::LINK_STATUS, &mut status);
            if status == i32::from(gl::FALSE) {
                #[cfg(feature = "debug")]
                {
                    let mut log_length: i32 = 0;
                    gl::GetProgramiv(addr, gl::INFO_LOG_LENGTH, &mut log_length);
                    let mut log: Vec<u8> = Vec::with_capacity(log_length as usize);
                    gl::GetProgramInfoLog(
                        addr,
                        log_length,
                        ptr::null_mut(),
                        log.as_mut_ptr() as *mut i8,
                    );
                    log.set_len(log_length as usize);
                    eprintln!(
                        "[ERR] Couldn't link program, log:\n{}",
                        String::from_utf8_lossy(&log[..])
                    );
                }

                return None;
            }

            let mut program = Program {
                addr,
                shaders: Vec::with_capacity(shaders.len()),
                uniforms_location: HashMap::new(),
            };

            for shader in shaders.into_iter() {
                gl::DetachShader(addr, shader.addr);
                program.shaders.push(shader.clone());

                for uniform in &shader.uniforms {
                    let uniform_cstr = CString::new(uniform.as_bytes()).unwrap();
                    let location = gl::GetUniformLocation(addr, uniform_cstr.as_ptr());
                    program.uniforms_location.insert(uniform.clone(), location);
                }
            }

            Some(program)
        }
    }
}
