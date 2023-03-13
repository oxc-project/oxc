use crate::hasher::{extract_first_and_last_two_bytes, hash_u32};

#[derive(Debug)]
pub struct HashTable<'a>(Vec<Option<&'a str>>);

impl<'a> HashTable<'a> {
    pub fn empty(size: usize) -> Self {
        Self(vec![None; size])
    }

    pub fn try_generate(
        &mut self,
        keys: &[&'a str],
        seed: u32,
        hasher: impl Fn(&[u8], u32) -> u32,
    ) -> bool {
        self.0.fill(None);
        for s in keys {
            let slice = s.as_bytes();
            let hash_code = hasher(slice, seed);
            let idx = hash_code as usize % self.0.len();
            let slot = &mut self.0[idx];
            if slot.is_some() {
                return false;
            }
            *slot = Some(s);
        }
        true
    }
}

#[derive(Debug)]
pub struct HashTableMeta<'a> {
    pub size: usize,
    pub seed: u32,
    pub table: HashTable<'a>,
}

#[allow(clippy::all)]
pub fn generate_hash_table<'a>(keys: &[&'a str]) -> HashTableMeta<'a> {
    const ATTEMPTS: u32 = 50_000_000;
    const MAX_SIZE: usize = 1024;

    let keys: Vec<_> = keys.iter().copied().collect();
    let mut seed = 0x0070_ABCA;
    let mut size = keys.len().next_power_of_two();
    while size <= MAX_SIZE {
        let mut table = HashTable::empty(size);
        for _ in 0..ATTEMPTS {
            if table.try_generate(&keys, seed, |slice, seed| unsafe {
                let selection = extract_first_and_last_two_bytes(slice);
                hash_u32(selection, seed)
            }) {
                return HashTableMeta { size, seed, table };
            }
            seed += 1;
        }
        size *= 2;
    }

    panic!("Failed to generate Hash Table.")
}
