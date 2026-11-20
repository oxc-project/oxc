use std::ptr::NonNull;

use napi::JsString;

use oxc_allocator::Allocator;
use oxc_data_structures::types::implements;
use oxc_parser::Source;
use oxc_span::SourceType;

/// Source text in its own [`Allocator`], which can be sent to another thread.
///
/// A JS string can only be read on main thread. An async task does its work on another thread.
/// A [`Source`] borrows its [`Allocator`], so the two cannot be moved into a task together.
///
/// `SendableSource` owns the [`Allocator`], and holds the raw parts of the [`Source`],
/// so it can be moved into a task.
///
/// 1. On main thread, create a `SendableSource` with [`SendableSource::from_napi_js_string`].
///    This copies the JS string directly into the [`Allocator`].
/// 2. Move the `SendableSource` into the task, in an `Option`.
/// 3. In the task's `compute` method, take the `SendableSource` out of the `Option`,
///    and get the [`Source`] with [`SendableSource::source`].
///
/// Taking the `SendableSource` out of the task means it's dropped at the end of `compute`,
/// so the [`Allocator`]'s memory is freed on the other thread. If it was left in the task,
/// it'd be freed on main thread, when the task is dropped.
pub struct SendableSource {
    /// Pointer to start of source text, within `allocator`'s arena.
    ptr: NonNull<u8>,
    /// Length of source text in bytes.
    len: usize,
    /// Allocator which source text is stored in.
    allocator: Allocator,
}

const _: () = assert!(implements!(Allocator: Send));

// SAFETY: `Allocator` is `Send` (checked above). `ptr` is not, but it points into
// `allocator`'s arena, and nothing else has a pointer or reference to that memory.
// `SendableSource` owns both, so they move to another thread together.
unsafe impl Send for SendableSource {}

impl SendableSource {
    /// Create a [`SendableSource`] from a JS string.
    ///
    /// The string is converted to UTF-8, and written directly into a new [`Allocator`].
    ///
    /// # Errors
    ///
    /// Returns [`napi::Error`] if `js_string` is not a JS string.
    pub fn from_napi_js_string(js_string: &JsString<'_>) -> napi::Result<Self> {
        let allocator = Allocator::default();

        // `SourceType` is a placeholder. The real `SourceType` is passed to `source`.
        let source = Source::from_napi_js_string(js_string, SourceType::script(), &allocator)?;
        let (ptr, len, ..) = source.into_raw_parts();

        Ok(Self { ptr, len, allocator })
    }

    /// Get the [`Source`], to be parsed as `source_type`.
    ///
    /// The [`Source`] is in this `SendableSource`'s [`Allocator`].
    /// Use [`Source::allocator`] to get the [`Allocator`].
    pub fn source(&mut self, source_type: SourceType) -> Source<'_> {
        // SAFETY: `ptr` and `len` are the raw parts of a `Source` which was created
        // in `self.allocator`. So they describe a region of `len` initialized bytes
        // followed by padding, in that allocator's arena.
        //
        // That `Source` was consumed by `into_raw_parts`. Any `Source` returned by a previous call
        // to this method, and anything which borrows from it, borrowed `self` mutably,
        // so cannot exist now. So nothing has a pointer or reference to the region.
        //
        // `ptr` is private, so the region is only accessed via the `Source` returned here.
        // The source text was created from a JS string, so it's valid UTF-8.
        unsafe { Source::from_raw_parts(self.ptr, self.len, true, source_type, &self.allocator) }
    }
}
