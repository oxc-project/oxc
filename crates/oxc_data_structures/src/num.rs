//! `U*` types which are limited in range, so top bit is always unset.
//!
//! e.g. [`U7`] represents an unsigned integer in the range 0 - `i8::MAX as u8` (0 - 127).
//!
//! These types have 3 useful features, compared to their primitive counterparts:
//!
//! 1. They allow expressing the legal range of values in the type system.
//!
//! 2. The compiler understands this, so can prove that e.g. `u7.to_u8() * 2` cannot wrap around.
//!
//! 3. [`U7`], [`U15`], [`U31`] and [`U63`] all have niches.
//!    So `Option<U7>` is 1 byte, same as `u8`; `Option<U31>` is 4 bytes, same as `u32`.
//!
//! These types all have the same memory layout as their primitive counterparts.
//! e.g. [`U31`] has same layout as [`u32`].
//!
//! Therefore:
//! * `to_*` and `from_*_unchecked` methods are zero-cost.
//! * `from_*_checked` methods are branchless.
//! * `from_*` methods are just a single bounds check.
//! * `from_*` and `from_*_checked` methods are zero cost if the value is statically known.
//!
//! <https://godbolt.org/z/M8ca4xTa8>

// All methods marked `#[inline(always)]` because they're either no-ops, or very cheap
#![expect(clippy::inline_always)]

use std::{
    cmp::Ordering,
    convert::TryFrom,
    error::Error,
    fmt::{self, Debug, Display},
    hash::{Hash, Hasher},
    hint::unreachable_unchecked,
};

/// Error type for failed conversions.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct TryFromIntError(());

impl Display for TryFromIntError {
    #[expect(deprecated)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(self.description(), f)
    }
}

impl Error for TryFromIntError {
    fn description(&self) -> &'static str {
        "out of range integral type conversion attempted"
    }
}

/// Inner type of [`U7`].
#[derive(Clone, Copy)]
#[repr(u8)]
enum U7Inner {
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
}

impl U7Inner {
    /// Convert [`U7Inner`] to [`u8`].
    #[inline(always)]
    pub const fn to_u8(self) -> u8 {
        self as u8
    }

