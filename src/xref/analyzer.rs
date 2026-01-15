// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryReader};
use crate::xref::{CallGraph, XRef, XRefError, GraphNode, GraphEdge, NodeKind, EdgeKind};
use std::sync::Arc;

pub struct XRefAnalyzer {
    graph: CallGraph,
    reader: Arc<dyn MemoryReader>,
}

impl XRefAnalyzer {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self {
            graph: CallGraph::new(),
            reader,
        }
    }

    pub fn analyze(&mut self, start: Address, end: Address) -> Result<(), XRefError> {
        let size = (end.as_u64() - start.as_u64()) as usize;
        let data = self.reader.read_bytes(start, size)
            .map_err(|e| XRefError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;
        for i in 0..data.len().saturating_sub(4) {
            let addr = start + i as u64;
            if let Ok(value) = self.reader.read_u32(addr) {
                let target = Address::new(value as u64);
                let edge = GraphEdge::new(addr, target, EdgeKind::Call);
                self.graph.add_edge(edge);
            }
        }
        Ok(())
    }

    pub fn graph(&self) -> &CallGraph {
        &self.graph
    }

    pub fn add_node(&mut self, node: GraphNode) {
        self.graph.add_node(node);
    }

    pub fn add_edge(&mut self, edge: GraphEdge) {
        self.graph.add_edge(edge);
    }
}
