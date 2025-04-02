//! Unsigned integer types.
//!
//! TODO

use std::{
    cmp::Ordering,
    fmt::{self, Debug, Display},
    hash::{Hash, Hasher},
};

use paste::paste;
use seq_macro::seq;

use crate::assert_unchecked;

/// Macro to define an unsigned integer type which wraps a primitive.
///
/// `primitive_wrapper!(U8, 8, u8)` =
/// * Type name: U8
/// * Number of bits: 8
/// * primitive value: u8
macro_rules! primitive_wrapper {
    ($name:ident, $bits:literal, $primitive:ident) => {
        #[doc = concat!(" ", $bits, "-bit unsigned integer.")]
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
        #[repr(transparent)]
        pub struct $name($primitive);

        impl $name {
            #[doc = concat!(" Maximum value for [`", stringify!($name), "`].")]
            pub const MAX: usize = $primitive::MAX as usize;

            #[doc = concat!(" Number of bits required to represent a [`", stringify!($name), "`].")]
            pub const BITS: usize = $bits;

            #[doc = concat!(" Convert `usize` to [`", stringify!($name), "`], without checks.")]
            #[doc = ""]
            #[doc = " # SAFETY"]
            #[doc = concat!(" `n` must be less than or equal to `(1 << ", $bits, ") - 1`.")]
            #[inline(always)]
            pub const unsafe fn from_usize_unchecked(n: usize) -> Self {
                // SAFETY: Caller guarantees `n` is in range
                unsafe { assert_unchecked!(n <= Self::MAX) };
                #[expect(clippy::cast_possible_truncation)]
                Self(n as $primitive)
            }

            #[doc = concat!(" Convert [`", stringify!($name), "`] to `", stringify!($primitive), "`.")]
            #[inline(always)]
            pub const fn to_primitive(self) -> $primitive {
                self.0 as $primitive
            }

            #[doc = concat!(" Convert [`", stringify!($name), "`] to `usize`.")]
            #[inline(always)]
            pub const fn to_usize(self) -> usize {
                self.0 as usize
            }
        }
    };
}

// Primitive wrappers
primitive_wrapper!(U8, 8, u8);
primitive_wrapper!(U16, 16, u16);
primitive_wrapper!(U32, 32, u32);

/// Macro to define an unsigned integer type which wraps an enum.
///
/// `niched!((pub) U7, u8, 7, 127)` =
/// * Visibility: pub
/// * Type name: U7
/// * Primitive type: u8
/// * Number of bits: 7
/// * Maximum value: 127
macro_rules! niched {
    (($($vis:tt)*) $name:ident, $primitive:ident, $bits:literal, $max:literal) => {
        paste! {
            seq!(N in 0..=$max {
                #[allow(non_snake_case, dead_code, clippy::allow_attributes)]
                mod [<__ $name>] {
                    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
                    #[repr($primitive)]
                    enum Inner {
                        #[default]
                        #(
                            _~N = N,
                        )*
                    }

                    #[doc = concat!(" ", $bits, "-bit unsigned integer.")]
                    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
                    #[repr(transparent)]
                    pub struct $name(Inner);

                    impl $name {
                        #[doc = concat!(" Maximum value for [`", stringify!($name), "`].")]
                        pub const MAX: usize = $max;

                        #[doc = concat!(" Number of bits required to represent a [`", stringify!($name), "`].")]
                        pub const BITS: usize = $bits;

                        #[doc = concat!(" Convert `usize` to [`", stringify!($name), "`], without checks.")]
                        #[doc = ""]
                        #[doc = " # SAFETY"]
                        #[doc = concat!(" `n` must be less than or equal to ", $max, ".")]
                        #[inline(always)]
                        pub const unsafe fn from_usize_unchecked(n: usize) -> Self {
                            match n {
                                #(
                                    N => Self(Inner::_~N),
                                )*
                                _ => {
                                    // SAFETY: Caller guarantees `n` is in range
                                    unsafe { std::hint::unreachable_unchecked() }
                                }
                            }
                        }

                        #[doc = concat!(" Convert [`", stringify!($name), "`] to `", stringify!($primitive), "`.")]
                        #[inline(always)]
                        pub const fn to_primitive(self) -> $primitive {
                            self.0 as $primitive
                        }

                        #[doc = concat!(" Convert [`", stringify!($name), "`] to `usize`.")]
                        #[inline(always)]
                        pub const fn to_usize(self) -> usize {
                            self.0 as usize
                        }
                    }
                }
                $($vis)* use [<__ $name>]::$name;
            });
        }
    };
}

/// Macro to define a public unsigned integer type which wraps an enum, which is `#[repr(u8)]`.
///
/// `niched_u8!(U7, 7, 127)` =
/// * Type name: U7
/// * Number of bits: 7
/// * Maximum value: 127
macro_rules! niched_u8 {
    ($name:ident, $bits:literal, $max:literal) => {
        niched!((pub) $name, u8, $bits, $max);
    }
}

niched_u8!(U1, 1, 1);
niched_u8!(U2, 2, 3);
niched_u8!(U3, 3, 7);
niched_u8!(U4, 4, 15);
niched_u8!(U5, 5, 31);
niched_u8!(U6, 6, 63);
niched_u8!(U7, 7, 127);

/// Macro to define a private unsigned integer type which wraps an enum, which is `#[repr(u16)]`.
///
/// These types are only useful for constructing composite types.
/// e.g. `U24` is composed from a `U16` and a `D8`.
///
/// `niched_u16!(D7, 7, 127)` =
/// * Type name: D7
/// * Number of bits: 7
/// * Maximum value: 127
macro_rules! niched_u16 {
    ($name:ident, $bits:literal, $max:literal) => {
        niched!(() $name, u16, $bits, $max);
    }
}

