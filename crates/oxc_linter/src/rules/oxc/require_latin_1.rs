use std::fmt::Write as _;

use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Default, Clone)]
pub struct RequireLatin1;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Requires every character in the source file, including comments, to be
    /// within Latin-1 (U+0000 through U+00FF).
    ///
    /// String values can still contain any Unicode character: write characters
    /// outside Latin-1 using Unicode escapes, such as `"\u2014"` for an em dash.
    ///
    /// ### Why is this bad?
    ///
    /// V8 can store Latin-1 source text using one byte per character. A character
    /// outside Latin-1 can require two-byte storage for the source string. This
    /// can increase memory usage, especially when many isolates load a large bundle.
    /// Check the final bundle too, since a bundler or formatter may restore Unicode characters.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// const message = "compose Outputs — later";
    /// // the suffix shrinks instead — 12 chars
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const message = "compose Outputs \u2014 later";
    /// // the suffix shrinks instead -- 12 chars
    /// const city = "Montréal";
    /// ```
    ///
    /// ### Fixes
    ///
    /// Fixes escape characters in strings and untagged templates, and use numeric
    /// entities in JSX text and quoted attributes. Common punctuation in ordinary comments
    /// is replaced with ASCII; other comment characters become `U+XXXX` notation.
    ///
    /// Identifiers, regular expressions, tagged template text, and whitespace
    /// outside literals require manual edits. Escaping tagged templates or regular
    /// expressions can change observable raw text. Line separators in comments
    /// are also left unchanged because they can affect automatic semicolon insertion.
    /// JSDoc, legal comments, annotations, and comment directives require manual edits
    /// because they can contain meaningful types, paths, or configuration.
    ///
    /// Inspired by [Alchemy's Latin-1 rule](https://github.com/alchemy-run/alchemy/pull/2864).
    RequireLatin1,
    oxc,
    nursery,
    conditional_fix,
    version = "next",
    short_description = "Require Latin-1 source text to allow compact storage in JavaScript engines.",
);

#[derive(Clone, Copy, PartialEq, Eq)]
enum FixContext {
    None,
    Escape,
    Entity,
    Comment,
}

struct NonLatin1Character {
    span: Span,
    character: char,
    context: FixContext,
}

