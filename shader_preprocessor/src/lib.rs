extern crate mini_graph;

use mini_graph::Graph;

use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::SystemTime;
use std::vec::Vec;

pub struct ShaderFile {
    path: PathBuf,
    source: String,
    lm_time: SystemTime,
}

impl PartialEq for ShaderFile {
    // For our usage, we just want shaders sources to be considered equal when they have the same path.
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl Eq for ShaderFile {}

pub struct ShaderDB {
    base_path: PathBuf,
    shaders: Graph<ShaderFile>,
}

impl ShaderDB {
    pub fn new(base_path: &Path) -> ShaderDB {
        ShaderDB {
            base_path: base_path.to_path_buf(),
            shaders: Graph::new(),
        }
    }

    pub fn find_shader(&self, shader: &ShaderFile) -> Option<(&usize, &ShaderFile)> {
        for (id, node) in self.shaders.iter_node() {
            if node == shader {
                return Some((id, node));
            }
        }

        None
    }

    pub fn find_shader_by_path(&self, path: &Path) -> Option<(&usize, &ShaderFile)> {
        for (id, node) in self.shaders.iter_node() {
            if node.path == path {
                return Some((id, node));
            }
        }

        None
    }

    pub fn find_shader_by_path_mut(&mut self, path: &Path) -> Option<(&usize, &mut ShaderFile)> {
        for (id, node) in self.shaders.iter_mut_node() {
            if node.path == path {
                return Some((id, node));
            }
        }

        None
    }

    pub fn load_shader(&mut self, path: &Path) -> Option<usize> {
        let previous = self.find_shader_by_path_mut(path);
        // If we already loaded the shader, we reload it.
        if previous.is_some() {
            let shdr = previous.unwrap();
            let id = shdr.0.clone();
            let mut obj = shdr.1;
            let new_time = get_lm_time(path).unwrap();
            // Avoid reloading the sources if the file date didn't change.
            if new_time > obj.lm_time {
                obj.source = read_file(path).unwrap();
                obj.lm_time = get_lm_time(path).unwrap();
            }
            return Some(id);
        }

        let src = read_file(path);
        if src.is_none() {
            return None;
        }

        let lm_time = get_lm_time(path);
        if lm_time.is_none() {
            return None;
        }

        Some(self.shaders.add_node(ShaderFile {
            path: path.to_path_buf(),
            source: src.unwrap(),
            lm_time: lm_time.unwrap(),
        }))
    }

    pub fn process_shader_deps(&mut self, shader_id: usize) {
        let shader = self.shaders.get_node(&shader_id).unwrap();
        let includes = find_include(&shader.source);
        for include in includes {
            let shader_include = self.find_shader_by_path(&PathBuf::from(include.clone()));
            if shader_include.is_some() {
                let id = shader_include.unwrap().0.clone();
                self.shaders.add_child(&shader_id, &id);
            } else {
                let id = self.load_shader(&PathBuf::from(include.clone())).unwrap();
                self.shaders.add_child(&shader_id, &id);
            }
        }
    }

    pub fn compile_shader(&self, shader_id: usize) -> Option<String> {
        let shader = self.shaders.get_node(&shader_id).unwrap();
        let mut final_src = String::new();

        for line in shader.source.lines() {
            let finc = get_include(line);
            if finc.is_some() {
                let path = finc.unwrap();
                let deps = self.find_shader_by_path(&PathBuf::from(path));
                let deps_id = deps.unwrap().0.clone();
                let finc_src = self.compile_shader(deps_id).unwrap();
                final_src.push_str(&finc_src);
            }
            else {
                final_src.push_str(line);
            }
        }

        Some(final_src)
    }
}

pub struct ShaderSource {
    path: String,
    source: String,
    includes: Vec<String>,
    uniforms: Vec<String>,
}

pub fn get_lm_time(path: &Path) -> Option<std::time::SystemTime> {
    let stat = fs::metadata(path).unwrap();
    stat.modified().ok()
}

pub fn read_file(path: &Path) -> Option<String> {
    let res_file = File::open(path);
    if res_file.is_err() {
        return None;
    }

    let mut file = res_file.unwrap();
    let mut src = String::new();
    let res_read = file.read_to_string(&mut src);
    if res_read.is_err() {
        return None;
    }

    Some(src)
}

pub fn get_include(line: &str) -> Option<String> {
    if line.starts_with("#include") {
        let fname_slice = line[8..].trim();
        let mut fname = String::new();
        let mut entered = false;
        for c in fname_slice.chars() {
            if entered {
                if c == '"' || c == '\'' {
                    break;
                }
                fname.push(c);
            } else {
                if c == '"' || c == '\'' {
                    entered = true;
                }
            }
        }

        return Some(fname);
    }
    None
}

pub fn find_include(sources: &str) -> Vec<String> {
    let mut files_names: Vec<String> = Vec::new();
    for line in sources.lines() {
        let include_file = get_include(line);
        if include_file.is_some() {
            files_names.push(include_file.unwrap());
        }
    }

    files_names
}

pub fn find_uniforms(src: &str) -> Vec<String> {
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

pub fn parse_shader(path: &Path) -> Option<ShaderSource> {
    let sources = read_file(path);
    if sources.is_some() {
        let src = sources.unwrap();
        let includes = find_include(&src);
        let uniforms = find_uniforms(&src);

        let full_path = String::from(path.canonicalize().unwrap().to_str().unwrap());

        return Some(ShaderSource {
            path: full_path,
            source: src,
            includes,
            uniforms,
        });
    }

    None
}

pub fn build_dependency_graph<'a>(shdr: &'a ShaderSource) -> Graph<&'a ShaderSource> {
    let mut deps: Graph<&ShaderSource> = Graph::new();
    if shdr.includes.is_empty() {
        return deps;
    }

    let shdr_id = deps.add_node(&shdr);
    let base_path = Path::new(&shdr.path).parent().unwrap();
    for dependency in &shdr.includes {
        let mut dpath = PathBuf::from(dependency);
        if !dpath.has_root() {
            dpath = base_path.with_file_name(dpath);
        }

        let shdr_deps = parse_shader(&dpath).unwrap();
    }

    deps
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn can_parse_include() {
        let src = "test;\n#define LOL;\n#include \"test.h\";\nabcd defg";
        let v = find_include(&src);
        assert_eq!(v, vec!["test.h"]);
    }
}
