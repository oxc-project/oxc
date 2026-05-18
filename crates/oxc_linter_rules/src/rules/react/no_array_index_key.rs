use oxc_ast::{
    AstKind,
    ast::{
        Argument, BinaryExpression, BindingIdentifier, CallExpression, Expression,
        JSXAttributeItem, JSXAttributeName, JSXAttributeValue, JSXElement, JSXExpression,
        ObjectPropertyKind, PropertyKey,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::SymbolId;
use oxc_span::Span;

use crate::{AstNode, ast_util::is_method_call, context::LintContext, rule::Rule};

fn no_array_index_key_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Usage of Array index in keys is not allowed")
        .with_help("Use a unique data-dependent key to avoid unnecessary rerenders")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoArrayIndexKey;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Warn if an element uses an Array index in its key.
    ///
    /// ### Why is this bad?
    ///
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
    version = "0.13.0",
);

fn check_jsx_element<'a>(
    jsx: &'a JSXElement,
    node: &'a AstNode,
    ctx: &'a LintContext,
    prop_name: &'static str,
) {
    let Some(index_param_symbol_id) = find_index_param_symbol_id(node, ctx) else {
        return;
    };

    for attr in &jsx.opening_element.attributes {
        let JSXAttributeItem::Attribute(attr) = attr else {
            continue;
        };

        let JSXAttributeName::Identifier(ident) = &attr.name else {
            continue;
        };

        if ident.name.as_str() != prop_name {
            continue;
        }

        let Some(JSXAttributeValue::ExpressionContainer(container)) = &attr.value else {
            continue;
        };

        if jsx_expression_uses_index(ctx, index_param_symbol_id, &container.expression) {
            ctx.diagnostic(no_array_index_key_diagnostic(attr.span));
        }
    }
}

fn check_react_clone_element<'a>(
    call_expr: &'a CallExpression,
    node: &'a AstNode,
    ctx: &'a LintContext,
) {
    let Some(index_param_symbol_id) = find_index_param_symbol_id(node, ctx) else {
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

            if key_ident.name.as_str() == "key"
                && expression_uses_index(ctx, index_param_symbol_id, &prop.value)
            {
                ctx.diagnostic(no_array_index_key_diagnostic(prop.span));
            }
        }
    }
}

fn is_index_reference(ctx: &LintContext, symbol_id: SymbolId, expr: &Expression) -> bool {
    if let Expression::Identifier(ident) = expr.get_inner_expression()
        && ctx.scoping().get_reference(ident.reference_id()).symbol_id() == Some(symbol_id)
    {
        return true;
    }
    false
}

fn binary_expression_uses_index(
    ctx: &LintContext,
    symbol_id: SymbolId,
    bin: &BinaryExpression,
) -> bool {
    is_index_reference(ctx, symbol_id, &bin.left)
        || is_index_reference(ctx, symbol_id, &bin.right)
        || matches!(&bin.left, Expression::BinaryExpression(l) if binary_expression_uses_index(ctx, symbol_id, l))
        || matches!(&bin.right, Expression::BinaryExpression(r) if binary_expression_uses_index(ctx, symbol_id, r))
}

fn expression_uses_index(ctx: &LintContext, symbol_id: SymbolId, expr: &Expression) -> bool {
    // key={index}
    if is_index_reference(ctx, symbol_id, expr) {
        return true;
    }

    match expr {
        // key={`abc${index}`}
        Expression::TemplateLiteral(tmpl) => {
            tmpl.expressions.iter().any(|e| is_index_reference(ctx, symbol_id, e))
        }
        // key={1 + index}
        Expression::BinaryExpression(bin) => binary_expression_uses_index(ctx, symbol_id, bin),
        Expression::CallExpression(call) => {
            // key={index.toString()}
            if let Expression::StaticMemberExpression(member) = &call.callee
                && member.property.name == "toString"
                && is_index_reference(ctx, symbol_id, &member.object)
            {
                return true;
            }
            // key={String(index)}
            if let Expression::Identifier(callee) = &call.callee
                && callee.name == "String"
                && call
                    .arguments
                    .first()
                    .and_then(Argument::as_expression)
                    .is_some_and(|arg| is_index_reference(ctx, symbol_id, arg))
            {
                return true;
            }
            false
        }
        _ => false,
    }
}

