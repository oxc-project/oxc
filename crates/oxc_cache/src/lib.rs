//! Caching for Oxc's command-line tools: remember what a run read and found, so a later run
//! only redoes what changed.
//!
//! - [`Digest`]: BLAKE3 of file contents, and of the values keys are made of.
//! - [`Observation`]: a `stat` and when it was taken, trusted only outside its racy window.
//! - [`Store`]: content-addressed keys, bounded by generations ([`Retention`]).
//! - [`Cache`]: per-file results kept by `stat` or by content ([`Strategy`]), one of them
//!   shared through a [`Store`].
//! - [`CacheDir`]: where it's kept, checksummed and written atomically.
//! - [`BuildId`]: the running binary, so one build's results are never used by another.
//!
//! What to cache, under which keys, and where, is the caller's: these are the mechanisms.
//!
//! A cache never changes an answer by failing: anything unreadable is a miss,
//! anything unwritable is skipped.

mod build;
mod cache;
mod digest;
mod dir;
mod stat;
mod store;

pub use build::BuildId;
pub use cache::{Cache, Entry, Strategy};
pub use digest::Digest;
pub use dir::CacheDir;
pub use stat::{Observation, Stat, Timestamp};
pub use store::{Generation, Retention, Store};