    /// Create a [`U7`] from a [`u8`], without check for validity.
    ///
    /// # SAFETY
    /// Caller must ensure `n` is less than or equal to 127.
    #[inline(always)]
    pub const unsafe fn from_u8_unchecked(n: u8) -> Self {
        match n {
            0 => Self::_0,
            1 => Self::_1,
            2 => Self::_2,
            3 => Self::_3,
            4 => Self::_4,
            5 => Self::_5,
            6 => Self::_6,
            7 => Self::_7,
            8 => Self::_8,
            9 => Self::_9,
            10 => Self::_10,
            11 => Self::_11,
            12 => Self::_12,
            13 => Self::_13,
            14 => Self::_14,
            15 => Self::_15,
            16 => Self::_16,
            17 => Self::_17,
            18 => Self::_18,
            19 => Self::_19,
            20 => Self::_20,
            21 => Self::_21,
            22 => Self::_22,
            23 => Self::_23,
            24 => Self::_24,
            25 => Self::_25,
            26 => Self::_26,
            27 => Self::_27,
            28 => Self::_28,
            29 => Self::_29,
            30 => Self::_30,
            31 => Self::_31,
            32 => Self::_32,
            33 => Self::_33,
            34 => Self::_34,
            35 => Self::_35,
            36 => Self::_36,
            37 => Self::_37,
            38 => Self::_38,
            39 => Self::_39,
            40 => Self::_40,
            41 => Self::_41,
            42 => Self::_42,
            43 => Self::_43,
            44 => Self::_44,
            45 => Self::_45,
            46 => Self::_46,
            47 => Self::_47,
            48 => Self::_48,
            49 => Self::_49,
            50 => Self::_50,
            51 => Self::_51,
            52 => Self::_52,
            53 => Self::_53,
            54 => Self::_54,
            55 => Self::_55,
            56 => Self::_56,
            57 => Self::_57,
            58 => Self::_58,
            59 => Self::_59,
            60 => Self::_60,
            61 => Self::_61,
            62 => Self::_62,
            63 => Self::_63,
            64 => Self::_64,
            65 => Self::_65,
            66 => Self::_66,
            67 => Self::_67,
            68 => Self::_68,
            69 => Self::_69,
            70 => Self::_70,
            71 => Self::_71,
            72 => Self::_72,
            73 => Self::_73,
            74 => Self::_74,
            75 => Self::_75,
            76 => Self::_76,
            77 => Self::_77,
            78 => Self::_78,
            79 => Self::_79,
            80 => Self::_80,
            81 => Self::_81,
            82 => Self::_82,
            83 => Self::_83,
            84 => Self::_84,
            85 => Self::_85,
            86 => Self::_86,
            87 => Self::_87,
            88 => Self::_88,
            89 => Self::_89,
            90 => Self::_90,
            91 => Self::_91,
            92 => Self::_92,
            93 => Self::_93,
            94 => Self::_94,
            95 => Self::_95,
            96 => Self::_96,
            97 => Self::_97,
            98 => Self::_98,
            99 => Self::_99,
            100 => Self::_100,
            101 => Self::_101,
            102 => Self::_102,
            103 => Self::_103,
            104 => Self::_104,
            105 => Self::_105,
            106 => Self::_106,
            107 => Self::_107,
            108 => Self::_108,
            109 => Self::_109,
            110 => Self::_110,
            111 => Self::_111,
            112 => Self::_112,
            113 => Self::_113,
            114 => Self::_114,
            115 => Self::_115,
            116 => Self::_116,
            117 => Self::_117,
            118 => Self::_118,
            119 => Self::_119,
            120 => Self::_120,
            121 => Self::_121,
            122 => Self::_122,
            123 => Self::_123,
            124 => Self::_124,
            125 => Self::_125,
            126 => Self::_126,
            127 => Self::_127,
            // SAFETY: Caller guarantees `n` is `<= 127`
            _ => unsafe { unreachable_unchecked() },
        }
    }
}

/// Type representing an unsigned integer in the range 0 - `i8::MAX as u8`.
///
/// i.e. a [`u8`] where top bit is always unset.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct U7(U7Inner);

impl U7 {
    /// Minimum [`u8`] value for a [`U7`].
    pub const MIN_PRIMITIVE: u8 = 0;

    /// Maximum [`u8`] value for a [`U7`].
    pub const MAX_PRIMITIVE: u8 = i8::MAX as u8;

    /// Minimum [`U7`] value.
    pub const MIN: Self = Self::from_u8(Self::MIN_PRIMITIVE);

    /// Maximum [`U7`] value.
    pub const MAX: Self = Self::from_u8(Self::MAX_PRIMITIVE);

    /// Zero [`U7`] value.
    pub const ZERO: Self = Self::MIN;

    /// Convert [`U7`] to [`u8`].
    #[inline(always)]
    pub const fn to_u8(self) -> u8 {
        self.0.to_u8()
    }

    /// Create a [`U7`] from a [`u8`].
    ///
    /// # Panics
    /// Panics if `n` is greater than [`U7::MAX_PRIMITIVE`].
    #[inline(always)]
    pub const fn from_u8(n: u8) -> Self {
        Self::from_u8_checked(n).expect("Out of range")
    }

    /// Create a [`U7`] from a [`u8`], with check for validity.
    ///
    /// Returns `None` if `n` is greater than [`U7::MAX_PRIMITIVE`].
    #[inline(always)]
    pub const fn from_u8_checked(n: u8) -> Option<Self> {
        if n <= Self::MAX_PRIMITIVE {
            // SAFETY: We just checked that `n` is in range
            Some(unsafe { Self::from_u8_unchecked(n) })
        } else {
            None
        }
    }

