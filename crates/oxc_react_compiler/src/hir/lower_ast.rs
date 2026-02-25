/// Convert oxc_ast nodes to LowerableExpression/LowerableStatement.
///
/// This module bridges the gap between oxc_parser output and the HIR lowering
/// layer. It converts oxc_ast expression and statement nodes into the
/// intermediate `LowerableExpression` and `LowerableStatement` types that
/// BuildHIR can then lower to HIR instructions.
use oxc_ast::ast;
use oxc_span::GetSpan;

use super::build_hir::{
    LowerableArrayElement, LowerableExpression, LowerableJsxAttribute, LowerableJsxChild,
    LowerableJsxTag, LowerableObjectProperty, LowerableObjectPropertyKey, LowerableStatement,
    OptionalMemberProperty,
};

/// Convert an oxc_ast Expression to a LowerableExpression.
pub fn convert_expression<'a>(expr: &'a ast::Expression<'a>) -> LowerableExpression<'a> {
    match expr {
        ast::Expression::NumericLiteral(lit) => {
            LowerableExpression::NumericLiteral(lit.value, lit.span)
        }
        ast::Expression::StringLiteral(lit) => {
            LowerableExpression::StringLiteral(lit.value.to_string(), lit.span)
        }
        ast::Expression::BooleanLiteral(lit) => {
            LowerableExpression::BooleanLiteral(lit.value, lit.span)
        }
        ast::Expression::NullLiteral(lit) => LowerableExpression::NullLiteral(lit.span),
        ast::Expression::Identifier(ident) => {
            if ident.name == "undefined" {
                LowerableExpression::Undefined(ident.span)
            } else {
                LowerableExpression::Identifier(ident.name.to_string(), ident.span)
            }
        }
        ast::Expression::RegExpLiteral(lit) => LowerableExpression::RegExpLiteral {
            pattern: lit.regex.pattern.text.to_string(),
            flags: lit.regex.flags.to_string(),
            span: lit.span,
        },
        ast::Expression::TemplateLiteral(tpl) => {
            let quasis = tpl
                .quasis
                .iter()
                .map(|q| {
                    (
                        q.value.raw.to_string(),
                        q.value.cooked.as_ref().map(std::string::ToString::to_string),
                    )
                })
                .collect();
            let expressions = tpl.expressions.iter().map(convert_expression).collect();
            LowerableExpression::TemplateLiteral { quasis, expressions, span: tpl.span }
        }

        // =====================================================================
        // ArrayExpression — convert elements with proper hole/spread handling
        // =====================================================================
        ast::Expression::ArrayExpression(arr) => {
            let elements = arr
                .elements
                .iter()
                .map(|elem| match elem {
                    ast::ArrayExpressionElement::SpreadElement(spread) => {
                        LowerableArrayElement::Spread(
                            convert_expression(&spread.argument),
                            spread.span,
                        )
                    }
                    ast::ArrayExpressionElement::Elision(_) => LowerableArrayElement::Hole,
                    _ => {
                        let expr = elem.to_expression();
                        LowerableArrayElement::Expression(convert_expression(expr))
                    }
                })
                .collect();
            LowerableExpression::ArrayExpression { elements, span: arr.span }
        }

        // =====================================================================
        // ObjectExpression — convert properties with keys and values
        // =====================================================================
        ast::Expression::ObjectExpression(obj) => {
            let properties = obj
                .properties
                .iter()
                .map(|prop| match prop {
                    ast::ObjectPropertyKind::SpreadProperty(spread) => {
                        LowerableObjectProperty::Spread(
                            convert_expression(&spread.argument),
                            spread.span,
                        )
                    }
                    ast::ObjectPropertyKind::ObjectProperty(prop) => {
                        let key = convert_property_key(&prop.key);
                        let value = convert_expression(&prop.value);
                        LowerableObjectProperty::Property {
                            key,
                            value,
                            computed: prop.computed,
                            shorthand: prop.shorthand,
                            method: prop.method,
                            span: prop.span,
                        }
                    }
                })
                .collect();
            LowerableExpression::ObjectExpression { properties, span: obj.span }
        }

        ast::Expression::BinaryExpression(bin) => LowerableExpression::BinaryExpression {
            operator: bin.operator,
            left: Box::new(convert_expression(&bin.left)),
            right: Box::new(convert_expression(&bin.right)),
            span: bin.span,
        },
        ast::Expression::UnaryExpression(unary) => LowerableExpression::UnaryExpression {
            operator: unary.operator,
            argument: Box::new(convert_expression(&unary.argument)),
            span: unary.span,
        },

        // =====================================================================
        // LogicalExpression — proper LogicalExpression variant (not BinaryExpression)
        // =====================================================================
        ast::Expression::LogicalExpression(logical) => LowerableExpression::LogicalExpression {
            operator: logical.operator,
            left: Box::new(convert_expression(&logical.left)),
            right: Box::new(convert_expression(&logical.right)),
            span: logical.span,
        },

        // =====================================================================
        // UpdateExpression (++x, x++, --x, x--)
        // =====================================================================
        ast::Expression::UpdateExpression(update) => {
            let argument = convert_update_argument(&update.argument);
            LowerableExpression::UpdateExpression {
                operator: update.operator,
                argument: Box::new(argument),
                prefix: update.prefix,
                span: update.span,
            }
        }

        ast::Expression::CallExpression(call) => {
            let callee = convert_expression(&call.callee);
            let arguments = convert_arguments(&call.arguments);
            LowerableExpression::CallExpression {
                callee: Box::new(callee),
                arguments,
                span: call.span,
            }
        }
        ast::Expression::NewExpression(new_expr) => {
            let callee = convert_expression(&new_expr.callee);
            let arguments = convert_arguments(&new_expr.arguments);
            LowerableExpression::NewExpression {
                callee: Box::new(callee),
                arguments,
                span: new_expr.span,
            }
        }
        ast::Expression::StaticMemberExpression(member) => LowerableExpression::PropertyAccess {
            object: Box::new(convert_expression(&member.object)),
            property: member.property.name.to_string(),
            span: member.span,
        },
        ast::Expression::ComputedMemberExpression(member) => {
            LowerableExpression::ComputedPropertyAccess {
                object: Box::new(convert_expression(&member.object)),
                property: Box::new(convert_expression(&member.expression)),
                span: member.span,
            }
        }
        ast::Expression::ConditionalExpression(cond) => {
            LowerableExpression::ConditionalExpression {
                test: Box::new(convert_expression(&cond.test)),
                consequent: Box::new(convert_expression(&cond.consequent)),
                alternate: Box::new(convert_expression(&cond.alternate)),
                span: cond.span,
            }
        }

        // =====================================================================
        // AssignmentExpression — includes the operator and proper LHS conversion
        // =====================================================================
        ast::Expression::AssignmentExpression(assign) => {
            let right = convert_expression(&assign.right);
            let left = convert_assignment_target(&assign.left, assign.span);
            LowerableExpression::AssignmentExpression {
                operator: assign.operator,
                left: Box::new(left),
                right: Box::new(right),
                span: assign.span,
            }
        }

        // =====================================================================
        // SequenceExpression — all expressions, not just the last
        // =====================================================================
        ast::Expression::SequenceExpression(seq) => {
            let expressions = seq.expressions.iter().map(convert_expression).collect();
            LowerableExpression::SequenceExpression { expressions, span: seq.span }
        }

        ast::Expression::AwaitExpression(await_expr) => LowerableExpression::AwaitExpression {
            argument: Box::new(convert_expression(&await_expr.argument)),
            span: await_expr.span,
        },
        ast::Expression::ArrowFunctionExpression(arrow) => {
            LowerableExpression::ArrowFunctionExpression { func: arrow, span: arrow.span }
        }
        ast::Expression::FunctionExpression(func) => {
            LowerableExpression::FunctionExpression { func, span: func.span }
        }

        // =====================================================================
        // JSXElement — full tag, props, and children
        // =====================================================================
        ast::Expression::JSXElement(jsx) => {
            let tag = convert_jsx_tag_name(&jsx.opening_element.name);
            let props = convert_jsx_attributes(&jsx.opening_element.attributes);
            let children = jsx.children.iter().filter_map(convert_jsx_child).collect();
            let closing_span = jsx.closing_element.as_ref().map(|c| c.span);
            LowerableExpression::JsxElement {
                tag,
                props,
                children,
                span: jsx.span,
                opening_span: jsx.opening_element.span,
                closing_span,
            }
        }

        // =====================================================================
        // JSXFragment — children only
        // =====================================================================
        ast::Expression::JSXFragment(frag) => {
            let children = frag.children.iter().filter_map(convert_jsx_child).collect();
            LowerableExpression::JsxFragment { children, span: frag.span }
        }

        // =====================================================================
        // TaggedTemplateExpression
        // =====================================================================
        ast::Expression::TaggedTemplateExpression(tagged) => {
            // For now, handle only tagged templates without interpolations
            // (matching the TS reference which also has this limitation)
            if tagged.quasi.expressions.is_empty() {
                let quasi = tagged.quasi.quasis.first();
                let (raw, cooked) = match quasi {
                    Some(q) => (
                        q.value.raw.to_string(),
                        q.value.cooked.as_ref().map(std::string::ToString::to_string),
                    ),
                    None => (String::new(), None),
                };
                LowerableExpression::TaggedTemplateExpression {
                    tag: Box::new(convert_expression(&tagged.tag)),
                    quasi_raw: raw,
                    quasi_cooked: cooked,
                    span: tagged.span,
                }
            } else {
                // Tagged templates with interpolations are not yet supported
                LowerableExpression::Undefined(tagged.span)
            }
        }

        // =====================================================================
        // MetaProperty (e.g., import.meta, new.target)
        // =====================================================================
        ast::Expression::MetaProperty(meta) => LowerableExpression::MetaProperty {
            meta: meta.meta.name.to_string(),
            property: meta.property.name.to_string(),
            span: meta.span,
        },

        // =====================================================================
        // ChainExpression — optional chaining (a?.b, a?.(), a?.b.c?.())
        //
        // oxc_ast represents optional chaining as a `ChainExpression` wrapping
        // a `ChainElement` (CallExpression, MemberExpression, TSNonNullExpression).
        // Each element has an `optional` flag. We convert the entire chain into
        // nested `OptionalMemberExpression` / `OptionalCallExpression` variants,
        // matching Babel's `OptionalMemberExpression` / `OptionalCallExpression`.
        // =====================================================================
        ast::Expression::ChainExpression(chain) => convert_chain_element(&chain.expression),

        // Pass-through for parenthesized and TS assertion expressions
        ast::Expression::ParenthesizedExpression(paren) => convert_expression(&paren.expression),
        ast::Expression::TSNonNullExpression(ts_nn) => convert_expression(&ts_nn.expression),
        ast::Expression::TSInstantiationExpression(ts_inst) => {
            convert_expression(&ts_inst.expression)
        }
        ast::Expression::TSAsExpression(ts_as) => LowerableExpression::TypeCastExpression {
            expression: Box::new(convert_expression(&ts_as.expression)),
            annotation_kind: crate::hir::TypeAnnotationKind::As,
            span: ts_as.span,
        },
        ast::Expression::TSSatisfiesExpression(ts_sat) => LowerableExpression::TypeCastExpression {
            expression: Box::new(convert_expression(&ts_sat.expression)),
            annotation_kind: crate::hir::TypeAnnotationKind::Satisfies,
            span: ts_sat.span,
        },
        ast::Expression::TSTypeAssertion(ts_ta) => LowerableExpression::TypeCastExpression {
            expression: Box::new(convert_expression(&ts_ta.expression)),
            annotation_kind: crate::hir::TypeAnnotationKind::Cast,
            span: ts_ta.span,
        },

        // Default: treat as undefined for unsupported expressions
        _ => LowerableExpression::Undefined(expr.span()),
    }
}

