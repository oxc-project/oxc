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

use crate::{AstNode, context::LintContext, rule::Rule};

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
    short_description = "Warn if an element uses an Array index in its key.",
);

fn check_jsx_element<'a>(jsx: &'a JSXElement, node: &'a AstNode, ctx: &'a LintContext) {
    let mut key_attrs = jsx
        .opening_element
        .attributes
        .iter()
        .filter_map(|attr| {
            if let JSXAttributeItem::Attribute(attr) = attr
                && let JSXAttributeName::Identifier(ident) = &attr.name
                && ident.name.as_str() == "key"
                && let Some(JSXAttributeValue::ExpressionContainer(container)) = &attr.value
            {
                return Some((attr.span, &container.expression));
            }
            None
        })
        .peekable();

    if key_attrs.peek().is_none() {
        return;
    }
    let Some(index_param_symbol_id) = find_index_param_symbol_id(node, ctx) else {
        return;
    };

    for (span, expression) in key_attrs {
        if jsx_expression_uses_index(ctx, index_param_symbol_id, expression) {
            ctx.diagnostic(no_array_index_key_diagnostic(span));
        }
    }
}

fn check_react_clone_element<'a>(
    call_expr: &'a CallExpression,
    node: &'a AstNode,
    ctx: &'a LintContext,
) {
    if !(2..=3).contains(&call_expr.arguments.len()) {
        return;
    }
    let Expression::StaticMemberExpression(member) = &call_expr.callee else {
        return;
    };
    if member.property.name != "cloneElement" {
        return;
    }
    let Expression::Identifier(object) = member.object.without_parentheses() else {
        return;
    };
    if object.name != "React" {
        return;
    }
    let Some(Argument::ObjectExpression(obj_expr)) = call_expr.arguments.get(1) else {
        return;
    };

    let mut key_props = obj_expr
        .properties
        .iter()
        .filter_map(|prop_kind| {
            let ObjectPropertyKind::ObjectProperty(prop) = prop_kind else {
                return None;
            };
            let PropertyKey::StaticIdentifier(key_ident) = &prop.key else {
                return None;
            };
            (key_ident.name.as_str() == "key").then_some((prop.span, &prop.value))
        })
        .peekable();

    if key_props.peek().is_none() {
        return;
    }
    let Some(index_param_symbol_id) = find_index_param_symbol_id(node, ctx) else {
        return;
    };

    for (span, value) in key_props {
        if expression_uses_index(ctx, index_param_symbol_id, value) {
            ctx.diagnostic(no_array_index_key_diagnostic(span));
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

            // Index is 2nd callback arg for map/forEach/…, 3rd for reduce/reduceRight.
            let index_pos = match expr.property.name.as_str() {
                "every" | "filter" | "find" | "findIndex" | "flatMap" | "forEach" | "map"
                | "some" => 1,
                "reduce" | "reduceRight" => 2,
                _ => continue,
            };
            return find_index_param_symbol_id_by_position(call_expr, index_pos);
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

impl Rule for NoArrayIndexKey {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::JSXElement(jsx) => {
                check_jsx_element(jsx, node, ctx);
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
            <Hello key={thing.id} key={index} />
          ));
        ",
        r"things.map((thing, index) => (
            React.cloneElement(thing, { key: index })
          ));
        ",
        r"things.map((thing, index) => (
            React.cloneElement(thing, { key: thing.id, key: index })
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