niched_u16!(D1, 1, 1);
niched_u16!(D2, 2, 3);
niched_u16!(D3, 3, 7);
niched_u16!(D4, 4, 15);
niched_u16!(D5, 5, 31);
niched_u16!(D6, 6, 63);
niched_u16!(D7, 7, 127);
niched_u16!(D8, 8, 255);

/// Macro to define an unsigned integer type which wraps a niched type and a non-niched type.
///
/// e.g. `U15` combines a `U8` as low bits and `U7` as high bits.
/// `U23` combines a `U16` as low bits and `D7` as high bits.
macro_rules! composite {
    ($name:ident, $low:ident, $high:ident, $primitive:ident, $bits: literal) => {
        #[doc = concat!(" ", $bits, "-bit unsigned integer.")]
        #[derive(Clone, Copy)]
        #[repr(C)]
        pub struct $name {
            align: [$primitive; 0],
            #[cfg(target_endian = "little")]
            low: $low,
            high: $high,
            #[cfg(target_endian = "big")]
            low: $low,
        }

        impl $name {
            #[doc = concat!(" Maximum value for [`", stringify!($name), "`].")]
            pub const MAX: usize = (1 << $bits) - 1;

            #[doc = concat!(" Number of bits required to represent a [`", stringify!($name), "`].")]
            pub const BITS: usize = $bits;

            #[doc = concat!(" Convert `usize` to [`", stringify!($name), "`], without checks.")]
            #[doc = ""]
            #[doc = " # SAFETY"]
            #[doc = concat!(" `n` must be less than or equal to `(1 << ", $bits, ") - 1`.")]
            #[cfg_attr(target_endian = "big", expect(clippy::inconsistent_struct_constructor))]
            #[inline(always)]
            pub const unsafe fn from_usize_unchecked(n: usize) -> Self {
                // SAFETY: Caller guarantees `n` is in legal range
                let (high, low) = unsafe {
                    let high = $high::from_usize_unchecked(n >> $low::BITS);
                    let low = $low::from_usize_unchecked(n & ((1 << $low::BITS) - 1));
                    (high, low)
                };
                Self { align: [], low, high }
            }

            #[doc = concat!(" Convert [`", stringify!($name), "`] to `", stringify!($primitive), "`.")]
            #[inline(always)]
            pub const fn to_primitive(self) -> $primitive {
                (self.low.to_primitive() as $primitive) | ((self.high.to_primitive() as $primitive) << $low::BITS)
            }

            #[doc = concat!(" Convert [`", stringify!($name), "`] to `usize`.")]
            #[inline(always)]
            pub const fn to_usize(self) -> usize {
                self.to_primitive() as usize
            }
        }

        // Implement traits by delegating to same methods on primitive counterparts

        impl Eq for $name {}

        impl PartialEq for $name {
            #[inline(always)]
            fn eq(&self, other: &Self) -> bool {
                self.to_primitive() == other.to_primitive()
            }
        }

        impl Ord for $name {
            #[inline(always)]
            fn cmp(&self, other: &Self) -> Ordering {
                self.to_primitive().cmp(&other.to_primitive())
            }
        }

        impl PartialOrd for $name {
            #[inline(always)]
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }

        impl Hash for $name {
            #[inline(always)]
            fn hash<H: Hasher>(&self, state: &mut H) {
                self.to_primitive().hash(state);
            }
        }

        impl Default for $name {
            #[inline(always)]
            fn default() -> Self {
                // SAFETY: 0 is a valid value
                unsafe { Self::from_usize_unchecked(0) }
            }
        }
    };
}

// `U1` - `U7` are defined with `niched!` macro.
// `U8` is defined with `primitive_wrapper!` macro.
composite!(U9, U8, U1, u16, 9);
composite!(U10, U8, U2, u16, 10);
composite!(U11, U8, U3, u16, 11);
composite!(U12, U8, U4, u16, 12);
composite!(U13, U8, U5, u16, 13);
composite!(U14, U8, U6, u16, 14);
composite!(U15, U8, U7, u16, 15);
// `U16` is defined with `primitive_wrapper!` macro.
composite!(U17, U16, D1, u32, 17);
composite!(U18, U16, D2, u32, 18);
composite!(U19, U16, D3, u32, 19);
composite!(U20, U16, D4, u32, 20);
composite!(U21, U16, D5, u32, 21);
composite!(U22, U16, D6, u32, 22);
composite!(U23, U16, D7, u32, 23);
composite!(U24, U16, D8, u32, 24);
composite!(U25, U16, U9, u32, 25);
composite!(U26, U16, U10, u32, 26);
composite!(U27, U16, U11, u32, 27);
composite!(U28, U16, U12, u32, 28);
composite!(U29, U16, U13, u32, 29);
composite!(U30, U16, U14, u32, 30);
composite!(U31, U16, U15, u32, 31);
// `U32` is defined with `primitive_wrapper!` macro.

/// Macro to implement `Display` and `Debug` for types,
/// by delegating to the same method on their primitive counterpart.
macro_rules! impl_fmt {
    ($($name:ident,)+) => {
        $(
            impl Display for $name {
                #[inline(always)]
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    Display::fmt(&self.to_primitive(), f)
                }
            }

            impl Debug for $name {
                #[inline(always)]
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    Debug::fmt(&self.to_primitive(), f)
                }
            }
        )+
    };
}

impl_fmt!(
    U1, U2, U3, U4, U5, U6, U7, U8, U9, U10, U11, U12, U13, U14, U15, U16, U17, U18, U19, U20, U21,
    U22, U23, U24, U25, U26, U27, U28, U29, U30, U31, U32,
);
