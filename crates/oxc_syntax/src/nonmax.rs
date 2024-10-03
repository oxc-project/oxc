//! Niched non-max types, similar to `NonZero*`, but with the illegal values at top of the range,
//! rather than 0.
//!
//! `nonmax` crate provides similar types, but they work by storing the value internally as a `NonZero*`,
//! and inverting the bits on every read/write (`native_u32 = nonmax_u32 as u32 ^ u32::MAX`).
//! This reserves only a single illegal value, but has (small) runtime cost on every read/write.
//!
//! The `NonMax*` types provided here make a different trade-off.
//!
//! `NonMaxU8` wraps an enum with a single illegal value (255).
//! All the other types store the value as a number of `u8`s + a single `NonMaxU8` as highest byte.
//! i.e. they reserve all values which have 255 as the highest byte.
//!
//! * `NonMaxU8` can represent the values `0` to `254`.
//! * `NonMaxU16` can represent the values `0` to `(255 << 8) - 1`.
//! * `NonMaxU32` can represent the values `0` to `(255 << 24) - 1`.
//! * `NonMaxU64` can represent the values `0` to `(255 << 56) - 1`.
//! * `NonMaxU128` can represent the values `0` to `(255 << 120) - 1`.
//!
//! We trade approx 0.4% of the legal range being unavailable, in return for zero runtime cost
//! when reading or writing the values.
//!
//! All `NonMax*` types have a single niche (even though they have multiple illegal values).
//! `Option<NonMax*>` is same size as its native equivalent, for all `NonMax*` types.
//! e.g. `size_of::<Option<NonMaxU32>>() == size_of::<u32>()`.

use std::{
    cmp, fmt,
    hash::{Hash, Hasher},
    mem::{align_of, size_of},
    ops::{BitAnd, BitAndAssign},
};

