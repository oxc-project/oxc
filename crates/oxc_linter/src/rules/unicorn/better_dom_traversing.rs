use oxc_ast::{
    AstKind,
    ast::{
        CallExpression, ComputedMemberExpression, Expression, MemberExpression,
        StaticMemberExpression,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    rule::Rule,
    utils::is_node_value_not_dom_node,
};

fn first_child_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer `.firstChild` over `.childNodes[0]`.")
        .with_help("Replace `.childNodes[0]` with `.firstChild`.")
        .with_label(span)
}

fn first_element_child_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer `.firstElementChild` over `.children[0]`.")
        .with_help("Replace `.children[0]` with `.firstElementChild`.")
        .with_label(span)
}

fn query_selector_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer `.querySelector()` over positional child traversal.")
        .with_help("Use `.querySelector()` with a CSS selector to access the child element.")
        .with_label(span)
}

fn closest_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer `.closest()` over chaining `.parentElement`.")
        .with_help(
            "Use `.closest(selector)` to find an ancestor element instead of chaining `.parentElement` calls.",
        )
        .with_label(span)
}

fn merge_query_selector_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer merging chained `.querySelector()` calls.")
        .with_help("Combine the selectors into a single `.querySelector()` call.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct BetterDomTraversing;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefer better DOM traversal APIs over less ideal ones.
    ///
    /// ### Why is this bad?
    ///
    /// Using DOM-specific properties and methods leads to clearer, more maintainable code.
    /// For example, `.firstChild` is more readable than `.childNodes[0]`, and `.closest()`
    /// is more robust than chaining `.parentElement`.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// element.childNodes[0];
    /// element.children[0];
    /// element.children[5];
    /// element.parentElement.parentElement;
    /// element.querySelector("a").querySelector("b");
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// element.firstChild;
    /// element.firstElementChild;
    /// element.querySelector(':scope > :nth-child(6)');
    /// element.closest('.container');
    /// element.querySelector("a b");
    /// ```
    BetterDomTraversing,
    unicorn,
    style,
    conditional_suggestion,
    version = "next",
    short_description = "Prefer better DOM traversal APIs.",
);

impl Rule for BetterDomTraversing {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::ComputedMemberExpression(computed) => {
                check_child_nodes_index(computed, node, ctx);
            }
            AstKind::StaticMemberExpression(static_member) => {
                check_parent_element_chain(static_member, node, ctx);
            }
            AstKind::CallExpression(call_expr) => {
                check_query_selector_chain(call_expr, node, ctx);
            }
            _ => {}
        }
    }
}

/// Check for `.childNodes[0]` and `.children[n]` patterns.
fn check_child_nodes_index<'a>(
    computed: &ComputedMemberExpression<'a>,
    node: &AstNode<'a>,
    ctx: &LintContext<'a>,
) {
    // Skip optional chaining: `element.children?.[0]`
    if computed.optional {
        return;
    }

    // Check that the property is a numeric literal
    let Expression::NumericLiteral(numeric_lit) = &computed.expression else {
        return;
    };

    // Must be a safe integer >= 0, with no fractional part
    let value = numeric_lit.value;
    if !value.is_finite() || value < 0.0 || value.fract() != 0.0 {
        return;
    }
    #[expect(clippy::cast_sign_loss)]
    let index = value as u32;

    // The object must be a non-optional STATIC member expression
    // like `.childNodes` or `.children` (not computed like `["children"]`)
    let Some(MemberExpression::StaticMemberExpression(member)) =
        computed.object.as_member_expression()
    else {
        return;
    };

    if member.optional {
        return;
    }

    let collection_name = match member.property.name.as_str() {
        "childNodes" => "childNodes",
        "children" => "children",
        _ => return,
    };

    // Don't flag if the root object is a non-DOM value
    if is_node_value_not_dom_node(&member.object) {
        return;
    }

    // Check if this is a nested indexed DOM collection:
    // e.g. `element.children[1].children[2]` - the inner `children[2]`
    // should not be flagged by itself (it's flagged as part of the outer pattern)
    if is_nested_indexed_dom_collection(node, ctx) {
        return;
    }

    // The span from the property name (e.g. `childNodes`) to the end of `]`
    // This is what we'll replace with the better property name.
    let property_span = member.property.span;

    if collection_name == "childNodes" && index == 0 {
        if has_comments_inside(computed.span, ctx) {
            ctx.diagnostic(first_child_diagnostic(computed.span));
        } else {
            ctx.diagnostic_with_suggestion(first_child_diagnostic(computed.span), |fixer| {
                let start = property_span.start;
                let end = computed.span.end;
                fixer.replace(Span::new(start, end), "firstChild")
            });
        }
        return;
    }

    if collection_name == "children" && index == 0 {
        if has_comments_inside(computed.span, ctx) {
            ctx.diagnostic(first_element_child_diagnostic(computed.span));
        } else {
            ctx.diagnostic_with_suggestion(
                first_element_child_diagnostic(computed.span),
                |fixer| {
                    let start = property_span.start;
                    let end = computed.span.end;
                    fixer.replace(Span::new(start, end), "firstElementChild")
                },
            );
        }
        return;
    }

    if collection_name == "children" {
        ctx.diagnostic(query_selector_diagnostic(computed.span));
    }
}

