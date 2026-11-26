use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;

use oxc_ast::{
    AstKind,
    ast::{Argument, CallExpression, Expression, MethodDefinitionKind, ObjectMethodKind},
};
use oxc_cfg::{
    EdgeType, ErrorEdgeKind, InstructionKind, ReturnInstructionKind,
    graph::{
        Direction,
        visit::{Control, DfsEvent, set_depth_first_search},
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    rule::{DefaultRuleConfig, Rule},
};

fn getter_return_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Expected to always return a value in getter.")
        .with_help("Return a value from all code paths in getter.")
        .with_label(span)
}

#[derive(Debug, Default, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct GetterReturn {
    /// When set to `true`, allows getters to implicitly return `undefined` with a `return` statement containing no expression.
    pub allow_implicit: bool,
}

/// Methods that take a property descriptor as their third argument.
const DESCRIPTOR_METHODS: [(&str, &str); 2] =
    [("Object", "defineProperty"), ("Reflect", "defineProperty")];

/// Methods that take an object of property descriptors as their second argument.
const DESCRIPTOR_MAP_METHODS: [(&str, &str); 2] =
    [("Object", "create"), ("Object", "defineProperties")];

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Requires all getters to have a `return` statement.
    ///
    /// ### Why is this bad?
    ///
    /// Getters should always return a value. If they don't, it's probably a mistake.
    ///
    /// This rule does not run on TypeScript files, since type checking will
    /// catch getters that do not return a value.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// class Person {
    ///     get name() {
    ///         // no return
    ///     }
    /// }
    ///
    /// const obj = {
    ///     get foo() {
    ///         // object getter are also checked
    ///     }
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// class Person {
    ///     get name() {
    ///         return this._name;
    ///     }
    /// }
    /// ```
    GetterReturn,
    eslint,
    correctness,
    recommended,
    config = GetterReturn,
    version = "0.0.3",
    short_description = "Requires all getters to have a `return` statement.",
);

impl Rule for GetterReturn {
    fn from_configuration(value: Value) -> Result<Self, serde_json::error::Error> {
        DefaultRuleConfig::<Self>::from_value(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::Function(func) if !func.is_typescript_syntax() => {
                self.run_diagnostic(node, ctx, func.span);
            }
            AstKind::ArrowFunctionExpression(expr) => {
                self.run_diagnostic(node, ctx, expr.span);
            }
            _ => {}
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        // https://eslint.org/docs/latest/rules/getter-return#handled_by_typescript
        !ctx.source_type().is_typescript()
    }
}

impl GetterReturn {
    /// Checks whether it is necessary to check the node
    fn is_wanted_node(node: &AstNode, ctx: &LintContext<'_>) -> bool {
        let parent = ctx.nodes().parent_node(node.id());
        match parent.kind() {
            AstKind::MethodDefinition(mdef) => matches!(mdef.kind, MethodDefinitionKind::Get),
            AstKind::ObjectMethodProperty(prop) => {
                if matches!(prop.kind, ObjectMethodKind::Get) {
                    return true;
                }
                prop.key.is_specific_static_name("get")
                    && is_property_descriptor(ctx.nodes().parent_node(parent.id()), ctx)
            }
            AstKind::ObjectProperty(prop) => {
                prop.key.is_specific_static_name("get")
                    && is_property_descriptor(ctx.nodes().parent_node(parent.id()), ctx)
            }
            _ => false,
        }
    }