    /// Create a [`U7`] from a [`u8`], without check for validity.
    ///
    /// # SAFETY
    /// Caller must ensure `n` is less than or equal to [`U7::MAX_PRIMITIVE`].
    #[inline(always)]
    pub const unsafe fn from_u8_unchecked(n: u8) -> Self {
        // SAFETY: Caller guarantees `n <= U7::MAX_PRIMITIVE`
        unsafe { Self(U7Inner::from_u8_unchecked(n)) }
    }
}

impl From<U7> for u8 {
    #[inline(always)]
    fn from(u: U7) -> u8 {
        u.to_u8()
    }
}

impl TryFrom<u8> for U7 {
    type Error = TryFromIntError;

    #[inline(always)]
    fn try_from(n: u8) -> Result<Self, TryFromIntError> {
        match Self::from_u8_checked(n) {
            Some(u) => Ok(u),
            None => Err(TryFromIntError(())),
        }
    }
}

/// Type representing an unsigned integer in the range 0 - `i16::MAX as u16`.
///
/// i.e. a [`u16`] where top bit is always unset.
//
// Note: Set field order depending on endianness. This makes conversion methods very cheap,
// as `U15` has same memory layout as `u16`.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct U15 {
    align: [u16; 0],
    #[cfg(target_endian = "little")]
    low: u8,
    high: U7,
    #[cfg(target_endian = "big")]
    low: u8,
}

impl U15 {
    /// Minimum [`u16`] value for a [`U15`].
    pub const MIN_PRIMITIVE: u16 = 0;

    /// Maximum [`u16`] value for a [`U15`].
    pub const MAX_PRIMITIVE: u16 = i16::MAX as u16;

    /// Minimum [`U15`] value.
    pub const MIN: Self = Self::from_u16(Self::MIN_PRIMITIVE);

    /// Maximum [`U15`] value.
    pub const MAX: Self = Self::from_u16(Self::MAX_PRIMITIVE);

    /// Zero [`U15`] value.
    pub const ZERO: Self = Self::MIN;

    /// Convert [`U15`] to [`u16`].
    #[inline(always)]
    pub const fn to_u16(self) -> u16 {
        self.low as u16 | ((self.high.to_u8() as u16) << 8)
    }

    /// Create a [`U15`] from a [`u16`].
    ///
    /// # Panics
    /// Panics if `n` is greater than [`U15::MAX_PRIMITIVE`].
    #[inline(always)]
    pub const fn from_u16(n: u16) -> Self {
        Self::from_u16_checked(n).expect("Out of range")
    }

    /// Create a [`U15`] from a [`u16`], with check for validity.
    ///
    /// Returns `None` if `n` is greater than [`U15::MAX_PRIMITIVE`].
    #[inline(always)]
    pub const fn from_u16_checked(n: u16) -> Option<Self> {
        if n <= Self::MAX_PRIMITIVE {
            // SAFETY: We just checked that `n` is in range
            Some(unsafe { Self::from_u16_unchecked(n) })
        } else {
            None
        }
    }

    /// Create a [`U15`] from a [`u16`], without check for validity.
    ///
    /// # SAFETY
    /// Caller must ensure `n` is less than or equal to [`U15::MAX_PRIMITIVE`].
    #[inline(always)]
    #[expect(clippy::cast_possible_truncation)]
    #[cfg_attr(target_endian = "big", expect(clippy::inconsistent_struct_constructor))]
    pub const unsafe fn from_u16_unchecked(n: u16) -> Self {
        let low = n as u8;
        // SAFETY: Caller guarantees `n <= U15::MAX_PRIMITIVE`, so top 8 bits are `<= U7::MAX_PRIMITIVE`
        let high = unsafe { U7::from_u8_unchecked((n >> 8) as u8) };
        Self { align: [], low, high }
    }
}

impl From<U15> for u16 {
    #[inline(always)]
    fn from(u: U15) -> u16 {
        u.to_u16()
    }
}

impl TryFrom<u16> for U15 {
    type Error = TryFromIntError;

    #[inline(always)]
    fn try_from(n: u16) -> Result<Self, TryFromIntError> {
        match Self::from_u16_checked(n) {
            Some(u) => Ok(u),
            None => Err(TryFromIntError(())),
        }
    }
}

