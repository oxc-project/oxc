use std::ops::Deref;

use oxc_ast::{ast::ImportDeclarationSpecifier, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::SymbolId;
use oxc_span::{CompactStr, Span};

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_import_type_annotations_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "typescript-eslint(consistent-type-imports): `import()` type annotations are forbidden.",
    )
    .with_label(span)
}

fn avoid_import_type_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "typescript-eslint(consistent-type-imports): Use an `import` instead of an `import type`.",
    )
    .with_label(span)
}
fn type_over_value_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn( "typescript-eslint(consistent-type-imports): All imports in the declaration are only used as types. Use `import type`."
    )
    .with_label(span)
}

fn some_imports_are_only_types_diagnostic(span0: Span, x1: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "typescript-eslint(consistent-type-imports): Imports {x1} are only used as type."
    ))
    .with_label(span0)
}

#[derive(Debug, Default, Clone)]
pub struct ConsistentTypeImports(Box<ConsistentTypeImportsConfig>);

impl Deref for ConsistentTypeImports {
    type Target = ConsistentTypeImportsConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// <https://github.com/typescript-eslint/typescript-eslint/blob/main/packages/eslint-plugin/docs/rules/consistent-type-imports.mdx>
#[derive(Default, Debug, Clone)]
pub struct ConsistentTypeImportsConfig {
    disallow_type_annotations: DisallowTypeAnnotations,
    // TODO: Remove
    #[allow(unused)]
    fix_style: FixStyle,
    prefer: Prefer,
}

// The default of `disallowTypeAnnotations` is `true`.
#[derive(Debug, Clone)]
struct DisallowTypeAnnotations(bool);

impl DisallowTypeAnnotations {
    fn new(value: bool) -> Self {
        Self(value)
    }
}

impl Default for DisallowTypeAnnotations {
    fn default() -> Self {
        Self(true)
    }
}

#[derive(Default, Debug, Clone)]
enum FixStyle {
    #[default]
    SeparateTypeImports,
    InlineTypeImports,
}

#[derive(Default, Debug, Clone)]
enum Prefer {
    #[default]
    TypeImports,
    NoTypeImports,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce consistent usage of type imports.
    ///
    /// ### Why is this bad?
    ///
    /// inconsistent usage of type imports can make the code harder to read and understand.
    ///
    /// ### Example
    /// ```javascript
    /// import { Foo } from 'Foo';
    /// type T = Foo;
    ///
    /// type S = import("Foo");
    /// ```
    ConsistentTypeImports,
    nursery,
);

impl Rule for ConsistentTypeImports {
    fn from_configuration(value: serde_json::Value) -> Self {
        let config = value.get(0).and_then(serde_json::Value::as_object).map_or(
            ConsistentTypeImportsConfig::default(),
            |config| {
                let disallow_type_annotations = config
                    .get("disallowTypeAnnotations")
                    .and_then(serde_json::Value::as_bool)
                    .map(DisallowTypeAnnotations::new)
                    .unwrap_or_default();
                let fix_style = config.get("fixStyle").and_then(serde_json::Value::as_str).map_or(
                    FixStyle::SeparateTypeImports,
                    |fix_style| match fix_style {
                        "inline-type-imports" => FixStyle::InlineTypeImports,
                        _ => FixStyle::SeparateTypeImports,
                    },
                );
                let prefer = config.get("prefer").and_then(serde_json::Value::as_str).map_or(
                    Prefer::TypeImports,
                    |prefer| match prefer {
                        "no-type-imports" => Prefer::NoTypeImports,
                        _ => Prefer::TypeImports,
                    },
                );

                ConsistentTypeImportsConfig { disallow_type_annotations, fix_style, prefer }
            },
        );
        Self(Box::new(config))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if self.disallow_type_annotations.0 {
            //  `import()` type annotations are forbidden.
            // `type Foo = import('foo')`
            if let AstKind::TSImportType(import_type) = node.kind() {
                ctx.diagnostic(no_import_type_annotations_diagnostic(import_type.span));
                return;
            }
        }

        if matches!(self.prefer, Prefer::NoTypeImports) {
            match node.kind() {
                // `import type { Foo } from 'foo'`
                AstKind::ImportDeclaration(import_decl) => {
                    if import_decl.import_kind.is_type() {
                        ctx.diagnostic(avoid_import_type_diagnostic(import_decl.span));
                    }
                }
                // import { type Foo } from 'foo'
                AstKind::ImportSpecifier(import_specifier) => {
                    if import_specifier.import_kind.is_type() {
                        ctx.diagnostic(avoid_import_type_diagnostic(import_specifier.span));
                    }
                }
                _ => {}
            }
            return;
        }

        let AstKind::ImportDeclaration(import_decl) = node.kind() else {
            return;
        };

        // Store references that only used as type and without type qualifier.
        // For example:
        // ```typescript
        // import { A, B, type C } from 'foo';
        // const a: A;
        // const b: B;
        // const c: C;
        // ```
        // `A` and `B` are only used as type references.
        let mut type_references_without_type_qualifier = vec![];
        // If all specifiers are only used as type references.
        let mut is_only_type_references = false;

        if let Some(specifiers) = &import_decl.specifiers {
            for specifier in specifiers {
                let Some(symbol_id) = specifier.local().symbol_id.get() else {
                    continue;
                };
                let no_type_qualifier = match specifier {
                    ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                        specifier.import_kind.is_value()
                    }
                    ImportDeclarationSpecifier::ImportDefaultSpecifier(_)
                    | ImportDeclarationSpecifier::ImportNamespaceSpecifier(_) => true,
                };

                if no_type_qualifier && is_only_has_type_references(symbol_id, ctx) {
                    type_references_without_type_qualifier.push(specifier);
                }
            }

            is_only_type_references =
                type_references_without_type_qualifier.len() == specifiers.len();
        }

        if import_decl.import_kind.is_value() && !type_references_without_type_qualifier.is_empty()
        {
            // `import type {} from 'foo' assert { type: 'json' }` is invalid
            // Import assertions cannot be used with type-only imports or exports.
            if is_only_type_references && import_decl.with_clause.is_none() {
                ctx.diagnostic(type_over_value_diagnostic(import_decl.span));
                return;
            }

            let names = type_references_without_type_qualifier
                .iter()
                .map(|specifier| specifier.name())
                .collect::<Vec<_>>();

            // ['foo', 'bar', 'baz' ] => "foo, bar, and baz".
            let type_imports = format_word_list(&names);

            ctx.diagnostic(some_imports_are_only_types_diagnostic(import_decl.span, &type_imports));
        }
    }
}

