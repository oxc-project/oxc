//! Project-level coordination for cross-file type checking.
//!
//! This crate provides the `Project` struct, which sits between the
//! file system and the checker. It manages global types from lib.d.ts,
//! resolves module specifiers, and will eventually own per-file semantic
//! data.
//!
//! Mirrors tsgo's `Program` interface (defined in `checker/checker.go`,
//! implemented in `compiler/`).
//!
//! # Type Arena Ownership
//!
//! The caller owns the `TypeArena`. The Project borrows it mutably
//! during construction (to allocate global types), then doesn't need
//! mutable access again. The Checker borrows it mutably during checking.
//! All TypeIds (global + per-file) live in one arena.
//!
//! ```text
//! let mut arena = TypeArena::with_capacity(64);
//! let project = Project::new(&mut arena);  // allocates globals
//! // ... arena is free to borrow again ...
//! let checker = Checker::new_with_host(semantic, &mut arena, &project);
//! ```
//!
//! ## Future parallelism note
//!
//! For parallel per-file checking, the single `&mut TypeArena` will be
//! replaced with per-thread arenas + a merge step (see
//! checker_architecture.md §4). The `Project::check_file()` API (once
//! added) will encapsulate this so callers don't need to change.

use oxc_checker::{CheckerHost, GlobalTypes, allocate_intrinsics};
use oxc_span::CompactStr;
use oxc_types::{TypeArena, TypeId};
use rustc_hash::FxHashMap;

/// Project-level state for cross-file type checking.
///
/// Owns the global type map (not the arena — the caller owns that).
/// Implements `CheckerHost` so checkers can query global types and
/// resolve cross-file imports.
// NOTE: v1 uses eager loading and a single caller-owned arena.
// Future optimization paths:
// - Lazy file loading (OnceCell per file) — doesn't affect this struct
// - Per-thread arenas for parallel checking — Project would own the
//   merge logic and hand out per-thread arenas
pub struct Project {
    /// Global types extracted from lib.d.ts.
    /// TypeIds point into the caller-owned TypeArena.
    global_types: FxHashMap<CompactStr, TypeId>,
}

impl Project {
    /// Create a new project, parsing lib.d.ts for global types.
    ///
    /// Global types are allocated into `arena`. After this call returns,
    /// the arena is free to be borrowed by checkers. The TypeIds stored
    /// in `global_types` remain valid as long as the arena is alive.
    pub fn new(arena: &TypeArena) -> Self {
        let intrinsics = allocate_intrinsics(arena);
        let globals = GlobalTypes::from_lib(arena, &intrinsics);
        Self {
            global_types: globals.types,
        }
    }
}

impl CheckerHost for Project {
    fn get_global_type(&self, name: &str) -> Option<TypeId> {
        self.global_types.get(name).copied()
    }

    fn resolve_import(
        &self,
        _from_file: &str,
        _module_specifier: &str,
        _export_name: &str,
    ) -> Option<TypeId> {
        // TODO: Phase 3-4 — module resolution + cross-file type queries
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_loads_global_types() {
        let arena = TypeArena::with_capacity(64);
        let project = Project::new(&arena);
        let _ = project.get_global_type("Array");
    }

    #[test]
    fn project_implements_checker_host() {
        let arena = TypeArena::with_capacity(64);
        let project = Project::new(&arena);
        let host: &dyn CheckerHost = &project;
        assert!(host.resolve_import("test.ts", "./foo", "x").is_none());
    }
}