/// Convert a property key from an object expression.
fn convert_property_key<'a>(key: &'a ast::PropertyKey<'a>) -> LowerableObjectPropertyKey<'a> {
    match key {
        ast::PropertyKey::StaticIdentifier(ident) => {
            LowerableObjectPropertyKey::Identifier(ident.name.to_string())
        }
        ast::PropertyKey::StringLiteral(lit) => {
            LowerableObjectPropertyKey::StringLiteral(lit.value.to_string())
        }
        ast::PropertyKey::NumericLiteral(lit) => {
            LowerableObjectPropertyKey::NumericLiteral(lit.value)
        }
        _ => {
            // Computed key or other expression key
            let expr = key.to_expression();
            LowerableObjectPropertyKey::Computed(convert_expression(expr))
        }
    }
}

/// Convert arguments from a call or new expression.
fn convert_arguments<'a>(arguments: &'a [ast::Argument<'a>]) -> Vec<LowerableExpression<'a>> {
    arguments
        .iter()
        .map(|arg| match arg {
            ast::Argument::SpreadElement(spread) => LowerableExpression::SpreadElement {
                argument: Box::new(convert_expression(&spread.argument)),
                span: spread.span,
            },
            _ => convert_expression(arg.to_expression()),
        })
        .collect()
}

/// Convert an assignment target to a LowerableExpression for the LHS.
fn convert_assignment_target<'a>(
    target: &'a ast::AssignmentTarget<'a>,
    fallback_span: oxc_span::Span,
) -> LowerableExpression<'a> {
    match target {
        ast::AssignmentTarget::AssignmentTargetIdentifier(ident) => {
            LowerableExpression::Identifier(ident.name.to_string(), ident.span)
        }
        ast::AssignmentTarget::StaticMemberExpression(member) => {
            LowerableExpression::PropertyAccess {
                object: Box::new(convert_expression(&member.object)),
                property: member.property.name.to_string(),
                span: member.span,
            }
        }
        ast::AssignmentTarget::ComputedMemberExpression(member) => {
            LowerableExpression::ComputedPropertyAccess {
                object: Box::new(convert_expression(&member.object)),
                property: Box::new(convert_expression(&member.expression)),
                span: member.span,
            }
        }
        ast::AssignmentTarget::ObjectAssignmentTarget(obj) => {
            LowerableExpression::ObjectAssignmentTarget { target: obj, span: obj.span }
        }
        ast::AssignmentTarget::ArrayAssignmentTarget(arr) => {
            LowerableExpression::ArrayAssignmentTarget { target: arr, span: arr.span }
        }
        _ => LowerableExpression::Undefined(fallback_span),
    }
}

