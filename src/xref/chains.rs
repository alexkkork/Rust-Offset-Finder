// Tue Jan 15 2026 - Alex

use crate::memory::{Address, MemoryReader};
use crate::xref::{CallGraph, EdgeKind, XRefKind};
use std::sync::Arc;
use std::collections::{HashMap, HashSet, VecDeque};
use std::cmp::Ordering;
use std::fmt;

/// Represents a chain of references from one point to another
#[derive(Debug, Clone)]
pub struct ReferenceChain {
    /// Starting address
    pub start: Address,
    /// Ending address
    pub end: Address,
    /// Links in the chain
    pub links: Vec<ChainLink>,
    /// Total chain length
    pub length: usize,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,
}

impl ReferenceChain {
    pub fn new(start: Address, end: Address) -> Self {
        Self {
            start,
            end,
            links: Vec::new(),
            length: 0,
            confidence: 1.0,
        }
    }

    pub fn add_link(&mut self, link: ChainLink) {
        self.confidence *= link.confidence;
        self.links.push(link);
        self.length = self.links.len();
    }

    pub fn is_empty(&self) -> bool {
        self.links.is_empty()
    }

    pub fn get_addresses(&self) -> Vec<Address> {
        let mut addrs = vec![self.start];
        for link in &self.links {
            addrs.push(link.target);
        }
        addrs
    }

    pub fn contains(&self, addr: Address) -> bool {
        if self.start == addr || self.end == addr {
            return true;
        }
        self.links.iter().any(|l| l.source == addr || l.target == addr)
    }

    pub fn get_types(&self) -> Vec<ChainLinkType> {
        self.links.iter().map(|l| l.link_type).collect()
    }

    pub fn all_same_type(&self) -> bool {
        if self.links.is_empty() {
            return true;
        }
        let first = self.links[0].link_type;
        self.links.iter().all(|l| l.link_type == first)
    }

    /// Reverse the chain direction
    pub fn reverse(&self) -> ReferenceChain {
        let mut reversed = ReferenceChain::new(self.end, self.start);
        for link in self.links.iter().rev() {
            reversed.add_link(ChainLink {
                source: link.target,
                target: link.source,
                link_type: link.link_type,
                confidence: link.confidence,
                metadata: link.metadata.clone(),
            });
        }
        reversed
    }
}

impl fmt::Display for ReferenceChain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:016x}", self.start.as_u64())?;
        for link in &self.links {
            let arrow = match link.link_type {
                ChainLinkType::Call => " --call-> ",
                ChainLinkType::Jump => " --jump-> ",
                ChainLinkType::DataRef => " --data-> ",
                ChainLinkType::Indirect => " --ind-> ",
                ChainLinkType::Return => " --ret-> ",
                ChainLinkType::Unknown => " --> ",
            };
            write!(f, "{}{:016x}", arrow, link.target.as_u64())?;
        }
        write!(f, " (conf: {:.2})", self.confidence)
    }
}

/// A single link in a reference chain
#[derive(Debug, Clone)]
pub struct ChainLink {
    /// Source of the link
    pub source: Address,
    /// Target of the link
    pub target: Address,
    /// Type of link
    pub link_type: ChainLinkType,
    /// Confidence that this link is correct
    pub confidence: f64,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl ChainLink {
    pub fn new(source: Address, target: Address, link_type: ChainLinkType) -> Self {
        Self {
            source,
            target,
            link_type,
            confidence: 1.0,
            metadata: HashMap::new(),
        }
    }

    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence;
        self
    }

    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
}

impl fmt::Display for ChainLink {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:016x} --{:?}--> {:016x}", 
            self.source.as_u64(), self.link_type, self.target.as_u64())
    }
}

/// Types of links in a reference chain
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChainLinkType {
    /// Direct function call
    Call,
    /// Jump/branch
    Jump,
    /// Data reference (pointer dereference)
    DataRef,
    /// Indirect reference (computed)
    Indirect,
    /// Return from function
    Return,
    /// Unknown type
    Unknown,
}

impl From<EdgeKind> for ChainLinkType {
    fn from(kind: EdgeKind) -> Self {
        match kind {
            EdgeKind::Call => ChainLinkType::Call,
            EdgeKind::Jump => ChainLinkType::Jump,
            EdgeKind::Reference | EdgeKind::Data => ChainLinkType::DataRef,
            EdgeKind::String | EdgeKind::Constant => ChainLinkType::Unknown,
        }
    }
}

impl From<XRefKind> for ChainLinkType {
    fn from(kind: XRefKind) -> Self {
        match kind {
            XRefKind::Call => ChainLinkType::Call,
            XRefKind::Jump => ChainLinkType::Jump,
            XRefKind::Data => ChainLinkType::DataRef,
            XRefKind::String => ChainLinkType::DataRef,
            XRefKind::Unknown => ChainLinkType::Unknown,
        }
    }
}