impl Rule for RequireLatin1 {
    // The parser limits source lengths to u32; a UTF-8 character uses at most four bytes.
    #[expect(clippy::cast_possible_truncation)]
    fn run_once(&self, ctx: &LintContext<'_>) {
        let source = ctx.source_text();
        if source.is_ascii() {
            return;
        }
        let mut characters: Vec<_> = source
            .char_indices()
            .filter(|&(_, character)| character > '\u{ff}')
            .map(|(offset, character)| NonLatin1Character {
                span: Span::sized(offset as u32, character.len_utf8() as u32),
                character,
                context: FixContext::None,
            })
            .collect();
        if characters.is_empty() {
            return;
        }

        // Only inspect the AST when the file needs diagnostics. Classify disjoint
        // literal/comment ranges using binary searches over the offending characters.
        for node in ctx.nodes() {
            let (span, context) = match node.kind() {
                AstKind::StringLiteral(literal) => (
                    literal.span,
                    if matches!(ctx.nodes().parent_kind(node.id()), AstKind::JSXAttribute(_)) {
                        FixContext::Entity
                    } else {
                        FixContext::Escape
                    },
                ),
                AstKind::TemplateLiteral(template) => {
                    if !matches!(
                        ctx.nodes().parent_kind(node.id()),
                        AstKind::TaggedTemplateExpression(_)
                    ) {
                        for quasi in &template.quasis {
                            set_context(&mut characters, quasi.span, FixContext::Escape);
                        }
                    }
                    continue;
                }
                AstKind::JSXText(text) => (text.span, FixContext::Entity),
                _ => continue,
            };
            set_context(&mut characters, span, context);
        }
        for comment in ctx.comments() {
            if !comment.is_normal() {
                continue;
            }
            let content = ctx.source_range(comment.content_span()).trim_start();
            // Preserve triple-slash references, source URLs, and configuration directives.
            if content.starts_with(['/', '#', '@'])
                || ["eslint", "oxlint", "global", "exported"]
                    .iter()
                    .any(|prefix| content.starts_with(prefix))
            {
                continue;
            }
            set_context(&mut characters, comment.span, FixContext::Comment);
        }

        // Adjacent edits conflict in the fixer, so replace each contiguous run together.
        for group in characters.chunk_by(|left, right| {
            left.context == right.context
                && (left.span.end == right.span.start
                    || (left.context == FixContext::Escape
                        && &source[left.span.end as usize..right.span.start as usize] == "\\"))
        }) {
            let first = &group[0];
            let span = Span::new(first.span.start, group.last().unwrap().span.end);
            let message = if group.len() == 1 {
                format!("U+{:04X} is outside Latin-1.", u32::from(first.character))
            } else {
                "Characters outside Latin-1.".to_string()
            };
            let diagnostic = OxcDiagnostic::warn(message)
                .with_help(
                    "Use an escape or a Latin-1 alternative to allow one-byte source storage.",
                )
                .with_label(span);

            ctx.diagnostic_with_fix(diagnostic, |fixer| {
                if first.context == FixContext::None {
                    return fixer.noop();
                }
                let mut span = span;
                let mut replacement = String::new();
                for entry in group {
                    let character = entry.character;
                    match entry.context {
                        FixContext::Escape => {
                            let backslashes = source.as_bytes()[..entry.span.start as usize]
                                .iter()
                                .rev()
                                .take_while(|&&byte| byte == b'\\')
                                .count();
                            if backslashes % 2 == 1 {
                                // Replace the identity escape too, so `"\—"` retains its value.
                                if entry.span.start == first.span.start {
                                    span.start -= 1;
                                }
                                if matches!(character, '\u{2028}' | '\u{2029}') {
                                    // Preserve a line continuation so a string cannot become
                                    // a strict-mode directive after fixing.
                                    replacement.push_str("\\\n");
                                    continue;
                                }
                            }
                            for unit in character.encode_utf16(&mut [0; 2]) {
                                write!(replacement, "\\u{unit:04x}").unwrap();
                            }
                        }
                        FixContext::Entity => {
                            write!(replacement, "&#x{:X};", u32::from(character)).unwrap();
                        }
                        FixContext::Comment => {
                            // Do not turn `•/` into a closing block-comment delimiter.
                            if let Some(ascii) = comment_replacement(character)
                                && !(ascii == "*"
                                    && source.as_bytes().get(entry.span.end as usize)
                                        == Some(&b'/'))
                            {
                                replacement.push_str(ascii);
                            } else {
                                write!(replacement, "U+{:04X}", u32::from(character)).unwrap();
                            }
                        }
                        FixContext::None => unreachable!(),
                    }
                }
                fixer.replace(span, replacement)
            });
        }
    }
}

fn set_context(characters: &mut [NonLatin1Character], span: Span, context: FixContext) {
    let start = characters.partition_point(|character| character.span.start < span.start);
    for character in &mut characters[start..] {
        if character.span.start >= span.end {
            break;
        }
        if context != FixContext::Comment || !matches!(character.character, '\u{2028}' | '\u{2029}')
        {
            character.context = context;
        }
    }
}

