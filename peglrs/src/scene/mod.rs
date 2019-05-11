use crate::mesh::Mesh;
use crate::shaders::Program;

use cgmath::Matrix4;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug)]
struct DrawableObject {
    pub program: Rc<Program>,
    pub mesh: Rc<Mesh>,
}

impl DrawableObject {
    pub fn draw() {}
}

enum NodeType {
    MeshNode(Rc<Mesh>),
    TransformNode(Matrix4<f32>),
}

struct Graph<T> {
    counter: usize,
    nodes: HashMap<usize, T>,
    dir: HashMap<usize, Vec<usize>>,
}

impl<T> Graph<T> {
    fn new() -> Graph<T> {
        Graph {
            counter: 0,
            nodes: HashMap::new(),
            dir: HashMap::new(),
        }
    }

    fn add_node(&mut self, node: T) -> usize {
        self.nodes.insert(self.counter, node);
        let c = self.counter;
        self.counter += 1;
        return c;
    }

    fn get_node(&self, id: &usize) -> Option<&T> {
        self.nodes.get(&id)
    }

    fn get_node_child(&self, id: usize) -> Vec<&T> {
        let mut childs: Vec<&T> = Vec::new();
        let neighbors = self.dir.get(&id);
        if neighbors.map_or(true, |n| n.is_empty()) {
            return childs;
        }

        let neighbors = neighbors.unwrap();
        for neighbor in neighbors {
            if let Some(node) = self.get_node(neighbor) {
                childs.push(node);
            }
        }
        return childs;
    }
}
