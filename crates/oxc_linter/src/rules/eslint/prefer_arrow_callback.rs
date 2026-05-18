use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use oxc_ast::{
    AstKind,
    ast::{
        Argument, Function, FunctionType, IdentifierReference, MetaProperty, Super, ThisExpression,
    },
};
use oxc_ast_visit::Visit;
use oxc_codegen::{Context, Gen};
use oxc_diagnostics::OxcDiagnostic;
use oxc_ecmascript::{BoundNames, IsSimpleParameterList};
use oxc_macros::declare_oxc_lint;
use oxc_semantic::ScopeFlags;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    rule::{DefaultRuleConfig, Rule},
};

fn prefer_arrow_callback_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected function expression.")
        .with_help("Use an arrow function instead.")
        .with_label(span)
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
struct PreferArrowCallbackConfig {
    allow_named_functions: bool,
    allow_unbound_this: bool,
}

impl Default for PreferArrowCallbackConfig {
    fn default() -> Self {
        Self { allow_named_functions: false, allow_unbound_this: true }
    }
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
pub struct PreferArrowCallback(PreferArrowCallbackConfig);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Requires using arrow functions for callbacks.
    ///
    /// ### Why is this bad?
    ///
    /// Arrow functions are generally better suited for callbacks because they:
    ///
    /// - inherit `this` from the surrounding scope, avoiding a common source of bugs;
    /// - are shorter and easier to read;
    /// - cannot be used as constructors, which is desirable for callbacks.
    ///
    /// ### Options
    ///
    /// ```json
    /// {
    ///   "prefer-arrow-callback": ["error", {
    ///     "allowNamedFunctions": false,
    ///     "allowUnboundThis": true
    ///   }]
    /// }
    /// ```
    ///
    /// - `allowNamedFunctions` (default `false`) — when `true`, named function
    ///   expressions are allowed.
    /// - `allowUnboundThis` (default `true`) — when `false`, function
    ///   expressions that reference `this` are reported even when they are not
    ///   bound to a `this` value.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// foo(function (a) { return a; });
    /// foo(function () { return this.a; }.bind(this));
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// foo(a => a);
    /// foo(function*() { yield; });
    /// foo(function () { this; });
    /// foo(function bar() { bar(); });
    /// ```
    PreferArrowCallback,
    eslint,
    style,
    fix,
    config = PreferArrowCallback,
    version = "1.65.0",
);

impl Rule for PreferArrowCallback {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::Function(func) = node.kind() else {
            return;
        };
        if func.r#type != FunctionType::FunctionExpression {
            return;
        }
        if func.generator {
            return;
        }
        if self.0.allow_named_functions && func.id.is_some() {
            return;
        }

        let Some(body) = &func.body else {
            return;
        };

        let info = get_callback_info(node, func, ctx);
        if !info.is_callback {
            return;
        }

        if references_function_name(func, ctx) {
            return;
        }

        let mut scanner = ScopeScanner { this: false, sup: false, meta: false, arguments: false };
        scanner.visit_function_body(body);

        if scanner.sup || scanner.meta {
            return;
        }
        if scanner.arguments && !params_bind(func, "arguments") {
            return;
        }

        let allow_unbound = self.0.allow_unbound_this;
        if allow_unbound && scanner.this && !info.is_lexical_this {
            return;
        }

        let diagnostic = prefer_arrow_callback_diagnostic(func.span);

        let can_fix_this = info.is_lexical_this || !scanner.this;
        let no_dup_params = !has_duplicate_simple_params(func);
        let first_param_not_this = !first_param_is_this(func);

        if can_fix_this && no_dup_params && first_param_not_this {
            ctx.diagnostic_with_fix(diagnostic, |fixer| build_fix(fixer, ctx, node, func, &info));
        } else {
            ctx.diagnostic(diagnostic);
        }
    }
}

