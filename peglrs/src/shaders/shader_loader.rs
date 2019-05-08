use crate::utils;
use gl;
use std::path::Path;

use std::collections::HashMap;
use std::ffi::CString;
use std::fs;
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use cgmath::{Array, Matrix, Matrix4, Vector2, Vector3, Vector4};

use super::*;

#[derive(Debug)]
pub struct ProgramDB {
    programs: HashMap<u32, Arc<Mutex<Program>>>,
    counter: u32,
}

impl ProgramDB {
    pub fn new() -> ProgramDB {
        ProgramDB {
            programs: HashMap::new(),
            counter: 0,
        }
    }

    pub fn add(&mut self, program: Arc<Mutex<Program>>) -> u32 {
        let id = self.counter;
        self.programs.insert(id, program);
        self.counter += 1;
        id
    }

    pub fn rm(&mut self, id: u32) -> Option<Arc<Mutex<Program>>> {
        self.programs.remove(&id)
    }
}

#[derive(Debug)]
pub struct ShaderManager {
    pub db: Arc<Mutex<ProgramDB>>,
    pub watcher: thread::JoinHandle<()>,
    pub sender: Sender<Arc<Mutex<Program>>>,
    pub receiver: Receiver<Arc<Mutex<Program>>>,
}

impl ShaderManager {
    fn should_reload_shader(shader: &Arc<Mutex<Shader>>) -> bool {
        let shad = shader.lock().unwrap();
        let last_modifed = shad.last_modified;
        let path = Path::new(&shad.path);
        let stat = fs::metadata(path).unwrap();
        let new_modified = stat.modified().unwrap();
        new_modified > last_modifed
    }

    fn check_program_for_reload(program: &Arc<Mutex<Program>>) -> bool {
        let prog_borrow = program.lock().unwrap();
        let shaders = &prog_borrow.shaders;
        for shader in shaders {
            if ShaderManager::should_reload_shader(shader) {
                // println!("Need reloading: {}", shader.path);
                return true;
            }
        }
        false
    }

    fn flag_program_for_reload(
        program_db: &Arc<Mutex<ProgramDB>>,
        sender: &Sender<Arc<Mutex<Program>>>,
    ) {
        let db_borrow = program_db.lock().unwrap();
        let program_borrow = &db_borrow.programs;
        for (id, program) in program_borrow.iter() {
            if ShaderManager::check_program_for_reload(program) {
                sender.send(program.clone()).unwrap();
            }
        }
    }

    pub fn new() -> ShaderManager {
        let db: Arc<Mutex<ProgramDB>> = Arc::new(Mutex::new(ProgramDB::new()));
        let (sender, receiver) = mpsc::channel();

        let db_clone = db.clone();
        let thread_sender = mpsc::Sender::clone(&sender);
        let watcher = thread::spawn(move || loop {
            ShaderManager::flag_program_for_reload(&db_clone, &thread_sender);
            thread::sleep(Duration::from_millis(1000));
        });

        ShaderManager {
            db,
            watcher,
            receiver,
            sender,
        }
    }

    pub fn handle_reload(&mut self) {
        let mut reloaded_ids: Vec<u32> = Vec::new();
        let mut flagged = Vec::new();

        // In some cases, we can receive several time the same program.
        // So we delete copycat first.
        for program in self.receiver.try_iter() {
            let addr = program.lock().unwrap().addr;
            println!("r {}:{:?}", addr, reloaded_ids);
            if(reloaded_ids.binary_search(&addr).is_err()) {
                reloaded_ids.push(addr);
                flagged.push(program);
            }
        }

        for program in flagged {
            let mut prog_borrow = program.lock().unwrap();
            let shaders = &prog_borrow.shaders;
            for shader in shaders {
                println!("Reloading shader: {}", shader.lock().unwrap());
            }
            prog_borrow.reload();
        }
    }

    pub fn load_program(&mut self, shaders_path: &Vec<&Path>) -> u32 {
        let mut shaders: Vec<Arc<Mutex<Shader>>> = Vec::with_capacity(shaders_path.len());
        for shader_path in shaders_path {
            let shd = Shader::load_shader(shader_path).unwrap();
            shaders.push(Arc::new(Mutex::new(shd)));
        }

        let program = Program::load_program(&shaders).unwrap();
        let mut db = self.db.lock().unwrap();
        db.add(Arc::new(Mutex::new(program)))
    }

    pub fn rm_program(&mut self, id: u32) -> Option<Arc<Mutex<Program>>> {
        let mut db = self.db.lock().unwrap();
        db.rm(id)
    }

    pub fn get_program(&self, id: u32) -> Option<Arc<Mutex<Program>>> {
        let mut db = self.db.lock().unwrap();
        let res = db.programs.get(&id);
        if let Some(prog) = res {
            return Some(prog.clone());
        }
        None
    }
}

