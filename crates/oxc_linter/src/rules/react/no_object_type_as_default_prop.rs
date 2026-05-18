use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    ast_util::outermost_paren_parent,
    context::{ContextHost, LintContext},
    rule::Rule,
    utils::{is_hoc_call, is_react_component_name},
};

fn no_object_type_as_default_prop_diagnostic(kind: ForbiddenKind, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Do not use {} as default prop value. Use a stable reference instead.",
        kind.label()
    ))
    .with_help(
        "Default values are re-created on every render and break referential equality, causing unnecessary re-renders. Move the value out of the component or memoize it.",
    )
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoObjectTypeAsDefaultProp;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows using object, array, function, class, regex, JSX, or `new`-constructed
    /// values as default values for destructured React component props.
    ///
    /// ### Why is this bad?
    ///
    /// Default values of destructured parameters are evaluated on every render. When the
    /// default is an object literal, array literal, function expression, class expression,
    /// regular expression, `new` expression, or JSX element, a new reference is created on
    /// every render. Passing that fresh reference to child components or hook dependency
    /// arrays defeats memoization and causes unnecessary re-renders.
    ///
    /// Note: you do not need to enable this rule when using React Compiler, since React
    /// Compiler memoizes default values automatically.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// function Foo({ items = [] }) {
    ///   return <List items={items} />;
    /// }
    ///
    /// const Bar = ({ config = {} }) => <div data-config={config} />;
    ///
    /// function Baz({ onClick = () => {} }) {
    ///   return <button onClick={onClick} />;
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// const DEFAULT_ITEMS = [];
    /// function Foo({ items = DEFAULT_ITEMS }) {
    ///   return <List items={items} />;
    /// }
    ///
    /// const Bar = ({ name = "world" }) => <div>{name}</div>;
    ///
    /// function Baz({ onClick }) {
    ///   return <button onClick={onClick} />;
    /// }
    /// ```
    NoObjectTypeAsDefaultProp,
    react,
    perf,
    version = "next",
);

impl Rule for NoObjectTypeAsDefaultProp {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::AssignmentPattern(assign_pat) = node.kind() else { return };

        let right = assign_pat.right.get_inner_expression();
        let Some(kind) = forbidden_default_kind(right) else { return };

        if !is_in_component_first_param(node, ctx) {
            return;
        }

        ctx.diagnostic(no_object_type_as_default_prop_diagnostic(kind, right.span()));
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_jsx()
    }
}

#[derive(Debug, Clone, Copy)]
enum ForbiddenKind {
    Object,
    Array,
    Function,
    Class,
    New,
    Regex,
    Jsx,
}

impl ForbiddenKind {
    fn label(self) -> &'static str {
        match self {
            Self::Object => "an object literal",
            Self::Array => "an array literal",
            Self::Function => "a function expression",
            Self::Class => "a class expression",
            Self::New => "a `new` expression",
            Self::Regex => "a regular expression literal",
            Self::Jsx => "a JSX element",
        }
    }
}

fn forbidden_default_kind(expr: &Expression) -> Option<ForbiddenKind> {
    match expr {
        Expression::ObjectExpression(_) => Some(ForbiddenKind::Object),
        Expression::ArrayExpression(_) => Some(ForbiddenKind::Array),
        Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_) => {
            Some(ForbiddenKind::Function)
        }
        Expression::ClassExpression(_) => Some(ForbiddenKind::Class),
        Expression::NewExpression(_) => Some(ForbiddenKind::New),
        Expression::RegExpLiteral(_) => Some(ForbiddenKind::Regex),
        Expression::JSXElement(_) | Expression::JSXFragment(_) => Some(ForbiddenKind::Jsx),
        _ => None,
    }
}

/// Returns true when `node` is an `AssignmentPattern` nested inside the destructuring
/// pattern of the first parameter of a function component.
fn is_in_component_first_param<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
    for ancestor in ctx.nodes().ancestors(node.id()).skip(1) {
        match ancestor.kind() {
            AstKind::BindingProperty(_)
            | AstKind::ObjectPattern(_)
            | AstKind::ArrayPattern(_)
            | AstKind::AssignmentPattern(_)
            | AstKind::BindingRestElement(_) => {}
            AstKind::FormalParameter(_) => return is_first_param_of_component(ancestor, ctx),
            _ => return false,
        }
    }
    false
}

fn is_first_param_of_component<'a>(formal_param_node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
    let AstKind::FormalParameter(this_param) = formal_param_node.kind() else { return false };

    let params_node = ctx.nodes().parent_node(formal_param_node.id());
    let AstKind::FormalParameters(params) = params_node.kind() else { return false };

    if !params.items.first().is_some_and(|first| std::ptr::eq(first, this_param)) {
        return false;
    }

    let func_node = ctx.nodes().parent_node(params_node.id());
    is_function_component(func_node, ctx)
}

