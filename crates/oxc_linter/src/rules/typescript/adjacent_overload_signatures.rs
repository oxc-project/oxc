use oxc_ast::{
    ast::{
        ClassElement, Declaration, ExportDefaultDeclarationKind, Expression, FunctionType,
        ModuleDeclaration, PropertyKey, Statement, TSSignature,
    },
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{Atom, GetSpan, Span};

use crate::{ast_util::get_name_from_property_key, context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error(
    "typescript-eslint(adjacent-overload-signatures): All {0:?} signatures should be adjacent."
)]
#[diagnostic(severity(warning))]
struct AdjacentOverloadSignaturesDiagnostic(Atom, #[label] pub Option<Span>, #[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct AdjacentOverloadSignatures;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Require that function overload signatures be consecutive.
    ///
    /// ### Why is this bad?
    /// Function overload signatures represent multiple ways
    /// a function can be called, potentially with different return types.
    /// It's typical for an interface or type alias describing a function to place all overload signatures next to each other.
    /// If Signatures placed elsewhere in the type are easier to be missed by future developers reading the code.
    ///
    /// ### Example
    /// ```typescript
    /// declare namespace Foo {
    ///   export function foo(s: string): void;
    ///   export function foo(n: number): void;
    ///   export function bar(): void;
    ///   export function foo(sn: string | number): void;
    /// }
    ///
    /// type Foo = {
    ///   foo(s: string): void;
    ///   foo(n: number): void;
    ///   bar(): void;
    ///   foo(sn: string | number): void;
    /// };
    ///
    /// interface Foo {
    ///   foo(s: string): void;
    ///   foo(n: number): void;
    ///   bar(): void;
    ///   foo(sn: string | number): void;
    /// }
    ///
    /// class Foo {
    ///   foo(s: string): void;
    ///   foo(n: number): void;
    ///   bar(): void {}
    ///   foo(sn: string | number): void {}
    /// }
    ///
    /// export function foo(s: string): void;
    /// export function foo(n: number): void;
    /// export function bar(): void;
    /// export function foo(sn: string | number): void;
    /// ```
    AdjacentOverloadSignatures,
    correctness
);

#[derive(PartialEq, Debug)]
enum MethodKind {
    Private,
    Quoted,
    Normal,
    Expression,
}

fn get_kind_from_key(key: &PropertyKey) -> MethodKind {
    match key {
        PropertyKey::Identifier(_) => MethodKind::Normal,
        PropertyKey::PrivateIdentifier(_) => MethodKind::Private,
        PropertyKey::Expression(expr) => match expr {
            Expression::StringLiteral(_) => MethodKind::Normal,
            Expression::NumberLiteral(_)
            | Expression::BigintLiteral(_)
            | Expression::TemplateLiteral(_)
            | Expression::RegExpLiteral(_)
            | Expression::NullLiteral(_) => MethodKind::Quoted,
            _ => MethodKind::Expression,
        },
    }
}

#[derive(Debug)]
struct Method {
    name: Atom,
    r#static: bool,
    call_signature: bool,
    kind: MethodKind,
    span: Span,
}

impl Method {
    fn is_same_method(&self, other: Option<&Self>) -> bool {
        other.map_or(false, |other| {
            self.name == other.name
                && self.r#static == other.r#static
                && self.call_signature == other.call_signature
                && self.kind == other.kind
        })
    }
}

trait GetMethod {
    fn get_method(&self) -> Option<Method>;
}

impl GetMethod for ClassElement<'_> {
    fn get_method(&self) -> Option<Method> {
        match self {
            ClassElement::MethodDefinition(def) => {
                get_name_from_property_key(&def.key).map(|name| Method {
                    name,
                    r#static: def.r#static,
                    call_signature: false,
                    kind: get_kind_from_key(&def.key),
                    span: Span::new(def.span.start, def.key.span().end),
                })
            }
            _ => None,
        }
    }
}

