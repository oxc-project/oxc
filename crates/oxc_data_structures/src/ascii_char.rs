//! All code below copied from unstable `std::ascii::Char`.
//! <https://doc.rust-lang.org/1.81.0/src/core/ascii/ascii_char.rs.html>
//!
//! Only modifications are:
//! * Adding aliases for common characters.
//! * Omitting `impl [AsciiChar]` due to orphan rules.
//! * A few adjustments to work around lack of unstable features + features currently unavailable
//!   in our MSRV.

use std::fmt::{self, Debug, Display, Formatter, Write};

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(u8)]
pub enum AsciiChar {
    /// U+0000 (The default variant)
    Null = 0,
    /// U+0001
    StartOfHeading = 1,
    /// U+0002
    StartOfText = 2,
    /// U+0003
    EndOfText = 3,
    /// U+0004
    EndOfTransmission = 4,
    /// U+0005
    Enquiry = 5,
    /// U+0006
    Acknowledge = 6,
    /// U+0007
    Bell = 7,
    /// U+0008
    Backspace = 8,
    /// U+0009
    CharacterTabulation = 9,
    /// U+000A
    LineFeed = 10,
    /// U+000B
    LineTabulation = 11,
    /// U+000C
    FormFeed = 12,
    /// U+000D
    CarriageReturn = 13,
    /// U+000E
    ShiftOut = 14,
    /// U+000F
    ShiftIn = 15,
    /// U+0010
    DataLinkEscape = 16,
    /// U+0011
    DeviceControlOne = 17,
    /// U+0012
    DeviceControlTwo = 18,
    /// U+0013
    DeviceControlThree = 19,
    /// U+0014
    DeviceControlFour = 20,
    /// U+0015
    NegativeAcknowledge = 21,
    /// U+0016
    SynchronousIdle = 22,
    /// U+0017
    EndOfTransmissionBlock = 23,
    /// U+0018
    Cancel = 24,
    /// U+0019
    EndOfMedium = 25,
    /// U+001A
    Substitute = 26,
    /// U+001B
    Escape = 27,
    /// U+001C
    InformationSeparatorFour = 28,
    /// U+001D
    InformationSeparatorThree = 29,
    /// U+001E
    InformationSeparatorTwo = 30,
    /// U+001F
    InformationSeparatorOne = 31,
    /// U+0020
    Space = 32,
    /// U+0021
    ExclamationMark = 33,
    /// U+0022
    QuotationMark = 34,
    /// U+0023
    NumberSign = 35,
    /// U+0024
    DollarSign = 36,
    /// U+0025
    PercentSign = 37,
    /// U+0026
    Ampersand = 38,
    /// U+0027
    Apostrophe = 39,
    /// U+0028
    LeftParenthesis = 40,
    /// U+0029
    RightParenthesis = 41,
    /// U+002A
    Asterisk = 42,
    /// U+002B
    PlusSign = 43,
    /// U+002C
    Comma = 44,
    /// U+002D
    HyphenMinus = 45,
    /// U+002E
    FullStop = 46,
    /// U+002F
    Solidus = 47,
    /// U+0030
    Digit0 = 48,
    /// U+0031
    Digit1 = 49,
    /// U+0032
    Digit2 = 50,
    /// U+0033
    Digit3 = 51,
    /// U+0034
    Digit4 = 52,
    /// U+0035
    Digit5 = 53,
    /// U+0036
    Digit6 = 54,
    /// U+0037
    Digit7 = 55,
    /// U+0038
    Digit8 = 56,
    /// U+0039
    Digit9 = 57,
    /// U+003A
    Colon = 58,
    /// U+003B
    Semicolon = 59,
    /// U+003C
    LessThanSign = 60,
    /// U+003D
    EqualsSign = 61,
    /// U+003E
    GreaterThanSign = 62,
    /// U+003F
    QuestionMark = 63,
    /// U+0040
    CommercialAt = 64,
    /// U+0041
    CapitalA = 65,
    /// U+0042
    CapitalB = 66,
    /// U+0043
    CapitalC = 67,
    /// U+0044
    CapitalD = 68,
    /// U+0045
    CapitalE = 69,
    /// U+0046
    CapitalF = 70,
    /// U+0047
    CapitalG = 71,
    /// U+0048
    CapitalH = 72,
    /// U+0049
    CapitalI = 73,
    /// U+004A
    CapitalJ = 74,
    /// U+004B
    CapitalK = 75,
    /// U+004C
    CapitalL = 76,
    /// U+004D
    CapitalM = 77,
    /// U+004E
    CapitalN = 78,
    /// U+004F
    CapitalO = 79,
    /// U+0050
    CapitalP = 80,
    /// U+0051
    CapitalQ = 81,
    /// U+0052
    CapitalR = 82,
    /// U+0053
    CapitalS = 83,
    /// U+0054
    CapitalT = 84,
    /// U+0055
    CapitalU = 85,
    /// U+0056
    CapitalV = 86,
    /// U+0057
    CapitalW = 87,
    /// U+0058
    CapitalX = 88,
    /// U+0059
    CapitalY = 89,
    /// U+005A
    CapitalZ = 90,
    /// U+005B
    LeftSquareBracket = 91,
    /// U+005C
    ReverseSolidus = 92,
    /// U+005D
    RightSquareBracket = 93,
    /// U+005E
    CircumflexAccent = 94,
    /// U+005F
    LowLine = 95,
    /// U+0060
    GraveAccent = 96,
    /// U+0061
    SmallA = 97,
    /// U+0062
    SmallB = 98,
    /// U+0063
    SmallC = 99,
    /// U+0064
    SmallD = 100,
    /// U+0065
    SmallE = 101,
    /// U+0066
    SmallF = 102,
    /// U+0067
    SmallG = 103,
    /// U+0068
    SmallH = 104,
    /// U+0069
    SmallI = 105,
    /// U+006A
    SmallJ = 106,
    /// U+006B
    SmallK = 107,
    /// U+006C
    SmallL = 108,
    /// U+006D
    SmallM = 109,
    /// U+006E
    SmallN = 110,
    /// U+006F
    SmallO = 111,
    /// U+0070
    SmallP = 112,
    /// U+0071
    SmallQ = 113,
    /// U+0072
    SmallR = 114,
    /// U+0073
    SmallS = 115,
    /// U+0074
    SmallT = 116,
    /// U+0075
    SmallU = 117,
    /// U+0076
    SmallV = 118,
    /// U+0077
    SmallW = 119,
    /// U+0078
    SmallX = 120,
    /// U+0079
    SmallY = 121,
    /// U+007A
    SmallZ = 122,
    /// U+007B
    LeftCurlyBracket = 123,
    /// U+007C
    VerticalLine = 124,
    /// U+007D
    RightCurlyBracket = 125,
    /// U+007E
    Tilde = 126,
    /// U+007F
    Delete = 127,
}

