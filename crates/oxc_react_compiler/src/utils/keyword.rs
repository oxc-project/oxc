/// Reserved word checks for JavaScript identifiers.
///
/// Port of `Utils/Keyword.ts` from the React Compiler.
///
/// <https://tc39.es/ecma262/multipage/ecmascript-language-lexical-grammar.html#sec-keywords-and-reserved-words>
///
/// Note: `await` and `yield` are contextually allowed as identifiers.
///   await: reserved inside async functions and modules
///   yield: reserved inside generator functions
///
/// Note: `async` is not reserved.
fn is_keyword(name: &str) -> bool {
    matches!(
        name,
        "break"
            | "case"
            | "catch"
            | "class"
            | "const"
            | "continue"
            | "debugger"
            | "default"
            | "delete"
            | "do"
            | "else"
            | "enum"
            | "export"
            | "extends"
            | "false"
            | "finally"
            | "for"
            | "function"
            | "if"
            | "import"
            | "in"
            | "instanceof"
            | "new"
            | "null"
            | "return"
            | "super"
            | "switch"
            | "this"
            | "throw"
            | "true"
            | "try"
            | "typeof"
            | "var"
            | "void"
            | "while"
            | "with"
    )
}

/// Reserved when a module has a 'use strict' directive.
fn is_strict_mode_reserved_word(name: &str) -> bool {
    matches!(
        name,
        "let"
            | "static"
            | "implements"
            | "interface"
            | "package"
            | "private"
            | "protected"
            | "public"
    )
}

/// The names `arguments` and `eval` are not keywords, but they are subject to
/// some restrictions in strict mode code.
fn is_strict_mode_restricted_word(name: &str) -> bool {
    matches!(name, "eval" | "arguments")
}

/// Conservative check for whether an identifier name is reserved or not.
/// We assume that code is written with strict mode.
pub fn is_reserved_word(identifier_name: &str) -> bool {
    is_keyword(identifier_name)
        || is_strict_mode_reserved_word(identifier_name)
        || is_strict_mode_restricted_word(identifier_name)
}