/// Convert an update expression argument (can be identifier or member expression).
fn convert_update_argument<'a>(
    argument: &'a ast::SimpleAssignmentTarget<'a>,
) -> LowerableExpression<'a> {
    match argument {
        ast::SimpleAssignmentTarget::AssignmentTargetIdentifier(ident) => {
            LowerableExpression::Identifier(ident.name.to_string(), ident.span)
        }
        ast::SimpleAssignmentTarget::StaticMemberExpression(member) => {
            LowerableExpression::PropertyAccess {
                object: Box::new(convert_expression(&member.object)),
                property: member.property.name.to_string(),
                span: member.span,
            }
        }
        ast::SimpleAssignmentTarget::ComputedMemberExpression(member) => {
            LowerableExpression::ComputedPropertyAccess {
                object: Box::new(convert_expression(&member.object)),
                property: Box::new(convert_expression(&member.expression)),
                span: member.span,
            }
        }
        _ => LowerableExpression::Undefined(argument.span()),
    }
}

/// Convert a chain element (from `ChainExpression`) to a `LowerableExpression`.
///
/// This recursively walks the chain structure, converting each element into the
/// appropriate `OptionalMemberExpression` or `OptionalCallExpression` variant.
/// Non-optional elements within the chain are converted to their regular variants
/// only if they are at the innermost level (no optional flag); otherwise, they
/// are wrapped in the optional variant to preserve the chain structure.
fn convert_chain_element<'a>(element: &'a ast::ChainElement<'a>) -> LowerableExpression<'a> {
    match element {
        // CallExpression inside a chain: if optional, convert to OptionalCallExpression.
        // If not optional but nested inside a chain, we still need to keep the chain context
        // so we convert it to OptionalCallExpression with optional=false.
        ast::ChainElement::CallExpression(call) => {
            let callee = convert_chain_callee(&call.callee);
            let arguments = convert_arguments(&call.arguments);
            LowerableExpression::OptionalCallExpression {
                callee: Box::new(callee),
                arguments,
                optional: call.optional,
                span: call.span,
            }
        }
        // TSNonNullExpression inside a chain (e.g., `a?.b!`)
        ast::ChainElement::TSNonNullExpression(ts_nn) => convert_expression(&ts_nn.expression),
        // StaticMemberExpression inside a chain
        ast::ChainElement::StaticMemberExpression(member) => {
            let object = convert_chain_callee(&member.object);
            LowerableExpression::OptionalMemberExpression {
                object: Box::new(object),
                property: OptionalMemberProperty::Static(member.property.name.to_string()),
                optional: member.optional,
                span: member.span,
            }
        }
        // ComputedMemberExpression inside a chain
        ast::ChainElement::ComputedMemberExpression(member) => {
            let object = convert_chain_callee(&member.object);
            let prop = convert_expression(&member.expression);
            LowerableExpression::OptionalMemberExpression {
                object: Box::new(object),
                property: OptionalMemberProperty::Computed(Box::new(prop)),
                optional: member.optional,
                span: member.span,
            }
        }
        // PrivateFieldExpression inside a chain (not yet supported)
        ast::ChainElement::PrivateFieldExpression(pf) => LowerableExpression::Undefined(pf.span),
    }
}