// Aliases. These are not in `std::ascii::Char` implementation.
#[expect(non_upper_case_globals)]
impl AsciiChar {
    pub const Tab: AsciiChar = AsciiChar::CharacterTabulation;
    pub const SingleQuote: AsciiChar = AsciiChar::Apostrophe;
    pub const DoubleQuote: AsciiChar = AsciiChar::QuotationMark;
}

impl AsciiChar {
    /// Creates an ascii character from the byte `b`,
    /// or returns `None` if it's too large.
    #[inline]
    pub const fn from_u8(b: u8) -> Option<Self> {
        if b <= 127 {
            // SAFETY: Just checked that `b` is in-range
            Some(unsafe { Self::from_u8_unchecked(b) })
        } else {
            None
        }
    }

    /// Creates an ASCII character from the byte `b`, without checking whether it's valid.
    ///
    /// # SAFETY
    /// `b` must be in range `0..=127`.
    #[inline]
    #[expect(clippy::missing_safety_doc)] // Clippy is wrong
    pub const unsafe fn from_u8_unchecked(b: u8) -> Self {
        debug_assert!(b <= 127);
        // SAFETY: Our safety precondition is that `b` is in-range
        unsafe { std::mem::transmute::<u8, Self>(b) }
    }

    /// When passed the *number* `0`, `1`, …, `9`, returns the *character*
    /// `'0'`, `'1'`, …, `'9'` respectively.
    ///
    /// If `d >= 10`, returns `None`.
    #[inline]
    pub const fn digit(d: u8) -> Option<Self> {
        if d < 10 {
            // SAFETY: Just checked it's in-range.
            Some(unsafe { Self::digit_unchecked(d) })
        } else {
            None
        }
    }

