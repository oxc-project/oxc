use std::ops::Deref;

use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::{
    identifier::is_irregular_whitespace, line_terminator::is_irregular_line_terminator,
};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

fn no_irregular_whitespace_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected irregular whitespace")
        .with_help("Try to remove the irregular whitespace")
        .with_label(span)
}

#[derive(Debug, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct NoIrregularWhitespaceConfig {
    /// Whether to skip irregular whitespace in string literals.
    skip_strings: bool,
    /// Whether to skip irregular whitespace in comments.
    skip_comments: bool,
    /// Whether to skip irregular whitespace in regular expression literals.
    skip_reg_exps: bool,
    /// Whether to skip irregular whitespace in template literals.
    skip_templates: bool,
    /// Whether to skip irregular whitespace in JSX text.
    #[serde(rename = "skipJSXText")]
    skip_jsx_text: bool,
}

impl Default for NoIrregularWhitespaceConfig {
    fn default() -> Self {
        Self {
            skip_strings: true,
            skip_comments: true,
            skip_reg_exps: true,
            skip_templates: true,
            skip_jsx_text: true,
        }
    }
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct NoIrregularWhitespace(Box<NoIrregularWhitespaceConfig>);

impl Deref for NoIrregularWhitespace {
    type Target = NoIrregularWhitespaceConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows the use of irregular whitespace characters in the code.
    ///
    /// ### Why is this bad?
    ///
    /// Irregular whitespace characters are invisible to most editors and can
    /// cause unexpected behavior, making code harder to debug and maintain.
    /// They can also cause issues with code formatting and parsing.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// // Contains irregular whitespace characters (invisible)
    /// function example() {
    ///   var foo = 'bar'; // irregular whitespace before 'bar'
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// function example() {
    ///   var foo = 'bar'; // regular spaces only
    /// }
    /// ```
    NoIrregularWhitespace,
    eslint,
    correctness,
    config = NoIrregularWhitespaceConfig,
    version = "0.1.1",
);

/// Check if a character is irregular whitespace for linting purposes.
/// This includes irregular whitespace characters, irregular line terminators
/// (U+2028/U+2029), and U+180E (Mongolian Vowel Separator) to match ESLint's
/// behavior. U+180E is not in `is_irregular_whitespace` because it's Unicode
/// category "Cf" (Format), not "Zs" (Space Separator), so the parser must not
/// treat it as whitespace per the ECMAScript spec.
fn is_lint_irregular_whitespace(c: char) -> bool {
    is_irregular_whitespace(c) || is_irregular_line_terminator(c) || c == '\u{180e}'
}

/// Report irregular whitespace characters found within a span of source text.
fn report_irregular_whitespace_in_span(ctx: &LintContext, source_text: &str, span: Span) {
    let start = span.start as usize;
    let slice = &source_text[start..span.end as usize];
    for (i, c) in slice.char_indices() {
        if is_lint_irregular_whitespace(c) {
            #[expect(clippy::cast_possible_truncation)]
            let offset = span.start + i as u32;
            #[expect(clippy::cast_possible_truncation)]
            let len = c.len_utf8() as u32;
            ctx.diagnostic(no_irregular_whitespace_diagnostic(Span::new(offset, offset + len)));
        }
    }
}

impl Rule for NoIrregularWhitespace {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run_once(&self, ctx: &LintContext) {
        // Report code-context irregular whitespace detected by the lexer.
        // Skip BOM (U+FEFF) at position 0 — it's a valid byte order mark.
        for span in ctx.semantic().irregular_whitespaces() {
            if span.start == 0 && ctx.source_range(*span) == "\u{feff}" {
                continue;
            }
            ctx.diagnostic(no_irregular_whitespace_diagnostic(*span));
        }