macro_rules! impl_nonmax {
    ($nonmax:ident, $native:ident) => {
        const _: () = {
            assert!(size_of::<$nonmax>() == size_of::<$native>());
            assert!(align_of::<$nonmax>() == align_of::<$native>());
        };

        impl $nonmax {
            /// Zero for this non-max integer type
            // SAFETY: 0 is a valid value for this type.
            pub const ZERO: Self = unsafe { Self::new_unchecked(0) };

            /// One for this non-max integer type.
            // SAFETY: 1 is a valid value for this type.
            pub const ONE: Self = unsafe { Self::new_unchecked(1) };

            /// The smallest value that can be represented by this non-max integer type, 0.
            pub const MIN: Self = Self::ZERO;

            /// The largest value that can be represented by this non-max integer type.
            // SAFETY: This is a valid value for this type.
            // Equals highest byte 254, all other bytes 255.
            // This corresponds to highest byte being a `NonMaxU8` and all other bytes being `u8`s.
            pub const MAX: Self = unsafe {
                Self::new_unchecked_internal(((255 as $native) << ($native::BITS - 8)) - 1)
            };

            /// The size of this non-max integer type in bits.
            pub const BITS: u32 = $native::BITS;

            #[doc = concat!("Create new `", stringify!($nonmax), "` from `", stringify!($native), "`.")]
            #[doc = ""]
            #[doc = concat!("Returns `None` if `n` is greater than `", stringify!($nonmax), "::MAX.get()`.")]
            #[inline]
            pub const fn new(n: $native) -> Option<Self> {
                if n <= Self::MAX.get() {
                    // SAFETY: We just checked `n` does not exceed `Self::MAX.get()`
                    Some(unsafe { Self::new_unchecked(n) })
                } else {
                    None
                }
            }

            #[doc = concat!("Create new `", stringify!($nonmax), "` from `", stringify!($native), "`,")]
            #[doc = "panicking if `n` is invalid."]
            #[doc = ""]
            #[doc = "# Panics"]
            #[doc = concat!("Panics if `n` is greater than `", stringify!($nonmax), "::MAX.get()`.")]
            #[inline]
            pub const fn new_checked(n: $native) -> Self {
                assert!(n <= Self::MAX.get());
                // SAFETY: We just checked `n` does not exceed `Self::MAX.get()`
                unsafe { Self::new_unchecked(n) }
            }

            #[doc = concat!("Create new `", stringify!($nonmax), "` from `usize`.")]
            #[doc = ""]
            #[doc = "# Panics"]
            #[doc = concat!("Panics if `n` is greater than `", stringify!($nonmax), "::MAX.get()`.")]
            #[inline]
            #[allow(clippy::cast_possible_truncation)]
            pub const fn from_usize(n: usize) -> Self {
                // If nonmax type is larger than `usize` (on 64-bit systems, only `NonMaxU128`),
                // any `usize` is guaranteed to be valid, so skip assertion
                if size_of::<Self>() <= size_of::<usize>() {
                    assert!(n <= Self::MAX.get() as usize);
                }

                // SAFETY: We just checked `n` does not exceed `Self::MAX.get()`
                unsafe { Self::new_unchecked(n as $native) }
            }

            #[doc = concat!("Create new `", stringify!($nonmax), "` from `", stringify!($native), "`, without checking validity of the input.")]
            #[doc = ""]
            #[doc = "# SAFETY"]
            #[doc = concat!("Caller must ensure `n` does not exceed `", stringify!($nonmax), "::MAX.get()`.")]
            #[inline]
            #[allow(clippy::missing_safety_doc)]
            pub const unsafe fn new_unchecked(n: $native) -> Self {
                debug_assert!(n <= Self::MAX.get());
                // SAFETY: Caller guarantees `n` does not exceed `Self::MAX.get()`.
                // All bit patterns of native type `<= Self::MAX.get()` are valid bit patterns for non-max type.
                // Size and align of non-max type and native type are the same.
                unsafe { std::mem::transmute::<$native, Self>(n) }
            }

            /// Internal version of `new_unchecked`, without debug assertion.
            /// Only used for initializing `MAX` constant, to avoid circularity.
            #[inline]
            #[allow(clippy::missing_safety_doc)]
            const unsafe fn new_unchecked_internal(n: $native) -> Self {
                // SAFETY: Caller guarantees `n` does not exceed `Self::MAX.get()`.
                // All bit patterns of native type `<= Self::MAX.get()` are valid bit patterns for non-max type.
                // Size and align of non-max type and native type are the same.
                unsafe { std::mem::transmute::<$native, Self>(n) }
            }

            #[doc = concat!("Convert `", stringify!($nonmax), "` to `", stringify!($native), "`.")]
            #[inline]
            pub const fn get(self) -> $native {
                // SAFETY: All valid bit patterns of non-max type are valid bit patterns for native type.
                // Size and align of non-max type and native type are the same.
                unsafe { std::mem::transmute::<Self, $native>(self) }
            }
        }

        impl Default for $nonmax {
            fn default() -> Self {
                Self::ZERO
            }
        }

        impl From<$nonmax> for $native {
            #[doc = concat!("Convert `", stringify!($nonmax), "` to `", stringify!($native), "`.")]
            #[inline]
            fn from(n: $nonmax) -> $native {
                n.get()
            }
        }

        impl TryFrom<$native> for $nonmax {
            type Error = ();

            #[doc = concat!("Try to convert `", stringify!($native), "` to `", stringify!($nonmax), "`.")]
            #[doc = ""]
            #[doc = concat!("Returns `Err` if `n` is greater than `", stringify!($nonmax), "::MAX.get()`.")]
            #[inline]
            fn try_from(n: $native) -> Result<Self, ()> {
                match Self::new(n) {
                    Some(n) => Ok(n),
                    None => Err(()),
                }
            }
        }

        impl PartialEq<Self> for $nonmax {
            #[inline]
            fn eq(&self, other: &Self) -> bool {
                self.get() == other.get()
            }
        }

        impl Eq for $nonmax {}

        impl PartialOrd for $nonmax {
            #[inline]
            fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
                Some(Ord::cmp(self, other))
            }
        }

        impl Ord for $nonmax {
            #[inline]
            fn cmp(&self, other: &Self) -> cmp::Ordering {
                Ord::cmp(&self.get(), &other.get())
            }
        }

        impl Hash for $nonmax {
            #[inline]
            fn hash<H: Hasher>(&self, hasher: &mut H) {
                Hash::hash(&self.get(), hasher)
            }
        }

        // `NonZero*` can implement `BitOr`, but not `BitAnd`.
        // `NonMax*` can implement `BitAnd` but not `BitOr`.

        impl BitAnd<Self> for $nonmax {
            type Output = Self;

            #[inline]
            fn bitand(self, rhs: Self) -> Self::Output {
                // SAFETY: `self` is non-max type, so cannot have 255 as its top byte.
                // This means at least one bit of top byte is 0.
                // Therefore AND with any number will also have at least 1 bit of top byte unset.
                // So top byte of result cannot be 255, which means result is a legal non-max value.
                unsafe { Self::new_unchecked(self.get() & rhs.get()) }
            }
        }

        impl BitAnd<$native> for $nonmax {
            type Output = Self;

            #[inline]
            fn bitand(self, rhs: $native) -> Self::Output {
                // SAFETY: `self` is non-max type, so cannot have 255 as its top byte.
                // This means at least one bit of top byte is 0.
                // Therefore AND with any number will also have at least 1 bit of top byte unset.
                // So top byte of result cannot be 255, which means result is a legal non-max value.
                unsafe { Self::new_unchecked(self.get() & rhs) }
            }
        }

        impl BitAnd<$nonmax> for $native {
            type Output = $nonmax;

            #[inline]
            fn bitand(self, rhs: $nonmax) -> Self::Output {
                // SAFETY: `rhs` is non-max type, so cannot have 255 as its top byte.
                // This means at least one bit of top byte is 0.
                // Therefore AND with any number will also have at least 1 bit of top byte unset.
                // So top byte of result cannot be 255, which means result is a legal non-max value.
                unsafe { $nonmax::new_unchecked(self & rhs.get()) }
            }
        }

        impl BitAndAssign<Self> for $nonmax {
            #[inline]
            fn bitand_assign(&mut self, rhs: Self) {
                *self = *self & rhs;
            }
        }

        impl BitAndAssign<$native> for $nonmax {
            #[inline]
            fn bitand_assign(&mut self, rhs: $native) {
                *self = *self & rhs;
            }
        }

        impl BitAndAssign<$nonmax> for $native {
            #[inline]
            fn bitand_assign(&mut self, rhs: $nonmax) {
                *self = *self & rhs.get();
            }
        }

        // https://doc.rust-lang.org/1.47.0/src/core/num/mod.rs.html#173-175
        impl_fmt!(Debug, $nonmax);
        impl_fmt!(Display, $nonmax);
        impl_fmt!(Binary, $nonmax);
        impl_fmt!(Octal, $nonmax);
        impl_fmt!(LowerHex, $nonmax);
        impl_fmt!(UpperHex, $nonmax);
    };
}

