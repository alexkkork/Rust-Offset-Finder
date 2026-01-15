// Wed Jan 15 2026 - Alex

use crate::memory::Address;
use crate::xref::{GraphNode, GraphEdge, XRef, XRefKind};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct CallGraph {
    nodes: HashMap<u64, GraphNode>,
    edges: Vec<GraphEdge>,
    outgoing: HashMap<u64, Vec<usize>>,
    incoming: HashMap<u64, Vec<usize>>,
}

impl CallGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
            outgoing: HashMap::new(),
            incoming: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, node: GraphNode) {
        self.nodes.insert(node.address().as_u64(), node);
    }

    pub fn add_edge(&mut self, edge: GraphEdge) {
        let edge_idx = self.edges.len();
        let from_addr = edge.from().as_u64();
        let to_addr = edge.to().as_u64();
        self.edges.push(edge);
        self.outgoing.entry(from_addr).or_insert_with(Vec::new).push(edge_idx);
        self.incoming.entry(to_addr).or_insert_with(Vec::new).push(edge_idx);
    }

    pub fn get_node(&self, address: Address) -> Option<&GraphNode> {
        self.nodes.get(&address.as_u64())
    }

    pub fn get_outgoing(&self, address: Address) -> Vec<&GraphEdge> {
        self.outgoing.get(&address.as_u64())
            .map(|indices| indices.iter().map(|&i| &self.edges[i]).collect())
            .unwrap_or_default()
    }

    pub fn get_incoming(&self, address: Address) -> Vec<&GraphEdge> {
        self.incoming.get(&address.as_u64())
            .map(|indices| indices.iter().map(|&i| &self.edges[i]).collect())
            .unwrap_or_default()
    }

    pub fn nodes(&self) -> impl Iterator<Item = &GraphNode> {
        self.nodes.values()
    }

    pub fn edges(&self) -> impl Iterator<Item = &GraphEdge> {
        self.edges.iter()
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    pub fn get_references_to(&self, target: Address) -> Vec<XRef> {
        self.incoming.get(&target.as_u64())
            .map(|indices| {
                indices.iter()
                    .map(|&i| {
                        let edge = &self.edges[i];
                        XRef::new(edge.from(), edge.to(), XRefKind::Call)
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn get_references_from(&self, source: Address) -> Vec<XRef> {
        self.outgoing.get(&source.as_u64())
            .map(|indices| {
                indices.iter()
                    .map(|&i| {
                        let edge = &self.edges[i];
                        XRef::new(edge.from(), edge.to(), XRefKind::Call)
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}

impl Default for CallGraph {
    fn default() -> Self {
        Self::new()
    }
}
