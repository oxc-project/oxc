use oxc_allocator::{Allocator, CloneIn};
use oxc_ast::{
    AstBuilder, AstKind,
    ast::{ImportDeclaration, ImportDeclarationSpecifier, ImportOrExportKind},
};
use oxc_codegen::{Context, Gen};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, SPAN, Span};
use serde_json::Value;

use crate::{AstNode, context::LintContext, fixer::RuleFixer, rule::Rule};

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
            let (value_specifiers, type_specifiers) = split_import_specifiers_by_kind(specifiers);
            if type_specifiers.is_empty() {
                return;
            }

            for item in &type_specifiers {
                ctx.diagnostic_with_fix(
                    consistent_type_specifier_style_diagnostic(item.span(), &self.mode),
                    |fixer| {
                        let mut import_source = String::new();

                        if !value_specifiers.is_empty() {
                            let value_import_declaration =
                                gen_value_import_declaration(fixer, import_decl, &value_specifiers);
                            import_source.push_str(&value_import_declaration);
                        }

                        let type_import_declaration =
                            gen_type_import_declaration(fixer, import_decl, &type_specifiers);
                        import_source.push_str(&type_import_declaration);

                        fixer
                            .replace(import_decl.span, import_source.trim_end().to_string())
                            .with_message("Convert to a `top-level` type import")
                    },
                );
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

fn split_import_specifiers_by_kind<'a, I>(specifiers: I) -> (Vec<I::Item>, Vec<I::Item>)
where
    I: IntoIterator<Item = &'a ImportDeclarationSpecifier<'a>>,
{
    let (value_kind_specifiers, type_kind_specifiers) = specifiers.into_iter().fold(
        (vec![], vec![]),
        |(mut value_kind_specifiers, mut type_kind_specifiers), it| {
            match it {
                ImportDeclarationSpecifier::ImportDefaultSpecifier(_) => {
                    value_kind_specifiers.push(it);
                }
                ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                    if specifier.import_kind.is_value() {
                        value_kind_specifiers.push(it);
                    } else {
                        type_kind_specifiers.push(it);
                    }
                }
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(_) => {}
            }
            (value_kind_specifiers, type_kind_specifiers)
        },
    );
    (value_kind_specifiers, type_kind_specifiers)
}

fn gen_value_import_declaration<'c, 'a: 'c>(
    fixer: RuleFixer<'c, 'a>,
    import_decl: &'a ImportDeclaration<'a>,
    specifiers: &Vec<&'a ImportDeclarationSpecifier<'a>>,
) -> String {
    let mut codegen = fixer.codegen();

    let alloc = Allocator::default();
    let ast_builder = AstBuilder::new(&alloc);

    let specifiers: Vec<_> = specifiers.iter().map(|it| it.clone_in(&alloc)).collect();
    let import_declaration = ast_builder.alloc_import_declaration(
        SPAN,
        Some(oxc_allocator::Vec::from_iter_in(specifiers, &alloc)),
        import_decl.source.clone_in(&alloc),
        None,
        import_decl.with_clause.clone_in(&alloc),
        ImportOrExportKind::Value,
    );

    import_declaration.print(&mut codegen, Context::empty());
    codegen.into_source_text()
}

fn gen_type_import_declaration<'c, 'a: 'c>(
    fixer: RuleFixer<'c, 'a>,
    import_decl: &'a ImportDeclaration<'a>,
    specifiers: &Vec<&'a ImportDeclarationSpecifier<'a>>,
) -> String {
    let mut codegen = fixer.codegen();

    let alloc = Allocator::default();
    let ast_builder = AstBuilder::new(&alloc);

    let specifiers: Vec<_> = specifiers
        .iter()
        .filter_map(|it| match it {
            ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                Some(ast_builder.import_declaration_specifier_import_specifier(
                    SPAN,
                    specifier.imported.clone_in(&alloc),
                    specifier.local.clone_in(&alloc),
                    ImportOrExportKind::Value,
                ))
            }
            _ => None,
        })
        .collect();
    let import_declaration = ast_builder.alloc_import_declaration(
        SPAN,
        Some(oxc_allocator::Vec::from_iter_in(specifiers, &alloc)),
        import_decl.source.clone_in(&alloc),
        None,
        import_decl.with_clause.clone_in(&alloc),
        ImportOrExportKind::Type,
    );

    import_declaration.print(&mut codegen, Context::empty());
    codegen.into_source_text()
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
        (
            "import { type Foo } from 'Foo';",
            "import type { Foo } from 'Foo';",
            Some(json!(["prefer-top-level"])),
        ),
        (
            "import { type Foo as Bar } from 'Foo';",
            "import type { Foo as Bar } from 'Foo';",
            Some(json!(["prefer-top-level"])),
        ),
        (
            "import { type Foo, type Bar } from 'Foo';",
            "import type { Foo, Bar } from 'Foo';",
            Some(json!(["prefer-top-level"])),
        ),
        (
            "import { type Foo, type Bar } from 'Foo';",
            "import type { Foo, Bar } from 'Foo';",
            Some(json!(["prefer-top-level"])),
        ),
        (
            "import { Foo, type Bar } from 'Foo';",
            "import { Foo } from 'Foo';\nimport type { Bar } from 'Foo';",
            Some(json!(["prefer-top-level"])),
        ),
        (
            "import { type Foo, Bar } from 'Foo';",
            "import { Bar } from 'Foo';\nimport type { Foo } from 'Foo';",
            Some(json!(["prefer-top-level"])),
        ),
        (
            "import Foo, { type Bar } from 'Foo';",
            "import Foo from 'Foo';\nimport type { Bar } from 'Foo';",
            Some(json!(["prefer-top-level"])),
        ),
        (
            "import Foo, { type Bar, Baz } from 'Foo';",
            "import Foo, { Baz } from 'Foo';\nimport type { Bar } from 'Foo';",
            Some(json!(["prefer-top-level"])),
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
