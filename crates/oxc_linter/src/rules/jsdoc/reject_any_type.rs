use std::ops::Range;

use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::identifier::is_identifier_part_ascii;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{should_ignore_as_internal, should_ignore_as_private},
};

fn reject_any_type_diagnostic(span: Span, type_name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Unexpected `{type_name}` type."))
        .with_help(format!("Replace `{type_name}` with a more specific type."))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct RejectAnyType;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Reports uses of the `any` and `*` types in JSDoc tag types.
    ///
    /// ### Why is this bad?
    ///
    /// `any` and `*` accept every value, so they switch off type checking for whatever
    /// they describe. A more specific type documents what is actually expected and lets
    /// tools catch mistakes.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// /**
    ///  * @param {any} abc
    ///  * @returns {Promise<*>}
    ///  */
    /// function quux (abc) {}
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// /**
    ///  * @param {string} abc
    ///  * @returns {Promise<number>}
    ///  */
    /// function quux (abc) {}
    /// ```
    RejectAnyType,
    jsdoc,
    restriction,
    recommended,
    version = "next",
    short_description = "Reports use of the `any` or `*` type.",
);

/// Tags whose `{...}` is a type expression.
///
/// These are the tags that `eslint-plugin-jsdoc`'s tag structure allows or requires a type
/// for in its default `typescript` mode. Unknown tags are skipped, because without a type
/// parser there is no way to tell whether their `{...}` is a type, e.g. Ramda's `@sig {*} -> *`.
const TAGS_WITH_TYPE: [&str; 29] = [
    "arg",
    "argument",
    "augments",
    "class",
    "const",
    "constant",
    "constructor",
    "enum",
    "exception",
    "extends",
    "implements",
    "member",
    "modifies",
    "module",
    "namespace",
    "param",
    "prop",
    "property",
    "return",
    "returns",
    "satisfies",
    "template",
    "this",
    "throws",
    "type",
    "typedef",
    "var",
    "yield",
    "yields",
];

impl Rule for RejectAnyType {
    fn run_once(&self, ctx: &LintContext) {
        let settings = &ctx.settings().jsdoc;

        for jsdoc in ctx
            .jsdoc()
            .iter_all()
            .filter(|jsdoc| !should_ignore_as_internal(jsdoc, settings))
            .filter(|jsdoc| !should_ignore_as_private(jsdoc, settings))
        {
            for tag in jsdoc.tags() {
                if !TAGS_WITH_TYPE.contains(&tag.kind.parsed()) {
                    continue;
                }

                let Some(type_part) = tag.r#type() else {
                    continue;
                };

                // Inline tags such as `{@link any}` are not types.
                if type_part.parsed().starts_with('@') {
                    continue;
                }

                let raw = type_part.raw();
                for range in find_any_types(raw) {
                    let type_name = &raw[range.clone()];
                    let start =
                        type_part.span.start + u32::try_from(range.start).unwrap_or_default();
                    let end = type_part.span.start + u32::try_from(range.end).unwrap_or_default();
                    ctx.diagnostic(reject_any_type_diagnostic(Span::new(start, end), type_name));
                }
            }
        }
    }
}

/// Parts of a type expression where `{` and `}` need to be tracked.
enum Frame {
    /// The text of a template literal type, e.g. the `prefix-` in `` `prefix-${string}` ``.
    TemplateText,
    /// A `${...}` interpolation, with the number of `{` opened inside it that are not closed yet.
    Interpolation(u32),
}

/// Finds the `any` and `*` types in a JSDoc type expression, returning the byte range of each.
///
/// The text of string literals and template literals is skipped, as is the `*` margin at the
/// start of each line of a multiline type.
fn find_any_types(type_text: &str) -> Vec<Range<usize>> {
    let bytes = type_text.as_bytes();
    let mut found = vec![];
    let mut frames = vec![];
    let mut i = 0;

    while i < bytes.len() {
        let byte = bytes[i];

        if matches!(frames.last(), Some(Frame::TemplateText)) {
            match byte {
                b'`' => {
                    frames.pop();
                }
                b'\\' => i += 1,
                b'$' if bytes.get(i + 1) == Some(&b'{') => {
                    frames.push(Frame::Interpolation(0));
                    i += 1;
                }
                _ => {}
            }
            i += 1;
            continue;
        }

        match byte {
            b'\'' | b'"' => {
                i = skip_string_literal(bytes, i);
                continue;
            }
            b'`' => frames.push(Frame::TemplateText),
            b'{' => {
                if let Some(Frame::Interpolation(open_braces)) = frames.last_mut() {
                    *open_braces += 1;
                }
            }
            b'}' => match frames.last_mut() {
                Some(Frame::Interpolation(0)) => {
                    frames.pop();
                }
                Some(Frame::Interpolation(open_braces)) => *open_braces -= 1,
                _ => {}
            },
            b'\n' => {
                let line = &bytes[i + 1..];
                let indent = line.iter().take_while(|byte| matches!(byte, b' ' | b'\t')).count();
                if line.get(indent) == Some(&b'*') {
                    i += indent + 1;
                }
            }
            b'*' => found.push(i..i + 1),
            _ if is_identifier_byte(byte) => {
                let start = i;
                while i < bytes.len() && is_identifier_byte(bytes[i]) {
                    i += 1;
                }
                if &bytes[start..i] == b"any" && is_type_name(bytes, start..i) {
                    found.push(start..i);
                }
                continue;
            }
            _ => {}
        }
        i += 1;
    }

    found
}