/// Analyzer for finding and analyzing reference chains
pub struct ChainAnalyzer {
    reader: Arc<dyn MemoryReader>,
    call_graph: CallGraph,
    max_chain_length: usize,
    allow_cycles: bool,
}

impl ChainAnalyzer {
    pub fn new(reader: Arc<dyn MemoryReader>, call_graph: CallGraph) -> Self {
        Self {
            reader,
            call_graph,
            max_chain_length: 20,
            allow_cycles: false,
        }
    }

    pub fn with_max_length(mut self, max: usize) -> Self {
        self.max_chain_length = max;
        self
    }

    pub fn with_cycles(mut self, allow: bool) -> Self {
        self.allow_cycles = allow;
        self
    }

    /// Find all chains from source to target
    pub fn find_chains(&self, source: Address, target: Address) -> Vec<ReferenceChain> {
        let mut chains = Vec::new();
        let mut path = Vec::new();
        let mut visited = HashSet::new();

        self.find_chains_dfs(source, target, &mut path, &mut visited, &mut chains);

        // Sort by length (shorter is better)
        chains.sort_by_key(|c| c.length);
        chains
    }

    fn find_chains_dfs(
        &self,
        current: Address,
        target: Address,
        path: &mut Vec<ChainLink>,
        visited: &mut HashSet<u64>,
        chains: &mut Vec<ReferenceChain>,
    ) {
        if path.len() >= self.max_chain_length {
            return;
        }

        if !self.allow_cycles && visited.contains(&current.as_u64()) {
            return;
        }

        visited.insert(current.as_u64());

        if current == target && !path.is_empty() {
            let mut chain = ReferenceChain::new(
                path.first().map(|l| l.source).unwrap_or(current),
                target,
            );
            for link in path.iter() {
                chain.add_link(link.clone());
            }
            chains.push(chain);
        } else {
            for edge in self.call_graph.get_outgoing(current) {
                let link = ChainLink::new(current, edge.to(), edge.kind().into());
                path.push(link);
                self.find_chains_dfs(edge.to(), target, path, visited, chains);
                path.pop();
            }
        }

        visited.remove(&current.as_u64());
    }

    /// Find the shortest chain from source to target using BFS
    pub fn find_shortest_chain(&self, source: Address, target: Address) -> Option<ReferenceChain> {
        if source == target {
            return Some(ReferenceChain::new(source, target));
        }

        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        let mut parent: HashMap<u64, (u64, ChainLink)> = HashMap::new();

        queue.push_back(source);
        visited.insert(source.as_u64());

        while let Some(current) = queue.pop_front() {
            for edge in self.call_graph.get_outgoing(current) {
                let next = edge.to();
                if visited.contains(&next.as_u64()) {
                    continue;
                }

                visited.insert(next.as_u64());
                let link = ChainLink::new(current, next, edge.kind().into());
                parent.insert(next.as_u64(), (current.as_u64(), link));

                if next == target {
                    // Reconstruct path
                    return Some(self.reconstruct_chain(source, target, &parent));
                }

                queue.push_back(next);
            }
        }

        None
    }

    fn reconstruct_chain(
        &self,
        source: Address,
        target: Address,
        parent: &HashMap<u64, (u64, ChainLink)>,
    ) -> ReferenceChain {
        let mut chain = ReferenceChain::new(source, target);
        let mut links = Vec::new();
        let mut current = target.as_u64();

        while current != source.as_u64() {
            if let Some((prev, link)) = parent.get(&current) {
                links.push(link.clone());
                current = *prev;
            } else {
                break;
            }
        }

        links.reverse();
        for link in links {
            chain.add_link(link);
        }

        chain
    }

    /// Find all chains from a source to any target in a set
    pub fn find_chains_to_any(&self, source: Address, targets: &HashSet<Address>) -> Vec<ReferenceChain> {
        let mut all_chains = Vec::new();
        for target in targets {
            let mut chains = self.find_chains(source, *target);
            all_chains.append(&mut chains);
        }
        all_chains
    }

    /// Find all chains from any source in a set to a target
    pub fn find_chains_from_any(&self, sources: &HashSet<Address>, target: Address) -> Vec<ReferenceChain> {
        let mut all_chains = Vec::new();
        for source in sources {
            let mut chains = self.find_chains(*source, target);
            all_chains.append(&mut chains);
        }
        all_chains
    }

    /// Find all call chains leading to a function
    pub fn find_callers_chain(&self, target: Address, max_depth: usize) -> Vec<ReferenceChain> {
        let mut chains = Vec::new();
        let mut visited = HashSet::new();
        
        self.find_callers_recursive(target, &mut Vec::new(), &mut visited, &mut chains, max_depth);
        chains
    }

