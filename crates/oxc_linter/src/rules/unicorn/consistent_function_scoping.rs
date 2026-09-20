use std::cell::OnceCell;

use oxc_ast::{AstKind, ast::*};
use oxc_ast_visit::{Visit, walk};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{NodeId, ReferenceId, ScopeFlags, ScopeId, SymbolId};
use oxc_span::{GetSpan, Span};
use rustc_hash::{FxHashMap, FxHashSet};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AstNode,
    ast_util::{get_function_like_declaration, is_node_call_like_argument, outermost_paren_parent},
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
    utils::is_react_hook,
};

fn consistent_function_scoping(
    fn_span: Span,
    parent_scope_span: Option<Span>,
    parent_scope_kind: Option<&'static str>,
    function_name: Option<&str>,
) -> OxcDiagnostic {
    let function_label = if let Some(name) = function_name {
        format!("Function `{name}` does not capture any variables from its parent scope")
    } else {
        "This function does not use any variables from its parent scope".into()
    };

    let d = OxcDiagnostic::warn(function_label).with_help(match function_name {
        Some(name) => {
            format!("Move `{name}` to the outer scope to avoid recreating it on every call.")
        }
        None => {
            "Move this function to the outer scope to avoid recreating it on every call.".into()
        }
    });

    match parent_scope_span {
        Some(parent) => d.with_labels([
            parent.label("Outer scope where this function is defined"),
            fn_span.primary_label(if let Some(parent_scope_kind) = parent_scope_kind {
                format!(
                    "This function does not use any variables from the parent {parent_scope_kind}"
                )
            } else {
                "This function does not use any variables from here".into()
            }),
        ]),
        None => d.with_label(fn_span),
    }
}

#[derive(Debug, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct ConsistentFunctionScoping {
    /// Whether to check scoping with arrow functions.
    check_arrow_functions: bool,
}

impl Default for ConsistentFunctionScoping {
    fn default() -> Self {
        Self { check_arrow_functions: true }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow functions that are declared in a scope which does not capture
    /// any variables from the outer scope.
    ///
    /// ### Why is this bad?
    ///
    /// Moving function declarations to the highest possible scope improves
    /// readability, directly [improves performance](https://stackoverflow.com/questions/80802/does-use-of-anonymous-functions-affect-performance/81329#81329)
    /// and allows JavaScript engines to better [optimize your performance](https://ponyfoo.com/articles/javascript-performance-pitfalls-v8#optimization-limit).
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// export function doFoo(foo) {
    ///   // Does not capture anything from the scope, can be moved to the outer scope
    ///	  function doBar(bar) {
    ///	    return bar === 'bar';
    ///	  }
    ///	  return doBar;
    /// }
    ///
    /// function doFoo(foo) {
    ///   const doBar = bar => {
    ///     return bar === 'bar';
    ///   };
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// function doBar(bar) {
    ///   return bar === 'bar';
    /// }
    ///
    /// export function doFoo(foo) {
    ///   return doBar;
    /// }
    ///
    /// export function doFoo(foo) {
    ///   function doBar(bar) {
    ///     return bar === 'bar' && foo.doBar(bar);
    ///   }
    ///   return doBar;
    /// }
    /// ```
    ///
    /// ### Limitations
    ///
    /// This rule does not detect or remove extraneous code blocks inside of functions:
    ///
    /// ```js
    /// function doFoo(foo) {
    ///   {
    ///     function doBar(bar) {
    ///       return bar;
    ///     }
    ///   }
    ///
    ///   return foo;
    /// }
    /// ```
    ///
    /// It also ignores functions that contain `JSXElement` references:
    ///
    /// ```jsx
    /// function doFoo(FooComponent) {
    ///   function Bar() {
    ///     return <FooComponent/>;
    ///   }
    ///
    ///   return Bar;
    /// };
    /// ```
    ///
    /// [Immediately invoked function expressions (IIFE)](https://en.wikipedia.org/wiki/Immediately_invoked_function_expression) are ignored:
    ///
    /// ```js
    /// (function () {
    ///   function doFoo(bar) {
    ///       return bar;
    ///   }
    /// })();
    /// ```
    ConsistentFunctionScoping,
    unicorn,
    suspicious,
    pending,
    config = ConsistentFunctionScoping,
    version = "0.8.0",
    short_description = "Disallow functions that are declared in a scope which does not capture any variables from the outer scope.",
);

impl Rule for ConsistentFunctionScoping {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        DefaultRuleConfig::<Self>::from_value(value).map(DefaultRuleConfig::into_inner)
    }

    fn run_once(&self, ctx: &LintContext) {
        let mut captures = CaptureCache::default();
        // The semantic scope table is much smaller than the AST. Each function owns one scope.
        for scope_id in ctx.scoping().scope_descendants_from_root() {
            if ctx.scoping().scope_flags(scope_id).is_function() {
                let node = ctx.nodes().get_node(ctx.scoping().get_node_id(scope_id));
                self.check_function(node, ctx, &mut captures);
            }
        }
    }

    fn should_run(&self, ctx: &crate::context::ContextHost) -> bool {
        // .d.ts files are never run, so there are no perf considerations for them.
        !ctx.source_type().is_typescript_definition()
    }
}

