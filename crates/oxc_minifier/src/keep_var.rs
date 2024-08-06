#[allow(clippy::wildcard_imports)]
use oxc_ast::{ast::*, syntax_directed_operations::BoundNames, AstBuilder, Visit};
use oxc_span::{Atom, Span, SPAN};
use oxc_syntax::scope::ScopeFlags;

pub struct KeepVar<'a> {
    ast: AstBuilder<'a>,
    vars: std::vec::Vec<(Atom<'a>, Span)>,
}

impl<'a> Visit<'a> for KeepVar<'a> {
    fn visit_variable_declaration(&mut self, decl: &VariableDeclaration<'a>) {
        if decl.kind.is_var() {
            decl.bound_names(&mut |ident| {
                self.vars.push((ident.name.clone(), ident.span));
            });
        }
    }

    fn visit_function(&mut self, _it: &Function<'a>, _flags: ScopeFlags) {
        /* skip functions */
    }

    fn visit_arrow_function_expression(&mut self, _it: &ArrowFunctionExpression<'a>) {}

    fn visit_class(&mut self, _it: &Class<'a>) {
        /* skip classes */
    }
}

impl<'a> KeepVar<'a> {
    pub fn new(ast: AstBuilder<'a>) -> Self {
        Self { ast, vars: std::vec![] }
    }

    pub fn get_variable_declaration_statement(self) -> Option<Statement<'a>> {
        if self.vars.is_empty() {
            return None;
        }

        let kind = VariableDeclarationKind::Var;
        let decls = self.ast.vec_from_iter(self.vars.into_iter().map(|(name, span)| {
            let binding_kind = self.ast.binding_pattern_kind_binding_identifier(span, name);
            let id =
                self.ast.binding_pattern::<Option<TSTypeAnnotation>>(binding_kind, None, false);
            self.ast.variable_declarator(span, kind, id, None, false)
        }));

        let decl = self.ast.variable_declaration(SPAN, kind, decls, false);
        let stmt = self.ast.statement_declaration(self.ast.declaration_from_variable(decl));
        Some(stmt)
    }
}