impl GetMethod for TSSignature<'_> {
    fn get_method(&self) -> Option<Method> {
        match self {
            TSSignature::TSMethodSignature(sig) => {
                get_name_from_property_key(&sig.key).map(|name| Method {
                    name,
                    r#static: false,
                    call_signature: false,
                    kind: get_kind_from_key(&sig.key),
                    span: sig.key.span(),
                })
            }
            TSSignature::TSCallSignatureDeclaration(sig) => Some(Method {
                name: Atom::from("call"),
                r#static: false,
                call_signature: true,
                kind: MethodKind::Normal,
                span: sig.span,
            }),
            TSSignature::TSConstructSignatureDeclaration(decl) => Some(Method {
                name: Atom::from("new"),
                r#static: false,
                call_signature: false,
                kind: MethodKind::Normal,
                span: Span::new(decl.span.start, decl.span.start + 3),
            }),
            _ => None,
        }
    }
}

impl GetMethod for ModuleDeclaration<'_> {
    fn get_method(&self) -> Option<Method> {
        match self {
            ModuleDeclaration::ExportDefaultDeclaration(default_decl) => {
                let decl_kind = &default_decl.declaration;

                match decl_kind {
                    ExportDefaultDeclarationKind::FunctionDeclaration(func_decl) => {
                        if matches!(
                            func_decl.r#type,
                            FunctionType::FunctionDeclaration | FunctionType::TSDeclareFunction
                        ) {
                            func_decl.id.as_ref().map(|id| Method {
                                name: id.name.clone(),
                                r#static: false,
                                call_signature: false,
                                kind: MethodKind::Normal,
                                span: id.span,
                            })
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            }
            ModuleDeclaration::ExportNamedDeclaration(named_decl) => {
                if let Some(Declaration::FunctionDeclaration(func_decl)) = &named_decl.declaration {
                    return func_decl.id.as_ref().map(|id| Method {
                        name: id.name.clone(),
                        r#static: false,
                        call_signature: false,
                        kind: MethodKind::Normal,
                        span: id.span,
                    });
                }
                None
            }
            _ => None,
        }
    }
}

impl GetMethod for Declaration<'_> {
    fn get_method(&self) -> Option<Method> {
        match self {
            Declaration::FunctionDeclaration(func_decl) => {
                if matches!(
                    func_decl.r#type,
                    FunctionType::FunctionDeclaration | FunctionType::TSDeclareFunction
                ) {
                    func_decl.id.as_ref().map(|id| Method {
                        name: id.name.clone(),
                        r#static: false,
                        call_signature: false,
                        kind: MethodKind::Normal,
                        span: id.span,
                    })
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl GetMethod for Statement<'_> {
    fn get_method(&self) -> Option<Method> {
        match self {
            Statement::ModuleDeclaration(decl) => decl.get_method(),
            Statement::Declaration(decl) => decl.get_method(),
            _ => None,
        }
    }
}

fn check_and_report(methods: &Vec<Option<Method>>, ctx: &LintContext<'_>) {
    let mut last_method: Option<&Method> = None;
    let mut seen_methods: Vec<&Method> = Vec::new();

    for method in methods {
        if let Some(method) = method {
            let index = seen_methods.iter().position(|m| method.is_same_method(Some(m)));

            if index.is_some() && !method.is_same_method(last_method) {
                let name = if method.r#static {
                    Atom::from(format!("static {0}", method.name))
                } else {
                    method.name.clone()
                };

                let last_same_method =
                    seen_methods.iter().rev().find(|m| m.is_same_method(Some(method)));

                ctx.diagnostic(AdjacentOverloadSignaturesDiagnostic(
                    name,
                    last_same_method.map(|m| m.span),
                    method.span,
                ));
            } else {
                seen_methods.push(method);
            }
            last_method = Some(method);
        } else {
            last_method = None;
        }
    }
}

impl Rule for AdjacentOverloadSignatures {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::Class(class) => {
                let members = &class.body.body;
                let methods = members.iter().map(GetMethod::get_method).collect();
                check_and_report(&methods, ctx);
            }
            AstKind::TSTypeLiteral(literal) => {
                let methods = literal.members.iter().map(GetMethod::get_method).collect();
                check_and_report(&methods, ctx);
            }
            AstKind::Program(program) => {
                let methods = program.body.iter().map(GetMethod::get_method).collect();

                check_and_report(&methods, ctx);
            }
            AstKind::TSModuleBlock(block) => {
                let methods = block.body.iter().map(GetMethod::get_method).collect();

                check_and_report(&methods, ctx);
            }
            AstKind::TSInterfaceDeclaration(decl) => {
                let methods = decl.body.body.iter().map(GetMethod::get_method).collect();

                check_and_report(&methods, ctx);
            }
            AstKind::BlockStatement(stmt) => {
                let methods = stmt.body.iter().map(GetMethod::get_method).collect();

                check_and_report(&methods, ctx);
            }
            AstKind::FunctionBody(body) => {
                let methods = body.statements.iter().map(GetMethod::get_method).collect();

                check_and_report(&methods, ctx);
            }
            _ => {}
        }
    }
}

#[allow(clippy::too_many_lines)]
#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"function error(a: string);
      function error(b: number);
      function error(ab: string | number) {}
      export { error };"#,
        r#"import { connect } from 'react-redux';
      export interface ErrorMessageModel {
        message: string;
      }
      function mapStateToProps() {}
      function mapDispatchToProps() {}
      export default connect(mapStateToProps, mapDispatchToProps)(ErrorMessage);"#,
        r#"export const foo = 'a',
      bar = 'b';
      export interface Foo {}
      export class Foo {}"#,
        r#"export interface Foo {}
      export const foo = 'a',
        bar = 'b';
      export class Foo {}"#,
        r#"const foo = 'a',
      bar = 'b';
      interface Foo {}
      class Foo {}"#,
        r#"interface Foo {}
      const foo = 'a',
        bar = 'b';
      class Foo {}"#,
        r#"export class Foo {}
      export class Bar {}
      export type FooBar = Foo | Bar;"#,
        r#"export interface Foo {}
      export class Foo {}
      export class Bar {}
      export type FooBar = Foo | Bar;"#,
        r#"export function foo(s: string);
      export function foo(n: number);
      export function foo(sn: string | number) {}
      export function bar(): void {}
      export function baz(): void {}"#,
        r#"function foo(s: string);
      function foo(n: number);
      function foo(sn: string | number) {}
      function bar(): void {}
      function baz(): void {}"#,
        r#"declare function foo(s: string);
      declare function foo(n: number);
      declare function foo(sn: string | number);
      declare function bar(): void;
      declare function baz(): void;"#,
        r#"declare module 'Foo' {
        export function foo(s: string): void;
        export function foo(n: number): void;
        export function foo(sn: string | number): void;
        export function bar(): void;
        export function baz(): void;
      }"#,
        r#"declare namespace Foo {
        export function foo(s: string): void;
        export function foo(n: number): void;
        export function foo(sn: string | number): void;
        export function bar(): void;
        export function baz(): void;
      }"#,
        r#"type Foo = {
        foo(s: string): void;
        foo(n: number): void;
        foo(sn: string | number): void;
        bar(): void;
        baz(): void;
      };"#,
        r#"type Foo = {
        foo(s: string): void;
        ['foo'](n: number): void;
        foo(sn: string | number): void;
        bar(): void;
        baz(): void;
      };"#,
        r#"interface Foo {
        (s: string): void;
        (n: number): void;
        (sn: string | number): void;
        foo(n: number): void;
        bar(): void;
        baz(): void;
      }"#,
        r#"interface Foo {
        (s: string): void;
        (n: number): void;
        (sn: string | number): void;
        foo(n: number): void;
        bar(): void;
        baz(): void;
        call(): void;
      }"#,
        r#"interface Foo {
        foo(s: string): void;
        foo(n: number): void;
        foo(sn: string | number): void;
        bar(): void;
        baz(): void;
      }"#,
        r#"interface Foo {
        foo(s: string): void;
        ['foo'](n: number): void;
        foo(sn: string | number): void;
        bar(): void;
        baz(): void;
      }"#,
        r#"interface Foo {
        foo(): void;
        bar: {
          baz(s: string): void;
          baz(n: number): void;
          baz(sn: string | number): void;
        };
      }"#,
        r#"interface Foo {
        new (s: string);
        new (n: number);
        new (sn: string | number);
        foo(): void;
      }"#,
        r#"class Foo {
        constructor(s: string);
        constructor(n: number);
        constructor(sn: string | number) {}
        bar(): void {}
        baz(): void {}
      }"#,
        r#"class Foo {
        foo(s: string): void;
        foo(n: number): void;
        foo(sn: string | number): void {}
        bar(): void {}
        baz(): void {}
      }"#,
        r#"class Foo {
        foo(s: string): void;
        ['foo'](n: number): void;
        foo(sn: string | number): void {}
        bar(): void {}
        baz(): void {}
      }"#,
        r#"class Foo {
        name: string;
        foo(s: string): void;
        foo(n: number): void;
        foo(sn: string | number): void {}
        bar(): void {}
        baz(): void {}
      }"#,
        r#"class Foo {
        name: string;
        static foo(s: string): void;
        static foo(n: number): void;
        static foo(sn: string | number): void {}
        bar(): void {}
        baz(): void {}
      }"#,
        r#"class Test {
        static test() {}
        untest() {}
        test() {}
      }"#,
        "export default function <T>(foo: T) {}",
        "export default function named<T>(foo: T) {}",
        r#"interface Foo {
        [Symbol.toStringTag](): void;
        [Symbol.iterator](): void;
      }"#,
        r#"class Test {
        #private(): void;
        #private(arg: number): void {}
      
        bar() {}
      
        '#private'(): void;
        '#private'(arg: number): void {}
      }"#,
        r#"function wrap() {
        function foo(s: string);
        function foo(n: number);
        function foo(sn: string | number) {}
      }"#,
        r#"if (true) {
        function foo(s: string);
        function foo(n: number);
        function foo(sn: string | number) {}
      }"#,
    ];

    let fail = vec![
        r#"function wrap() {
        function foo(s: string);
        function foo(n: number);
        type bar = number;
        function foo(sn: string | number) {}
      }"#,
        r#"if (true) {
        function foo(s: string);
        function foo(n: number);
        let a = 1;
        function foo(sn: string | number) {}
        foo(a);
      }"#,
        r#"export function foo(s: string);
      export function foo(n: number);
      export function bar(): void {}
      export function baz(): void {}
      export function foo(sn: string | number) {}"#,
        r#"export function foo(s: string);
      export function foo(n: number);
      export type bar = number;
      export type baz = number | string;
      export function foo(sn: string | number) {}"#,
        r#"function foo(s: string);
      function foo(n: number);
      function bar(): void {}
      function baz(): void {}
      function foo(sn: string | number) {}"#,
        r#"function foo(s: string);
      function foo(n: number);
      type bar = number;
      type baz = number | string;
      function foo(sn: string | number) {}"#,
        // commented because it would raise a syntax error,
        // which is beyond the scope of linter's responsibilities.
        // r#"function foo(s: string) {}
        // function foo(n: number) {}
        // const a = '';
        // const b = '';
        // function foo(sn: string | number) {}"#,
        // r#"function foo(s: string) {}
        // function foo(n: number) {}
        // class Bar {}
        // function foo(sn: string | number) {}"#,
        // r#"function foo(s: string) {}
        // function foo(n: number) {}
        // function foo(sn: string | number) {}
        // class Bar {
        //   foo(s: string);
        //   foo(n: number);
        //   name: string;
        //   foo(sn: string | number) {}
        // }"#
        r#"declare function foo(s: string);
      declare function foo(n: number);
      declare function bar(): void;
      declare function baz(): void;
      declare function foo(sn: string | number);"#,
        r#"declare function foo(s: string);
      declare function foo(n: number);
      const a = '';
      const b = '';
      declare function foo(sn: string | number);"#,
        r#"declare module 'Foo' {
        export function foo(s: string): void;
        export function foo(n: number): void;
        export function bar(): void;
        export function baz(): void;
        export function foo(sn: string | number): void;
      }"#,
        r#"declare module 'Foo' {
        export function foo(s: string): void;
        export function foo(n: number): void;
        export function foo(sn: string | number): void;
        function baz(s: string): void;
        export function bar(): void;
        function baz(n: number): void;
        function baz(sn: string | number): void;
      }"#,
        r#"declare namespace Foo {
        export function foo(s: string): void;
        export function foo(n: number): void;
        export function bar(): void;
        export function baz(): void;
        export function foo(sn: string | number): void;
      }"#,
        r#"declare namespace Foo {
        export function foo(s: string): void;
        export function foo(n: number): void;
        export function foo(sn: string | number): void;
        function baz(s: string): void;
        export function bar(): void;
        function baz(n: number): void;
        function baz(sn: string | number): void;
      }"#,
        r#"type Foo = {
        foo(s: string): void;
        foo(n: number): void;
        bar(): void;
        baz(): void;
        foo(sn: string | number): void;
      };"#,
        r#"type Foo = {
        foo(s: string): void;
        ['foo'](n: number): void;
        bar(): void;
        baz(): void;
        foo(sn: string | number): void;
      };"#,
        r#"type Foo = {
        foo(s: string): void;
        name: string;
        foo(n: number): void;
        foo(sn: string | number): void;
        bar(): void;
        baz(): void;
      };"#,
        r#"interface Foo {
        (s: string): void;
        foo(n: number): void;
        (n: number): void;
        (sn: string | number): void;
        bar(): void;
        baz(): void;
        call(): void;
      }"#,
        r#"interface Foo {
        foo(s: string): void;
        foo(n: number): void;
        bar(): void;
        baz(): void;
        foo(sn: string | number): void;
      }"#,
        r#"interface Foo {
        foo(s: string): void;
        ['foo'](n: number): void;
        bar(): void;
        baz(): void;
        foo(sn: string | number): void;
      }"#,
        r#"interface Foo {
        foo(s: string): void;
        'foo'(n: number): void;
        bar(): void;
        baz(): void;
        foo(sn: string | number): void;
      }"#,
        r#"interface Foo {
        foo(s: string): void;
        name: string;
        foo(n: number): void;
        foo(sn: string | number): void;
        bar(): void;
        baz(): void;
      }"#,
        r#"interface Foo {
        foo(): void;
        bar: {
          baz(s: string): void;
          baz(n: number): void;
          foo(): void;
          baz(sn: string | number): void;
        };
      }"#,
        r#"interface Foo {
        new (s: string);
        new (n: number);
        foo(): void;
        bar(): void;
        new (sn: string | number);
      }"#,
        r#"interface Foo {
        new (s: string);
        foo(): void;
        new (n: number);
        bar(): void;
        new (sn: string | number);
      }"#,
        r#"class Foo {
        constructor(s: string);
        constructor(n: number);
        bar(): void {}
        baz(): void {}
        constructor(sn: string | number) {}
      }"#,
        r#"class Foo {
        foo(s: string): void;
        foo(n: number): void;
        bar(): void {}
        baz(): void {}
        foo(sn: string | number): void {}
      }"#,
        r#"class Foo {
        foo(s: string): void;
        ['foo'](n: number): void;
        bar(): void {}
        baz(): void {}
        foo(sn: string | number): void {}
      }"#,
        r#"class Foo {
        // prettier-ignore
        "foo"(s: string): void;
        foo(n: number): void;
        bar(): void {}
        baz(): void {}
        foo(sn: string | number): void {}
      }"#,
        r#"class Foo {
        constructor(s: string);
        name: string;
        constructor(n: number);
        constructor(sn: string | number) {}
        bar(): void {}
        baz(): void {}
      }"#,
        r#"class Foo {
        foo(s: string): void;
        name: string;
        foo(n: number): void;
        foo(sn: string | number): void {}
        bar(): void {}
        baz(): void {}
      }"#,
        r#"class Foo {
        static foo(s: string): void;
        name: string;
        static foo(n: number): void;
        static foo(sn: string | number): void {}
        bar(): void {}
        baz(): void {}
      }"#,
        r#"class Test {
        #private(): void;
        '#private'(): void;
        #private(arg: number): void {}
        '#private'(arg: number): void {}
      }"#,
    ];

    Tester::new_without_config(AdjacentOverloadSignatures::NAME, pass, fail).test_and_snapshot();
}