fn comment_replacement(character: char) -> Option<&'static str> {
    Some(match character {
        '‐' | '‑' | '‒' | '–' | '−' | '─' | '━' => "-",
        '—' | '―' => "--",
        '‘' | '’' | '‚' | '‛' | '′' => "'",
        '“' | '”' | '„' | '″' => "\"",
        '…' => "...",
        '•' => "*",
        '←' => "<-",
        '→' => "->",
        '↔' | '⇄' => "<->",
        '⇒' => "=>",
        '⇔' => "<=>",
        '≤' => "<=",
        '≥' => ">=",
        '≠' => "!=",
        '│' | '┃' => "|",
        '┌' | '┐' | '└' | '┘' | '├' | '┤' | '┬' | '┴' | '┼' => "+",
        _ => return None,
    })
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "const message = 'plain ASCII';",
        "const café = 'Montréal ÿ'; // © 2026",
        r"const message = '\u2014\u{1f600}';",
        r"const template = `\u2014${value}\u2026`;",
        r"const expression = /\u2014/u;",
        r"const \u03b1 = 1;",
        r#"const element = <div title="&#x2014;">&#x1F600;</div>;"#,
        "// oxlint-disable-next-line oxc/require-latin1\nconst value = '—';",
    ];
    let fail = vec![
        "const message = 'compose Outputs — later';",
        "// the suffix shrinks instead — 12 chars",
        "/* … → “hello” */",
        "const template = `—${value}…`;",
        "const emoji = '😀';",
        "const α = 1;",
        "class C { #α = 1; }",
        "const expression = /—/u;",
        "const raw = String.raw`—`;",
        "const raw = tag`—${'…'}→`;",
        "const element = <div title='—'>…</div>;",
        "const element = <组件 />;",
        "const value = 1;\u{2003}value;",
        "/* before\u{2028}after */ const value = 1;",
    ];
    let fix = vec![
        (r"const message = '—\…';", r"const message = '\u2014\u2026';"),
        (
            "function f() { 'use\\\u{2028} strict'; return this; }",
            "function f() { 'use\\\n strict'; return this; }",
        ),
        ("const message = '—';", r"const message = '\u2014';"),
        (r"const message = '\—';", r"const message = '\u2014';"),
        (r"const message = '\\—';", r"const message = '\\\u2014';"),
        (r"const message = '\\\—';", r"const message = '\\\u2014';"),
        ("const emoji = '😀';", r"const emoji = '\ud83d\ude00';"),
        ("const message = 'Ā—😀';", r"const message = '\u0100\u2014\ud83d\ude00';"),
        (r"const message = '\—😀';", r"const message = '\u2014\ud83d\ude00';"),
        ("const message = '\\\u{2028}—';", "const message = '\\\n\\u2014';"),
        ("const template = `—${'…'}→`;", r"const template = `\u2014${'\u2026'}\u2192`;"),
        ("const raw = tag`—${'…'}→`;", r"const raw = tag`—${'\u2026'}→`;"),
        ("const raw = tag`—${`…`}→`;", r"const raw = tag`—${`\u2026`}→`;"),
        ("// the suffix shrinks instead — 12 chars", "// the suffix shrinks instead -- 12 chars"),
        ("/* … → “hello” */", "/* ... -> \"hello\" */"),
        ("/* 中文 */", "/* U+4E2DU+6587 */"),
        ("/* ─── */", "/* --- */"),
        ("/* —\u{2028}→ */", "/* --\u{2028}-> */"),
        ("/** @type {'—'} */ const value = '—';", r"/** @type {'—'} */ const value = '\u2014';"),
        (
            "import(/* webpackChunkName: '—' */ './module.js');",
            "import(/* webpackChunkName: '—' */ './module.js');",
        ),
        ("/// <reference path='包.d.ts' />", "/// <reference path='包.d.ts' />"),
        ("//# sourceMappingURL=包.js.map", "//# sourceMappingURL=包.js.map"),
        ("// @custom —", "// @custom —"),
        ("/*! license — */", "/*! license — */"),
        ("/* eslint custom: ['error', '—'] */", "/* eslint custom: ['error', '—'] */"),
        ("/* *\u{200b}/ */", "/* *U+200B/ */"),
        ("/* •/ */", "/* U+2022/ */"),
        (
            "const element = <div title='—'>…😀</div>;",
            "const element = <div title='&#x2014;'>&#x2026;&#x1F600;</div>;",
        ),
        ("const element = <div title={'—'} />;", r"const element = <div title={'\u2014'} />;"),
        ("const message = '\u{2028}';", r"const message = '\u2028';"),
        ("const message = '\\\u{2028}';", "const message = '\\\n';"),
        ("const template = `\u{2029}`;", r"const template = `\u2029`;"),
        ("const template = `\\\u{2029}`;", "const template = `\\\n`;"),
        ("const α = /—/;", "const α = /—/;"),
        ("const raw = String.raw`—`;", "const raw = String.raw`—`;"),
        ("const value = 1;\u{2003}value;", "const value = 1;\u{2003}value;"),
        (
            "/* before\u{2028}after */ const value = 1;",
            "/* before\u{2028}after */ const value = 1;",
        ),
    ];

    Tester::new(RequireLatin1::NAME, RequireLatin1::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
