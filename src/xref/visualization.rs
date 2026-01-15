// Tue Jan 15 2026 - Alex

use crate::memory::Address;
use crate::xref::{CallGraph, GraphNode, EdgeKind, NodeKind};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::io::Write;

/// Export format for call graph visualization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    /// GraphViz DOT format
    Dot,
    /// JSON format
    Json,
    /// GraphML format (for yEd, Gephi)
    GraphML,
    /// CSV format (edges list)
    Csv,
    /// D3.js compatible JSON
    D3Json,
    /// Mermaid diagram format
    Mermaid,
}

/// Options for graph export
#[derive(Debug, Clone)]
pub struct ExportOptions {
    /// Whether to include node labels
    pub include_labels: bool,
    /// Whether to include edge labels
    pub include_edge_labels: bool,
    /// Whether to colorize by node type
    pub colorize: bool,
    /// Whether to cluster related nodes
    pub cluster: bool,
    /// Maximum nodes to export (for large graphs)
    pub max_nodes: Option<usize>,
    /// Filter to specific node types
    pub node_filter: Option<HashSet<NodeKind>>,
    /// Custom node colors
    pub custom_colors: HashMap<NodeKind, String>,
    /// Graph title
    pub title: Option<String>,
    /// Direction (TB, LR, BT, RL)
    pub direction: GraphDirection,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            include_labels: true,
            include_edge_labels: true,
            colorize: true,
            cluster: false,
            max_nodes: None,
            node_filter: None,
            custom_colors: HashMap::new(),
            title: None,
            direction: GraphDirection::TopBottom,
        }
    }
}

impl ExportOptions {
    pub fn minimal() -> Self {
        Self {
            include_labels: false,
            include_edge_labels: false,
            colorize: false,
            cluster: false,
            max_nodes: None,
            node_filter: None,
            custom_colors: HashMap::new(),
            title: None,
            direction: GraphDirection::TopBottom,
        }
    }
}

/// Graph layout direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphDirection {
    TopBottom,
    BottomTop,
    LeftRight,
    RightLeft,
}

impl GraphDirection {
    fn to_dot(&self) -> &'static str {
        match self {
            GraphDirection::TopBottom => "TB",
            GraphDirection::BottomTop => "BT",
            GraphDirection::LeftRight => "LR",
            GraphDirection::RightLeft => "RL",
        }
    }
}

/// Graph exporter for various formats
pub struct GraphExporter {
    options: ExportOptions,
}

impl GraphExporter {
    pub fn new() -> Self {
        Self {
            options: ExportOptions::default(),
        }
    }

    pub fn with_options(options: ExportOptions) -> Self {
        Self { options }
    }

    /// Export call graph to DOT format
    pub fn to_dot(&self, graph: &CallGraph) -> String {
        let mut dot = String::new();

        // Header
        let title = self.options.title.as_deref().unwrap_or("CallGraph");
        dot.push_str(&format!("digraph \"{}\" {{\n", title));
        dot.push_str(&format!("  rankdir={};\n", self.options.direction.to_dot()));
        dot.push_str("  node [shape=box, style=filled];\n");
        dot.push_str("  edge [arrowsize=0.8];\n\n");

        // Nodes
        let nodes: Vec<_> = graph.nodes().collect();
        let node_count = if let Some(max) = self.options.max_nodes {
            nodes.len().min(max)
        } else {
            nodes.len()
        };

        for node in nodes.iter().take(node_count) {
            if let Some(ref filter) = self.options.node_filter {
                if !filter.contains(&node.kind()) {
                    continue;
                }
            }

            let color = self.get_node_color(node.kind());
            let label = if self.options.include_labels {
                format!("{}\\n{:x}", node.name(), node.address().as_u64())
            } else {
                format!("{:x}", node.address().as_u64())
            };

            dot.push_str(&format!(
                "  \"{}\" [label=\"{}\", fillcolor=\"{}\"];\n",
                node.address().as_u64(),
                label,
                color
            ));
        }

        dot.push('\n');

        // Edges
        for edge in graph.edges() {
            let style = self.get_edge_style(edge.kind());
            let label = if self.options.include_edge_labels {
                format!(" [label=\"{:?}\", {}]", edge.kind(), style)
            } else {
                format!(" [{}]", style)
            };

            dot.push_str(&format!(
                "  \"{}\" -> \"{}\"{}\n",
                edge.from().as_u64(),
                edge.to().as_u64(),
                label
            ));
        }

        dot.push_str("}\n");
        dot
    }

