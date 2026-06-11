//! Basic expression type inference (pass B).
//!
//! Literal and object/array-literal expressions produce *fresh* types
//! ([`Type::Fresh`]), mirroring tsc: freshness gates contextual widening and
//! excess-property checks, and is stripped at bindings and relations.
//! Anything v0 cannot infer precisely returns `any`, which relates as `True`
//! and stays silent — the same no-false-positives principle as `relate`.

use oxc_ast::ast::{
    ArrayExpressionElement, BinaryOperator, Expression, ObjectPropertyKind, PropertyKind,
    UnaryOperator,
};

use oxc_span::GetSpan;

use crate::{
    diagnostics,
    ir::{Member, ObjectShape, Type, TypeId, TypeTable},
    lower::{Lowerer, property_key_name},
};

use super::{CheckResolver, FileChecker, Head, Relation, relate};

impl FileChecker<'_> {
    pub(super) fn infer(&mut self, expr: &Expression<'_>) -> TypeId {
        match expr {
            Expression::BooleanLiteral(b) => self.fresh(Type::BooleanLiteral(b.value)),
            Expression::NullLiteral(_) => TypeTable::NULL,
            Expression::NumericLiteral(n) => self.fresh(Type::NumberLiteral(n.value)),
            Expression::BigIntLiteral(b) => {
                self.fresh(Type::BigIntLiteral(b.value.as_str().into()))
            }
            Expression::StringLiteral(s) => {
                self.fresh(Type::StringLiteral(s.value.as_str().into()))
            }
            Expression::TemplateLiteral(t) => {
                if t.expressions.is_empty() && t.quasis.len() == 1 {
                    let text = t.quasis[0].value.cooked.as_deref().unwrap_or("");
                    return self.fresh(Type::StringLiteral(text.into()));
                }
                // tsc folds template expressions whose substitutions are all
                // literal-typed into a fresh string literal type.
                let mut folded = String::new();
                let mut foldable = true;
                for (i, quasi) in t.quasis.iter().enumerate() {
                    folded.push_str(quasi.value.cooked.as_deref().unwrap_or(""));
                    if let Some(sub) = t.expressions.get(i) {
                        let ty = self.infer(sub);
                        let peeled = self.peel(ty).unwrap_or(ty);
                        match self.view.get(peeled) {
                            Type::StringLiteral(s) => folded.push_str(s),
                            Type::NumberLiteral(n) => {
                                use std::fmt::Write;
                                let _ = write!(folded, "{n}");
                            }
                            Type::BooleanLiteral(b) => {
                                use std::fmt::Write;
                                let _ = write!(folded, "{b}");
                            }
                            _ => foldable = false,
                        }
                    }
                }
                if foldable {
                    self.fresh(Type::StringLiteral(folded.into_boxed_str()))
                } else {
                    TypeTable::STRING
                }
            }
            Expression::Identifier(ident) => {
                let name = ident.name.as_str();
                if name == "undefined" {
                    return TypeTable::UNDEFINED;
                }
                if self.use_flow {
                    if let Some(&flow) = self.flow_values.get(name) {
                        return flow;
                    }
                } else {
                    for scope in self.narrow_stack.iter().rev() {
                        if let Some(&narrowed) = scope.get(name) {
                            return narrowed;
                        }
                    }
                }
                self.value_names.get(name).copied().unwrap_or(TypeTable::ANY)
            }
            Expression::ArrayExpression(array) => {
                // A fresh array literal is inferred as a fresh *tuple* of its
                // element types: without full contextual typing this is the
                // representation that relates correctly against both array
                // targets (element-wise) and tuple targets (member-wise).
                let mut elements: Vec<TypeId> = Vec::new();
                for element in &array.elements {
                    match element {
                        ArrayExpressionElement::SpreadElement(spread) => {
                            self.infer(&spread.argument);
                            return self.view.push(Type::Array(TypeTable::ANY));
                        }
                        ArrayExpressionElement::Elision(_) => {
                            elements.push(TypeTable::UNDEFINED);
                        }
                        _ => {
                            let ty = self.infer(element.to_expression());
                            elements.push(ty);
                        }
                    }
                }
                self.fresh(Type::Tuple(elements.into_boxed_slice()))
            }
            Expression::ObjectExpression(object) => {
                let mut members: Vec<Member> = Vec::new();
                let mut inexact = false;
                for property in &object.properties {
                    match property {
                        ObjectPropertyKind::ObjectProperty(prop) => {
                            let name =
                                if prop.computed { None } else { property_key_name(&prop.key) };
                            let value = self.infer(&prop.value);
                            match (name, prop.kind) {
                                (Some(name), PropertyKind::Init) => {
                                    members.push(Member { name, ty: value, optional: false });
                                }
                                _ => inexact = true,
                            }
                        }
                        ObjectPropertyKind::SpreadProperty(spread) => {
                            self.infer(&spread.argument);
                            inexact = true;
                        }
                    }
                }
                self.fresh(Type::Object(ObjectShape {
                    members: members.into_boxed_slice(),
                    inexact,
                }))
            }
            Expression::TSAsExpression(as_expr) => {
                let inner = self.infer(&as_expr.expression);
                if is_const_assertion(&as_expr.type_annotation) {
                    // `as const` produces regular (non-fresh) literal types.
                    self.regular_type(inner)
                } else {
                    let target = self.lower_ts_type(&as_expr.type_annotation);
                    // TS2352: assertions require overlap (tsc compares the
                    // *widened* expression type in both directions).
                    let regular = self.regular_type(inner);
                    let widened = self.widen_literal(regular);
                    if relate(&self.view, widened, target) == Relation::False
                        && relate(&self.view, target, widened) == Relation::False
                    {
                        self.diagnostics.push(diagnostics::no_overlap_conversion(
                            &self.view,
                            widened,
                            target,
                            as_expr.span,
                        ));
                    }
                    target
                }
            }
            Expression::TSTypeAssertion(assertion) => {
                self.infer(&assertion.expression);
                self.lower_ts_type(&assertion.type_annotation)
            }
            Expression::TSSatisfiesExpression(satisfies) => {
                let source = self.infer(&satisfies.expression);
                let target = self.lower_ts_type(&satisfies.type_annotation);
                let span = self.satisfies_keyword_span(satisfies);
                self.check_assignable(
                    Some(&satisfies.expression),
                    source,
                    target,
                    span,
                    Head::Satisfies,
                );
                source
            }
            Expression::TSNonNullExpression(non_null) => {
                let inner = self.infer(&non_null.expression);
                if let Type::Union(members) = self.view.get(inner) {
                    let remaining: Vec<TypeId> = members
                        .iter()
                        .copied()
                        .filter(|m| !matches!(self.view.get(*m), Type::Null | Type::Undefined))
                        .collect();
                    return match remaining.len() {
                        0 => TypeTable::NEVER,
                        1 => remaining[0],
                        _ => self.view.push(Type::Union(remaining.into_boxed_slice())),
                    };
                }
                inner
            }
            Expression::ParenthesizedExpression(paren) => self.infer(&paren.expression),
            Expression::UnaryExpression(unary) => match unary.operator {
                UnaryOperator::UnaryNegation => {
                    if let Expression::NumericLiteral(n) = &unary.argument {
                        return self.fresh(Type::NumberLiteral(-n.value));
                    }
                    self.infer(&unary.argument);
                    TypeTable::NUMBER
                }
                UnaryOperator::UnaryPlus | UnaryOperator::BitwiseNot => {
                    self.infer(&unary.argument);
                    TypeTable::NUMBER
                }
                UnaryOperator::LogicalNot | UnaryOperator::Delete => {
                    self.infer(&unary.argument);
                    TypeTable::BOOLEAN
                }
                UnaryOperator::Typeof => {
                    self.infer(&unary.argument);
                    TypeTable::STRING
                }
                UnaryOperator::Void => {
                    self.infer(&unary.argument);
                    TypeTable::UNDEFINED
                }
            },
            Expression::BinaryExpression(binary) => {
                let left = self.infer(&binary.left);
                let right = self.infer(&binary.right);
                if is_arithmetic(binary.operator) {
                    if self.definitely_not_arithmetic(left) {
                        self.diagnostics.push(diagnostics::arithmetic_left(binary.left.span()));
                    }
                    if self.definitely_not_arithmetic(right) {
                        self.diagnostics.push(diagnostics::arithmetic_right(binary.right.span()));
                    }
                }
                match binary.operator {
                    BinaryOperator::Equality
                    | BinaryOperator::Inequality
                    | BinaryOperator::StrictEquality
                    | BinaryOperator::StrictInequality
                    | BinaryOperator::LessThan
                    | BinaryOperator::LessEqualThan
                    | BinaryOperator::GreaterThan
                    | BinaryOperator::GreaterEqualThan
                    | BinaryOperator::Instanceof
                    | BinaryOperator::In => TypeTable::BOOLEAN,
                    BinaryOperator::Subtraction
                    | BinaryOperator::Multiplication
                    | BinaryOperator::Division
                    | BinaryOperator::Remainder
                    | BinaryOperator::Exponential
                    | BinaryOperator::ShiftLeft
                    | BinaryOperator::ShiftRight
                    | BinaryOperator::ShiftRightZeroFill
                    | BinaryOperator::BitwiseAnd
                    | BinaryOperator::BitwiseOR
                    | BinaryOperator::BitwiseXOR => TypeTable::NUMBER,
                    // `+` is string-or-number.
                    BinaryOperator::Addition => TypeTable::ANY,
                }
            }
            Expression::ConditionalExpression(conditional) => {
                self.infer(&conditional.test);
                // Ternary branches are guarded by an unmodeled condition.
                let saved = self.narrow_reliable;
                self.narrow_reliable = false;
                let consequent = self.infer(&conditional.consequent);
                let consequent = self.regular_type(consequent);
                let alternate = self.infer(&conditional.alternate);
                let alternate = self.regular_type(alternate);
                self.narrow_reliable = saved;
                if consequent == alternate {
                    consequent
                } else {
                    self.view.push(Type::Union(Box::new([consequent, alternate])))
                }
            }
            Expression::LogicalExpression(logical) => {
                self.infer(&logical.left);
                self.infer(&logical.right);
                TypeTable::ANY
            }
            Expression::ArrowFunctionExpression(arrow) => {
                let resolver = CheckResolver {
                    env: self.env,
                    type_names: &self.type_names,
                    namespace_imports: &self.namespace_imports,
                    pending: std::cell::RefCell::new(Vec::new()),
                };
                let shape = Lowerer::new(&mut self.view.sink, &resolver)
                    .lower_function_shape(&arrow.params, arrow.return_type.as_deref());
                let ty = self.view.push(Type::Function(Box::new(shape)));
                if let Some(return_type) = &arrow.return_type
                    && !arrow.r#async
                {
                    let target = self.lower_ts_type(&return_type.type_annotation);
                    let shadowed = self.bind_params(&arrow.params);
                    self.check_return_body(target, &arrow.body, arrow.expression);
                    self.unbind_params(shadowed);
                }
                ty
            }
            Expression::FunctionExpression(func) => {
                let resolver = CheckResolver {
                    env: self.env,
                    type_names: &self.type_names,
                    namespace_imports: &self.namespace_imports,
                    pending: std::cell::RefCell::new(Vec::new()),
                };
                let shape = Lowerer::new(&mut self.view.sink, &resolver)
                    .lower_function_shape(&func.params, func.return_type.as_deref());
                let ty = self.view.push(Type::Function(Box::new(shape)));
                if let (Some(return_type), Some(body)) = (&func.return_type, &func.body)
                    && !func.r#async
                    && !func.generator
                {
                    let target = self.lower_ts_type(&return_type.type_annotation);
                    let shadowed = self.bind_params(&func.params);
                    self.check_return_body(target, body, false);
                    self.unbind_params(shadowed);
                }
                ty
            }
            Expression::CallExpression(call) => {
                let callee = self.infer(&call.callee);
                let shape = match self.peel(callee).map(|p| self.view.get(p)) {
                    Some(Type::Function(shape)) => Some((**shape).clone()),
                    _ => None,
                };
                let Some(shape) = shape else {
                    for argument in &call.arguments {
                        if let Some(expr) = argument.as_expression() {
                            self.infer(expr);
                        }
                    }
                    return TypeTable::ANY;
                };
                let mut failed = false;
                for (i, argument) in call.arguments.iter().enumerate() {
                    let Some(expr) = argument.as_expression() else { break };
                    let source = self.infer(expr);
                    if failed {
                        continue; // tsgo reports only the first bad argument
                    }
                    let Some(param) = shape.params.get(i) else { continue };
                    // Identifier arguments may be flow-narrowed inside function
                    // bodies; at top level straight-line flow is tracked.
                    if !self.use_flow && matches!(strip_arg(expr), Expression::Identifier(_)) {
                        continue;
                    }
                    failed = self.check_assignable(
                        Some(expr),
                        source,
                        param.ty,
                        expr.span(),
                        Head::Argument,
                    );
                }
                shape.ret
            }
            Expression::StaticMemberExpression(member) => {
                // Namespace import member access.
                if let Expression::Identifier(object) = &member.object
                    && let Some(&file_id) = self.namespace_imports.get(object.name.as_str())
                {
                    let file = self.env.file(file_id);
                    let name = member.property.name.as_str();
                    return if let Some(entry) = file.exports.get(name) {
                        self.symbol_value_type(*entry)
                    } else {
                        if !file.opaque_exports {
                            self.diagnostics.push(diagnostics::property_does_not_exist(
                                name,
                                &format!(
                                    "typeof import(\"{}\")",
                                    super::module_display_path(&file.path)
                                ),
                                member.property.span,
                            ));
                        }
                        TypeTable::ANY
                    };
                }
                let object = self.infer(&member.object);
                if member.optional {
                    return TypeTable::ANY;
                }
                let Some(peeled) = self.peel(object) else { return TypeTable::ANY };
                match self.view.get(peeled) {
                    Type::EnumValue(symbol) => {
                        let symbol = *symbol;
                        let name = member.property.name.as_str();
                        let index = match &self.env.symbol(symbol).kind {
                            crate::ir::SymbolKind::Enum { members } => {
                                members.iter().position(|m| &*m.name == name)
                            }
                            _ => None,
                        };
                        return if let Some(index) = index {
                            self.view.push(Type::EnumMember {
                                symbol,
                                index: u32::try_from(index).unwrap_or(u32::MAX),
                            })
                        } else {
                            let enum_name = self.env.symbol(symbol).name.clone();
                            self.diagnostics.push(diagnostics::property_does_not_exist(
                                name,
                                &format!("typeof {enum_name}"),
                                member.property.span,
                            ));
                            TypeTable::ANY
                        };
                    }
                    // Class statics are unmodeled.
                    Type::ClassValue(_) => return TypeTable::ANY,
                    // Union member access: the member must exist on every
                    // constituent (TS2339 when definitely absent somewhere).
                    Type::Union(constituents) => {
                        let constituents = constituents.to_vec();
                        let name = member.property.name.as_str();
                        let mut tys = Vec::new();
                        for constituent in constituents {
                            let Some(shape_id) = self.peel(constituent) else {
                                return TypeTable::ANY;
                            };
                            let Type::Object(shape) = self.view.get(shape_id) else {
                                return TypeTable::ANY;
                            };
                            if let Some(m) = shape.members.iter().find(|m| &*m.name == name) {
                                tys.push(m.ty);
                            } else {
                                // Inside unmodeled guards the union may
                                // be narrowed — only definite contexts
                                // report.
                                if shape.inexact || !(self.use_flow || self.narrow_reliable) {
                                    return TypeTable::ANY;
                                }
                                let type_string = super::print::type_to_string(&self.view, object);
                                self.diagnostics.push(diagnostics::property_does_not_exist(
                                    name,
                                    &type_string,
                                    member.property.span,
                                ));
                                return TypeTable::ANY;
                            }
                        }
                        return match tys.len() {
                            0 => TypeTable::ANY,
                            1 => tys[0],
                            _ => self.view.push(Type::Union(tys.into_boxed_slice())),
                        };
                    }
                    _ => {}
                }
                let Type::Object(shape) = self.view.get(peeled) else { return TypeTable::ANY };
                let name = member.property.name.as_str();
                if let Some(m) = shape.members.iter().find(|m| &*m.name == name) {
                    return m.ty;
                }
                if !shape.inexact {
                    let type_string = super::print::type_to_string(&self.view, object);
                    self.diagnostics.push(diagnostics::property_does_not_exist(
                        name,
                        &type_string,
                        member.property.span,
                    ));
                }
                TypeTable::ANY
            }
            Expression::NewExpression(new_expr) => {
                let callee = self.infer(&new_expr.callee);
                for argument in &new_expr.arguments {
                    if let Some(expr) = argument.as_expression() {
                        self.infer(expr);
                    }
                }
                match self.peel(callee).map(|p| self.view.get(p)) {
                    Some(Type::ClassValue(symbol)) => {
                        let symbol = *symbol;
                        self.view.push(Type::Ref {
                            target: crate::ir::RefTarget::Symbol(symbol),
                            args: Box::new([]),
                        })
                    }
                    _ => TypeTable::ANY,
                }
            }
            // await, dynamic import, tagged templates, ... need lib types and
            // signatures — `any` in v0.
            _ => TypeTable::ANY,
        }
    }

    /// Definitely invalid as an arithmetic operand (TS2362/2363). Anything
    /// possibly numeric (any/unknown/unions/enums/unresolved) stays silent.
    fn definitely_not_arithmetic(&self, ty: TypeId) -> bool {
        let Some(peeled) = self.peel(ty) else { return false };
        matches!(
            self.view.get(peeled),
            Type::String
                | Type::StringLiteral(_)
                | Type::Boolean
                | Type::BooleanLiteral(_)
                | Type::Object(_)
                | Type::Tuple(_)
                | Type::Array(_)
                | Type::Function(_)
                | Type::ObjectKeyword
                | Type::Symbol
        )
    }
}

fn is_arithmetic(op: BinaryOperator) -> bool {
    matches!(
        op,
        BinaryOperator::Subtraction
            | BinaryOperator::Multiplication
            | BinaryOperator::Division
            | BinaryOperator::Remainder
            | BinaryOperator::Exponential
            | BinaryOperator::ShiftLeft
            | BinaryOperator::ShiftRight
            | BinaryOperator::ShiftRightZeroFill
            | BinaryOperator::BitwiseAnd
            | BinaryOperator::BitwiseOR
            | BinaryOperator::BitwiseXOR
    )
}

/// Strip parens for the identifier-argument check.
fn strip_arg<'b, 'a>(expr: &'b Expression<'a>) -> &'b Expression<'a> {
    let mut expr = expr;
    while let Expression::ParenthesizedExpression(paren) = expr {
        expr = &paren.expression;
    }
    expr
}

fn is_const_assertion(ty: &oxc_ast::ast::TSType<'_>) -> bool {
    use oxc_ast::ast::{TSType, TSTypeName};
    if let TSType::TSTypeReference(reference) = ty
        && let TSTypeName::IdentifierReference(ident) = &reference.type_name
    {
        return ident.name == "const";
    }
    false
}
