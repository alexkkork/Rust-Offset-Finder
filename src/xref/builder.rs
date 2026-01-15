// Tue Jan 13 2026 - Alex

use crate::memory::Address;
use crate::xref::{CallGraph, GraphNode, GraphEdge, NodeKind, EdgeKind};

pub struct CallGraphBuilder {
    graph: CallGraph,
}

impl CallGraphBuilder {
    pub fn new() -> Self {
        Self {
            graph: CallGraph::new(),
        }
    }

    pub fn add_function(mut self, address: Address, name: Option<String>) -> Self {
        let node = GraphNode::new(address, NodeKind::Function).with_name(name.unwrap_or_default());
        self.graph.add_node(node);
        self
    }

    pub fn add_call(mut self, from: Address, to: Address) -> Self {
        let edge = GraphEdge::new(from, to, EdgeKind::Call);
        self.graph.add_edge(edge);
        self
    }

    pub fn build(self) -> CallGraph {
        self.graph
    }
}

impl Default for CallGraphBuilder {
    fn default() -> Self {
        Self::new()
    }
}
