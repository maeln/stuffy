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

pub struct Graph<T> {
    counter: usize,
    nodes: HashMap<usize, T>,
    dir: HashMap<usize, Vec<usize>>,
}

impl<T> Graph<T> {
    pub fn new() -> Graph<T> {
        Graph {
            counter: 0,
            nodes: HashMap::new(),
            dir: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, node: T) -> usize {
        self.nodes.insert(self.counter, node);
        let c = self.counter;
        self.counter += 1;
        return c;
    }

    pub fn get_node(&self, id: &usize) -> Option<&T> {
        self.nodes.get(&id)
    }

    pub fn get_node_child(&self, id: &usize) -> Vec<&T> {
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

    pub fn add_child(&mut self, parent: &usize, child: &usize) {
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

    pub fn rm_child(&mut self, parent: &usize, child: &usize) {
        if let Some(dir) = self.dir.get_mut(parent) {
            match dir.binary_search(child) {
                Ok(id) => dir.remove(id),
                Err(_) => 0,
            };
        }
    }

    pub fn rm_node(&mut self, nid: &usize) {
        if self.nodes.get(nid).is_some() {
            self.nodes.remove(nid);
            self.dir.remove(nid);

            for (_, childs) in self.dir.iter_mut() {
                match childs.binary_search(nid) {
                    Ok(id) => childs.remove(id),
                    Err(_) => 0,
                };
            }
        }
    }
}

impl<T: Debug> Debug for Graph<T> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "digraph {{\n")?;
        for (_, v) in self.nodes.iter() {
            write!(f, "\t{:?};\n", v)?;
        }

        for (parent, childs) in self.dir.iter() {
            for child in childs {
                write!(
                    f,
                    "\t{:?} -> {:?};\n",
                    self.get_node(&parent).unwrap(),
                    self.get_node(&child).unwrap()
                )?;
            }
        }
        write!(f, "}}")
    }
}
