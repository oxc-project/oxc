use std::{
    any::{Any, TypeId},
    sync::Arc,
};

use rustc_hash::FxBuildHasher;

/// State shared by every file linted in one run, for rules that memoize work on the module graph.
///
/// The runtime creates one per run and hands it to every file, so nothing stored here outlives
/// the module records it refers to or leaks between the language server's runs over an edited
/// graph. [`Linter::run`](crate::Linter::run) gives each file its own, which shares nothing.
///
/// Slots are keyed by type. Rule options that change what a rule computes must be part of the
/// key inside that type, since nested configs can run one rule with several configurations.
#[derive(Default)]
pub struct CrossFileCache {
    slots: papaya::HashMap<TypeId, Arc<dyn Any + Send + Sync>, FxBuildHasher>,
}

impl CrossFileCache {
    /// The slot for `T`, created on first access.
    pub fn get_or_default<T: Default + Send + Sync + 'static>(&self) -> Arc<T> {
        let slots = self.slots.pin();
        let slot = slots.get_or_insert_with(TypeId::of::<T>(), || {
            Arc::new(T::default()) as Arc<dyn Any + Send + Sync>
        });
        // Only a `T` is ever stored under `TypeId::of::<T>()`.
        Arc::clone(slot).downcast().unwrap_or_else(|_| unreachable!())
    }
}
