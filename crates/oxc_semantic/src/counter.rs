//! Visitor to count nodes, scopes, symbols and references in AST.
//! These counts can be used to pre-allocate sufficient capacity in `AstNodes`,
//! `ScopeTree`, and `SymbolTable` to store info for all these items.

use std::{cell::Cell, cmp::max};

use oxc_ast::{
    ast::{
        BindingIdentifier, IdentifierReference, JSXElementName, JSXMemberExpressionObject,
        TSEnumMemberName, TSModuleDeclarationName,
    },
    visit::walk::{walk_ts_enum_member_name, walk_ts_module_declaration_name},
    AstKind, Visit,
};
use oxc_syntax::scope::{ScopeFlags, ScopeId};

#[derive(Default, Debug)]
pub struct Counter {
    pub nodes_count: usize,
    pub scopes_count: usize,
    pub symbols_count: usize,
    pub references_count: usize,
    current_scope_depth: usize,
    pub max_scope_depth: usize,
}

impl<'a> Visit<'a> for Counter {
    #[inline]
    fn enter_node(&mut self, _: AstKind<'a>) {
        self.nodes_count += 1;
    }

    #[inline]
    fn enter_scope(&mut self, _: ScopeFlags, _: &Cell<Option<ScopeId>>) {
        self.scopes_count += 1;

        self.current_scope_depth += 1;
        self.max_scope_depth = max(self.max_scope_depth, self.current_scope_depth);
    }
    #[inline]
    fn leave_scope(&mut self) {
        self.current_scope_depth -= 1;
    }

    #[inline]
    fn visit_binding_identifier(&mut self, _: &BindingIdentifier<'a>) {
        self.nodes_count += 1;
        self.symbols_count += 1;
    }

    #[inline]
    fn visit_identifier_reference(&mut self, _: &IdentifierReference<'a>) {
        self.nodes_count += 1;
        self.references_count += 1;
    }

    #[inline]
    fn visit_jsx_member_expression_object(&mut self, it: &JSXMemberExpressionObject<'a>) {
        self.nodes_count += 1;
        match it {
            JSXMemberExpressionObject::MemberExpression(expr) => {
                self.visit_jsx_member_expression(expr);
            }
            JSXMemberExpressionObject::Identifier(_) => {
                self.nodes_count += 1;
                self.references_count += 1;
            }
        }
    }

    #[inline]
    fn visit_jsx_element_name(&mut self, it: &JSXElementName<'a>) {
        self.nodes_count += 1;
        match it {
            JSXElementName::Identifier(ident) => {
                self.nodes_count += 1;
                if ident.name.chars().next().is_some_and(char::is_uppercase) {
                    self.references_count += 1;
                }
            }
            JSXElementName::NamespacedName(name) => self.visit_jsx_namespaced_name(name),
            JSXElementName::MemberExpression(expr) => self.visit_jsx_member_expression(expr),
        }
    }

    #[inline]
    fn visit_ts_enum_member_name(&mut self, it: &TSEnumMemberName<'a>) {
        if !it.is_expression() {
            self.symbols_count += 1;
        }
        walk_ts_enum_member_name(self, it);
    }

    #[inline]
    fn visit_ts_module_declaration_name(&mut self, it: &TSModuleDeclarationName<'a>) {
        self.symbols_count += 1;
        walk_ts_module_declaration_name(self, it);
    }
}