/// Check if this computed member expression is a nested indexed DOM collection.
/// e.g. `element.children[1].children[2]` where `node` is the inner `children[2]`.
fn is_nested_indexed_dom_collection(node: &AstNode<'_>, ctx: &LintContext<'_>) -> bool {
    let parent_kind = ctx.nodes().parent_kind(node.id());

    // Parent should be a StaticMemberExpression with property "children"
    let AstKind::StaticMemberExpression(parent_member) = parent_kind else {
        return false;
    };

    if parent_member.property.name.as_str() != "children" {
        return false;
    }

    // Grandparent should be a ComputedMemberExpression
    let parent_id = ctx.nodes().parent_id(node.id());
    let grandparent_kind = ctx.nodes().parent_kind(parent_id);
    matches!(grandparent_kind, AstKind::ComputedMemberExpression(_))
}

/// Check for `.parentElement.parentElement` chains.
fn check_parent_element_chain<'a>(
    static_member: &StaticMemberExpression<'a>,
    node: &AstNode<'a>,
    ctx: &LintContext<'a>,
) {
    // Must have property "parentElement"
    if static_member.property.name.as_str() != "parentElement" {
        return;
    }

    // Must not be optional
    if static_member.optional {
        return;
    }

    // The object must also be a non-optional STATIC `.parentElement` member expression
    let Some(MemberExpression::StaticMemberExpression(object_member)) =
        static_member.object.as_member_expression()
    else {
        return;
    };

    if object_member.optional || object_member.property.name.as_str() != "parentElement" {
        return;
    }

    // Must be the outermost `.parentElement` in the chain
    if !is_outermost_parent_element_chain(node, ctx) {
        return;
    }

    // Root must not be a non-DOM value
    let root = get_parent_element_chain_root(static_member);
    if is_node_value_not_dom_node(root) {
        return;
    }

    ctx.diagnostic(closest_diagnostic(static_member.span));
}

/// Check if this node is the outermost `.parentElement` in a chain.
/// A node is outermost if its parent is NOT a `.parentElement` member expression
/// whose object is this node.
fn is_outermost_parent_element_chain(node: &AstNode<'_>, ctx: &LintContext<'_>) -> bool {
    let parent_kind = ctx.nodes().parent_kind(node.id());

    let AstKind::StaticMemberExpression(parent_member) = parent_kind else {
        return true;
    };

    if parent_member.property.name.as_str() != "parentElement" {
        return true;
    }

    // If the parent's object is the current node (a member expression),
    // then we're not the outermost.
    !matches!(
        parent_member.object.as_member_expression(),
        Some(MemberExpression::StaticMemberExpression(_))
    )
}