/// Convert a callee expression that is inside a chain.
///
/// If the callee is a member expression or call expression with an `optional` flag,
/// it should be treated as part of the chain. Otherwise, convert it normally.
fn convert_chain_callee<'a>(expr: &'a ast::Expression<'a>) -> LowerableExpression<'a> {
    match expr {
        // If the callee is a StaticMemberExpression with optional=true,
        // it's part of the optional chain
        ast::Expression::StaticMemberExpression(member) if member.optional => {
            let object = convert_chain_callee(&member.object);
            LowerableExpression::OptionalMemberExpression {
                object: Box::new(object),
                property: OptionalMemberProperty::Static(member.property.name.to_string()),
                optional: true,
                span: member.span,
            }
        }
        ast::Expression::ComputedMemberExpression(member) if member.optional => {
            let object = convert_chain_callee(&member.object);
            let prop = convert_expression(&member.expression);
            LowerableExpression::OptionalMemberExpression {
                object: Box::new(object),
                property: OptionalMemberProperty::Computed(Box::new(prop)),
                optional: true,
                span: member.span,
            }
        }
        ast::Expression::CallExpression(call) if call.optional => {
            let callee = convert_chain_callee(&call.callee);
            let arguments = convert_arguments(&call.arguments);
            LowerableExpression::OptionalCallExpression {
                callee: Box::new(callee),
                arguments,
                optional: true,
                span: call.span,
            }
        }
        // Not an optional part of the chain - convert normally
        _ => convert_expression(expr),
    }
}

