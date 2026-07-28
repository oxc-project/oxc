use std::{
    num::NonZeroU32,
    sync::atomic::{AtomicU32, Ordering},
};

/// Unique identification for a group.
///
/// See [crate::Formatter::group_id] on how to get a unique id.
///
/// The debug name lives in a side table (see `debug_names` below),
/// not an inline field, to keep the layout of [crate::FormatElement] (which embeds `GroupId` through `Tag`) identical in debug and release builds.
/// (See the size assertion in `format_element/mod.rs`.)
#[repr(transparent)]
#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub struct GroupId {
    value: NonZeroU32,
}

impl GroupId {
    #[cfg_attr(not(debug_assertions), expect(unused_variables))]
    fn new(value: NonZeroU32, debug_name: &'static str) -> Self {
        #[cfg(debug_assertions)]
        debug_names::record(value, debug_name);
        Self { value }
    }
}

impl std::fmt::Debug for GroupId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[cfg(debug_assertions)]
        if let Some(name) = debug_names::lookup(self.value) {
            return write!(f, "#{name}-{}", self.value);
        }
        write!(f, "#{}", self.value)
    }
}

/// Side table mapping `GroupId` values to their debug names.
///
/// Ids restart from 1 for every [`UniqueGroupIdBuilder`] (one per format run),
/// so concurrent or successive runs overwrite each other's entries,
/// the table is best-effort debug info for IR dumps, not authoritative.
#[cfg(debug_assertions)]
mod debug_names {
    use std::{num::NonZeroU32, sync::Mutex};

    /// Names indexed by `GroupId` value - 1
    static NAMES: Mutex<Vec<Option<&'static str>>> = Mutex::new(Vec::new());

    pub(super) fn record(value: NonZeroU32, name: &'static str) {
        let index = value.get() as usize - 1;
        let mut names = NAMES.lock().unwrap();
        if names.len() <= index {
            names.resize(index + 1, None);
        }
        names[index] = Some(name);
    }

    pub(super) fn lookup(value: NonZeroU32) -> Option<&'static str> {
        NAMES.lock().unwrap().get(value.get() as usize - 1).copied().flatten()
    }
}

impl From<GroupId> for u32 {
    fn from(id: GroupId) -> Self {
        id.value.get()
    }
}

/// Builder to construct unique group ids that are unique if created with the same builder.
pub struct UniqueGroupIdBuilder {
    next_id: AtomicU32,
}

impl UniqueGroupIdBuilder {
    /// Creates a new unique group id with the given debug name.
    ///
    /// # Panics
    ///
    /// Panics if the internal counter overflows `u32::MAX`.
    pub fn group_id(&self, debug_name: &'static str) -> GroupId {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let id = NonZeroU32::new(id).unwrap_or_else(|| panic!("Group ID counter overflowed"));

        GroupId::new(id, debug_name)
    }
}

impl Default for UniqueGroupIdBuilder {
    fn default() -> Self {
        UniqueGroupIdBuilder {
            // Start with 1 because `GroupId` wraps a `NonZeroU32` to reduce memory usage.
            next_id: AtomicU32::new(1),
        }
    }
}
