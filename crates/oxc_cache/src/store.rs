//! A content-addressed store of results, bounded by generations.
//!
//! A generation is a run that writes the store. Each key carries the newest generation that
//! used it; writing drops the keys no run of the last [`Retention::generations`] used, then the
//! oldest generations while the store is over [`Retention::max_bytes`].

use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

use crate::digest::Digest;

/// What an entry costs besides its value.
const ENTRY_BYTES: usize = size_of::<Digest>() + size_of::<u32>();

/// How long a [`Store`] keeps keys, and how big it may grow.
#[derive(Debug, Clone, Copy)]
pub struct Retention {
    /// Keys no run of this many generations used are dropped.
    pub generations: u32,
    /// Past this size, the oldest generations' keys are dropped (never the newest's).
    pub max_bytes: usize,
}

impl Default for Retention {
    fn default() -> Self {
        Self { generations: 128, max_bytes: 64 << 20 }
    }
}

impl Retention {
    /// A key used within this many generations isn't stamped again, so a key every run uses is
    /// written once per `slack` runs. Collection keeps `generations + slack` generations, so a
    /// key used in one of the last `generations` is never dropped.
    fn slack(self) -> u32 {
        self.generations / 4
    }
}

/// What a run used and found, to commit to a [`Store`] as its next generation.
#[derive(Debug, Default)]
pub struct Generation {
    /// Keys the run answered from.
    pub used: Vec<Digest>,
    /// Keys the run found, with their values.
    pub added: Vec<(Digest, Box<[u8]>)>,
}

impl Generation {
    /// Returns whether the run used and found nothing.
    pub fn is_empty(&self) -> bool {
        self.used.is_empty() && self.added.is_empty()
    }
}

/// A value, and the newest generation that used it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Kept {
    /// The newest generation that added or used it.
    stamp: u32,
    /// The result, as the caller serialized it.
    value: Box<[u8]>,
}

/// Content-addressed values, bounded by generations.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Store {
    /// The newest generation written.
    generation: u32,
    /// The values, by key.
    keys: FxHashMap<Digest, Kept>,
}

impl Store {
    /// Returns the value kept for `key`.
    pub fn get(&self, key: &Digest) -> Option<&[u8]> {
        self.keys.get(key).map(|kept| &*kept.value)
    }

    /// Merges `generation` into the store as its next one, and collects by `retention`.
    /// Returns whether anything changed: whether to write it.
    pub fn commit(&mut self, generation: &Generation, retention: Retention) -> bool {
        let Generation { used, added } = generation;
        let next = self.generation.wrapping_add(1);
        let stale = next.saturating_sub(retention.slack());
        let mut changed = false;
        for (key, value) in added {
            if self.keys.get(key).is_none_or(|kept| kept.value != *value) {
                self.keys.insert(*key, Kept { stamp: next, value: value.clone() });
                changed = true;
            }
        }
        for key in used.iter().chain(added.iter().map(|(key, _)| key)) {
            if let Some(kept) = self.keys.get_mut(key)
                && kept.stamp < stale
            {
                kept.stamp = next;
                changed = true;
            }
        }
        if changed {
            self.generation = next;
            self.collect(retention);
        }
        changed
    }

    /// Drop keys older than the kept generations, then the oldest generations while over the
    /// size cap, always keeping the newest generation's keys.
    fn collect(&mut self, retention: Retention) {
        let newest = self.generation;
        let oldest_kept = newest.saturating_sub(retention.generations + retention.slack());
        self.keys.retain(|_, kept| kept.stamp > oldest_kept);
        while self.bytes() > retention.max_bytes {
            let Some(oldest) = self.keys.values().map(|kept| kept.stamp).min() else { break };
            if oldest >= newest {
                break;
            }
            self.keys.retain(|_, kept| kept.stamp > oldest);
        }
    }