// Given an array of words, returns an English-friendly concatenation, separated with commas, with
// the `and` clause inserted before the last item.
//
// Example: ['foo', 'bar', 'baz' ] returns the string "foo, bar, and baz".
fn format_word_list(words: &[CompactStr]) -> String {
    match words.len() {
        0 => String::new(),
        1 => words[0].to_string(),
        2 => format!("{} and {}", words[0], words[1]),
        _ => {
            let mut result = String::new();
            for (i, word) in words.iter().enumerate() {
                if i == words.len() - 1 {
                    result.push_str(&format!("and {word}"));
                } else {
                    result.push_str(&format!("{word}, "));
                }
            }
            result
        }
    }
}

// Returns `true` if the symbol is only used as a type reference, and `false` otherwise.
// Specifically, return `false` if the symbol does not have any references.
fn is_only_has_type_references(symbol_id: SymbolId, ctx: &LintContext) -> bool {
    let mut peekable_iter = ctx.semantic().symbol_references(symbol_id).peekable();

    if peekable_iter.peek().is_none() {
        return false;
    }
    peekable_iter.all(oxc_semantic::Reference::is_type)
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            "
              import Foo from 'foo';
              const foo: Foo = new Foo();
            ",
            None,
        ),
        (
            "
              import foo from 'foo';
              const foo: foo.Foo = foo.fn();
            ",
            None,
        ),
        (
            "
              import { A, B } from 'foo';
              const foo: A = B();
              const bar = new A();
            ",
            None,
        ),
        (
            "
              import Foo from 'foo';
                  ",
            None,
        ),
        // TODO: Need fix: https://github.com/oxc-project/oxc/issues/3799
        // (
        //     "
        //       import Foo from 'foo';
        //       type T<Foo> = Foo; // shadowing
        //     ",
        //     None,
        // ),
        // (
        //     "
        //       import Foo from 'foo';
        //       function fn() {
        //         type Foo = {}; // shadowing
        //         let foo: Foo;
        //       }
        //     ",
        //     None,
        // ),
        (
            "
              import { A, B } from 'foo';
              const b = B;
            ",
            None,
        ),
        (
            "
              import { A, B, C as c } from 'foo';
              const d = c;
            ",
            None,
        ),
        (
            "
              import {} from 'foo'; // empty
            ",
            None,
        ),
        (
            "
              let foo: import('foo');
              let bar: import('foo').Bar;
            ",
            Some(serde_json::json!([{ "disallowTypeAnnotations": false }])),
        ),
        (
            "
              import Foo from 'foo';
              let foo: Foo;
            ",
            Some(serde_json::json!([{ "prefer": "no-type-imports" }])),
        ),
        // (
        //     "
        //       import type Type from 'foo';

        //       type T = typeof Type;
        //       type T = typeof Type.foo;
        //     ",
        //     None,
        // ),
        // (
        //     "
        //       import type { Type } from 'foo';

        //       type T = typeof Type;
        //       type T = typeof Type.foo;
        //     ",
        //     None,
        // ),
        // (
        //     "
        //       import type * as Type from 'foo';

        //       type T = typeof Type;
        //       type T = typeof Type.foo;
        //     ",
        //     None,
        // ),
        // (
        //     "
        //       import Type from 'foo';

        //       type T = typeof Type;
        //       type T = typeof Type.foo;
        //     ",
        //     Some(serde_json::json!([{ "prefer": "no-type-imports" }])),
        // ),
        // (
        //     "
        //       import { Type } from 'foo';

        //       type T = typeof Type;
        //       type T = typeof Type.foo;
        //     ",
        //     Some(serde_json::json!([{ "prefer": "no-type-imports" }])),
        // ),
        // (
        //     "
        //       import * as Type from 'foo';

        //       type T = typeof Type;
        //       type T = typeof Type.foo;
        //     ",
        //     Some(serde_json::json!([{ "prefer": "no-type-imports" }])),
        // ),
        (
            "
              import * as Type from 'foo' assert { type: 'json' };
              const a: typeof Type = Type;
            ",
            Some(serde_json::json!([{ "prefer": "no-type-imports" }])),
        ),
        (
            "
              import { type A } from 'foo';
              type T = A;
            ",
            None,
        ),
        (
            "
              import { type A, B } from 'foo';
              type T = A;
              const b = B;
            ",
            None,
        ),
        (
            "
              import { type A, type B } from 'foo';
              type T = A;
              type Z = B;
            ",
            None,
        ),
        (
            "
              import { B } from 'foo';
              import { type A } from 'foo';
              type T = A;
              const b = B;
            ",
            None,
        ),
        (
            "
              import { B, type A } from 'foo';
              type T = A;
              const b = B;
            ",
            Some(serde_json::json!([{ "fixStyle": "inline-type-imports" }])),
        ),
        (
            "
              import { B } from 'foo';
              import type A from 'baz';
              type T = A;
              const b = B;
            ",
            Some(serde_json::json!([{ "fixStyle": "inline-type-imports" }])),
        ),
        (
            "
              import { type B } from 'foo';
              import type { A } from 'foo';
              type T = A;
              const b = B;
            ",
            Some(serde_json::json!([{ "fixStyle": "inline-type-imports" }])),
        ),
        (
            "
              import { B, type C } from 'foo';
              import type A from 'baz';
              type T = A;
              type Z = C;
              const b = B;
            ",
            Some(
                serde_json::json!([            { "prefer": "type-imports", "fixStyle": "inline-type-imports" },          ]),
            ),
        ),
        (
            "
              import { B } from 'foo';
              import type { A } from 'foo';
              type T = A;
              const b = B;
            ",
            Some(
                serde_json::json!([            { "prefer": "type-imports", "fixStyle": "inline-type-imports" },          ]),
            ),
        ),
        (
            "
              import { B } from 'foo';
              import { A } from 'foo';
              type T = A;
              const b = B;
            ",
            Some(
                serde_json::json!([            { "prefer": "no-type-imports", "fixStyle": "inline-type-imports" },          ]),
            ),
        ),
        (
            "
              import Type from 'foo';
              
              export { Type }; // is a value export
              export default Type; // is a value export
            ",
            None,
        ),
        (
            "
              import type Type from 'foo';
        
              export { Type }; // is a type-only export
              export default Type; // is a type-only export
              export type { Type }; // is a type-only export
            ",
            None,
        ),
        (
            "
              import { Type } from 'foo';
        
              export { Type }; // is a value export
              export default Type; // is a value export
            ",
            None,
        ),
        (
            "
              import type { Type } from 'foo';
        
              export { Type }; // is a type-only export
              export default Type; // is a type-only export
              export type { Type }; // is a type-only export
            ",
            None,
        ),
        (
            "
              import * as Type from 'foo';
        
              export { Type }; // is a value export
              export default Type; // is a value export
            ",
            None,
        ),
        (
            "
              import type * as Type from 'foo';
        
              export { Type }; // is a type-only export
              export default Type; // is a type-only export
              export type { Type }; // is a type-only export
            ",
            None,
        ),
        (
            "
              import Type from 'foo';
        
              export { Type }; // is a type-only export
              export default Type; // is a type-only export
              export type { Type }; // is a type-only export
            ",
            Some(serde_json::json!([{ "prefer": "no-type-imports" }])),
        ),
        (
            "
              import { Type } from 'foo';
        
              export { Type }; // is a type-only export
              export default Type; // is a type-only export
              export type { Type }; // is a type-only export
            ",
            Some(serde_json::json!([{ "prefer": "no-type-imports" }])),
        ),
        (
            "
              import * as Type from 'foo';
        
              export { Type }; // is a type-only export
              export default Type; // is a type-only export
              export type { Type }; // is a type-only export
            ",
            Some(serde_json::json!([{ "prefer": "no-type-imports" }])),
        ),
        // TODO: https://github.com/typescript-eslint/typescript-eslint/issues/2455#issuecomment-685015542
        // import React has side effect.
        // (
        //     "
        //       import React from 'react';

        //       export const ComponentFoo: React.FC = () => {
        //         return <div>Foo Foo</div>;
        //       };
        //     ",
        //     None,
        // ),
        // (
        //     "
        //       import { h } from 'some-other-jsx-lib';

        //       export const ComponentFoo: h.FC = () => {
        //         return <div>Foo Foo</div>;
        //       };
        //     ",
        //     None,
        // ),
        // (
        //     "
        //       import { Fragment } from 'react';

        //       export const ComponentFoo: Fragment = () => {
        //         return <>Foo Foo</>;
        //       };
        //     ",
        //     None,
        // ),
        (
            "
              import Default, * as Rest from 'module';
              const a: typeof Default = Default;
              const b: typeof Rest = Rest;
            ",
            None,
        ),
        (
            "
              import type * as constants from './constants';
        
              export type Y = {
                [constants.X]: ReadonlyArray<string>;
              };
            ",
            None,
        ),
        (
            "
              import A from 'foo';
              export = A;
            ",
            None,
        ),
        (
            "
              import type A from 'foo';
              export = A;
            ",
            None,
        ),
        (
            "
              import type A from 'foo';
              export = {} as A;
            ",
            None,
        ),
        (
            "
              import { type A } from 'foo';
              export = {} as A;
            ",
            None,
        ),
        (
            "
              import type T from 'mod';
              const x = T;
            ",
            None,
        ),
        (
            "
              import type { T } from 'mod';
              const x = T;
            ",
            None,
        ),
        (
            "
              import { type T } from 'mod';
              const x = T;
            ",
            None,
        ),
        // TODO: To support decorator in this rule, need <https://github.com/oxc-project/oxc/pull/3645>
        // experimentalDecorators: true + emitDecoratorMetadata: true
        // (
        //     "
        //     import Foo from 'foo';
        //     @deco
        //     class A {
        //       constructor(foo: Foo) {}
        //     }
        //   ",
        //     None,
        // ),
        // (
        //     "
        //     import Foo from 'foo';
        //     class A {
        //       @deco
        //       foo: Foo;
        //     }
        //   ",
        //     None,
        // ),
        // (
        //     "
        //     import Foo from 'foo';
        //     class A {
        //       @deco
        //       foo(foo: Foo) {}
        //     }
        //   ",
        //     None,
        // ),
        // (
        //     "
        //     import Foo from 'foo';
        //     class A {
        //       @deco
        //       foo(): Foo {}
        //     }
        //   ",
        //     None,
        // ),
        // (
        //     "
        //     import Foo from 'foo';
        //     class A {
        //       foo(@deco foo: Foo) {}
        //     }
        //   ",
        //     None,
        // ),
        // (
        //     "
        //     import Foo from 'foo';
        //     class A {
        //       @deco
        //       set foo(value: Foo) {}
        //     }
        //   ",
        //     None,
        // ),
        // (
        //     "
        //     import Foo from 'foo';
        //     class A {
        //       @deco
        //       get foo() {}

        //       set foo(value: Foo) {}
        //     }
        //   ",
        //     None,
        // ),
        // (
        //     "
        //     import Foo from 'foo';
        //     class A {
        //       @deco
        //       get foo() {}

        //       set ['foo'](value: Foo) {}
        //     }
        //   ",
        //     None,
        // ),
        // (
        //     "
        //     import type { Foo } from 'foo';
        //     const key = 'k';
        //     class A {
        //       @deco
        //       get [key]() {}

        //       set [key](value: Foo) {}
        //     }
        //   ",
        //     None,
        // ),
        // (
        //     "
        //     import * as foo from 'foo';
        //     @deco
        //     class A {
        //       constructor(foo: foo.Foo) {}
        //     }
        //   ",
        //     None,
        // ),
        // (
        //     "
        //     import type { ClassA } from './classA';

        //     export class ClassB {
        //       public constructor(node: ClassA) {}
        //     }
        //   ",
        //     None,
        // ),
        // (
        //     "
        //     import type Foo from 'foo';
        //     @deco
        //     class A {
        //       constructor(foo: Foo) {}
        //     }
        //   ",
        //     None,
        // ),
        // (
        //     "
        //     import type { Foo } from 'foo';
        //     @deco
        //     class A {
        //       constructor(foo: Foo) {}
        //     }
        //   ",
        //     None,
        // ),
        // (
        //     "
        //     import type { Type } from 'foo';
        //     import { Foo, Bar } from 'foo';
        //     @deco
        //     class A {
        //       constructor(foo: Foo) {}
        //     }
        //     type T = Bar;
        //   ",
        //     None,
        // ),
        // (
        //     "
        //     import { V } from 'foo';
        //     import type { Foo, Bar, T } from 'foo';
        //     @deco
        //     class A {
        //       constructor(foo: Foo) {}
        //       foo(@deco bar: Bar) {}
        //     }
        //   ",
        //     None,
        // ),
        // (
        //     "
        //     import type { Foo, T } from 'foo';
        //     import { V } from 'foo';
        //     @deco
        //     class A {
        //       constructor(foo: Foo) {}
        //     }
        //   ",
        //     None,
        // ),
        // (
        //     "
        //     import type * as Type from 'foo';
        //     @deco
        //     class A {
        //       constructor(foo: Type.Foo) {}
        //     }
        //   ",
        //     None,
        // ),
    ];

    let fail = vec![
        (
            "
              import Foo from 'foo';
              let foo: Foo;
              type Bar = Foo;
              interface Baz {
                foo: Foo;
              }
              function fn(a: Foo): Foo {}
            ",
            None,
        ),
        (
            "
              import Foo from 'foo';
              let foo: Foo;
            ",
            Some(serde_json::json!([{ "prefer": "type-imports" }])),
        ),
        (
            "
              import Foo from 'foo';
              let foo: Foo;
            ",
            Some(
                serde_json::json!([            { "prefer": "type-imports", "fixStyle": "inline-type-imports" },          ]),
            ),
        ),
        (
            "
              import { A, B } from 'foo';
              let foo: A;
              let bar: B;
            ",
            None,
        ),
        (
            "
              import { A as a, B as b } from 'foo';
              let foo: a;
              let bar: b;
            ",
            None,
        ),
        (
            "
              import Foo from 'foo';
              type Bar = typeof Foo; // TSTypeQuery
            ",
            None,
        ),
        (
            "
              import foo from 'foo';
              type Bar = foo.Bar; // TSQualifiedName
            ",
            None,
        ),
        (
            "
              import foo from 'foo';
              type Baz = (typeof foo.bar)['Baz']; // TSQualifiedName & TSTypeQuery
            ",
            None,
        ),
        (
            "
              import * as A from 'foo';
              let foo: A.Foo;
            ",
            None,
        ),
        (
            "
              import A, { B } from 'foo';
              let foo: A;
              let bar: B;
            ",
            None,
        ),
        (
            "
              import A, {} from 'foo';
              let foo: A;
            ",
            None,
        ),
        (
            "
              import { A, B } from 'foo';
              const foo: A = B();
            ",
            None,
        ),
        (
            "
              import { A, B, C } from 'foo';
              const foo: A = B();
              let bar: C;
            ",
            None,
        ),
        (
            "
              import { A, B, C, D } from 'foo';
              const foo: A = B();
              type T = { bar: C; baz: D };
            ",
            None,
        ),
        (
            "
              import A, { B, C, D } from 'foo';
              B();
              type T = { foo: A; bar: C; baz: D };
            ",
            None,
        ),
        (
            "
              import A, { B } from 'foo';
              B();
              type T = A;
            ",
            None,
        ),
        (
            "
              import type Already1Def from 'foo';
              import type { Already1 } from 'foo';
              import A, { B } from 'foo';
              import { C, D, E } from 'bar';
              import type { Already2 } from 'bar';
              type T = { b: B; c: C; d: D };
            ",
            None,
        ),
        (
            "
              import A, { /* comment */ B } from 'foo';
              type T = B;
            ",
            None,
        ),
        (
            "
              import { A, B, C } from 'foo';
              import { D, E, F, } from 'bar';
              type T = A | D;
            ",
            None,
        ),
        (
            "
              import { A, B, C } from 'foo';
              import { D, E, F, } from 'bar';
              type T = B | E;
            ",
            None,
        ),
        (
            "
              import { A, B, C } from 'foo';
              import { D, E, F, } from 'bar';
              type T = C | F;
            ",
            None,
        ),
        (
            "
              import { Type1, Type2 } from 'named_types';
              import Type from 'default_type';
              import * as Types from 'namespace_type';
              import Default, { Named } from 'default_and_named_type';
              type T = Type1 | Type2 | Type | Types.A | Default | Named;
            ",
            None,
        ),
        (
            "
              import { Value1, Type1 } from 'named_import';
              import Type2, { Value2 } from 'default_import';
              import Value3, { Type3 } from 'default_import2';
              import Type4, { Type5, Value4 } from 'default_and_named_import';
              type T = Type1 | Type2 | Type3 | Type4 | Type5;
            ",
            None,
        ),
        (
            "
              let foo: import('foo');
              let bar: import('foo').Bar;
            ",
            None,
        ),
        (
            "
              let foo: import('foo');
            ",
            Some(serde_json::json!([{ "prefer": "type-imports" }])),
        ),
        (
            "
              import type Foo from 'foo';
              let foo: Foo;
            ",
            Some(serde_json::json!([{ "prefer": "no-type-imports" }])),
        ),
        (
            "
              import type { Foo } from 'foo';
              let foo: Foo;
            ",
            Some(serde_json::json!([{ "prefer": "no-type-imports" }])),
        ),
        // (
        //     "
        //       import Type from 'foo';

        //       type T = typeof Type;
        //       type T = typeof Type.foo;
        //     ",
        //     None,
        // ),
        // (
        //     "
        //       import { Type } from 'foo';

        //       type T = typeof Type;
        //       type T = typeof Type.foo;
        //     ",
        //     None,
        // ),
        // (
        //     "
        //       import * as Type from 'foo';

        //       type T = typeof Type;
        //       type T = typeof Type.foo;
        //     ",
        //     None,
        // ),
        // (
        //     "
        //       import type Type from 'foo';

        //       type T = typeof Type;
        //       type T = typeof Type.foo;
        //     ",
        //     Some(serde_json::json!([{ "prefer": "no-type-imports" }])),
        // ),
        // (
        //     "
        //       import type { Type } from 'foo';

        //       type T = typeof Type;
        //       type T = typeof Type.foo;
        //     ",
        //     Some(serde_json::json!([{ "prefer": "no-type-imports" }])),
        // ),
        // (
        //     "
        //       import type * as Type from 'foo';

        //       type T = typeof Type;
        //       type T = typeof Type.foo;
        //     ",
        //     Some(serde_json::json!([{ "prefer": "no-type-imports" }])),
        // ),
        (
            "
              import Type from 'foo';
        
              export type { Type }; // is a type-only export
            ",
            None,
        ),
        (
            "
              import { Type } from 'foo';
        
              export type { Type }; // is a type-only export
            ",
            None,
        ),
        (
            "
              import * as Type from 'foo';
        
              export type { Type }; // is a type-only export
            ",
            None,
        ),
        (
            "
              import type Type from 'foo';
        
              export { Type }; // is a type-only export
              export default Type; // is a type-only export
              export type { Type }; // is a type-only export
            ",
            Some(serde_json::json!([{ "prefer": "no-type-imports" }])),
        ),
        (
            "
              import type { Type } from 'foo';
        
              export { Type }; // is a type-only export
              export default Type; // is a type-only export
              export type { Type }; // is a type-only export
            ",
            Some(serde_json::json!([{ "prefer": "no-type-imports" }])),
        ),
        (
            "
              import type * as Type from 'foo';
              
              export { Type }; // is a type-only export
              export default Type; // is a type-only export
              export type { Type }; // is a type-only export
            ",
            Some(serde_json::json!([{ "prefer": "no-type-imports" }])),
        ),
        (
            "
              import type /*comment*/ * as AllType from 'foo';
              import type // comment
              DefType from 'foo';
              import type /*comment*/ { Type } from 'foo';
              
              type T = { a: AllType; b: DefType; c: Type };
            ",
            Some(serde_json::json!([{ "prefer": "no-type-imports" }])),
        ),
        (
            "
              import Default, * as Rest from 'module';
              const a: Rest.A = '';
            ",
            Some(serde_json::json!([{ "prefer": "type-imports" }])),
        ),
        (
            "
              import Default, * as Rest from 'module';
              const a: Default = '';
            ",
            Some(serde_json::json!([{ "prefer": "type-imports" }])),
        ),
        (
            "
              import Default, * as Rest from 'module';
              const a: Default = '';
              const b: Rest.A = '';
            ",
            Some(serde_json::json!([{ "prefer": "type-imports" }])),
        ),
        (
            "
              import Default, /*comment*/ * as Rest from 'module';
              const a: Default = '';
            ",
            Some(serde_json::json!([{ "prefer": "type-imports" }])),
        ),
        (
            "
              import Default /*comment1*/, /*comment2*/ { Data } from 'module';
              const a: Default = '';
            ",
            Some(serde_json::json!([{ "prefer": "type-imports" }])),
        ),
        // (
        //     "
        //       import Foo from 'foo';
        //       @deco
        //       class A {
        //         constructor(foo: Foo) {}
        //       }
        //     ",
        //     None,
        // ),
        (
            "
              import { type A, B } from 'foo';
              type T = A;
              const b = B;
            ",
            Some(serde_json::json!([{ "prefer": "no-type-imports" }])),
        ),
        (
            "
              import { A, B, type C } from 'foo';
              type T = A | C;
              const b = B;
            ",
            Some(serde_json::json!([{ "prefer": "type-imports" }])),
        ),
        (
            "
              import { A, B } from 'foo';
              let foo: A;
              let bar: B;
            ",
            Some(
                serde_json::json!([            { "prefer": "type-imports", "fixStyle": "inline-type-imports" },          ]),
            ),
        ),
        (
            "
              import { A, B } from 'foo';
              
              let foo: A;
              B();
            ",
            Some(
                serde_json::json!([            { "prefer": "type-imports", "fixStyle": "inline-type-imports" },          ]),
            ),
        ),
        (
            "
              import { A, B } from 'foo';
              type T = A;
              B();
            ",
            Some(
                serde_json::json!([            { "prefer": "type-imports", "fixStyle": "inline-type-imports" },          ]),
            ),
        ),
        (
            "
              import { A } from 'foo';
              import { B } from 'foo';
              type T = A;
              type U = B;
            ",
            Some(
                serde_json::json!([            { "prefer": "type-imports", "fixStyle": "inline-type-imports" },          ]),
            ),
        ),
        (
            "
              import { A } from 'foo';
              import B from 'foo';
              type T = A;
              type U = B;
            ",
            Some(
                serde_json::json!([            { "prefer": "type-imports", "fixStyle": "inline-type-imports" },          ]),
            ),
        ),
        (
            "
              import A, { B, C } from 'foo';
              type T = B;
              type U = C;
              A();
            ",
            Some(
                serde_json::json!([            { "prefer": "type-imports", "fixStyle": "inline-type-imports" },          ]),
            ),
        ),
        (
            "
              import A, { B, C } from 'foo';
              type T = B;
              type U = C;
              type V = A;
            ",
            Some(
                serde_json::json!([            { "prefer": "type-imports", "fixStyle": "inline-type-imports" },          ]),
            ),
        ),
        (
            "
              import A, { B, C as D } from 'foo';
              type T = B;
              type U = D;
              type V = A;
            ",
            Some(
                serde_json::json!([            { "prefer": "type-imports", "fixStyle": "inline-type-imports" },          ]),
            ),
        ),
        (
            "
              import { /* comment */ A, B } from 'foo';
              type T = A;
            ",
            Some(
                serde_json::json!([            { "prefer": "type-imports", "fixStyle": "inline-type-imports" },          ]),
            ),
        ),
        (
            "
              import { B, /* comment */ A } from 'foo';
              type T = A;
            ",
            Some(
                serde_json::json!([            { "prefer": "type-imports", "fixStyle": "inline-type-imports" },          ]),
            ),
        ),
        (
            "
              import { A, B, C } from 'foo';
              import type { D } from 'deez';
              
              const foo: A = B();
              let bar: C;
              let baz: D;
            ",
            Some(
                serde_json::json!([            { "prefer": "type-imports", "fixStyle": "inline-type-imports" },          ]),
            ),
        ),
        (
            "
              import { A, B, type C } from 'foo';
              import type { D } from 'deez';
              const foo: A = B();
              let bar: C;
              let baz: D;
            ",
            Some(
                serde_json::json!([            { "prefer": "type-imports", "fixStyle": "inline-type-imports" },          ]),
            ),
        ),
        (
            "
              import A from 'foo';
              export = {} as A;
            ",
            Some(
                serde_json::json!([            { "prefer": "type-imports", "fixStyle": "inline-type-imports" },          ]),
            ),
        ),
        (
            "
              import { A } from 'foo';
              export = {} as A;
            ",
            Some(
                serde_json::json!([            { "prefer": "type-imports", "fixStyle": "inline-type-imports" },          ]),
            ),
        ),
        // (
        //     "
        //         import Foo from 'foo';
        //         @deco
        //         class A {
        //           constructor(foo: Foo) {}
        //         }
        //     ",
        //     None,
        // ),
        // (
        //     "
        //         import Foo from 'foo';
        //         class A {
        //           @deco
        //           foo: Foo;
        //         }
        //     ",
        //     None,
        // ),
        // (
        //     "
        //         import Foo from 'foo';
        //         class A {
        //           @deco
        //           foo(foo: Foo) {}
        //         }
        //     ",
        //     None,
        // ),
        (
            "
                import Foo from 'foo';
                class A {
                  @deco
                  foo(): Foo {}
                }
            ",
            None,
        ),
        (
            "
                import Foo from 'foo';
                class A {
                  foo(@deco foo: Foo) {}
                }
            ",
            None,
        ),
        (
            "
                import Foo from 'foo';
                class A {
                  @deco
                  set foo(value: Foo) {}
                }
            ",
            None,
        ),
        (
            "
                import Foo from 'foo';
                class A {
                  @deco
                  get foo() {}
              
                  set foo(value: Foo) {}
                }
            ",
            None,
        ),
        (
            "
                import Foo from 'foo';
                class A {
                  @deco
                  get foo() {}
              
                  set ['foo'](value: Foo) {}
                }
            ",
            None,
        ),
        // (
        //     "
        //         import * as foo from 'foo';
        //         @deco
        //         class A {
        //           constructor(foo: foo.Foo) {}
        //         }
        //     ",
        //     None,
        // ),
        (
            "
              import 'foo';
              import { Foo, Bar } from 'foo';
              function test(foo: Foo) {}
            ",
            None,
        ),
        (
            "
              import {} from 'foo';
              import { Foo, Bar } from 'foo';
              function test(foo: Foo) {}
            ",
            None,
        ),
        // experimentalDecorators: true + emitDecoratorMetadata: true
        // (
        //     "
        //       import Foo from 'foo';
        //       export type T = Foo;
        //     ",
        //     None,
        // ),
    ];

    // let fix = vec![
    // (
    //     "
    //       import Foo from 'foo';
    //       let foo: Foo;
    //       type Bar = Foo;
    //       interface Baz {
    //         foo: Foo;
    //       }
    //       function fn(a: Foo): Foo {}
    //     ",
    //     "
    //       import type Foo from 'foo';
    //       let foo: Foo;
    //       type Bar = Foo;
    //       interface Baz {
    //         foo: Foo;
    //       }
    //       function fn(a: Foo): Foo {}
    //     ",
    //     None,
    // ),
    //     (
    //         "
    //           import Foo from 'foo';
    //           let foo: Foo;
    //         ",
    //         "
    //           import type Foo from 'foo';
    //           let foo: Foo;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import Foo from 'foo';
    //           let foo: Foo;
    //         ",
    //         "
    //           import type Foo from 'foo';
    //           let foo: Foo;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import { A, B } from 'foo';
    //           let foo: A;
    //           let bar: B;
    //         ",
    //         "
    //           import type { A, B } from 'foo';
    //           let foo: A;
    //           let bar: B;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import { A as a, B as b } from 'foo';
    //           let foo: a;
    //           let bar: b;
    //         ",
    //         "
    //           import type { A as a, B as b } from 'foo';
    //           let foo: a;
    //           let bar: b;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import Foo from 'foo';
    //           type Bar = typeof Foo; // TSTypeQuery
    //         ",
    //         "
    //           import type Foo from 'foo';
    //           type Bar = typeof Foo; // TSTypeQuery
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import foo from 'foo';
    //           type Bar = foo.Bar; // TSQualifiedName
    //         ",
    //         "
    //           import type foo from 'foo';
    //           type Bar = foo.Bar; // TSQualifiedName
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import foo from 'foo';
    //           type Baz = (typeof foo.bar)['Baz']; // TSQualifiedName & TSTypeQuery
    //         ",
    //         "
    //           import type foo from 'foo';
    //           type Baz = (typeof foo.bar)['Baz']; // TSQualifiedName & TSTypeQuery
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import * as A from 'foo';
    //           let foo: A.Foo;
    //         ",
    //         "
    //           import type * as A from 'foo';
    //           let foo: A.Foo;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import A, { B } from 'foo';
    //           let foo: A;
    //           let bar: B;
    //         ",
    //         "
    //           import type { B } from 'foo';
    //           import type A from 'foo';
    //           let foo: A;
    //           let bar: B;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import A, {} from 'foo';
    //           let foo: A;
    //         ",
    //         "
    //           import type A from 'foo';
    //           let foo: A;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import { A, B } from 'foo';
    //           const foo: A = B();
    //         ",
    //         "
    //           import type { A} from 'foo';
    //           import { B } from 'foo';
    //           const foo: A = B();
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import { A, B, C } from 'foo';
    //           const foo: A = B();
    //           let bar: C;
    //         ",
    //         "
    //           import type { A, C } from 'foo';
    //           import { B } from 'foo';
    //           const foo: A = B();
    //           let bar: C;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import { A, B, C, D } from 'foo';
    //           const foo: A = B();
    //           type T = { bar: C; baz: D };
    //         ",
    //         "
    //           import type { A, C, D } from 'foo';
    //           import { B } from 'foo';
    //           const foo: A = B();
    //           type T = { bar: C; baz: D };
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import A, { B, C, D } from 'foo';
    //           B();
    //           type T = { foo: A; bar: C; baz: D };
    //         ",
    //         "
    //           import type { C, D } from 'foo';
    //           import type A from 'foo';
    //           import { B } from 'foo';
    //           B();
    //           type T = { foo: A; bar: C; baz: D };
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import A, { B } from 'foo';
    //           B();
    //           type T = A;
    //         ",
    //         "
    //           import type A from 'foo';
    //           import { B } from 'foo';
    //           B();
    //           type T = A;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import type Already1Def from 'foo';
    //           import type { Already1 } from 'foo';
    //           import A, { B } from 'foo';
    //           import { C, D, E } from 'bar';
    //           import type { Already2 } from 'bar';
    //           type T = { b: B; c: C; d: D };
    //         ",
    //         "
    //           import type Already1Def from 'foo';
    //           import type { Already1 , B } from 'foo';
    //           import A from 'foo';
    //           import { E } from 'bar';
    //           import type { Already2 , C, D} from 'bar';
    //           type T = { b: B; c: C; d: D };
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import A, { /* comment */ B } from 'foo';
    //           type T = B;
    //         ",
    //         "
    //           import type { /* comment */ B } from 'foo';
    //           import A from 'foo';
    //           type T = B;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import { A, B, C } from 'foo';
    //           import { D, E, F, } from 'bar';
    //           type T = A | D;
    //         ",
    //         "
    //           import type { A} from 'foo';
    //           import { B, C } from 'foo';
    //           import type { D} from 'bar';
    //           import { E, F, } from 'bar';
    //           type T = A | D;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import { A, B, C } from 'foo';
    //           import { D, E, F, } from 'bar';
    //           type T = B | E;
    //         ",
    //         "
    //           import type { B} from 'foo';
    //           import { A, C } from 'foo';
    //           import type { E} from 'bar';
    //           import { D, F, } from 'bar';
    //           type T = B | E;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import { A, B, C } from 'foo';
    //           import { D, E, F, } from 'bar';
    //           type T = C | F;
    //         ",
    //         "
    //           import type { C } from 'foo';
    //           import { A, B } from 'foo';
    //           import type { F} from 'bar';
    //           import { D, E } from 'bar';
    //           type T = C | F;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import { Type1, Type2 } from 'named_types';
    //           import Type from 'default_type';
    //           import * as Types from 'namespace_type';
    //           import Default, { Named } from 'default_and_named_type';
    //           type T = Type1 | Type2 | Type | Types.A | Default | Named;
    //         ",
    //         "
    //           import type { Type1, Type2 } from 'named_types';
    //           import type Type from 'default_type';
    //           import type * as Types from 'namespace_type';
    //           import type { Named } from 'default_and_named_type';
    //           import type Default from 'default_and_named_type';
    //           type T = Type1 | Type2 | Type | Types.A | Default | Named;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import { Value1, Type1 } from 'named_import';
    //           import Type2, { Value2 } from 'default_import';
    //           import Value3, { Type3 } from 'default_import2';
    //           import Type4, { Type5, Value4 } from 'default_and_named_import';
    //           type T = Type1 | Type2 | Type3 | Type4 | Type5;
    //         ",
    //         "
    //           import type { Type1 } from 'named_import';
    //           import { Value1 } from 'named_import';
    //           import type Type2 from 'default_import';
    //           import { Value2 } from 'default_import';
    //           import type { Type3 } from 'default_import2';
    //           import Value3 from 'default_import2';
    //           import type { Type5} from 'default_and_named_import';
    //           import type Type4 from 'default_and_named_import';
    //           import { Value4 } from 'default_and_named_import';
    //           type T = Type1 | Type2 | Type3 | Type4 | Type5;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import type Foo from 'foo';
    //           let foo: Foo;
    //         ",
    //         "
    //           import Foo from 'foo';
    //           let foo: Foo;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import type { Foo } from 'foo';
    //           let foo: Foo;
    //         ",
    //         "
    //           import { Foo } from 'foo';
    //           let foo: Foo;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import Type from 'foo';
    //
    //           type T = typeof Type;
    //           type T = typeof Type.foo;
    //         ",
    //         "
    //           import type Type from 'foo';
    //
    //           type T = typeof Type;
    //           type T = typeof Type.foo;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import { Type } from 'foo';
    //
    //           type T = typeof Type;
    //           type T = typeof Type.foo;
    //         ",
    //         "
    //           import type { Type } from 'foo';
    //
    //           type T = typeof Type;
    //           type T = typeof Type.foo;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import * as Type from 'foo';
    //
    //           type T = typeof Type;
    //           type T = typeof Type.foo;
    //         ",
    //         "
    //           import type * as Type from 'foo';
    //
    //           type T = typeof Type;
    //           type T = typeof Type.foo;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import type Type from 'foo';
    //
    //           type T = typeof Type;
    //           type T = typeof Type.foo;
    //         ",
    //         "
    //           import Type from 'foo';
    //
    //           type T = typeof Type;
    //           type T = typeof Type.foo;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import type { Type } from 'foo';
    //
    //           type T = typeof Type;
    //           type T = typeof Type.foo;
    //         ",
    //         "
    //           import { Type } from 'foo';
    //
    //           type T = typeof Type;
    //           type T = typeof Type.foo;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import type * as Type from 'foo';
    //
    //           type T = typeof Type;
    //           type T = typeof Type.foo;
    //         ",
    //         "
    //           import * as Type from 'foo';
    //
    //           type T = typeof Type;
    //           type T = typeof Type.foo;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import Type from 'foo';
    //
    //           export type { Type }; // is a type-only export
    //         ",
    //         "
    //           import type Type from 'foo';
    //
    //           export type { Type }; // is a type-only export
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import { Type } from 'foo';
    //
    //           export type { Type }; // is a type-only export
    //         ",
    //         "
    //           import type { Type } from 'foo';
    //
    //           export type { Type }; // is a type-only export
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import * as Type from 'foo';
    //
    //           export type { Type }; // is a type-only export
    //         ",
    //         "
    //           import type * as Type from 'foo';
    //
    //           export type { Type }; // is a type-only export
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import type Type from 'foo';
    //
    //           export { Type }; // is a type-only export
    //           export default Type; // is a type-only export
    //           export type { Type }; // is a type-only export
    //         ",
    //         "
    //           import Type from 'foo';
    //
    //           export { Type }; // is a type-only export
    //           export default Type; // is a type-only export
    //           export type { Type }; // is a type-only export
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import type { Type } from 'foo';
    //
    //           export { Type }; // is a type-only export
    //           export default Type; // is a type-only export
    //           export type { Type }; // is a type-only export
    //         ",
    //         "
    //           import { Type } from 'foo';
    //
    //           export { Type }; // is a type-only export
    //           export default Type; // is a type-only export
    //           export type { Type }; // is a type-only export
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import type * as Type from 'foo';
    //
    //           export { Type }; // is a type-only export
    //           export default Type; // is a type-only export
    //           export type { Type }; // is a type-only export
    //         ",
    //         "
    //           import * as Type from 'foo';
    //
    //           export { Type }; // is a type-only export
    //           export default Type; // is a type-only export
    //           export type { Type }; // is a type-only export
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import type /*comment*/ * as AllType from 'foo';
    //           import type // comment
    //           DefType from 'foo';
    //           import type /*comment*/ { Type } from 'foo';
    //
    //           type T = { a: AllType; b: DefType; c: Type };
    //         ",
    //         "
    //           import /*comment*/ * as AllType from 'foo';
    //           import // comment
    //           DefType from 'foo';
    //           import /*comment*/ { Type } from 'foo';
    //
    //           type T = { a: AllType; b: DefType; c: Type };
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import Default, * as Rest from 'module';
    //           const a: Rest.A = '';
    //         ",
    //         "
    //           import type * as Rest from 'module';
    //           import Default from 'module';
    //           const a: Rest.A = '';
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import Default, * as Rest from 'module';
    //           const a: Default = '';
    //         ",
    //         "
    //           import type Default from 'module';
    //           import * as Rest from 'module';
    //           const a: Default = '';
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import Default, * as Rest from 'module';
    //           const a: Default = '';
    //           const b: Rest.A = '';
    //         ",
    //         "
    //           import type * as Rest from 'module';
    //           import type Default from 'module';
    //           const a: Default = '';
    //           const b: Rest.A = '';
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import Default, /*comment*/ * as Rest from 'module';
    //           const a: Default = '';
    //         ",
    //         "
    //           import type Default from 'module';
    //           import /*comment*/ * as Rest from 'module';
    //           const a: Default = '';
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import Default /*comment1*/, /*comment2*/ { Data } from 'module';
    //           const a: Default = '';
    //         ",
    //         "
    //           import type Default /*comment1*/ from 'module';
    //           import /*comment2*/ { Data } from 'module';
    //           const a: Default = '';
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import Foo from 'foo';
    //           @deco
    //           class A {
    //             constructor(foo: Foo) {}
    //           }
    //         ",
    //         "
    //           import type Foo from 'foo';
    //           @deco
    //           class A {
    //             constructor(foo: Foo) {}
    //           }
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import { type A, B } from 'foo';
    //           type T = A;
    //           const b = B;
    //         ",
    //         "
    //           import { A, B } from 'foo';
    //           type T = A;
    //           const b = B;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import { A, B, type C } from 'foo';
    //           type T = A | C;
    //           const b = B;
    //         ",
    //         "
    //           import type { A} from 'foo';
    //           import { B, type C } from 'foo';
    //           type T = A | C;
    //           const b = B;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import { A, B } from 'foo';
    //           let foo: A;
    //           let bar: B;
    //         ",
    //         "
    //           import { type A, type B } from 'foo';
    //           let foo: A;
    //           let bar: B;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import { A, B } from 'foo';
    //
    //           let foo: A;
    //           B();
    //         ",
    //         "
    //           import { type A, B } from 'foo';
    //
    //           let foo: A;
    //           B();
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import { A, B } from 'foo';
    //           type T = A;
    //           B();
    //         ",
    //         "
    //           import { type A, B } from 'foo';
    //           type T = A;
    //           B();
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import { A } from 'foo';
    //           import { B } from 'foo';
    //           type T = A;
    //           type U = B;
    //         ",
    //         "
    //           import { type A } from 'foo';
    //           import { type B } from 'foo';
    //           type T = A;
    //           type U = B;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import { A } from 'foo';
    //           import B from 'foo';
    //           type T = A;
    //           type U = B;
    //         ",
    //         "
    //           import { type A } from 'foo';
    //           import type B from 'foo';
    //           type T = A;
    //           type U = B;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import A, { B, C } from 'foo';
    //           type T = B;
    //           type U = C;
    //           A();
    //         ",
    //         "
    //           import A, { type B, type C } from 'foo';
    //           type T = B;
    //           type U = C;
    //           A();
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import A, { B, C } from 'foo';
    //           type T = B;
    //           type U = C;
    //           type V = A;
    //         ",
    //         "
    //           import {type B, type C} from 'foo';
    //           import type A from 'foo';
    //           type T = B;
    //           type U = C;
    //           type V = A;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import A, { B, C as D } from 'foo';
    //           type T = B;
    //           type U = D;
    //           type V = A;
    //         ",
    //         "
    //           import {type B, type C as D} from 'foo';
    //           import type A from 'foo';
    //           type T = B;
    //           type U = D;
    //           type V = A;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import { /* comment */ A, B } from 'foo';
    //           type T = A;
    //         ",
    //         "
    //           import { /* comment */ type A, B } from 'foo';
    //           type T = A;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import { B, /* comment */ A } from 'foo';
    //           type T = A;
    //         ",
    //         "
    //           import { B, /* comment */ type A } from 'foo';
    //           type T = A;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import { A, B, C } from 'foo';
    //           import type { D } from 'deez';
    //
    //           const foo: A = B();
    //           let bar: C;
    //           let baz: D;
    //         ",
    //         "
    //           import { type A, B, type C } from 'foo';
    //           import type { D } from 'deez';
    //
    //           const foo: A = B();
    //           let bar: C;
    //           let baz: D;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import { A, B, type C } from 'foo';
    //           import type { D } from 'deez';
    //           const foo: A = B();
    //           let bar: C;
    //           let baz: D;
    //         ",
    //         "
    //           import { type A, B, type C } from 'foo';
    //           import type { D } from 'deez';
    //           const foo: A = B();
    //           let bar: C;
    //           let baz: D;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import A from 'foo';
    //           export = {} as A;
    //         ",
    //         "
    //           import type A from 'foo';
    //           export = {} as A;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import { A } from 'foo';
    //           export = {} as A;
    //         ",
    //         "
    //           import { type A } from 'foo';
    //           export = {} as A;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //             import Foo from 'foo';
    //             /@deco
    //             class A {
    //               constructor(foo: Foo) {}
    //             }
    //         ",
    //         "
    //             import type Foo from 'foo';
    //             @deco
    //             class A {
    //               constructor(foo: Foo) {}
    //             }
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //             import Foo from 'foo';
    //             class A {
    //               @deco
    //               foo: Foo;
    //             }
    //         ",
    //         "
    //             import type Foo from 'foo';
    //             class A {
    //               @deco
    //               foo: Foo;
    //             }
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //             import Foo from 'foo';
    //             class A {
    //               @deco
    //               foo(foo: Foo) {}
    //             }
    //         ",
    //         "
    //             import type Foo from 'foo';
    //             class A {
    //               @deco
    //               foo(foo: Foo) {}
    //             }
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //             import Foo from 'foo';
    //             class A {
    //               @deco
    //               foo(): Foo {}
    //             }
    //         ",
    //         "
    //             import type Foo from 'foo';
    //             class A {
    //               @deco
    //               foo(): Foo {}
    //             }
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //             import Foo from 'foo';
    //             class A {
    //               foo(@deco foo: Foo) {}
    //             }
    //         ",
    //         "
    //             import type Foo from 'foo';
    //             class A {
    //               foo(@deco foo: Foo) {}
    //             }
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //             import Foo from 'foo';
    //             class A {
    //               @deco
    //               set foo(value: Foo) {}
    //             }
    //         ",
    //         "
    //             import type Foo from 'foo';
    //             class A {
    //               @deco
    //               set foo(value: Foo) {}
    //             }
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //             import Foo from 'foo';
    //             class A {
    //               @deco
    //               get foo() {}
    //
    //               set foo(value: Foo) {}
    //             }
    //         ",
    //         "
    //             import type Foo from 'foo';
    //             class A {
    //               @deco
    //               get foo() {}
    //
    //               set foo(value: Foo) {}
    //             }
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //             import Foo from 'foo';
    //             class A {
    //               @deco
    //               get foo() {}
    //
    //               set ['foo'](value: Foo) {}
    //             }
    //         ",
    //         "
    //             import type Foo from 'foo';
    //             class A {
    //               @deco
    //               get foo() {}
    //
    //               set ['foo'](value: Foo) {}
    //             }
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //             import * as foo from 'foo';
    //             @deco
    //             class A {
    //               constructor(foo: foo.Foo) {}
    //             }
    //         ",
    //         "
    //             import type * as foo from 'foo';
    //             @deco
    //             class A {
    //               constructor(foo: foo.Foo) {}
    //             }
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import 'foo';
    //           import { Foo, Bar } from 'foo';
    //           function test(foo: Foo) {}
    //         ",
    //         "
    //           import 'foo';
    //           import type { Foo} from 'foo';
    //           import { Bar } from 'foo';
    //           function test(foo: Foo) {}
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import {} from 'foo';
    //           import { Foo, Bar } from 'foo';
    //           function test(foo: Foo) {}
    //         ",
    //         "
    //           import {} from 'foo';
    //           import type { Foo} from 'foo';
    //           import { Bar } from 'foo';
    //           function test(foo: Foo) {}
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import Foo from 'foo';
    //           export type T = Foo;
    //         ",
    //         "
    //           import type Foo from 'foo';
    //           export type T = Foo;
    //         ",
    //         None,
    //     ),
    // ];
    Tester::new(ConsistentTypeImports::NAME, pass, fail).test_and_snapshot();
}
