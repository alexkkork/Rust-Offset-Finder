// Tue Jan 13 2026 - Alex

use crate::memory::Address;
use crate::xref::{CallGraph, GraphNode};

pub struct XRefFilter {
    min_calls: usize,
    max_calls: usize,
    include_kinds: Vec<String>,
}

impl XRefFilter {
    pub fn new() -> Self {
        Self {
            min_calls: 0,
            max_calls: usize::MAX,
            include_kinds: Vec::new(),
        }
    }

    pub fn with_min_calls(mut self, min: usize) -> Self {
        self.min_calls = min;
        self
    }

    pub fn with_max_calls(mut self, max: usize) -> Self {
        self.max_calls = max;
        self
    }

    pub fn with_kinds(mut self, kinds: Vec<String>) -> Self {
        self.include_kinds = kinds;
        self
    }

    pub fn filter_nodes<'a>(&self, graph: &'a CallGraph) -> Vec<&'a GraphNode> {
        let min_calls = self.min_calls;
        let max_calls = self.max_calls;
        graph.nodes()
            .filter(|node| {
                let call_count = graph.get_incoming(node.address()).len();
                call_count >= min_calls && call_count <= max_calls
            })
            .collect()
    }

    pub fn filter_by_call_count<'a>(&self, graph: &'a CallGraph, min: usize, max: usize) -> Vec<&'a GraphNode> {
        graph.nodes()
            .filter(|node| {
                let call_count = graph.get_incoming(node.address()).len();
                call_count >= min && call_count <= max
            })
            .collect()
    }

    pub fn filter_highly_referenced<'a>(&self, graph: &'a CallGraph, threshold: usize) -> Vec<&'a GraphNode> {
        graph.nodes()
            .filter(|node| graph.get_incoming(node.address()).len() >= threshold)
            .collect()
    }

    pub fn filter_entry_points<'a>(&self, graph: &'a CallGraph) -> Vec<&'a GraphNode> {
        graph.nodes()
            .filter(|node| graph.get_incoming(node.address()).is_empty())
            .collect()
    }

    pub fn filter_leaf_nodes<'a>(&self, graph: &'a CallGraph) -> Vec<&'a GraphNode> {
        graph.nodes()
            .filter(|node| graph.get_outgoing(node.address()).is_empty())
            .collect()
    }

    pub fn get_call_statistics(&self, graph: &CallGraph) -> CallStatistics {
        let mut total_calls = 0;
        let mut max_incoming = 0;
        let mut max_outgoing = 0;
        let mut node_count = 0;

        for node in graph.nodes() {
            let incoming = graph.get_incoming(node.address()).len();
            let outgoing = graph.get_outgoing(node.address()).len();
            total_calls += outgoing;
            max_incoming = max_incoming.max(incoming);
            max_outgoing = max_outgoing.max(outgoing);
            node_count += 1;
        }

        CallStatistics {
            total_nodes: node_count,
            total_edges: total_calls,
            max_incoming_calls: max_incoming,
            max_outgoing_calls: max_outgoing,
            avg_calls_per_node: if node_count > 0 { total_calls as f64 / node_count as f64 } else { 0.0 },
        }
    }
}

impl Default for XRefFilter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct CallStatistics {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub max_incoming_calls: usize,
    pub max_outgoing_calls: usize,
    pub avg_calls_per_node: f64,
}

pub fn find_common_targets(graph: &CallGraph, sources: &[Address]) -> Vec<Address> {
    if sources.is_empty() {
        return Vec::new();
    }

    let mut target_counts: std::collections::HashMap<u64, usize> = std::collections::HashMap::new();
    for source in sources {
        for edge in graph.get_outgoing(*source) {
            *target_counts.entry(edge.to().as_u64()).or_insert(0) += 1;
        }
    }

    target_counts
        .iter()
        .filter(|(_, &count)| count == sources.len())
        .map(|(&addr, _)| Address::new(addr))
        .collect()
}

pub fn find_hub_nodes(graph: &CallGraph, min_connections: usize) -> Vec<Address> {
    graph.nodes()
        .filter(|node| {
            let total = graph.get_incoming(node.address()).len() + graph.get_outgoing(node.address()).len();
            total >= min_connections
        })
        .map(|node| node.address())
        .collect()
}
