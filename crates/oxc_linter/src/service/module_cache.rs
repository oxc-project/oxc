use std::{
    path::Path,
    sync::{Arc, Condvar, Mutex},
};

use dashmap::DashMap;
use oxc_semantic::ModuleRecord;
use rustc_hash::FxHashMap;

/// `CacheState` and `CacheStateEntry` are used to fix the problem where
/// there is a brief moment when a concurrent fetch can miss the cache.
///
/// Given `ModuleMap` is a `DashMap`, which conceptually is a `RwLock<HashMap>`.
/// When two requests read the map at the exact same time from different threads,
/// both will miss the cache so both thread will make a request.
///
/// See the "problem section" in <https://medium.com/@polyglot_factotum/rust-concurrency-patterns-condvars-and-locks-e278f18db74f>
/// and the solution is copied here to fix the issue.
pub(super) type CacheState = Mutex<FxHashMap<Box<Path>, Arc<(Mutex<CacheStateEntry>, Condvar)>>>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum CacheStateEntry {
    ReadyToConstruct,
    PendingStore(usize),
}

/// Keyed by canonicalized path
pub(super) type ModuleMap = DashMap<Box<Path>, ModuleState>;

#[derive(Clone)]
pub(super) enum ModuleState {
    Resolved(Arc<ModuleRecord>),
    Ignored,
}