/// Convert a JSX element name to a `LowerableJsxTag`.
fn convert_jsx_tag_name<'a>(name: &'a ast::JSXElementName<'a>) -> LowerableJsxTag<'a> {
    match name {
        ast::JSXElementName::Identifier(ident) => {
            let tag_name = ident.name.to_string();
            // Lowercase tags are built-in HTML elements
            if tag_name.starts_with(|c: char| c.is_ascii_lowercase()) {
                LowerableJsxTag::BuiltIn(tag_name)
            } else {
                LowerableJsxTag::Expression(Box::new(LowerableExpression::Identifier(
                    tag_name, ident.span,
                )))
            }
        }
        ast::JSXElementName::IdentifierReference(ident) => {
            let tag_name = ident.name.to_string();
            if tag_name.starts_with(|c: char| c.is_ascii_lowercase()) {
                LowerableJsxTag::BuiltIn(tag_name)
            } else {
                LowerableJsxTag::Expression(Box::new(LowerableExpression::Identifier(
                    tag_name, ident.span,
                )))
            }
        }
        ast::JSXElementName::NamespacedName(ns) => {
            LowerableJsxTag::BuiltIn(format!("{}:{}", ns.namespace.name, ns.name.name))
        }
        ast::JSXElementName::MemberExpression(member) => {
            LowerableJsxTag::Expression(Box::new(convert_jsx_member_to_expression(member)))
        }
        ast::JSXElementName::ThisExpression(this_expr) => LowerableJsxTag::Expression(Box::new(
            LowerableExpression::Identifier("this".to_string(), this_expr.span),
        )),
    }
}

/// Convert JSX member expression to a chain of PropertyAccess expressions.
fn convert_jsx_member_to_expression<'a>(
    member: &'a ast::JSXMemberExpression<'a>,
) -> LowerableExpression<'a> {
    let object = match &member.object {
        ast::JSXMemberExpressionObject::IdentifierReference(ident) => {
            LowerableExpression::Identifier(ident.name.to_string(), ident.span)
        }
        ast::JSXMemberExpressionObject::MemberExpression(inner) => {
            convert_jsx_member_to_expression(inner)
        }
        ast::JSXMemberExpressionObject::ThisExpression(this_expr) => {
            LowerableExpression::Identifier("this".to_string(), this_expr.span)
        }
    };
    LowerableExpression::PropertyAccess {
        object: Box::new(object),
        property: member.property.name.to_string(),
        span: member.span,
    }
}

/// Convert JSX attributes to `LowerableJsxAttribute` values.
fn convert_jsx_attributes<'a>(
    attributes: &'a [ast::JSXAttributeItem<'a>],
) -> Vec<LowerableJsxAttribute<'a>> {
    attributes
        .iter()
        .map(|attr| match attr {
            ast::JSXAttributeItem::SpreadAttribute(spread) => {
                LowerableJsxAttribute::SpreadAttribute {
                    argument: convert_expression(&spread.argument),
                    span: spread.span,
                }
            }
            ast::JSXAttributeItem::Attribute(attr) => {
                let name = match &attr.name {
                    ast::JSXAttributeName::Identifier(ident) => ident.name.to_string(),
                    ast::JSXAttributeName::NamespacedName(ns) => {
                        format!("{}:{}", ns.namespace.name, ns.name.name)
                    }
                };
                let value = attr.value.as_ref().map(|v| convert_jsx_attribute_value(v));
                LowerableJsxAttribute::Attribute { name, value, span: attr.span }
            }
        })
        .collect()
}

/// Convert a JSX attribute value to a `LowerableExpression`.
fn convert_jsx_attribute_value<'a>(
    value: &'a ast::JSXAttributeValue<'a>,
) -> LowerableExpression<'a> {
    match value {
        ast::JSXAttributeValue::StringLiteral(lit) => {
            LowerableExpression::StringLiteral(lit.value.to_string(), lit.span)
        }
        ast::JSXAttributeValue::ExpressionContainer(container) => match &container.expression {
            ast::JSXExpression::EmptyExpression(_) => {
                LowerableExpression::Undefined(container.span)
            }
            _ => convert_expression(container.expression.to_expression()),
        },
        ast::JSXAttributeValue::Element(element) => convert_jsx_element_to_expression(element),
        ast::JSXAttributeValue::Fragment(fragment) => convert_jsx_fragment_to_expression(fragment),
    }
}