    fn run_diagnostic<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>, span: Span) {
        if !Self::is_wanted_node(node, ctx) {
            return;
        }

        let cfg = ctx.cfg();

        let graph = cfg.graph();
        let definitely_returns_in_all_codepaths = 'returns: {
            // The expression is the equivalent of return.
            // Therefore, if a function is an expression, it always returns its value.
            //
            // Example expression:
            // ```js
            // const fn = () => 1;
            // ```
            if let AstKind::ArrowFunctionExpression(arrow_expr) = node.kind() {
                if arrow_expr.is_expression() {
                    break 'returns true;
                }
                // If the signature of function supports the return of the `undefined` value,
                // you do not need to check this rule
                if let AstKind::Function(func) = node.kind()
                    && let Some(ret) = &func.return_type
                    && ret.type_annotation.is_maybe_undefined()
                {
                    break 'returns true;
                }
            }
            let output =
                set_depth_first_search(graph, Some(ctx.nodes().cfg_id(node.id())), |event| {
                    match event {
                        // We only need to check paths that are normal or jump.
                        DfsEvent::TreeEdge(a, b) => {
                            if graph.edges_connecting(a, b).any(|e| {
                                matches!(
                                    e.weight(),
                                    EdgeType::Normal
                                        | EdgeType::Jump
                                        | EdgeType::Error(ErrorEdgeKind::Explicit)
                                )
                            }) {
                                Control::Continue
                            } else {
                                Control::Prune
                            }
                        }
                        DfsEvent::Discover(basic_block_id, _) => {
                            let return_instruction =
                                cfg.basic_block(basic_block_id).instructions().iter().find(|it| {
                                    match it.kind {
                                        // Throws are classified as returning.
                                        InstructionKind::Return(_) | InstructionKind::Throw => true,

                                        // Ignore irrelevant elements.
                                        InstructionKind::ImplicitReturn
                                        | InstructionKind::Break(_)
                                        | InstructionKind::Continue(_)
                                        | InstructionKind::Iteration(_)
                                        | InstructionKind::Unreachable
                                        | InstructionKind::Condition
                                        | InstructionKind::Statement => false,
                                    }
                                });

                            let does_return = return_instruction.is_some_and(|ret| {
                                !matches!( ret.kind,
                                InstructionKind::Return(ReturnInstructionKind::ImplicitUndefined)
                                    if !self.allow_implicit
                                )
                            });

                            // Return true as the second argument to signify we should
                            // continue walking this branch, as we haven't seen anything
                            // that will signify to us that this path of the program will
                            // definitely return or throw.
                            if graph.edges_directed(basic_block_id, Direction::Outgoing).any(|e| {
                                matches!(
                                    e.weight(),
                                    EdgeType::Jump
                                        | EdgeType::Normal
                                        | EdgeType::Backedge
                                        | EdgeType::Error(ErrorEdgeKind::Explicit)
                                )
                            }) {
                                Control::Continue
                            } else if does_return {
                                Control::Prune
                            } else {
                                Control::Break(())
                            }
                        }
                        _ => Control::Continue,
                    }
                });

            output.break_value().is_none()
        };

        if !definitely_returns_in_all_codepaths {
            ctx.diagnostic(getter_return_diagnostic(span));
        }
    }
}

/// Is `node` an object expression used as a property descriptor?
///
/// Port of ESLint's `isPropertyDescriptor`.
fn is_property_descriptor<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
    let AstKind::ObjectExpression(descriptor) = node.kind() else {
        return false;
    };
    let parent = parent_skipping_parens(node, ctx);
    match parent.kind() {
        AstKind::CallExpression(call) => {
            is_argument_of_global_method_call(call, descriptor.span, 2, &DESCRIPTOR_METHODS, ctx)
        }
        AstKind::ObjectProperty(prop)
            if prop.value.without_parentheses().span() == descriptor.span =>
        {
            let map = ctx.nodes().parent_node(parent.id());
            let AstKind::ObjectExpression(map_expr) = map.kind() else {
                return false;
            };
            let AstKind::CallExpression(call) = parent_skipping_parens(map, ctx).kind() else {
                return false;
            };
            is_argument_of_global_method_call(call, map_expr.span, 1, &DESCRIPTOR_MAP_METHODS, ctx)
        }
        _ => false,
    }
}