struct CallbackInfo {
    is_callback: bool,
    is_lexical_this: bool,
    bind_this_call_span: Option<Span>,
}

fn get_callback_info<'a>(
    node: &AstNode<'a>,
    func: &Function<'a>,
    ctx: &LintContext<'a>,
) -> CallbackInfo {
    let mut info =
        CallbackInfo { is_callback: false, is_lexical_this: false, bind_this_call_span: None };
    let mut current_span = func.span;
    let mut bound = false;
    let mut just_passed_bind = false;
    for parent in ctx.nodes().ancestors(node.id()) {
        match parent.kind() {
            AstKind::ParenthesizedExpression(p) => {
                current_span = p.span;
                // Preserve just_passed_bind so wrappers around `.bind` still chain to the call.
            }
            AstKind::ChainExpression(c) => {
                current_span = c.span;
                // Preserve just_passed_bind: `function?.bind` chain still callable as `.bind(this)`.
            }
            AstKind::LogicalExpression(l) => {
                current_span = l.span;
                just_passed_bind = false;
            }
            AstKind::ConditionalExpression(c) => {
                current_span = c.span;
                just_passed_bind = false;
            }
            AstKind::SequenceExpression(s) => {
                if s.expressions.last().map(GetSpan::span) != Some(current_span) {
                    return info;
                }
                current_span = s.span;
                just_passed_bind = false;
            }
            AstKind::StaticMemberExpression(m) => {
                if m.object.span() != current_span || m.property.name != "bind" {
                    return info;
                }
                current_span = m.span;
                just_passed_bind = true;
            }
            AstKind::ComputedMemberExpression(m) => {
                if m.object.span() != current_span
                    || m.static_property_name().is_none_or(|name| name.as_str() != "bind")
                {
                    return info;
                }
                current_span = m.span;
                just_passed_bind = true;
            }
            AstKind::CallExpression(call) => {
                if call.callee.span() == current_span {
                    if !just_passed_bind {
                        // The function (or wrapped form) is being called directly — IIFE.
                        return info;
                    }
                    if !bound {
                        bound = true;
                        if call.arguments.len() == 1
                            && matches!(&call.arguments[0], Argument::ThisExpression(_))
                        {
                            info.is_lexical_this = true;
                            info.bind_this_call_span = Some(call.span);
                        }
                    }
                    current_span = call.span;
                    just_passed_bind = false;
                } else {
                    info.is_callback = true;
                    return info;
                }
            }
            AstKind::NewExpression(new_expr) => {
                if new_expr.callee.span() == current_span {
                    return info;
                }
                info.is_callback = true;
                return info;
            }
            _ => return info,
        }
    }
    info
}

fn build_fix<'a>(
    fixer: RuleFixer<'_, 'a>,
    ctx: &LintContext<'a>,
    node: &AstNode<'a>,
    func: &Function<'a>,
    info: &CallbackInfo,
) -> RuleFix {
    let arrow_text = build_arrow_text(func, fixer);

    let (replace_span, effective_span) = if info.is_lexical_this {
        let Some(call_span) = info.bind_this_call_span else { return fixer.noop() };
        let mut span = call_span;
        // Extend through wrapping ChainExpression if present.
        for ancestor in ctx.nodes().ancestors(node.id()) {
            if matches!(ancestor.kind(), AstKind::ChainExpression(_))
                && ancestor.kind().span().contains_inclusive(span)
            {
                span = ancestor.kind().span();
                continue;
            }
            break;
        }
        (span, span)
    } else {
        (func.span, func.span)
    };

    let needs_parens = needs_paren_wrap(node, effective_span, ctx);

    let replacement = if needs_parens { format!("({arrow_text})") } else { arrow_text };

    fixer.replace(replace_span, replacement)
}

