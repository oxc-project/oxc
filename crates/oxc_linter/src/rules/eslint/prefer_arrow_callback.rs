use oxc_ast::{
    AstKind,
    ast::{
        Argument, BindingPattern, CallExpression, ChainElement, Expression, Function, FunctionType,
        IdentifierReference, MetaProperty, Super, ThisExpression,
    },
};
use oxc_ast_visit::Visit;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::ScopeFlags;
use oxc_span::{GetSpan, Span};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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
    version = "next",
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

        let fn_name = func.id.as_ref().map(|id| id.name.as_str());
        let arguments_param = func.params.items.iter().any(|p| param_binds(p, "arguments"));

        let mut scanner = ScopeScanner {
            depth: 0,
            this: false,
            sup: false,
            meta: false,
            arguments: false,
            fn_name,
            fn_name_referenced: false,
        };
        scanner.visit_function_body(body);

        if scanner.sup || scanner.meta {
            return;
        }
        if fn_name.is_some() && scanner.fn_name_referenced {
            return;
        }
        if scanner.arguments && !arguments_param {
            return;
        }

        let info = get_callback_info(node, func, ctx);
        if !info.is_callback {
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
}

fn get_callback_info<'a>(
    node: &AstNode<'a>,
    func: &Function<'a>,
    ctx: &LintContext<'a>,
) -> CallbackInfo {
    let mut info = CallbackInfo { is_callback: false, is_lexical_this: false };
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
    let arrow_text = build_arrow_text(func, ctx);

    let (replace_span, effective_span) = if info.is_lexical_this {
        // The `.bind(this)` chain must be `function.bind(this)` directly on the function.
        let Some((_member_span, call_span)) = direct_bind_this(node, func, ctx) else {
            return fixer.noop();
        };
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

fn build_arrow_text<'a>(func: &Function<'a>, ctx: &LintContext<'a>) -> String {
    let mut text = String::new();
    if func.r#async {
        text.push_str("async ");
    }
    if let Some(tp) = &func.type_parameters {
        text.push_str(ctx.source_range(tp.span));
    }
    let params_text = ctx.source_range(func.params.span);
    text.push_str(params_text);
    let pre_body_end = if let Some(rt) = &func.return_type {
        text.push_str(ctx.source_range(rt.span));
        rt.span.end
    } else {
        func.params.span.end
    };
    text.push_str(" =>");
    let body = func.body.as_ref().expect("function expression always has body");
    let between = ctx.source_range(Span::new(pre_body_end, body.span.start));
    text.push_str(between);
    text.push_str(ctx.source_range(body.span));
    text
}

/// If function's direct parent is `MemberExpression(.bind)` and that
/// MemberExpression's parent is a `CallExpression` whose only argument is
/// `this`, return the spans of the MemberExpression and the CallExpression.
fn direct_bind_this<'a>(
    node: &AstNode<'a>,
    func: &Function<'a>,
    ctx: &LintContext<'a>,
) -> Option<(Span, Span)> {
    let mut ancestors = ctx.nodes().ancestors(node.id());
    let mem_node = ancestors.next()?;
    let AstKind::StaticMemberExpression(member) = mem_node.kind() else {
        return None;
    };
    if member.object.span() != func.span || member.property.name != "bind" {
        return None;
    }
    let mut next = ancestors.next()?;
    if matches!(next.kind(), AstKind::ChainExpression(_)) {
        next = ancestors.next()?;
    }
    let AstKind::CallExpression(call) = next.kind() else {
        return None;
    };
    if call.callee.span() != member.span && !ancestor_chain_callee_matches(call, member.span) {
        return None;
    }
    if call.arguments.len() != 1 {
        return None;
    }
    if !matches!(&call.arguments[0], Argument::ThisExpression(_)) {
        return None;
    }
    Some((member.span, call.span))
}

fn ancestor_chain_callee_matches(call: &CallExpression<'_>, member_span: Span) -> bool {
    let Expression::ChainExpression(chain) = &call.callee else {
        return false;
    };
    match &chain.expression {
        ChainElement::ComputedMemberExpression(m) => m.span == member_span,
        ChainElement::StaticMemberExpression(m) => m.span == member_span,
        ChainElement::PrivateFieldExpression(m) => m.span == member_span,
        ChainElement::CallExpression(_) | ChainElement::TSNonNullExpression(_) => false,
    }
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
    let names: Vec<&str> = func
        .params
        .items
        .iter()
        .filter_map(|p| match &p.pattern {
            BindingPattern::BindingIdentifier(id) => Some(id.name.as_str()),
            _ => None,
        })
        .collect();
    if names.len() != func.params.items.len() {
        return false;
    }
    let mut seen = rustc_hash::FxHashSet::default();
    for n in &names {
        if !seen.insert(*n) {
            return true;
        }
    }
    false
}