    /// Export call graph to JSON format
    pub fn to_json(&self, graph: &CallGraph) -> String {
        let mut json = String::new();
        json.push_str("{\n");

        // Nodes
        json.push_str("  \"nodes\": [\n");
        let nodes: Vec<_> = graph.nodes().collect();
        for (i, node) in nodes.iter().enumerate() {
            json.push_str(&format!(
                "    {{\"id\": \"{}\", \"name\": \"{}\", \"kind\": \"{:?}\", \"address\": \"0x{:x}\"}}",
                node.address().as_u64(),
                node.name(),
                node.kind(),
                node.address().as_u64()
            ));
            if i < nodes.len() - 1 {
                json.push(',');
            }
            json.push('\n');
        }
        json.push_str("  ],\n");

        // Edges
        json.push_str("  \"edges\": [\n");
        let edges: Vec<_> = graph.edges().collect();
        for (i, edge) in edges.iter().enumerate() {
            json.push_str(&format!(
                "    {{\"source\": \"{}\", \"target\": \"{}\", \"kind\": \"{:?}\"}}",
                edge.from().as_u64(),
                edge.to().as_u64(),
                edge.kind()
            ));
            if i < edges.len() - 1 {
                json.push(',');
            }
            json.push('\n');
        }
        json.push_str("  ]\n");

        json.push_str("}\n");
        json
    }

    /// Export call graph to D3.js compatible JSON format
    pub fn to_d3_json(&self, graph: &CallGraph) -> String {
        let mut json = String::new();
        json.push_str("{\n");

        // Create node index mapping
        let nodes: Vec<_> = graph.nodes().collect();
        let node_indices: HashMap<u64, usize> = nodes.iter()
            .enumerate()
            .map(|(i, n)| (n.address().as_u64(), i))
            .collect();

        // Nodes
        json.push_str("  \"nodes\": [\n");
        for (i, node) in nodes.iter().enumerate() {
            let color = self.get_node_color(node.kind());
            json.push_str(&format!(
                "    {{\"id\": {}, \"name\": \"{}\", \"group\": {}, \"color\": \"{}\"}}",
                i,
                node.name(),
                self.node_kind_to_group(node.kind()),
                color
            ));
            if i < nodes.len() - 1 {
                json.push(',');
            }
            json.push('\n');
        }
        json.push_str("  ],\n");

        // Links
        json.push_str("  \"links\": [\n");
        let edges: Vec<_> = graph.edges().collect();
        let mut valid_edges = Vec::new();
        for edge in &edges {
            if let (Some(&source), Some(&target)) = (
                node_indices.get(&edge.from().as_u64()),
                node_indices.get(&edge.to().as_u64())
            ) {
                valid_edges.push((source, target, edge));
            }
        }

        for (i, (source, target, edge)) in valid_edges.iter().enumerate() {
            json.push_str(&format!(
                "    {{\"source\": {}, \"target\": {}, \"value\": {}}}",
                source,
                target,
                self.edge_kind_to_weight(edge.kind())
            ));
            if i < valid_edges.len() - 1 {
                json.push(',');
            }
            json.push('\n');
        }
        json.push_str("  ]\n");

        json.push_str("}\n");
        json
    }

    /// Export call graph to GraphML format
    pub fn to_graphml(&self, graph: &CallGraph) -> String {
        let mut xml = String::new();

        xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        xml.push_str("<graphml xmlns=\"http://graphml.graphdrawing.org/xmlns\"\n");
        xml.push_str("    xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\"\n");
        xml.push_str("    xsi:schemaLocation=\"http://graphml.graphdrawing.org/xmlns\n");
        xml.push_str("        http://graphml.graphdrawing.org/xmlns/1.0/graphml.xsd\">\n\n");

        // Key definitions
        xml.push_str("  <key id=\"name\" for=\"node\" attr.name=\"name\" attr.type=\"string\"/>\n");
        xml.push_str("  <key id=\"kind\" for=\"node\" attr.name=\"kind\" attr.type=\"string\"/>\n");
        xml.push_str("  <key id=\"address\" for=\"node\" attr.name=\"address\" attr.type=\"string\"/>\n");
        xml.push_str("  <key id=\"edgekind\" for=\"edge\" attr.name=\"kind\" attr.type=\"string\"/>\n\n");

        xml.push_str("  <graph id=\"CallGraph\" edgedefault=\"directed\">\n");

        // Nodes
        for node in graph.nodes() {
            xml.push_str(&format!("    <node id=\"n{}\">\n", node.address().as_u64()));
            xml.push_str(&format!("      <data key=\"name\">{}</data>\n", 
                escape_xml(&node.name())));
            xml.push_str(&format!("      <data key=\"kind\">{:?}</data>\n", node.kind()));
            xml.push_str(&format!("      <data key=\"address\">0x{:x}</data>\n", 
                node.address().as_u64()));
            xml.push_str("    </node>\n");
        }

        // Edges
        let mut edge_id = 0;
        for edge in graph.edges() {
            xml.push_str(&format!(
                "    <edge id=\"e{}\" source=\"n{}\" target=\"n{}\">\n",
                edge_id,
                edge.from().as_u64(),
                edge.to().as_u64()
            ));
            xml.push_str(&format!("      <data key=\"edgekind\">{:?}</data>\n", edge.kind()));
            xml.push_str("    </edge>\n");
            edge_id += 1;
        }

        xml.push_str("  </graph>\n");
        xml.push_str("</graphml>\n");
        xml
    }