fn parent_skipping_parens<'s, 'a>(node: &AstNode<'a>, ctx: &'s LintContext<'a>) -> &'s AstNode<'a> {
    let mut parent = ctx.nodes().parent_node(node.id());
    while let AstKind::ParenthesizedExpression(_) = parent.kind() {
        parent = ctx.nodes().parent_node(parent.id());
    }
    parent
}

/// Is the expression at `span` argument `index` of a call to one of `methods` on a global object?
fn is_argument_of_global_method_call(
    call: &CallExpression<'_>,
    span: Span,
    index: usize,
    methods: &[(&str, &str)],
    ctx: &LintContext<'_>,
) -> bool {
    if call
        .arguments
        .get(index)
        .and_then(Argument::as_expression)
        .is_none_or(|arg| arg.without_parentheses().span() != span)
    {
        return false;
    }
    let member = match call.callee.without_parentheses() {
        Expression::ChainExpression(chain) => chain.expression.member_expression(),
        callee => callee.as_member_expression(),
    };
    let Some(member) = member else {
        return false;
    };
    let Expression::Identifier(object) = member.object().without_parentheses() else {
        return false;
    };
    let Some(property) = member.static_property_name() else {
        return false;
    };
    methods.iter().any(|&(object_name, method)| object.name == object_name && property == method)
        && ctx.is_reference_to_global_variable(object)
}

