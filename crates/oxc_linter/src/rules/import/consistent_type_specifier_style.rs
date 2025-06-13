use oxc_ast::{AstKind, ast::ImportDeclarationSpecifier};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use serde_json::Value;

use crate::{AstNode, context::LintContext, rule::Rule};

fn consistent_type_specifier_style_diagnostic(span: Span, mode: &Mode) -> OxcDiagnostic {
    let (warn_msg, help_msg) = if *mode == Mode::PreferInline {
        (
            "Prefer using inline type specifiers instead of a top-level type-only import.",
            "Replace top‐level import type with an inline type specifier.",
        )
    } else {
        (
            "Prefer using a top-level type-only import instead of inline type specifiers.",
            "Replace inline type specifiers with a top‐level import type statement.",
        )
    };
    OxcDiagnostic::warn(warn_msg).with_help(help_msg).with_label(span)
}

#[derive(Debug, Default, PartialEq, Clone)]
enum Mode {
    #[default]
    PreferTopLevel,
    PreferInline,
}

impl Mode {
    pub fn from(raw: &str) -> Self {
        if raw == "prefer-inline" { Self::PreferInline } else { Self::PreferTopLevel }
    }
}

#[derive(Debug, Default, Clone)]
pub struct ConsistentTypeSpecifierStyle {
    mode: Mode,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule either enforces or bans the use of inline type-only markers for named imports.
    ///
    /// ### Why is this bad?
    ///
    /// Mixing top-level `import type { Foo } from 'foo'` with inline `{ type Bar }`
    /// forces readers to mentally switch contexts when scanning your imports.
    /// Enforcing one style makes it immediately obvious which imports are types and which are value imports.
    ///
    /// ### Examples
    ///
    /// Examples of incorrect code for the default `prefer-top-level` option:
    /// ```typescript
    /// import {type Foo} from 'Foo';
    /// import Foo, {type Bar} from 'Foo';
    /// ```
    ///
    /// Examples of correct code for the default option:
    /// ```typescript
    /// import type {Foo} from 'Foo';
    /// import type Foo, {Bar} from 'Foo';
    /// ```
    ///
    /// Examples of incorrect code for the `prefer-inline` option:
    /// ```typescript
    /// import type {Foo} from 'Foo';
    /// import type Foo, {Bar} from 'Foo';
    /// ```
    ///
    /// Examples of correct code for the `prefer-inline` option:
    /// ```typescript
    /// import {type Foo} from 'Foo';
    /// import Foo, {type Bar} from 'Foo';
    /// ```
    ConsistentTypeSpecifierStyle,
    import,
    style,
    conditional_fix
);

