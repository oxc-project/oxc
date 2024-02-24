#[rustfmt::skip]
pub fn is_keyword(s: &str) -> bool {
    matches!(s, "as" | "do" | "if" | "in" | "is" | "of"
            | "any" | "for" | "get" | "let" | "new" | "out" | "set" | "try" | "var"
            | "case" | "else" | "enum" | "from" | "meta" | "null" | "this" | "true" | "type" | "void" | "with"
            | "async" | "await" | "break" | "catch" | "class" | "const" | "false" | "infer" | "keyof" | "never" | "super" | "throw" | "using" | "while" | "yield"
            | "assert" | "bigint" | "delete" | "export" | "global" | "import" | "module" | "number" | "object" | "public" | "return" | "static" | "string" | "switch" | "symbol" | "target" | "typeof" | "unique"
            | "asserts" | "boolean" | "declare" | "default" | "extends" | "finally" | "package" | "private" | "require" | "unknown" | "abstract"
            | "accessor" | "continue" | "debugger" | "function" | "override" | "readonly" | "interface" | "intrinsic" | "namespace" | "protected" | "satisfies" | "undefined"
            | "implements" | "instanceof"
            | "constructor"
    )
}
