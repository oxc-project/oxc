use convert_case::Casing;
use oxc_span::Atom;
use oxc_syntax::unicode_id_start::{is_id_continue, is_id_start};

pub const RESERVED_WORDS_ES3_ONLY: phf::Set<&str> = phf::phf_set![
    "abstract",
    "boolean",
    "byte",
    "char",
    "double",
    "enum",
    "final",
    "float",
    "goto",
    "implements",
    "int",
    "interface",
    "long",
    "native",
    "package",
    "private",
    "protected",
    "public",
    "short",
    "static",
    "synchronized",
    "throws",
    "transient",
    "volatile",
];

const RESERVED_WORD_STRICT: phf::Set<&str> = phf::phf_set![
    "implements",
    "interface",
    "let",
    "package",
    "private",
    "protected",
    "public",
    "static",
    "yield",
];

pub const KEYWORDS: phf::Set<&str> = phf::phf_set![
    "break",
    "case",
    "catch",
    "continue",
    "debugger",
    "default",
    "do",
    "else",
    "finally",
    "for",
    "function",
    "if",
    "return",
    "switch",
    "throw",
    "try",
    "var",
    "const",
    "while",
    "with",
    "new",
    "this",
    "super",
    "class",
    "extends",
    "export",
    "import",
    "null",
    "true",
    "false",
    "in",
    "instanceof",
    "typeof",
    "void",
    "delete",
];

/// https://github.com/babel/babel/blob/ff3481746a830e0e94626de4c4cb075ea5f2f5dc/packages/babel-helper-validator-identifier/src/identifier.ts#L85-L109
pub fn is_identifier_name<T: AsRef<str>>(name: T) -> bool {
    let string = name.as_ref();
    if string.is_empty() {
        return false;
    }
    let mut is_first = true;
    for ch in string.chars() {
        if is_first {
            is_first = false;
            if !is_id_start(ch) {
                return false;
            }
        } else if !is_id_continue(ch) {
            return false;
        }
    }
    true
}

pub fn is_valid_identifier<T: AsRef<str>>(name: T, reserved: bool) -> bool {
    if reserved
        && (KEYWORDS.contains(name.as_ref()) || is_strict_reserved_word(name.as_ref(), true))
    {
        return false;
    }
    is_identifier_name(name)
}

pub fn is_strict_reserved_word<T: AsRef<str>>(name: T, in_module: bool) -> bool {
    is_reserved_word(name.as_ref(), in_module) || RESERVED_WORD_STRICT.contains(name.as_ref())
}

pub fn is_reserved_word<T: AsRef<str>>(name: T, in_module: bool) -> bool {
    (in_module && name.as_ref() == "await") || name.as_ref() == "enum"
}

/// https://github.com/babel/babel/blob/main/packages/babel-types/src/validators/isValidES3Identifier.ts#L35
pub fn is_valid_es3_identifier(name: &Atom) -> bool {
    is_valid_identifier(name, true) && !RESERVED_WORDS_ES3_ONLY.contains(name.as_str())
}

pub fn to_identifier(name: &str) -> String {
    let name =
        name.chars().map(|item| if is_id_continue(item) { item } else { '-' }).collect::<String>();
    let name = name.trim_start_matches(|ch: char| ch.is_ascii_digit() || ch == '-');
    let mut name = name.to_case(convert_case::Case::Camel);
    if !is_valid_identifier(&name, true) {
        name = format!("_{name}");
    }
    if name.is_empty() {
        name
    } else {
        "_".to_string()
    }
}
