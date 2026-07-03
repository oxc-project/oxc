// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use oxc_ast::ast::{
    ArrowFunctionExpression, AssignmentExpression, AssignmentTarget, BindingPattern,
    CallExpression, Class, Expression, Function, Program, VariableDeclaration, VariableDeclarator,
};
use oxc_ast_visit::{Visit, walk};
use oxc_semantic::ScopeFlags;

/// Whether the program contains a component (Uppercase name) or hook (`use[A-Z0-9]`).
pub fn has_react_like_functions(program: &Program) -> bool {
    let mut visitor = ReactLikeVisitor { found: false, current_name: None };
    visitor.visit_program(program);
    visitor.found
}

pub fn has_resource_management_declarations(program: &Program) -> bool {
    let mut visitor = ResourceManagementVisitor { found: false };
    visitor.visit_program(program);
    visitor.found
}

use crate::react_compiler_hir::environment::is_react_like_name;

/// `memo`/`forwardRef` (optionally `React.`-qualified): their callback is a
/// component even when anonymous, so such files must not be skipped.
fn is_component_wrapper(callee: &Expression) -> bool {
    let is_wrapper = |name: &str| matches!(name, "memo" | "forwardRef");
    match callee {
        Expression::Identifier(id) => is_wrapper(&id.name),
        Expression::StaticMemberExpression(m) => is_wrapper(&m.property.name),
        _ => false,
    }
}

struct ReactLikeVisitor<'a> {
    found: bool,
    current_name: Option<&'a str>,
}

struct ResourceManagementVisitor {
    found: bool,
}

impl<'a> Visit<'a> for ResourceManagementVisitor {
    fn visit_variable_declaration(&mut self, decl: &VariableDeclaration<'a>) {
        if self.found {
            return;
        }
        if decl.kind.is_using() {
            self.found = true;
            return;
        }
        walk::walk_variable_declaration(self, decl);
    }
}

impl<'a> Visit<'a> for ReactLikeVisitor<'a> {
    fn visit_variable_declarator(&mut self, decl: &VariableDeclarator<'a>) {
        if self.found {
            return;
        }

        let name = match &decl.id {
            BindingPattern::BindingIdentifier(ident) => Some(ident.name.as_str()),
            _ => None,
        };

        let prev_name = self.current_name.take();
        self.current_name = name;

        if let Some(init) = &decl.init {
            self.visit_expression(init);
        }

        self.current_name = prev_name;
    }

    fn visit_assignment_expression(&mut self, expr: &AssignmentExpression<'a>) {
        if self.found {
            return;
        }

        let name = match &expr.left {
            AssignmentTarget::AssignmentTargetIdentifier(ident) => Some(ident.name.as_str()),
            _ => None,
        };

        let prev_name = self.current_name.take();
        self.current_name = name;

        self.visit_expression(&expr.right);

        self.current_name = prev_name;
    }

    fn visit_function(&mut self, func: &Function<'a>, _flags: ScopeFlags) {
        if self.found {
            return;
        }

        if let Some(id) = &func.id {
            if is_react_like_name(&id.name) {
                self.found = true;
                return;
            }
        }

        if func.id.is_none() {
            if let Some(name) = &self.current_name {
                if is_react_like_name(name) {
                    self.found = true;
                    return;
                }
            }
        }

        // Don't traverse into the function body
    }

    fn visit_arrow_function_expression(&mut self, _expr: &ArrowFunctionExpression<'a>) {
        if self.found {
            return;
        }

        if let Some(name) = &self.current_name {
            if is_react_like_name(name) {
                self.found = true;
                return;
            }
        }

        // Don't traverse into the function body
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if self.found {
            return;
        }
        // A function passed to `memo`/`forwardRef` is a component even without a
        // React-like name.
        if is_component_wrapper(&call.callee)
            && call.arguments.iter().any(|arg| {
                matches!(
                    arg.as_expression(),
                    Some(
                        Expression::FunctionExpression(_) | Expression::ArrowFunctionExpression(_)
                    )
                )
            })
        {
            self.found = true;
            return;
        }
        walk::walk_call_expression(self, call);
    }

    fn visit_class(&mut self, _class: &Class<'a>) {
        // Skip class bodies entirely
    }
}
