// Tue Jan 13 2026 - Alex

use crate::memory::Address;
use crate::analysis::{BasicBlock, Instruction};
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;

#[derive(Debug, Clone)]
pub struct ControlFlowGraph {
    blocks: HashMap<u64, BasicBlock>,
    entry: Option<Address>,
    exits: Vec<Address>,
    dominators: HashMap<u64, u64>,
    post_dominators: HashMap<u64, u64>,
    loops: Vec<Loop>,
}

#[derive(Debug, Clone)]
pub struct Loop {
    header: Address,
    blocks: Vec<Address>,
    exits: Vec<Address>,
    back_edges: Vec<(Address, Address)>,
    nesting_level: usize,
}

#[derive(Debug, Clone)]
pub struct DominatorTree {
    root: Address,
    children: HashMap<u64, Vec<Address>>,
    parent: HashMap<u64, Address>,
}

impl ControlFlowGraph {
    pub fn new() -> Self {
        Self {
            blocks: HashMap::new(),
            entry: None,
            exits: Vec::new(),
            dominators: HashMap::new(),
            post_dominators: HashMap::new(),
            loops: Vec::new(),
        }
    }

    pub fn from_blocks(blocks: Vec<BasicBlock>) -> Self {
        let mut cfg = Self::new();

        if let Some(first) = blocks.first() {
            cfg.entry = Some(first.start());
        }

        for block in blocks {
            if block.is_exit() {
                cfg.exits.push(block.start());
            }
            cfg.blocks.insert(block.start().as_u64(), block);
        }

        cfg.compute_dominators();
        cfg.find_loops();

        cfg
    }

    pub fn entry(&self) -> Option<Address> {
        self.entry
    }

    pub fn exits(&self) -> &[Address] {
        &self.exits
    }

    pub fn block_count(&self) -> usize {
        self.blocks.len()
    }

    pub fn get_block(&self, addr: Address) -> Option<&BasicBlock> {
        self.blocks.get(&addr.as_u64())
    }

    pub fn get_block_mut(&mut self, addr: Address) -> Option<&mut BasicBlock> {
        self.blocks.get_mut(&addr.as_u64())
    }

    pub fn blocks(&self) -> impl Iterator<Item = &BasicBlock> {
        self.blocks.values()
    }

    pub fn blocks_sorted(&self) -> Vec<&BasicBlock> {
        let mut blocks: Vec<_> = self.blocks.values().collect();
        blocks.sort_by_key(|b| b.start().as_u64());
        blocks
    }

    pub fn add_block(&mut self, block: BasicBlock) {
        let addr = block.start().as_u64();
        if self.entry.is_none() {
            self.entry = Some(block.start());
        }
        if block.is_exit() {
            self.exits.push(block.start());
        }
        self.blocks.insert(addr, block);
    }

    pub fn remove_block(&mut self, addr: Address) -> Option<BasicBlock> {
        let removed = self.blocks.remove(&addr.as_u64());
        if let Some(entry) = self.entry {
            if entry == addr {
                self.entry = None;
            }
        }
        self.exits.retain(|&e| e != addr);
        removed
    }

    pub fn successors(&self, addr: Address) -> Vec<Address> {
        self.blocks
            .get(&addr.as_u64())
            .map(|b| b.successors().to_vec())
            .unwrap_or_default()
    }

    pub fn predecessors(&self, addr: Address) -> Vec<Address> {
        self.blocks
            .get(&addr.as_u64())
            .map(|b| b.predecessors().to_vec())
            .unwrap_or_default()
    }

    pub fn dominator(&self, addr: Address) -> Option<Address> {
        self.dominators.get(&addr.as_u64()).map(|&d| Address::new(d))
    }

    pub fn dominates(&self, dom: Address, target: Address) -> bool {
        if dom == target {
            return true;
        }
        let mut current = target;
        while let Some(d) = self.dominator(current) {
            if d == dom {
                return true;
            }
            if d == current {
                break;
            }
            current = d;
        }
        false
    }

    pub fn post_dominator(&self, addr: Address) -> Option<Address> {
        self.post_dominators.get(&addr.as_u64()).map(|&d| Address::new(d))
    }

    pub fn loops(&self) -> &[Loop] {
        &self.loops
    }

    pub fn is_in_loop(&self, addr: Address) -> bool {
        self.loops.iter().any(|l| l.contains(addr))
    }

    pub fn get_loop(&self, addr: Address) -> Option<&Loop> {
        self.loops.iter().find(|l| l.header == addr)
    }