/// Type representing an unsigned integer in the range 0 - `i32::MAX as u32`.
///
/// i.e. a [`u32`] where top bit is always unset.
//
// Note: Set field order depending on endianness. This makes conversion methods very cheap,
// as `U31` has same memory layout as `u32`.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct U31 {
    align: [u32; 0],
    #[cfg(target_endian = "little")]
    low: u16,
    high: U15,
    #[cfg(target_endian = "big")]
    low: u16,
}

impl U31 {
    /// Minimum [`u32`] value for a [`U31`].
    pub const MIN_PRIMITIVE: u32 = 0;

    /// Maximum [`u32`] value for a [`U31`].
    pub const MAX_PRIMITIVE: u32 = i32::MAX as u32;

    /// Minimum [`U31`] value.
    pub const MIN: Self = Self::from_u32(Self::MIN_PRIMITIVE);

    /// Maximum [`U31`] value.
    pub const MAX: Self = Self::from_u32(Self::MAX_PRIMITIVE);

    /// Zero [`U31`] value.
    pub const ZERO: Self = Self::MIN;

    /// Convert [`U31`] to [`u32`].
    #[inline(always)]
    pub const fn to_u32(self) -> u32 {
        self.low as u32 | ((self.high.to_u16() as u32) << 16)
    }

    /// Create a [`U31`] from a [`u32`].
    ///
    /// # Panics
    /// Panics if `n` is greater than [`U31::MAX_PRIMITIVE`].
    #[inline(always)]
    pub const fn from_u32(n: u32) -> Self {
        Self::from_u32_checked(n).expect("Out of range")
    }

    /// Create a [`U31`] from a [`u32`], with check for validity.
    ///
    /// Returns `None` if `n` is greater than [`U31::MAX_PRIMITIVE`].
    #[inline(always)]
    pub const fn from_u32_checked(n: u32) -> Option<Self> {
        if n <= Self::MAX_PRIMITIVE {
            // SAFETY: We just checked that `n` is in range
            Some(unsafe { Self::from_u32_unchecked(n) })
        } else {
            None
        }
    }

    /// Create a [`U31`] from a [`u32`], without check for validity.
    ///
    /// # SAFETY
    /// Caller must ensure `n` is less than or equal to [`U31::MAX_PRIMITIVE`].
    #[inline(always)]
    #[expect(clippy::cast_possible_truncation)]
    #[cfg_attr(target_endian = "big", expect(clippy::inconsistent_struct_constructor))]
    pub const unsafe fn from_u32_unchecked(n: u32) -> Self {
        let low = n as u16;
        // SAFETY: Caller guarantees `n <= U31::MAX_PRIMITIVE`, so top 16 bits are `<= U15::MAX_PRIMITIVE`
        let high = unsafe { U15::from_u16_unchecked((n >> 16) as u16) };
        Self { align: [], low, high }
    }
}

impl From<U31> for u32 {
    #[inline(always)]
    fn from(u: U31) -> u32 {
        u.to_u32()
    }
}

impl TryFrom<u32> for U31 {
    type Error = TryFromIntError;

    #[inline(always)]
    fn try_from(n: u32) -> Result<Self, TryFromIntError> {
        match Self::from_u32_checked(n) {
            Some(u) => Ok(u),
            None => Err(TryFromIntError(())),
        }
    }
}

/// Type representing an unsigned integer in the range 0 - `i64::MAX as u64`.
///
/// i.e. a [`u64`] where top bit is always unset.
//
// Note: Set field order depending on endianness. This makes conversion methods very cheap,
// as `U63` has same memory layout as `u64`.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct U63 {
    align: [u64; 0],
    #[cfg(target_endian = "little")]
    low: u32,
    high: U31,
    #[cfg(target_endian = "big")]
    low: u32,
}

impl U63 {
    /// Minimum [`u64`] value for a [`U63`].
    pub const MIN_PRIMITIVE: u64 = 0;