fn first_param_is_this(func: &Function<'_>) -> bool {
    func.this_param.is_some()
}

fn param_binds(p: &oxc_ast::ast::FormalParameter<'_>, name: &str) -> bool {
    match &p.pattern {
        BindingPattern::BindingIdentifier(id) => id.name == name,
        _ => false,
    }
}

struct ScopeScanner<'a> {
    depth: u32,
    this: bool,
    sup: bool,
    meta: bool,
    arguments: bool,
    fn_name: Option<&'a str>,
    fn_name_referenced: bool,
}

impl<'a> Visit<'a> for ScopeScanner<'a> {
    fn visit_function(&mut self, it: &Function<'a>, flags: ScopeFlags) {
        self.depth += 1;
        oxc_ast_visit::walk::walk_function(self, it, flags);
        self.depth -= 1;
    }

    // Arrow functions do not introduce a `this`/`super`/`arguments` scope.
    // Default walk applies so signals bubble up to the enclosing function.

    fn visit_this_expression(&mut self, _it: &ThisExpression) {
        if self.depth == 0 {
            self.this = true;
        }
    }

    fn visit_super(&mut self, _it: &Super) {
        if self.depth == 0 {
            self.sup = true;
        }
    }

    fn visit_meta_property(&mut self, it: &MetaProperty<'a>) {
        if self.depth == 0 && it.meta.name == "new" && it.property.name == "target" {
            self.meta = true;
        }
    }

    fn visit_identifier_reference(&mut self, it: &IdentifierReference<'a>) {
        if self.depth == 0 && it.name == "arguments" {
            self.arguments = true;
        }
        if let Some(name) = self.fn_name
            && it.name == name
        {
            self.fn_name_referenced = true;
        }
    }

    // Skip class scope so `this`, `super`, `new.target` inside class methods/blocks
    // don't pollute the outer function scope.
    fn visit_class(&mut self, it: &oxc_ast::ast::Class<'a>) {
        self.depth += 1;
        oxc_ast_visit::walk::walk_class(self, it);
        self.depth -= 1;
    }

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
        ("foo(a => a);", None),
        ("foo((a:string) => a);", None),
        ("foo(function*() {});", None),
        ("foo(function() { this; });", None),
        (
            "foo(function bar(a:string) {});",
            Some(serde_json::json!([{ "allowNamedFunctions": true }])),
        ),
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
        ("foo(function bar() {});", None),
        ("foo(function(a:string) {});", Some(serde_json::json!([{ "allowNamedFunctions": true }]))),
        ("foo(function bar() {});", Some(serde_json::json!([{ "allowNamedFunctions": false }]))),
        ("foo(function() {});", None),
        ("foo(nativeCb || function() {});", None),
        ("foo(bar ? function() {} : function() {});", None),
        ("foo(function() { (function() { this; }); });", None),
        ("foo(function() { this; }.bind(this));", None),
        ("foo(bar || function() { this; }.bind(this));", None),
        ("foo(function() { (() => this); }.bind(this));", None),
        ("foo(function bar(a:string) { a; });", None),
        ("foo(function(a:any) { a; });", None),
        ("foo(function(arguments:any) { arguments; });", None),
        (
            "foo(function(a:string) { this; });",
            Some(serde_json::json!([{ "allowUnboundThis": false }])),
        ),
        (
            "foo(function() { (() => this); });",
            Some(serde_json::json!([{ "allowUnboundThis": false }])),
        ),
        ("qux(function(foo:string, bar:number, baz:string) { return foo * 2; })", None),
        (
            "qux(function(foo:number, bar:number, baz:number) { return foo * bar; }.bind(this))",
            None,
        ),
        ("qux(function(foo:any, bar:any, baz:any) { return foo * this.qux; }.bind(this))", None),
        ("foo(function() {}.bind(this, somethingElse))", None),
        (
            "qux(function(foo = 1, [bar = 2] = [], {qux: baz = 3} = {foo: 'bar'}) { return foo + bar; });",
            None,
        ),
        ("qux(function(baz:string, baz:string) { })", None),
        ("qux(function( /* no params */ ) { })", None),
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
        ("foo(function():string { return 'foo' });", None),
        ("test('foo', function (this: any) {});", None),
        ("test('foo', function (this: any, x: number) { x; });", None),
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
        ("foo(function bar() {});", "foo(() => {});", None),
        (
            "foo(function(a:string) {});",
            "foo((a:string) => {});",
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
        ("foo(function():string { return 'foo' });", "foo(():string => { return 'foo' });", None),
    ];

    Tester::new(PreferArrowCallback::NAME, PreferArrowCallback::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
