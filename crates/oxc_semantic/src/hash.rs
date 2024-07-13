use std::hash::Hasher;

use hashbrown::hash_table::{Drain, Entry, HashTable};
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
        self.inner.find(hash.0, |line| line.name.as_ref() == name).map(|entry| &entry.reference_ids)
    }

    #[allow(dead_code)]
    pub fn get_mut(&mut self, name: &str, hash: IdentifierHash) -> Option<&mut Vec<ReferenceId>> {
        self.inner
            .find_mut(hash.0, |line| line.name.as_ref() == name)
            .map(|entry| &mut entry.reference_ids)
    }

    pub fn insert(&mut self, name: CompactStr, reference_id: ReferenceId) {
        let hash = IdentifierHash::new(&name);
        let entry = self.inner.entry(hash.0, |line| line.name == name, |entry| entry.hash.0);
        match entry {
            Entry::Occupied(mut entry) => {
                entry.get_mut().reference_ids.push(reference_id);
            }
            Entry::Vacant(entry) => {
                entry.insert(Line { name, hash, reference_ids: vec![reference_id] });
            }
        }
    }

    pub fn extend(
        &mut self,
        name: CompactStr,
        hash: IdentifierHash,
        reference_ids: Vec<ReferenceId>,
    ) {
        let entry = self.inner.entry(hash.0, |line| line.name == name, |line| line.hash.0);
        match entry {
            Entry::Occupied(mut entry) => {
                entry.get_mut().reference_ids.extend(reference_ids);
            }
            Entry::Vacant(entry) => {
                entry.insert(Line { name, hash, reference_ids });
            }
        }
    }

    pub fn drain(&mut self) -> Drain<Line> {
        self.inner.drain()
    }
}
