pub mod shader_loader;

use std::collections::HashMap;
use std::fmt;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::SystemTime;

use gl;

#[derive(Debug)]
pub enum ShaderType {
    VERTEX,
    FRAGMENT,
    GEOMETRY,
    COMPUTE,
}

impl fmt::Display for ShaderType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let strext = match self {
            &ShaderType::VERTEX => "VERTEX",
            &ShaderType::FRAGMENT => "FRAGMENT",
            &ShaderType::GEOMETRY => "GEOMETRY",
            &ShaderType::COMPUTE => "COMPUTE",
        };
        write!(f, "{}", strext)
    }
}

#[derive(Debug)]
pub struct Shader {
    pub addr: u32,
    pub path: String,
    pub uniforms: Vec<String>,
    pub shader_type: ShaderType,
    pub last_modified: SystemTime,
}

impl fmt::Display for Shader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} ({})", self.path, self.shader_type)
    }
}

#[derive(Debug)]
pub struct Program {
    pub addr: u32,
    pub shaders: Vec<Arc<Mutex<Shader>>>,
    pub uniforms_location: HashMap<String, i32>,
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.addr);
        }
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.addr);
        }
    }
}

pub fn get_shader_type(path: &Path) -> Option<ShaderType> {
    let ext = path.extension().and_then(|extension| extension.to_str());
    match ext {
        Some("vs") => Some(ShaderType::VERTEX),
        Some("fs") => Some(ShaderType::FRAGMENT),
        Some("gs") => Some(ShaderType::GEOMETRY),
        Some("cs") => Some(ShaderType::COMPUTE),
        _ => None,
    }
}

pub fn get_gl_shader_type(shader_type: &ShaderType) -> Option<u32> {
    match shader_type {
        ShaderType::VERTEX => Some(gl::VERTEX_SHADER),
        ShaderType::FRAGMENT => Some(gl::FRAGMENT_SHADER),
        ShaderType::GEOMETRY => Some(gl::GEOMETRY_SHADER),
        ShaderType::COMPUTE => Some(gl::COMPUTE_SHADER),
    }
}