/// Get the root of a `.parentElement` chain.
fn get_parent_element_chain_root<'a>(
    mut member: &'a StaticMemberExpression<'a>,
) -> &'a Expression<'a> {
    loop {
        match &member.object {
            Expression::StaticMemberExpression(inner)
                if inner.property.name.as_str() == "parentElement" && !inner.optional =>
            {
                member = inner;
            }
            _ => return &member.object,
        }
    }
}

/// Check for chained `.querySelector()` calls.
fn check_query_selector_chain<'a>(
    call_expr: &'a CallExpression<'a>,
    node: &AstNode<'a>,
    ctx: &LintContext<'a>,
) {
    // Must be a non-optional call with exactly 1 argument
    if call_expr.optional || call_expr.arguments.len() != 1 {
        return;
    }

    // Callee must be a non-optional `.querySelector` member expression
    let Some(callee_member) = call_expr.callee.as_member_expression() else {
        return;
    };

    if callee_member.optional() || callee_member.static_property_name() != Some("querySelector") {
        return;
    }

    // Skip if this call is followed by another `.querySelector()` call
    // (only report on the outermost call in the chain)
    if is_followed_by_query_selector(node, ctx) {
        return;
    }

    // Gather the chain of querySelector calls
    let Some(chain) = gather_query_selector_chain(call_expr) else {
        return;
    };

    if has_comments_inside(call_expr.span, ctx) || !can_merge_selector_values(&chain.selectors) {
        ctx.diagnostic(merge_query_selector_diagnostic(call_expr.span));
        return;
    }

    ctx.diagnostic_with_suggestion(merge_query_selector_diagnostic(call_expr.span), |fixer| {
        fix_merge_query_selector(&chain, fixer, ctx)
    });
}

/// Check if this CallExpression node is followed by another non-optional
/// `.querySelector()` call whose argument is a static selector.
/// e.g. In `a.querySelector("b").querySelector("c")`, the inner `querySelector("b")`
/// call IS followed by a static `.querySelector("c")`.
fn is_followed_by_query_selector(node: &AstNode<'_>, ctx: &LintContext<'_>) -> bool {
    let parent_kind = ctx.nodes().parent_kind(node.id());

    // Parent should be a non-optional StaticMemberExpression with property "querySelector"
    let AstKind::StaticMemberExpression(parent_member) = parent_kind else {
        return false;
    };

    if parent_member.optional || parent_member.property.name.as_str() != "querySelector" {
        return false;
    }

    // Grandparent should be a non-optional CallExpression
    let parent_id = ctx.nodes().parent_id(node.id());
    let grandparent_kind = ctx.nodes().parent_kind(parent_id);
    let AstKind::CallExpression(grandparent_call) = grandparent_kind else {
        return false;
    };

    if grandparent_call.optional || grandparent_call.arguments.len() != 1 {
        return false;
    }

    // The following call must have a static selector argument
    grandparent_call.arguments[0]
        .as_expression()
        .is_some_and(|arg| get_static_selector_value(arg).is_some())
}

struct QuerySelectorChainData<'a> {
    /// The root expression (before the first `.querySelector()`)
    root: &'a Expression<'a>,
    /// The selectors gathered from innermost to outermost order
    selectors: Vec<String>,
    /// The quote character used in the first selector (`'` or `"`)
    quote: char,
    /// Span covering the entire chain from root to the outermost call
    span: Span,
}

/// Check if a set of selectors can be safely merged.
/// Selectors containing `,` or `:scope` cannot be merged.
fn can_merge_selector_values(selectors: &[String]) -> bool {
    selectors.iter().all(|s| !s.contains(',') && !s.contains(":scope"))
}