fn build_arrow_text<'a>(func: &Function<'a>, fixer: RuleFixer<'_, 'a>) -> String {
    let mut text = String::new();
    if func.r#async {
        text.push_str("async ");
    }
    if let Some(tp) = &func.type_parameters {
        let mut codegen = fixer.codegen();
        tp.print(&mut codegen, Context::empty());
        text.push_str(&codegen.into_source_text());
    }
    let params_text = fixer.source_range(func.params.span);
    text.push_str(params_text);
    let pre_body_end = if let Some(rt) = &func.return_type {
        text.push_str(fixer.source_range(rt.span));
        rt.span.end
    } else {
        func.params.span.end
    };
    text.push_str(" =>");
    let body = func.body.as_ref().expect("function expression always has body");
    let between = fixer.source_range(Span::new(pre_body_end, body.span.start));
    text.push_str(between);
    text.push_str(fixer.source_range(body.span));
    text
}

fn needs_paren_wrap<'a>(node: &AstNode<'a>, effective_span: Span, ctx: &LintContext<'a>) -> bool {
    let mut already_parens = false;
    for parent in ctx.nodes().ancestors(node.id()) {
        let span = parent.kind().span();
        // Skip ancestors whose span is inside or equal to effective_span.
        if span.start >= effective_span.start && span.end <= effective_span.end {
            continue;
        }
        match parent.kind() {
            AstKind::ParenthesizedExpression(_) => {
                already_parens = true;
            }
            AstKind::ChainExpression(_) => {}
            AstKind::CallExpression(_)
            | AstKind::ConditionalExpression(_)
            | AstKind::NewExpression(_) => return false,
            _ => return !already_parens,
        }
    }
    false
}

fn has_duplicate_simple_params(func: &Function<'_>) -> bool {
    if func.params.items.len() < 2 || !func.params.is_simple_parameter_list() {
        return false;
    }

    let mut duplicate = false;
    let mut seen = rustc_hash::FxHashSet::default();
    seen.reserve(func.params.items.len());
    func.params.bound_names(&mut |ident| {
        if !seen.insert(ident.name.as_str()) {
            duplicate = true;
        }
    });
    duplicate
}

fn first_param_is_this(func: &Function<'_>) -> bool {
    func.this_param.is_some()
}

fn params_bind(func: &Function<'_>, name: &str) -> bool {
    let mut binds_name = false;
    func.params.bound_names(&mut |ident| {
        if ident.name == name {
            binds_name = true;
        }
    });
    binds_name
}

fn references_function_name(func: &Function<'_>, ctx: &LintContext<'_>) -> bool {
    let Some(id) = &func.id else {
        return false;
    };
    let body_span = func.body.as_ref().map_or(func.span, |body| body.span);
    ctx.scoping().get_resolved_references(id.symbol_id()).any(|reference| {
        let ref_node = ctx.nodes().get_node(reference.node_id());
        body_span.contains_inclusive(ref_node.span())
    })
}

struct ScopeScanner {
    this: bool,
    sup: bool,
    meta: bool,
    arguments: bool,
}

