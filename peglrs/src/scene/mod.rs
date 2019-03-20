use crate::mesh::Mesh;
use crate::shaders::Program;

use std::rc::Rc;

#[derive(Debug)]
struct DrawableObject {
    pub program: Rc<Program>,
    pub mesh: Rc<Mesh>,
}

impl DrawableObject {
    pub fn draw() {}
}