    /// Export call graph to CSV format (edge list)
    pub fn to_csv(&self, graph: &CallGraph) -> String {
        let mut csv = String::new();

        // Header
        csv.push_str("source_addr,source_name,target_addr,target_name,edge_kind\n");

        // Create node lookup
        let nodes: HashMap<u64, &GraphNode> = graph.nodes()
            .map(|n| (n.address().as_u64(), n))
            .collect();

        // Edges
        for edge in graph.edges() {
            let source_name = nodes.get(&edge.from().as_u64())
                .map(|n| n.name())
                .unwrap_or("unknown".to_string());
            let target_name = nodes.get(&edge.to().as_u64())
                .map(|n| n.name())
                .unwrap_or("unknown".to_string());

            csv.push_str(&format!(
                "0x{:x},{},0x{:x},{},{:?}\n",
                edge.from().as_u64(),
                escape_csv(&source_name),
                edge.to().as_u64(),
                escape_csv(&target_name),
                edge.kind()
            ));
        }

        csv
    }

    /// Export call graph to Mermaid format
    pub fn to_mermaid(&self, graph: &CallGraph) -> String {
        let mut mermaid = String::new();

        let direction = match self.options.direction {
            GraphDirection::TopBottom => "TD",
            GraphDirection::BottomTop => "BT",
            GraphDirection::LeftRight => "LR",
            GraphDirection::RightLeft => "RL",
        };

        mermaid.push_str(&format!("graph {}\n", direction));

        // Create short IDs for nodes
        let nodes: Vec<_> = graph.nodes().collect();
        let node_ids: HashMap<u64, String> = nodes.iter()
            .enumerate()
            .map(|(i, n)| (n.address().as_u64(), format!("N{}", i)))
            .collect();

        // Node definitions
        for node in &nodes {
            let id = &node_ids[&node.address().as_u64()];
            let label = if self.options.include_labels {
                format!("{}[{}]", id, escape_mermaid(&node.name()))
            } else {
                format!("{}[{:x}]", id, node.address().as_u64())
            };
            mermaid.push_str(&format!("    {}\n", label));
        }

        // Edges
        for edge in graph.edges() {
            if let (Some(from_id), Some(to_id)) = (
                node_ids.get(&edge.from().as_u64()),
                node_ids.get(&edge.to().as_u64())
            ) {
                let arrow = match edge.kind() {
                    EdgeKind::Call => "-->",
                    EdgeKind::Jump => "-.->",
                    EdgeKind::Reference | EdgeKind::Data => "-.->",
                    EdgeKind::String | EdgeKind::Constant => "~~>",
                };

                if self.options.include_edge_labels {
                    mermaid.push_str(&format!(
                        "    {} {}|{:?}| {}\n",
                        from_id, arrow, edge.kind(), to_id
                    ));
                } else {
                    mermaid.push_str(&format!("    {} {} {}\n", from_id, arrow, to_id));
                }
            }
        }

        mermaid
    }

    /// Export to specified format
    pub fn export(&self, graph: &CallGraph, format: ExportFormat) -> String {
        match format {
            ExportFormat::Dot => self.to_dot(graph),
            ExportFormat::Json => self.to_json(graph),
            ExportFormat::GraphML => self.to_graphml(graph),
            ExportFormat::Csv => self.to_csv(graph),
            ExportFormat::D3Json => self.to_d3_json(graph),
            ExportFormat::Mermaid => self.to_mermaid(graph),
        }
    }

    /// Write export to file
    pub fn export_to_file(&self, graph: &CallGraph, format: ExportFormat, path: &str) -> std::io::Result<()> {
        let content = self.export(graph, format);
        let mut file = std::fs::File::create(path)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }

    fn get_node_color(&self, kind: NodeKind) -> &str {
        if let Some(color) = self.options.custom_colors.get(&kind) {
            return color;
        }

        if !self.options.colorize {
            return "#ffffff";
        }

        match kind {
            NodeKind::Function => "#lightblue",
            NodeKind::Data => "#lightgreen",
            NodeKind::External => "#lightyellow",
            NodeKind::Unknown => "#lightgray",
            NodeKind::String => "#lightsalmon",
            NodeKind::Constant => "#lightcyan",
        }
    }

    fn get_edge_style(&self, kind: EdgeKind) -> &str {
        match kind {
            EdgeKind::Call => "color=blue",
            EdgeKind::Jump => "color=red, style=dashed",
            EdgeKind::Reference => "color=green, style=dotted",
            EdgeKind::Data => "color=purple, style=dotted",
            EdgeKind::String => "color=orange, style=dotted",
            EdgeKind::Constant => "color=gray, style=dotted",
        }
    }

    fn node_kind_to_group(&self, kind: NodeKind) -> usize {
        match kind {
            NodeKind::Function => 1,
            NodeKind::Data => 2,
            NodeKind::External => 3,
            NodeKind::Unknown => 0,
            NodeKind::String => 4,
            NodeKind::Constant => 5,
        }
    }

    fn edge_kind_to_weight(&self, kind: EdgeKind) -> usize {
        match kind {
            EdgeKind::Call => 3,
            EdgeKind::Jump => 2,
            EdgeKind::Reference => 1,
            EdgeKind::Data => 2,
            EdgeKind::String => 1,
            EdgeKind::Constant => 1,
        }
    }
}

impl Default for GraphExporter {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to escape XML special characters
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
     .replace('"', "&quot;")
     .replace('\'', "&apos;")
}

/// Helper to escape CSV special characters
fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

/// Helper to escape Mermaid special characters
fn escape_mermaid(s: &str) -> String {
    s.replace('[', "&#91;")
     .replace(']', "&#93;")
     .replace('(', "&#40;")
     .replace(')', "&#41;")
     .replace('{', "&#123;")
     .replace('}', "&#125;")
}

/// Subgraph extractor for focusing on specific parts of the graph
pub struct SubgraphExtractor;

impl SubgraphExtractor {
    /// Extract subgraph containing all paths from source to target
    pub fn extract_paths(graph: &CallGraph, source: Address, target: Address) -> CallGraph {
        let mut subgraph = CallGraph::new();
        let mut relevant_nodes = HashSet::new();

        // Find all paths using DFS
        Self::find_paths_dfs(graph, source, target, &mut Vec::new(), &mut relevant_nodes);

        // Add relevant nodes and edges
        for addr in &relevant_nodes {
            if let Some(node) = graph.get_node(Address::new(*addr)) {
                subgraph.add_node(node.clone());
            }
        }

        for edge in graph.edges() {
            if relevant_nodes.contains(&edge.from().as_u64()) && 
               relevant_nodes.contains(&edge.to().as_u64()) {
                subgraph.add_edge(edge.clone());
            }
        }

        subgraph
    }

    fn find_paths_dfs(
        graph: &CallGraph,
        current: Address,
        target: Address,
        path: &mut Vec<u64>,
        relevant: &mut HashSet<u64>
    ) -> bool {
        if path.contains(&current.as_u64()) {
            return false; // Cycle detected
        }

        path.push(current.as_u64());

        if current == target {
            // Found path - mark all nodes
            for &addr in path.iter() {
                relevant.insert(addr);
            }
            path.pop();
            return true;
        }

        let mut found = false;
        for edge in graph.get_outgoing(current) {
            if Self::find_paths_dfs(graph, edge.to(), target, path, relevant) {
                found = true;
            }
        }

        path.pop();
        found
    }

    /// Extract subgraph within N hops of a node
    pub fn extract_neighborhood(graph: &CallGraph, center: Address, hops: usize) -> CallGraph {
        let mut subgraph = CallGraph::new();
        let mut visited = HashSet::new();
        let mut current_level = vec![center];
        let mut next_level = Vec::new();

        for _ in 0..=hops {
            for addr in &current_level {
                if visited.contains(&addr.as_u64()) {
                    continue;
                }
                visited.insert(addr.as_u64());

                if let Some(node) = graph.get_node(*addr) {
                    subgraph.add_node(node.clone());
                }

                // Add neighbors
                for edge in graph.get_outgoing(*addr) {
                    next_level.push(edge.to());
                    subgraph.add_edge(edge.clone());
                }
                for edge in graph.get_incoming(*addr) {
                    next_level.push(edge.from());
                    subgraph.add_edge(edge.clone());
                }
            }

            current_level = next_level;
            next_level = Vec::new();
        }

        subgraph
    }