impl ConsistentFunctionScoping {
    fn check_function<'a>(
        &self,
        node: &AstNode<'a>,
        ctx: &LintContext<'a>,
        captures: &mut CaptureCache,
    ) {
        enum FunctionLikeBody<'a, 'b> {
            Function(&'b FunctionBody<'a>),
            Arrow(&'b ArrowFunctionBody<'a>),
        }

        let (
            function_declaration_symbol_id,
            function_name,
            function_body,
            reporter_span,
            function_scope_id,
        ) =
            match node.kind() {
                AstKind::Function(function) => {
                    if function.is_typescript_syntax() {
                        return;
                    }

                    let func_scope_id = function.scope_id();
                    if let Some(parent_scope_id) = ctx.scoping().scope_parent_id(func_scope_id) {
                        // Example: const foo = function bar() {};
                        // The bar function scope id is 1. In order to ignore this rule,
                        // its parent's scope id (in this case `foo`'s scope id is 0 and is equal to root scope id)
                        // should be considered.
                        //
                        // We also allow functions declared in TS module/namespace blocks.
                        let flags = ctx.scoping().scope_flags(parent_scope_id);
                        if flags.intersects(ScopeFlags::Top | ScopeFlags::TsModuleBlock) {
                            return;
                        }
                    }

                    // NOTE: function.body will always be some here because of
                    // checks in `is_typescript_syntax`
                    let Some(function_body) = &function.body else { return };

                    if let Some(binding_ident) = get_function_like_declaration(node, ctx) {
                        (
                            binding_ident.symbol_id(),
                            Some(binding_ident.name.as_str()),
                            FunctionLikeBody::Function(function_body),
                            function.id.as_ref().map_or(
                                Span::sized(function.span.start, 8),
                                |func_binding_ident| func_binding_ident.span,
                            ),
                            func_scope_id,
                        )
                    } else if let Some(function_id) = &function.id {
                        (
                            function_id.symbol_id(),
                            Some(function_id.name.as_str()),
                            FunctionLikeBody::Function(function_body),
                            function_id.span(),
                            func_scope_id,
                        )
                    } else {
                        return;
                    }
                }
                AstKind::ArrowFunctionExpression(arrow_function) if self.check_arrow_functions => {
                    let Some(binding_ident) = get_function_like_declaration(node, ctx) else {
                        return;
                    };

                    (
                        binding_ident.symbol_id(),
                        Some(binding_ident.name.as_str()),
                        FunctionLikeBody::Arrow(&arrow_function.body),
                        binding_ident.span(),
                        arrow_function.scope_id(),
                    )
                }
                _ => return,
            };

        // if the function is declared at the root scope or in a TS
        // module/namespace block, we don't need to check anything
        let scope = ctx.scoping().symbol_scope_id(function_declaration_symbol_id);
        if ctx.scoping().scope_flags(scope).intersects(ScopeFlags::Top | ScopeFlags::TsModuleBlock)
        {
            return;
        }

        if matches!(
            outermost_paren_parent(node, ctx).map(AstNode::kind),
            Some(AstKind::ReturnStatement(_))
        ) || is_node_call_like_argument(node, ctx)
        {
            return;
        }

        if is_parent_scope_iife(node, ctx) || is_in_react_hook(node, ctx) {
            return;
        }

        // Collect parameter defaults as well as body references: either can capture the parent.
        let (function_var_references, private_references, has_lexical_capture) = {
            let mut rf = ReferencesFinder::default();
            match node.kind() {
                AstKind::Function(function) => rf.visit_formal_parameters(&function.params),
                AstKind::ArrowFunctionExpression(arrow) => {
                    rf.visit_formal_parameters(&arrow.params);
                }
                _ => unreachable!(),
            }
            match function_body {
                FunctionLikeBody::Function(body) => rf.visit_function_body(body),
                FunctionLikeBody::Arrow(body) => rf.visit_arrow_function_body(body),
            }
            (rf.references, rf.private_references, rf.has_lexical_capture)
        };

        if has_lexical_capture && matches!(node.kind(), AstKind::ArrowFunctionExpression(_)) {
            return;
        }

        let parent_scope_id = ctx.scoping().scope_parent_id(function_scope_id).unwrap();
        if captures.has_private_capture(
            &private_references,
            function_scope_id,
            parent_scope_id,
            ctx,
        ) {
            return;
        }
        let parent_scope_flags = ctx.scoping().scope_flags(parent_scope_id);
        for reference_id in function_var_references {
            let reference = ctx.scoping().get_reference(reference_id);
            let Some(symbol_id) = reference.symbol_id() else { continue };
            if ctx.scoping().symbol_flags(symbol_id).is_import()
                || captures.last_checked_in.insert(symbol_id, function_scope_id)
                    == Some(function_scope_id)
            {
                continue;
            }
            let scope_id = ctx.scoping().symbol_scope_id(symbol_id);
            if ctx.scoping().scope_is_descendant_of(function_scope_id, scope_id)
                && symbol_id != function_declaration_symbol_id
            {
                // Both class declarations and expressions have an internal self binding.
                // Moving past the class body would lose that binding.
                if matches!(
                    ctx.nodes().get_node(ctx.scoping().symbol_declaration(symbol_id)).kind(),
                    AstKind::Class(class)
                        if ctx.scoping().scope_parent_id(parent_scope_id) == Some(class.scope_id())
                ) {
                    return;
                }

                // References to more distant ancestors do not prevent moving the function out
                // of its parent function. Preserve the existing handling of block scopes.
                if parent_scope_flags.is_function()
                    // Direct eval can introduce parent bindings that static resolution cannot see.
                    && !parent_scope_flags.contains_direct_eval()
                    && scope_id != parent_scope_id
                    && !matches!(
                        ctx.nodes().get_node(ctx.scoping().symbol_declaration(symbol_id)).kind(),
                        AstKind::Function(function) if function.scope_id() == parent_scope_id
                    )
                    // Index each ancestor symbol once per file, including across candidate functions.
                    && !captures.is_referenced_in(symbol_id, parent_scope_id, ctx)
                {
                    continue;
                }
                return;
            }
        }

        let (maybe_parent_scope_span, maybe_parent_scope_type) =
            get_short_span_for_fn_scope(ctx, function_declaration_symbol_id, scope)
                .map_or((None, None), |(span, rtype)| (Some(span), Some(rtype)));

        ctx.diagnostic(consistent_function_scoping(
            reporter_span,
            maybe_parent_scope_span,
            maybe_parent_scope_type,
            function_name,
        ));
    }
}