// https://doc.rust-lang.org/1.47.0/src/core/num/mod.rs.html#31-43
macro_rules! impl_fmt {
    ($trait:ident, $nonmax:ident) => {
        impl fmt::$trait for $nonmax {
            #[inline]
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::$trait::fmt(&self.get(), f)
            }
        }
    };
}

/// `u8` with a niche for maximum value.
///
/// Equivalent of `NonZeroU8`, but with illegal value of `u8::MAX`, instead of 0.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct NonMaxU8(NonMaxU8Internal);

#[derive(Clone, Copy)]
#[repr(u8)]
enum NonMaxU8Internal {
    _0 = 0,
    _1 = 1,
    _2 = 2,
    _3 = 3,
    _4 = 4,
    _5 = 5,
    _6 = 6,
    _7 = 7,
    _8 = 8,
    _9 = 9,
    _10 = 10,
    _11 = 11,
    _12 = 12,
    _13 = 13,
    _14 = 14,
    _15 = 15,
    _16 = 16,
    _17 = 17,
    _18 = 18,
    _19 = 19,
    _20 = 20,
    _21 = 21,
    _22 = 22,
    _23 = 23,
    _24 = 24,
    _25 = 25,
    _26 = 26,
    _27 = 27,
    _28 = 28,
    _29 = 29,
    _30 = 30,
    _31 = 31,
    _32 = 32,
    _33 = 33,
    _34 = 34,
    _35 = 35,
    _36 = 36,
    _37 = 37,
    _38 = 38,
    _39 = 39,
    _40 = 40,
    _41 = 41,
    _42 = 42,
    _43 = 43,
    _44 = 44,
    _45 = 45,
    _46 = 46,
    _47 = 47,
    _48 = 48,
    _49 = 49,
    _50 = 50,
    _51 = 51,
    _52 = 52,
    _53 = 53,
    _54 = 54,
    _55 = 55,
    _56 = 56,
    _57 = 57,
    _58 = 58,
    _59 = 59,
    _60 = 60,
    _61 = 61,
    _62 = 62,
    _63 = 63,
    _64 = 64,
    _65 = 65,
    _66 = 66,
    _67 = 67,
    _68 = 68,
    _69 = 69,
    _70 = 70,
    _71 = 71,
    _72 = 72,
    _73 = 73,
    _74 = 74,
    _75 = 75,
    _76 = 76,
    _77 = 77,
    _78 = 78,
    _79 = 79,
    _80 = 80,
    _81 = 81,
    _82 = 82,
    _83 = 83,
    _84 = 84,
    _85 = 85,
    _86 = 86,
    _87 = 87,
    _88 = 88,
    _89 = 89,
    _90 = 90,
    _91 = 91,
    _92 = 92,
    _93 = 93,
    _94 = 94,
    _95 = 95,
    _96 = 96,
    _97 = 97,
    _98 = 98,
    _99 = 99,
    _100 = 100,
    _101 = 101,
    _102 = 102,
    _103 = 103,
    _104 = 104,
    _105 = 105,
    _106 = 106,
    _107 = 107,
    _108 = 108,
    _109 = 109,
    _110 = 110,
    _111 = 111,
    _112 = 112,
    _113 = 113,
    _114 = 114,
    _115 = 115,
    _116 = 116,
    _117 = 117,
    _118 = 118,
    _119 = 119,
    _120 = 120,
    _121 = 121,
    _122 = 122,
    _123 = 123,
    _124 = 124,
    _125 = 125,
    _126 = 126,
    _127 = 127,
    _128 = 128,
    _129 = 129,
    _130 = 130,
    _131 = 131,
    _132 = 132,
    _133 = 133,
    _134 = 134,
    _135 = 135,
    _136 = 136,
    _137 = 137,
    _138 = 138,
    _139 = 139,
    _140 = 140,
    _141 = 141,
    _142 = 142,
    _143 = 143,
    _144 = 144,
    _145 = 145,
    _146 = 146,
    _147 = 147,
    _148 = 148,
    _149 = 149,
    _150 = 150,
    _151 = 151,
    _152 = 152,
    _153 = 153,
    _154 = 154,
    _155 = 155,
    _156 = 156,
    _157 = 157,
    _158 = 158,
    _159 = 159,
    _160 = 160,
    _161 = 161,
    _162 = 162,
    _163 = 163,
    _164 = 164,
    _165 = 165,
    _166 = 166,
    _167 = 167,
    _168 = 168,
    _169 = 169,
    _170 = 170,
    _171 = 171,
    _172 = 172,
    _173 = 173,
    _174 = 174,
    _175 = 175,
    _176 = 176,
    _177 = 177,
    _178 = 178,
    _179 = 179,
    _180 = 180,
    _181 = 181,
    _182 = 182,
    _183 = 183,
    _184 = 184,
    _185 = 185,
    _186 = 186,
    _187 = 187,
    _188 = 188,
    _189 = 189,
    _190 = 190,
    _191 = 191,
    _192 = 192,
    _193 = 193,
    _194 = 194,
    _195 = 195,
    _196 = 196,
    _197 = 197,
    _198 = 198,
    _199 = 199,
    _200 = 200,
    _201 = 201,
    _202 = 202,
    _203 = 203,
    _204 = 204,
    _205 = 205,
    _206 = 206,
    _207 = 207,
    _208 = 208,
    _209 = 209,
    _210 = 210,
    _211 = 211,
    _212 = 212,
    _213 = 213,
    _214 = 214,
    _215 = 215,
    _216 = 216,
    _217 = 217,
    _218 = 218,
    _219 = 219,
    _220 = 220,
    _221 = 221,
    _222 = 222,
    _223 = 223,
    _224 = 224,
    _225 = 225,
    _226 = 226,
    _227 = 227,
    _228 = 228,
    _229 = 229,
    _230 = 230,
    _231 = 231,
    _232 = 232,
    _233 = 233,
    _234 = 234,
    _235 = 235,
    _236 = 236,
    _237 = 237,
    _238 = 238,
    _239 = 239,
    _240 = 240,
    _241 = 241,
    _242 = 242,
    _243 = 243,
    _244 = 244,
    _245 = 245,
    _246 = 246,
    _247 = 247,
    _248 = 248,
    _249 = 249,
    _250 = 250,
    _251 = 251,
    _252 = 252,
    _253 = 253,
    _254 = 254,
    // No variant for 255
}