    /// Extract strongly connected components
    pub fn extract_scc(graph: &CallGraph) -> Vec<CallGraph> {
        // Simplified - would need full Tarjan/Kosaraju implementation
        vec![graph.clone()]
    }
}

/// Graph statistics calculator
pub struct GraphStatistics;

impl GraphStatistics {
    pub fn calculate(graph: &CallGraph) -> GraphStats {
        let node_count = graph.len();
        let edge_count = graph.edge_count();

        // Calculate in/out degrees
        let mut in_degrees: HashMap<u64, usize> = HashMap::new();
        let mut out_degrees: HashMap<u64, usize> = HashMap::new();

        for node in graph.nodes() {
            let addr = node.address().as_u64();
            in_degrees.insert(addr, 0);
            out_degrees.insert(addr, 0);
        }

        for edge in graph.edges() {
            *out_degrees.entry(edge.from().as_u64()).or_default() += 1;
            *in_degrees.entry(edge.to().as_u64()).or_default() += 1;
        }

        let max_in = *in_degrees.values().max().unwrap_or(&0);
        let max_out = *out_degrees.values().max().unwrap_or(&0);
        let avg_in = if node_count > 0 {
            in_degrees.values().sum::<usize>() as f64 / node_count as f64
        } else {
            0.0
        };
        let avg_out = if node_count > 0 {
            out_degrees.values().sum::<usize>() as f64 / node_count as f64
        } else {
            0.0
        };

        // Count roots and leaves
        let roots = in_degrees.values().filter(|&&d| d == 0).count();
        let leaves = out_degrees.values().filter(|&&d| d == 0).count();

        GraphStats {
            node_count,
            edge_count,
            max_in_degree: max_in,
            max_out_degree: max_out,
            avg_in_degree: avg_in,
            avg_out_degree: avg_out,
            root_count: roots,
            leaf_count: leaves,
            density: if node_count > 1 {
                edge_count as f64 / (node_count * (node_count - 1)) as f64
            } else {
                0.0
            },
        }
    }
}

/// Statistics about a graph
#[derive(Debug, Clone)]
pub struct GraphStats {
    pub node_count: usize,
    pub edge_count: usize,
    pub max_in_degree: usize,
    pub max_out_degree: usize,
    pub avg_in_degree: f64,
    pub avg_out_degree: f64,
    pub root_count: usize,
    pub leaf_count: usize,
    pub density: f64,
}

impl fmt::Display for GraphStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Graph Statistics:")?;
        writeln!(f, "  Nodes: {}", self.node_count)?;
        writeln!(f, "  Edges: {}", self.edge_count)?;
        writeln!(f, "  Max in-degree: {}", self.max_in_degree)?;
        writeln!(f, "  Max out-degree: {}", self.max_out_degree)?;
        writeln!(f, "  Avg in-degree: {:.2}", self.avg_in_degree)?;
        writeln!(f, "  Avg out-degree: {:.2}", self.avg_out_degree)?;
        writeln!(f, "  Roots: {}", self.root_count)?;
        writeln!(f, "  Leaves: {}", self.leaf_count)?;
        writeln!(f, "  Density: {:.4}", self.density)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dot_export() {
        let mut graph = CallGraph::new();
        graph.add_node(GraphNode::new(Address::new(0x1000), "main".to_string(), NodeKind::Function));
        graph.add_node(GraphNode::new(Address::new(0x2000), "helper".to_string(), NodeKind::Function));
        graph.add_edge(GraphEdge::new(Address::new(0x1000), Address::new(0x2000), EdgeKind::Call));

        let exporter = GraphExporter::new();
        let dot = exporter.to_dot(&graph);

        assert!(dot.contains("digraph"));
        assert!(dot.contains("main"));
        assert!(dot.contains("helper"));
    }

    #[test]
    fn test_json_export() {
        let mut graph = CallGraph::new();
        graph.add_node(GraphNode::new(Address::new(0x1000), "func".to_string(), NodeKind::Function));

        let exporter = GraphExporter::new();
        let json = exporter.to_json(&graph);

        assert!(json.contains("\"nodes\""));
        assert!(json.contains("func"));
    }
}
