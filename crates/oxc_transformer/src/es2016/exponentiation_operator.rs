use std::{cell::RefCell, rc::Rc};

use oxc_allocator::Vec;
use oxc_ast::{ast::*, AstBuilder};
use oxc_semantic::SymbolTable;
use oxc_span::{Atom, Span};
use oxc_syntax::operator::{AssignmentOperator, BinaryOperator};

use crate::{
    options::{TransformOptions, TransformTarget},
    utils::CreateVars,
};

/// ES2016: Exponentiation Operator
///
/// References:
/// * <https://babel.dev/docs/babel-plugin-transform-exponentiation-operator>
/// * <https://github.com/babel/babel/blob/main/packages/babel-plugin-transform-exponentiation-operator>
/// * <https://github.com/babel/babel/blob/main/packages/babel-helper-builder-binary-assignment-operator-visitor>
pub struct ExponentiationOperator<'a> {
    ast: Rc<AstBuilder<'a>>,
    symbols: Rc<RefCell<SymbolTable>>,
    vars: Vec<'a, VariableDeclarator<'a>>,
}

struct Exploded<'a> {
    reference: AssignmentTarget<'a>,
    uid: Expression<'a>,
}

impl<'a> CreateVars<'a> for ExponentiationOperator<'a> {
    fn ast(&self) -> &AstBuilder<'a> {
        &self.ast
    }

    fn vars_mut(&mut self) -> &mut Vec<'a, VariableDeclarator<'a>> {
        &mut self.vars
    }
}

impl<'a> ExponentiationOperator<'a> {
    pub fn new(
        ast: Rc<AstBuilder<'a>>,
        symbols: Rc<RefCell<SymbolTable>>,
        options: &TransformOptions,
    ) -> Option<Self> {
        (options.target < TransformTarget::ES2016 || options.exponentiation_operator).then(|| {
            let vars = ast.new_vec();
            Self { ast, symbols, vars }
        })
    }

    pub fn transform_expression(&mut self, expr: &mut Expression<'a>) {
        // left ** right
        if let Expression::BinaryExpression(binary_expr) = expr {
            if binary_expr.operator == BinaryOperator::Exponential {
                let left = self.ast.move_expression(&mut binary_expr.left);
                let right = self.ast.move_expression(&mut binary_expr.right);
                *expr = self.math_pow(left, right);
            }
        }

        // left **= right
        if let Expression::AssignmentExpression(assign_expr) = expr {
            if assign_expr.operator == AssignmentOperator::Exponential {
                let mut nodes = self.ast.new_vec();
                let left = self.ast.move_assignment_target(&mut assign_expr.left);
                let Some(Exploded { reference, uid }) = self.explode(left, &mut nodes) else {
                    return;
                };
                let right = self.ast.move_expression(&mut assign_expr.right);
                let right = self.math_pow(uid, right);
                let assign_expr = self.ast.assignment_expression(
                    Span::default(),
                    AssignmentOperator::Assign,
                    reference,
                    right,
                );
                nodes.push(assign_expr);
                *expr = self.ast.sequence_expression(Span::default(), nodes);
            }
        }
    }