    pub fn compute_dominators(&mut self) {
        let entry = match self.entry {
            Some(e) => e,
            None => return,
        };

        let all_blocks: HashSet<u64> = self.blocks.keys().copied().collect();
        let mut doms: HashMap<u64, HashSet<u64>> = HashMap::new();

        doms.insert(entry.as_u64(), [entry.as_u64()].into_iter().collect());

        for &addr in &all_blocks {
            if addr != entry.as_u64() {
                doms.insert(addr, all_blocks.clone());
            }
        }

        let mut changed = true;
        while changed {
            changed = false;
            for &addr in &all_blocks {
                if addr == entry.as_u64() {
                    continue;
                }

                let preds = self.predecessors(Address::new(addr));
                if preds.is_empty() {
                    continue;
                }

                let mut new_dom = all_blocks.clone();
                for pred in &preds {
                    if let Some(pred_dom) = doms.get(&pred.as_u64()) {
                        new_dom = new_dom.intersection(pred_dom).copied().collect();
                    }
                }
                new_dom.insert(addr);

                if new_dom != *doms.get(&addr).unwrap_or(&HashSet::new()) {
                    doms.insert(addr, new_dom);
                    changed = true;
                }
            }
        }

        for (&addr, dom_set) in &doms {
            if addr == entry.as_u64() {
                self.dominators.insert(addr, addr);
                continue;
            }

            let mut immediate_dom = None;
            for &dom in dom_set {
                if dom == addr {
                    continue;
                }
                let is_immediate = dom_set.iter()
                    .filter(|&&d| d != addr && d != dom)
                    .all(|&d| {
                        if let Some(d_set) = doms.get(&d) {
                            d_set.contains(&dom)
                        } else {
                            false
                        }
                    });
                if is_immediate {
                    immediate_dom = Some(dom);
                    break;
                }
            }

            if let Some(idom) = immediate_dom {
                self.dominators.insert(addr, idom);
            }
        }
    }

    pub fn find_loops(&mut self) {
        self.loops.clear();

        let entry = match self.entry {
            Some(e) => e,
            None => return,
        };

        let mut back_edges = Vec::new();

        for block in self.blocks.values() {
            for succ in block.successors() {
                if self.dominates(*succ, block.start()) {
                    back_edges.push((block.start(), *succ));
                }
            }
        }

        for (tail, header) in back_edges {
            let mut loop_blocks = HashSet::new();
            loop_blocks.insert(header);

            let mut work_list = vec![tail];
            while let Some(block) = work_list.pop() {
                if loop_blocks.insert(block) {
                    for pred in self.predecessors(block) {
                        if !loop_blocks.contains(&pred) {
                            work_list.push(pred);
                        }
                    }
                }
            }

            let mut exits = Vec::new();
            for &block in &loop_blocks {
                for succ in self.successors(block) {
                    if !loop_blocks.contains(&succ) {
                        exits.push(succ);
                    }
                }
            }

            let lp = Loop {
                header,
                blocks: loop_blocks.into_iter().collect(),
                exits,
                back_edges: vec![(tail, header)],
                nesting_level: 0,
            };
            self.loops.push(lp);
        }

        self.compute_loop_nesting();
    }

    fn compute_loop_nesting(&mut self) {
        let loop_count = self.loops.len();
        for i in 0..loop_count {
            let mut nesting = 0;
            for j in 0..loop_count {
                if i != j && self.loops[j].contains(self.loops[i].header) {
                    nesting += 1;
                }
            }
            self.loops[i].nesting_level = nesting;
        }
    }

    pub fn build_dominator_tree(&self) -> DominatorTree {
        let entry = self.entry.unwrap_or(Address::new(0));
        let mut tree = DominatorTree {
            root: entry,
            children: HashMap::new(),
            parent: HashMap::new(),
        };

        for (&addr, &dom) in &self.dominators {
            if addr != dom {
                tree.children.entry(dom).or_insert_with(Vec::new).push(Address::new(addr));
                tree.parent.insert(addr, Address::new(dom));
            }
        }

        tree
    }

    pub fn reverse_post_order(&self) -> Vec<Address> {
        let entry = match self.entry {
            Some(e) => e,
            None => return Vec::new(),
        };

        let mut visited = HashSet::new();
        let mut order = Vec::new();
        self.dfs_post_order(entry, &mut visited, &mut order);
        order.reverse();
        order
    }

