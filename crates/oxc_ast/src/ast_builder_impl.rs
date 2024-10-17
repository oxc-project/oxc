#![allow(
    clippy::fn_params_excessive_bools,
    clippy::must_use_candidate, // must_use_candidate is too annoying for this file
    clippy::too_many_arguments,
    clippy::unused_self,
)]

use std::mem;

use oxc_allocator::{Allocator, Box, FromIn, String, Vec};
use oxc_span::{Atom, GetSpan, Span};
use oxc_syntax::{number::NumberBase, operator::UnaryOperator};

#[allow(clippy::wildcard_imports)]
use crate::ast::*;
use crate::AstBuilder;

/// Type that can be used in any AST builder method call which requires an `IntoIn<'a, Anything<'a>>`.
/// Pass `NONE` instead of `None::<Anything<'a>>`.
#[allow(clippy::upper_case_acronyms)]
pub struct NONE;

impl<'a, T> FromIn<'a, NONE> for Option<Box<'a, T>> {
    fn from_in(_: NONE, _: &'a Allocator) -> Self {
        None
    }
}

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
    pub fn vec<T>(self) -> Vec<'a, T> {
        Vec::new_in(self.allocator)
    }

    #[inline]
    pub fn vec_with_capacity<T>(self, capacity: usize) -> Vec<'a, T> {
        Vec::with_capacity_in(capacity, self.allocator)
    }

    #[inline]
    pub fn vec1<T>(self, value: T) -> Vec<'a, T> {
        let mut vec = self.vec_with_capacity(1);
        vec.push(value);
        vec
    }

    #[inline]
    pub fn vec_from_iter<T, I: IntoIterator<Item = T>>(self, iter: I) -> Vec<'a, T> {
        Vec::from_iter_in(iter, self.allocator)
    }

    #[inline]
    pub fn str(self, value: &str) -> &'a str {
        String::from_str_in(value, self.allocator).into_bump_str()
    }

    #[inline]
    pub fn atom(self, value: &str) -> Atom<'a> {
        Atom::from(String::from_str_in(value, self.allocator).into_bump_str())
    }

    /// # SAFETY
    /// This method is completely unsound and should not be used.
    /// We need to remove all uses of it. Please don't add any more!
    /// <https://github.com/oxc-project/oxc/issues/3483>
    #[allow(clippy::missing_safety_doc)]
    #[inline]
    pub unsafe fn copy<T>(self, src: &T) -> T {
        // SAFETY: Not safe (see above)
        unsafe { std::mem::transmute_copy(src) }
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
    pub fn move_assignment_target(self, target: &mut AssignmentTarget<'a>) -> AssignmentTarget<'a> {
        let dummy =
            self.simple_assignment_target_identifier_reference(Span::default(), Atom::from(""));
        mem::replace(target, dummy.into())
    }

    #[inline]
    pub fn move_declaration(self, decl: &mut Declaration<'a>) -> Declaration<'a> {
        let empty_decl = self.variable_declaration(
            Span::default(),
            VariableDeclarationKind::Var,
            self.vec(),
            false,
        );
        let empty_decl = Declaration::VariableDeclaration(self.alloc(empty_decl));
        mem::replace(decl, empty_decl)
    }

    #[inline]
    pub fn move_variable_declaration(
        self,
        decl: &mut VariableDeclaration<'a>,
    ) -> VariableDeclaration<'a> {
        let empty_decl = self.variable_declaration(
            Span::default(),
            VariableDeclarationKind::Var,
            self.vec(),
            false,
        );
        mem::replace(decl, empty_decl)
    }

    pub fn move_array_expression_element(
        self,
        element: &mut ArrayExpressionElement<'a>,
    ) -> ArrayExpressionElement<'a> {
        let empty_element = self.array_expression_element_elision(Span::default());
        mem::replace(element, empty_element)
    }

    #[inline]
    pub fn move_vec<T>(self, vec: &mut Vec<'a, T>) -> Vec<'a, T> {
        mem::replace(vec, self.vec())
    }

    /* ---------- Constructors ---------- */

    /// `0`
    #[inline]
    pub fn number_0(self) -> Expression<'a> {
        self.expression_numeric_literal(Span::default(), 0.0, "0", NumberBase::Decimal)
    }

    /// `void 0`
    #[inline]
    pub fn void_0(self, span: Span) -> Expression<'a> {
        let num = self.number_0();
        Expression::UnaryExpression(self.alloc(self.unary_expression(
            span,
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
        self.formal_parameter(span, self.vec(), pattern, None, false, false)
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
        self.alloc(
            self.function(r#type, span, id, false, false, false, NONE, NONE, params, NONE, body),
        )
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
            self.vec(),
            None,
            ImportOrExportKind::Value,
            NONE,
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
            NONE,
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
