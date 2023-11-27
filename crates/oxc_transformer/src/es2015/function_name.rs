use std::rc::Rc;

use lazy_static::lazy_static;
use oxc_ast::{ast::*, AstBuilder};
use oxc_span::{Atom, Span};
use oxc_syntax::operator::AssignmentOperator;
use regex::Regex;

use crate::options::{TransformOptions, TransformTarget};

/// ES2015: Function Name
///
/// References:
/// * <https://babel.dev/docs/babel-plugin-transform-function-name>
/// * <https://github.com/babel/babel/tree/main/packages/babel-plugin-transform-function-name>
pub struct FunctionName<'a> {
    ast: Rc<AstBuilder<'a>>,
    unicode_escapes: bool,
}

impl<'a> FunctionName<'a> {
    pub fn new(ast: Rc<AstBuilder<'a>>, options: &TransformOptions) -> Option<Self> {
        (options.target < TransformTarget::ES2015 || options.function_name).then(|| Self {
            ast,
            // TODO hook up the the plugin
            unicode_escapes: true,
        })
    }

    pub fn transform_assignment_expression(&mut self, expr: &mut AssignmentExpression<'a>) {
        if expr.right.is_function() && matches!(expr.operator, AssignmentOperator::Assign) {
            if let AssignmentTarget::SimpleAssignmentTarget(
                SimpleAssignmentTarget::AssignmentTargetIdentifier(target),
            ) = &expr.left
            {
                let id =
                    create_valid_identifier(target.span, target.name.clone(), self.unicode_escapes);

                if let Some(id) = &id {
                    self.transform_expression(&mut expr.right, id);
                }
            }
        }
    }

    pub fn transform_object_expression(&mut self, expr: &mut ObjectExpression<'a>) {
        for property_kind in expr.properties.iter_mut() {
            if let ObjectPropertyKind::ObjectProperty(property) = property_kind {
                if property.computed || !property.value.is_function() {
                    continue;
                }

                let id = match &property.key {
                    PropertyKey::Identifier(ident) => create_valid_identifier(
                        ident.span,
                        ident.name.clone(),
                        self.unicode_escapes,
                    ),
                    PropertyKey::PrivateIdentifier(ident) => create_valid_identifier(
                        ident.span,
                        ident.name.clone(),
                        self.unicode_escapes,
                    ),
                    PropertyKey::Expression(_) => continue,
                };

                if let Some(id) = &id {
                    self.transform_expression(&mut property.value, id);
                }
            }
        }
    }

    pub fn transform_variable_declarator(&mut self, decl: &mut VariableDeclarator<'a>) {
        let Some(init) = &mut decl.init else { return };

        if let BindingPatternKind::BindingIdentifier(id) = &decl.id.kind {
            self.transform_expression(init, id);
        };
    }

    // Internal only
    fn transform_expression(&mut self, expr: &mut Expression<'a>, id: &BindingIdentifier) {
        // TODO check local bindings
        // () => {}
        if let Expression::ArrowExpression(arrow) = expr {
            let arrow = self.ast.copy(&**arrow);

            let func = self.ast.function_expression(self.ast.function(
                FunctionType::FunctionExpression,
                arrow.span,
                Some(id.to_owned()),
                arrow.expression,
                arrow.generator,
                arrow.r#async,
                arrow.params,
                Some(arrow.body),
                arrow.type_parameters,
                arrow.return_type,
                Modifiers::default(),
            ));

            *expr = func;
        }

        // function () {} -> function name() {}
        if let Expression::FunctionExpression(func) = expr {
            if func.id.is_none() {
                func.id = Some(id.to_owned());
            }
        }
    }
}

// https://github.com/babel/babel/blob/main/packages/babel-helper-function-name/src/index.ts
// https://github.com/babel/babel/blob/main/packages/babel-types/src/converters/toBindingIdentifierName.ts#L3
fn create_valid_identifier(
    span: Span,
    atom: Atom,
    unicode_escapes: bool,
) -> Option<BindingIdentifier> {
    lazy_static! {
        static ref UNICODE_NAME: Regex = Regex::new(r"[\uD800-\uDFFF]").unwrap();
    }

    if !unicode_escapes && UNICODE_NAME.is_match(atom.as_str()) {
        return None;
    }

    // TODO: Better way?
    Some(if atom == "eval" || atom == "arguments" || atom == "null" {
        BindingIdentifier::new(span, Atom::from(format!("_{}", atom.as_str())))
    } else {
        BindingIdentifier::new(span, atom)
    })
}
