use std::hash::Hasher;

use hashbrown::hash_table::{Drain, HashTable};
use rustc_hash::FxHasher;

use oxc_span::CompactStr;

use crate::reference::ReferenceId;

#[derive(Clone, Copy, PartialEq)]
pub struct IdentifierHash(u64);

impl IdentifierHash {
    fn new(name: &str) -> Self {
        let mut hasher = FxHasher::default();
        hasher.write(name.as_bytes());
        Self(hasher.finish())
    }
}

pub struct Line {
    pub name: CompactStr,
    pub hash: IdentifierHash,
    pub reference_ids: Vec<ReferenceId>,
}

#[derive(Default)]
pub struct TempUnresolvedReferences {
    inner: HashTable<Line>,
}

impl TempUnresolvedReferences {
    #[allow(dead_code)]
    pub fn get(&self, name: &str, hash: IdentifierHash) -> Option<&Vec<ReferenceId>> {
        self.inner
            .find(hash.0, |line| line.hash == hash && line.name.as_ref() == name)
            .map(|entry| &entry.reference_ids)
    }

    #[allow(dead_code)]
    pub fn get_mut(&mut self, name: &str, hash: IdentifierHash) -> Option<&mut Vec<ReferenceId>> {
        self.inner
            .find_mut(hash.0, |line| line.hash == hash && line.name.as_ref() == name)
            .map(|entry| &mut entry.reference_ids)
    }

    pub fn insert(&mut self, name: CompactStr, reference_id: ReferenceId) {
        let hash = IdentifierHash::new(&name);
        if let Some(line) = self.inner.find_mut(hash.0, |line| line.name == name) {
            line.reference_ids.push(reference_id);
        } else {
            self.inner.insert_unique(
                hash.0,
                Line { name, hash, reference_ids: vec![reference_id] },
                |line| line.hash.0,
            );
        }
    }

    pub fn extend(
        &mut self,
        name: CompactStr,
        hash: IdentifierHash,
        reference_ids: Vec<ReferenceId>,
    ) {
        if let Some(line) = self.inner.find_mut(hash.0, |line| line.name == name) {
            line.reference_ids.extend(reference_ids);
        } else {
            self.inner
                .insert_unique(hash.0, Line { name, hash, reference_ids }, |line| line.hash.0);
        }
    }

    pub fn drain(&mut self) -> Drain<Line> {
        self.inner.drain()
    }
}