#[derive(Default)]
struct CaptureCache {
    // Check each symbol once per candidate without clearing or reallocating a set each time.
    last_checked_in: FxHashMap<SymbolId, ScopeId>,
    reference_scopes: FxHashMap<SymbolId, FxHashSet<ScopeId>>,
    private_scopes: OnceCell<FxHashMap<NodeId, ScopeId>>,
}

impl CaptureCache {
    fn has_private_capture(
        &self,
        references: &[NodeId],
        function_scope_id: ScopeId,
        parent_scope_id: ScopeId,
        ctx: &LintContext,
    ) -> bool {
        if references.is_empty() {
            return false;
        }
        let private_scopes = self.private_scopes.get_or_init(|| private_reference_scopes(ctx));
        // A declaration must move past class bodies to reach an outer statement scope.
        let destination_scope_id = ctx
            .scoping()
            .scope_ancestors(parent_scope_id)
            .skip(1)
            .find(|&id| {
                !matches!(ctx.nodes().kind(ctx.scoping().get_node_id(id)), AstKind::Class(_))
            })
            .unwrap_or_else(|| ctx.scoping().root_scope_id());
        for reference_id in references {
            let Some(&class_scope_id) = private_scopes.get(reference_id) else {
                continue;
            };
            // Classes declared inside the candidate move together with it.
            if !ctx.scoping().scope_is_descendant_of(class_scope_id, function_scope_id)
                && destination_scope_id != class_scope_id
                && !ctx.scoping().scope_is_descendant_of(destination_scope_id, class_scope_id)
            {
                return true;
            }
        }
        false
    }

    fn is_referenced_in(
        &mut self,
        symbol_id: SymbolId,
        scope_id: ScopeId,
        ctx: &LintContext,
    ) -> bool {
        self.reference_scopes
            .entry(symbol_id)
            .or_insert_with(|| {
                ctx.scoping()
                    .get_resolved_references(symbol_id)
                    .map(|reference| ctx.nodes().get_node(reference.node_id()).scope_id())
                    .collect()
            })
            .contains(&scope_id)
    }
}

// Only needed for a candidate containing private names; most files never build this index.
fn private_reference_scopes(ctx: &LintContext) -> FxHashMap<NodeId, ScopeId> {
    // Private references use a separate semantic table from ordinary identifier references.
    let mut private_definitions = FxHashMap::default();
    for (class_id, &node_id) in ctx.classes().iter_enumerated() {
        let AstKind::Class(class) = ctx.nodes().kind(node_id) else { continue };
        for element in &ctx.classes().elements[class_id] {
            if element.is_private {
                private_definitions.insert((class_id, element.name.as_ref()), class.scope_id());
            }
        }
    }
    let mut private_reference_scopes = FxHashMap::default();
    for (class_id, _) in ctx.classes().iter_enumerated() {
        for reference in ctx.classes().iter_private_identifiers(class_id) {
            if let Some(&scope_id) = ctx
                .classes()
                .ancestors(class_id)
                .find_map(|id| private_definitions.get(&(id, reference.name.as_str())))
            {
                private_reference_scopes.insert(reference.id, scope_id);
            }
        }
    }
    private_reference_scopes
}

#[derive(Default)]
struct ReferencesFinder {
    has_lexical_capture: bool,
    references: Vec<ReferenceId>,
    private_references: Vec<NodeId>,
    lexical_scope_depth: usize,
}

impl<'a> Visit<'a> for ReferencesFinder {
    fn visit_identifier_reference(&mut self, it: &IdentifierReference<'a>) {
        self.references.push(it.reference_id());
    }

    fn visit_private_identifier(&mut self, it: &PrivateIdentifier<'a>) {
        self.private_references.push(it.node_id.get());
    }

    fn visit_jsx_element_name(&mut self, _it: &JSXElementName<'a>) {
        // Ignore references in JSX elements e.g. `Foo` in `<Foo>`.
        // No need to walk children as only references they may contain are also JSX identifiers.
    }

    fn visit_this_expression(&mut self, _: &ThisExpression) {
        if self.lexical_scope_depth == 0 {
            self.has_lexical_capture = true;
        }
    }