    /// Maximum [`u64`] value for a [`U63`].
    pub const MAX_PRIMITIVE: u64 = i64::MAX as u64;

    /// Minimum [`U63`] value.
    pub const MIN: Self = Self::from_u64(Self::MIN_PRIMITIVE);

    /// Maximum [`U63`] value.
    pub const MAX: Self = Self::from_u64(Self::MAX_PRIMITIVE);

    /// Zero [`U63`] value.
    pub const ZERO: Self = Self::MIN;

    /// Convert [`U63`] to [`u64`].
    #[inline(always)]
    pub const fn to_u64(self) -> u64 {
        self.low as u64 | ((self.high.to_u32() as u64) << 32)
    }

    /// Create a [`U63`] from a [`u64`].
    ///
    /// # Panics
    /// Panics if `n` is greater than [`U63::MAX_PRIMITIVE`].
    #[inline(always)]
    pub const fn from_u64(n: u64) -> Self {
        Self::from_u64_checked(n).expect("Out of range")
    }

    /// Create a [`U63`] from a [`u64`], with check for validity.
    ///
    /// Returns `None` if `n` is greater than [`U63::MAX_PRIMITIVE`].
    #[inline(always)]
    pub const fn from_u64_checked(n: u64) -> Option<Self> {
        if n <= Self::MAX_PRIMITIVE {
            // SAFETY: We just checked that `n` is in range
            Some(unsafe { Self::from_u64_unchecked(n) })
        } else {
            None
        }
    }

    /// Create a [`U63`] from a [`u64`], without check for validity.
    ///
    /// # SAFETY
    /// Caller must ensure `n` is less than or equal to [`U63::MAX_PRIMITIVE`].
    #[inline(always)]
    #[expect(clippy::cast_possible_truncation)]
    #[cfg_attr(target_endian = "big", expect(clippy::inconsistent_struct_constructor))]
    pub const unsafe fn from_u64_unchecked(n: u64) -> Self {
        let low = n as u32;
        // SAFETY: Caller guarantees `n <= U63::MAX_PRIMITIVE`, so top 32 bits are `<= U31::MAX_PRIMITIVE`
        let high = unsafe { U31::from_u32_unchecked((n >> 32) as u32) };
        Self { align: [], low, high }
    }
}

impl From<U63> for u64 {
    #[inline(always)]
    fn from(u: U63) -> u64 {
        u.to_u64()
    }
}

impl TryFrom<u64> for U63 {
    type Error = TryFromIntError;

    #[inline(always)]
    fn try_from(n: u64) -> Result<Self, TryFromIntError> {
        match Self::from_u64_checked(n) {
            Some(u) => Ok(u),
            None => Err(TryFromIntError(())),
        }
    }
}

/// Macro to implement `Eq`, `PartialEq`, `Ord`, `PartialOrd`, `Hash`, `Display`, and `Debug`.
///
/// All just delegate to the same methods for their primitive types.
macro_rules! impls {
    ($ty:ident, $to_primitive:ident) => {
        impl Eq for $ty {}

        impl PartialEq for $ty {
            #[inline(always)]
            fn eq(&self, other: &Self) -> bool {
                self.$to_primitive() == other.$to_primitive()
            }
        }

        impl Ord for $ty {
            #[inline(always)]
            fn cmp(&self, other: &Self) -> Ordering {
                self.$to_primitive().cmp(&other.$to_primitive())
            }
        }

        impl PartialOrd for $ty {
            #[inline(always)]
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }

        impl Hash for $ty {
            #[inline(always)]
            fn hash<H: Hasher>(&self, state: &mut H) {
                self.$to_primitive().hash(state);
            }
        }

        impl Display for $ty {
            #[inline(always)]
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                Display::fmt(&self.$to_primitive(), f)
            }
        }

        impl Debug for $ty {
            #[inline(always)]
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                Debug::fmt(&self.$to_primitive(), f)
            }
        }
    };
}

impls!(U7, to_u8);
impls!(U15, to_u16);
impls!(U31, to_u32);
impls!(U63, to_u64);
