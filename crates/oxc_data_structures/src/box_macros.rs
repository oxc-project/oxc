//! Macros for creating boxed slices and arrays.

// TODO: We might be able to make these more performant by using `MaybeUninit`.

/// Macro to create a boxed slice (`Box<[T]>`).
///
/// Similar to standard library's `vec!` macro.
///
/// If length is a const (known at compile time), you would likely be better off using a boxed array instead
/// (`boxed_array!` macro).
///
/// # Example
/// ```
/// # fn get_length_somehow() -> usize { 5 }
///
/// use oxc_data_structures::box_macros::boxed_slice;
///
/// let len = get_length_somehow();
/// // Creates a `Box<[u64]>`
/// let boxed = boxed_slice![0u64; len];
/// ```
#[macro_export]
macro_rules! boxed_slice {
    ($value:expr; $len:expr) => {
        ::std::vec![$value; $len].into_boxed_slice()
    };
}

pub use boxed_slice;

/// Macro to create a boxed array (`Box<[T; N]>`).
///
/// Similar to standard library's `vec!` macro.
///
/// `$len` must be a const expression.
///
/// # Example
/// ```
/// use oxc_data_structures::box_macros::boxed_array;
///
/// const LENGTH: usize = 5;
/// // Creates a `Box<[u64; LENGTH]>`
/// let boxed = boxed_array![0u64; LENGTH];
/// ```
#[macro_export]
macro_rules! boxed_array {
    ($value:expr; $len:expr) => {{
        // Make sure `$len` is const
        const LEN: usize = $len;
        let boxed_slice = ::std::vec![$value; LEN].into_boxed_slice();
        // `.ok()` is to support types which are not `Debug`
        #[allow(clippy::allow_attributes)]
        #[allow(clippy::missing_panics_doc, reason = "infallible")]
        ::std::boxed::Box::<[_; LEN]>::try_from(boxed_slice).ok().unwrap()
    }};
}

pub use boxed_array;
