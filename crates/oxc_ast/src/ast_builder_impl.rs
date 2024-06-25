#![allow(
    clippy::fn_params_excessive_bools,
    clippy::must_use_candidate, // must_use_candidate is too annoying for this file
    clippy::too_many_arguments,
    clippy::unused_self,
)]

use std::mem;

use oxc_allocator::{Allocator, Box, String, Vec};
use oxc_span::{Atom, GetSpan, Span, SPAN};
use oxc_syntax::{
    number::{BigintBase, NumberBase},
    operator::UnaryOperator,
};

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
        let null_literal = NullLiteral::new(expr.span());
        let null_expr = self.literal_null_expression(null_literal);
        mem::replace(expr, null_expr)
    }

    #[inline]
    pub fn move_statement(self, stmt: &mut Statement<'a>) -> Statement<'a> {
        let empty_stmt = self.empty_statement(stmt.span());
        mem::replace(stmt, Statement::EmptyStatement(empty_stmt))
    }

    #[inline]
    pub fn move_statement_vec(self, stmts: &mut Vec<'a, Statement<'a>>) -> Vec<'a, Statement<'a>> {
        mem::replace(stmts, self.new_vec())
    }

    #[inline]
    pub fn move_assignment_target(self, target: &mut AssignmentTarget<'a>) -> AssignmentTarget<'a> {
        let ident = IdentifierReference::new(Span::default(), "".into());
        let dummy = self.simple_assignment_target_identifier(ident);
        mem::replace(target, dummy)
    }

    #[inline]
    pub fn move_declaration(self, decl: &mut Declaration<'a>) -> Declaration<'a> {
        let empty_decl = self.variable_declaration(
            Span::default(),
            VariableDeclarationKind::Var,
            self.new_vec(),
            false,
        );
        let empty_decl = Declaration::VariableDeclaration(empty_decl);
        mem::replace(decl, empty_decl)
    }

    /* ---------- Constructors ---------- */

    /// `void 0`
    #[inline]
    pub fn void_0(self) -> Expression<'a> {
        let left = self.number_literal(Span::default(), 0.0, "0", NumberBase::Decimal);
        let num = self.literal_number_expression(left);
        Expression::UnaryExpression(self.unary_expression(
            Span::default(),
            UnaryOperator::Void,
            num,
        ))
    }

    /* ---------- Literals ---------- */

    #[inline]
    pub fn number_literal(
        self,
        span: Span,
        value: f64,
        raw: &'a str,
        base: NumberBase,
    ) -> NumericLiteral<'a> {
        NumericLiteral { span, value, raw, base }
    }

    #[inline]
    pub fn bigint_literal(self, span: Span, raw: Atom<'a>, base: BigintBase) -> BigIntLiteral<'a> {
        BigIntLiteral { span, raw, base }
    }

    #[inline]
    pub fn literal_string_expression(self, literal: StringLiteral<'a>) -> Expression<'a> {
        Expression::StringLiteral(self.alloc(literal))
    }

    #[inline]
    pub fn literal_boolean_expression(self, literal: BooleanLiteral) -> Expression<'a> {
        Expression::BooleanLiteral(self.alloc(literal))
    }

    #[inline]
    pub fn literal_null_expression(self, literal: NullLiteral) -> Expression<'a> {
        Expression::NullLiteral(self.alloc(literal))
    }

    #[inline]
    pub fn literal_regexp_expression(self, literal: RegExpLiteral<'a>) -> Expression<'a> {
        Expression::RegExpLiteral(self.alloc(literal))
    }

    #[inline]
    pub fn literal_number_expression(self, literal: NumericLiteral<'a>) -> Expression<'a> {
        Expression::NumericLiteral(self.alloc(literal))
    }

    #[inline]
    pub fn literal_bigint_expression(self, literal: BigIntLiteral<'a>) -> Expression<'a> {
        Expression::BigIntLiteral(self.alloc(literal))
    }

    #[inline]
    pub fn literal_template_expression(self, literal: TemplateLiteral<'a>) -> Expression<'a> {
        Expression::TemplateLiteral(self.alloc(literal))
    }

    #[inline]
    pub fn identifier_reference_expression(self, ident: IdentifierReference<'a>) -> Expression<'a> {
        Expression::Identifier(self.alloc(ident))
    }

    /* ---------- Statements ---------- */

    #[inline]
    pub fn block(self, span: Span, body: Vec<'a, Statement<'a>>) -> Box<'a, BlockStatement<'a>> {
        self.alloc(BlockStatement::new(span, body))
    }

    #[inline]
    pub fn using_statement(
        self,
        span: Span,
        declarations: Vec<'a, VariableDeclarator<'a>>,
        is_await: bool,
    ) -> Statement<'a> {
        Statement::UsingDeclaration(self.alloc(UsingDeclaration { span, is_await, declarations }))
    }

    /* ---------- Expressions ---------- */

    #[inline]
    pub fn array_assignment_target_maybe_default(
        self,
        array: ArrayAssignmentTarget<'a>,
    ) -> AssignmentTargetMaybeDefault<'a> {
        AssignmentTargetMaybeDefault::ArrayAssignmentTarget(self.alloc(array))
    }

    #[inline]
    pub fn simple_assignment_target_identifier(
        self,
        ident: IdentifierReference<'a>,
    ) -> AssignmentTarget<'a> {
        AssignmentTarget::AssignmentTargetIdentifier(self.alloc(ident))
    }

    #[inline]
    pub fn simple_assignment_target_member_expression(
        self,
        expr: MemberExpression<'a>,
    ) -> AssignmentTarget<'a> {
        AssignmentTarget::from(expr)
    }

    #[inline]
    pub fn class_expression(self, class: Box<'a, Class<'a>>) -> Expression<'a> {
        Expression::ClassExpression(class)
    }

    #[inline]
    pub fn function_expression(self, function: Box<'a, Function<'a>>) -> Expression<'a> {
        Expression::FunctionExpression(function)
    }

    #[inline]
    pub fn member_expression(self, expr: MemberExpression<'a>) -> Expression<'a> {
        Expression::from(expr)
    }

    #[inline]
    pub fn computed_member(
        self,
        span: Span,
        object: Expression<'a>,
        expression: Expression<'a>,
        optional: bool, // for optional chaining
    ) -> MemberExpression<'a> {
        MemberExpression::ComputedMemberExpression(self.alloc(ComputedMemberExpression {
            span,
            object,
            expression,
            optional,
        }))
    }

    #[inline]
    pub fn static_member(
        self,
        span: Span,
        object: Expression<'a>,
        property: IdentifierName<'a>,
        optional: bool, // for optional chaining
    ) -> MemberExpression<'a> {
        MemberExpression::StaticMemberExpression(self.alloc(StaticMemberExpression {
            span,
            object,
            property,
            optional,
        }))
    }

    #[inline]
    pub fn private_field(
        self,
        span: Span,
        object: Expression<'a>,
        field: PrivateIdentifier<'a>,
        optional: bool,
    ) -> MemberExpression<'a> {
        MemberExpression::PrivateFieldExpression(self.alloc(PrivateFieldExpression {
            span,
            object,
            field,
            optional,
        }))
    }

    #[inline]
    pub fn template_literal_expression(
        self,
        template_literal: TemplateLiteral<'a>,
    ) -> Expression<'a> {
        Expression::TemplateLiteral(self.alloc(template_literal))
    }

    /* ---------- Functions ---------- */
    #[inline]
    pub fn function_declaration(self, func: Box<'a, Function<'a>>) -> Statement<'a> {
        Statement::FunctionDeclaration(func)
    }

    #[inline]
    pub fn plain_formal_parameter(
        self,
        span: Span,
        pattern: BindingPattern<'a>,
    ) -> Box<'a, FormalParameter<'a>> {
        self.formal_parameter(span, pattern, None, false, false, self.new_vec())
    }

    #[inline]
    pub fn plain_function(
        self,
        r#type: FunctionType,
        span: Span,
        id: Option<BindingIdentifier<'a>>,
        params: Box<'a, FormalParameters<'a>>,
        body: Option<Box<'a, FunctionBody<'a>>>,
    ) -> Box<'a, Function<'a>> {
        self.function(r#type, span, id, false, false, false, None, None, params, body, None)
    }

    /* ---------- Class ---------- */

    #[inline]
    pub fn class_declaration(self, class: Box<'a, Class<'a>>) -> Statement<'a> {
        Statement::ClassDeclaration(class)
    }

    #[inline]
    pub fn class_property(
        self,
        r#type: PropertyDefinitionType,
        span: Span,
        key: PropertyKey<'a>,
        value: Option<Expression<'a>>,
        computed: bool,
        r#static: bool,
        declare: bool,
        r#override: bool,
        optional: bool,
        definite: bool,
        readonly: bool,
        type_annotation: Option<Box<'a, TSTypeAnnotation<'a>>>,
        accessibility: Option<TSAccessibility>,
        decorators: Vec<'a, Decorator<'a>>,
    ) -> ClassElement<'a> {
        ClassElement::PropertyDefinition(self.alloc(PropertyDefinition {
            r#type,
            span,
            key,
            value,
            computed,
            r#static,
            declare,
            r#override,
            optional,
            definite,
            readonly,
            type_annotation,
            accessibility,
            decorators,
        }))
    }

    pub fn class_method(
        self,
        r#type: MethodDefinitionType,
        span: Span,
        key: PropertyKey<'a>,
        kind: MethodDefinitionKind,
        value: Box<'a, Function<'a>>,
        computed: bool,
        r#static: bool,
        r#override: bool,
        optional: bool,
        accessibility: Option<TSAccessibility>,
        decorators: Vec<'a, Decorator<'a>>,
    ) -> ClassElement<'a> {
        ClassElement::MethodDefinition(self.alloc(MethodDefinition {
            r#type,
            span,
            decorators,
            key,
            value,
            kind,
            computed,
            r#static,
            r#override,
            optional,
            accessibility,
        }))
    }

    #[inline]
    pub fn class_constructor(self, span: Span, value: Box<'a, Function<'a>>) -> ClassElement<'a> {
        ClassElement::MethodDefinition(self.alloc(MethodDefinition {
            r#type: MethodDefinitionType::MethodDefinition,
            span,
            key: self.property_key_expression(self.identifier_reference_expression(
                IdentifierReference::new(SPAN, "constructor".into()),
            )),
            kind: MethodDefinitionKind::Constructor,
            value,
            computed: false,
            r#static: false,
            r#override: false,
            optional: false,
            accessibility: None,
            decorators: self.new_vec(),
        }))
    }

    /* ---------- Patterns ---------- */

    #[inline]
    pub fn binding_pattern_identifier(
        self,
        identifier: BindingIdentifier<'a>,
    ) -> BindingPatternKind<'a> {
        BindingPatternKind::BindingIdentifier(self.alloc(identifier))
    }

    #[inline]
    pub fn rest_element(
        self,
        span: Span,
        argument: BindingPattern<'a>,
    ) -> Box<'a, BindingRestElement<'a>> {
        self.alloc(BindingRestElement { span, argument })
    }

    #[inline]
    pub fn property_key_identifier(self, ident: IdentifierName<'a>) -> PropertyKey<'a> {
        PropertyKey::StaticIdentifier(self.alloc(ident))
    }

    #[inline]
    pub fn property_key_private_identifier(self, ident: PrivateIdentifier<'a>) -> PropertyKey<'a> {
        PropertyKey::PrivateIdentifier(self.alloc(ident))
    }

    #[inline]
    pub fn property_key_expression(self, expr: Expression<'a>) -> PropertyKey<'a> {
        PropertyKey::from(expr)
    }

    /* ---------- Modules ---------- */

    #[inline]
    pub fn module_declaration(self, decl: ModuleDeclaration<'a>) -> Statement<'a> {
        Statement::from(decl)
    }

    #[inline]
    pub fn plain_export_named_declaration_declaration(
        self,
        span: Span,
        declaration: Declaration<'a>,
    ) -> Box<'a, ExportNamedDeclaration<'a>> {
        self.export_named_declaration(
            span,
            Some(declaration),
            self.new_vec(),
            None,
            ImportOrExportKind::Value,
            None,
        )
    }

    #[inline]
    pub fn plain_export_named_declaration(
        self,
        span: Span,
        specifiers: Vec<'a, ExportSpecifier<'a>>,
        source: Option<StringLiteral<'a>>,
    ) -> Box<'a, ExportNamedDeclaration<'a>> {
        self.export_named_declaration(
            span,
            None,
            specifiers,
            source,
            ImportOrExportKind::Value,
            None,
        )
    }

    /* ---------- TypeScript ---------- */

    #[inline]
    pub fn ts_type_operator_type(
        self,
        span: Span,
        operator: TSTypeOperatorOperator,
        type_annotation: TSType<'a>,
    ) -> TSType<'a> {
        TSType::TSTypeOperatorType(self.alloc(TSTypeOperator { span, operator, type_annotation }))
    }

    #[inline]
    pub fn ts_type_implement(
        self,
        span: Span,
        expression: TSTypeName<'a>,
        type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
    ) -> TSClassImplements<'a> {
        TSClassImplements { span, expression, type_parameters }
    }

    #[inline]
    pub fn ts_type_parameters(
        self,
        span: Span,
        params: Vec<'a, TSTypeParameter<'a>>,
    ) -> Box<'a, TSTypeParameterDeclaration<'a>> {
        self.alloc(TSTypeParameterDeclaration { span, params })
    }

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
    pub fn ts_type_arguments(
        self,
        span: Span,
        params: Vec<'a, TSType<'a>>,
    ) -> Box<'a, TSTypeParameterInstantiation<'a>> {
        self.alloc(TSTypeParameterInstantiation { span, params })
    }

    #[inline]
    pub fn ts_this_keyword(self, span: Span) -> TSType<'a> {
        TSType::TSThisType(self.alloc(TSThisType { span }))
    }

    #[inline]
    pub fn ts_bigint_keyword(self, span: Span) -> TSType<'a> {
        TSType::TSBigIntKeyword(self.alloc(TSBigIntKeyword { span }))
    }

    #[inline]
    pub fn ts_type_query_type(
        self,
        span: Span,
        expr_name: TSTypeQueryExprName<'a>,
        type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
    ) -> TSType<'a> {
        TSType::TSTypeQuery(self.alloc(TSTypeQuery { span, expr_name, type_parameters }))
    }

    #[inline]
    pub fn ts_enum_member_name_identifier(self, ident: IdentifierName<'a>) -> TSEnumMemberName<'a> {
        TSEnumMemberName::StaticIdentifier(self.alloc(ident))
    }

    #[inline]
    pub fn ts_enum_member_name_string_literal(
        self,
        lit: StringLiteral<'a>,
    ) -> TSEnumMemberName<'a> {
        TSEnumMemberName::StaticStringLiteral(self.alloc(lit))
    }

    #[inline]
    pub fn ts_enum_member_name_computed_property_name(
        self,
        expr: Expression<'a>,
    ) -> TSEnumMemberName<'a> {
        TSEnumMemberName::from(expr)
    }

    #[inline]
    pub fn ts_enum_member_name_number_literal(
        self,
        lit: NumericLiteral<'a>,
    ) -> TSEnumMemberName<'a> {
        TSEnumMemberName::StaticNumericLiteral(self.alloc(lit))
    }

    #[inline]
    pub fn ts_module_reference_external_module_reference(
        self,
        reference: TSExternalModuleReference<'a>,
    ) -> TSModuleReference<'a> {
        TSModuleReference::ExternalModuleReference(self.alloc(reference))
    }

    #[inline]
    pub fn ts_module_reference_type_name(self, reference: TSTypeName<'a>) -> TSModuleReference<'a> {
        TSModuleReference::from(reference)
    }

    #[inline]
    pub fn ts_type_predicate_name_this(self, ty: TSThisType) -> TSTypePredicateName<'a> {
        TSTypePredicateName::This(ty)
    }

    #[inline]
    pub fn ts_type_predicate_name_identifier(
        self,
        ident: IdentifierName<'a>,
    ) -> TSTypePredicateName<'a> {
        TSTypePredicateName::Identifier(self.alloc(ident))
    }

    #[inline]
    pub fn ts_type_query_expr_name_import_type(
        self,
        ty: TSImportType<'a>,
    ) -> TSTypeQueryExprName<'a> {
        TSTypeQueryExprName::TSImportType(self.alloc(ty))
    }

    #[inline]
    pub fn ts_type_query_expr_name_type_name(self, ty: TSTypeName<'a>) -> TSTypeQueryExprName<'a> {
        TSTypeQueryExprName::from(ty)
    }
}

impl<'a> From<Box<'a, TSConditionalType<'a>>> for TSType<'a> {
    fn from(it: Box<'a, TSConditionalType<'a>>) -> Self {
        TSType::TSConditionalType(it)
    }
}