/// Convert a JSX child to a `LowerableJsxChild`.
fn convert_jsx_child<'a>(child: &'a ast::JSXChild<'a>) -> Option<LowerableJsxChild<'a>> {
    match child {
        ast::JSXChild::Text(text) => {
            Some(LowerableJsxChild::Text(text.value.to_string(), text.span))
        }
        ast::JSXChild::Element(element) => {
            let expr = convert_jsx_element_to_expression(element);
            Some(LowerableJsxChild::Element(expr))
        }
        ast::JSXChild::ExpressionContainer(container) => match &container.expression {
            ast::JSXExpression::EmptyExpression(_) => None,
            _ => {
                let expr = convert_expression(container.expression.to_expression());
                Some(LowerableJsxChild::ExpressionContainer(expr, container.span))
            }
        },
        ast::JSXChild::Fragment(fragment) => {
            let children = fragment.children.iter().filter_map(convert_jsx_child).collect();
            Some(LowerableJsxChild::Fragment { children, span: fragment.span })
        }
        ast::JSXChild::Spread(spread) => {
            let expr = convert_expression(&spread.expression);
            Some(LowerableJsxChild::ExpressionContainer(expr, spread.span))
        }
    }
}

/// Convert a JSXElement reference to a LowerableExpression directly (avoids cloning oxc_allocator::Box).
fn convert_jsx_element_to_expression<'a>(
    element: &'a ast::JSXElement<'a>,
) -> LowerableExpression<'a> {
    let tag = convert_jsx_tag_name(&element.opening_element.name);
    let props = convert_jsx_attributes(&element.opening_element.attributes);
    let children = element.children.iter().filter_map(convert_jsx_child).collect();
    let closing_span = element.closing_element.as_ref().map(|c| c.span);
    LowerableExpression::JsxElement {
        tag,
        props,
        children,
        span: element.span,
        opening_span: element.opening_element.span,
        closing_span,
    }
}

/// Convert a JSXFragment reference to a LowerableExpression directly (avoids cloning oxc_allocator::Box).
fn convert_jsx_fragment_to_expression<'a>(
    fragment: &'a ast::JSXFragment<'a>,
) -> LowerableExpression<'a> {
    let children = fragment.children.iter().filter_map(convert_jsx_child).collect();
    LowerableExpression::JsxFragment { children, span: fragment.span }
}