impl Rule for ConsistentTypeSpecifierStyle {
    fn from_configuration(value: Value) -> Self {
        Self { mode: value.get(0).and_then(Value::as_str).map(Mode::from).unwrap_or_default() }
    }
    #[expect(clippy::cast_possible_truncation)]
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ImportDeclaration(import_decl) = node.kind() else {
            return;
        };
        let Some(specifiers) = &import_decl.specifiers else {
            return;
        };
        let len = specifiers.len();
        if len == 0
            || (len == 1
                && !matches!(specifiers[0], ImportDeclarationSpecifier::ImportSpecifier(_)))
        {
            return;
        }
        if self.mode == Mode::PreferTopLevel && import_decl.import_kind.is_value() {
            for item in specifiers {
                if matches!(item, ImportDeclarationSpecifier::ImportSpecifier(specifier) if specifier.import_kind.is_type())
                {
                    ctx.diagnostic(consistent_type_specifier_style_diagnostic(
                        item.span(),
                        &self.mode,
                    ));
                }
            }
        }
        if self.mode == Mode::PreferInline && import_decl.import_kind.is_type() {
            ctx.diagnostic_with_fix(
                consistent_type_specifier_style_diagnostic(import_decl.span, &self.mode),
                |fixer| {
                    let fixer = fixer.for_multifix();
                    let mut rule_fixes = fixer.new_fix_with_capacity(len);
                    for item in specifiers {
                        rule_fixes.push(fixer.insert_text_before(item, "type "));
                    }
                    // find the 'type' keyword and remove it
                    if let Some(type_token_span) = ctx
                        .source_range(Span::new(import_decl.span.start, specifiers[0].span().start))
                        .find("type")
                        .map(|pos| {
                            let start = import_decl.span.start + pos as u32;
                            Span::sized(start, 4)
                        })
                    {
                        let remove_fix = fixer.delete_range(type_token_span);
                        rule_fixes.push(remove_fix);
                    }
                    rule_fixes.with_message("Convert to an `inline` type import")
                },
            );
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;
    use serde_json::json;

    let pass = vec![
        ("import Foo from 'Foo'", None),
        ("import type Foo from 'Foo'", None),
        ("import { Foo } from 'Foo';", None),
        ("import { Foo as Bar } from 'Foo';", None),
        ("import * as Foo from 'Foo';", None),
        ("import 'Foo';", None),
        ("import {} from 'Foo';", None),
        ("import type {} from 'Foo';", None),
        ("import type { Foo as Bar } from 'Foo';", Some(json!(["prefer-top-level"]))),
        ("import type { Foo, Bar, Baz, Bam } from 'Foo';", Some(json!(["prefer-top-level"]))),
        ("import type {Foo} from 'Foo'", Some(json!(["prefer-top-level"]))),
        ("import {type Foo} from 'Foo'", Some(json!(["prefer-inline"]))),
        ("import Foo from 'Foo';", Some(json!(["prefer-inline"]))),
        ("import type Foo from 'Foo';", Some(json!(["prefer-inline"]))),
        ("import { Foo } from 'Foo';", Some(json!(["prefer-inline"]))),
        ("import { Foo as Bar } from 'Foo';", Some(json!(["prefer-inline"]))),
        ("import * as Foo from 'Foo';", Some(json!(["prefer-inline"]))),
        ("import 'Foo';", Some(json!(["prefer-inline"]))),
        ("import {} from 'Foo';", Some(json!(["prefer-inline"]))),
        ("import type {} from 'Foo';", Some(json!(["prefer-inline"]))),
        ("import { type Foo } from 'Foo';", Some(json!(["prefer-inline"]))),
        ("import { type Foo as Bar } from 'Foo';", Some(json!(["prefer-inline"]))),
        ("import { type Foo, type Bar, Baz, Bam } from 'Foo';", Some(json!(["prefer-inline"]))),
        ("import type * as Foo from 'Foo';", None),
    ];

    let fail = vec![
        ("import { type Foo, type Bar } from 'Foo'", None),
        ("import type { Foo } from 'Foo'", Some(json!(["prefer-inline"]))),
        ("import { type Foo as Bar } from 'Foo';", None),
        ("import { type Foo, type Bar } from 'Foo';", None),
        ("import { Foo, type Bar } from 'Foo';", None),
        ("import { type Foo, Bar } from 'Foo';", None),
        ("import Foo, { type Bar } from 'Foo';", None),
        ("import Foo, { type Bar, Baz } from 'Foo';", None),
        ("import { Component, type ComponentProps } from 'package-1';", None),
        ("import type { Foo, Bar, Baz } from 'Foo';", Some(json!(["prefer-inline"]))),
    ];

    let fix = vec![
        (
            "import type { foo, bar } from 'foo'",
            "import  { type foo, type bar } from 'foo'",
            Some(json!(["prefer-inline"])),
        ),
        (
            "import type{ foo } from 'foo'",
            "import { type foo } from 'foo'",
            Some(json!(["prefer-inline"])),
        ),
        (
            "import type /** comment */{ foo } from 'foo'",
            "import  /** comment */{ type foo } from 'foo'",
            Some(json!(["prefer-inline"])),
        ),
        (
            "import type { foo, /** comments */ bar } from 'foo'",
            "import  { type foo, /** comments */ type bar } from 'foo'",
            Some(json!(["prefer-inline"])),
        ),
        (
            r"
                import type {
                    bar,
                } from 'foo'
            ",
            r"
                import  {
                    type bar,
                } from 'foo'
            ",
            Some(json!(["prefer-inline"])),
        ),
    ];

    Tester::new(
        ConsistentTypeSpecifierStyle::NAME,
        ConsistentTypeSpecifierStyle::PLUGIN,
        pass,
        fail,
    )
    .expect_fix(fix)
    .test_and_snapshot();
}
