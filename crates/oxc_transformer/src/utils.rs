use std::{cell::Ref, mem};

use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_span::Span;

use crate::context::TransformerCtx;

pub trait CreateVars<'a> {
    fn ctx(&self) -> Ref<'_, TransformerCtx<'a>>;

    fn vars_mut(&mut self) -> &mut Vec<'a, VariableDeclarator<'a>>;

    fn add_vars_to_statements(&mut self, stmts: &mut Vec<'a, Statement<'a>>) {
        if self.vars_mut().is_empty() {
            return;
        }
        let new_vec = self.ctx().ast.new_vec();
        let decls = mem::replace(self.vars_mut(), new_vec);
        let kind = VariableDeclarationKind::Var;
        let decl =
            self.ctx().ast.variable_declaration(Span::default(), kind, decls, Modifiers::empty());
        let stmt = Statement::Declaration(Declaration::VariableDeclaration(decl));
        stmts.insert(0, stmt);
    }

    fn create_new_var(&mut self, expr: &Expression<'a>) -> IdentifierReference {
        let name = self.ctx().scopes().generate_uid_based_on_node(expr);
        self.ctx().add_binding(name.clone());

        // Add `var name` to scope
        // TODO: hookup symbol id
        let binding_identifier = BindingIdentifier::new(Span::default(), name.clone());
        let binding_pattern_kind = self.ctx().ast.binding_pattern_identifier(binding_identifier);
        let binding = self.ctx().ast.binding_pattern(binding_pattern_kind, None, false);
        let kind = VariableDeclarationKind::Var;
        let decl = self.ctx().ast.variable_declarator(Span::default(), kind, binding, None, false);
        self.vars_mut().push(decl);
        // TODO: add reference id and flag
        IdentifierReference::new(Span::default(), name)
    }

    /// Possibly generate a memoised identifier if it is not static and has consequences.
    /// <https://github.com/babel/babel/blob/419644f27c5c59deb19e71aaabd417a3bc5483ca/packages/babel-traverse/src/scope/index.ts#L578>
    fn maybe_generate_memoised(&mut self, expr: &Expression<'a>) -> Option<IdentifierReference> {
        if self.ctx().symbols().is_static(expr) {
            None
        } else {
            Some(self.create_new_var(expr))
        }
    }
}
