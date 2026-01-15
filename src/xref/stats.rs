// Tue Jan 13 2026 - Alex

use crate::xref::CallGraph;

pub struct XRefStats {
    node_count: usize,
    edge_count: usize,
    function_count: usize,
    data_count: usize,
}

impl XRefStats {
    pub fn from_graph(graph: &CallGraph) -> Self {
        let node_count = graph.len();
        let edge_count = graph.edge_count();
        let function_count = graph.nodes().filter(|n| n.is_function()).count();
        let data_count = node_count - function_count;
        Self {
            node_count,
            edge_count,
            function_count,
            data_count,
        }
    }

    pub fn node_count(&self) -> usize {
        self.node_count
    }

    pub fn edge_count(&self) -> usize {
        self.edge_count
    }

    pub fn function_count(&self) -> usize {
        self.function_count
    }

    pub fn data_count(&self) -> usize {
        self.data_count
    }
}