    fn find_callers_recursive(
        &self,
        current: Address,
        path: &mut Vec<ChainLink>,
        visited: &mut HashSet<u64>,
        chains: &mut Vec<ReferenceChain>,
        remaining_depth: usize,
    ) {
        if remaining_depth == 0 {
            return;
        }

        if visited.contains(&current.as_u64()) {
            return;
        }
        visited.insert(current.as_u64());

        for edge in self.call_graph.get_incoming(current) {
            let link = ChainLink::new(edge.from(), current, edge.kind().into());
            path.push(link);

            // Create a chain from this caller
            if !path.is_empty() {
                let mut chain = ReferenceChain::new(
                    path.first().map(|l| l.source).unwrap_or(current),
                    current,
                );
                for l in path.iter() {
                    chain.add_link(l.clone());
                }
                chains.push(chain);
            }

            self.find_callers_recursive(edge.from(), path, visited, chains, remaining_depth - 1);
            path.pop();
        }

        visited.remove(&current.as_u64());
    }

    /// Find all chains of a specific type
    pub fn find_chains_by_type(&self, source: Address, target: Address, link_type: ChainLinkType) -> Vec<ReferenceChain> {
        self.find_chains(source, target)
            .into_iter()
            .filter(|c| c.all_same_type() && c.get_types().first() == Some(&link_type))
            .collect()
    }

    /// Analyze chain complexity
    pub fn analyze_chain(&self, chain: &ReferenceChain) -> ChainAnalysis {
        let mut type_counts: HashMap<ChainLinkType, usize> = HashMap::new();
        let mut unique_addresses = HashSet::new();

        unique_addresses.insert(chain.start.as_u64());
        unique_addresses.insert(chain.end.as_u64());

        for link in &chain.links {
            *type_counts.entry(link.link_type).or_default() += 1;
            unique_addresses.insert(link.source.as_u64());
            unique_addresses.insert(link.target.as_u64());
        }

        let has_cycle = unique_addresses.len() < chain.links.len() + 1;
        let is_direct = chain.length == 1;
        let is_homogeneous = type_counts.len() <= 1;

        ChainAnalysis {
            length: chain.length,
            unique_nodes: unique_addresses.len(),
            type_distribution: type_counts,
            has_cycle,
            is_direct,
            is_homogeneous,
            confidence: chain.confidence,
            complexity_score: self.calculate_complexity(chain),
        }
    }

    fn calculate_complexity(&self, chain: &ReferenceChain) -> f64 {
        let mut score = chain.length as f64;

        // Indirect calls add complexity
        for link in &chain.links {
            match link.link_type {
                ChainLinkType::Indirect => score += 2.0,
                ChainLinkType::DataRef => score += 1.0,
                _ => {}
            }
        }

        // Low confidence adds complexity
        if chain.confidence < 0.8 {
            score += (1.0 - chain.confidence) * 5.0;
        }

        score
    }
}

/// Analysis results for a chain
#[derive(Debug, Clone)]
pub struct ChainAnalysis {
    pub length: usize,
    pub unique_nodes: usize,
    pub type_distribution: HashMap<ChainLinkType, usize>,
    pub has_cycle: bool,
    pub is_direct: bool,
    pub is_homogeneous: bool,
    pub confidence: f64,
    pub complexity_score: f64,
}

impl fmt::Display for ChainAnalysis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Chain Analysis:")?;
        writeln!(f, "  Length: {}", self.length)?;
        writeln!(f, "  Unique nodes: {}", self.unique_nodes)?;
        writeln!(f, "  Has cycle: {}", self.has_cycle)?;
        writeln!(f, "  Is direct: {}", self.is_direct)?;
        writeln!(f, "  Is homogeneous: {}", self.is_homogeneous)?;
        writeln!(f, "  Confidence: {:.2}", self.confidence)?;
        writeln!(f, "  Complexity score: {:.2}", self.complexity_score)?;
        writeln!(f, "  Type distribution:")?;
        for (typ, count) in &self.type_distribution {
            writeln!(f, "    {:?}: {}", typ, count)?;
        }
        Ok(())
    }
}

/// Builder for constructing chains programmatically
pub struct ChainBuilder {
    start: Option<Address>,
    links: Vec<ChainLink>,
}

impl ChainBuilder {
    pub fn new() -> Self {
        Self {
            start: None,
            links: Vec::new(),
        }
    }

    pub fn from(mut self, addr: Address) -> Self {
        self.start = Some(addr);
        self
    }

    pub fn call_to(mut self, target: Address) -> Self {
        if let Some(source) = self.links.last().map(|l| l.target).or(self.start) {
            self.links.push(ChainLink::new(source, target, ChainLinkType::Call));
        }
        self
    }

