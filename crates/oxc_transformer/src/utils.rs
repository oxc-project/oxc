use std::mem;

use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_span::{Atom, Span};

use crate::context::TransformerCtx;

// TODO:
// <https://github.com/babel/babel/blob/419644f27c5c59deb19e71aaabd417a3bc5483ca/packages/babel-traverse/src/scope/index.ts#L543>
pub fn generate_uid_based_on_node(expr: &Expression) -> Atom {
    let mut parts = std::vec::Vec::with_capacity(1);
    expr.gather(&mut |part| parts.push(part));
    let name = parts.join("$");
    Atom::from(format!("_{name}"))
}

// TODO: <https://github.com/babel/babel/blob/419644f27c5c59deb19e71aaabd417a3bc5483ca/packages/babel-traverse/src/scope/index.ts#L61>
pub trait GatherNodeParts {
    fn gather<F: FnMut(Atom)>(&self, f: &mut F);
}

impl<'a> GatherNodeParts for Expression<'a> {
    fn gather<F: FnMut(Atom)>(&self, f: &mut F) {
        match self {
            Self::Identifier(ident) => f(ident.name.clone()),
            Self::MemberExpression(expr) => expr.gather(f),
            _ => f(Atom::from("ref")),
        }
    }
}

impl<'a> GatherNodeParts for MemberExpression<'a> {
    fn gather<F: FnMut(Atom)>(&self, f: &mut F) {
        self.object().gather(f);
        match self {
            MemberExpression::ComputedMemberExpression(expr) => expr.expression.gather(f),
            MemberExpression::StaticMemberExpression(expr) => expr.property.gather(f),
            MemberExpression::PrivateFieldExpression(expr) => expr.field.gather(f),
        }
    }
}

impl GatherNodeParts for IdentifierName {
    fn gather<F: FnMut(Atom)>(&self, f: &mut F) {
        f(self.name.clone());
    }
}

impl GatherNodeParts for PrivateIdentifier {
    fn gather<F: FnMut(Atom)>(&self, f: &mut F) {
        f(self.name.clone());
    }
}

pub trait CreateVars<'a> {
    fn ctx(&self) -> &TransformerCtx<'a>;

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

    fn create_new_var(&mut self, expr: &Expression<'a>) -> Atom {
        let name = generate_uid_based_on_node(expr);
        // TODO: scope.push({ id: temp });

        // Add `var name` to scope
        let binding_identifier = BindingIdentifier::new(Span::default(), name.clone());
        let binding_pattern_kind = self.ctx().ast.binding_pattern_identifier(binding_identifier);
        let binding = self.ctx().ast.binding_pattern(binding_pattern_kind, None, false);
        let kind = VariableDeclarationKind::Var;
        let decl = self.ctx().ast.variable_declarator(Span::default(), kind, binding, None, false);
        self.vars_mut().push(decl);
        name
    }
}