impl_nonmax!(NonMaxU8, u8);

/// `u16` with niche for maximum values.
///
/// Equivalent of `NonZeroU16`, but with illegal values of `>= (255 << 8)`, instead of 0.
///
/// Although has 256 illegal values, this type has only a single niche.
#[derive(Clone, Copy)]
#[repr(C, align(2))]
pub struct NonMaxU16 {
    #[cfg(target_endian = "big")]
    top: NonMaxU8,
    bottom: u8,
    #[cfg(target_endian = "little")]
    top: NonMaxU8,
}

impl_nonmax!(NonMaxU16, u16);

/// `u32` with niche for maximum values.
///
/// Equivalent of `NonZeroU32`, but with illegal values of `>= (255 << 24)`, instead of 0.
///
/// Although has `1 << 24` illegal values, this type has only a single niche.
#[derive(Clone, Copy)]
#[repr(C, align(4))]
pub struct NonMaxU32 {
    #[cfg(target_endian = "big")]
    top: NonMaxU8,
    bottom: [u8; 3],
    #[cfg(target_endian = "little")]
    top: NonMaxU8,
}

impl_nonmax!(NonMaxU32, u32);

/// `u64` with niche for maximum values.
///
/// Equivalent of `NonZeroU64`, but with illegal values of `>= (255 << 56)`, instead of 0.
///
/// Although has `1 << 56` illegal values, this type has only a single niche.
#[derive(Clone, Copy)]
#[repr(C, align(8))]
pub struct NonMaxU64 {
    #[cfg(target_endian = "big")]
    top: NonMaxU8,
    bottom: [u8; 7],
    #[cfg(target_endian = "little")]
    top: NonMaxU8,
}

