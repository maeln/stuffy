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

    fn get_node_child(&self, id: &usize) -> Vec<&T> {
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

    fn add_child(&mut self, parent: &usize, child: &usize) {
        if !self.dir.contains_key(parent) {
            self.dir.insert(*parent, vec![*child]);
        } else {
            let mut children = self.dir.get_mut(parent).unwrap();
            match children.binary_search(child) {
                Ok(_) => (),
                Err(_) => children.push(*child),
            };
        }
    }

    fn rm_child(&mut self, parent: &usize, child: &usize) {
        if let Some(dir) = self.dir.get_mut(parent) {
            match dir.binary_search(child) {
                Ok(id) => dir.remove(id),
                Err(_) => 0,
            };
        }
    }
}
