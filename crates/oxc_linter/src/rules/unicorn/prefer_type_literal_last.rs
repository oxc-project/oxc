// Adapted from eslint-plugin-unicorn's prefer-type-literal-last rule.
// https://github.com/sindresorhus/eslint-plugin-unicorn/blob/v78.0.0/rules/prefer-type-literal-last.js
// Copyright (c) Sindre Sorhus. MIT license; see THIRD-PARTY-LICENSE.

use oxc_ast::{
    AstKind,
    ast::{TSType, TSUnionType},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::{identifier::is_white_space, line_terminator::is_line_terminator};

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    rule::Rule,
};

fn prefer_type_literal_last_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Move type literals to the end of this union.").with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferTypeLiteralLast;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Requires inline object type literals to follow other members in TypeScript unions.
    /// Keeps `null` and `undefined` after the type literals when it fixes a union.
    /// A union or intersection member alone does not cause a report.
    /// Comments between members or next to the union prevent an automatic fix.
    ///
    /// ### Why is this bad?
    ///
    /// An object type can span many lines. Put compact type members first to make the
    /// union easier to read.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// type Result = { value: string } | string;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// type Result = string | { value: string };
    /// type OptionalResult = string | { value: string } | undefined;
    /// ```
    PreferTypeLiteralLast,
    unicorn,
    style,
    recommended,
    fix,
    version = "next",
    short_description = "Require type literals to be last in union types.",
);

impl Rule for PreferTypeLiteralLast {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::TSUnionType(union) = node.kind() else { return };
        let mut first_literal = None;
        for ty in &union.types {
            match ty.without_parenthesized() {
                TSType::TSTypeLiteral(literal) => {
                    first_literal.get_or_insert(literal.span);
                }
                TSType::TSNullKeyword(_)
                | TSType::TSUndefinedKeyword(_)
                | TSType::TSUnionType(_)
                | TSType::TSIntersectionType(_) => {}
                _ => {
                    let Some(span) = first_literal else { continue };
                    let diagnostic = prefer_type_literal_last_diagnostic(span);
                    if has_unsafe_comment(union, ctx) {
                        ctx.diagnostic(diagnostic);
                    } else {
                        ctx.diagnostic_with_fix(diagnostic, |fixer| {
                            let mut replacement = String::with_capacity(union.span.size() as usize);
                            // Keep source order within each group and retain parentheses.
                            for group in 0..3 {
                                for ty in &union.types {
                                    if type_group(ty) != group {
                                        continue;
                                    }
                                    if !replacement.is_empty() {
                                        replacement.push_str(" | ");
                                    }
                                    replacement.push_str(ctx.source_range(ty.span()));
                                }
                            }
                            fixer.replace(union.span, replacement)
                        });
                    }
                    return;
                }
            }
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_typescript()
    }
}

fn type_group(ty: &TSType<'_>) -> u8 {
    match ty.without_parenthesized() {
        TSType::TSTypeLiteral(_) => 1,
        TSType::TSNullKeyword(_) | TSType::TSUndefinedKeyword(_) => 2,
        _ => 0,
    }
}

