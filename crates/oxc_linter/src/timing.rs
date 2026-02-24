use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use rustc_hash::FxHashMap;

/// Accumulates per-rule execution timings across files.
///
/// Files accumulate timings locally (no locking per rule call), then merge
/// into this shared store once per file.
#[derive(Debug, Default)]
pub struct TimingStore {
    inner: Mutex<FxHashMap<String, (Duration, u64)>>,
    /// Overhead durations not attributable to specific rules (e.g. tsgolint
    /// program load, JS plugin bridge).  Keyed by a human-readable label.
    overhead: Mutex<FxHashMap<String, Duration>>,
}

impl TimingStore {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// Merge per-file timing data. Called once per file.
    pub(crate) fn merge(&self, local: FxHashMap<String, (Duration, u64)>) {
        let mut inner = self.inner.lock().unwrap();
        for (name, (dur, count)) in local {
            let entry = inner.entry(name).or_default();
            entry.0 += dur;
            entry.1 += count;
        }
    }

    /// Collect all timing data sorted by total duration descending.
    pub fn collect(&self) -> Vec<(String, Duration, u64)> {
        let inner = self.inner.lock().unwrap();
        let mut entries: Vec<(String, Duration, u64)> =
            inner.iter().map(|(name, &(dur, count))| (name.clone(), dur, count)).collect();
        entries.sort_unstable_by(|a, b| b.1.cmp(&a.1));
        entries
    }

    /// Record an overhead duration (bridge/setup cost, not a specific rule).
    /// Zero durations are ignored.
    pub(crate) fn record_overhead(&self, name: &str, dur: Duration) {
        if dur.is_zero() {
            return;
        }
        *self.overhead.lock().unwrap().entry(name.to_owned()).or_default() += dur;
    }

    /// Collect overhead entries sorted by duration descending.
    pub fn collect_overhead(&self) -> Vec<(String, Duration)> {
        let overhead = self.overhead.lock().unwrap();
        let mut entries: Vec<(String, Duration)> =
            overhead.iter().map(|(name, &dur)| (name.clone(), dur)).collect();
        entries.sort_unstable_by(|a, b| b.1.cmp(&a.1));
        entries
    }
}