        // Report irregular whitespace inside comments when not skipping them.
        if !self.skip_comments {
            let source_text = ctx.semantic().source_text();
            for comment in ctx.semantic().comments() {
                report_irregular_whitespace_in_span(ctx, source_text, comment.span);
            }
        }
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::StringLiteral(lit) if !self.skip_strings => {
                report_irregular_whitespace_in_span(ctx, ctx.semantic().source_text(), lit.span);
            }
            AstKind::RegExpLiteral(lit) if !self.skip_reg_exps => {
                report_irregular_whitespace_in_span(ctx, ctx.semantic().source_text(), lit.span);
            }
            AstKind::TemplateLiteral(lit) if !self.skip_templates => {
                let source_text = ctx.semantic().source_text();
                // Only check template element (quasis) spans, not the full template literal.
                // Expressions inside ${...} are code context, already handled by run_once.
                for element in &lit.quasis {
                    report_irregular_whitespace_in_span(ctx, source_text, element.span);
                }
            }
            AstKind::JSXText(text) if !self.skip_jsx_text => {
                report_irregular_whitespace_in_span(ctx, ctx.semantic().source_text(), text.span);
            }
            _ => {}
        }
    }
}

#[expect(clippy::unicode_not_nfc, clippy::invisible_characters)]
#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (r"'\u000B';", None),
        (r"'\u000C';", None),
        (r"'\u0085';", None),
        (r"'\u00A0';", None),
        (r"'\u180E';", None),
        (r"'\ufeff';", None),
        (r"'\u2000';", None),
        (r"'\u2001';", None),
        (r"'\u2002';", None),
        (r"'\u2003';", None),
        (r"'\u2004';", None),
        (r"'\u2005';", None),
        (r"'\u2006';", None),
        (r"'\u2007';", None),
        (r"'\u2008';", None),
        (r"'\u2009';", None),
        (r"'\u200A';", None),
        (r"'\u200B';", None),
        (r"'\u2028';", None),
        (r"'\u2029';", None),
        (r"'\u202F';", None),
        (r"'\u205f';", None),
        (r"'\u3000';", None),
        ("'';", None),
        ("'';", None),
        ("'';", None),
        ("' ';", None),
        ("'᠎';", None),
        ("'﻿';", None),
        ("' ';", None),
        ("' ';", None),
        ("' ';", None),
        ("' ';", None),
        ("' ';", None),
        ("' ';", None),
        ("' ';", None),
        ("' ';", None),
        ("' ';", None),
        ("' ';", None),
        ("' ';", None),
        ("'​';", None),
        (r"'\ ';", None),
        (r"'\ ';", None),
        ("' ';", None),
        ("' ';", None),
        ("'　';", None),
        ("// ", None),
        ("// ", None),
        ("// ", None),
        ("//  ", None),
        ("// ᠎", None),
        ("// ﻿", None),
        ("//  ", None),
        ("//  ", None),
        ("//  ", None),
        ("//  ", None),
        ("//  ", None),
        ("//  ", None),
        ("//  ", None),
        ("//  ", None),
        ("//  ", None),
        ("//  ", None),
        ("//  ", None),
        ("// ​", None),
        ("//  ", None),
        ("//  ", None),
        ("// 　", None),
        ("/*  */", None),
        ("/*  */", None),
        ("/*  */", None),
        ("/*   */", None),
        ("/* ᠎ */", None),
        ("/* ﻿ */", None),
        ("/*   */", None),
        ("/*   */", None),
        ("/*   */", None),
        ("/*   */", None),
        ("/*   */", None),
        ("/*   */", None),
        ("/*   */", None),
        ("/*   */", None),
        ("/*   */", None),
        ("/*   */", None),
        ("/*   */", None),
        ("/* ​ */", None),
        ("/*   */", None),
        ("/*   */", None),
        ("/*   */", None),
        ("/*   */", None),
        ("/* 　 */", None),
        ("//", None),
        ("//", None),
        ("//", None),
        ("/ /", None),
        ("/᠎/", None),
        ("/﻿/", None),
        ("/ /", None),
        ("/ /", None),
        ("/ /", None),
        ("/ /", None),
        ("/ /", None),
        ("/ /", None),
        ("/ /", None),
        ("/ /", None),
        ("/ /", None),
        ("/ /", None),
        ("/ /", None),
        ("/​/", None),
        ("/ /", None),
        ("/ /", None),
        ("/　/", None),
        ("``", None),                   // { "ecmaVersion": 6 },
        ("``", None),                   // { "ecmaVersion": 6 },
        ("``", None),                   // { "ecmaVersion": 6 },
        ("` `", None),                   // { "ecmaVersion": 6 },
        ("`᠎`", None),                    // { "ecmaVersion": 6 },
        ("`﻿`", None),                    // { "ecmaVersion": 6 },
        ("` `", None),                   // { "ecmaVersion": 6 },
        ("` `", None),                   // { "ecmaVersion": 6 },
        ("` `", None),                   // { "ecmaVersion": 6 },
        ("` `", None),                   // { "ecmaVersion": 6 },
        ("` `", None),                   // { "ecmaVersion": 6 },
        ("` `", None),                   // { "ecmaVersion": 6 },
        ("` `", None),                   // { "ecmaVersion": 6 },
        ("` `", None),                   // { "ecmaVersion": 6 },
        ("` `", None),                   // { "ecmaVersion": 6 },
        ("` `", None),                   // { "ecmaVersion": 6 },
        ("` `", None),                   // { "ecmaVersion": 6 },
        ("`​`", None),                    // { "ecmaVersion": 6 },
        ("` `", None),                   // { "ecmaVersion": 6 },
        ("` `", None),                   // { "ecmaVersion": 6 },
        ("`　`", None),                  // { "ecmaVersion": 6 },
        ("`　${foo}　`", None),          // { "ecmaVersion": 6 },
        ("const error = ` 　 `;", None), // { "ecmaVersion": 6 },
        (
            "const error = `
            　`;",
            None,
        ), // { "ecmaVersion": 6 },
        (
            "const error = `　
            `;",
            None,
        ), // { "ecmaVersion": 6 },
        (
            "const error = `
            　
            `;",
            None,
        ), // { "ecmaVersion": 6 },
        (
            "const error = `foo　bar
            foo　bar`;",
            None,
        ), // { "ecmaVersion": 6 },
        ("<div></div>;", None), // { "parserOptions": { "ecmaFeatures": { "jsx": true } } },
        ("<div></div>;", None), // { "parserOptions": { "ecmaFeatures": { "jsx": true } } },
        ("<div></div>;", None), // { "parserOptions": { "ecmaFeatures": { "jsx": true } } },
        ("<div> </div>;", None), // { "parserOptions": { "ecmaFeatures": { "jsx": true } } },
        ("<div>᠎</div>;", None),  // { "parserOptions": { "ecmaFeatures": { "jsx": true } } },
        ("<div>﻿</div>;", None),  // { "parserOptions": { "ecmaFeatures": { "jsx": true } } },
        ("<div> </div>;", None), // { "parserOptions": { "ecmaFeatures": { "jsx": true } } },
        ("<div> </div>;", None), // { "parserOptions": { "ecmaFeatures": { "jsx": true } } },
        ("<div> </div>;", None), // { "parserOptions": { "ecmaFeatures": { "jsx": true } } },
        ("<div> </div>;", None), // { "parserOptions": { "ecmaFeatures": { "jsx": true } } },
        ("<div> </div>;", None), // { "parserOptions": { "ecmaFeatures": { "jsx": true } } },
        ("<div> </div>;", None), // { "parserOptions": { "ecmaFeatures": { "jsx": true } } },
        ("<div> </div>;", None), // { "parserOptions": { "ecmaFeatures": { "jsx": true } } },
        ("<div> </div>;", None), // { "parserOptions": { "ecmaFeatures": { "jsx": true } } },
        ("<div> </div>;", None), // { "parserOptions": { "ecmaFeatures": { "jsx": true } } },
        ("<div> </div>;", None), // { "parserOptions": { "ecmaFeatures": { "jsx": true } } },
        ("<div> </div>;", None), // { "parserOptions": { "ecmaFeatures": { "jsx": true } } },
        ("<div>​</div>;", None),  // { "parserOptions": { "ecmaFeatures": { "jsx": true } } },
        ("<div> </div>;", None), // { "parserOptions": { "ecmaFeatures": { "jsx": true } } },
        ("<div> </div>;", None), // { "parserOptions": { "ecmaFeatures": { "jsx": true } } },
        ("<div>　</div>;", None), // { "parserOptions": { "ecmaFeatures": { "jsx": true } } },
        ("﻿console.log('hello BOM');", None),
        ("// ", None),
        ("// ", None),
        ("// ", None),
        ("//  ", None),
        ("// ᠎", None),
        ("// ﻿", None),
        ("//  ", None),
        ("//  ", None),
        ("//  ", None),
        ("//  ", None),
        ("//  ", None),
        ("//  ", None),
        ("//  ", None),
        ("//  ", None),
        ("//  ", None),
        ("//  ", None),
        ("//  ", None),
        ("// ​", None),
        ("//  ", None),
        ("//  ", None),
        ("// 　", None),
        ("/*  */", None),
        ("/*  */", None),
        ("/*  */", None),
        ("/*   */", None),
        ("/* ᠎ */", None),
        ("/* ﻿ */", None),
        ("/*   */", None),
        ("/*   */", None),
        ("/*   */", None),
        ("/*   */", None),
        ("/*   */", None),
        ("/*   */", None),
        ("/*   */", None),
        ("/*   */", None),
        ("/*   */", None),
        ("/*   */", None),
        ("/*   */", None),
        ("/* ​ */", None),
        ("/*   */", None),
        ("/*   */", None),
        ("/*   */", None),
        ("/*   */", None),
        ("/* 　 */", None),
        ("var any = /　/, other = //;", None),
        ("var any = `　`, other = ``;", None), // { "ecmaVersion": 6 },
        ("<div></div>;", None), // { "parserOptions": { "ecmaFeatures": { "jsx": true, }, }, },
        ("<div></div>;", None), // { "parserOptions": { "ecmaFeatures": { "jsx": true, }, }, },
        ("<div></div>;", None), // { "parserOptions": { "ecmaFeatures": { "jsx": true, }, }, },
        ("<div> </div>;", None), // { "parserOptions": { "ecmaFeatures": { "jsx": true, }, }, },
        ("<div>᠎</div>;", None),  // { "parserOptions": { "ecmaFeatures": { "jsx": true, }, }, },
        ("<div>﻿</div>;", None),  // { "parserOptions": { "ecmaFeatures": { "jsx": true, }, }, },
        ("<div> </div>;", None), // { "parserOptions": { "ecmaFeatures": { "jsx": true, }, }, },
        ("<div> </div>;", None), // { "parserOptions": { "ecmaFeatures": { "jsx": true, }, }, },
        ("<div> </div>;", None), // { "parserOptions": { "ecmaFeatures": { "jsx": true, }, }, },
        ("<div> </div>;", None), // { "parserOptions": { "ecmaFeatures": { "jsx": true, }, }, },
        ("<div> </div>;", None), // { "parserOptions": { "ecmaFeatures": { "jsx": true, }, }, },
        ("<div> </div>;", None), // { "parserOptions": { "ecmaFeatures": { "jsx": true, }, }, },
        ("<div> </div>;", None), // { "parserOptions": { "ecmaFeatures": { "jsx": true, }, }, },
        ("<div> </div>;", None), // { "parserOptions": { "ecmaFeatures": { "jsx": true, }, }, },
        ("<div> </div>;", None), // { "parserOptions": { "ecmaFeatures": { "jsx": true, }, }, },
        ("<div> </div>;", None), // { "parserOptions": { "ecmaFeatures": { "jsx": true, }, }, },
        ("<div> </div>;", None), // { "parserOptions": { "ecmaFeatures": { "jsx": true, }, }, },
        ("<div>​</div>;", None),  // { "parserOptions": { "ecmaFeatures": { "jsx": true, }, }, },
        ("<div> </div>;", None), // { "parserOptions": { "ecmaFeatures": { "jsx": true, }, }, },
        ("<div> </div>;", None), // { "parserOptions": { "ecmaFeatures": { "jsx": true, }, }, },
        ("<div>　</div>;", None), // { "parserOptions": { "ecmaFeatures": { "jsx": true, }, }, },
    ];

    let fail = vec![
        ("var any  = 'thing';", None),
        ("var any  = 'thing';", None),
        ("var any   = 'thing';", None),
        ("var any ﻿ = 'thing';", None),
        ("var any   = 'thing';", None),
        ("var any   = 'thing';", None),
        ("var any   = 'thing';", None),
        ("var any   = 'thing';", None),
        ("var any   = 'thing';", None),
        ("var any   = 'thing';", None),
        ("var any   = 'thing';", None),
        ("var any   = 'thing';", None),
        ("var any   = 'thing';", None),
        ("var any   = 'thing';", None),
        ("var any   = 'thing';", None),
        ("var any   = 'thing';", None),
        ("var any   = 'thing';", None),
        ("var any   = 'thing';", None),
        ("var any   = 'thing';", None),
        ("var any 　 = 'thing';", None),
        (
            "var a = 'b', c = 'd',
            e = 'f' ",
            None,
        ),
        (
            "var any 　 = 'thing', other 　 = 'thing';
            var third 　 = 'thing';",
            None,
        ),
        ("var any = '　', other = '';", Some(serde_json::json!([{ "skipStrings": false }]))),
        ("var any = `　`, other = ``;", Some(serde_json::json!([{ "skipTemplates": false }]))), // { "ecmaVersion": 6 },
        ("`something ${　 10} another thing`", None), // { "ecmaVersion": 6 },
        ("`something ${10　} another thing`", None),  // { "ecmaVersion": 6 },
        (
            "　
            `　template`",
            None,
        ), // { "ecmaVersion": 6 },
        (
            "　
            `　multiline
            template`",
            None,
        ), // { "ecmaVersion": 6 },
        ("　`　template`", None),                     // { "ecmaVersion": 6 },
        (
            "　`　multiline
            template`",
            None,
        ), // { "ecmaVersion": 6 },
        ("`　template`　", None),                     // { "ecmaVersion": 6 },
        (
            "`　multiline
            template`　",
            None,
        ), // { "ecmaVersion": 6 },
        (
            "`　template`
            　",
            None,
        ), // { "ecmaVersion": 6 },
        (
            "`　multiline
            template`
            　",
            None,
        ), // { "ecmaVersion": 6 },
        ("var foo =  bar;", None),
        ("var foo =bar;", None),
        ("var foo =  bar;", None),
        ("var foo =  bar;", None),
        ("var foo =   bar;", None),
        ("var foo = bar;", None),
        ("", None),
        ("   ", None),
        (
            "var foo = 
            bar;",
            None,
        ),
        (
            "var foo =
            bar;",
            None,
        ),
        (
            "var foo = 
            bar
            ;
            ",
            None,
        ),
        ("var foo =  bar;", None),
        ("var foo =  bar;", None),
        ("var foo = bar; ", None),
        (" ", None),
        ("foo  ", None),
        ("foo  ", None),
        (
            "foo 
             ",
            None,
        ),
        ("foo ", None),
        ("// 　", Some(serde_json::json!([{ "skipComments": false }]))),
        ("/* 　 */", Some(serde_json::json!([{ "skipComments": false }]))),
        ("var any = /　/, other = /​/;", Some(serde_json::json!([{ "skipRegExps": false }]))),
        ("var any = `　`, other = `​`;", Some(serde_json::json!([{ "skipTemplates": false }]))),
        ("<div>　</div>;", Some(serde_json::json!([{ "skipJSXText": false }]))),
    ];

    Tester::new(NoIrregularWhitespace::NAME, NoIrregularWhitespace::PLUGIN, pass, fail)
        .test_and_snapshot();
}
