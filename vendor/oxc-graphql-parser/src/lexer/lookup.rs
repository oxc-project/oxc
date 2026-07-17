/// Classification of a leading byte, used to dispatch token lexing.
///
/// One 256-entry table lookup replaces the chain of per-character checks the
/// lexer would otherwise run for every token start.
#[derive(Clone, Copy)]
#[repr(u8)]
pub(crate) enum ByteClass {
    Other,
    Bang,
    Dollar,
    Amp,
    LParen,
    RParen,
    Comma,
    Colon,
    Eq,
    At,
    LBracket,
    RBracket,
    LCurly,
    RCurly,
    Pipe,
    /// <https://spec.graphql.org/October2021/#NameStart>
    Name,
    Zero,
    Digit,
    Quote,
    Hash,
    Dot,
    Minus,
    /// <https://spec.graphql.org/October2021/#WhiteSpace> and
    /// <https://spec.graphql.org/October2021/#LineTerminator>
    Whitespace,
    /// First byte of a UTF-8 byte order mark.
    Bom,
}

static BYTE_CLASS: [ByteClass; 256] = byte_class_lut();

#[inline]
pub(crate) fn byte_class(c: u8) -> ByteClass {
    BYTE_CLASS[c as usize]
}

/// <https://spec.graphql.org/October2021/#NameStart>
#[inline]
pub(crate) fn is_namestart(c: u8) -> bool {
    matches!(byte_class(c), ByteClass::Name)
}

const fn byte_class_lut() -> [ByteClass; 256] {
    let mut lut = [ByteClass::Other; 256];

    lut[b'!' as usize] = ByteClass::Bang;
    lut[b'$' as usize] = ByteClass::Dollar;
    lut[b'&' as usize] = ByteClass::Amp;
    lut[b'(' as usize] = ByteClass::LParen;
    lut[b')' as usize] = ByteClass::RParen;
    lut[b',' as usize] = ByteClass::Comma;
    lut[b':' as usize] = ByteClass::Colon;
    lut[b'=' as usize] = ByteClass::Eq;
    lut[b'@' as usize] = ByteClass::At;
    lut[b'[' as usize] = ByteClass::LBracket;
    lut[b']' as usize] = ByteClass::RBracket;
    lut[b'{' as usize] = ByteClass::LCurly;
    lut[b'}' as usize] = ByteClass::RCurly;
    lut[b'|' as usize] = ByteClass::Pipe;

    let mut c = b'a';
    while c <= b'z' {
        lut[c as usize] = ByteClass::Name;
        c += 1;
    }
    let mut c = b'A';
    while c <= b'Z' {
        lut[c as usize] = ByteClass::Name;
        c += 1;
    }
    lut[b'_' as usize] = ByteClass::Name;

    lut[b'0' as usize] = ByteClass::Zero;
    let mut c = b'1';
    while c <= b'9' {
        lut[c as usize] = ByteClass::Digit;
        c += 1;
    }

    lut[b'"' as usize] = ByteClass::Quote;
    lut[b'#' as usize] = ByteClass::Hash;
    lut[b'.' as usize] = ByteClass::Dot;
    lut[b'-' as usize] = ByteClass::Minus;

    lut[b'\t' as usize] = ByteClass::Whitespace;
    lut[b' ' as usize] = ByteClass::Whitespace;
    lut[b'\n' as usize] = ByteClass::Whitespace;
    lut[b'\r' as usize] = ByteClass::Whitespace;

    lut[0xEF] = ByteClass::Bom;

    lut
}