    fn dfs_post_order(&self, addr: Address, visited: &mut HashSet<u64>, order: &mut Vec<Address>) {
        if !visited.insert(addr.as_u64()) {
            return;
        }

        for succ in self.successors(addr) {
            self.dfs_post_order(succ, visited, order);
        }

        order.push(addr);
    }

    pub fn find_natural_loops(&self) -> Vec<Vec<Address>> {
        self.loops.iter().map(|l| l.blocks.clone()).collect()
    }

    pub fn is_reducible(&self) -> bool {
        for block in self.blocks.values() {
            let preds = block.predecessors();
            if preds.len() <= 1 {
                continue;
            }

            let mut has_dominating_pred = false;
            for pred in preds {
                if self.dominates(*pred, block.start()) {
                    has_dominating_pred = true;
                    break;
                }
            }

            if !has_dominating_pred && preds.len() > 1 {
                return false;
            }
        }
        true
    }

    pub fn split_critical_edges(&mut self) {
        let mut edges_to_split = Vec::new();

        for block in self.blocks.values() {
            if block.successor_count() > 1 {
                for succ in block.successors() {
                    if let Some(succ_block) = self.blocks.get(&succ.as_u64()) {
                        if succ_block.predecessor_count() > 1 {
                            edges_to_split.push((block.start(), *succ));
                        }
                    }
                }
            }
        }

        for (from, to) in edges_to_split {
            let new_addr = Address::new(from.as_u64() | 0x8000000000000000);
            let new_block = BasicBlock::new(new_addr, new_addr + 4, Vec::new());
            self.add_block(new_block);

            if let Some(from_block) = self.blocks.get_mut(&from.as_u64()) {
                from_block.remove_successor(to);
                from_block.add_successor(new_addr);
            }

            if let Some(to_block) = self.blocks.get_mut(&to.as_u64()) {
                to_block.remove_predecessor(from);
                to_block.add_predecessor(new_addr);
            }

            if let Some(new_block) = self.blocks.get_mut(&new_addr.as_u64()) {
                new_block.add_predecessor(from);
                new_block.add_successor(to);
            }
        }
    }
}

impl Default for ControlFlowGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ControlFlowGraph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Control Flow Graph ({} blocks)", self.blocks.len())?;
        if let Some(entry) = self.entry {
            writeln!(f, "Entry: {:016x}", entry.as_u64())?;
        }
        writeln!(f, "Exits: {:?}", self.exits.iter().map(|e| format!("{:016x}", e.as_u64())).collect::<Vec<_>>())?;
        writeln!(f, "Loops: {}", self.loops.len())?;

        for block in self.blocks_sorted() {
            writeln!(f, "{}", block)?;
        }

        Ok(())
    }
}

impl Loop {
    pub fn header(&self) -> Address {
        self.header
    }

    pub fn blocks(&self) -> &[Address] {
        &self.blocks
    }

    pub fn exits(&self) -> &[Address] {
        &self.exits
    }

    pub fn back_edges(&self) -> &[(Address, Address)] {
        &self.back_edges
    }

    pub fn nesting_level(&self) -> usize {
        self.nesting_level
    }

    pub fn contains(&self, addr: Address) -> bool {
        self.blocks.iter().any(|&b| b == addr)
    }

    pub fn size(&self) -> usize {
        self.blocks.len()
    }

    pub fn is_single_exit(&self) -> bool {
        self.exits.len() == 1
    }
}

impl DominatorTree {
    pub fn root(&self) -> Address {
        self.root
    }

    pub fn children(&self, addr: Address) -> &[Address] {
        self.children.get(&addr.as_u64()).map(|v| v.as_slice()).unwrap_or(&[])
    }

    pub fn parent(&self, addr: Address) -> Option<Address> {
        self.parent.get(&addr.as_u64()).copied()
    }

    pub fn depth(&self, addr: Address) -> usize {
        let mut d = 0;
        let mut current = addr;
        while let Some(p) = self.parent(current) {
            d += 1;
            current = p;
        }
        d
    }

    pub fn lca(&self, a: Address, b: Address) -> Address {
        let mut ancestors_a = HashSet::new();
        let mut current = a;
        ancestors_a.insert(current);
        while let Some(p) = self.parent(current) {
            ancestors_a.insert(p);
            current = p;
        }

        current = b;
        if ancestors_a.contains(&current) {
            return current;
        }
        while let Some(p) = self.parent(current) {
            if ancestors_a.contains(&p) {
                return p;
            }
            current = p;
        }

        self.root
    }
}
