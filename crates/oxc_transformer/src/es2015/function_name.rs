use std::rc::Rc;

use oxc_ast::{ast::*, AstBuilder};

use crate::options::{TransformOptions, TransformTarget};

/// ES2015: Function Name
///
/// References:
/// * <https://babel.dev/docs/babel-plugin-transform-function-name>
/// * <https://github.com/babel/babel/tree/main/packages/babel-plugin-transform-function-name>
pub struct FunctionName<'a> {
    ast: Rc<AstBuilder<'a>>,
}

impl<'a> FunctionName<'a> {
    pub fn new(ast: Rc<AstBuilder<'a>>, options: &TransformOptions) -> Option<Self> {
        (options.target < TransformTarget::ES2015 || options.function_name).then(|| Self { ast })
    }

    pub fn transform_assignment_expression(&mut self, expr: &mut AssignmentExpression<'a>) {
        if let AssignmentTarget::SimpleAssignmentTarget(
            SimpleAssignmentTarget::AssignmentTargetIdentifier(target),
        ) = &expr.left
        {
            if expr.right.is_function() {
                let id = BindingIdentifier::new(target.span, target.name.clone());

                self.transform_expression(&mut expr.right, &id);
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
                    PropertyKey::Identifier(ident) => {
                        BindingIdentifier::new(ident.span, ident.name.clone())
                    }
                    PropertyKey::PrivateIdentifier(ident) => {
                        BindingIdentifier::new(ident.span, ident.name.clone())
                    }
                    PropertyKey::Expression(_) => continue,
                };

                self.transform_expression(&mut property.value, &id);
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
