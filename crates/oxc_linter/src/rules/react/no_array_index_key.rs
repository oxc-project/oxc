use oxc_ast::{
    ast::{
        Argument, CallExpression, Expression, JSXAttributeItem, JSXAttributeName,
        JSXAttributeValue, JSXElement, JSXExpression, ObjectPropertyKind, PropertyKey,
    },
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{ast_util::is_method_call, context::LintContext, rule::Rule, AstNode};

fn no_array_index_key_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Usage of Array index in keys is not allowed")
        .with_help("Use a unique data-dependent key to avoid unnecessary rerenders")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoArrayIndexKey;

declare_oxc_lint!(
    /// ### What it does
    /// Warn if an element uses an Array index in its key.
    ///
    /// ### Why is this bad?
    /// It's a bad idea to use the array index since it doesn't uniquely identify your elements.
    /// In cases where the array is sorted or an element is added to the beginning of the array,
    /// the index will be changed even though the element representing that index may be the same.
    /// This results in unnecessary renders.
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
    react,
    perf,
);

fn check_jsx_element<'a>(
    jsx: &'a JSXElement,
    node: &'a AstNode,
    ctx: &'a LintContext,
    prop_name: &'static str,
) {
    let Some(index_param_name) = find_index_param_name(node, ctx) else {
        return;
    };

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

        if expr.name.as_str() == index_param_name {
            ctx.diagnostic(no_array_index_key_diagnostic(attr.span));
        }
    }
}

fn check_react_clone_element<'a>(
    call_expr: &'a CallExpression,
    node: &'a AstNode,
    ctx: &'a LintContext,
) {
    let Some(index_param_name) = find_index_param_name(node, ctx) else {
        return;
    };

    if is_method_call(call_expr, Some(&["React"]), Some(&["cloneElement"]), Some(2), Some(3)) {
        let Some(Argument::ObjectExpression(obj_expr)) = call_expr.arguments.get(1) else {
            return;
        };

        for prop_kind in &obj_expr.properties {
            let ObjectPropertyKind::ObjectProperty(prop) = prop_kind else {
                continue;
            };

            let PropertyKey::StaticIdentifier(key_ident) = &prop.key else {
                continue;
            };

            let Expression::Identifier(value_ident) = &prop.value else {
                continue;
            };

            if key_ident.name.as_str() == "key" && value_ident.name.as_str() == index_param_name {
                ctx.diagnostic(no_array_index_key_diagnostic(obj_expr.span));
            }
        }
    }
}

fn find_index_param_name<'a>(node: &'a AstNode, ctx: &'a LintContext) -> Option<&'a str> {
    for ancestor in ctx.nodes().ancestors(node.id()).skip(1) {
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
    call_expr.arguments.first().and_then(|argument| match argument {
        Argument::ArrowFunctionExpression(arrow_fn_expr) => {
            Some(arrow_fn_expr.params.items.get(position)?.pattern.get_identifier_name()?.as_str())
        }
        Argument::FunctionExpression(regular_fn_expr) => Some(
            regular_fn_expr.params.items.get(position)?.pattern.get_identifier_name()?.as_str(),
        ),
        _ => None,
    })
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
    // things.reduce((collection, thing, index) => (collection.concat(<Hello key={index} />)), []);
    "reduce",
    // things.reduceRight((collection, thing, index) => (collection.concat(<Hello key={index} />)), []);
    "reduceRight",
};

impl Rule for NoArrayIndexKey {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::JSXElement(jsx) => {
                check_jsx_element(jsx, node, ctx, "key");
            }
            AstKind::CallExpression(call_expr) => {
                check_react_clone_element(call_expr, node, ctx);
            }
            _ => (),
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
        r"things.map((thing, index) => (
            React.cloneElement(thing, { key: thing.id })
          ));
        ",
        r"things.forEach((thing, index) => {
            otherThings.push(<Hello key={thing.id} />);
          });
        ",
        r"things.filter((thing, index) => {
            otherThings.push(<Hello key={thing.id} />);
          });
        ",
        r"things.some((thing, index) => {
            otherThings.push(<Hello key={thing.id} />);
          });
        ",
        r"things.every((thing, index) => {
            otherThings.push(<Hello key={thing.id} />);
          });
        ",
        r"things.find((thing, index) => {
            otherThings.push(<Hello key={thing.id} />);
          });
        ",
        r"things.findIndex((thing, index) => {
            otherThings.push(<Hello key={thing.id} />);
          });
        ",
        r"things.flatMap((thing, index) => (
            <Hello key={thing.id} />
          ));
        ",
        r"things.reduce((collection, thing, index) => (
            collection.concat(<Hello key={thing.id} />)
          ), []);
        ",
        r"things.reduceRight((collection, thing, index) => (
            collection.concat(<Hello key={thing.id} />)
          ), []);
        ",
    ];

    let fail = vec![
        r"things.map((thing, index) => (
            <Hello key={index} />
          ));
        ",
        r"things.map((thing, index) => (
            React.cloneElement(thing, { key: index })
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
        r"things.reduce((collection, thing, index) => (
            collection.concat(<Hello key={index} />)
          ), []);
        ",
        r"things.reduceRight((collection, thing, index) => (
            collection.concat(<Hello key={index} />)
          ), []);
        ",
    ];

    Tester::new(NoArrayIndexKey::NAME, NoArrayIndexKey::PLUGIN, pass, fail).test_and_snapshot();
}
