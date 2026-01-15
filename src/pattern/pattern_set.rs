// Tue Jan 13 2026 - Alex

use crate::pattern::{Signature, PatternError};
use std::collections::HashMap;

pub struct PatternSet {
    signatures: HashMap<String, Signature>,
}

impl PatternSet {
    pub fn new() -> Self {
        Self {
            signatures: HashMap::new(),
        }
    }

    pub fn add(&mut self, signature: Signature) {
        self.signatures.insert(signature.name().to_string(), signature);
    }

    pub fn get(&self, name: &str) -> Option<&Signature> {
        self.signatures.get(name)
    }

    pub fn remove(&mut self, name: &str) -> Option<Signature> {
        self.signatures.remove(name)
    }

    pub fn contains(&self, name: &str) -> bool {
        self.signatures.contains_key(name)
    }

    pub fn len(&self) -> usize {
        self.signatures.len()
    }

    pub fn is_empty(&self) -> bool {
        self.signatures.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Signature> {
        self.signatures.values()
    }

    pub fn names(&self) -> impl Iterator<Item = &String> {
        self.signatures.keys()
    }
}

impl Default for PatternSet {
    fn default() -> Self {
        Self::new()
    }
}