fn has_unsafe_comment(union: &TSUnionType<'_>, ctx: &LintContext<'_>) -> bool {
    let comments = ctx.comments();
    let first = comments.partition_point(|comment| comment.span.end <= union.span.start);
    // A comment next to the union can describe the first or last member.
    if let Some(comment) = first.checked_sub(1).map(|index| &comments[index])
        && ctx
            .source_range(Span::new(comment.span.end, union.span.start))
            .chars()
            .all(|c| is_white_space(c) || is_line_terminator(c))
    {
        return true;
    }
    let mut types = union.types.iter().peekable();
    for comment in &comments[first..] {
        if comment.span.start >= union.span.end {
            return ctx
                .source_range(Span::new(union.span.end, comment.span.start))
                .chars()
                .all(|c| is_white_space(c) || is_line_terminator(c));
        }
        while types.peek().is_some_and(|ty| ty.span().end <= comment.span.start) {
            types.next();
        }
        if !types.peek().is_some_and(|ty| ty.span().contains_inclusive(comment.span)) {
            return true;
        }
    }
    false
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "type T = ({}) | (null) | ((undefined));",
        "type T = ({}) | ((A & B));",
        "type T = {} & string;",
        "const value = {} | 1;",
        "type ElementUnion = Other | {foo: string};",
        "type ElementUnion = A | B | {foo: string} | {bar: string};",
        "type ElementUnion = {foo: string} | {bar: string};",
        "type ElementUnion = Promise<{foo: string}> | Other;",
        "type ElementUnion = {foo: string} | (A | B);",
        "type ElementUnion = {foo: string} | (A & B);",
        "type ElementUnion = {foo: string} | ({bar: string} & {baz: boolean});",
        "type ElementUnion = {foo: string} | A & B;",
        "type ElementUnion = Other | {
                foo: string;
            };",
        "type ElementUnion = {foo: string} | undefined;",
        "type ElementUnion = {foo: string} | null;",
        "type ElementUnion = string | {foo: string} | undefined;",
        "type ElementUnion = {foo: string} | null | undefined;",
        "type ElementUnion = {foo: string} | {bar: string} | undefined;",
        "type ElementUnion = undefined | {foo: string};",
        "type ElementUnion = {
                foo: string;
            } | undefined;",
    ];

    let fail = vec![
        "type ElementUnion = {foo: string} | Other;",
        "type ElementUnion = A | {foo: string} | B;",
        "type ElementUnion = {foo: string} | A | {bar: string} | B;",
        "type ElementUnion = {foo: string} | string;",
        "type ElementUnion = {foo: string} | Other | undefined;",
        "type ElementUnion = {foo: string} | undefined | Other;",
        "type ElementUnion = undefined | {foo: string} | Other;",
        "type ElementUnion = ({foo: string}) | Other;",
        "type ElementUnion = {foo: string} | (() => string);",
        "type ElementUnion = {foo: string} | (new () => Element);",
        "type ElementUnion = {foo: string} | (A extends B ? C : D);",
        "type ElementUnion = {foo: string} | (A | B) | Other;",
        "type ElementUnion = {foo: string} | ({bar: string} | Other);",
        "type ElementUnion = {
                /* description */
                foo: string;
            } | Other;",
        "type ElementUnion = {foo: string} /* comment */ | Other;",
        "type ElementUnion = {foo: string} |
                // comment
                Other;",
        "type ElementUnion = /* comment */ {foo: string} | Other;",
        "type ElementUnion = {foo: string} | (/* comment */ Other);",
        "type ElementUnion = {foo: string} | Other /* comment */;",
        "type ElementUnion = {foo: string} | Other /* comment */",
        "/* before */ type ElementUnion = {foo: string} | Other; /* after */",
    ];

    let fix = vec![
        ("type T = | {} | A;", "type T = A | {};"),
        ("type T = (({})) | ((undefined)) | A;", "type T = A | (({})) | ((undefined));"),
        ("type T = (/* object */ {}) | A;", "type T = A | (/* object */ {});"),
        ("type T = {} | (A /* name */);", "type T = (A /* name */) | {};"),
        (
            "type T = null | {} | B | { x: string } | A | undefined;",
            "type T = B | A | {} | { x: string } | null | undefined;",
        ),
        ("let value: {} | string;", "let value: string | {};"),
        ("type T = [{} | string];", "type T = [string | {}];"),
        ("type T = {} | (A & B) | C;", "type T = (A & B) | C | {};"),
        ("type T = { inner: {} | string } | Other;", "type T = Other | { inner: {} | string };"),
        (
            "type ElementUnion = {foo: string} /* comment */ | Other;",
            "type ElementUnion = {foo: string} /* comment */ | Other;",
        ),
        (
            "type ElementUnion = {foo: string} |\n // comment\n Other;",
            "type ElementUnion = {foo: string} |\n // comment\n Other;",
        ),
        (
            "type ElementUnion = /* comment */ {foo: string} | Other;",
            "type ElementUnion = /* comment */ {foo: string} | Other;",
        ),
        (
            "type ElementUnion = {foo: string} | Other /* comment */;",
            "type ElementUnion = {foo: string} | Other /* comment */;",
        ),
        (
            "type ElementUnion = {foo: string} | Other /* comment */",
            "type ElementUnion = {foo: string} | Other /* comment */",
        ),
        ("type T = /* leading */\u{feff}{} | A;", "type T = /* leading */\u{feff}{} | A;"),
        ("type T = {} | A\u{feff}/* trailing */;", "type T = {} | A\u{feff}/* trailing */;"),
        ("type T = (/* leading */ {} | A);", "type T = (/* leading */ {} | A);"),
        ("type T = ({} | A /* trailing */);", "type T = ({} | A /* trailing */);"),
        ("type T = | /* leading */ {} | A;", "type T = | /* leading */ {} | A;"),
        (
            "type ElementUnion = {foo: string} | Other;",
            "type ElementUnion = Other | {foo: string};",
        ),
        (
            "type ElementUnion = A | {foo: string} | B;",
            "type ElementUnion = A | B | {foo: string};",
        ),
        (
            "type ElementUnion = {foo: string} | A | {bar: string} | B;",
            "type ElementUnion = A | B | {foo: string} | {bar: string};",
        ),
        (
            "type ElementUnion = {foo: string} | string;",
            "type ElementUnion = string | {foo: string};",
        ),
        (
            "type ElementUnion = {foo: string} | Other | undefined;",
            "type ElementUnion = Other | {foo: string} | undefined;",
        ),
        (
            "type ElementUnion = {foo: string} | undefined | Other;",
            "type ElementUnion = Other | {foo: string} | undefined;",
        ),
        (
            "type ElementUnion = undefined | {foo: string} | Other;",
            "type ElementUnion = Other | {foo: string} | undefined;",
        ),
        (
            "type ElementUnion = ({foo: string}) | Other;",
            "type ElementUnion = Other | ({foo: string});",
        ),
        (
            "type ElementUnion = {foo: string} | (() => string);",
            "type ElementUnion = (() => string) | {foo: string};",
        ),
        (
            "type ElementUnion = {foo: string} | (new () => Element);",
            "type ElementUnion = (new () => Element) | {foo: string};",
        ),
        (
            "type ElementUnion = {foo: string} | (A extends B ? C : D);",
            "type ElementUnion = (A extends B ? C : D) | {foo: string};",
        ),
        (
            "type ElementUnion = {foo: string} | (A | B) | Other;",
            "type ElementUnion = (A | B) | Other | {foo: string};",
        ),
        (
            "type ElementUnion = {foo: string} | ({bar: string} | Other);",
            "type ElementUnion = {foo: string} | (Other | {bar: string});",
        ),
        (
            "type ElementUnion = {
                /* description */
                foo: string;
            } | Other;",
            "type ElementUnion = Other | {
                /* description */
                foo: string;
            };",
        ),
        (
            "type ElementUnion = {foo: string} | (/* comment */ Other);",
            "type ElementUnion = (/* comment */ Other) | {foo: string};",
        ),
        (
            "/* before */ type ElementUnion = {foo: string} | Other; /* after */",
            "/* before */ type ElementUnion = Other | {foo: string}; /* after */",
        ),
    ];

    Tester::new(PreferTypeLiteralLast::NAME, PreferTypeLiteralLast::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