    /// Roughly what the store costs, to bound by [`Retention::max_bytes`].
    fn bytes(&self) -> usize {
        self.keys.values().map(|kept| ENTRY_BYTES + kept.value.len()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RETENTION: Retention = Retention { generations: 128, max_bytes: 64 << 20 };
    const KEEP: u32 = RETENTION.generations;
    const SLACK: u32 = KEEP / 4;
    const MAX_BYTES: usize = RETENTION.max_bytes;

    fn key(n: u32) -> Digest {
        Digest::of(&n)
    }

    fn store(generation: u32, entries: &[(u32, u32)]) -> Store {
        let kept = |stamp| Kept { stamp, value: Box::default() };
        Store {
            generation,
            keys: entries.iter().map(|&(k, stamp)| (key(k), kept(stamp))).collect(),
        }
    }

    fn stamp(store: &Store, k: u32) -> Option<u32> {
        store.keys.get(&key(k)).map(|kept| kept.stamp)
    }

    fn added(keys: impl IntoIterator<Item = Digest>) -> Vec<(Digest, Box<[u8]>)> {
        keys.into_iter().map(|k| (k, Box::default())).collect()
    }

    #[test]
    fn nothing_new_writes_nothing() {
        let mut store = store(10, &[(1, 10)]);
        assert!(!store.commit(&Generation { used: vec![key(1)], added: vec![] }, RETENTION));
        assert!(
            !store.commit(&Generation { used: vec![], added: added([key(1)]) }, RETENTION),
            "a key already kept and fresh isn't new"
        );
        assert_eq!(stamp(&store, 1), Some(10));
    }

    #[test]
    fn a_key_used_within_keep_survives_and_older_ones_go() {
        // Committing makes generation `next`: keys stamped `next - KEEP - SLACK` or older go.
        let g = KEEP + SLACK + 10;
        let next = g + 1;
        let mut store = store(g, &[(1, next - KEEP - SLACK), (2, next - KEEP - SLACK + 1), (3, g)]);
        assert!(store.commit(&Generation { used: vec![], added: added([key(4)]) }, RETENTION));
        assert_eq!(stamp(&store, 1), None, "older than keep + slack");
        assert_eq!(stamp(&store, 2), Some(next - KEEP - SLACK + 1));
        assert_eq!(stamp(&store, 4), Some(next));
    }

    #[test]
    fn a_stale_key_used_is_stamped_again() {
        let mut store = store(100, &[(1, 100 - SLACK - 1), (2, 100)]);
        assert!(store.commit(&Generation { used: vec![key(1), key(2)], added: vec![] }, RETENTION));
        assert_eq!(stamp(&store, 1), Some(101));
        assert_eq!(stamp(&store, 2), Some(100), "used within the slack: not rewritten");
    }

    #[test]
    fn over_the_cap_drops_oldest_generations_but_never_the_newest() {
        let n = u32::try_from(MAX_BYTES / ENTRY_BYTES).unwrap();
        let old: Vec<(u32, u32)> = (0..n).map(|k| (k, 50)).collect();
        let mut store = store(60, &old);
        let new: Vec<Digest> = (n..n + 10).map(key).collect();
        assert!(store.commit(&Generation { used: vec![], added: added(new.clone()) }, RETENTION));
        assert!(store.keys.len() <= n as usize);
        assert!(new.iter().all(|k| store.get(k).is_some()));
        assert!(store.get(&key(0)).is_none());
    }

    #[test]
    fn a_value_is_kept_with_its_key_and_counts_toward_the_cap() {
        let mut store = Store::default();
        let value: Box<[u8]> = vec![7; 100].into();
        let retention = Retention { generations: 8, max_bytes: 3 * (ENTRY_BYTES + 100) };
        for k in 0..5 {
            store.commit(
                &Generation { used: vec![], added: vec![(key(k), value.clone())] },
                retention,
            );
        }
        assert_eq!(store.get(&key(4)), Some(&*value));
        assert!(store.bytes() <= retention.max_bytes, "the oldest go once the values pass the cap");
    }

    const KEYS: u8 = 64;

    /// A run: the keys it answered from, and those it found.
    #[derive(Debug, Clone)]
    struct Commit {
        used: Vec<u8>,
        added: Vec<u8>,
    }

    use quickcheck::Arbitrary;

    impl Arbitrary for Commit {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            // A small key space, so keys are used again and again.
            let keys = |g: &mut quickcheck::Gen| -> Vec<u8> {
                let n = usize::arbitrary(g) % 4;
                std::iter::repeat_with(|| u8::arbitrary(g) % KEYS).take(n).collect()
            };
            Self { used: keys(g), added: keys(g) }
        }
    }

    /// For any sequence of runs: a key is kept while it was added or used (while kept) within
    /// the last `KEEP` generations, a key never added is never kept, and a commit that says
    /// nothing changed changed nothing.
    #[test]
    fn keeps_every_key_used_within_keep_generations() {
        #[expect(clippy::needless_pass_by_value, reason = "quickcheck passes values")]
        fn property(runs: Vec<Commit>, skips: Vec<u8>) -> bool {
            let mut store = Store::default();
            // The generation each key was last added, or used while kept.
            let mut touched: FxHashMap<u8, u32> = FxHashMap::default();
            for (i, run) in runs.iter().enumerate() {
                // Runs writing in between (other run shapes) age keys too.
                store.generation += u32::from(skips.get(i).copied().unwrap_or(0) % 48);
                let used: Vec<u8> = run
                    .used
                    .iter()
                    .copied()
                    .filter(|k| store.get(&key(u32::from(*k))).is_some())
                    .collect();
                let before = store.keys.clone();
                let generation = store.generation;
                let changed = store.commit(
                    &Generation {
                        used: run.used.iter().map(|k| key(u32::from(*k))).collect(),
                        added: added(run.added.iter().map(|k| key(u32::from(*k)))),
                    },
                    RETENTION,
                );
                if !changed && (store.keys != before || store.generation != generation) {
                    return false;
                }
                for k in used.iter().chain(&run.added) {
                    touched.insert(*k, store.generation);
                }
                for k in 0..KEYS {
                    let kept = store.get(&key(u32::from(k))).is_some();
                    let recent = touched.get(&k).is_some_and(|g| store.generation - g < KEEP);
                    if (recent && !kept) || (kept && !touched.contains_key(&k)) {
                        return false;
                    }
                }
            }
            true
        }
        quickcheck::QuickCheck::new()
            .tests(500)
            .rng(quickcheck::Gen::new(400))
            .quickcheck(property as fn(Vec<Commit>, Vec<u8>) -> bool);
    }
}