impl_nonmax!(NonMaxU64, u64);

/// `u128` with niche for maximum values.
///
/// Equivalent of `NonZeroU128`, but with illegal values of `>= (255 << 120)`, instead of 0.
///
/// Although has `1 << 120` illegal values, this type has only a single niche.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct NonMaxU128 {
    #[cfg(target_endian = "big")]
    top: NonMaxU8,
    bottom: [u8; 15],
    #[cfg(target_endian = "little")]
    top: NonMaxU8,
    // Align same as `u128` which is 16 on new versions of rustc, 8 on older versions
    _align: [u128; 0],
}

impl_nonmax!(NonMaxU128, u128);

/// `usize` with niche for maximum values.
///
/// Equivalent of `NonZeroUsize`, but with illegal values of
/// `>= (255 << ((std::mem::size_of::<usize>() - 1) * 8))`
/// (`255 << 56` on 64-bit systems, `255 << 24` on 32-bit systems), instead of 0.
///
/// Although has `1 << ((std::mem::size_of::<usize>() - 1) * 8)` illegal values,
/// this type has only a single niche.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct NonMaxUsize {
    #[cfg(target_endian = "big")]
    top: NonMaxU8,
    bottom: [u8; size_of::<usize>() - 1],
    #[cfg(target_endian = "little")]
    top: NonMaxU8,
    // Align same as `usize`
    _align: [usize; 0],
}

impl_nonmax!(NonMaxUsize, usize);

// https://doc.rust-lang.org/1.47.0/src/core/convert/num.rs.html#383-407
macro_rules! impl_from_nonmax {
    ($small:ident, $large:ident) => {
        impl From<$small> for $large {
            #[inline]
            fn from(small: $small) -> Self {
                // SAFETY: Smaller input type guarantees the value is valid for larger type
                unsafe { Self::new_unchecked(small.get().into()) }
            }
        }
    };
}

impl_from_nonmax!(NonMaxU8, NonMaxU16);
impl_from_nonmax!(NonMaxU8, NonMaxU32);
impl_from_nonmax!(NonMaxU8, NonMaxU64);
impl_from_nonmax!(NonMaxU8, NonMaxU128);
impl_from_nonmax!(NonMaxU8, NonMaxUsize);
impl_from_nonmax!(NonMaxU16, NonMaxU32);
impl_from_nonmax!(NonMaxU16, NonMaxU64);
impl_from_nonmax!(NonMaxU16, NonMaxU128);
impl_from_nonmax!(NonMaxU16, NonMaxUsize);
impl_from_nonmax!(NonMaxU32, NonMaxU64);
impl_from_nonmax!(NonMaxU32, NonMaxU128);
impl_from_nonmax!(NonMaxU64, NonMaxU128);

// https://doc.rust-lang.org/1.47.0/src/core/convert/num.rs.html#383-407
macro_rules! impl_from_smaller {
    ($small:ident, $nonmax:ident) => {
        impl From<$small> for $nonmax {
            #[inline]
            fn from(small: $small) -> Self {
                // SAFETY: Smaller input type guarantees the value is valid for larger type
                unsafe { Self::new_unchecked(small.into()) }
            }
        }
    };
}

// NB: `From<u16> for NonMaxUsize` is not valid on 16-bit systems.
// `nonmax` crate does provide that impl, but I (@overlookmotel) think it's unsound.
impl_from_smaller!(u8, NonMaxU16);
impl_from_smaller!(u8, NonMaxU32);
impl_from_smaller!(u8, NonMaxU64);
impl_from_smaller!(u8, NonMaxU128);
impl_from_smaller!(u8, NonMaxUsize);
impl_from_smaller!(u16, NonMaxU32);
impl_from_smaller!(u16, NonMaxU64);
impl_from_smaller!(u16, NonMaxU128);
impl_from_smaller!(u32, NonMaxU64);
impl_from_smaller!(u32, NonMaxU128);
impl_from_smaller!(u64, NonMaxU128);