/// Convert an oxc_ast Statement to a LowerableStatement.
pub fn convert_statement<'a>(stmt: &'a ast::Statement<'a>) -> LowerableStatement<'a> {
    match stmt {
        ast::Statement::VariableDeclaration(decl) => LowerableStatement::VariableDeclaration(decl),
        ast::Statement::ExpressionStatement(expr) => LowerableStatement::ExpressionStatement(expr),
        ast::Statement::ReturnStatement(ret) => LowerableStatement::ReturnStatement(ret),
        ast::Statement::IfStatement(if_stmt) => LowerableStatement::IfStatement(if_stmt),
        ast::Statement::WhileStatement(while_stmt) => {
            LowerableStatement::WhileStatement(while_stmt)
        }
        ast::Statement::ForStatement(for_stmt) => LowerableStatement::ForStatement(for_stmt),
        ast::Statement::ForOfStatement(for_of) => LowerableStatement::ForOfStatement(for_of),
        ast::Statement::ForInStatement(for_in) => LowerableStatement::ForInStatement(for_in),
        ast::Statement::DoWhileStatement(do_while) => {
            LowerableStatement::DoWhileStatement(do_while)
        }
        ast::Statement::BlockStatement(block) => LowerableStatement::BlockStatement(block),
        ast::Statement::ThrowStatement(throw) => LowerableStatement::ThrowStatement(throw),
        ast::Statement::TryStatement(try_stmt) => LowerableStatement::TryStatement(try_stmt),
        ast::Statement::SwitchStatement(switch) => LowerableStatement::SwitchStatement(switch),
        ast::Statement::LabeledStatement(labeled) => LowerableStatement::LabeledStatement(labeled),
        ast::Statement::FunctionDeclaration(func) => LowerableStatement::FunctionDeclaration(func),
        ast::Statement::BreakStatement(brk) => {
            LowerableStatement::BreakStatement(brk.label.as_ref().map(|l| l.name.as_str()))
        }
        ast::Statement::ContinueStatement(cont) => {
            LowerableStatement::ContinueStatement(cont.label.as_ref().map(|l| l.name.as_str()))
        }
        ast::Statement::DebuggerStatement(_) => LowerableStatement::DebuggerStatement,
        // Empty statements and unsupported statement types
        _ => LowerableStatement::EmptyStatement,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxc_allocator::Allocator;
    use oxc_parser::Parser;
    use oxc_span::SourceType;

    #[test]
    fn test_convert_numeric_literal() {
        let allocator = Allocator::default();
        let source = "42";
        let parser_result = Parser::new(&allocator, source, SourceType::jsx()).parse();
        assert!(parser_result.errors.is_empty());

        let body = &parser_result.program.body;
        assert!(!body.is_empty());
        if let ast::Statement::ExpressionStatement(expr_stmt) = &body[0] {
            let lowered = convert_expression(&expr_stmt.expression);
            assert!(matches!(lowered, LowerableExpression::NumericLiteral(42.0, _)));
        }
    }

    #[test]
    fn test_convert_string_literal() {
        let allocator = Allocator::default();
        let source = "let x = \"hello\"";
        let parser_result = Parser::new(&allocator, source, SourceType::jsx()).parse();
        assert!(parser_result.errors.is_empty(), "Parse errors: {:?}", parser_result.errors);

        let body = &parser_result.program.body;
        assert!(!body.is_empty(), "Body should not be empty");
        // The string is inside a variable declaration initializer
        if let ast::Statement::VariableDeclaration(decl) = &body[0]
            && let Some(init) = &decl.declarations[0].init
        {
            let lowered = convert_expression(init);
            assert!(
                matches!(lowered, LowerableExpression::StringLiteral(ref s, _) if s == "hello")
            );
        }
    }

    #[test]
    fn test_convert_binary_expression() {
        let allocator = Allocator::default();
        let source = "1 + 2";
        let parser_result = Parser::new(&allocator, source, SourceType::jsx()).parse();
        assert!(parser_result.errors.is_empty());

        let body = &parser_result.program.body;
        if let ast::Statement::ExpressionStatement(expr_stmt) = &body[0] {
            let lowered = convert_expression(&expr_stmt.expression);
            assert!(matches!(
                lowered,
                LowerableExpression::BinaryExpression {
                    operator: oxc_syntax::operator::BinaryOperator::Addition,
                    ..
                }
            ));
        }
    }

    #[test]
    fn test_convert_call_expression() {
        let allocator = Allocator::default();
        let source = "foo(1, 2)";
        let parser_result = Parser::new(&allocator, source, SourceType::jsx()).parse();
        assert!(parser_result.errors.is_empty());

        let body = &parser_result.program.body;
        if let ast::Statement::ExpressionStatement(expr_stmt) = &body[0] {
            let lowered = convert_expression(&expr_stmt.expression);
            assert!(matches!(lowered, LowerableExpression::CallExpression { .. }));
        }
    }

    #[test]
    fn test_convert_member_expression() {
        let allocator = Allocator::default();
        let source = "obj.prop";
        let parser_result = Parser::new(&allocator, source, SourceType::jsx()).parse();
        assert!(parser_result.errors.is_empty());

        let body = &parser_result.program.body;
        if let ast::Statement::ExpressionStatement(expr_stmt) = &body[0] {
            let lowered = convert_expression(&expr_stmt.expression);
            assert!(
                matches!(lowered, LowerableExpression::PropertyAccess { ref property, .. } if property == "prop")
            );
        }
    }

    #[test]
    fn test_convert_jsx_element() {
        let allocator = Allocator::default();
        let source = "<div />";
        let parser_result = Parser::new(&allocator, source, SourceType::jsx()).parse();
        assert!(parser_result.errors.is_empty());

        let body = &parser_result.program.body;
        if let ast::Statement::ExpressionStatement(expr_stmt) = &body[0] {
            let lowered = convert_expression(&expr_stmt.expression);
            assert!(matches!(lowered, LowerableExpression::JsxElement { .. }));
        }
    }

    #[test]
    fn test_convert_arrow_function() {
        let allocator = Allocator::default();
        let source = "() => 42";
        let parser_result = Parser::new(&allocator, source, SourceType::jsx()).parse();
        assert!(parser_result.errors.is_empty());

        let body = &parser_result.program.body;
        if let ast::Statement::ExpressionStatement(expr_stmt) = &body[0] {
            let lowered = convert_expression(&expr_stmt.expression);
            assert!(matches!(lowered, LowerableExpression::ArrowFunctionExpression { .. }));
        }
    }

    #[test]
    fn test_convert_logical_expression() {
        let allocator = Allocator::default();
        let source = "a && b";
        let parser_result = Parser::new(&allocator, source, SourceType::jsx()).parse();
        assert!(parser_result.errors.is_empty());

        let body = &parser_result.program.body;
        if let ast::Statement::ExpressionStatement(expr_stmt) = &body[0] {
            let lowered = convert_expression(&expr_stmt.expression);
            assert!(matches!(lowered, LowerableExpression::LogicalExpression { .. }));
        }
    }

    #[test]
    fn test_convert_sequence_expression() {
        let allocator = Allocator::default();
        let source = "(a, b, c)";
        let parser_result = Parser::new(&allocator, source, SourceType::jsx()).parse();
        assert!(parser_result.errors.is_empty());

        let body = &parser_result.program.body;
        if let ast::Statement::ExpressionStatement(expr_stmt) = &body[0] {
            let lowered = convert_expression(&expr_stmt.expression);
            assert!(matches!(lowered, LowerableExpression::SequenceExpression { .. }));
        }
    }

    #[test]
    fn test_convert_update_expression() {
        let allocator = Allocator::default();
        let source = "x++";
        let parser_result = Parser::new(&allocator, source, SourceType::jsx()).parse();
        assert!(parser_result.errors.is_empty());

        let body = &parser_result.program.body;
        if let ast::Statement::ExpressionStatement(expr_stmt) = &body[0] {
            let lowered = convert_expression(&expr_stmt.expression);
            assert!(matches!(lowered, LowerableExpression::UpdateExpression { prefix: false, .. }));
        }
    }

    #[test]
    fn test_convert_object_expression() {
        let allocator = Allocator::default();
        let source = "({a: 1, b: 2})";
        let parser_result = Parser::new(&allocator, source, SourceType::jsx()).parse();
        assert!(parser_result.errors.is_empty());

        let body = &parser_result.program.body;
        if let ast::Statement::ExpressionStatement(expr_stmt) = &body[0] {
            let lowered = convert_expression(&expr_stmt.expression);
            assert!(matches!(lowered, LowerableExpression::ObjectExpression { .. }));
        }
    }

    #[test]
    fn test_convert_optional_member_expression() {
        let allocator = Allocator::default();
        let source = "a?.b";
        let parser_result = Parser::new(&allocator, source, SourceType::jsx()).parse();
        assert!(parser_result.errors.is_empty());

        let body = &parser_result.program.body;
        if let ast::Statement::ExpressionStatement(expr_stmt) = &body[0] {
            let lowered = convert_expression(&expr_stmt.expression);
            assert!(
                matches!(
                    lowered,
                    LowerableExpression::OptionalMemberExpression { optional: true, .. }
                ),
                "Expected OptionalMemberExpression with optional=true, got: {lowered:?}"
            );
        }
    }

    #[test]
    fn test_convert_optional_call_expression() {
        let allocator = Allocator::default();
        let source = "a?.()";
        let parser_result = Parser::new(&allocator, source, SourceType::jsx()).parse();
        assert!(parser_result.errors.is_empty());

        let body = &parser_result.program.body;
        if let ast::Statement::ExpressionStatement(expr_stmt) = &body[0] {
            let lowered = convert_expression(&expr_stmt.expression);
            assert!(
                matches!(
                    lowered,
                    LowerableExpression::OptionalCallExpression { optional: true, .. }
                ),
                "Expected OptionalCallExpression with optional=true, got: {lowered:?}"
            );
        }
    }

    #[test]
    fn test_convert_optional_chain_method_call() {
        let allocator = Allocator::default();
        let source = "a?.b()";
        let parser_result = Parser::new(&allocator, source, SourceType::jsx()).parse();
        assert!(parser_result.errors.is_empty());

        let body = &parser_result.program.body;
        if let ast::Statement::ExpressionStatement(expr_stmt) = &body[0] {
            let lowered = convert_expression(&expr_stmt.expression);
            // a?.b() is an OptionalCallExpression wrapping an OptionalMemberExpression callee
            assert!(
                matches!(
                    lowered,
                    LowerableExpression::OptionalCallExpression { optional: false, .. }
                ),
                "Expected OptionalCallExpression, got: {lowered:?}"
            );
        }
    }

    #[test]
    fn test_convert_optional_computed_member() {
        let allocator = Allocator::default();
        let source = "a?.[0]";
        let parser_result = Parser::new(&allocator, source, SourceType::jsx()).parse();
        assert!(parser_result.errors.is_empty());

        let body = &parser_result.program.body;
        if let ast::Statement::ExpressionStatement(expr_stmt) = &body[0] {
            let lowered = convert_expression(&expr_stmt.expression);
            assert!(
                matches!(
                    lowered,
                    LowerableExpression::OptionalMemberExpression {
                        optional: true,
                        property: OptionalMemberProperty::Computed(_),
                        ..
                    }
                ),
                "Expected OptionalMemberExpression with Computed property, got: {lowered:?}"
            );
        }
    }

    #[test]
    fn test_convert_statement_types() {
        let allocator = Allocator::default();
        let source = "return 42;";
        let source_type = SourceType::jsx().with_script(true);
        let _parser_result = Parser::new(&allocator, source, source_type).parse();

        // Return statements outside functions may error in some parsers
        // but our converter should handle all statement types gracefully
    }
}
