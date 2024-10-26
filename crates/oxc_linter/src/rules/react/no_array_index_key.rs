use oxc_ast::{
    ast::{
        Argument, CallExpression, Expression, JSXAttributeItem, JSXAttributeName,
        JSXAttributeValue, JSXElement, JSXExpression,
    },
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_array_index_key_diagnostic(span: Span) -> OxcDiagnostic {
    // See <https://oxc.rs/docs/contribute/linter/adding-rules.html#diagnostics> for details
    OxcDiagnostic::warn("Usage of Array index in keys is not allowed")
        .with_help("Should use the unique key to avoid unnecessary renders")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoArrayIndexKey;

declare_oxc_lint!(
    /// ### What it does
    ///
    ///
    /// ### Why is this bad?
    ///
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// things.map((thing, index) => (
    ///     <Hello key={index} />
    /// ));
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// things.map((thing, index) => (
    ///     <Hello key={thing.id} />
    /// ));
    /// ```
    NoArrayIndexKey,
    nursery, // TODO: change category to `correctness`, `suspicious`, `pedantic`, `perf`, `restriction`, or `style`
             // See <https://oxc.rs/docs/contribute/linter.html#rule-category> for details

    pending  // TODO: describe fix capabilities. Remove if no fix can be done,
             // keep at 'pending' if you think one could be added but don't know how.
             // Options are 'fix', 'fix_dangerous', 'suggestion', and 'conditional_fix_suggestion'
);

fn check_jsx_element<'a>(
    jsx: &'a JSXElement,
    node: &'a AstNode,
    ctx: &'a LintContext,
    prop_name: &'static str,
) {
    let index_param_name_option = find_index_param_name(node, ctx);

    for attr in &jsx.opening_element.attributes {
        let JSXAttributeItem::Attribute(attr) = attr else {
            return;
        };

        let JSXAttributeName::Identifier(ident) = &attr.name else {
            return;
        };

        if ident.name.as_str() != prop_name {
            return;
        }

        let Some(JSXAttributeValue::ExpressionContainer(container)) = &attr.value else {
            return;
        };

        let JSXExpression::Identifier(expr) = &container.expression else {
            return;
        };

        if let Some(index_param_name) = index_param_name_option {
            if expr.name.as_str() == index_param_name {
                ctx.diagnostic(no_array_index_key_diagnostic(jsx.span));
            }
        }
    }
}

fn find_index_param_name<'a>(node: &'a AstNode, ctx: &'a LintContext) -> Option<&'a str> {
    for ancestor in ctx.nodes().iter_parents(node.id()).skip(1) {
        if let AstKind::CallExpression(call_expr) = ancestor.kind() {
            let Expression::StaticMemberExpression(expr) = &call_expr.callee else {
                return None;
            };

            if SECOND_INDEX_METHODS.contains(expr.property.name.as_str()) {
                return find_index_param_name_by_position(call_expr, 1);
            }

            if THIRD_INDEX_METHODS.contains(expr.property.name.as_str()) {
                return find_index_param_name_by_position(call_expr, 2);
            }
        }
    }

    None
}

fn find_index_param_name_by_position<'a>(
    call_expr: &'a CallExpression,
    position: usize,
) -> Option<&'a str> {
    match &call_expr.arguments[0] {
        Argument::ArrowFunctionExpression(arrow_fn_expr) => {
            if let Some(index_param) = arrow_fn_expr.params.items.get(position) {
                if let Some(index_param_name) = index_param.pattern.get_identifier() {
                    return Some(index_param_name.as_str());
                }
            }
        }

        Argument::FunctionExpression(regular_fn_expr) => {
            if let Some(index_param) = regular_fn_expr.params.items.get(position) {
                if let Some(index_param_name) = index_param.pattern.get_identifier() {
                    return Some(index_param_name.as_str());
                }
            }
        }

        _ => (),
    }

    None
}

const SECOND_INDEX_METHODS: phf::Set<&'static str> = phf::phf_set! {
    // things.map((thing, index) => (<Hello key={index} />));
    "map",
    // things.forEach((thing, index) => {otherThings.push(<Hello key={index} />);});
    "forEach",
    // things.filter((thing, index) => {otherThings.push(<Hello key={index} />);});
    "filter",
    // things.some((thing, index) => {otherThings.push(<Hello key={index} />);});
    "some",
    // things.every((thing, index) => {otherThings.push(<Hello key={index} />);});
    "every",
    // things.find((thing, index) => {otherThings.push(<Hello key={index} />);});
    "find",
    // things.findIndex((thing, index) => {otherThings.push(<Hello key={index} />);});
    "findIndex",
    // things.flatMap((thing, index) => (<Hello key={index} />));
    "flatMap",
};

const THIRD_INDEX_METHODS: phf::Set<&'static str> = phf::phf_set! {
    "reduce",
};

impl Rule for NoArrayIndexKey {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::JSXElement(jsx) = node.kind() {
            check_jsx_element(jsx, node, ctx, "key");
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"things.map((thing) => (
            <Hello key={thing.id} />
          ));
        ",
    ];

    let fail = vec![
        r"things.map((thing, index) => (
            <Hello key={index} />
          ));
        ",
        r"things.forEach((thing, index) => {
            otherThings.push(<Hello key={index} />);
          });
        ",
        r"things.filter((thing, index) => {
            otherThings.push(<Hello key={index} />);
          });
        ",
        r"things.some((thing, index) => {
            otherThings.push(<Hello key={index} />);
          });
        ",
        r"things.every((thing, index) => {
            otherThings.push(<Hello key={index} />);
          });
        ",
        r"things.find((thing, index) => {
            otherThings.push(<Hello key={index} />);
          });
        ",
        r"things.findIndex((thing, index) => {
            otherThings.push(<Hello key={index} />);
          });
        ",
        r"things.flatMap((thing, index) => (
            <Hello key={index} />
          ));
        ",
    ];

    Tester::new(NoArrayIndexKey::NAME, pass, fail).test_and_snapshot();
}
