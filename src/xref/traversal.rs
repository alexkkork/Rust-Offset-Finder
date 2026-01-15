// Tue Jan 13 2026 - Alex

use crate::memory::Address;
use crate::xref::{CallGraph, XRefError};
use std::collections::{HashSet, VecDeque};

pub struct XRefTraverser {
    graph: CallGraph,
    visited: HashSet<u64>,
    max_depth: usize,
}

impl XRefTraverser {
    pub fn new(graph: CallGraph, max_depth: usize) -> Self {
        Self {
            graph,
            visited: HashSet::new(),
            max_depth,
        }
    }

    pub fn traverse_bfs(&mut self, start: Address) -> Result<Vec<Address>, XRefError> {
        let mut result = Vec::new();
        let mut queue = VecDeque::new();
        queue.push_back((start, 0));
        self.visited.insert(start.as_u64());

        while let Some((current, depth)) = queue.pop_front() {
            if depth > self.max_depth {
                continue;
            }
            result.push(current);

            let outgoing: Vec<Address> = self.graph.get_outgoing(current).iter().map(|e| e.to()).collect();
            for next in outgoing {
                if !self.visited.contains(&next.as_u64()) {
                    self.visited.insert(next.as_u64());
                    queue.push_back((next, depth + 1));
                }
            }
        }

        Ok(result)
    }

    pub fn traverse_dfs(&mut self, start: Address) -> Result<Vec<Address>, XRefError> {
        let mut result = Vec::new();
        self.dfs_recursive(start, 0, &mut result)?;
        Ok(result)
    }

    fn dfs_recursive(&mut self, current: Address, depth: usize, result: &mut Vec<Address>) -> Result<(), XRefError> {
        if depth > self.max_depth {
            return Ok(());
        }
        if self.visited.contains(&current.as_u64()) {
            return Ok(());
        }
        self.visited.insert(current.as_u64());
        result.push(current);

        let outgoing: Vec<Address> = self.graph.get_outgoing(current).iter().map(|e| e.to()).collect();
        for next in outgoing {
            self.dfs_recursive(next, depth + 1, result)?;
        }

        Ok(())
    }

    pub fn find_callers(&self, target: Address) -> Vec<Address> {
        self.graph.get_incoming(target).iter().map(|e| e.from()).collect()
    }

    pub fn find_callees(&self, source: Address) -> Vec<Address> {
        self.graph.get_outgoing(source).iter().map(|e| e.to()).collect()
    }

    pub fn find_path(&mut self, from: Address, to: Address) -> Option<Vec<Address>> {
        let mut queue = VecDeque::new();
        let mut parent: std::collections::HashMap<u64, u64> = std::collections::HashMap::new();
        queue.push_back(from);
        self.visited.clear();
        self.visited.insert(from.as_u64());

        while let Some(current) = queue.pop_front() {
            if current.as_u64() == to.as_u64() {
                let mut path = vec![to];
                let mut node = to.as_u64();
                while let Some(&p) = parent.get(&node) {
                    path.push(Address::new(p));
                    node = p;
                }
                path.reverse();
                return Some(path);
            }

            let outgoing: Vec<Address> = self.graph.get_outgoing(current).iter().map(|e| e.to()).collect();
            for next in outgoing {
                if !self.visited.contains(&next.as_u64()) {
                    self.visited.insert(next.as_u64());
                    parent.insert(next.as_u64(), current.as_u64());
                    queue.push_back(next);
                }
            }
        }

        None
    }

    pub fn reset(&mut self) {
        self.visited.clear();
    }

    pub fn visited_count(&self) -> usize {
        self.visited.len()
    }

    pub fn graph(&self) -> &CallGraph {
        &self.graph
    }
}

pub fn find_all_references(graph: &CallGraph, target: Address) -> Vec<Address> {
    graph.get_incoming(target).iter().map(|e| e.from()).collect()
}

pub fn count_references(graph: &CallGraph, target: Address) -> usize {
    graph.get_incoming(target).len()
}

pub fn get_reference_chain(graph: &CallGraph, start: Address, depth: usize) -> Vec<Vec<Address>> {
    let mut chains = Vec::new();
    let mut current_chain = vec![start];
    build_chains(graph, start, depth, &mut current_chain, &mut chains, &mut HashSet::new());
    chains
}

fn build_chains(
    graph: &CallGraph,
    current: Address,
    remaining_depth: usize,
    current_chain: &mut Vec<Address>,
    chains: &mut Vec<Vec<Address>>,
    visited: &mut HashSet<u64>,
) {
    if remaining_depth == 0 {
        chains.push(current_chain.clone());
        return;
    }

    if visited.contains(&current.as_u64()) {
        return;
    }
    visited.insert(current.as_u64());

    let outgoing: Vec<Address> = graph.get_outgoing(current).iter().map(|e| e.to()).collect();
    if outgoing.is_empty() {
        chains.push(current_chain.clone());
    } else {
        for next in outgoing {
            current_chain.push(next);
            build_chains(graph, next, remaining_depth - 1, current_chain, chains, visited);
            current_chain.pop();
        }
    }

    visited.remove(&current.as_u64());
}
