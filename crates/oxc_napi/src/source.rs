use std::{
    alloc::Layout,
    ptr::{self, NonNull},
    slice, str,
};

use napi::{
    bindgen_prelude::FromNapiValue,
    check_status,
    sys::{napi_env, napi_get_value_string_utf8, napi_value},
};

use oxc_allocator::Allocator;
use oxc_data_structures::types::implements;
use oxc_parser::Source;
use oxc_span::SourceType;

/// Source text in its own [`Allocator`], which can be sent to another thread.
///
/// # Usage
///
/// Use [`NapiSource`] as the type of the source text parameter of a `#[napi]` function,
/// and call [`source`] to get the [`Source`] to parse.
///
/// `NapiSource` implements [`FromNapiValue`], so NAPI-RS creates it from the JS string
/// on main thread, before the function's body runs. The string is converted to UTF-8,
/// and written directly into the `NapiSource`'s [`Allocator`]. This avoids the intermediate
/// `String` which NAPI-RS creates when a function takes a `String` parameter, and then copying
/// that `String` into the arena.
///
/// Give the parameter a `ts_arg_type`, so it's typed as `string` in the TypeScript definitions.
///
/// ```ignore
/// #[napi]
/// pub fn parse_sync(
///     filename: String,
///     #[napi(ts_arg_type = "string")]
///     mut source_text: NapiSource,
/// ) -> ParseResult {
///     let source_type = SourceType::from_path(&filename).unwrap_or_default();
///     let source = source_text.source(source_type);
///     // ...
/// }
/// ```
///
/// # Sending to another thread
///
/// A JS string can only be read on main thread. A [`Source`] borrows its [`Allocator`],
/// so the two cannot be moved to another thread together.
///
/// `NapiSource` owns the [`Allocator`], and holds the raw parts of the [`Source`],
/// so it is [`Send`]. That allows using it in functions which do their work on another thread:
///
/// - `async` functions. Their parameters must be [`Send`]. The function owns the `NapiSource`,
///   so it's dropped when the function's body ends, on the thread which the body ran on.
///
/// - Functions which return an `AsyncTask`. Move the `NapiSource` into the task,
///   in an `Option`. In the task's `compute` method, take the `NapiSource` out of
///   the `Option`, and get the [`Source`] with [`source`].
///
///   Taking the `NapiSource` out of the task means it's dropped at the end of `compute`,
///   on the other thread. If it was left in the task, it'd be dropped on main thread,
///   when the task is dropped.
///
/// [`source`]: Self::source
//
// # Invariants
//
// 1. `ptr` points to a region of memory `len + Source::PADDING` bytes long,
//    within `allocator`'s arena.
// 2. The first `len` bytes of the region are initialized, and are valid UTF-8.
// 3. No reference to any part of the region exists, except:
//    - Ones which have the lifetime of a `Source` returned by `source` method
//      (that `Source`, and references obtained from it).
//    - `&str`s returned by `as_str` method, which have the lifetime of a shared borrow of `self`.
//
// (2) keeps holding after `source` has been called, because nothing writes to the source text.
// A `Source` leaves its source bytes unchanged, and the pointer which `Source::into_raw_parts`
// returns is read-only.
//
// Raw pointers to the region may exist (e.g. from `Source::into_raw_parts`). That's not a problem.
// Reading via one requires `unsafe` code, and it's only valid for reads during the lifetime
// of the `Source` it came from.
pub struct NapiSource {
    /// Pointer to start of source text, within `allocator`'s arena.
    ptr: NonNull<u8>,
    /// Length of source text in bytes.
    len: usize,
    /// Allocator which source text is stored in.
    allocator: Allocator,
}

const _: () = assert!(implements!(Allocator: Send));

// SAFETY: `Allocator` is `Send` (checked above). `ptr` is not, but it points into
// `allocator`'s arena. `NapiSource` owns both, so they move to another thread together.
// A `NapiSource` can only be moved while it's not borrowed, so no `Source` returned by `source`,
// no reference obtained from one, and no `&str` returned by `as_str`, can exist when it moves.
unsafe impl Send for NapiSource {}

