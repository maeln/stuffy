use crate::mesh::Mesh;
use crate::shaders::Program;

use cgmath::Matrix4;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter, Result};
use std::rc::Rc;

#[derive(Debug)]
struct DrawableObject {
    pub program: Rc<Program>,
    pub mesh: Rc<Mesh>,
}

impl DrawableObject {
    pub fn draw() {}
}

pub enum NodeType {
    MeshNode(Rc<Mesh>),
    TransformNode(Matrix4<f32>),
}
