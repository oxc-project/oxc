use std::fmt;

use oxc_ast::{
    ast::{
        BindingIdentifier, BindingPattern, BindingPatternKind, Expression, JSXAttributeItem,
        JSXAttributeValue,
    },
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_semantic::SymbolId;
use oxc_span::Span;

use crate::{context::ContextHost, rule::Rule, AstNode, LintContext};

fn react_perf_inline_diagnostic(message: &'static str, attr_span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(message)
        .with_help(r"simplify props or memoize props in the parent component (https://react.dev/reference/react/memo#my-component-rerenders-when-a-prop-is-an-object-or-array).")
        .with_label(attr_span)
}
fn react_perf_reference_diagnostic(
    message: &'static str,
    attr_span: Span,
    decl_span: Span,
    init_span: Option<Span>,
) -> OxcDiagnostic {
    let mut diagnostic = OxcDiagnostic::warn(message)
        .with_help(r"simplify props or memoize props in the parent component (https://react.dev/reference/react/memo#my-component-rerenders-when-a-prop-is-an-object-or-array).")
        .with_label(
            decl_span.label("The prop was declared here"),
        );

    if let Some(init_span) = init_span {
        diagnostic = diagnostic.and_label(init_span.label("And assigned a new value here"));
    }

    diagnostic.and_label(attr_span.label("And used here"))
}

pub(crate) trait ReactPerfRule: Sized + Default + fmt::Debug {
    const MESSAGE: &'static str;

    /// Check if an [`Expression`] violates a react perf rule. If it does,
    /// report the [`OxcDiagnostic`] and return `true`.
    ///
    /// [`OxcDiagnostic`]: oxc_diagnostics::OxcDiagnostic
    fn check_for_violation_on_expr(&self, expr: &Expression<'_>) -> Option<Span>;
    /// Check if a node of some [`AstKind`] violates a react perf rule. If it does,
    /// report the [`OxcDiagnostic`] and return `true`.
    ///
    /// [`OxcDiagnostic`]: oxc_diagnostics::OxcDiagnostic
    fn check_for_violation_on_ast_kind(
        &self,
        kind: &AstKind<'_>,
        symbol_id: SymbolId,
    ) -> Option<(/* decl */ Span, /* init */ Option<Span>)>;
}

impl<R> Rule for R
where
    R: ReactPerfRule,
{
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        // new objects/arrays/etc created at the root scope do not get
        // re-created on each render and thus do not affect performance.
        if node.scope_id() == ctx.scopes().root_scope_id() {
            return;
        }

        // look for JSX attributes whose values are expressions (foo={bar}) (as opposed to
        // spreads ({...foo}) or just boolean attributes) (<div foo />)
        let AstKind::JSXAttributeItem(JSXAttributeItem::Attribute(attr)) = node.kind() else {
            return;
        };
        let Some(JSXAttributeValue::ExpressionContainer(container)) = attr.value.as_ref() else {
            return;
        };
        let Some(expr) = container.expression.as_expression() else {
            return;
        };

        // strip parenthesis and TS type casting expressions
        let expr = expr.get_inner_expression();
        // When expr is a violation, this fn will report the appropriate
        // diagnostic and return true.
        if let Some(attr_span) = self.check_for_violation_on_expr(expr) {
            ctx.diagnostic(react_perf_inline_diagnostic(Self::MESSAGE, attr_span));
            return;
        }

        // check for new objects/arrays/etc declared within the render function,
        // which is effectively the same as passing a new object/array/etc
        // directly as a prop.
        let Expression::Identifier(ident) = expr else {
            return;
        };
        let Some(symbol_id) = ctx.symbols().get_reference(ident.reference_id()).symbol_id() else {
            return;
        };
        // Symbols declared at the root scope won't (or, at least, shouldn't) be
        // re-assigned inside component render functions, so we can safely
        // ignore them.
        if ctx.symbols().get_scope_id(symbol_id) == ctx.scopes().root_scope_id() {
            return;
        }

        let declaration_node = ctx.nodes().get_node(ctx.symbols().get_declaration(symbol_id));
        if let Some((decl_span, init_span)) =
            self.check_for_violation_on_ast_kind(&declaration_node.kind(), symbol_id)
        {
            ctx.diagnostic(react_perf_reference_diagnostic(
                Self::MESSAGE,
                ident.span,
                decl_span,
                init_span,
            ));
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_jsx()
    }
}

pub fn is_constructor_matching_name(callee: &Expression<'_>, name: &str) -> bool {
    let Expression::Identifier(ident) = callee else {
        return false;
    };
    ident.name == name
}

pub fn find_initialized_binding<'a, 'b>(
    binding: &'b BindingPattern<'a>,
    symbol_id: SymbolId,
) -> Option<(&'b BindingIdentifier<'a>, &'b Expression<'a>)> {
    match &binding.kind {
        BindingPatternKind::AssignmentPattern(assignment) => {
            match &assignment.left.kind {
                BindingPatternKind::BindingIdentifier(id) => {
                    // look for `x = {}`, or recurse if lhs is a binding pattern
                    if id.symbol_id() == symbol_id {
                        return Some((id.as_ref(), &assignment.right));
                    }
                    None
                }
                BindingPatternKind::ObjectPattern(obj) => {
                    for prop in &obj.properties {
                        let maybe_initialized_binding =
                            find_initialized_binding(&prop.value, symbol_id);
                        if maybe_initialized_binding.is_some() {
                            return maybe_initialized_binding;
                        }
                    }
                    None
                }
                BindingPatternKind::ArrayPattern(arr) => {
                    for el in &arr.elements {
                        let Some(el) = el else {
                            continue;
                        };
                        let maybe_initialized_binding = find_initialized_binding(el, symbol_id);
                        if maybe_initialized_binding.is_some() {
                            return maybe_initialized_binding;
                        }
                    }
                    None
                }
                // assignment patterns should not have an assignment pattern on
                // the left.
                BindingPatternKind::AssignmentPattern(_) => None,
            }
        }
        BindingPatternKind::ObjectPattern(obj) => {
            for prop in &obj.properties {
                let maybe_initialized_binding = find_initialized_binding(&prop.value, symbol_id);
                if maybe_initialized_binding.is_some() {
                    return maybe_initialized_binding;
                }
            }
            None
        }
        BindingPatternKind::ArrayPattern(arr) => {
            for el in &arr.elements {
                let Some(el) = el else {
                    continue;
                };
                let maybe_initialized_binding = find_initialized_binding(el, symbol_id);
                if maybe_initialized_binding.is_some() {
                    return maybe_initialized_binding;
                }
            }
            None
        }
        BindingPatternKind::BindingIdentifier(_) => None,
    }
}