impl NapiSource {
    /// Get the [`Source`], to be parsed as `source_type`.
    ///
    /// The source is in this `NapiSource`'s [`Allocator`].
    /// Use [`Source::allocator`] to get the [`Allocator`].
    pub fn source(&mut self, source_type: SourceType) -> Source<'_> {
        // SAFETY:
        // - `ptr` points to a region of `len + Source::PADDING` bytes in `self.allocator`'s arena.
        // - The first `len` bytes are initialized, and are valid UTF-8.
        // - No reference to any part of the region exists. There are 2 ways to get one:
        //   1. From a `Source` returned by a previous call to this method.
        //      That `Source`, and every reference obtained from it (e.g. the `&str` from
        //      `Source::into_str`, or `source_text` in an AST), has the lifetime of that call's
        //      mutable borrow of `self`.
        //   2. From `as_str`. That `&str` has the lifetime of a shared borrow of `self`.
        //   This call borrows `self` mutably, so none of them can exist now.
        // - While the `Source` returned here exists, the region is only read or written via it.
        //   `ptr` is private, `allocator` does not hand out the region again,
        //   and `as_str` cannot be called while `self` is mutably borrowed.
        unsafe { Source::from_raw_parts(self.ptr, self.len, true, source_type, &self.allocator) }
    }

    /// Get the source text as a `&str`.
    ///
    /// Unlike [`Source::as_str`], this cannot fail. The source text was created from a JS string,
    /// so it's always valid UTF-8.
    pub fn as_str(&self) -> &str {
        // SAFETY: `ptr` points to `len` initialized bytes.
        // They cannot be written to while the returned `&str` exists. Getting a `Source`
        // requires a mutable borrow of `self`, which cannot overlap with the shared borrow
        // which the `&str` holds.
        // The first `len` bytes are valid UTF-8.
        unsafe {
            let bytes = slice::from_raw_parts(self.ptr.as_ptr(), self.len);
            str::from_utf8_unchecked(bytes)
        }
    }
}

impl FromNapiValue for NapiSource {
    /// Create a [`NapiSource`] from a JS value, which should be a JS string.
    ///
    /// The string is converted to UTF-8, and written directly into a new [`Allocator`].
    ///
    /// # Errors
    ///
    /// Returns [`napi::Error`] if either:
    ///
    /// - `napi_val` is not a JS string.
    /// - The string as UTF-8, plus padding, is larger than `isize::MAX` bytes.
    ///
    /// # SAFETY
    ///
    /// - `env` must be a valid `napi_env`.
    /// - `napi_val` must be a valid `napi_value`, belonging to `env`.
    /// - Must be called on the thread which `env` belongs to.
    unsafe fn from_napi_value(env: napi_env, napi_val: napi_value) -> napi::Result<Self> {
        // `napi_get_value_string_utf8` writes a NUL terminator after the string.
        // It goes in the 1st byte of the padding.
        const _: () = assert!(Source::PADDING >= 1);

        // Get length of the string as UTF-8, in bytes.
        // N-API returns an error status if the value is not a string.
        let mut len = 0;
        // SAFETY: Caller guarantees `env` and `napi_val` are valid.
        // When buffer pointer is null, N-API only writes the length to `len`.
        let status =
            unsafe { napi_get_value_string_utf8(env, napi_val, ptr::null_mut(), 0, &raw mut len) };
        check_status!(status, "Source text must be a string")?;

        // Allocate space for the string and padding in a new arena
        let padded_len = len.checked_add(Source::PADDING);
        let layout = padded_len.and_then(|padded_len| Layout::from_size_align(padded_len, 1).ok());
        let Some(layout) = layout else {
            return Err(napi::Error::from_reason("Source text is too long"));
        };

        let allocator = Allocator::default();
        let ptr = allocator.alloc_layout(layout);

        // Write the string into the allocated space, as UTF-8
        let mut written_len = 0;
        // SAFETY: Caller guarantees `env` and `napi_val` are valid.
        // `ptr` points to a new allocation of `len + PADDING` bytes, so it's valid for writing
        // `len + 1` bytes, which is the buffer size passed to N-API (`len` bytes of string + NUL).
        let status = unsafe {
            napi_get_value_string_utf8(
                env,
                napi_val,
                ptr.as_ptr().cast(),
                len + 1,
                &raw mut written_len,
            )
        };
        check_status!(status, "Failed to copy JS string")?;

        // N-API writes at most 1 byte less than buffer size, not including the NUL terminator
        debug_assert!(written_len <= len);

        // This upholds the invariants of `NapiSource`:
        // - `ptr` points to a new allocation of `len + PADDING` bytes in `allocator`'s arena.
        //   No reference to it exists.
        // - `written_len <= len`.
        // - The first `written_len` bytes were initialized by N-API.
        // - N-API specifies that `napi_get_value_string_utf8` writes the string encoded as UTF-8.
        //   A JS string can contain lone surrogates, which cannot be encoded as UTF-8,
        //   so a conforming implementation must replace them (Node writes U+FFFD).
        //   NAPI-RS relies on the same guarantee when it converts a JS string to a `String`.
        Ok(Self { ptr, len: written_len, allocator })
    }
}