/// Walk up the callee.object chain to gather all querySelector calls.
/// Returns `None` if the chain doesn't have at least 2 querySelector calls,
/// or if the root is a non-DOM value.
fn gather_query_selector_chain<'a>(
    call_expr: &'a CallExpression<'a>,
) -> Option<QuerySelectorChainData<'a>> {
    let mut current: &CallExpression = call_expr;
    let mut selectors: Vec<String> = Vec::new();
    let mut quote = '\'';
    // Track the callee.object of the last successfully collected call.
    // This will be the root expression if we break mid-chain.
    let mut last_root: Option<&'a Expression<'a>> = None;

    // Walk up through querySelector calls
    loop {
        if current.optional || current.arguments.len() != 1 {
            break;
        }

        let Some(callee_member) = current.callee.as_member_expression() else {
            break;
        };

        if callee_member.optional() || callee_member.static_property_name() != Some("querySelector")
        {
            break;
        }

        // Get the selector argument
        let Some(arg) = current.arguments[0].as_expression() else {
            break;
        };

        let Some(selector_value) = get_static_selector_value(arg) else {
            break;
        };

        selectors.push(selector_value.to_string());

        // Detect quote from the outermost call's argument
        if selectors.len() == 1 {
            if let Expression::StringLiteral(lit) = arg {
                if lit.raw.is_some_and(|r| r.starts_with('"')) {
                    quote = '"';
                }
            }
        }

        // Move to the object of the member expression
        let next = callee_member.object();
        match next {
            Expression::CallExpression(next_call) => {
                last_root = Some(callee_member.object());
                current = next_call;
            }
            _ => {
                // Reached a non-call root
                if selectors.len() < 2 {
                    return None;
                }

                if is_node_value_not_dom_node(next) {
                    return None;
                }

                if matches!(next, Expression::ChainExpression(_)) {
                    return None;
                }

                let start = next.span().start;
                let end = call_expr.span.end;

                return Some(QuerySelectorChainData {
                    root: next,
                    selectors,
                    quote,
                    span: Span::new(start, end),
                });
            }
        }
    }

    // We broke out of the loop (non-static selector, optional call, etc.)
    if selectors.len() < 2 {
        return None;
    }

    // If we broke on an optional call or optional member, don't flag
    if current.optional {
        return None;
    }
    if let Some(callee) = current.callee.as_member_expression() {
        if callee.optional() {
            return None;
        }
    }

    // The root is the callee.object of the innermost successfully collected call
    let root = last_root?;

    if is_node_value_not_dom_node(root) {
        return None;
    }

    let start = root.span().start;
    let end = call_expr.span.end;

    Some(QuerySelectorChainData { root, selectors, quote, span: Span::new(start, end) })
}

/// Get the value of a static selector argument.
/// Accepts string literals and template literals without expressions.
fn get_static_selector_value<'a>(expr: &'a Expression<'a>) -> Option<&'a str> {
    match expr {
        Expression::StringLiteral(lit) => Some(lit.value.as_str()),
        Expression::TemplateLiteral(lit) if lit.expressions.is_empty() && lit.quasis.len() == 1 => {
            lit.quasis[0].value.cooked.as_deref()
        }
        _ => None,
    }
}

/// Fix: merge chained `.querySelector()` calls into one.
fn fix_merge_query_selector<'a>(
    chain: &QuerySelectorChainData<'a>,
    fixer: RuleFixer<'_, 'a>,
    _ctx: &LintContext<'a>,
) -> RuleFix {
    let root_text = fixer.source_range(chain.root.span());

    // Check if root is a document-like object (no `:scope` needed for `document`)
    let is_document = is_document_object(chain.root);

    let selector = if is_document {
        let mut reversed = chain.selectors.clone();
        reversed.reverse();
        reversed.join(" ")
    } else {
        let mut reversed = chain.selectors.clone();
        reversed.reverse();
        format!(":scope {}", reversed.join(" "))
    };

    let replacement =
        format!("{root_text}.querySelector({quote}{selector}{quote})", quote = chain.quote);

    fixer.replace(chain.span, replacement)
}

/// Check if an expression is a document-like object.
fn is_document_object(expr: &Expression<'_>) -> bool {
    match expr {
        Expression::Identifier(ident) if ident.name.as_str() == "document" => true,
        Expression::StaticMemberExpression(member) => {
            if member.property.name.as_str() != "document" {
                return false;
            }
            if let Expression::Identifier(ident) = &member.object {
                matches!(ident.name.as_str(), "globalThis" | "window")
            } else {
                false
            }
        }
        _ => false,
    }
}

