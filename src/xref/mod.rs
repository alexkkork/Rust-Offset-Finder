// Tue Jan 13 2026 - Alex

pub mod analyzer;
pub mod callgraph;
pub mod reference;
pub mod traversal;
pub mod builder;
pub mod error;
pub mod cache;
pub mod node;
pub mod edge;
pub mod path;
pub mod filter;
pub mod stats;

pub use analyzer::XRefAnalyzer;
pub use callgraph::CallGraph;
pub use reference::XRef;
pub use traversal::XRefTraverser;
pub use builder::CallGraphBuilder;
pub use error::XRefError;
pub use node::GraphNode;
pub use edge::GraphEdge;
pub use path::XRefPath;
pub use filter::XRefFilter;
pub use stats::XRefStats;
pub use node::NodeKind;
pub use edge::EdgeKind;
