use oxc_ast::{
    ast::{ImportDeclarationSpecifier, ModuleDeclaration},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{self, Error},
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{Atom, Span};

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("typescript-eslint(consistent-type-imports):")]
#[diagnostic(severity(warning), help(""))]
struct ConsistentTypeImportsDiagnostic(#[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("typescript-eslint(consistent-type-imports): All imports in the declaration are only used as types.")]
#[diagnostic(severity(warning), help("Please use `import type` instead."))]
struct TypeOverValueDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct ConsistentTypeImports;

declare_oxc_lint!(
    /// ### What it does
    ///
    ///
    /// ### Why is this bad?
    ///
    ///
    /// ### Example
    /// ```javascript
    /// ```
    ConsistentTypeImports,
    correctness
);

impl Rule for ConsistentTypeImports {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::ModuleDeclaration(ModuleDeclaration::ImportDeclaration(decl)) = node.kind()
        {
            if decl.import_kind.is_type() {
                return;
            }
            if let Some(specifiers) = &decl.specifiers {
                specifiers.iter().for_each(|specifier| match specifier {
                    ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                        if !specifier.import_kind.is_type() {
                            check_specifier(ctx, &specifier.local.name, specifier.span);
                        }
                    }
                    ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                        check_specifier(ctx, &specifier.local.name, specifier.span);
                    }
                    ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                        check_specifier(ctx, &specifier.local.name, specifier.span);
                    }
                });
            }
        }
    }
}

