use std::{
    path::Path,
    sync::{Arc, Condvar, Mutex},
};

use dashmap::{mapref::one::Ref, DashMap};
use oxc_semantic::ModuleRecord;
use rustc_hash::FxHashMap;
use std::num::NonZeroUsize;

/// `CacheState` and `CacheStateEntry` are used to fix the problem where
/// there is a brief moment when a concurrent fetch can miss the cache.
///
/// Given `ModuleMap` is a `DashMap`, which conceptually is a `RwLock<HashMap>`.
/// When two requests read the map at the exact same time from different threads,
/// both will miss the cache so both thread will make a request.
///
/// See the "problem section" in <https://medium.com/@polyglot_factotum/rust-concurrency-patterns-condvars-and-locks-e278f18db74f>
/// and the solution is copied here to fix the issue.
type CacheState = Mutex<FxHashMap<Box<Path>, Arc<(Mutex<CacheStateEntry>, Condvar)>>>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CacheStateEntry {
    ReadyToConstruct,
    PendingStore(NonZeroUsize),
}

/// Keyed by canonicalized path
type ModuleMap = DashMap<Box<Path>, ModuleState>;

#[derive(Clone)]
pub(super) enum ModuleState {
    Resolved(Arc<ModuleRecord>),
    Ignored,
}

#[derive(Default)]
pub(super) struct ModuleCache {
    cache_state: Arc<CacheState>,
    modules: ModuleMap,
}

impl ModuleCache {
    #[inline]
    pub fn get(&self, path: &Path) -> Option<Ref<'_, Box<Path>, ModuleState>> {
        self.modules.get(path)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.modules.len()
    }

    pub(super) fn init_cache_state(&self, path: &Path) -> bool {
        let (lock, cvar) = {
            let mut state_map = self.cache_state.lock().expect("Failed to lock cache state");
            &*Arc::clone(state_map.entry(path.to_path_buf().into_boxed_path()).or_insert_with(
                || Arc::new((Mutex::new(CacheStateEntry::ReadyToConstruct), Condvar::new())),
            ))
        };
        let mut state = cvar
            .wait_while(lock.lock().expect("Failed lock inner cache state"), |state| {
                matches!(*state, CacheStateEntry::PendingStore(_))
            })
            .unwrap();

        let cache_hit = if self.modules.contains_key(path) {
            true
        } else {
            let i = if let CacheStateEntry::PendingStore(i) = *state { i.get() } else { 0 };
            // SAFETY: 1 + any natural number is always non-zero.
            *state = CacheStateEntry::PendingStore(unsafe { NonZeroUsize::new_unchecked(i + 1) });
            false
        };

        if *state == CacheStateEntry::ReadyToConstruct {
            cvar.notify_one();
        }

        drop(state);
        cache_hit
    }

    /// # Panics
    /// If a cache entry for `path` does not exist. You must call `init_cache_state` first.
    pub(super) fn add_resolved_module(&self, path: &Path, module_record: Arc<ModuleRecord>) {
        self.modules
            .insert(path.to_path_buf().into_boxed_path(), ModuleState::Resolved(module_record));

        self.update_cache_state(path);
    }

    /// # Panics
    /// If a cache entry for `path` does not exist. You must call `init_cache_state` first.
    pub(super) fn ignore_path(&self, path: &Path) {
        self.modules.insert(path.to_path_buf().into_boxed_path(), ModuleState::Ignored);
        self.update_cache_state(path);
    }

    /// # Panics
    /// If a cache entry for `path` does not exist. You must call `init_cache_state` first.
    fn update_cache_state(&self, path: &Path) {
        let (lock, cvar) = {
            let mut state_map = self.cache_state.lock().expect("Failed to lock cache state");
            &*Arc::clone(
                state_map
                    .get_mut(path)
                    .expect("Entry in http-cache state to have been previously inserted"),
            )
        };
        let mut state = lock.lock().expect("Failed lock inner cache state");
        if let CacheStateEntry::PendingStore(i) = *state {
            let new = i.get() - 1;
            if new == 0 {
                *state = CacheStateEntry::ReadyToConstruct;
                // Notify the next thread waiting in line, if there is any.
                cvar.notify_one();
            } else {
                // SAFETY: new is never 0 because the previous branch checks for it.
                *state = CacheStateEntry::PendingStore(unsafe { NonZeroUsize::new_unchecked(new) });
            }
        }
    }
}
