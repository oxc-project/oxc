use oxc_ast::{
    AstKind,
    ast::{BindingIdentifier, BindingPattern, Expression, JSXAttribute, JSXAttributeValue},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_semantic::SymbolId;
use oxc_span::Span;
use oxc_str::CompactStr;
use oxc_syntax::scope::ScopeId;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    LintContext, context::ContextHost, rule::DefaultRuleConfig, utils::is_react_component_name,
};

#[derive(Debug, Default, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct ReactPerfConfig {
    /// Controls whether native elements (lowercase-first-letter tags such as `div`)
    /// are ignored by the rule. Set to `"all"` to ignore every attribute on native
    /// elements, or to an array of attribute names to ignore only those attributes
    /// on native elements.
    native_allow_list: NativeAllowList,
}

impl ReactPerfConfig {
    pub fn native_allow_list(&self) -> &NativeAllowList {
        &self.native_allow_list
    }
}

#[derive(Debug, Default, Clone, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum NativeAllowList {
    All(AllKeyword),
    List(Vec<CompactStr>),
    #[default]
    #[serde(skip)]
    None,
}

#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum AllKeyword {
    All,
}

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

pub fn react_perf_from_configuration<T>(value: serde_json::Value) -> Result<T, serde_json::Error>
where
    T: Default + serde::de::DeserializeOwned,
{
    serde_json::from_value::<DefaultRuleConfig<T>>(value).map(DefaultRuleConfig::into_inner)
}

pub fn run_react_perf_rule<'a>(
    attr: &JSXAttribute<'a>,
    scope_id: ScopeId,
    ctx: &LintContext<'a>,
    message: &'static str,
    native_allow_list: &NativeAllowList,
    check_for_violation_on_expr: impl Fn(&Expression<'_>) -> Option<Span>,
    check_for_violation_on_ast_kind: impl Fn(
        &AstKind<'_>,
        SymbolId,
    ) -> Option<(
        /* decl */ Span,
        /* init */ Option<Span>,
    )>,
) {
    // new objects/arrays/etc created at the root scope do not get
    // re-created on each render and thus do not affect performance.
    if scope_id == ctx.scoping().root_scope_id() {
        return;
    }

    // look for JSX attributes whose values are expressions (foo={bar}) (as opposed to
    // spreads ({...foo}) or just boolean attributes) (<div foo />)
    let Some(JSXAttributeValue::ExpressionContainer(container)) = attr.value.as_ref() else {
        return;
    };
    let Some(expr) = container.expression.as_expression() else {
        return;
    };

    // skip native elements (lowercase-first-letter tags like `div`) that are
    // exempted by the `nativeAllowList` configuration.
    if !matches!(native_allow_list, NativeAllowList::None)
        && let AstKind::JSXOpeningElement(opening) = ctx.nodes().parent_kind(attr.node_id())
        && let Some(tag_name) = opening.name.get_identifier_name()
        && !is_react_component_name(&tag_name)
    {
        match native_allow_list {
            NativeAllowList::All(_) => return,
            NativeAllowList::List(names) => {
                if let Some(attr_name) = attr.name.as_identifier()
                    && names.iter().any(|n| n.as_str().eq_ignore_ascii_case(&attr_name.name))
                {
                    return;
                }
            }
            NativeAllowList::None => {}
        }
    }

    // strip parenthesis and TS type casting expressions
    let expr = expr.get_inner_expression();
    // When expr is a violation, this fn will report the appropriate
    // diagnostic and return true.
    if let Some(attr_span) = check_for_violation_on_expr(expr) {
        ctx.diagnostic(react_perf_inline_diagnostic(message, attr_span));
        return;
    }

    // check for new objects/arrays/etc declared within the render function,
    // which is effectively the same as passing a new object/array/etc
    // directly as a prop.
    let Expression::Identifier(ident) = expr else {
        return;
    };
    let Some(symbol_id) = ctx.scoping().get_reference(ident.reference_id()).symbol_id() else {
        return;
    };
    // Symbols declared at the root scope won't (or, at least, shouldn't) be
    // re-assigned inside component render functions, so we can safely
    // ignore them.
    if ctx.scoping().symbol_scope_id(symbol_id) == ctx.scoping().root_scope_id() {
        return;
    }

    let declaration_node = ctx.nodes().get_node(ctx.scoping().symbol_declaration(symbol_id));
    if let Some((decl_span, init_span)) =
        check_for_violation_on_ast_kind(&declaration_node.kind(), symbol_id)
    {
        ctx.diagnostic(react_perf_reference_diagnostic(message, ident.span, decl_span, init_span));
    }
}

pub fn should_run_react_perf(ctx: &ContextHost) -> bool {
    ctx.source_type().is_jsx()
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
    match &binding {
        BindingPattern::AssignmentPattern(assignment) => {
            match &assignment.left {
                BindingPattern::BindingIdentifier(id) => {
                    // look for `x = {}`, or recurse if lhs is a binding pattern
                    if id.symbol_id() == symbol_id {
                        return Some((id.as_ref(), &assignment.right));
                    }
                    None
                }
                BindingPattern::ObjectPattern(obj) => {
                    for prop in &obj.properties {
                        let maybe_initialized_binding =
                            find_initialized_binding(&prop.value, symbol_id);
                        if maybe_initialized_binding.is_some() {
                            return maybe_initialized_binding;
                        }
                    }
                    None
                }
                BindingPattern::ArrayPattern(arr) => {
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
                BindingPattern::AssignmentPattern(_) => None,
            }
        }
        BindingPattern::ObjectPattern(obj) => {
            for prop in &obj.properties {
                let maybe_initialized_binding = find_initialized_binding(&prop.value, symbol_id);
                if maybe_initialized_binding.is_some() {
                    return maybe_initialized_binding;
                }
            }
            None
        }
        BindingPattern::ArrayPattern(arr) => {
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
        BindingPattern::BindingIdentifier(_) => None,
    }
}
