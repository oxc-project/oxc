#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub struct Diagnostic {
    pub off: u32,
    pub len: u32,
    pub code: DiagCode,
    pub severity: DiagSeverity,
}

#[repr(u16)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Debug)]
pub enum DiagCode {
    #[default]
    Ok = 0,
    UnterminatedString = 1,
    UnterminatedTemplate = 2,
    UnterminatedBlockComment = 3,
    UnterminatedRegexp = 4,
    LineTerminatorInRegexp = 5,
    InvalidUtf8 = 6,
    InvalidUnicodeEscape = 7,
    InvalidIdentifierEscape = 8,
    InvalidNumericSeparator = 9,
    InvalidBigint = 10,
    InvalidNumericLiteral = 11,
    InvalidHashbangPosition = 12,
    InvalidRegexpFlag = 13,
    DuplicateRegexpFlag = 14,
    InvalidRegexpGrammar = 15,
    OracleDepthExceeded = 16,
    AllocationLimitExceeded = 17,
    UnexpectedCharacter = 18,
    LineTerminatorInString = 19,
    HtmlCommentInModule = 20,
    UnterminatedJsxElement = 21,
    UnterminatedJsxTag = 22,
    UnterminatedJsxContainer = 23,
    JsxClosingTagMismatch = 24,
    JsxTextInvalidCharacter = 25,
}

#[repr(u16)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Debug)]
pub enum DiagSeverity {
    #[default]
    Error = 0,
    Warning = 1,
}
