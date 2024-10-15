use oxc_ast::{
    ast::{
        Argument, BindingPattern, BindingPatternKind, BindingRestElement, CallExpression,
        Expression, FormalParameters, FunctionBody, MethodDefinition, Statement, TSAccessibility,
    },
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{context::LintContext, rule::Rule, AstNode};

/// ```js
/// class A { constructor(){} }
/// ```
fn no_empty_constructor(constructor_span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Empty constructors are unnecessary")
        .with_label(constructor_span)
        .with_help("Remove the constructor or add code to it.")
}

/// ```js
/// class A { }
/// class B extends A {
///     constructor() {
///         super();
///     }
/// }
/// ```
fn no_redundant_super_call(constructor_span: Span, super_span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Redundant super call in constructor")
        .with_labels([
            constructor_span.primary_label("This constructor is unnecessary,"),
            super_span.label("because it only passes arguments through to the superclass"),
        ])
        .with_help("Subclasses automatically use the constructor of their superclass, making this redundant.\nRemove this constructor or add code to it.")
}

#[derive(Debug, Default, Clone)]
pub struct NoUselessConstructor;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow unnecessary constructors
    ///
    /// This rule flags class constructors that can be safely removed without
    /// changing how the class works.
    ///
    /// ES2015 provides a default class constructor if one is not specified. As
    /// such, it is unnecessary to provide an empty constructor or one that
    /// simply delegates into its parent class, as in the following examples:
    ///
    ///
    /// ### Example
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// class A {
    ///     constructor () {
    ///     }
    /// }
    ///
    /// class B extends A {
    ///     constructor (...args) {
    ///       super(...args);
    ///     }
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// class A { }
    ///
    /// class B {
    ///     constructor () {
    ///         doSomething();
    ///     }
    /// }
    ///
    /// class C extends A {
    ///     constructor() {
    ///         super('foo');
    ///     }
    /// }
    ///
    /// class D extends A {
    ///     constructor() {
    ///         super();
    ///         doSomething();
    ///     }
    /// }
    ///```
    NoUselessConstructor,
    suspicious,
    fix
);

impl Rule for NoUselessConstructor {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::MethodDefinition(constructor) = node.kind() else {
            return;
        };
        if !constructor.kind.is_constructor() {
            return;
        }
        let Some(body) = &constructor.value.body else {
            return;
        };
        // allow `private private constructor() {}`. However, `public private
        // constructor() {}` is the same as `constructor() {}` and so is not allowed.
        if constructor.accessibility.is_some_and(|access| {
            matches!(access, TSAccessibility::Private | TSAccessibility::Protected)
        }) {
            return;
        }

        let class = ctx
            .nodes()
            .iter_parents(node.id())
            .skip(1)
            .find(|parent| matches!(parent.kind(), AstKind::Class(_)));
        debug_assert!(class.is_some(), "Found a constructor outside of a class definition");
        let Some(class_node) = class else {
            return;
        };
        let AstKind::Class(class) = class_node.kind() else { unreachable!() };
        if class.declare {
            return;
        }

        if class.super_class.is_none() {
            lint_empty_constructor(ctx, constructor, body);
        } else {
            lint_redundant_super_call(ctx, constructor, body);
        }
    }
}

// Check for an empty constructor in a class without a superclass.
fn lint_empty_constructor<'a>(
    ctx: &LintContext<'a>,
    constructor: &MethodDefinition<'a>,
    body: &FunctionBody<'a>,
) {
    if !body.statements.is_empty() {
        return;
    }

    // allow constructors with access modifiers since they actually declare
    // class members
    if constructor
        .value
        .params
        .items
        .iter()
        .any(|param| param.accessibility.is_some() || param.readonly)
    {
        return;
    }

    ctx.diagnostic_with_fix(no_empty_constructor(constructor.span), |fixer| {
        fixer.delete_range(constructor.span)
    });
}

fn lint_redundant_super_call<'a>(
    ctx: &LintContext<'a>,
    constructor: &MethodDefinition<'a>,
    body: &FunctionBody<'a>,
) {
    let Some(super_call) = is_single_super_call(body) else {
        return;
    };

    let params = &*constructor.value.params;
    let super_args = &super_call.arguments;

    if is_only_simple_params(params)
        && !is_overriding(params)
        && (is_spread_arguments(super_args) || is_passing_through(params, super_args))
    {
        ctx.diagnostic_with_fix(
            no_redundant_super_call(constructor.key.span(), super_call.span()),
            |fixer| fixer.delete_range(constructor.span),
        );
    }
}

fn is_overriding(params: &FormalParameters) -> bool {
    params.items.iter().any(|param| param.r#override)
}

/// Check if a function body only contains a single super call. Ignores directives.
///
/// Returns the call expression if the body contains a single super call, otherwise [`None`].
fn is_single_super_call<'f, 'a: 'f>(body: &'f FunctionBody<'a>) -> Option<&'f CallExpression<'a>> {
    if body.statements.len() != 1 {
        return None;
    }
    let Statement::ExpressionStatement(expr) = &body.statements[0] else { return None };
    let Expression::CallExpression(call) = &expr.expression else { return None };

    matches!(call.callee, Expression::Super(_)).then(|| call.as_ref())
}