/// Check if there are any comments inside the given span.
fn has_comments_inside(span: Span, ctx: &LintContext<'_>) -> bool {
    ctx.has_comments_between(span)
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "element.firstChild;",
        "element.firstElementChild;",
        r#"element.querySelector("li");"#,
        r#"element.closest("form");"#,
        "children[0];",
        "parentElement.parentElement;",
        "element.children[index];",
        r#"element.children["0"];"#,
        "element.children[-1];",
        "element.children[1.5];",
        "element.childNodes[index];",
        "element.childNodes[1];",
        r#"element["children"][0];"#,
        "element[children][0];",
        r#"element["childNodes"][0];"#,
        "element[childNodes][0];",
        r#"element["parentElement"].parentElement;"#,
        r#"element.parentElement["parentElement"];"#,
        "element?.children[0];",
        "element.children?.[0];",
        "element?.parentElement.parentElement;",
        "element.parentElement?.parentElement;",
        r#"element.querySelector?.("a").querySelector("b");"#,
        r#"element.querySelector("a")?.querySelector("b");"#,
        r#"element.querySelector?.("a").querySelector("b").querySelector("c");"#,
        r#"element.querySelector("a")?.querySelector("b").querySelector("c");"#,
        "element.querySelector();",
        r#"element.querySelector("a", root).querySelector("b");"#,
        r#"element.querySelector("a").querySelector();"#,
        r#"element.querySelector("a").querySelector("b", root);"#,
        r#"element.querySelectorAll("a").querySelector("b");"#,
        r#"element.querySelector("a").querySelectorAll("b");"#,
        r#"element.querySelector(selector).querySelector("b");"#,
        r#"element.querySelector("a").querySelector(selector);"#,
        r#"element.querySelector(`${selector}`).querySelector("b");"#,
        r#"element.querySelector("a").querySelector(`${selector}`);"#,
        r#"element.querySelector(tag`a`).querySelector("b");"#,
    ];

    let fail = vec![
        "element.childNodes[0];",
        "element.children[0];",
        "element.children[1];",
        "element.children[10];",
        "element.children[1].children[2];",
        "element.parentElement.parentElement;",
        "element.parentElement.parentElement.parentElement;",
        r#"element.querySelector("a").querySelector("b");"#,
        r#"document.querySelector("a").querySelector("b");"#,
        r#"document.body.querySelector("a").querySelector("b");"#,
        "element.querySelector('a').querySelector('b');",
        "element.querySelector(`a`).querySelector(`b`);",
        r#"element.querySelector("a").querySelector(`b`);"#,
        r#"element.querySelector("a > b").querySelector(".c");"#,
        r#"element.querySelector(".a, .b").querySelector(".c");"#,
        r#"element.querySelector(".a").querySelector(".b, .c");"#,
        r#"element.querySelector(":scope a").querySelector("b");"#,
        r#"element.querySelector("a").querySelector("b").querySelector("c");"#,
        r#"element.querySelector("a").querySelector("b").querySelector(selector);"#,
        r#"element.querySelector(selector).querySelector("b").querySelector("c");"#,
        r#"(getElement()).querySelector("a").querySelector("b");"#,
        r#"(foo || bar).querySelector("a").querySelector("b");"#,
        r#"element.querySelector("a").querySelector("b")?.querySelector("c");"#,
        r#"element.querySelector("a").querySelector("b")?.foo;"#,
        r#"element.querySelector("a").querySelector("b").foo?.querySelector("c");"#,
        r#"(element?.querySelector("a")).querySelector("b").querySelector("c");"#,
        "element.childNodes[/* comment */ 0];",
        "element.children[/* comment */ 0];",
        r#"element.querySelector(/* comment */ "a").querySelector("b");"#,
        r#"element.querySelector("a").querySelector(/* comment */ "b");"#,
        "const item = element\n                .children[0];",
        r#"const item = element
                .querySelector("a")
                .querySelector("b");"#,
    ];

    let fix = vec![
        // .childNodes[0] → .firstChild
        ("element.childNodes[0];", "element.firstChild;"),
        // .children[0] → .firstElementChild
        ("element.children[0];", "element.firstElementChild;"),
        // querySelector chain merging
        (
            r#"element.querySelector("a").querySelector("b");"#,
            r#"element.querySelector(":scope a b");"#,
        ),
        (r#"document.querySelector("a").querySelector("b");"#, r#"document.querySelector("a b");"#),
        (
            r#"document.body.querySelector("a").querySelector("b");"#,
            r#"document.body.querySelector(":scope a b");"#,
        ),
        ("element.querySelector('a').querySelector('b');", "element.querySelector(':scope a b');"),
        (
            r#"element.querySelector("a").querySelector("b").querySelector("c");"#,
            r#"element.querySelector(":scope a b c");"#,
        ),
        (
            r#"(getElement()).querySelector("a").querySelector("b");"#,
            r#"(getElement()).querySelector(":scope a b");"#,
        ),
        (
            r#"(foo || bar).querySelector("a").querySelector("b");"#,
            r#"(foo || bar).querySelector(":scope a b");"#,
        ),
        // Chained querySelector with non-static tail — only merges static part
        (
            r#"element.querySelector("a").querySelector("b").querySelector(selector);"#,
            r#"element.querySelector(":scope a b").querySelector(selector);"#,
        ),
        (
            r#"element.querySelector(selector).querySelector("b").querySelector("c");"#,
            r#"element.querySelector(selector).querySelector(":scope b c");"#,
        ),
        // Cases with comments — no fix applied
        ("element.childNodes[/* comment */ 0];", "element.childNodes[/* comment */ 0];"),
        ("element.children[/* comment */ 0];", "element.children[/* comment */ 0];"),
        (
            r#"element.querySelector(/* comment */ "a").querySelector("b");"#,
            r#"element.querySelector(/* comment */ "a").querySelector("b");"#,
        ),
        (
            r#"element.querySelector("a").querySelector(/* comment */ "b");"#,
            r#"element.querySelector("a").querySelector(/* comment */ "b");"#,
        ),
        // Selectors with commas / :scope — no merge fix
        (
            r#"element.querySelector(".a, .b").querySelector(".c");"#,
            r#"element.querySelector(".a, .b").querySelector(".c");"#,
        ),
        (
            r#"element.querySelector(".a").querySelector(".b, .c");"#,
            r#"element.querySelector(".a").querySelector(".b, .c");"#,
        ),
        (
            r#"element.querySelector(":scope a").querySelector("b");"#,
            r#"element.querySelector(":scope a").querySelector("b");"#,
        ),
        // Non-fixable (no suggestion possible):
        ("element.children[1];", "element.children[1];"),
        ("element.children[10];", "element.children[10];"),
        ("element.children[1].children[2];", "element.children[1].children[2];"),
        ("element.parentElement.parentElement;", "element.parentElement.parentElement;"),
        (
            "element.parentElement.parentElement.parentElement;",
            "element.parentElement.parentElement.parentElement;",
        ),
        // querySelector chains with optional chaining tails
        (
            r#"element.querySelector("a").querySelector("b")?.querySelector("c");"#,
            r#"element.querySelector(":scope a b")?.querySelector("c");"#,
        ),
        (
            r#"element.querySelector("a").querySelector("b")?.foo;"#,
            r#"element.querySelector(":scope a b")?.foo;"#,
        ),
        (
            r#"element.querySelector("a").querySelector("b").foo?.querySelector("c");"#,
            r#"element.querySelector(":scope a b").foo?.querySelector("c");"#,
        ),
        (
            r#"(element?.querySelector("a")).querySelector("b").querySelector("c");"#,
            r#"(element?.querySelector("a")).querySelector(":scope b c");"#,
        ),
    ];

    Tester::new(BetterDomTraversing::NAME, BetterDomTraversing::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