impl Shader {
    fn parse_uniforms(src: &str) -> Vec<String> {
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

    fn compile_shader(path: &Path, source: &str, shader_type: &ShaderType) -> Option<u32> {
        let c_source = CString::new(source.as_bytes()).ok();
        if c_source.is_none() {
            #[cfg(feature = "debug")]
            eprintln!("[ERR] Couldn't load source for shader {}", path.display());

            return None;
        }

        let c_source = c_source.unwrap();
        let gl_type = get_gl_shader_type(&shader_type);

        unsafe {
            let addr = gl::CreateShader(gl_type.unwrap());
            gl::ShaderSource(addr, 1, &c_source.as_ptr(), ptr::null());
            gl::CompileShader(addr);

            let mut status: i32 = 0;
            gl::GetShaderiv(addr, gl::COMPILE_STATUS, &mut status);
            if status == i32::from(gl::FALSE) {
                let mut log_len: i32 = 0;
                gl::GetShaderiv(addr, gl::INFO_LOG_LENGTH, &mut log_len);
                let mut log: Vec<u8> = Vec::with_capacity(log_len as usize);
                gl::GetShaderInfoLog(addr, log_len, ptr::null_mut(), log.as_mut_ptr() as *mut i8);
                log.set_len(log_len as usize);
                eprintln!(
                    "[ERR] Couldn't compile shader {}, log:\n{}",
                    path.display(),
                    String::from_utf8_lossy(&log[..])
                );

                return None;
            }
            return Some(addr);
        }
    }

    pub fn load_shader(path: &Path) -> Option<Shader> {
        #[cfg(feature = "debug")]
        println!("[NFO] Loading shader {}", path.display());

        let src = utils::load_file(path);
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

        let shader_type = shader_type.unwrap();
        let addr = Shader::compile_shader(path, &src, &shader_type);
        if addr.is_none() {
            return None;
        }

        let stat = fs::metadata(path).unwrap();
        Some(Shader {
            addr: addr.unwrap(),
            path: String::from(path.to_str().unwrap()),
            uniforms: Shader::parse_uniforms(&src),
            shader_type: shader_type,
            last_modified: stat.modified().unwrap(),
        })
    }

    pub fn reload(&mut self) {
        let path = Path::new(&self.path);
        let src = utils::load_file(path);
        if src.is_none() {
            #[cfg(feature = "debug")]
            eprintln!("[ERR] Couldn't load source for shader {}", path.display());
            return;
        }

        let src = src.unwrap();
        let new_addr = Shader::compile_shader(path, &src, &self.shader_type);
        if let Some(addr) = new_addr {
            let stat = fs::metadata(path).unwrap();
            unsafe {
                gl::DeleteShader(self.addr);
                self.addr = addr;
                self.last_modified = stat.modified().unwrap();
            }
        } else {
            eprintln!("[ERR] Couldn't reload shader {}", path.display());
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

    pub fn set_i32(&self, name: &str, value: i32) {
        unsafe {
            gl::Uniform1i(self.uniforms_location[name], value);
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

    pub fn link_program(shaders: &Vec<Arc<Mutex<Shader>>>) -> Option<u32> {
        unsafe {
            let addr = gl::CreateProgram();
            for shader in shaders {
                let shad = shader.lock().unwrap();
                gl::AttachShader(addr, shad.addr);
            }
            gl::LinkProgram(addr);

            let mut status: i32 = 0;
            gl::GetProgramiv(addr, gl::LINK_STATUS, &mut status);
            if status == i32::from(gl::FALSE) {
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
                return None;
            }

            for shader in shaders.into_iter() {
                let shad = shader.lock().unwrap();
                gl::DetachShader(addr, shad.addr);
                for uniform in &shad.uniforms {
                    let uniform_cstr = CString::new(uniform.as_bytes()).unwrap();
                    let location = gl::GetUniformLocation(addr, uniform_cstr.as_ptr());
                }
            }

            return Some(addr);
        }
    }

    pub fn load_program(shaders: &Vec<Arc<Mutex<Shader>>>) -> Option<Program> {
        let program_addr = Program::link_program(shaders);
        if let Some(addr) = program_addr {
            let mut program = Program {
                addr,
                shaders: Vec::with_capacity(shaders.len()),
                uniforms_location: HashMap::new(),
            };

            for shader in shaders.into_iter() {
                program.shaders.push(shader.clone());
                let shad = shader.lock().unwrap();
                for uniform in &shad.uniforms {
                    let uniform_cstr = CString::new(uniform.as_bytes()).unwrap();
                    unsafe {
                        let location = gl::GetUniformLocation(addr, uniform_cstr.as_ptr());
                        program.uniforms_location.insert(uniform.clone(), location);
                    }
                }
            }

            return Some(program);
        }

        None
    }

    pub fn reload(&mut self) {
        for shader in &self.shaders {
            let mut shad = shader.lock().unwrap();
            shad.reload();
        }

        let program_addr = Program::link_program(&self.shaders);
        if let Some(addr) = program_addr {
            unsafe {
                gl::DeleteProgram(self.addr);
            }
            self.addr = addr;

            self.uniforms_location.clear();
            for shader in self.shaders.iter() {
                let shad = shader.lock().unwrap();
                for uniform in &shad.uniforms {
                    let uniform_cstr = CString::new(uniform.as_bytes()).unwrap();
                    unsafe {
                        let location = gl::GetUniformLocation(addr, uniform_cstr.as_ptr());
                        self.uniforms_location.insert(uniform.clone(), location);
                    }
                }
            }
        }
    }
}