/// Returns `false` if any parameter is an array/object unpacking binding or an
/// assignment pattern.
fn is_only_simple_params(params: &FormalParameters) -> bool {
    params.iter_bindings().all(|param| param.kind.is_binding_identifier())
}

fn is_spread_arguments(super_args: &[Argument<'_>]) -> bool {
    super_args.len() == 1 && super_args[0].is_spread()
}

fn is_passing_through<'a>(
    constructor_params: &FormalParameters<'a>,
    super_args: &[Argument<'a>],
) -> bool {
    if constructor_params.parameters_count() != super_args.len() {
        return false;
    }
    if let Some(rest) = &constructor_params.rest {
        let all_but_last = super_args.iter().take(super_args.len() - 1);
        let Some(last_arg) = super_args.iter().next_back() else { return false };
        constructor_params
            .items
            .iter()
            .zip(all_but_last)
            .all(|(param, arg)| is_matching_identifier_pair(&param.pattern, arg))
            && is_matching_rest_spread_pair(rest, last_arg)
    } else {
        constructor_params
            .iter_bindings()
            .zip(super_args)
            .all(|(param, arg)| is_matching_identifier_pair(param, arg))
    }
}

fn is_matching_identifier_pair<'a>(param: &BindingPattern<'a>, arg: &Argument<'a>) -> bool {
    match (&param.kind, arg) {
        (BindingPatternKind::BindingIdentifier(param), Argument::Identifier(arg)) => {
            param.name == arg.name
        }
        _ => false,
    }
}
fn is_matching_rest_spread_pair<'a>(rest: &BindingRestElement<'a>, arg: &Argument<'a>) -> bool {
    match (&rest.argument.kind, arg) {
        (BindingPatternKind::BindingIdentifier(param), Argument::SpreadElement(spread)) => {
            matches!(&spread.argument, Expression::Identifier(ident) if param.name == ident.name)
        }
        _ => false,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "class A { }",
        "class A { constructor(){ doSomething(); } }",
        "class A extends B { constructor(){} }",
        "class A extends B { constructor(){ super('foo'); } }",
        "class A extends B { constructor(foo, bar){ super(foo, bar, 1); } }",
        "class A extends B { constructor(){ super(); doSomething(); } }",
        "class A extends B { constructor(...args){ super(...args); doSomething(); } }",
        "class A { dummyMethod(){ doSomething(); } }",
        "class A extends B.C { constructor() { super(foo); } }",
        "class A extends B.C { constructor([a, b, c]) { super(...arguments); } }",
        "class A extends B.C { constructor(a = f()) { super(...arguments); } }",
        "class A extends B { constructor(a, b, c) { super(a, b); } }",
        "class A extends B { constructor(foo, bar){ super(foo); } }",
        "class A extends B { constructor(test) { super(); } }",
        "class A extends B { constructor() { foo; } }",
        "class A extends B { constructor(foo, bar) { super(bar); } }",
        // ts
        "declare class A { constructor(options: any); }", // {                "parser": require("../../fixtures/parsers/typescript-parsers/declare-class")            }
        "class A { private constructor() {} }",
        "class A { protected constructor() {} }",
        "class A { constructor(private x: number) {} }",
        "class A { constructor(public x: number) {} }",
        "class A { constructor(protected x: number) {} }",
        "class A { constructor(readonly x: number) {} }",
        "class A { constructor(private readonly x: number) {} }",
        "class A extends B { constructor(override x: number) { super(x); } }",
        "
        class A {
            protected foo: number | undefined;
            constructor(foo?: number) {
                this.foo = foo;
            }
        }
        class B extends A {
            protected foo: number;
            constructor(foo: number = 0) {
                super(foo);
            }
        }
        ",
        "
        class A {
            protected foo: number | undefined;
            constructor(foo?: number) {
                this.foo = foo;
            }
        }
        class B extends A {
            constructor(foo?: number) {
                super(foo ?? 0);
            }
        }
        ",
        // TODO: type aware linting :(
        // "
        // class A {
        //     protected foo: string;
        //     constructor(foo: string) {
        //         this.foo = foo;
        //     }
        // }
        // class B extends A {
        //     constructor(foo: 'a' | 'b') {
        //         super(foo);
        //     }
        // }
        // ",
    ];

    let fail = vec![
        "class A { constructor(){} }",
        "class A { 'constructor'(){} }",
        "class A extends B { constructor() { super(); } }",
        "class A extends B {
    constructor() {
        super();
    }
}",
        "class A extends B { constructor(foo){ super(foo); } }",
        "class A extends B { constructor(foo, bar){ super(foo, bar); } }",
        "class A extends B { constructor(...args){ super(...args); } }",
        "class A extends B.C { constructor() { super(...arguments); } }",
        "class A extends B { constructor(a, b, ...c) { super(...arguments); } }",
        "class A extends B { constructor(a, b, ...c) { super(a, b, ...c); } }",
        // ts
        "class A { public constructor(){} }",
    ];

    let fix = vec![
        ("class A { constructor(){} }", "class A {  }"),
        (
            r"
class A extends B { constructor() { super(); } foo() { bar(); } }",
            r"
class A extends B {  foo() { bar(); } }",
        ),
    ];

    Tester::new(NoUselessConstructor::NAME, pass, fail).expect_fix(fix).test_and_snapshot();
}