fn jsx_expression_uses_index(ctx: &LintContext, symbol_id: SymbolId, expr: &JSXExpression) -> bool {
    match expr {
        JSXExpression::EmptyExpression(_) => false,
        _ => expr.as_expression().is_some_and(|e| expression_uses_index(ctx, symbol_id, e)),
    }
}

fn find_index_param_symbol_id<'a>(node: &'a AstNode, ctx: &'a LintContext) -> Option<SymbolId> {
    for ancestor in ctx.nodes().ancestors(node.id()) {
        if let AstKind::CallExpression(call_expr) = ancestor.kind() {
            let Expression::StaticMemberExpression(expr) = &call_expr.callee else {
                continue;
            };

            if SECOND_INDEX_METHODS.contains(&expr.property.name.as_str()) {
                return find_index_param_symbol_id_by_position(call_expr, 1);
            }

            if THIRD_INDEX_METHODS.contains(&expr.property.name.as_str()) {
                return find_index_param_symbol_id_by_position(call_expr, 2);
            }
        }
    }

    None
}

fn find_index_param_symbol_id_by_position(
    call_expr: &CallExpression,
    position: usize,
) -> Option<SymbolId> {
    call_expr.arguments.first().and_then(|argument| match argument {
        Argument::ArrowFunctionExpression(arrow_fn_expr) => Some(
            arrow_fn_expr
                .params
                .items
                .get(position)?
                .pattern
                .get_binding_identifier()
                .map(BindingIdentifier::symbol_id)?,
        ),
        Argument::FunctionExpression(regular_fn_expr) => Some(
            regular_fn_expr
                .params
                .items
                .get(position)?
                .pattern
                .get_binding_identifier()
                .map(BindingIdentifier::symbol_id)?,
        ),
        _ => None,
    })
}

// things[`${method_name}`]((thing, index) => (<Hello key={index} />));
const SECOND_INDEX_METHODS: [&str; 8] =
    ["every", "filter", "find", "findIndex", "flatMap", "forEach", "map", "some"];

const THIRD_INDEX_METHODS: [&str; 2] = [
    // things.reduce((collection, thing, index) => (collection.concat(<Hello key={index} />)), []);
    "reduce",
    // things.reduceRight((collection, thing, index) => (collection.concat(<Hello key={index} />)), []);
    "reduceRight",
];

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
        // https://github.com/oxc-project/oxc/issues/20939
        r"things.map((thing, index) => (
            <Hello key={getKey(thing.id, index)} />
          ));
        ",
        r"things.map((thing, index) => (
            React.cloneElement(thing, { key: getKey(thing.id, index) })
          ));
        ",
        // https://github.com/oxc-project/oxc/issues/21110
        r"things.map((thing, index) => (
            <Hello key={`${thing.type + index}`} />
          ));
        ",
        r"things.map((thing, index) => (
            React.cloneElement(thing, { key: `${thing.type + index}` })
          ));
        ",
        r"things.map((thing, index) => (
            <Hello key={`abc${String(index)}`} />
          ));
        ",
        r"things.map((thing, index) => (
            React.cloneElement(thing, { key: `abc${index.toString()}` })
          ));
        ",
    ];

    let fail = vec![
        r"things.map((thing, index) => (
            <Hello key={index} />
          ));
        ",
        r"things.map((thing, index) => (
            <Hello key={`abc${index}`} />
          ));
        ",
        r"things.map((thing, index) => (
            <Hello key={1 + index} />
          ));
        ",
        r"things.map((thing, index) => (
            <Hello thing={thing} key={index} />
          ));
        ",
        r"things.map((thing, index) => (
            React.cloneElement(thing, { key: index })
          ));
        ",
        r"things.map((thing, index) => (
            React.cloneElement(thing, {
              key: index
            })
          ));
        ",
        r"things.map((thing, index) => (
            React.cloneElement(thing, { key: `abc${index}` })
          ));
        ",
        r"things.map((thing, index) => (
            React.cloneElement(thing, { key: 1 + index })
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
        r"things.map((thing, index) => (
            <Hello key={index.toString()} />
          ));
        ",
        r"things.map((thing, index) => (
            <Hello key={String(index)} />
          ));
        ",
        r"things.map((thing, index) => (
            React.cloneElement(thing, { key: index.toString() })
          ));
        ",
        r"things.map((thing, index) => (
            React.cloneElement(thing, { key: String(index) })
          ));
        ",
    ];

    Tester::new(NoArrayIndexKey::NAME, NoArrayIndexKey::PLUGIN, pass, fail).test_and_snapshot();
}
