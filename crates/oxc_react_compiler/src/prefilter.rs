// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use oxc_ast::ast::{
    AssignmentTarget, CallExpression, Expression, Function, Program, VariableDeclarator,
};
use oxc_ast_visit::{Visit, walk};

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

use react_compiler_hir::environment::is_react_like_name;

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
    fn visit_variable_declaration(&mut self, decl: &oxc_ast::ast::VariableDeclaration<'a>) {
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
            oxc_ast::ast::BindingPattern::BindingIdentifier(ident) => Some(ident.name.as_str()),
            _ => None,
        };

        let prev_name = self.current_name.take();
        self.current_name = name;

        if let Some(init) = &decl.init {
            self.visit_expression(init);
        }

        self.current_name = prev_name;
    }

    fn visit_assignment_expression(&mut self, expr: &oxc_ast::ast::AssignmentExpression<'a>) {
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

    fn visit_function(&mut self, func: &Function<'a>, _flags: oxc_semantic::ScopeFlags) {
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

    fn visit_arrow_function_expression(
        &mut self,
        _expr: &oxc_ast::ast::ArrowFunctionExpression<'a>,
    ) {
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

    fn visit_class(&mut self, _class: &oxc_ast::ast::Class<'a>) {
        // Skip class bodies entirely
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_react_like_name() {
        assert!(is_react_like_name("Component"));
        assert!(is_react_like_name("MyComponent"));
        assert!(is_react_like_name("A"));
        assert!(is_react_like_name("useState"));
        assert!(is_react_like_name("useEffect"));
        assert!(is_react_like_name("use0"));

        assert!(!is_react_like_name("component"));
        assert!(!is_react_like_name("myFunction"));
        assert!(!is_react_like_name("use"));
        assert!(!is_react_like_name("user"));
        assert!(!is_react_like_name("useful"));
        assert!(!is_react_like_name(""));
    }
}