    /// `left ** right` -> `Math.pow(left, right)`
    fn math_pow(&mut self, left: Expression<'a>, right: Expression<'a>) -> Expression<'a> {
        let ident_math = IdentifierReference::new(Span::default(), Atom::from("Math"));
        let object = self.ast.identifier_reference_expression(ident_math);
        let property = IdentifierName::new(Span::default(), Atom::from("pow"));
        let callee = self.ast.static_member_expression(Span::default(), object, property, false);
        let mut arguments = self.ast.new_vec_with_capacity(2);
        arguments.push(Argument::Expression(left));
        arguments.push(Argument::Expression(right));
        self.ast.call_expression(Span::default(), callee, arguments, false, None)
    }

    /// Change `lhs **= 2` to `var temp; temp = lhs, lhs = Math.pow(temp, 2);`.
    /// If the lhs is a member expression `obj.ref` or `obj[ref]`, assign them to a temporary variable so side-effects are not computed twice.
    /// For `obj.ref`, change it to `var _obj; _obj = obj, _obj["ref"] = Math.pow(_obj["ref"], 2)`.
    /// For `obj[ref]`, change it to `var _obj, _ref; _obj = obj, _ref = ref, _obj[_ref] = Math.pow(_obj[_ref], 2);`.
    fn explode(
        &mut self,
        node: AssignmentTarget<'a>,
        nodes: &mut Vec<'a, Expression<'a>>,
    ) -> Option<Exploded<'a>> {
        let node = match node {
            AssignmentTarget::SimpleAssignmentTarget(target) => target,
            AssignmentTarget::AssignmentTargetPattern(_) => {
                // Invalid Syntax
                return None;
            }
        };
        let obj = self.get_obj_ref(self.ast.copy(&node), nodes)?;
        let (reference, uid) = match node {
            SimpleAssignmentTarget::AssignmentTargetIdentifier(ident) => {
                let reference = AssignmentTarget::SimpleAssignmentTarget(
                    SimpleAssignmentTarget::AssignmentTargetIdentifier(ident),
                );
                (reference, obj)
            }
            SimpleAssignmentTarget::MemberAssignmentTarget(member_expr) => {
                let computed = member_expr.is_computed();
                let prop = self.get_prop_ref(member_expr.unbox(), nodes)?;
                let optional = false;
                let obj_clone = self.ast.copy(&obj);
                let span = Span::default();
                let (reference, uid) = match &prop {
                    Expression::Identifier(ident) if !computed => {
                        let ident = IdentifierName::new(span, ident.name.clone());
                        (
                            self.ast.static_member(span, obj_clone, ident.clone(), optional),
                            self.ast.static_member_expression(span, obj, ident, optional),
                        )
                    }
                    _ => {
                        let prop_clone = self.ast.copy(&prop);
                        (
                            self.ast.computed_member(span, obj_clone, prop_clone, optional),
                            self.ast.computed_member_expression(span, obj, prop, optional),
                        )
                    }
                };
                (
                    AssignmentTarget::SimpleAssignmentTarget(
                        self.ast.simple_assignment_target_member_expression(reference),
                    ),
                    uid,
                )
            }
            _ => return None,
        };
        Some(Exploded { reference, uid })
    }

    /// Make sure side-effects of evaluating `obj` of `obj.ref` and `obj[ref]` only happen once.
    fn get_obj_ref(
        &mut self,
        node: SimpleAssignmentTarget<'a>,
        nodes: &mut Vec<'a, Expression<'a>>,
    ) -> Option<Expression<'a>> {
        let reference = match node {
            SimpleAssignmentTarget::AssignmentTargetIdentifier(ident) => {
                if ident
                    .reference_id
                    .get()
                    .is_some_and(|reference_id| self.symbols.borrow().has_binding(reference_id))
                {
                    // this variable is declared in scope so we can be 100% sure
                    // that evaluating it multiple times won't trigger a getter
                    // or something else
                    return Some(self.ast.identifier_reference_expression(ident.unbox()));
                }
                // could possibly trigger a getter so we need to only evaluate it once
                self.ast.identifier_reference_expression(ident.unbox())
            }
            SimpleAssignmentTarget::MemberAssignmentTarget(member_expr) => {
                let expr = match member_expr.unbox() {
                    MemberExpression::ComputedMemberExpression(e) => e.object,
                    MemberExpression::StaticMemberExpression(e) => e.object,
                    MemberExpression::PrivateFieldExpression(e) => e.object,
                };
                // the object reference that we need to save is locally declared
                // so as per the previous comment we can be 100% sure evaluating
                // it multiple times will be safe
                // Super cannot be directly assigned so lets return it also
                if matches!(expr, Expression::Super(_))
                    || matches!(&expr, Expression::Identifier(ident) if
                        ident.reference_id.get().is_some_and(|reference_id| self.symbols.borrow().has_binding(reference_id)))
                {
                    return Some(expr);
                }
                expr
            }
            _ => return None,
        };
        Some(self.add_new_reference(reference, nodes))
    }

    /// Make sure side-effects of evaluating `ref` of `obj.ref` and `obj[ref]` only happen once.
    fn get_prop_ref(
        &mut self,
        node: MemberExpression<'a>,
        nodes: &mut Vec<'a, Expression<'a>>,
    ) -> Option<Expression<'a>> {
        let prop = match node {
            MemberExpression::ComputedMemberExpression(expr) => {
                let expr = expr.expression;
                if expr.is_literal() {
                    return Some(expr);
                }
                expr
            }
            MemberExpression::StaticMemberExpression(expr) => {
                let ident = expr.property;
                let string_literal = StringLiteral::new(Span::default(), ident.name);
                return Some(self.ast.literal_string_expression(string_literal));
            }
            MemberExpression::PrivateFieldExpression(_) => {
                // From babel: "We can't generate property ref for private name, please install `@babel/plugin-transform-class-properties`"
                return None;
            }
        };
        Some(self.add_new_reference(prop, nodes))
    }

    fn add_new_reference(
        &mut self,
        expr: Expression<'a>,
        nodes: &mut Vec<'a, Expression<'a>>,
    ) -> Expression<'a> {
        let name = self.create_new_var(&expr);
        // Add new reference `_name = name` to nodes
        let ident = IdentifierReference::new(Span::default(), name);
        let target = self.ast.simple_assignment_target_identifier(ident.clone());
        let target = AssignmentTarget::SimpleAssignmentTarget(target);
        let op = AssignmentOperator::Assign;
        nodes.push(self.ast.assignment_expression(Span::default(), op, target, expr));
        self.ast.identifier_reference_expression(ident)
    }
}

#[test]
fn test() {
    use crate::{
        options::{TransformOptions, TransformTarget},
        tester::Tester,
    };

    let options =
        TransformOptions { target: TransformTarget::ES2015, ..TransformOptions::default() };

    let tests = &[(
        "let x = {}; let y = 0; let z = 0; x[z++] **= y;",
        "var _ref; let x = {}; let y = 0; let z = 0; _ref = z++,x[_ref] = Math.pow(x[_ref], y);",
    )];

    Tester::new("test.js", options).test(tests);
}