#[test]
fn test() {
    use crate::tester::Tester;
    let pass = vec![
        ("var foo = { get bar(){return true;} };", None),
        ("var foo = { get bar() {return;} };", Some(serde_json::json!([{ "allowImplicit": true }]))),
        ("var foo = { get bar(){return true;} };", Some(serde_json::json!([{ "allowImplicit": true }]))),
        ("var foo = { get bar(){if(bar) {return;} return true;} };", Some(serde_json::json!([{ "allowImplicit": true }]))),
        ("class foo { get bar(){return true;} }", None),
        ("class foo { get bar(){if(baz){return true;} else {return false;} } }", None),
        ("class foo { get(){return true;} }", None),
        ("class foo { get bar(){return true;} }", Some(serde_json::json!([{ "allowImplicit": true }]))),
        ("class foo { get bar(){return;} }", Some(serde_json::json!([{ "allowImplicit": true }]))),
        ("Object.defineProperty(foo, \"bar\", { get: function () {return true;}});", None),
        ("Object.defineProperty(foo, \"bar\", { get: function () { ~function (){ return true; }();return true;}});", None),
        ("Object.defineProperty(foo, \"bar\", { set: function () {}});", None),
        ("Object.defineProperty(foo, \"bar\", { set: () => {}});", None),
        ("Object.defineProperties(foo, { bar: { get: function () {return true;}} });", None),
        ("Object.defineProperties(foo, { bar: { get: function () { ~function (){ return true; }(); return true;}} });", None),
        ("Object.defineProperties(foo, { bar: { set: function () {}} });", None),
        ("Reflect.defineProperty(foo, \"bar\", { get: function () {return true;}});", None),
        ("Reflect.defineProperty(foo, \"bar\", { get: function () { ~function (){ return true; }();return true;}});", None),
        ("Object.create(foo, { bar: { get() {return true;} } });", None),
        ("Object.create(foo, { bar: { get: function () {return true;} } });", None),
        ("Object.create(foo, { bar: { get: () => {return true;} } });", None),
        ("Object.defineProperty(foo, \"bar\", { get: function () {return true;}});", Some(serde_json::json!([{ "allowImplicit": true }]))),
        ("Object.defineProperty(foo, \"bar\", { get: function (){return;}});", Some(serde_json::json!([{ "allowImplicit": true }]))),
        ("Object.defineProperties(foo, { bar: { get: function () {return true;}} });", Some(serde_json::json!([{ "allowImplicit": true }]))),
        ("Object.defineProperties(foo, { bar: { get: function () {return;}} });", Some(serde_json::json!([{ "allowImplicit": true }]))),
        ("Reflect.defineProperty(foo, \"bar\", { get: function () {return true;}});", Some(serde_json::json!([{ "allowImplicit": true }]))),
        ("var get = function(){};", None),
        ("var get = function(){ return true; };", None),
        ("var foo = { bar(){} };", None),
        ("var foo = { bar(){ return true; } };", None),
        ("var foo = { bar: function(){} };", None),
        ("var foo = { bar: function(){return;} };", None),
        ("var foo = { bar: function(){return true;} };", None),
        ("var foo = { get: function () {} }", None),
        ("var foo = { get: () => {}};", None),
        ("class C { get; foo() {} }", None),
        ("foo.defineProperty(null, { get() {} });", None),
        ("foo.defineProperties(null, { bar: { get() {} } });", None),
        ("foo.create(null, { bar: { get() {} } });", None),
        // not a property descriptor
        ("Object.create({ get() {} });", None),
        ("Object.create(foo, { get() {} });", None),
        ("Object.defineProperty(foo, { get() {} }, 'bar');", None),
        ("const key = 'value'; Object.defineProperty({}, 'x', { [key]: function () {} });", None),
        // `Object` and `Reflect` must be the globals
        ("function register(Reflect) { Reflect.defineProperty({}, 'item', { get: function () {} }); }", None),
        ("function create(Object) { Object.create(null, { x: { get() {} } }); }", None),
        ("var foo = { get willThrowSoValid() { throw MyException() } };", None),
        (
            "const originalClearTimeout = targetWindow.clearTimeout;
        Object.defineProperty(targetWindow, 'vscodeOriginalClearTimeout', { get: () => originalClearTimeout });
        ",
            None,
        ),
        (r"
        var foo = {
                get bar() {
                        let name = ([] || [])[1];
                        return name;
                },
        };
        ", None),
        ("var foo = { get bar() { try { return a(); } finally {  } } };", None),
        ("
        var foo = {
            get bar() {
                switch (baz) {
                    case VS_LIGHT_THEME: return a;
                    case VS_HC_THEME: return b;
                    case VS_HC_LIGHT_THEME: return c;
                    default: return d;
                }
            }
        };
        ", None),
        // adapted from: https://github.com/1024pix/pix/blob/1352bd8d7f6070f1ff8da79867f543c1c1926e59/mon-pix/app/components/progress-bar.js#L29-L43
        ("
        export default class ProgressBar extends Component {
            get steps() {
                const steps = [];

                for (let i = 0; i < this.maxStepsNumber; i++) {
                    steps.push({
                        stepnum: i + 1,
                    });
                }

                return steps;
            }
        }", None),
        ("
        var foo = {
            get bar() {
                for (let i = 0; i<10; i++) {
                    if (i === 5) {
                        return i;
                    }
                }
                return 0;
            }
        }", None),
        ("
        var foo = {
            get bar() {
                let i = 0;
                while (i < 10) {
                    if (i === 5) {
                        return i;
                    }
                    i++;
                }
                return 0;
            }
        }", None),
    ];

    let fail = vec![
        ("var foo = { get bar() {} };", None),
        ("var foo = { get\n bar () {} };", None),
        ("var foo = { get bar(){if(baz) {return true;}} };", None),
        ("var foo = { get bar() { ~function () {return true;}} };", None),
        ("var foo = { get bar() { return; } };", None),
        ("var foo = { get bar() {} };", Some(serde_json::json!([{ "allowImplicit": true }]))),
        (
            "var foo = { get bar() {if (baz) {return;}} };",
            Some(serde_json::json!([{ "allowImplicit": true }])),
        ),
        ("class foo { get bar(){} }", None),
        ("var foo = class {\n  static get\nbar(){} }", None),
        ("class foo { get bar(){ if (baz) { return true; }}}", None),
        ("class foo { get bar(){ ~function () { return true; }()}}", None),
        ("class foo { get bar(){} }", Some(serde_json::json!([{ "allowImplicit": true }]))),
        (
            "class foo { get bar(){if (baz) {return true;} } }",
            Some(serde_json::json!([{ "allowImplicit": true }])),
        ),
        ("Object.defineProperty(foo, 'bar', { get: function (){}});", None),
        ("Object.defineProperty(foo, 'bar', { get: function getfoo (){}});", None),
        ("Object.defineProperty(foo, 'bar', { get(){} });", None),
        ("Object.defineProperty(foo, 'bar', { get: () => {}});", None),
        ("Object.defineProperty(foo, \"bar\", { get: function (){if(bar) {return true;}}});", None),
        (
            "Object.defineProperty(foo, \"bar\", { get: function (){ ~function () { return true; }()}});",
            None,
        ),
        ("Reflect.defineProperty(foo, 'bar', { get: function (){}});", None),
        ("Object.create(foo, { bar: { get: function() {} } })", None),
        ("Object.create(foo, { bar: { get() {} } })", None),
        ("Object.create(foo, { bar: { get: () => {} } })", None),
        ("Object.create(null, { x: { get() {} } });", None),
        ("Object.defineProperty({}, 'x', { ['get']: function () {} });", None),
        ("Object.defineProperty(foo, 'bar', ({ get: function () {} }));", None),
        ("Object.defineProperties(foo, { bar: ({ get() {} }) });", None),
        (
            "Object.defineProperties(foo, { bar: { get: function () {}} });",
            Some(serde_json::json!([{ "allowImplicit": true }])),
        ),
        (
            "Object.defineProperties(foo, { bar: { get: function (){if(bar) {return true;}}}});",
            Some(serde_json::json!([{ "allowImplicit": true }])),
        ),
        (
            "Object.defineProperties(foo, { bar: { get: function () {~function () { return true; }()}} });",
            Some(serde_json::json!([{ "allowImplicit": true }])),
        ),
        (
            "Object.defineProperty(foo, \"bar\", { get: function (){}});",
            Some(serde_json::json!([{ "allowImplicit": true }])),
        ),
        (
            "Object.create(foo, { bar: { get: function (){} } });",
            Some(serde_json::json!([{ "allowImplicit": true }])),
        ),
        (
            "Reflect.defineProperty(foo, \"bar\", { get: function (){}});",
            Some(serde_json::json!([{ "allowImplicit": true }])),
        ),
        ("Object?.defineProperty(foo, 'bar', { get: function (){} });", None),
        ("(Object?.defineProperty)(foo, 'bar', { get: function (){} });", None),
        (
            "Object?.defineProperty(foo, 'bar', { get: function (){} });",
            Some(serde_json::json!([{ "allowImplicit": true }])),
        ),
        (
            "(Object?.defineProperty)(foo, 'bar', { get: function (){} });",
            Some(serde_json::json!([{ "allowImplicit": true }])),
        ),
        (
            "(Object?.create)(foo, { bar: { get: function (){} } });",
            Some(serde_json::json!([{ "allowImplicit": true }])),
        ),
        ("var foo = { get bar() { try { return a(); } catch {} } };", None),
        ("var foo = { get bar() { try { return a(); } catch {  } finally {  } } };", None),
        (
            "
        var foo = {
            get bar() {
                for (let i = 0; i<10; i++) {
                    return i;
                }
            }
        }",
            None,
        ),
        (
            "
        var foo = {
            get bar() {
                let i = 0;
                while (i < 10) {
                    return i;
                }
            }
        }",
            None,
        ),
    ];

    Tester::new(GetterReturn::NAME, GetterReturn::PLUGIN, pass, fail)
        .change_rule_path_extension("js")
        .test_and_snapshot();

    // TypeScript tests
    let pass = vec![(
        "var foo = {
            get bar(): boolean | undefined {
                if (Math.random() > 0.5) {
                    return true;
                }
            }
        };",
        None,
    )];

    let fail = vec![];

    Tester::new(GetterReturn::NAME, GetterReturn::PLUGIN, pass, fail).test();
}