fn check_specifier(ctx: &LintContext, name: &Atom, span: Span) {
    let root_scope_id = ctx.semantic().scopes().root_scope_id();
    if let Some(symbol_id) = ctx.semantic().scopes().get_binding(root_scope_id, name) {
        if !ctx.semantic().symbols().resolved_references[symbol_id].is_empty()
            && ctx.semantic().symbol_references(symbol_id).all(oxc_semantic::Reference::is_type)
        {
            ctx.diagnostic(TypeOverValueDiagnostic(span));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            r"
			      import Foo from 'foo';
			      const foo: Foo = new Foo();
			    ",
            None,
        ),
        (
            r"
			      import foo from 'foo';
			      const foo: foo.Foo = foo.fn();
			    ",
            None,
        ),
        (
            r"
			      import { A, B } from 'foo';
			      const foo: A = B();
			      const bar = new A();
			    ",
            None,
        ),
        (
            r"
			      import Foo from 'foo';
			    ",
            None,
        ),
        (
            r"
			      import Foo from 'foo';
			      type T<Foo> = Foo; // shadowing
			    ",
            None,
        ),
        (
            r"
			      import Foo from 'foo';
			      function fn() {
			        type Foo = {}; // shadowing
			        let foo: Foo;
			      }
			    ",
            None,
        ),
        (
            r"
			      import { A, B } from 'foo';
			      const b = B;
			    ",
            None,
        ),
        (
            r"
			      import { A, B, C as c } from 'foo';
			      const d = c;
			    ",
            None,
        ),
        (
            r"
			      import {} from 'foo'; // empty
			    ",
            None,
        ),
        (
            r"
			        let foo: import('foo');
			        let bar: import('foo').Bar;
			      ",
            Some(serde_json::json!([{ "disallowTypeAnnotations": false }])),
        ),
        (
            r"
			        import Foo from 'foo';
			        let foo: Foo;
			      ",
            Some(serde_json::json!([{ "prefer": "no-type-imports" }])),
        ),
        (
            r"
			      import type Type from 'foo';
			
			      type T1 = typeof Type;
			      type T2 = typeof Type.foo;
			    ",
            None,
        ),
        (
            r"
			      import type { Type } from 'foo';
			
			      type T1 = typeof Type;
			      type T2 = typeof Type.foo;
			    ",
            None,
        ),
        (
            r"
			      import type * as Type from 'foo';
			
			      type T1 = typeof Type;
			      type T2 = typeof Type.foo;
			    ",
            None,
        ),
        (
            r"
			        import Type from 'foo';
			
			        type T1 = typeof Type;
			        type T2 = typeof Type.foo;
			      ",
            Some(serde_json::json!([{ "prefer": "no-type-imports" }])),
        ),
        (
            r"
			        import { Type } from 'foo';
			
			        type T1 = typeof Type;
			        type T2 = typeof Type.foo;
			      ",
            Some(serde_json::json!([{ "prefer": "no-type-imports" }])),
        ),
        (
            r"
			        import * as Type from 'foo';
			
			        type T1 = typeof Type;
			        type T2 = typeof Type.foo;
			      ",
            Some(serde_json::json!([{ "prefer": "no-type-imports" }])),
        ),
        (
            r"
			        import * as Type from 'foo' assert { type: 'json' };
			        const a: typeof Type = Type;
			      ",
            Some(serde_json::json!([{ "prefer": "no-type-imports" }])),
        ),
        (
            r"
			      import { type A } from 'foo';
			      type T = A;
			    ",
            None,
        ),
        (
            r"
			      import { type A, B } from 'foo';
			      type T = A;
			      const b = B;
			    ",
            None,
        ),
        (
            r"
			      import { type A, type B } from 'foo';
			      type T = A;
			      type Z = B;
			    ",
            None,
        ),
        (
            r"
			      import { B } from 'foo';
			      import { type A } from 'foo';
			      type T = A;
			      const b = B;
			    ",
            None,
        ),
        (
            r"
			        import { B, type A } from 'foo';
			        type T = A;
			        const b = B;
			      ",
            Some(serde_json::json!([{ "fixStyle": "inline-type-imports" }])),
        ),
        (
            r"
			        import { B } from 'foo';
			        import type A from 'baz';
			        type T = A;
			        const b = B;
			      ",
            Some(serde_json::json!([{ "fixStyle": "inline-type-imports" }])),
        ),
        (
            r"
			        import { type B } from 'foo';
			        import type { A } from 'foo';
			        type T = A;
			        const b = B;
			      ",
            Some(serde_json::json!([{ "fixStyle": "inline-type-imports" }])),
        ),
        (
            r"
			        import { B, type C } from 'foo';
			        import type A from 'baz';
			        type T = A;
			        type Z = C;
			        const b = B;
			      ",
            Some(
                serde_json::json!([{ "prefer": "type-imports", "fixStyle": "inline-type-imports" }]),
            ),
        ),
        (
            r"
			        import { B } from 'foo';
			        import type { A } from 'foo';
			        type T = A;
			        const b = B;
			      ",
            Some(
                serde_json::json!([{ "prefer": "type-imports", "fixStyle": "inline-type-imports" }]),
            ),
        ),
        (
            r"
			        import { B } from 'foo';
			        import { A } from 'foo';
			        type T = A;
			        const b = B;
			      ",
            Some(
                serde_json::json!([{ "prefer": "no-type-imports", "fixStyle": "inline-type-imports" }]),
            ),
        ),
        (
            r"
			      import Type from 'foo';
			
			      export { Type }; // is a value export
			      export default Type; // is a value export
			    ",
            None,
        ),
        (
            r"
			      import type Type from 'foo';
			
			      export { Type }; // is a type-only export
			      export default Type; // is a type-only export
			      export type { Type }; // is a type-only export
			    ",
            None,
        ),
        (
            r"
			      import { Type } from 'foo';
			
			      export { Type }; // is a value export
			      export default Type; // is a value export
			    ",
            None,
        ),
        (
            r"
			      import type { Type } from 'foo';
			
			      export { Type }; // is a type-only export
			      export default Type; // is a type-only export
			      export type { Type }; // is a type-only export
			    ",
            None,
        ),
        (
            r"
			      import * as Type from 'foo';
			
			      export { Type }; // is a value export
			      export default Type; // is a value export
			    ",
            None,
        ),
        (
            r"
			      import type * as Type from 'foo';
			
			      export { Type }; // is a type-only export
			      export default Type; // is a type-only export
			      export type { Type }; // is a type-only export
			    ",
            None,
        ),
        (
            r"
			        import Type from 'foo';
			
			        export { Type }; // is a type-only export
			        export default Type; // is a type-only export
			        export type { Type }; // is a type-only export
			      ",
            Some(serde_json::json!([{ "prefer": "no-type-imports" }])),
        ),
        (
            r"
			        import { Type } from 'foo';
			
			        export { Type }; // is a type-only export
			        export default Type; // is a type-only export
			        export type { Type }; // is a type-only export
			      ",
            Some(serde_json::json!([{ "prefer": "no-type-imports" }])),
        ),
        (
            r"
			        import * as Type from 'foo';
			
			        export { Type }; // is a type-only export
			        export default Type; // is a type-only export
			        export type { Type }; // is a type-only export
			      ",
            Some(serde_json::json!([{ "prefer": "no-type-imports" }])),
        ),
        (
            r"
			        import React from 'react';
			
			        export const ComponentFoo: React.FC = () => {
			          return <div>Foo Foo</div>;
			        };
			      ",
            None,
        ),
        (
            r"
			        import { h } from 'some-other-jsx-lib';
			
			        export const ComponentFoo: h.FC = () => {
			          return <div>Foo Foo</div>;
			        };
			      ",
            None,
        ),
        (
            r"
			        import { Fragment } from 'react';
			
			        export const ComponentFoo: Fragment = () => {
			          return <>Foo Foo</>;
			        };
			      ",
            None,
        ),
        (
            r"
			      import Default, * as Rest from 'module';
			      const a: typeof Default = Default;
			      const b: typeof Rest = Rest;
			    ",
            None,
        ),
        (
            r"
			        import Foo from 'foo';
			        @deco
			        class A {
			          constructor(foo: Foo) {}
			        }
			      ",
            None,
        ),
        (
            r"
			        import Foo from 'foo';
			        class A {
			          @deco
			          foo: Foo;
			        }
			      ",
            None,
        ),
        (
            r"
			        import Foo from 'foo';
			        class A {
			          @deco
			          foo(foo: Foo) {}
			        }
			      ",
            None,
        ),
        (
            r"
			        import Foo from 'foo';
			        class A {
			          @deco
			          foo(): Foo {}
			        }
			      ",
            None,
        ),
        (
            r"
			        import Foo from 'foo';
			        class A {
			          foo(@deco foo: Foo) {}
			        }
			      ",
            None,
        ),
        (
            r"
			        import Foo from 'foo';
			        class A {
			          @deco
			          set foo(value: Foo) {}
			        }
			      ",
            None,
        ),
        (
            r"
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
            r"
			        import Foo from 'foo';
			        class A {
			          @deco
			          get foo() {}
			
			          set ['foo'](value: Foo) {}
			        }
			      ",
            None,
        ),
        (
            r"
			        import type { Foo } from 'foo';
			        const key = 'k';
			        class A {
			          @deco
			          get [key]() {}
			
			          set [key](value: Foo) {}
			        }
			      ",
            None,
        ),
        (
            r"
			        import * as foo from 'foo';
			        @deco
			        class A {
			          constructor(foo: foo.Foo) {}
			        }
			      ",
            None,
        ),
        (
            r"
			        import Foo from 'foo';
			        @deco
			        class A {
			          constructor(foo: Foo) {}
			        }
			      ",
            None,
        ),
        (
            r"
			        import Foo from 'foo';
			        class A {
			          @deco
			          foo: Foo;
			        }
			      ",
            None,
        ),
        (
            r"
			        import Foo from 'foo';
			        class A {
			          @deco
			          foo(foo: Foo) {}
			        }
			      ",
            None,
        ),
        (
            r"
			        import Foo from 'foo';
			        class A {
			          @deco
			          foo(): Foo {}
			        }
			      ",
            None,
        ),
        (
            r"
			        import Foo from 'foo';
			        class A {
			          foo(@deco foo: Foo) {}
			        }
			      ",
            None,
        ),
        (
            r"
			        import Foo from 'foo';
			        class A {
			          @deco
			          set foo(value: Foo) {}
			        }
			      ",
            None,
        ),
        (
            r"
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
            r"
			        import Foo from 'foo';
			        class A {
			          @deco
			          get foo() {}
			
			          set ['foo'](value: Foo) {}
			        }
			      ",
            None,
        ),
        (
            r"
			        import type { Foo } from 'foo';
			        const key = 'k';
			        class A {
			          @deco
			          get [key]() {}
			
			          set [key](value: Foo) {}
			        }
			      ",
            None,
        ),
        (
            r"
			        import * as foo from 'foo';
			        @deco
			        class A {
			          constructor(foo: foo.Foo) {}
			        }
			      ",
            None,
        ),
        (
            r"
			        import type { ClassA } from './classA';
			
			        export class ClassB {
			          public constructor(node: ClassA) {}
			        }
			      ",
            None,
        ),
        (
            r"
			import type * as constants from './constants';
			
			export type Y = {
			  [constants.X]: ReadonlyArray<string>;
			};
			    ",
            None,
        ),
    ];

    let fail = vec![
        (
            r"
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
            r"
			        import Foo from 'foo';
			        let foo: Foo;
			      ",
            Some(serde_json::json!([{ "prefer": "type-imports" }])),
        ),
        (
            r"
			        import Foo from 'foo';
			        let foo: Foo;
			      ",
            Some(
                serde_json::json!([{ "prefer": "type-imports", "fixStyle": "inline-type-imports" }]),
            ),
        ),
        (
            r"
			        import { A, B } from 'foo';
			        let foo: A;
			        let bar: B;
			      ",
            None,
        ),
        (
            r"
			        import { A as a, B as b } from 'foo';
			        let foo: a;
			        let bar: b;
			      ",
            None,
        ),
        (
            r"
			        import Foo from 'foo';
			        type Bar = typeof Foo; // TSTypeQuery
			      ",
            None,
        ),
        (
            r"
			        import foo from 'foo';
			        type Bar = foo.Bar; // TSQualifiedName
			      ",
            None,
        ),
        (
            r"
			        import foo from 'foo';
			        type Baz = (typeof foo.bar)['Baz']; // TSQualifiedName & TSTypeQuery
			      ",
            None,
        ),
        (
            r"
			        import * as A from 'foo';
			        let foo: A.Foo;
			      ",
            None,
        ),
        (
            r"
			import A, { B } from 'foo';
			let foo: A;
			let bar: B;
			      ",
            None,
        ),
        (
            r"
			import { A, B } from 'foo';
			const foo: A = B();
			      ",
            None,
        ),
        (
            r"
			import { A, B, C } from 'foo';
			const foo: A = B();
			let bar: C;
			      ",
            None,
        ),
        (
            r"
			import { A, B, C, D } from 'foo';
			const foo: A = B();
			type T = { bar: C; baz: D };
			      ",
            None,
        ),
        (
            r"
			import A, { B, C, D } from 'foo';
			B();
			type T = { foo: A; bar: C; baz: D };
			      ",
            None,
        ),
        (
            r"
			import A, { B } from 'foo';
			B();
			type T = A;
			      ",
            None,
        ),
        (
            r"
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
            r"
			import A, { /* comment */ B } from 'foo';
			type T = B;
			      ",
            None,
        ),
        (
            r"
			import { Type1, Type2 } from 'named_types';
			import Type from 'default_type';
			import * as Types from 'namespace_type';
			import Default, { Named } from 'default_and_named_type';
			type T = Type1 | Type2 | Type | Types.A | Default | Named;
			      ",
            None,
        ),
        (
            r"
			import { Value1, Type1 } from 'named_import';
			import Type2, { Value2 } from 'default_import';
			import Value3, { Type3 } from 'default_import2';
			import Type4, { Type5, Value4 } from 'default_and_named_import';
			type T = Type1 | Type2 | Type3 | Type4 | Type5;
			      ",
            None,
        ),
        (
            r"
			        let foo: import('foo');
			        let bar: import('foo').Bar;
			      ",
            None,
        ),
        (
            r"
			        let foo: import('foo');
			      ",
            Some(serde_json::json!([{ "prefer": "type-imports" }])),
        ),
        (
            r"
			        import type Foo from 'foo';
			        let foo: Foo;
			      ",
            Some(serde_json::json!([{ "prefer": "no-type-imports" }])),
        ),
        (
            r"
			        import type { Foo } from 'foo';
			        let foo: Foo;
			      ",
            Some(serde_json::json!([{ "prefer": "no-type-imports" }])),
        ),
        (
            r"
			        import Type from 'foo';
			
			        type T = typeof Type;
			        type T = typeof Type.foo;
			      ",
            None,
        ),
        (
            r"
			        import { Type } from 'foo';
			
			        type T = typeof Type;
			        type T = typeof Type.foo;
			      ",
            None,
        ),
        (
            r"
			        import * as Type from 'foo';
			
			        type T = typeof Type;
			        type T = typeof Type.foo;
			      ",
            None,
        ),
        (
            r"
			        import type Type from 'foo';
			
			        type T = typeof Type;
			        type T = typeof Type.foo;
			      ",
            Some(serde_json::json!([{ "prefer": "no-type-imports" }])),
        ),
        (
            r"
			        import type { Type } from 'foo';
			
			        type T = typeof Type;
			        type T = typeof Type.foo;
			      ",
            Some(serde_json::json!([{ "prefer": "no-type-imports" }])),
        ),
        (
            r"
			        import type * as Type from 'foo';
			
			        type T = typeof Type;
			        type T = typeof Type.foo;
			      ",
            Some(serde_json::json!([{ "prefer": "no-type-imports" }])),
        ),
        (
            r"
			        import Type from 'foo';
			
			        export type { Type }; // is a type-only export
			      ",
            None,
        ),
        (
            r"
			        import { Type } from 'foo';
			
			        export type { Type }; // is a type-only export
			      ",
            None,
        ),
        (
            r"
			        import * as Type from 'foo';
			
			        export type { Type }; // is a type-only export
			      ",
            None,
        ),
        (
            r"
			        import type Type from 'foo';
			
			        export { Type }; // is a type-only export
			        export default Type; // is a type-only export
			        export type { Type }; // is a type-only export
			      ",
            Some(serde_json::json!([{ "prefer": "no-type-imports" }])),
        ),
        (
            r"
			        import type { Type } from 'foo';
			
			        export { Type }; // is a type-only export
			        export default Type; // is a type-only export
			        export type { Type }; // is a type-only export
			      ",
            Some(serde_json::json!([{ "prefer": "no-type-imports" }])),
        ),
        (
            r"
			        import type * as Type from 'foo';
			
			        export { Type }; // is a type-only export
			        export default Type; // is a type-only export
			        export type { Type }; // is a type-only export
			      ",
            Some(serde_json::json!([{ "prefer": "no-type-imports" }])),
        ),
        (
            r"
			import Default, * as Rest from 'module';
			const a: Rest.A = '';
			      ",
            Some(serde_json::json!([{ "prefer": "type-imports" }])),
        ),
        (
            r"
			import Default, * as Rest from 'module';
			const a: Default = '';
			      ",
            Some(serde_json::json!([{ "prefer": "type-imports" }])),
        ),
        (
            r"
			import Default, * as Rest from 'module';
			const a: Default = '';
			const b: Rest.A = '';
			      ",
            Some(serde_json::json!([{ "prefer": "type-imports" }])),
        ),
        (
            r"
			import Default, /*comment*/ * as Rest from 'module';
			const a: Default = '';
			      ",
            Some(serde_json::json!([{ "prefer": "type-imports" }])),
        ),
        (
            r"
			        import Foo from 'foo';
			        @deco
			        class A {
			          constructor(foo: Foo) {}
			        }
			      ",
            None,
        ),
        (
            r"
			        import type Foo from 'foo';
			        @deco
			        class A {
			          constructor(foo: Foo) {}
			        }
			      ",
            None,
        ),
        (
            r"
			        import type { Foo } from 'foo';
			        @deco
			        class A {
			          constructor(foo: Foo) {}
			        }
			      ",
            None,
        ),
        (
            r"
			        import { V } from 'foo';
			        import type { Foo, Bar, T } from 'foo';
			        @deco
			        class A {
			          constructor(foo: Foo) {}
			          foo(@deco bar: Bar) {}
			        }
			      ",
            None,
        ),
        (
            r"
			        import type { Foo, T } from 'foo';
			        import { V } from 'foo';
			        @deco
			        class A {
			          constructor(foo: Foo) {}
			        }
			      ",
            None,
        ),
        (
            r"
			        import type * as Type from 'foo';
			        @deco
			        class A {
			          constructor(foo: Type.Foo) {}
			        }
			      ",
            None,
        ),
        (
            r"
			        import type Foo from 'foo';
			        @deco
			        class A {
			          constructor(foo: Foo) {}
			        }
			      ",
            None,
        ),
        (
            r"
			        import type { Foo } from 'foo';
			        @deco
			        class A {
			          constructor(foo: Foo) {}
			        }
			      ",
            None,
        ),
        (
            r"
			        import { V } from 'foo';
			        import type { Foo, Bar, T } from 'foo';
			        @deco
			        class A {
			          constructor(foo: Foo) {}
			          foo(@deco bar: Bar) {}
			        }
			      ",
            None,
        ),
        (
            r"
			        import type { Foo, T } from 'foo';
			        import { V } from 'foo';
			        @deco
			        class A {
			          constructor(foo: Foo) {}
			        }
			      ",
            None,
        ),
        (
            r"
			        import type * as Type from 'foo';
			        @deco
			        class A {
			          constructor(foo: Type.Foo) {}
			        }
			      ",
            None,
        ),
        (
            r"
			import { type A, B } from 'foo';
			type T = A;
			const b = B;
			      ",
            Some(serde_json::json!([{ "prefer": "no-type-imports" }])),
        ),
        (
            r"
			import { A, B, type C } from 'foo';
			type T = A | C;
			const b = B;
			      ",
            Some(serde_json::json!([{ "prefer": "type-imports" }])),
        ),
        (
            r"
			        import { A, B } from 'foo';
			        let foo: A;
			        let bar: B;
			      ",
            Some(
                serde_json::json!([{ "prefer": "type-imports", "fixStyle": "inline-type-imports" }]),
            ),
        ),
        (
            r"
			        import { A, B } from 'foo';
			
			        let foo: A;
			        B();
			      ",
            Some(
                serde_json::json!([{ "prefer": "type-imports", "fixStyle": "inline-type-imports" }]),
            ),
        ),
        (
            r"
			        import { A, B } from 'foo';
			        type T = A;
			        B();
			      ",
            Some(
                serde_json::json!([{ "prefer": "type-imports", "fixStyle": "inline-type-imports" }]),
            ),
        ),
        (
            r"
			        import { A } from 'foo';
			        import { B } from 'foo';
			        type T = A;
			        type U = B;
			      ",
            Some(
                serde_json::json!([{ "prefer": "type-imports", "fixStyle": "inline-type-imports" }]),
            ),
        ),
        (
            r"
			        import { A } from 'foo';
			        import B from 'foo';
			        type T = A;
			        type U = B;
			      ",
            Some(
                serde_json::json!([{ "prefer": "type-imports", "fixStyle": "inline-type-imports" }]),
            ),
        ),
        (
            r"
			import A, { B, C } from 'foo';
			type T = B;
			type U = C;
			A();
			      ",
            Some(
                serde_json::json!([{ "prefer": "type-imports", "fixStyle": "inline-type-imports" }]),
            ),
        ),
        (
            r"
			import A, { B, C } from 'foo';
			type T = B;
			type U = C;
			type V = A;
			      ",
            Some(
                serde_json::json!([{ "prefer": "type-imports", "fixStyle": "inline-type-imports" }]),
            ),
        ),
        (
            r"
			import A, { B, C as D } from 'foo';
			type T = B;
			type U = D;
			type V = A;
			      ",
            Some(
                serde_json::json!([{ "prefer": "type-imports", "fixStyle": "inline-type-imports" }]),
            ),
        ),
        (
            r"
			        import { /* comment */ A, B } from 'foo';
			        type T = A;
			      ",
            Some(
                serde_json::json!([{ "prefer": "type-imports", "fixStyle": "inline-type-imports" }]),
            ),
        ),
        (
            r"
			        import { B, /* comment */ A } from 'foo';
			        type T = A;
			      ",
            Some(
                serde_json::json!([{ "prefer": "type-imports", "fixStyle": "inline-type-imports" }]),
            ),
        ),
        (
            r"
			import { A, B, C } from 'foo';
			import type { D } from 'deez';
			
			const foo: A = B();
			let bar: C;
			let baz: D;
			      ",
            Some(
                serde_json::json!([{ "prefer": "type-imports", "fixStyle": "inline-type-imports" }]),
            ),
        ),
        (
            r"
			import { A, B, type C } from 'foo';
			import type { D } from 'deez';
			const foo: A = B();
			let bar: C;
			let baz: D;
			      ",
            Some(
                serde_json::json!([{ "prefer": "type-imports", "fixStyle": "inline-type-imports" }]),
            ),
        ),
    ];

    Tester::new(ConsistentTypeImports::NAME, vec![], fail).test_and_snapshot();
}