    /// When passed the *number* `0`, `1`, …, `9`, returns the *character*
    /// `'0'`, `'1'`, …, `'9'` respectively, without checking that it's in-range.
    ///
    /// # SAFETY
    ///
    /// This is immediate UB if called with `d > 64`.
    ///
    /// If `d >= 10` and `d <= 64`, this is allowed to return any value or panic.
    /// Notably, it should not be expected to return hex digits, or any other
    /// reasonable extension of the decimal digits.
    ///
    /// (This lose safety condition is intended to simplify soundness proofs
    /// when writing code using this method, since the implementation doesn't
    /// need something really specific, not to make those other arguments do
    /// something useful. It might be tightened before stabilization.)
    #[inline]
    #[expect(clippy::missing_safety_doc)] // Clippy is wrong
    pub const unsafe fn digit_unchecked(d: u8) -> Self {
        debug_assert!(d < 10);

        // SAFETY: `'0'` through `'9'` are U+00030 through U+0039,
        // so because `d` must be 64 or less the addition can return at most
        // 112 (0x70), which doesn't overflow and is within the ASCII range.
        unsafe {
            // `std::ascii::Char` uses `u8::unchecked_add`, but this is unavailable in our current MSRV.
            // When we bump our MSRV to >= 1.79.0, clippy will raise a warning on the dummy
            // `_unchecked_add_is_unsupported` function below.
            // Then delete the dummy function, and replace `let Some(byte) = ...` line with:
            // `let byte = b'0'.unchecked_add(d);`
            #[cfg(clippy)]
            #[expect(clippy::incompatible_msrv)]
            unsafe fn _unchecked_add_is_unsupported() {
                let _ = b'0'.unchecked_add(1);
            }

            let Some(byte) = b'0'.checked_add(d) else { std::hint::unreachable_unchecked() };
            Self::from_u8_unchecked(byte)
        }
    }

    /// Gets this ASCII character as a byte.
    #[inline]
    pub const fn to_u8(self) -> u8 {
        self as u8
    }

    /// Gets this ASCII character as a `char` Unicode Scalar Value.
    #[inline]
    pub const fn to_char(self) -> char {
        self as u8 as char
    }

    /// Views this ASCII character as a one-code-unit UTF-8 `str`.
    #[inline]
    pub const fn as_str(&self) -> &str {
        let slice = std::slice::from_ref(self);
        ascii_slice_as_str(slice)
    }
}

#[inline]
const fn ascii_slice_as_str(slice: &[AsciiChar]) -> &str {
    let str_ptr = std::ptr::from_ref(slice) as *const str;
    // SAFETY: Each ASCII codepoint in UTF-8 is encoded as one single-byte
    // code unit having the same value as the ASCII byte.
    unsafe { &*str_ptr }
}

macro_rules! into_int_impl {
    ($($ty:ty)*) => {
        $(
            impl From<AsciiChar> for $ty {
                #[inline]
                #[allow(clippy::cast_lossless)]
                fn from(chr: AsciiChar) -> $ty {
                    chr as u8 as $ty
                }
            }
        )*
    }
}

into_int_impl!(u8 u16 u32 u64 u128 char);

impl Display for AsciiChar {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        <str as Display>::fmt(self.as_str(), f)
    }
}

impl Debug for AsciiChar {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        #[inline]
        fn backslash(a: AsciiChar) -> ([AsciiChar; 4], u8) {
            ([AsciiChar::ReverseSolidus, a, AsciiChar::Null, AsciiChar::Null], 2)
        }

        let (buf, len) = match self {
            AsciiChar::Null => backslash(AsciiChar::Digit0),
            AsciiChar::CharacterTabulation => backslash(AsciiChar::SmallT),
            AsciiChar::CarriageReturn => backslash(AsciiChar::SmallR),
            AsciiChar::LineFeed => backslash(AsciiChar::SmallN),
            AsciiChar::ReverseSolidus => backslash(AsciiChar::ReverseSolidus),
            AsciiChar::Apostrophe => backslash(AsciiChar::Apostrophe),
            _ => {
                let byte = self.to_u8();
                #[expect(clippy::if_not_else)]
                if !byte.is_ascii_control() {
                    ([*self, AsciiChar::Null, AsciiChar::Null, AsciiChar::Null], 1)
                } else {
                    const HEX_DIGITS: [AsciiChar; 16] = [
                        AsciiChar::Digit0,
                        AsciiChar::Digit1,
                        AsciiChar::Digit2,
                        AsciiChar::Digit3,
                        AsciiChar::Digit4,
                        AsciiChar::Digit5,
                        AsciiChar::Digit6,
                        AsciiChar::Digit7,
                        AsciiChar::Digit8,
                        AsciiChar::Digit9,
                        AsciiChar::SmallA,
                        AsciiChar::SmallB,
                        AsciiChar::SmallC,
                        AsciiChar::SmallD,
                        AsciiChar::SmallE,
                        AsciiChar::SmallF,
                    ];

                    let hi = HEX_DIGITS[usize::from(byte >> 4)];
                    let lo = HEX_DIGITS[usize::from(byte & 0xf)];
                    ([AsciiChar::ReverseSolidus, AsciiChar::SmallX, hi, lo], 4)
                }
            }
        };

        f.write_char('\'')?;
        for byte in &buf[..len as usize] {
            f.write_str(byte.as_str())?;
        }
        f.write_char('\'')
    }
}