    fn visit_super(&mut self, _: &Super) {
        if self.lexical_scope_depth == 0 {
            self.has_lexical_capture = true;
        }
    }

    fn visit_new_target(&mut self, _: &NewTarget) {
        if self.lexical_scope_depth == 0 {
            self.has_lexical_capture = true;
        }
    }

    fn visit_property_definition(&mut self, property: &PropertyDefinition<'a>) {
        self.visit_decorators(&property.decorators);
        self.visit_property_key(&property.key);
        if let Some(annotation) = &property.type_annotation {
            self.visit_ts_type_annotation(annotation);
        }
        if let Some(value) = &property.value {
            // Computed keys capture the surrounding scope; initializers own their lexical context.
            self.lexical_scope_depth += 1;
            self.visit_expression(value);
            self.lexical_scope_depth -= 1;
        }
    }

    fn visit_accessor_property(&mut self, property: &AccessorProperty<'a>) {
        self.visit_decorators(&property.decorators);
        self.visit_property_key(&property.key);
        if let Some(annotation) = &property.type_annotation {
            self.visit_ts_type_annotation(annotation);
        }
        if let Some(value) = &property.value {
            self.lexical_scope_depth += 1;
            self.visit_expression(value);
            self.lexical_scope_depth -= 1;
        }
    }

    fn visit_static_block(&mut self, block: &StaticBlock<'a>) {
        self.lexical_scope_depth += 1;
        walk::walk_static_block(self, block);
        self.lexical_scope_depth -= 1;
    }

    fn visit_function(&mut self, func: &Function<'a>, flags: ScopeFlags) {
        self.lexical_scope_depth += 1;
        walk::walk_function(self, func, flags);
        self.lexical_scope_depth -= 1;
    }
}

fn is_parent_scope_iife<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
    if let Some(parent_node) = outermost_paren_parent(node, ctx)
        && let Some(parent_node) = outermost_paren_parent(parent_node, ctx)
        && matches!(parent_node.kind(), AstKind::Function(_) | AstKind::ArrowFunctionExpression(_))
        && let Some(call_node) = outermost_paren_parent(parent_node, ctx)
        && let AstKind::CallExpression(call) = call_node.kind()
    {
        // Check if the function is the callee (true IIFE)
        // Handle both direct calls and parenthesized calls
        let callee = &call.callee.without_parentheses();
        return callee.span().start <= parent_node.span().start
            && parent_node.span().end <= callee.span().end;
    }

    false
}

fn is_in_react_hook<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
    // Check immediate parent first, then use scope-based lookup
    // First check if the function is directly inside a React hook call
    let parent = ctx.nodes().parent_node(node.id());
    if let AstKind::CallExpression(call_expr) = parent.kind()
        && is_react_hook(&call_expr.callee)
    {
        return true;
    }

    // If not directly inside, check if we're inside a function that's inside a React hook
    let current_scope_id = match node.kind() {
        AstKind::Function(func) => func.scope_id(),
        AstKind::ArrowFunctionExpression(arrow) => arrow.scope_id(),
        _ => return false,
    };

    let scoping = ctx.scoping();

    // Check the parent scope's node (the function that contains us)
    if let Some(parent_scope_id) = scoping.scope_parent_id(current_scope_id) {
        let parent_scope_node_id = scoping.get_node_id(parent_scope_id);
        let parent_scope_node = ctx.nodes().get_node(parent_scope_node_id);

        // If the parent scope is a function, check if that function is inside a React hook
        if matches!(
            parent_scope_node.kind(),
            AstKind::Function(_) | AstKind::ArrowFunctionExpression(_)
        ) {
            let grandparent = ctx.nodes().parent_node(parent_scope_node_id);
            if let AstKind::CallExpression(call_expr) = grandparent.kind() {
                return is_react_hook(&call_expr.callee);
            }
        }
    }

    false
}