impl<'a> Visit<'a> for ScopeScanner {
    fn visit_function(&mut self, _it: &Function<'a>, _flags: ScopeFlags) {}

    fn visit_this_expression(&mut self, _it: &ThisExpression) {
        self.this = true;
    }

    fn visit_super(&mut self, _it: &Super) {
        self.sup = true;
    }

    fn visit_meta_property(&mut self, it: &MetaProperty<'a>) {
        if it.meta.name == "new" && it.property.name == "target" {
            self.meta = true;
        }
    }

    fn visit_identifier_reference(&mut self, it: &IdentifierReference<'a>) {
        if it.name == "arguments" {
            self.arguments = true;
        }
    }

    fn visit_class(&mut self, _it: &oxc_ast::ast::Class<'a>) {}

    fn visit_static_block(&mut self, _it: &oxc_ast::ast::StaticBlock<'a>) {}
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("foo(a => a);", None),
        ("foo(function*() {});", None),
        ("foo(function() { this; });", None),
        ("foo(function bar() {});", Some(serde_json::json!([{ "allowNamedFunctions": true }]))),
        ("foo(function() { (() => this); });", None),
        ("foo(function() { this; }.bind(obj));", None),
        ("foo(function() { this; }.call(this));", None),
        ("foo(a => { (function() {}); });", None),
        ("var foo = function foo() {};", None),
        ("(function foo() {})();", None),
        ("foo(function bar() { bar; });", None),
        ("foo(function bar() { arguments; });", None),
        ("foo(function bar() { arguments; }.bind(this));", None),
        ("foo(function bar() { new.target; });", None),
        ("foo(function bar() { new.target; }.bind(this));", None),
        ("foo(function bar() { this; }.bind(this, somethingElse));", None),
        ("foo((function() {}).bind.bar)", None),
        ("foo((function() { this.bar(); }).bind(obj).bind(this))", None),
        ("foo((a:string) => a);", None),
        (
            "foo(function bar(a:string) {});",
            Some(serde_json::json!([{ "allowNamedFunctions": true }])),
        ),
        ("test('clean', function (this: any) { this.foo = 'Cleaned!';});", None),
        ("obj.test('clean', function (foo) { this.foo = 'Cleaned!'; });", None),
    ];

    let fail = vec![
        ("foo(function bar() {});", None),
        ("foo(function() {});", Some(serde_json::json!([{ "allowNamedFunctions": true }]))),
        ("foo(function bar() {});", Some(serde_json::json!([{ "allowNamedFunctions": false }]))),
        ("foo(function() {});", None),
        ("foo(nativeCb || function() {});", None),
        ("foo(bar ? function() {} : function() {});", None),
        ("foo(function() { (function() { this; }); });", None),
        ("foo(function() { this; }.bind(this));", None),
        ("foo(bar || function() { this; }.bind(this));", None),
        ("foo(function() { (() => this); }.bind(this));", None),
        ("foo(function bar(a) { a; });", None),
        ("foo(function(a) { a; });", None),
        ("foo(function(arguments) { arguments; });", None),
        ("foo(function() { this; });", Some(serde_json::json!([{ "allowUnboundThis": false }]))),
        (
            "foo(function() { (() => this); });",
            Some(serde_json::json!([{ "allowUnboundThis": false }])),
        ),
        ("qux(function(foo, bar, baz) { return foo * 2; })", None),
        ("qux(function(foo, bar, baz) { return foo * bar; }.bind(this))", None),
        ("qux(function(foo, bar, baz) { return foo * this.qux; }.bind(this))", None),
        ("foo(function() {}.bind(this, somethingElse))", None),
        (
            "qux(function(foo = 1, [bar = 2] = [], {qux: baz = 3} = {foo: 'bar'}) { return foo + bar; });",
            None,
        ),
        ("qux(function(baz, baz) { })", None),
        ("qux(function( /* no params */ ) { })", None),
        (
            "qux(function( /* a */ foo /* b */ , /* c */ bar /* d */ , /* e */ baz /* f */ ) { return foo; })",
            None,
        ),
        ("qux(async function (foo = 1, bar = 2, baz = 3) { return baz; })", None),
        ("qux(async function (foo = 1, bar = 2, baz = 3) { return this; }.bind(this))", None),
        ("foo((bar || function() {}).bind(this))", None),
        ("foo(function() {}.bind(this).bind(obj))", None),
        ("foo?.(function() {});", None),
        ("foo?.(function() { return this; }.bind(this));", None),
        ("foo(function() { return this; }?.bind(this));", None),
        ("foo((function() { return this; }?.bind)(this));", None),
        (
            "
                        test(
                            function ()
                            { }
                        );
                        ",
            None,
        ),
        (
            "
                        test(
                            function (
                                ...args
                            ) /* Lorem ipsum
                            dolor sit amet. */ {
                                return args;
                            }
                        );
                        ",
            None,
        ),
        ("foo(function(a:string) {});", Some(serde_json::json!([{ "allowNamedFunctions": true }]))),
        ("foo(function bar(a:string) { a; });", None),
        ("foo(function(a:any) { a; });", None),
        ("foo(function(arguments:any) { arguments; });", None),
        (
            "foo(function(a:string) { this; });",
            Some(serde_json::json!([{ "allowUnboundThis": false }])),
        ),
        ("qux(function(foo:string, bar:number, baz:string) { return foo * 2; })", None),
        (
            "qux(function(foo:number, bar:number, baz:number) { return foo * bar; }.bind(this))",
            None,
        ),
        ("qux(function(foo:any, bar:any, baz:any) { return foo * this.qux; }.bind(this))", None),
        ("qux(function(baz:string, baz:string) { })", None),
        (
            "qux(function( /* a */ foo:string /* b */ , /* c */ bar:string /* d */ , /* e */ baz:string /* f */ ) { return foo; })",
            None,
        ),
        (
            "qux(async function (foo:number = 1, bar:number = 2, baz:number = 3) { return baz; })",
            None,
        ),
        (
            "qux(async function (foo:number = 1, bar:number = 2, baz:number = 3) { return this; }.bind(this))",
            None,
        ),
        ("foo(function():string { return 'foo' });", None),
        ("test('foo', function (this: any) {});", None),
        ("test('foo', function (this: any, x: number) { x; });", None),
        ("foo(function bar<T>(value: T) { return value; });", None),
    ];

    let fix = vec![
        ("foo(function bar() {});", "foo(() => {});", None),
        (
            "foo(function() {});",
            "foo(() => {});",
            Some(serde_json::json!([{ "allowNamedFunctions": true }])),
        ),
        (
            "foo(function bar() {});",
            "foo(() => {});",
            Some(serde_json::json!([{ "allowNamedFunctions": false }])),
        ),
        ("foo(function() {});", "foo(() => {});", None),
        ("foo(nativeCb || function() {});", "foo(nativeCb || (() => {}));", None),
        ("foo(bar ? function() {} : function() {});", "foo(bar ? () => {} : () => {});", None),
        (
            "foo(function() { (function() { this; }); });",
            "foo(() => { (function() { this; }); });",
            None,
        ),
        ("foo(function() { this; }.bind(this));", "foo(() => { this; });", None),
        ("foo(bar || function() { this; }.bind(this));", "foo(bar || (() => { this; }));", None),
        ("foo(function() { (() => this); }.bind(this));", "foo(() => { (() => this); });", None),
        ("foo(function bar(a) { a; });", "foo((a) => { a; });", None),
        ("foo(function(a) { a; });", "foo((a) => { a; });", None),
        ("foo(function(arguments) { arguments; });", "foo((arguments) => { arguments; });", None),
        (
            "qux(function(foo, bar, baz) { return foo * 2; })",
            "qux((foo, bar, baz) => { return foo * 2; })",
            None,
        ),
        (
            "qux(function(foo, bar, baz) { return foo * bar; }.bind(this))",
            "qux((foo, bar, baz) => { return foo * bar; })",
            None,
        ),
        (
            "qux(function(foo, bar, baz) { return foo * this.qux; }.bind(this))",
            "qux((foo, bar, baz) => { return foo * this.qux; })",
            None,
        ),
        (
            "foo(function() {}.bind(this, somethingElse))",
            "foo((() => {}).bind(this, somethingElse))",
            None,
        ),
        (
            "qux(function(foo = 1, [bar = 2] = [], {qux: baz = 3} = {foo: 'bar'}) { return foo + bar; });",
            "qux((foo = 1, [bar = 2] = [], {qux: baz = 3} = {foo: 'bar'}) => { return foo + bar; });",
            None,
        ),
        ("qux(function( /* no params */ ) { })", "qux(( /* no params */ ) => { })", None),
        (
            "qux(function( /* a */ foo /* b */ , /* c */ bar /* d */ , /* e */ baz /* f */ ) { return foo; })",
            "qux(( /* a */ foo /* b */ , /* c */ bar /* d */ , /* e */ baz /* f */ ) => { return foo; })",
            None,
        ),
        (
            "qux(async function (foo = 1, bar = 2, baz = 3) { return baz; })",
            "qux(async (foo = 1, bar = 2, baz = 3) => { return baz; })",
            None,
        ),
        (
            "qux(async function (foo = 1, bar = 2, baz = 3) { return this; }.bind(this))",
            "qux(async (foo = 1, bar = 2, baz = 3) => { return this; })",
            None,
        ),
        ("foo(function() {}.bind(this).bind(obj))", "foo((() => {}).bind(obj))", None),
        ("foo?.(function() {});", "foo?.(() => {});", None),
        ("foo?.(function() { return this; }.bind(this));", "foo?.(() => { return this; });", None),
        ("foo(function() { return this; }?.bind(this));", "foo(() => { return this; });", None),
        (
            "
                        test(
                            function ()
                            { }
                        );
                        ",
            "
                        test(
                            () =>
                            { }
                        );
                        ",
            None,
        ),
        (
            "
                        test(
                            function (
                                ...args
                            ) /* Lorem ipsum
                            dolor sit amet. */ {
                                return args;
                            }
                        );
                        ",
            "
                        test(
                            (
                                ...args
                            ) => /* Lorem ipsum
                            dolor sit amet. */ {
                                return args;
                            }
                        );
                        ",
            None,
        ),
        (
            "foo(function(a:string) {});",
            "foo((a:string) => {});",
            Some(serde_json::json!([{ "allowNamedFunctions": true }])),
        ),
        ("foo(function bar(a:string) { a; });", "foo((a:string) => { a; });", None),
        ("foo(function(a:any) { a; });", "foo((a:any) => { a; });", None),
        (
            "foo(function(arguments:any) { arguments; });",
            "foo((arguments:any) => { arguments; });",
            None,
        ),
        (
            "qux(function(foo:string, bar:number, baz:string) { return foo * 2; })",
            "qux((foo:string, bar:number, baz:string) => { return foo * 2; })",
            None,
        ),
        (
            "qux(function(foo:number, bar:number, baz:number) { return foo * bar; }.bind(this))",
            "qux((foo:number, bar:number, baz:number) => { return foo * bar; })",
            None,
        ),
        (
            "qux(function(foo:any, bar:any, baz:any) { return foo * this.qux; }.bind(this))",
            "qux((foo:any, bar:any, baz:any) => { return foo * this.qux; })",
            None,
        ),
        (
            "qux(function( /* a */ foo:string /* b */ , /* c */ bar:string /* d */ , /* e */ baz:string /* f */ ) { return foo; })",
            "qux(( /* a */ foo:string /* b */ , /* c */ bar:string /* d */ , /* e */ baz:string /* f */ ) => { return foo; })",
            None,
        ),
        (
            "qux(async function (foo:number = 1, bar:number = 2, baz:number = 3) { return baz; })",
            "qux(async (foo:number = 1, bar:number = 2, baz:number = 3) => { return baz; })",
            None,
        ),
        (
            "qux(async function (foo:number = 1, bar:number = 2, baz:number = 3) { return this; }.bind(this))",
            "qux(async (foo:number = 1, bar:number = 2, baz:number = 3) => { return this; })",
            None,
        ),
        ("foo(function():string { return 'foo' });", "foo(():string => { return 'foo' });", None),
        (
            "foo(function bar<T>(value: T) { return value; });",
            "foo(<T,>(value: T) => { return value; });",
            None,
        ),
    ];

    Tester::new(PreferArrowCallback::NAME, PreferArrowCallback::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