/// Returns the index just past the string literal that starts at `start`.
fn skip_string_literal(bytes: &[u8], start: usize) -> usize {
    let quote = bytes[start];
    let mut i = start + 1;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => i += 1,
            byte if byte == quote => return i + 1,
            _ => {}
        }
        i += 1;
    }
    bytes.len()
}

/// Non-ASCII bytes count as identifier bytes, so a multi-byte character is never split.
fn is_identifier_byte(byte: u8) -> bool {
    !byte.is_ascii() || is_identifier_part_ascii(char::from(byte))
}

/// Returns whether the identifier at `range` is used as a type, rather than as an object key,
/// a parameter name, a method name, or a member of a namepath such as `foo.any`.
fn is_type_name(bytes: &[u8], range: Range<usize>) -> bool {
    let before = bytes[..range.start].trim_ascii_end();
    let after = bytes[range.end..].trim_ascii_start();

    // `...any` is a rest type, but `foo.any`, `foo#any` and `foo~any` are namepath members.
    if (before.ends_with(b".") && !before.ends_with(b"..."))
        || before.ends_with(b"#")
        || before.ends_with(b"~")
    {
        return false;
    }

    match after.first() {
        // A method name, e.g. `{ any(): void }`.
        Some(b'(') => false,
        // A key or parameter name, e.g. `{ any: string }`, unless it is the true branch of a
        // conditional type, e.g. `T extends U ? any : never`.
        Some(b':') => before.ends_with(b"?"),
        // An optional key, e.g. `{ any?: string }`.
        Some(b'?') => !after[1..].trim_ascii_start().starts_with(b":"),
        _ => true,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            "
                    /**
                     * @param {SomeType} abc
                     */
                    function quux () {}
                  ",
            None,
            None,
        ),
        (
            "
                    /**
                     * @param {string} any
                     * @param {anything|Many|any_thing|$any|anyÄ} abc
                     */
                    function quux () {}
                  ",
            None,
            None,
        ),
        (
            "
                    /**
                     * @param {{any: string, any?: number, any(): void}} abc
                     * @param {(any: string) => void} def
                     */
                    function quux () {}
                  ",
            None,
            None,
        ),
        (
            "
                    /**
                     * @param {'any'|\"*\"|`any-*`} abc
                     * @param {foo.any|foo#any|foo~any|import('./any').Foo} def
                     */
                    function quux () {}
                  ",
            None,
            None,
        ),
        (
            "
                    /**
                     * @param {{
                     *   foo: string
                     * }} abc
                     */
                    function quux () {}
                  ",
            None,
            None,
        ),
        (
            "
                    /**
                     * @see {@link any}
                     * @requires {any}
                     * @sig {*} -> {*}
                     * @returns {@link any}
                     */
                    function quux () {}
                  ",
            None,
            None,
        ),
        (
            "
                    /**
                     * @private
                     * @param {any} abc
                     */
                    function quux () {}
                  ",
            None,
            Some(serde_json::json!({ "settings": { "jsdoc": { "ignorePrivate": true } } })),
        ),
    ];

    let fail = vec![
        (
            "
                    /**
                     * @param {any} abc
                     */
                    function quux () {}
                  ",
            None,
            None,
        ),
        (
            "
                    /**
                     * @param {*} abc
                     */
                    function quux () {}
                  ",
            None,
            None,
        ),
        (
            "
                    /**
                     * @param {string|Promise<any>} abc
                     */
                    function quux () {}
                  ",
            None,
            None,
        ),
        (
            "
                    /**
                     * @param {Array<*>|number} abc
                     */
                    function quux () {}
                  ",
            None,
            None,
        ),
        (
            "
                    /**
                     * @param {{bar: string, [Symbol.dispose](): any}}
                     */
                    function quux () {}
                  ",
            None,
            None,
        ),
        (
            "
                    /**
                     * @param {{bar: string, [Symbol.dispose](): any}}
                     */
                    function quux () {}
                  ",
            None,
            Some(serde_json::json!({ "settings": { "jsdoc": { "mode": "permissive", }, } })),
        ),
        (
            "
                    /**
                     * @param {Record<string, any>} abc
                     * @param {any[]} def
                     * @param {...*} ghi
                     * @param {?any} jkl
                     */
                    function quux () {}
                  ",
            None,
            None,
        ),
        (
            "
                    /**
                     * @param {function(any): *} abc
                     * @param {(a: any) => void} def
                     * @param {T extends string ? any : never} ghi
                     * @param {`prefix-${any}`} jkl
                     */
                    function quux () {}
                  ",
            None,
            None,
        ),
        (
            "
                    /**
                     * @param {{
                     *   foo: any
                     * }} abc
                     */
                    function quux () {}
                  ",
            None,
            None,
        ),
        (
            "
                    /**
                     * @type {any}
                     */
                    let foo;
                  ",
            None,
            None,
        ),
        (
            "
                    /**
                     * @typedef {Object} Foo
                     * @property {any} bar
                     */
                  ",
            None,
            None,
        ),
        (
            "
                    /**
                     * @returns {Promise<any>}
                     */
                    function quux () {}
                  ",
            None,
            None,
        ),
        (
            "
                    /**
                     * @template {Record<string, any>} T
                     * @this {any}
                     */
                    function quux () {}
                  ",
            None,
            None,
        ),
    ];

    Tester::new(RejectAnyType::NAME, RejectAnyType::PLUGIN, pass, fail).test_and_snapshot();
}