fn get_short_span_for_fn_scope(
    ctx: &LintContext<'_>,
    function_symbol_id: SymbolId,
    scope_id: ScopeId,
) -> Option<(Span, &'static str)> {
    let scoping = ctx.scoping();

    debug_assert!(!scoping.scope_flags(scope_id).contains(ScopeFlags::Top));

    let scope_id =
        match ctx.nodes().parent_kind(ctx.scoping().symbol_declaration(function_symbol_id)) {
            AstKind::AssignmentExpression(_) | AstKind::ObjectProperty(_) => {
                ctx.scoping().scope_parent_id(scope_id).unwrap_or(scope_id)
            }
            _ => scope_id,
        };

    let node_creating_parent_scope = ctx.nodes().get_node(scoping.get_node_id(scope_id));

    match node_creating_parent_scope.kind() {
        AstKind::Function(f) => f.id.as_ref().map(|id| (id.span(), "function")),
        AstKind::ArrowFunctionExpression(_) => {
            let parent = ctx.nodes().parent_kind(node_creating_parent_scope.id());
            match parent {
                AstKind::VariableDeclarator(v) => Some((v.id.span(), "arrow function")),
                AstKind::AssignmentExpression(a) => Some((a.left.span(), "arrow function")),
                _ => None,
            }
        }
        AstKind::Class(c) => c.id.as_ref().map(|id| (id.span(), "class")),
        // only cover keywords of control flow statements
        AstKind::ForInStatement(ForInStatement { span, .. })
        | AstKind::ForOfStatement(ForOfStatement { span, .. })
        | AstKind::ForStatement(ForStatement { span, .. }) => {
            Some((Span::sized(span.start, 3), "for loop"))
        }
        AstKind::TryStatement(TryStatement { span, .. }) => {
            Some((Span::sized(span.start, 3), "try statement"))
        }
        AstKind::IfStatement(IfStatement { span, .. }) => {
            Some((Span::sized(span.start, 2), "if statement"))
        }
        AstKind::DoWhileStatement(DoWhileStatement { span, .. }) => {
            Some((Span::sized(span.start, 2), "do while statement"))
        }
        AstKind::SwitchStatement(SwitchStatement { span, .. }) => {
            Some((Span::sized(span.start, 6), "switch statement"))
        }
        AstKind::WhileStatement(WhileStatement { span, .. }) => {
            Some((Span::sized(span.start, 5), "while statement"))
        }
        AstKind::CatchClause(CatchClause { span, .. }) => {
            Some((Span::sized(span.start, 5), "catch block"))
        }
        _ => None,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("function doFoo(foo) { return foo; }", None),
        ("function doFoo(foo) { return bar; }", None),
        ("const doFoo = function() {};", None),
        ("const doFoo = foo => foo;", None),
        ("foo => foo;", None),
        ("function doFoo(foo) { function doBar(bar) { return foo + bar; } return foo; }", None),
        (
            "const doFoo = function(foo) {
                function doBar(bar) {
                    return foo + bar;
                }
                return foo;
            };",
            None,
        ),
        (
            "const doFoo = function(foo) {
                const doBar = function(bar) {
                    return foo + bar;
                };
                return foo;
            };",
            None,
        ),
        (
            "function doFoo(foo) {
                const doBar = function(bar) {
                    return foo + bar;
                };
                return foo;
            }",
            None,
        ),
        (
            "function doFoo(foo) {
                function doBar(bar) {
                    return foo + bar;
                }
            }",
            None,
        ),
        (
            "function doFoo(foo = 'foo') {
                function doBar(bar) {
                    return foo + bar;
                }
            }",
            None,
        ),
        (
            "function doFoo() {
                const foo = 'foo';
                function doBar(bar) {
                    return foo + bar;
                }
                return foo;
            }",
            None,
        ),
        (
            "function doFoo(foo) {
                function doBar(bar) {
                    function doZaz(zaz) {
                        return foo + bar + zaz;
                    }
                    return bar;
                }
                return foo;
            }",
            None,
        ),
        ("for (let foo = 0; foo < 1; foo++) { function doBar(bar) { return bar + foo; } }", None),
        (
            "let foo = 0;
            function doFoo() {
                foo = 1;
                function doBar(bar) {
                    return foo + bar;
                }
                return foo;
            }",
            None,
        ),
        ("const doFoo = foo => { return foo; }", None),
        ("const doFoo = foo => bar => foo + bar;", None),
        ("const doFoo = () => { return bar => bar; } ", None),
        (
            "const doFoo = foo => {
                const doBar = bar => {
                    return foo + bar;
                }
                return foo;
            }",
            None,
        ),
        (
            "function doFoo() {
                {
                    const foo = 'foo';
                    function doBar(bar) {
                        return bar + foo;
                    }
                }
            }",
            None,
        ),
        (
            "function doFoo(foo) {
                function doBar(bar) {
                    foo.bar = bar;
                }
                function doZaz(zaz) {
                    doBar(zaz);
                }

                doZaz('zaz');
            };",
            None,
        ),
        ("function doFoo() { return function doBar() {}; }", None),
        ("function doFoo(Foo) { function doBar() { return new Foo(); } return doBar; };", None),
        ("function doFoo(FooComponent) { return <FooComponent />; } ", None),
        ("const foo = <JSX/>;", None),
        ("function foo() { function bar() { return <JSX a={foo()}/>; } }", None),
        ("function doFoo(Foo) { const doBar = () => this; return doBar(); };", None),
        ("function doFoo(Foo) { const doBar = () => () => this; return doBar(); };", None),
        ("function doFoo(Foo) { const doBar = () => () => () => this; return doBar(); };", None),
        ("useEffect(() => { function foo() {} }, []) ", None),
        ("React.useEffect(() => { function foo() {} }, [])", None),
        ("(function() { function bar() {} })();", None),
        ("(function() { function bar() {} }());", None),
        ("!function() { function bar() {} }();", None),
        ("(() => { function bar() {} })();", None),
        ("(async function() { function bar() {} })();", None),
        (" (async function * () { function bar() {} })();", None),
        ("function doFoo() { const doBar = (function(bar) { return bar; })(); }", None),
        (
            "const enrichErrors = (packageName, cliArgs, f) => async (...args) => {
                try {
                    return await f(...args);
                } catch (error) {
                    error.packageName = packageName;
                    error.cliArgs = cliArgs;
                    throw error;
                }
            };",
            None,
        ),
        (
            "export const canStepForward = ([X, Y]) => ([x, y]) => direction => {
                switch (direction) {
                    case 0:
                        return y !== 0
                    case 1:
                        return x !== X - 1
                    case 2:
                        return y !== Y - 1
                    case 3:
                        return x !== 0
                    default:
                        throw new Error('unknown direction')
                }
            }",
            None,
        ),
        (
            "
            'use strict';
            module.exports = function recordErrors(eventEmitter, stateArgument) {
                const stateVariable = stateArgument;
                function onError(error) {
                    stateVariable.inputError = error;
                }
                eventEmitter.once('error', onError);
            };",
            None,
        ),
        (
            "module.exports = function recordErrors(eventEmitter, stateArgument) {
                function onError(error) {
                    stateArgument.inputError = error;
                }
                function onError2(error) {
                    onError(error);
                }

                eventEmitter.once('error', onError2);
            };",
            None,
        ),
        (
            "function outer(stream) {
                let content;

                function inner() {
                    process.stdout.write(content);
                }

                inner();
            }",
            None,
        ),
        (
            "function outer () { const inner = () => {} }",
            Some(serde_json::json!([{ "checkArrowFunctions": false }])),
        ),
        (
            "
                type Data<T> = T extends 'error' ? Error : Record<string, unknown> | unknown[]

                type Method = 'info' | 'error'

                export function createLogger(name: string) {
                    // Two lint errors are on the next line.
                    const log = <T extends Method>(method: T) => (data: Data<T>) => {
                            try {
                                    // eslint-disable-next-line no-console
                                    console[method](JSON.stringify({ name, data }))
                            } catch (error) {
                                    console.error(error)
                            }
                    }

                    return {
                            info: log('info'),
                            error: log('error'),
                    }
                }
            ",
            None,
        ),
        (
            "test('it works', async function(assert) {
                function assertHeader(assertions) {
                    for (const [key, value] of Object.entries(assertions)) {
                        assert.strictEqual(
                            native[key],
                            value
                        );
                    }
                }

                // ...
            });",
            None,
        ),
        (
            "export function a(x: number) {
                const b = (y: number) => (z: number): number => x + y + z;
                return b(1)(2);
            }",
            None,
        ),
        // https://github.com/oxc-project/oxc/pull/4948#issuecomment-2295819822
        ("t.throws(() => receiveString(function a() {}), {})", None),
        ("function test () { t.throws(() => receiveString(function a() {}), {}) }", None),
        ("function foo() { let x = new Bar(function b() {}) }", None),
        ("module.exports = function foo() {};", None),
        ("module.exports.foo = function foo() {};", None),
        ("foo.bar.func = function foo() {};", None),
        (
            "let inner;

            function foo1() {
                inner = function() {}
            }
            function foo2() {
                inner = function() {}
            }",
            None,
        ),
        ("if(f) function f(){}", None),
        (
            "
            export namespace Foo {
                export function somePublicFn() {
                    return somePrivateFn();
                }
                function somePrivateFn() {
                    return 'private';
                }
            }
        ",
            None,
        ),
        (
            "
            export namespace Foo {
                export function somePublicFn() {
                    return private1() + private2();
                }
                const private1 = function private1() {
                    return 'private1';
                }
                const private2 = () => {
                    return 'private2';
                }
            }
        ",
            None,
        ),
        (
            "
            declare namespace Foo {
                function foo(): void;
            }
            ",
            None,
        ),
        (
            "
            declare module 'some-package' {
                function foo(): void;
            }
            ",
            None,
        ),
        ("function outer() { { let x; var inner = () => x; } return inner; }", None),
        (
            "const shared = 1;
            function outer() {
                console.log(shared);
                function inner() { return shared; }
                return inner();
            }",
            None,
        ),
        (
            "function outer(shared) {
                function inner() {
                    function nested() { return shared; }
                    return nested() + shared;
                }
                return inner();
            }",
            None,
        ),
        (
            "const shared = 1;
            function outer(local) {
                function inner(value = local) { return shared + value; }
                return inner();
            }",
            Some(serde_json::json!([{ "checkArrowFunctions": false }])),
        ),
        (
            "const shared = 1;
            function outer(local) {
                const inner = (value = local) => shared + value;
                return inner();
            }",
            None,
        ),
        (
            "const shared = 1;
            function outer(local) {
                const inner = function named({ value = local } = {}) { return shared + value; };
                return inner();
            }",
            None,
        ),
        (
            "const shared = 1;
            function outer() {
                const inner = (value = this) => shared + value;
                return inner();
            }",
            None,
        ),
        (
            "const shared = 1;
            function outer(local) {
                function inner() { return shared + shared + local; }
                return inner();
            }",
            None,
        ),
        (
            "consume(class Internal {
                method() {
                    function inner() { return Internal; }
                    return inner;
                }
            });",
            None,
        ),
        (
            "consume(class Internal {
                method() {
                    const inner = () => Internal;
                    return inner;
                }
            });",
            None,
        ),
        (
            "const shared = 1;
            class Box {
                #value;
                method() {
                    function read(box) { return shared + box.#value; }
                    return read;
                }
            }",
            None,
        ),
        (
            "const shared = 1;
            class Box {
                #value;
                method() {
                    const read = (box) => shared && #value in box;
                    return read;
                }
            }",
            None,
        ),
        (
            "const shared = 1;
            class Box {
                #value;
                method() {
                    function create() {
                        class Local {
                            read(box) { return shared + box.#value; }
                        }
                        return Local;
                    }
                    return create;
                }
            }",
            None,
        ),
        (
            "const shared = 1;
            class Derived extends Base {
                method() {
                    const inner = () => shared + super.value;
                    return inner;
                }
            }",
            None,
        ),
        (
            "const shared = 1;
            function outer() {
                const inner = () => shared + new.target;
                return inner;
            }",
            None,
        ),
        (
            "const shared = 1;
            class Derived extends Base {
                method() {
                    const inner = (value = super.value) => shared + value;
                    return inner;
                }
            }",
            None,
        ),
        (
            "const shared = 1;
            function outer() {
                const inner = () => () => shared + new.target;
                return inner;
            }",
            None,
        ),
        (
            "class External {
                method() {
                    function inner() { return External; }
                    return inner;
                }
            }",
            None,
        ),
        (
            "function outer() { const inner = () => class extends this.Base {}; return inner; }",
            None,
        ),
        (
            "function outer() { const inner = () => class { [new.target.name] = 1; }; return inner; }",
            None,
        ),
        (
            "class Base { method() { const inner = () => class { [super.key] = 1; }; return inner; } }",
            None,
        ),
        (
            "function outer(value) { const inner = () => class { field = value; }; return inner; }",
            None,
        ),
        (
            "class Box { #value; method() { const inner = () => class { field = box.#value; }; return inner; } }",
            None,
        ),
    ];

    let fail = vec![
        // start of cases that eslint-plugin-unicorn passes, but we fail.

        // declared function is inside a block statement
        (
            "function doFoo(foo) {
                {
                    function doBar(bar) {
                        return bar;
                    }
                }
                return foo;
            }",
            None,
        ),
        (
            "function doFoo(FooComponent) {
                function Bar() {
                    return <FooComponent />;
                }
                return Bar;
            };",
            None,
        ),
        (
            "function Foo() {
                function Bar () {
                    return <div />
                }
                return <div>{ Bar() }</div>
            }",
            None,
        ),
        ("function foo() { function bar() { return <JSX/>; } }", None),
        ("function doFoo(Foo) { const doBar = () => arguments; return doBar(); };", None),
        (
            "let inner;

            function outer() {
                inner = function inner() {}
            }",
            None,
        ),
        // end of cases that eslint-plugin-unicorn passes, but we fail.
        (
            "function doFoo(foo) {
                function doBar(bar) {
                    return bar;
                }
                return foo;
            }",
            None,
        ),
        (
            "function doFoo() {
                const foo = 'foo';
                function doBar(bar) {
                    return bar;
                }
                return foo;
            }",
            None,
        ),
        ("function doFoo() { function doBar(bar) { return bar; } }", None),
        ("const doFoo = function() { function doBar(bar) { return bar; } };", None),
        (
            "const doFoo = function() {
                const doBar = function(bar) {
                    return bar;
                };
            };",
            None,
        ),
        ("function doFoo() { const doBar = function(bar) { return bar; }; }", None),
        ("function doFoo() { const doBar = function(bar) { return bar; }; doBar(); }", None),
        ("const doFoo = () => { const doBar = bar => { return bar; } }", None),
        ("function doFoo(Foo) { function doBar() { return this; } return doBar(); };", None),
        (
            "function doFoo(Foo) { const doBar = () => (function() {return this})(); return doBar(); };",
            None,
        ),
        (
            "function doFoo(Foo) {
                const doBar = () => (function() {return () => this})();
                return doBar();
            };",
            None,
        ),
        (
            "function doFoo(Foo) {
                function doBar() {
                    return arguments;
                }
                return doBar();
            };",
            None,
        ),
        (
            "function doFoo(Foo) {
                const doBar = () => (function() {return arguments})();
                return doBar();
            };",
            None,
        ),
        (
            "function doFoo(foo) {
                function doBar(bar) {
                    return doBar(bar);
                }
                return foo;
            }",
            None,
        ),
        (
            "function doFoo(foo) {
                function doBar(bar) {
                    return bar;
                }
                return doBar;
            }",
            None,
        ),
        ("function doFoo() { function doBar() {} }", None),
        ("function doFoo(foo) { { { function doBar(bar) { return bar; } } } return foo; }", None),
        ("{ { function doBar(bar) { return bar; } } }", None),
        ("for (let foo = 0; foo < 1; foo++) { function doBar(bar) { return bar; } }", None),
        ("function foo() { function bar() {} }", None),
        ("function foo() { async function bar() {} }", None),
        ("function foo() { function* bar() {} }", None),
        ("function foo() { async function* bar() {} }", None),
        ("function foo() { const bar = () => {} }", None),
        // ("const doFoo = () => bar => bar;", None),
        ("function foo() { const bar = async () => {} }", None),
        ("function foo() { async function* baz() {} }", None),
        (
            "useEffect(() => {
                function foo() {
                    function bar() {
                    }
                }
            }, [])",
            None,
        ),
        (
            "(function() {
                function foo() {
                    function bar() {
                    }
                }
            })();",
            None,
        ),
        (
            "process.nextTick(() => {
                function returnsZero() {
                    return true;
                }
                process.exitCode = returnsZero();
            });",
            None,
        ),
        (
            "foo(
                // This is not an IIFE
                function() {
                    function bar() {
                    }
                },
                // This is an IIFE
                (function() {
                    function baz() {
                    }
                })(),
            )",
            None,
        ),
        (
            "// This is an IIFE
            (function() {
                function bar() {
                }
            })(
                // This is not IIFE
                function() {
                    function baz() {
                    }
                },
            )",
            None,
        ),
        (
            "function Foo() {
                const Bar = <div />
                function doBaz() {
                    return 42
                }
                return <div>{ doBaz() }</div>
            }",
            None,
        ),
        (
            "function Foo() {
                function Bar () {
                    return <div />
                }
                function doBaz() {
                    return 42
                }
                return <div>{ doBaz() }</div>
            }",
            None,
        ),
        (
            "function fn1() {
                function a() {
                    return <JSX a={b()}/>;
                }
                function b() {}
                function c() {}
            }
            function fn2() {
                function foo() {}
            }",
            None,
        ),
        (
            "const outer = () => { function inner() {} }",
            Some(serde_json::json!([{ "checkArrowFunctions": false }])),
        ),
        ("function foo() { function * bar() {} }", None),
        ("function foo() { async function * bar() {} }", None),
        // ("const doFoo = () => bar => bar;", None),
        ("function outer() { const inner = function inner() {}; }", None),
        (
            "export namespace Foo { export function outer() { const inner = function inner() {}; } }",
            None,
        ),
        (
            "jest.mock('@kbn/i18n-react', () => { return { I18nProvider: function MockI18nProvider() { }, }; });",
            None,
        ),
        (
            "import { notifications } from 'some-module';
            export const Outer = () => {
                const usesImport = () => {
                    notifications.show({ message: 'x' });
                };

                return usesImport;
            };",
            None,
        ),
        (
            "const shared = 1;
            function outer() {
                function inner() {
                    return shared;
                }
                return inner();
            }",
            Some(serde_json::json!([{ "checkArrowFunctions": false }])),
        ),
        (
            "function top(shared) {
                function outer() {
                    function inner() { return shared; }
                    return inner();
                }
                return outer();
            }",
            None,
        ),
        (
            "const shared = 1;
            function outer() {
                const inner = () => shared;
                return inner();
            }",
            None,
        ),
        (
            "const shared = 1;
            function outer() {
                const inner = function named() { return shared; };
                return inner();
            }",
            None,
        ),
        (
            "const shared = 1;
            function outer() {
                function inner(value = shared) { return shared + value; }
                return inner();
            }",
            None,
        ),
        (
            "const shared = 1;
            function first() {
                console.log(shared);
                function inner() { return shared; }
                return inner();
            }
            function second() {
                function inner() { return shared; }
                return inner();
            }
            function third() {
                function inner() { return shared; }
                return inner();
            }",
            None,
        ),
        (
            "consume(class Internal {
                method() {
                    function outer() {
                        function inner() { return Internal; }
                        return inner;
                    }
                    return outer;
                }
            });",
            None,
        ),
        (
            "const shared = 1;
            class Box {
                #value;
                method() {
                    function outer() {
                        function read(box) { return shared + box.#value; }
                        return read;
                    }
                    return outer;
                }
            }",
            None,
        ),
        (
            "const shared = 1;
            class Box {
                #value;
                method() {
                    function create() {
                        class Local {
                            #value;
                            read(box) { return shared + box.#value; }
                        }
                        return Local;
                    }
                    return create;
                }
            }",
            None,
        ),
        (
            "const shared = 1;
            function outer() {
                { consume(shared); }
                function inner() { return shared; }
            }",
            None,
        ),
        (
            "const shared = 1;
            function outer() {
                function inner() { return shared + new.target; }
                return inner;
            }",
            None,
        ),
        (
            "const shared = 1;
            function outer() {
                const inner = () => shared + (function () { return new.target; })();
                return inner;
            }",
            None,
        ),
        (
            "const shared = 1;
            function outer() {
                const inner = () => shared + ({ method() { return super.value; } }).method();
                return inner;
            }",
            None,
        ),
        (
            "function outer() { const inner = () => class { field = super.toString; }; return inner; }",
            None,
        ),
        (
            "function outer() { const inner = () => class { field = new.target; }; return inner; }",
            None,
        ),
        (
            "function outer() { const inner = () => class { static { super.toString; new.target; } }; return inner; }",
            None,
        ),
        (
            "function outer() { const inner = () => class { accessor field = super.toString; }; return inner; }",
            None,
        ),
        (
            "class External { method() { function outer() { function inner() { return External; } return inner; } return outer; } }",
            None,
        ),
    ];

    Tester::new(ConsistentFunctionScoping::NAME, ConsistentFunctionScoping::PLUGIN, pass, fail)
        .test_and_snapshot();
}

#[test]
fn test_direct_eval() {
    use crate::tester::Tester;

    let pass = vec![
        "var shared = 1;
        function outer() {
            eval('var shared = 2');
            function inner() { return shared; }
            return inner();
        }",
        "var shared = 1;
        function outer() {
            eval('var shared = 2');
            const inner = () => shared;
            return inner();
        }",
    ];
    Tester::new(
        ConsistentFunctionScoping::NAME,
        ConsistentFunctionScoping::PLUGIN,
        pass,
        Vec::<&str>::new(),
    )
    .change_rule_path_extension("cjs")
    .test();
}
