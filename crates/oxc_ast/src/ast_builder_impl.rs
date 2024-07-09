#![allow(
    clippy::fn_params_excessive_bools,
    clippy::must_use_candidate, // must_use_candidate is too annoying for this file
    clippy::too_many_arguments,
    clippy::unused_self,
)]

use std::mem;

use oxc_allocator::{Allocator, Box, String, Vec};
use oxc_span::{Atom, GetSpan, Span};
use oxc_syntax::{number::NumberBase, operator::UnaryOperator};

#[allow(clippy::wildcard_imports)]
use crate::ast::*;
use crate::AstBuilder;

impl<'a> AstBuilder<'a> {
    #[inline]
    pub fn new(allocator: &'a Allocator) -> Self {
        Self { allocator }
    }

    #[inline]
    pub fn alloc<T>(self, value: T) -> Box<'a, T> {
        Box::new_in(value, self.allocator)
    }

    #[inline]
    pub fn new_vec<T>(self) -> Vec<'a, T> {
        Vec::new_in(self.allocator)
    }

    #[inline]
    pub fn new_vec_with_capacity<T>(self, capacity: usize) -> Vec<'a, T> {
        Vec::with_capacity_in(capacity, self.allocator)
    }

    #[inline]
    pub fn new_vec_single<T>(self, value: T) -> Vec<'a, T> {
        let mut vec = self.new_vec_with_capacity(1);
        vec.push(value);
        vec
    }

    #[inline]
    pub fn new_vec_from_iter<T, I: IntoIterator<Item = T>>(self, iter: I) -> Vec<'a, T> {
        Vec::from_iter_in(iter, self.allocator)
    }

    #[inline]
    pub fn new_str(self, value: &str) -> &'a str {
        String::from_str_in(value, self.allocator).into_bump_str()
    }

    #[inline]
    pub fn new_atom(self, value: &str) -> Atom<'a> {
        Atom::from(String::from_str_in(value, self.allocator).into_bump_str())
    }

    /// # SAFETY
    /// This method is completely unsound and should not be used.
    /// We need to remove all uses of it. Please don't add any more!
    /// <https://github.com/oxc-project/oxc/issues/3483>
    #[inline]
    pub fn copy<T>(self, src: &T) -> T {
        // SAFETY: Not safe (see above)
        #[allow(unsafe_code)]
        unsafe {
            std::mem::transmute_copy(src)
        }
    }

    /// Moves the expression out by replacing it with a null expression.
    #[inline]
    pub fn move_expression(self, expr: &mut Expression<'a>) -> Expression<'a> {
        let null_expr = self.expression_null_literal(expr.span());
        mem::replace(expr, null_expr)
    }

    #[inline]
    pub fn move_statement(self, stmt: &mut Statement<'a>) -> Statement<'a> {
        let empty_stmt = self.empty_statement(stmt.span());
        mem::replace(stmt, Statement::EmptyStatement(self.alloc(empty_stmt)))
    }

    #[inline]
    pub fn move_statement_vec(self, stmts: &mut Vec<'a, Statement<'a>>) -> Vec<'a, Statement<'a>> {
        mem::replace(stmts, self.new_vec())
    }

    #[inline]
    pub fn move_assignment_target(self, target: &mut AssignmentTarget<'a>) -> AssignmentTarget<'a> {
        let dummy = self.simple_assignment_target_identifier_reference(Span::default(), "");
        mem::replace(target, dummy.into())
    }

    #[inline]
    pub fn move_declaration(self, decl: &mut Declaration<'a>) -> Declaration<'a> {
        let empty_decl = self.variable_declaration(
            Span::default(),
            VariableDeclarationKind::Var,
            self.new_vec(),
            false,
        );
        let empty_decl = Declaration::VariableDeclaration(self.alloc(empty_decl));
        mem::replace(decl, empty_decl)
    }

    /* ---------- Constructors ---------- */

    /// `void 0`
    #[inline]
    pub fn void_0(self) -> Expression<'a> {
        let num = self.expression_numeric_literal(Span::default(), 0.0, "0", NumberBase::Decimal);
        Expression::UnaryExpression(self.alloc(self.unary_expression(
            Span::default(),
            UnaryOperator::Void,
            num,
        )))
    }

    /* ---------- Functions ---------- */

    #[inline]
    pub fn plain_formal_parameter(
        self,
        span: Span,
        pattern: BindingPattern<'a>,
    ) -> FormalParameter<'a> {
        self.formal_parameter(span, pattern, None, false, false, self.new_vec())
    }

    #[inline]
    pub fn plain_function(
        self,
        r#type: FunctionType,
        span: Span,
        id: Option<BindingIdentifier<'a>>,
        params: FormalParameters<'a>,
        body: Option<FunctionBody<'a>>,
    ) -> Box<'a, Function<'a>> {
        self.alloc(self.function(
            r#type,
            span,
            id,
            false,
            false,
            false,
            Option::<TSTypeParameterDeclaration>::None,
            None,
            params,
            body,
            Option::<TSTypeAnnotation>::None,
        ))
    }

    /* ---------- Modules ---------- */

    #[inline]
    pub fn plain_export_named_declaration_declaration(
        self,
        span: Span,
        declaration: Declaration<'a>,
    ) -> Box<'a, ExportNamedDeclaration<'a>> {
        self.alloc(self.export_named_declaration(
            span,
            Some(declaration),
            self.new_vec(),
            None,
            ImportOrExportKind::Value,
            None,
        ))
    }

    #[inline]
    pub fn plain_export_named_declaration(
        self,
        span: Span,
        specifiers: Vec<'a, ExportSpecifier<'a>>,
        source: Option<StringLiteral<'a>>,
    ) -> Box<'a, ExportNamedDeclaration<'a>> {
        self.alloc(self.export_named_declaration(
            span,
            None,
            specifiers,
            source,
            ImportOrExportKind::Value,
            None,
        ))
    }

    /* ---------- TypeScript ---------- */

    #[inline]
    pub fn ts_interface_heritages(
        self,
        extends: Vec<'a, (Expression<'a>, Option<Box<'a, TSTypeParameterInstantiation<'a>>>, Span)>,
    ) -> Vec<'a, TSInterfaceHeritage<'a>> {
        Vec::from_iter_in(
            extends.into_iter().map(|(expression, type_parameters, span)| TSInterfaceHeritage {
                span,
                expression,
                type_parameters,
            }),
            self.allocator,
        )
    }

    #[inline]
    pub fn jsx_opening_fragment(self, span: Span) -> JSXOpeningFragment {
        JSXOpeningFragment { span }
    }

    #[inline]
    pub fn jsx_closing_fragment(self, span: Span) -> JSXClosingFragment {
        JSXClosingFragment { span }
    }
}
