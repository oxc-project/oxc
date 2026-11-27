# Oxc Cache

Building blocks for caching in command-line tools: remember what a run found for each file, so a later run only redoes the files that changed.

## Overview

A tool like `oxfmt --cache` needs to answer two questions quickly and correctly: _has this file changed since last time?_ and _have I already seen this exact input?_ This crate provides the mechanisms for both, and leaves the policy to the tool: what to cache, under which keys, where, and for how long.

A cache built on it never changes an answer by failing. Anything missing, corrupt or unreadable is a miss, and anything unwritable is skipped.

## Key Features

- **`Cache` and `Strategy`**: per-file results kept across runs. With `Strategy::Metadata`, a file whose `stat` is unchanged keeps its result without being read. With `Strategy::Content`, every file is read, and keeps a result put for the same content. Every result is also kept in a `Store` by key and content, for any cache in the same directory
- **`Digest`**: BLAKE3 of file contents, and of any `Serialize` value used as a key
- **`Stat` and `Observation`**: a file's identity (size, mtime, ctime, inode, device, mode; on Windows, size, mtime, creation time and attributes) and when it was taken, trusted only outside Git's racy window
- **`Store` and `Retention`**: a content-addressed set of keys, bounded by generations and size
- **`CacheDir`**: `postcard` files behind a BLAKE3 checksum, written atomically
- **`BuildId`**: the digest of the running binary, so results from one build are never used by another

## Usage

```rust
use std::{fs, path::Path, sync::Arc};

use oxc_cache::{Cache, CacheDir, Digest, Stat};

/// Whether `path` passes a check, answered from the cache when possible.
fn check(cache_dir: &Path, path: &Path, options: &str) -> Option<bool> {
    let dir = Arc::new(CacheDir::open(cache_dir)?);
    let cache = Cache::open(dir, "files");

    // Everything the result depends on besides the file's path and content.
    let key = Digest::of(&options);
    let stat = Stat::of_path(path)?;

    let passed = if let Some(passed) = cache.get_by_stat(path, key, stat) {
        passed // Unchanged: not even read.
    } else {
        let source = fs::read(path).ok()?;
        let content = Digest::of_bytes(&source);
        if let Some(passed) = cache.get_by_content(path, key, stat, content) {
            passed // Touched, changed back, or checked by another cache in the directory.
        } else {
            let passed = !source.is_empty(); // The actual work.
            cache.put(path, key, stat, content, passed);
            passed
        }
    };
    cache.save();
    Some(passed)
}
```

A result that can't be keyed by everything it depends on (one that read plugins or other files while computing it) isn't put.

## Architecture

### Trusting a `stat`

Each recorded file is compared with its own earlier `stat`, not with when the last run happened (as Make does), so a tool that sets an old mtime can't hide a change. On Unix, `ctime` is part of the identity, and no tool can set it. Windows has no `ctime` in stable Rust, so a same-size write that puts the mtime back goes unseen there. A `stat` taken within 2 s of the file's own timestamps is never trusted, because a write in the same timestamp tick wouldn't show (the same rule as Git's racy index). A file whose `stat` changed is hashed, and holds if its content didn't change.

### Per-file results

A `Cache` holds the last run's entries, and collects this run's from any thread. An entry is a file's path, the caller's key (the tool's build and options), the `stat` it was read with, its content's digest, and the caller's result. A lookup only answers with an entry for the same key. `get_by_stat` needs the same `stat`, outside the racy window, and is never used with `Strategy::Content`. `get_by_content` needs the same content, or finds a result any cache in the directory put in the `Store` for the same key, path and content. A path that isn't UTF-8 is never cached. `save` writes the entries only if they changed, so a run that changed nothing writes nothing.

### Bounding the store

Each `Store::commit` is a generation, and each key carries the newest generation that used it. Keys no run of the last `Retention::generations` used are dropped, then the oldest ones while the store is over `Retention::max_bytes`. A key every run uses is restamped only once in a while, so an unchanged store isn't rewritten on every run.

### Testing

Property tests (`quickcheck`) check the invariants:

- with either strategy, any answer from a file's `stat`, its content or the store is what computing would give, through writes, same-size writes with the mtime put back and runs with other keys; `Strategy::Content` never answers from a `stat`;
- the store keeps every key used within its retention, and only keys it was given;
- any flipped byte or truncation of a cache file reads as nothing;
- different values never share a digest.