    pub fn jump_to(mut self, target: Address) -> Self {
        if let Some(source) = self.links.last().map(|l| l.target).or(self.start) {
            self.links.push(ChainLink::new(source, target, ChainLinkType::Jump));
        }
        self
    }

    pub fn data_ref_to(mut self, target: Address) -> Self {
        if let Some(source) = self.links.last().map(|l| l.target).or(self.start) {
            self.links.push(ChainLink::new(source, target, ChainLinkType::DataRef));
        }
        self
    }

    pub fn build(self) -> Option<ReferenceChain> {
        let start = self.start?;
        let end = self.links.last().map(|l| l.target)?;

        let mut chain = ReferenceChain::new(start, end);
        for link in self.links {
            chain.add_link(link);
        }
        Some(chain)
    }
}

impl Default for ChainBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Compares and ranks chains
pub struct ChainRanker;

impl ChainRanker {
    /// Rank chains by multiple criteria
    pub fn rank(chains: &[ReferenceChain]) -> Vec<(usize, &ReferenceChain, f64)> {
        let mut ranked: Vec<_> = chains.iter()
            .enumerate()
            .map(|(i, c)| (i, c, Self::calculate_score(c)))
            .collect();

        ranked.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(Ordering::Equal));
        ranked
    }

    fn calculate_score(chain: &ReferenceChain) -> f64 {
        let mut score = 100.0;

        // Shorter chains are better
        score -= chain.length as f64 * 5.0;

        // Higher confidence is better
        score += chain.confidence * 20.0;

        // Direct calls are better than indirect
        for link in &chain.links {
            match link.link_type {
                ChainLinkType::Call => {}
                ChainLinkType::Jump => score -= 2.0,
                ChainLinkType::DataRef => score -= 5.0,
                ChainLinkType::Indirect => score -= 10.0,
                _ => score -= 3.0,
            }
        }

        score.max(0.0)
    }

    /// Get the best chain
    pub fn best(chains: &[ReferenceChain]) -> Option<&ReferenceChain> {
        Self::rank(chains).first().map(|(_, c, _)| *c)
    }

    /// Filter chains above a score threshold
    pub fn filter_by_score(chains: &[ReferenceChain], min_score: f64) -> Vec<&ReferenceChain> {
        Self::rank(chains)
            .into_iter()
            .filter(|(_, _, score)| *score >= min_score)
            .map(|(_, c, _)| c)
            .collect()
    }
}

/// Merges multiple chains
pub struct ChainMerger;

impl ChainMerger {
    /// Merge two chains if they share a common endpoint
    pub fn merge(chain1: &ReferenceChain, chain2: &ReferenceChain) -> Option<ReferenceChain> {
        if chain1.end == chain2.start {
            // chain1 -> chain2
            let mut merged = chain1.clone();
            for link in &chain2.links {
                merged.add_link(link.clone());
            }
            merged.end = chain2.end;
            Some(merged)
        } else if chain2.end == chain1.start {
            // chain2 -> chain1
            let mut merged = chain2.clone();
            for link in &chain1.links {
                merged.add_link(link.clone());
            }
            merged.end = chain1.end;
            Some(merged)
        } else {
            None
        }
    }

    /// Find all possible merges among a set of chains
    pub fn find_mergeable(chains: &[ReferenceChain]) -> Vec<(usize, usize)> {
        let mut pairs = Vec::new();
        
        for i in 0..chains.len() {
            for j in 0..chains.len() {
                if i != j && chains[i].end == chains[j].start {
                    pairs.push((i, j));
                }
            }
        }

        pairs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_builder() {
        let chain = ChainBuilder::new()
            .from(Address::new(0x1000))
            .call_to(Address::new(0x2000))
            .call_to(Address::new(0x3000))
            .build();

        assert!(chain.is_some());
        let chain = chain.unwrap();
        assert_eq!(chain.length, 2);
        assert_eq!(chain.start, Address::new(0x1000));
        assert_eq!(chain.end, Address::new(0x3000));
    }

    #[test]
    fn test_chain_link_types() {
        let link = ChainLink::new(
            Address::new(0x1000),
            Address::new(0x2000),
            ChainLinkType::Call,
        );

        assert_eq!(link.link_type, ChainLinkType::Call);
        assert_eq!(link.confidence, 1.0);
    }

    #[test]
    fn test_chain_reverse() {
        let mut chain = ReferenceChain::new(Address::new(0x1000), Address::new(0x3000));
        chain.add_link(ChainLink::new(Address::new(0x1000), Address::new(0x2000), ChainLinkType::Call));
        chain.add_link(ChainLink::new(Address::new(0x2000), Address::new(0x3000), ChainLinkType::Call));

        let reversed = chain.reverse();
        assert_eq!(reversed.start, Address::new(0x3000));
        assert_eq!(reversed.end, Address::new(0x1000));
    }
}