fn is_function_component<'a>(func_node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
    match func_node.kind() {
        AstKind::Function(func) => {
            if let Some(id) = &func.id
                && is_react_component_name(&id.name)
            {
                return true;
            }
            is_in_component_context(func_node, ctx)
        }
        AstKind::ArrowFunctionExpression(_) => is_in_component_context(func_node, ctx),
        _ => false,
    }
}

fn is_in_component_context<'a>(func_node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
    let Some(parent) = outermost_paren_parent(func_node, ctx.semantic()) else { return false };

    match parent.kind() {
        AstKind::VariableDeclarator(decl) => {
            decl.id.get_identifier_name().is_some_and(|name| is_react_component_name(&name))
        }
        AstKind::CallExpression(call) => {
            call.callee_name().is_some_and(|name| is_hoc_call(name, ctx))
        }
        _ => false,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // Primitive defaults are stable.
        "function Foo({ name = 'hello' }) { return <div>{name}</div>; }",
        "function Foo({ count = 0 }) { return <div>{count}</div>; }",
        "function Foo({ enabled = true }) { return <div /> }",
        "function Foo({ data = null }) { return <div /> }",
        "function Foo({ data = undefined }) { return <div /> }",
        // Identifier references (stable when declared outside the component).
        "const EMPTY = []; function Foo({ items = EMPTY }) { return <div /> }",
        // Member access is fine (presumed stable).
        "function Foo({ items = obj.value }) { return <div /> }",
        // No destructuring default.
        "function Foo(props) { return <div>{props.children}</div>; }",
        "function Foo({ items }) { return <div /> }",
        // Top-level (non-component) destructuring – should not trigger.
        "const { items = [] } = obj;",
        "function notAComponent({ items = [] }) { return items; }",
        "function lower_case({ items = [] }) { return items; }",
        // Inner destructure in a variable declaration within a component.
        "function Foo(props) { const { items = [] } = props; return <div /> }",
        // Not the first parameter.
        "function Foo(first, { items = [] }) { return <div /> }",
        // forwardRef – primitive default is fine.
        "const Foo = forwardRef(({ name = 'a' }, ref) => <div ref={ref}>{name}</div>);",
        // memo with primitive default.
        "const Foo = memo(({ name = 'a' }) => <div>{name}</div>);",
        // Non-React function (lowercase name) destructuring with object default.
        "const foo = ({ items = [] }) => items;",
        // Template literal without expressions.
        "function Foo({ greeting = `hello` }) { return <div>{greeting}</div>; }",
    ];

    let fail = vec![
        // Plain object literal default.
        "function Foo({ items = {} }) { return <div /> }",
        // Plain array literal default.
        "function Foo({ items = [] }) { return <div /> }",
        // Arrow function default.
        "function Foo({ onClick = () => {} }) { return <div /> }",
        // Function expression default.
        "function Foo({ onClick = function () {} }) { return <div /> }",
        // Class expression default.
        "function Foo({ Klass = class {} }) { return <div /> }",
        // `new` expression default.
        "function Foo({ items = new Set() }) { return <div /> }",
        "function Foo({ items = new Map() }) { return <div /> }",
        // Regex literal default.
        "function Foo({ pattern = /abc/ }) { return <div /> }",
        // JSX element default.
        "function Foo({ icon = <span /> }) { return <div /> }",
        // JSX fragment default. (eslint-plugin-react v7.37 does not flag this; we do
        // since fragments create a new reference each render just like JSX elements.)
        "function Foo({ icon = <></> }) { return <div /> }",
        // Arrow component.
        "const Foo = ({ items = [] }) => <div />;",
        // Function expression component.
        "const Foo = function Foo({ items = [] }) { return <div />; };",
        // Default parameter inside React.forwardRef.
        "const Foo = React.forwardRef(({ items = [] }, ref) => <div ref={ref} />);",
        // Default parameter inside memo.
        "const Foo = memo(({ items = [] }) => <div />);",
        // Nested destructuring with object default.
        "function Foo({ data: { items = [] } }) { return <div /> }",
        // Whole sub-pattern default.
        "function Foo({ data = {}, ...rest }) { return <div /> }",
        // Array destructuring inside object destructuring with default.
        "function Foo({ data: [first = []] }) { return <div /> }",
        // Multiple violations in one component.
        "function Foo({ a = [], b = {} }) { return <div /> }",
        // Parenthesized object default.
        "function Foo({ items = ({}) }) { return <div /> }",
    ];

    Tester::new(NoObjectTypeAsDefaultProp::NAME, NoObjectTypeAsDefaultProp::PLUGIN, pass, fail)
        .test_and_snapshot();
}
