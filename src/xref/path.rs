// Tue Jan 13 2026 - Alex

use crate::memory::Address;
use crate::xref::CallGraph;
use std::collections::{HashSet, VecDeque};

pub struct XRefPath {
    nodes: Vec<Address>,
}

impl XRefPath {
    pub fn new(nodes: Vec<Address>) -> Self {
        Self { nodes }
    }

    pub fn nodes(&self) -> &[Address] {
        &self.nodes
    }

    pub fn start(&self) -> Option<Address> {
        self.nodes.first().copied()
    }

    pub fn end(&self) -> Option<Address> {
        self.nodes.last().copied()
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }
}

pub fn find_path(graph: &CallGraph, from: Address, to: Address, max_depth: usize) -> Option<XRefPath> {
    let mut queue = VecDeque::new();
    let mut visited = HashSet::new();
    queue.push_back((from, vec![from]));
    visited.insert(from.as_u64());

    while let Some((current, path)) = queue.pop_front() {
        if path.len() > max_depth {
            continue;
        }
        if current == to {
            return Some(XRefPath::new(path));
        }
        for edge in graph.get_outgoing(current) {
            let next = edge.to();
            if !visited.contains(&next.as_u64()) {
                visited.insert(next.as_u64());
                let mut new_path = path.clone();
                new_path.push(next);
                queue.push_back((next, new_path));
            }
        }
    }
    None
}
