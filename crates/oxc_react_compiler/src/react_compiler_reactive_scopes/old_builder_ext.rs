//! Compatibility extension trait mirroring the pre-migration `AstBuilder` inherent
//! methods, re-expressed on the new `Type::new_*` / `Type::boxed` builder API.
//!
//! The `disable_old_builder` feature removes the deprecated inherent `ast.<node>(...)`
//! methods; this trait provides the same call surface (delegating to the new API) so
//! the ported codegen/splice back-end reads the same as its Babel-era structure.

#![allow(clippy::too_many_arguments, dead_code)]

use oxc_allocator::{ArenaBox, ArenaVec, IntoIn};
use oxc_ast::ast::*;
use oxc_ast::builder::AstBuilder;
use oxc_span::Span;
use oxc_str::{Ident, Str};
use oxc_syntax::number::NumberBase;

#[allow(clippy::too_many_arguments)]
pub(crate) trait OldBuilderExt<'a> {
    fn alloc<T>(&self, value: T) -> ArenaBox<'a, T>;
    fn vec<T>(&self) -> ArenaVec<'a, T>;
    fn vec1<T>(&self, value: T) -> ArenaVec<'a, T>;
    fn vec_from_iter<T, I: IntoIterator<Item = T>>(&self, iter: I) -> ArenaVec<'a, T>;
    fn expression_identifier<S1>(&self, span: Span, name: S1) -> Expression<'a>
    where
        S1: Into<Ident<'a>>;
    fn expression_string_literal<S1>(
        &self,
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
    ) -> Expression<'a>
    where
        S1: Into<Str<'a>>;
    fn expression_numeric_literal(
        &self,
        span: Span,
        value: f64,
        raw: Option<Str<'a>>,
        base: NumberBase,
    ) -> Expression<'a>;
    fn expression_null_literal(&self, span: Span) -> Expression<'a>;
    fn expression_boolean_literal(&self, span: Span, value: bool) -> Expression<'a>;
    fn expression_reg_exp_literal(
        &self,
        span: Span,
        regex: RegExp<'a>,
        raw: Option<Str<'a>>,
    ) -> Expression<'a>;
    fn expression_assignment(
        &self,
        span: Span,
        operator: AssignmentOperator,
        left: AssignmentTarget<'a>,
        right: Expression<'a>,
    ) -> Expression<'a>;
    fn expression_binary(
        &self,
        span: Span,
        left: Expression<'a>,
        operator: BinaryOperator,
        right: Expression<'a>,
    ) -> Expression<'a>;
    fn expression_logical(
        &self,
        span: Span,
        left: Expression<'a>,
        operator: LogicalOperator,
        right: Expression<'a>,
    ) -> Expression<'a>;
    fn expression_unary(
        &self,
        span: Span,
        operator: UnaryOperator,
        argument: Expression<'a>,
    ) -> Expression<'a>;
    fn expression_update(
        &self,
        span: Span,
        operator: UpdateOperator,
        prefix: bool,
        argument: SimpleAssignmentTarget<'a>,
    ) -> Expression<'a>;
    fn expression_call<T1>(
        &self,
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: ArenaVec<'a, Argument<'a>>,
        optional: bool,
    ) -> Expression<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>;
    fn expression_new<T1>(
        &self,
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: ArenaVec<'a, Argument<'a>>,
    ) -> Expression<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>;
    fn expression_conditional(
        &self,
        span: Span,
        test: Expression<'a>,
        consequent: Expression<'a>,
        alternate: Expression<'a>,
    ) -> Expression<'a>;
    fn expression_sequence(
        &self,
        span: Span,
        expressions: ArenaVec<'a, Expression<'a>>,
    ) -> Expression<'a>;
    fn expression_object(
        &self,
        span: Span,
        properties: ArenaVec<'a, ObjectPropertyKind<'a>>,
    ) -> Expression<'a>;
    fn expression_array(
        &self,
        span: Span,
        elements: ArenaVec<'a, ArrayExpressionElement<'a>>,
    ) -> Expression<'a>;
    fn expression_await(&self, span: Span, argument: Expression<'a>) -> Expression<'a>;
    fn expression_arrow_function<T1, T2, T3, T4>(
        &self,
        span: Span,
        expression: bool,
        r#async: bool,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        body: T4,
    ) -> Expression<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T4: IntoIn<'a, ArenaBox<'a, FunctionBody<'a>>>;
    fn expression_chain(&self, span: Span, expression: ChainElement<'a>) -> Expression<'a>;
    fn expression_meta_property(
        &self,
        span: Span,
        meta: IdentifierName<'a>,
        property: IdentifierName<'a>,
    ) -> Expression<'a>;
    fn expression_tagged_template<T1>(
        &self,
        span: Span,
        tag: Expression<'a>,
        type_arguments: T1,
        quasi: TemplateLiteral<'a>,
    ) -> Expression<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>;
    fn expression_ts_as(
        &self,
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
    ) -> Expression<'a>;
    fn expression_ts_satisfies(
        &self,
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
    ) -> Expression<'a>;
    fn statement_expression(&self, span: Span, expression: Expression<'a>) -> Statement<'a>;
    fn statement_return(&self, span: Span, argument: Option<Expression<'a>>) -> Statement<'a>;
    fn statement_if(
        &self,
        span: Span,
        test: Expression<'a>,
        consequent: Statement<'a>,
        alternate: Option<Statement<'a>>,
    ) -> Statement<'a>;
    fn statement_block(&self, span: Span, body: ArenaVec<'a, Statement<'a>>) -> Statement<'a>;
    fn statement_empty(&self, span: Span) -> Statement<'a>;
    fn statement_debugger(&self, span: Span) -> Statement<'a>;
    fn statement_break(&self, span: Span, label: Option<LabelIdentifier<'a>>) -> Statement<'a>;
    fn statement_continue(&self, span: Span, label: Option<LabelIdentifier<'a>>) -> Statement<'a>;
    fn statement_throw(&self, span: Span, argument: Expression<'a>) -> Statement<'a>;
    fn statement_try<T1, T2, T3>(
        &self,
        span: Span,
        block: T1,
        handler: T2,
        finalizer: T3,
    ) -> Statement<'a>
    where
        T1: IntoIn<'a, ArenaBox<'a, BlockStatement<'a>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, CatchClause<'a>>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, BlockStatement<'a>>>>;
    fn statement_switch(
        &self,
        span: Span,
        discriminant: Expression<'a>,
        cases: ArenaVec<'a, SwitchCase<'a>>,
    ) -> Statement<'a>;
    fn statement_labeled(
        &self,
        span: Span,
        label: LabelIdentifier<'a>,
        body: Statement<'a>,
    ) -> Statement<'a>;
    fn statement_do_while(
        &self,
        span: Span,
        body: Statement<'a>,
        test: Expression<'a>,
    ) -> Statement<'a>;
    fn statement_while(
        &self,
        span: Span,
        test: Expression<'a>,
        body: Statement<'a>,
    ) -> Statement<'a>;
    fn statement_for(
        &self,
        span: Span,
        init: Option<ForStatementInit<'a>>,
        test: Option<Expression<'a>>,
        update: Option<Expression<'a>>,
        body: Statement<'a>,
    ) -> Statement<'a>;
    fn statement_for_in(
        &self,
        span: Span,
        left: ForStatementLeft<'a>,
        right: Expression<'a>,
        body: Statement<'a>,
    ) -> Statement<'a>;
    fn statement_for_of(
        &self,
        span: Span,
        r#await: bool,
        left: ForStatementLeft<'a>,
        right: Expression<'a>,
        body: Statement<'a>,
    ) -> Statement<'a>;
    fn member_expression_computed(
        &self,
        span: Span,
        object: Expression<'a>,
        expression: Expression<'a>,
        optional: bool,
    ) -> MemberExpression<'a>;
    fn member_expression_static(
        &self,
        span: Span,
        object: Expression<'a>,
        property: IdentifierName<'a>,
        optional: bool,
    ) -> MemberExpression<'a>;
    fn binding_pattern_binding_identifier<S1>(&self, span: Span, name: S1) -> BindingPattern<'a>
    where
        S1: Into<Ident<'a>>;
    fn binding_pattern_object_pattern<T1>(
        &self,
        span: Span,
        properties: ArenaVec<'a, BindingProperty<'a>>,
        rest: T1,
    ) -> BindingPattern<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, BindingRestElement<'a>>>>;
    fn binding_pattern_array_pattern<T1>(
        &self,
        span: Span,
        elements: ArenaVec<'a, Option<BindingPattern<'a>>>,
        rest: T1,
    ) -> BindingPattern<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, BindingRestElement<'a>>>>;
    fn property_key_static_identifier<S1>(&self, span: Span, name: S1) -> PropertyKey<'a>
    where
        S1: Into<Ident<'a>>;
    fn array_expression_element_elision(&self, span: Span) -> ArrayExpressionElement<'a>;
    fn jsx_child_expression_container(
        &self,
        span: Span,
        expression: JSXExpression<'a>,
    ) -> JSXChild<'a>;
    fn jsx_child_text<S1>(&self, span: Span, value: S1, raw: Option<Str<'a>>) -> JSXChild<'a>
    where
        S1: Into<Str<'a>>;
    fn jsx_child_element<T1, T2>(
        &self,
        span: Span,
        opening_element: T1,
        children: ArenaVec<'a, JSXChild<'a>>,
        closing_element: T2,
    ) -> JSXChild<'a>
    where
        T1: IntoIn<'a, ArenaBox<'a, JSXOpeningElement<'a>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, JSXClosingElement<'a>>>>;
    fn jsx_child_fragment(
        &self,
        span: Span,
        opening_fragment: JSXOpeningFragment,
        children: ArenaVec<'a, JSXChild<'a>>,
        closing_fragment: JSXClosingFragment,
    ) -> JSXChild<'a>;
    fn jsx_element_name_namespaced_name(
        &self,
        span: Span,
        namespace: JSXIdentifier<'a>,
        name: JSXIdentifier<'a>,
    ) -> JSXElementName<'a>;
    fn jsx_element_name_member_expression(
        &self,
        span: Span,
        object: JSXMemberExpressionObject<'a>,
        property: JSXIdentifier<'a>,
    ) -> JSXElementName<'a>;
    fn jsx_element_name_identifier_reference<S1>(&self, span: Span, name: S1) -> JSXElementName<'a>
    where
        S1: Into<Ident<'a>>;
    fn jsx_element_name_identifier<S1>(&self, span: Span, name: S1) -> JSXElementName<'a>
    where
        S1: Into<Str<'a>>;
    fn jsx_member_expression_object_member_expression(
        &self,
        span: Span,
        object: JSXMemberExpressionObject<'a>,
        property: JSXIdentifier<'a>,
    ) -> JSXMemberExpressionObject<'a>;
    fn jsx_member_expression_object_identifier_reference<S1>(
        &self,
        span: Span,
        name: S1,
    ) -> JSXMemberExpressionObject<'a>
    where
        S1: Into<Ident<'a>>;
    fn jsx_attribute_value_string_literal<S1>(
        &self,
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
    ) -> JSXAttributeValue<'a>
    where
        S1: Into<Str<'a>>;
    fn jsx_attribute_value_expression_container(
        &self,
        span: Span,
        expression: JSXExpression<'a>,
    ) -> JSXAttributeValue<'a>;
    fn jsx_attribute_name_namespaced_name(
        &self,
        span: Span,
        namespace: JSXIdentifier<'a>,
        name: JSXIdentifier<'a>,
    ) -> JSXAttributeName<'a>;
    fn jsx_attribute_name_identifier<S1>(&self, span: Span, name: S1) -> JSXAttributeName<'a>
    where
        S1: Into<Str<'a>>;
    fn jsx_attribute_item_spread_attribute(
        &self,
        span: Span,
        argument: Expression<'a>,
    ) -> JSXAttributeItem<'a>;
    fn jsx_attribute_item_attribute(
        &self,
        span: Span,
        name: JSXAttributeName<'a>,
        value: Option<JSXAttributeValue<'a>>,
    ) -> JSXAttributeItem<'a>;
    fn variable_declarator<T1>(
        &self,
        span: Span,
        kind: VariableDeclarationKind,
        id: BindingPattern<'a>,
        type_annotation: T1,
        init: Option<Expression<'a>>,
        definite: bool,
    ) -> VariableDeclarator<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>;
    fn identifier_name<S1>(&self, span: Span, name: S1) -> IdentifierName<'a>
    where
        S1: Into<Ident<'a>>;
    fn label_identifier<S1>(&self, span: Span, name: S1) -> LabelIdentifier<'a>
    where
        S1: Into<Ident<'a>>;
    fn binding_identifier<S1>(&self, span: Span, name: S1) -> BindingIdentifier<'a>
    where
        S1: Into<Ident<'a>>;
    fn string_literal<S1>(&self, span: Span, value: S1, raw: Option<Str<'a>>) -> StringLiteral<'a>
    where
        S1: Into<Str<'a>>;
    fn directive<S1>(
        &self,
        span: Span,
        expression: StringLiteral<'a>,
        directive: S1,
    ) -> Directive<'a>
    where
        S1: Into<Str<'a>>;
    fn block_statement(&self, span: Span, body: ArenaVec<'a, Statement<'a>>) -> BlockStatement<'a>;
    fn switch_case(
        &self,
        span: Span,
        test: Option<Expression<'a>>,
        consequent: ArenaVec<'a, Statement<'a>>,
    ) -> SwitchCase<'a>;
    fn catch_clause<T1>(
        &self,
        span: Span,
        param: Option<CatchParameter<'a>>,
        body: T1,
    ) -> CatchClause<'a>
    where
        T1: IntoIn<'a, ArenaBox<'a, BlockStatement<'a>>>;
    fn catch_parameter<T1>(
        &self,
        span: Span,
        pattern: BindingPattern<'a>,
        type_annotation: T1,
    ) -> CatchParameter<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>;
    fn formal_parameter<T1, T2>(
        &self,
        span: Span,
        decorators: ArenaVec<'a, Decorator<'a>>,
        pattern: BindingPattern<'a>,
        type_annotation: T1,
        initializer: T2,
        optional: bool,
        accessibility: Option<TSAccessibility>,
        readonly: bool,
        r#override: bool,
    ) -> FormalParameter<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, Expression<'a>>>>;
    fn formal_parameter_rest<T1>(
        &self,
        span: Span,
        decorators: ArenaVec<'a, Decorator<'a>>,
        rest: BindingRestElement<'a>,
        type_annotation: T1,
    ) -> FormalParameterRest<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>;
    fn formal_parameters<T1>(
        &self,
        span: Span,
        kind: FormalParameterKind,
        items: ArenaVec<'a, FormalParameter<'a>>,
        rest: T1,
    ) -> FormalParameters<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, FormalParameterRest<'a>>>>;
    fn binding_rest_element(
        &self,
        span: Span,
        argument: BindingPattern<'a>,
    ) -> BindingRestElement<'a>;
    fn function<T1, T2, T3, T4, T5>(
        &self,
        span: Span,
        r#type: FunctionType,
        id: Option<BindingIdentifier<'a>>,
        generator: bool,
        r#async: bool,
        declare: bool,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
        body: T5,
    ) -> Function<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<ArenaBox<'a, FunctionBody<'a>>>>;
    fn object_property(
        &self,
        span: Span,
        kind: PropertyKind,
        key: PropertyKey<'a>,
        value: Expression<'a>,
        method: bool,
        shorthand: bool,
        computed: bool,
    ) -> ObjectProperty<'a>;
    fn binding_property(
        &self,
        span: Span,
        key: PropertyKey<'a>,
        value: BindingPattern<'a>,
        shorthand: bool,
        computed: bool,
    ) -> BindingProperty<'a>;
    fn spread_element(&self, span: Span, argument: Expression<'a>) -> SpreadElement<'a>;
    fn template_literal(
        &self,
        span: Span,
        quasis: ArenaVec<'a, TemplateElement<'a>>,
        expressions: ArenaVec<'a, Expression<'a>>,
    ) -> TemplateLiteral<'a>;
    fn template_element(
        &self,
        span: Span,
        value: TemplateElementValue<'a>,
        tail: bool,
    ) -> TemplateElement<'a>;
    fn jsx_identifier<S1>(&self, span: Span, name: S1) -> JSXIdentifier<'a>
    where
        S1: Into<Str<'a>>;
    fn jsx_opening_element<T1>(
        &self,
        span: Span,
        name: JSXElementName<'a>,
        type_arguments: T1,
        attributes: ArenaVec<'a, JSXAttributeItem<'a>>,
    ) -> JSXOpeningElement<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>;
    fn jsx_opening_fragment(&self, span: Span) -> JSXOpeningFragment;
    fn jsx_closing_element(&self, span: Span, name: JSXElementName<'a>) -> JSXClosingElement<'a>;
    fn jsx_closing_fragment(&self, span: Span) -> JSXClosingFragment;
    fn jsx_element<T1, T2>(
        &self,
        span: Span,
        opening_element: T1,
        children: ArenaVec<'a, JSXChild<'a>>,
        closing_element: T2,
    ) -> JSXElement<'a>
    where
        T1: IntoIn<'a, ArenaBox<'a, JSXOpeningElement<'a>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, JSXClosingElement<'a>>>>;
    fn jsx_fragment(
        &self,
        span: Span,
        opening_fragment: JSXOpeningFragment,
        children: ArenaVec<'a, JSXChild<'a>>,
        closing_fragment: JSXClosingFragment,
    ) -> JSXFragment<'a>;
    fn alloc_variable_declaration(
        &self,
        span: Span,
        kind: VariableDeclarationKind,
        declarations: ArenaVec<'a, VariableDeclarator<'a>>,
        declare: bool,
    ) -> ArenaBox<'a, VariableDeclaration<'a>>;
    fn alloc_static_member_expression(
        &self,
        span: Span,
        object: Expression<'a>,
        property: IdentifierName<'a>,
        optional: bool,
    ) -> ArenaBox<'a, StaticMemberExpression<'a>>;
    fn alloc_computed_member_expression(
        &self,
        span: Span,
        object: Expression<'a>,
        expression: Expression<'a>,
        optional: bool,
    ) -> ArenaBox<'a, ComputedMemberExpression<'a>>;
    fn alloc_call_expression<T1>(
        &self,
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: ArenaVec<'a, Argument<'a>>,
        optional: bool,
    ) -> ArenaBox<'a, CallExpression<'a>>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>;
    fn alloc_identifier_reference<S1>(
        &self,
        span: Span,
        name: S1,
    ) -> ArenaBox<'a, IdentifierReference<'a>>
    where
        S1: Into<Ident<'a>>;
    fn alloc_formal_parameters<T1>(
        &self,
        span: Span,
        kind: FormalParameterKind,
        items: ArenaVec<'a, FormalParameter<'a>>,
        rest: T1,
    ) -> ArenaBox<'a, FormalParameters<'a>>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, FormalParameterRest<'a>>>>;
    fn alloc_function_body(
        &self,
        span: Span,
        directives: ArenaVec<'a, Directive<'a>>,
        statements: ArenaVec<'a, Statement<'a>>,
    ) -> ArenaBox<'a, FunctionBody<'a>>;
    fn alloc_function<T1, T2, T3, T4, T5>(
        &self,
        span: Span,
        r#type: FunctionType,
        id: Option<BindingIdentifier<'a>>,
        generator: bool,
        r#async: bool,
        declare: bool,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
        body: T5,
    ) -> ArenaBox<'a, Function<'a>>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<ArenaBox<'a, FunctionBody<'a>>>>;
    fn alloc_spread_element(
        &self,
        span: Span,
        argument: Expression<'a>,
    ) -> ArenaBox<'a, SpreadElement<'a>>;
    fn alloc_jsx_text<S1>(
        &self,
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
    ) -> ArenaBox<'a, JSXText<'a>>
    where
        S1: Into<Str<'a>>;
    fn alloc_arrow_function_expression<T1, T2, T3, T4>(
        &self,
        span: Span,
        expression: bool,
        r#async: bool,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        body: T4,
    ) -> ArenaBox<'a, ArrowFunctionExpression<'a>>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T4: IntoIn<'a, ArenaBox<'a, FunctionBody<'a>>>;
    fn alloc_import_declaration<T1>(
        &self,
        span: Span,
        specifiers: Option<ArenaVec<'a, ImportDeclarationSpecifier<'a>>>,
        source: StringLiteral<'a>,
        phase: Option<ImportPhase>,
        with_clause: T1,
        import_kind: ImportOrExportKind,
    ) -> ArenaBox<'a, ImportDeclaration<'a>>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, WithClause<'a>>>>;
    fn alloc_import_specifier(
        &self,
        span: Span,
        imported: ModuleExportName<'a>,
        local: BindingIdentifier<'a>,
        import_kind: ImportOrExportKind,
    ) -> ArenaBox<'a, ImportSpecifier<'a>>;
    fn alloc_export_named_declaration<T1>(
        &self,
        span: Span,
        declaration: Option<Declaration<'a>>,
        specifiers: ArenaVec<'a, ExportSpecifier<'a>>,
        source: Option<StringLiteral<'a>>,
        export_kind: ImportOrExportKind,
        with_clause: T1,
    ) -> ArenaBox<'a, ExportNamedDeclaration<'a>>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, WithClause<'a>>>>;
    fn alloc_export_default_declaration(
        &self,
        span: Span,
        declaration: ExportDefaultDeclarationKind<'a>,
    ) -> ArenaBox<'a, ExportDefaultDeclaration<'a>>;
}

#[allow(clippy::too_many_arguments, clippy::inline_always)]
impl<'a> OldBuilderExt<'a> for AstBuilder<'a> {
    #[inline(always)]
    fn alloc<T>(&self, value: T) -> ArenaBox<'a, T> {
        ArenaBox::new_in(value, self)
    }
    #[inline(always)]
    fn vec<T>(&self) -> ArenaVec<'a, T> {
        ArenaVec::new_in(self)
    }
    #[inline(always)]
    fn vec1<T>(&self, value: T) -> ArenaVec<'a, T> {
        ArenaVec::from_value_in(value, self)
    }
    #[inline(always)]
    fn vec_from_iter<T, I: IntoIterator<Item = T>>(&self, iter: I) -> ArenaVec<'a, T> {
        ArenaVec::from_iter_in(iter, self)
    }
    fn expression_identifier<S1>(&self, span: Span, name: S1) -> Expression<'a>
    where
        S1: Into<Ident<'a>>,
    {
        Expression::new_identifier(span, name, self)
    }
    fn expression_string_literal<S1>(
        &self,
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
    ) -> Expression<'a>
    where
        S1: Into<Str<'a>>,
    {
        Expression::new_string_literal(span, value, raw, self)
    }
    fn expression_numeric_literal(
        &self,
        span: Span,
        value: f64,
        raw: Option<Str<'a>>,
        base: NumberBase,
    ) -> Expression<'a> {
        Expression::new_numeric_literal(span, value, raw, base, self)
    }
    fn expression_null_literal(&self, span: Span) -> Expression<'a> {
        Expression::new_null_literal(span, self)
    }
    fn expression_boolean_literal(&self, span: Span, value: bool) -> Expression<'a> {
        Expression::new_boolean_literal(span, value, self)
    }
    fn expression_reg_exp_literal(
        &self,
        span: Span,
        regex: RegExp<'a>,
        raw: Option<Str<'a>>,
    ) -> Expression<'a> {
        Expression::new_reg_exp_literal(span, regex, raw, self)
    }
    fn expression_assignment(
        &self,
        span: Span,
        operator: AssignmentOperator,
        left: AssignmentTarget<'a>,
        right: Expression<'a>,
    ) -> Expression<'a> {
        Expression::new_assignment_expression(span, operator, left, right, self)
    }
    fn expression_binary(
        &self,
        span: Span,
        left: Expression<'a>,
        operator: BinaryOperator,
        right: Expression<'a>,
    ) -> Expression<'a> {
        Expression::new_binary_expression(span, left, operator, right, self)
    }
    fn expression_logical(
        &self,
        span: Span,
        left: Expression<'a>,
        operator: LogicalOperator,
        right: Expression<'a>,
    ) -> Expression<'a> {
        Expression::new_logical_expression(span, left, operator, right, self)
    }
    fn expression_unary(
        &self,
        span: Span,
        operator: UnaryOperator,
        argument: Expression<'a>,
    ) -> Expression<'a> {
        Expression::new_unary_expression(span, operator, argument, self)
    }
    fn expression_update(
        &self,
        span: Span,
        operator: UpdateOperator,
        prefix: bool,
        argument: SimpleAssignmentTarget<'a>,
    ) -> Expression<'a> {
        Expression::new_update_expression(span, operator, prefix, argument, self)
    }
    fn expression_call<T1>(
        &self,
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: ArenaVec<'a, Argument<'a>>,
        optional: bool,
    ) -> Expression<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Expression::new_call_expression(span, callee, type_arguments, arguments, optional, self)
    }
    fn expression_new<T1>(
        &self,
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: ArenaVec<'a, Argument<'a>>,
    ) -> Expression<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Expression::new_new_expression(span, callee, type_arguments, arguments, self)
    }
    fn expression_conditional(
        &self,
        span: Span,
        test: Expression<'a>,
        consequent: Expression<'a>,
        alternate: Expression<'a>,
    ) -> Expression<'a> {
        Expression::new_conditional_expression(span, test, consequent, alternate, self)
    }
    fn expression_sequence(
        &self,
        span: Span,
        expressions: ArenaVec<'a, Expression<'a>>,
    ) -> Expression<'a> {
        Expression::new_sequence_expression(span, expressions, self)
    }
    fn expression_object(
        &self,
        span: Span,
        properties: ArenaVec<'a, ObjectPropertyKind<'a>>,
    ) -> Expression<'a> {
        Expression::new_object_expression(span, properties, self)
    }
    fn expression_array(
        &self,
        span: Span,
        elements: ArenaVec<'a, ArrayExpressionElement<'a>>,
    ) -> Expression<'a> {
        Expression::new_array_expression(span, elements, self)
    }
    fn expression_await(&self, span: Span, argument: Expression<'a>) -> Expression<'a> {
        Expression::new_await_expression(span, argument, self)
    }
    fn expression_arrow_function<T1, T2, T3, T4>(
        &self,
        span: Span,
        expression: bool,
        r#async: bool,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        body: T4,
    ) -> Expression<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T4: IntoIn<'a, ArenaBox<'a, FunctionBody<'a>>>,
    {
        Expression::new_arrow_function_expression(
            span,
            expression,
            r#async,
            type_parameters,
            params,
            return_type,
            body,
            self,
        )
    }
    fn expression_chain(&self, span: Span, expression: ChainElement<'a>) -> Expression<'a> {
        Expression::new_chain_expression(span, expression, self)
    }
    fn expression_meta_property(
        &self,
        span: Span,
        meta: IdentifierName<'a>,
        property: IdentifierName<'a>,
    ) -> Expression<'a> {
        Expression::new_meta_property(span, meta, property, self)
    }
    fn expression_tagged_template<T1>(
        &self,
        span: Span,
        tag: Expression<'a>,
        type_arguments: T1,
        quasi: TemplateLiteral<'a>,
    ) -> Expression<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Expression::new_tagged_template_expression(span, tag, type_arguments, quasi, self)
    }
    fn expression_ts_as(
        &self,
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
    ) -> Expression<'a> {
        Expression::new_ts_as_expression(span, expression, type_annotation, self)
    }
    fn expression_ts_satisfies(
        &self,
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
    ) -> Expression<'a> {
        Expression::new_ts_satisfies_expression(span, expression, type_annotation, self)
    }
    fn statement_expression(&self, span: Span, expression: Expression<'a>) -> Statement<'a> {
        Statement::new_expression_statement(span, expression, self)
    }
    fn statement_return(&self, span: Span, argument: Option<Expression<'a>>) -> Statement<'a> {
        Statement::new_return_statement(span, argument, self)
    }
    fn statement_if(
        &self,
        span: Span,
        test: Expression<'a>,
        consequent: Statement<'a>,
        alternate: Option<Statement<'a>>,
    ) -> Statement<'a> {
        Statement::new_if_statement(span, test, consequent, alternate, self)
    }
    fn statement_block(&self, span: Span, body: ArenaVec<'a, Statement<'a>>) -> Statement<'a> {
        Statement::new_block_statement(span, body, self)
    }
    fn statement_empty(&self, span: Span) -> Statement<'a> {
        Statement::new_empty_statement(span, self)
    }
    fn statement_debugger(&self, span: Span) -> Statement<'a> {
        Statement::new_debugger_statement(span, self)
    }
    fn statement_break(&self, span: Span, label: Option<LabelIdentifier<'a>>) -> Statement<'a> {
        Statement::new_break_statement(span, label, self)
    }
    fn statement_continue(&self, span: Span, label: Option<LabelIdentifier<'a>>) -> Statement<'a> {
        Statement::new_continue_statement(span, label, self)
    }
    fn statement_throw(&self, span: Span, argument: Expression<'a>) -> Statement<'a> {
        Statement::new_throw_statement(span, argument, self)
    }
    fn statement_try<T1, T2, T3>(
        &self,
        span: Span,
        block: T1,
        handler: T2,
        finalizer: T3,
    ) -> Statement<'a>
    where
        T1: IntoIn<'a, ArenaBox<'a, BlockStatement<'a>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, CatchClause<'a>>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, BlockStatement<'a>>>>,
    {
        Statement::new_try_statement(span, block, handler, finalizer, self)
    }
    fn statement_switch(
        &self,
        span: Span,
        discriminant: Expression<'a>,
        cases: ArenaVec<'a, SwitchCase<'a>>,
    ) -> Statement<'a> {
        Statement::new_switch_statement(span, discriminant, cases, self)
    }
    fn statement_labeled(
        &self,
        span: Span,
        label: LabelIdentifier<'a>,
        body: Statement<'a>,
    ) -> Statement<'a> {
        Statement::new_labeled_statement(span, label, body, self)
    }
    fn statement_do_while(
        &self,
        span: Span,
        body: Statement<'a>,
        test: Expression<'a>,
    ) -> Statement<'a> {
        Statement::new_do_while_statement(span, body, test, self)
    }
    fn statement_while(
        &self,
        span: Span,
        test: Expression<'a>,
        body: Statement<'a>,
    ) -> Statement<'a> {
        Statement::new_while_statement(span, test, body, self)
    }
    fn statement_for(
        &self,
        span: Span,
        init: Option<ForStatementInit<'a>>,
        test: Option<Expression<'a>>,
        update: Option<Expression<'a>>,
        body: Statement<'a>,
    ) -> Statement<'a> {
        Statement::new_for_statement(span, init, test, update, body, self)
    }
    fn statement_for_in(
        &self,
        span: Span,
        left: ForStatementLeft<'a>,
        right: Expression<'a>,
        body: Statement<'a>,
    ) -> Statement<'a> {
        Statement::new_for_in_statement(span, left, right, body, self)
    }
    fn statement_for_of(
        &self,
        span: Span,
        r#await: bool,
        left: ForStatementLeft<'a>,
        right: Expression<'a>,
        body: Statement<'a>,
    ) -> Statement<'a> {
        Statement::new_for_of_statement(span, r#await, left, right, body, self)
    }
    fn member_expression_computed(
        &self,
        span: Span,
        object: Expression<'a>,
        expression: Expression<'a>,
        optional: bool,
    ) -> MemberExpression<'a> {
        MemberExpression::new_computed_member_expression(span, object, expression, optional, self)
    }
    fn member_expression_static(
        &self,
        span: Span,
        object: Expression<'a>,
        property: IdentifierName<'a>,
        optional: bool,
    ) -> MemberExpression<'a> {
        MemberExpression::new_static_member_expression(span, object, property, optional, self)
    }
    fn binding_pattern_binding_identifier<S1>(&self, span: Span, name: S1) -> BindingPattern<'a>
    where
        S1: Into<Ident<'a>>,
    {
        BindingPattern::new_binding_identifier(span, name, self)
    }
    fn binding_pattern_object_pattern<T1>(
        &self,
        span: Span,
        properties: ArenaVec<'a, BindingProperty<'a>>,
        rest: T1,
    ) -> BindingPattern<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, BindingRestElement<'a>>>>,
    {
        BindingPattern::new_object_pattern(span, properties, rest, self)
    }
    fn binding_pattern_array_pattern<T1>(
        &self,
        span: Span,
        elements: ArenaVec<'a, Option<BindingPattern<'a>>>,
        rest: T1,
    ) -> BindingPattern<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, BindingRestElement<'a>>>>,
    {
        BindingPattern::new_array_pattern(span, elements, rest, self)
    }
    fn property_key_static_identifier<S1>(&self, span: Span, name: S1) -> PropertyKey<'a>
    where
        S1: Into<Ident<'a>>,
    {
        PropertyKey::new_static_identifier(span, name, self)
    }
    fn array_expression_element_elision(&self, span: Span) -> ArrayExpressionElement<'a> {
        ArrayExpressionElement::new_elision(span, self)
    }
    fn jsx_child_expression_container(
        &self,
        span: Span,
        expression: JSXExpression<'a>,
    ) -> JSXChild<'a> {
        JSXChild::new_expression_container(span, expression, self)
    }
    fn jsx_child_text<S1>(&self, span: Span, value: S1, raw: Option<Str<'a>>) -> JSXChild<'a>
    where
        S1: Into<Str<'a>>,
    {
        JSXChild::new_text(span, value, raw, self)
    }
    fn jsx_child_element<T1, T2>(
        &self,
        span: Span,
        opening_element: T1,
        children: ArenaVec<'a, JSXChild<'a>>,
        closing_element: T2,
    ) -> JSXChild<'a>
    where
        T1: IntoIn<'a, ArenaBox<'a, JSXOpeningElement<'a>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, JSXClosingElement<'a>>>>,
    {
        JSXChild::new_element(span, opening_element, children, closing_element, self)
    }
    fn jsx_child_fragment(
        &self,
        span: Span,
        opening_fragment: JSXOpeningFragment,
        children: ArenaVec<'a, JSXChild<'a>>,
        closing_fragment: JSXClosingFragment,
    ) -> JSXChild<'a> {
        JSXChild::new_fragment(span, opening_fragment, children, closing_fragment, self)
    }
    fn jsx_element_name_namespaced_name(
        &self,
        span: Span,
        namespace: JSXIdentifier<'a>,
        name: JSXIdentifier<'a>,
    ) -> JSXElementName<'a> {
        JSXElementName::new_namespaced_name(span, namespace, name, self)
    }
    fn jsx_element_name_member_expression(
        &self,
        span: Span,
        object: JSXMemberExpressionObject<'a>,
        property: JSXIdentifier<'a>,
    ) -> JSXElementName<'a> {
        JSXElementName::new_member_expression(span, object, property, self)
    }
    fn jsx_element_name_identifier_reference<S1>(&self, span: Span, name: S1) -> JSXElementName<'a>
    where
        S1: Into<Ident<'a>>,
    {
        JSXElementName::new_identifier_reference(span, name, self)
    }
    fn jsx_element_name_identifier<S1>(&self, span: Span, name: S1) -> JSXElementName<'a>
    where
        S1: Into<Str<'a>>,
    {
        JSXElementName::new_identifier(span, name, self)
    }
    fn jsx_member_expression_object_member_expression(
        &self,
        span: Span,
        object: JSXMemberExpressionObject<'a>,
        property: JSXIdentifier<'a>,
    ) -> JSXMemberExpressionObject<'a> {
        JSXMemberExpressionObject::new_member_expression(span, object, property, self)
    }
    fn jsx_member_expression_object_identifier_reference<S1>(
        &self,
        span: Span,
        name: S1,
    ) -> JSXMemberExpressionObject<'a>
    where
        S1: Into<Ident<'a>>,
    {
        JSXMemberExpressionObject::new_identifier_reference(span, name, self)
    }
    fn jsx_attribute_value_string_literal<S1>(
        &self,
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
    ) -> JSXAttributeValue<'a>
    where
        S1: Into<Str<'a>>,
    {
        JSXAttributeValue::new_string_literal(span, value, raw, self)
    }
    fn jsx_attribute_value_expression_container(
        &self,
        span: Span,
        expression: JSXExpression<'a>,
    ) -> JSXAttributeValue<'a> {
        JSXAttributeValue::new_expression_container(span, expression, self)
    }
    fn jsx_attribute_name_namespaced_name(
        &self,
        span: Span,
        namespace: JSXIdentifier<'a>,
        name: JSXIdentifier<'a>,
    ) -> JSXAttributeName<'a> {
        JSXAttributeName::new_namespaced_name(span, namespace, name, self)
    }
    fn jsx_attribute_name_identifier<S1>(&self, span: Span, name: S1) -> JSXAttributeName<'a>
    where
        S1: Into<Str<'a>>,
    {
        JSXAttributeName::new_identifier(span, name, self)
    }
    fn jsx_attribute_item_spread_attribute(
        &self,
        span: Span,
        argument: Expression<'a>,
    ) -> JSXAttributeItem<'a> {
        JSXAttributeItem::new_spread_attribute(span, argument, self)
    }
    fn jsx_attribute_item_attribute(
        &self,
        span: Span,
        name: JSXAttributeName<'a>,
        value: Option<JSXAttributeValue<'a>>,
    ) -> JSXAttributeItem<'a> {
        JSXAttributeItem::new_attribute(span, name, value, self)
    }
    fn variable_declarator<T1>(
        &self,
        span: Span,
        kind: VariableDeclarationKind,
        id: BindingPattern<'a>,
        type_annotation: T1,
        init: Option<Expression<'a>>,
        definite: bool,
    ) -> VariableDeclarator<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        VariableDeclarator::new(span, kind, id, type_annotation, init, definite, self)
    }
    fn identifier_name<S1>(&self, span: Span, name: S1) -> IdentifierName<'a>
    where
        S1: Into<Ident<'a>>,
    {
        IdentifierName::new(span, name, self)
    }
    fn label_identifier<S1>(&self, span: Span, name: S1) -> LabelIdentifier<'a>
    where
        S1: Into<Ident<'a>>,
    {
        LabelIdentifier::new(span, name, self)
    }
    fn binding_identifier<S1>(&self, span: Span, name: S1) -> BindingIdentifier<'a>
    where
        S1: Into<Ident<'a>>,
    {
        BindingIdentifier::new(span, name, self)
    }
    fn string_literal<S1>(&self, span: Span, value: S1, raw: Option<Str<'a>>) -> StringLiteral<'a>
    where
        S1: Into<Str<'a>>,
    {
        StringLiteral::new(span, value, raw, self)
    }
    fn directive<S1>(
        &self,
        span: Span,
        expression: StringLiteral<'a>,
        directive: S1,
    ) -> Directive<'a>
    where
        S1: Into<Str<'a>>,
    {
        Directive::new(span, expression, directive, self)
    }
    fn block_statement(&self, span: Span, body: ArenaVec<'a, Statement<'a>>) -> BlockStatement<'a> {
        BlockStatement::new(span, body, self)
    }
    fn switch_case(
        &self,
        span: Span,
        test: Option<Expression<'a>>,
        consequent: ArenaVec<'a, Statement<'a>>,
    ) -> SwitchCase<'a> {
        SwitchCase::new(span, test, consequent, self)
    }
    fn catch_clause<T1>(
        &self,
        span: Span,
        param: Option<CatchParameter<'a>>,
        body: T1,
    ) -> CatchClause<'a>
    where
        T1: IntoIn<'a, ArenaBox<'a, BlockStatement<'a>>>,
    {
        CatchClause::new(span, param, body, self)
    }
    fn catch_parameter<T1>(
        &self,
        span: Span,
        pattern: BindingPattern<'a>,
        type_annotation: T1,
    ) -> CatchParameter<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        CatchParameter::new(span, pattern, type_annotation, self)
    }
    fn formal_parameter<T1, T2>(
        &self,
        span: Span,
        decorators: ArenaVec<'a, Decorator<'a>>,
        pattern: BindingPattern<'a>,
        type_annotation: T1,
        initializer: T2,
        optional: bool,
        accessibility: Option<TSAccessibility>,
        readonly: bool,
        r#override: bool,
    ) -> FormalParameter<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, Expression<'a>>>>,
    {
        FormalParameter::new(
            span,
            decorators,
            pattern,
            type_annotation,
            initializer,
            optional,
            accessibility,
            readonly,
            r#override,
            self,
        )
    }
    fn formal_parameter_rest<T1>(
        &self,
        span: Span,
        decorators: ArenaVec<'a, Decorator<'a>>,
        rest: BindingRestElement<'a>,
        type_annotation: T1,
    ) -> FormalParameterRest<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        FormalParameterRest::new(span, decorators, rest, type_annotation, self)
    }
    fn formal_parameters<T1>(
        &self,
        span: Span,
        kind: FormalParameterKind,
        items: ArenaVec<'a, FormalParameter<'a>>,
        rest: T1,
    ) -> FormalParameters<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, FormalParameterRest<'a>>>>,
    {
        FormalParameters::new(span, kind, items, rest, self)
    }
    fn binding_rest_element(
        &self,
        span: Span,
        argument: BindingPattern<'a>,
    ) -> BindingRestElement<'a> {
        BindingRestElement::new(span, argument, self)
    }
    fn function<T1, T2, T3, T4, T5>(
        &self,
        span: Span,
        r#type: FunctionType,
        id: Option<BindingIdentifier<'a>>,
        generator: bool,
        r#async: bool,
        declare: bool,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
        body: T5,
    ) -> Function<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<ArenaBox<'a, FunctionBody<'a>>>>,
    {
        Function::new(
            span,
            r#type,
            id,
            generator,
            r#async,
            declare,
            type_parameters,
            this_param,
            params,
            return_type,
            body,
            self,
        )
    }
    fn object_property(
        &self,
        span: Span,
        kind: PropertyKind,
        key: PropertyKey<'a>,
        value: Expression<'a>,
        method: bool,
        shorthand: bool,
        computed: bool,
    ) -> ObjectProperty<'a> {
        ObjectProperty::new(span, kind, key, value, method, shorthand, computed, self)
    }
    fn binding_property(
        &self,
        span: Span,
        key: PropertyKey<'a>,
        value: BindingPattern<'a>,
        shorthand: bool,
        computed: bool,
    ) -> BindingProperty<'a> {
        BindingProperty::new(span, key, value, shorthand, computed, self)
    }
    fn spread_element(&self, span: Span, argument: Expression<'a>) -> SpreadElement<'a> {
        SpreadElement::new(span, argument, self)
    }
    fn template_literal(
        &self,
        span: Span,
        quasis: ArenaVec<'a, TemplateElement<'a>>,
        expressions: ArenaVec<'a, Expression<'a>>,
    ) -> TemplateLiteral<'a> {
        TemplateLiteral::new(span, quasis, expressions, self)
    }
    fn template_element(
        &self,
        span: Span,
        value: TemplateElementValue<'a>,
        tail: bool,
    ) -> TemplateElement<'a> {
        TemplateElement::new(span, value, tail, self)
    }
    fn jsx_identifier<S1>(&self, span: Span, name: S1) -> JSXIdentifier<'a>
    where
        S1: Into<Str<'a>>,
    {
        JSXIdentifier::new(span, name, self)
    }
    fn jsx_opening_element<T1>(
        &self,
        span: Span,
        name: JSXElementName<'a>,
        type_arguments: T1,
        attributes: ArenaVec<'a, JSXAttributeItem<'a>>,
    ) -> JSXOpeningElement<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        JSXOpeningElement::new(span, name, type_arguments, attributes, self)
    }
    fn jsx_opening_fragment(&self, span: Span) -> JSXOpeningFragment {
        JSXOpeningFragment::new(span, self)
    }
    fn jsx_closing_element(&self, span: Span, name: JSXElementName<'a>) -> JSXClosingElement<'a> {
        JSXClosingElement::new(span, name, self)
    }
    fn jsx_closing_fragment(&self, span: Span) -> JSXClosingFragment {
        JSXClosingFragment::new(span, self)
    }
    fn jsx_element<T1, T2>(
        &self,
        span: Span,
        opening_element: T1,
        children: ArenaVec<'a, JSXChild<'a>>,
        closing_element: T2,
    ) -> JSXElement<'a>
    where
        T1: IntoIn<'a, ArenaBox<'a, JSXOpeningElement<'a>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, JSXClosingElement<'a>>>>,
    {
        JSXElement::new(span, opening_element, children, closing_element, self)
    }
    fn jsx_fragment(
        &self,
        span: Span,
        opening_fragment: JSXOpeningFragment,
        children: ArenaVec<'a, JSXChild<'a>>,
        closing_fragment: JSXClosingFragment,
    ) -> JSXFragment<'a> {
        JSXFragment::new(span, opening_fragment, children, closing_fragment, self)
    }
    fn alloc_variable_declaration(
        &self,
        span: Span,
        kind: VariableDeclarationKind,
        declarations: ArenaVec<'a, VariableDeclarator<'a>>,
        declare: bool,
    ) -> ArenaBox<'a, VariableDeclaration<'a>> {
        VariableDeclaration::boxed(span, kind, declarations, declare, self)
    }
    fn alloc_static_member_expression(
        &self,
        span: Span,
        object: Expression<'a>,
        property: IdentifierName<'a>,
        optional: bool,
    ) -> ArenaBox<'a, StaticMemberExpression<'a>> {
        StaticMemberExpression::boxed(span, object, property, optional, self)
    }
    fn alloc_computed_member_expression(
        &self,
        span: Span,
        object: Expression<'a>,
        expression: Expression<'a>,
        optional: bool,
    ) -> ArenaBox<'a, ComputedMemberExpression<'a>> {
        ComputedMemberExpression::boxed(span, object, expression, optional, self)
    }
    fn alloc_call_expression<T1>(
        &self,
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: ArenaVec<'a, Argument<'a>>,
        optional: bool,
    ) -> ArenaBox<'a, CallExpression<'a>>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        CallExpression::boxed(span, callee, type_arguments, arguments, optional, self)
    }
    fn alloc_identifier_reference<S1>(
        &self,
        span: Span,
        name: S1,
    ) -> ArenaBox<'a, IdentifierReference<'a>>
    where
        S1: Into<Ident<'a>>,
    {
        IdentifierReference::boxed(span, name, self)
    }
    fn alloc_formal_parameters<T1>(
        &self,
        span: Span,
        kind: FormalParameterKind,
        items: ArenaVec<'a, FormalParameter<'a>>,
        rest: T1,
    ) -> ArenaBox<'a, FormalParameters<'a>>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, FormalParameterRest<'a>>>>,
    {
        FormalParameters::boxed(span, kind, items, rest, self)
    }
    fn alloc_function_body(
        &self,
        span: Span,
        directives: ArenaVec<'a, Directive<'a>>,
        statements: ArenaVec<'a, Statement<'a>>,
    ) -> ArenaBox<'a, FunctionBody<'a>> {
        FunctionBody::boxed(span, directives, statements, self)
    }
    fn alloc_function<T1, T2, T3, T4, T5>(
        &self,
        span: Span,
        r#type: FunctionType,
        id: Option<BindingIdentifier<'a>>,
        generator: bool,
        r#async: bool,
        declare: bool,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
        body: T5,
    ) -> ArenaBox<'a, Function<'a>>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<ArenaBox<'a, FunctionBody<'a>>>>,
    {
        Function::boxed(
            span,
            r#type,
            id,
            generator,
            r#async,
            declare,
            type_parameters,
            this_param,
            params,
            return_type,
            body,
            self,
        )
    }
    fn alloc_spread_element(
        &self,
        span: Span,
        argument: Expression<'a>,
    ) -> ArenaBox<'a, SpreadElement<'a>> {
        SpreadElement::boxed(span, argument, self)
    }
    fn alloc_jsx_text<S1>(
        &self,
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
    ) -> ArenaBox<'a, JSXText<'a>>
    where
        S1: Into<Str<'a>>,
    {
        JSXText::boxed(span, value, raw, self)
    }
    fn alloc_arrow_function_expression<T1, T2, T3, T4>(
        &self,
        span: Span,
        expression: bool,
        r#async: bool,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        body: T4,
    ) -> ArenaBox<'a, ArrowFunctionExpression<'a>>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T4: IntoIn<'a, ArenaBox<'a, FunctionBody<'a>>>,
    {
        ArrowFunctionExpression::boxed(
            span,
            expression,
            r#async,
            type_parameters,
            params,
            return_type,
            body,
            self,
        )
    }
    fn alloc_import_declaration<T1>(
        &self,
        span: Span,
        specifiers: Option<ArenaVec<'a, ImportDeclarationSpecifier<'a>>>,
        source: StringLiteral<'a>,
        phase: Option<ImportPhase>,
        with_clause: T1,
        import_kind: ImportOrExportKind,
    ) -> ArenaBox<'a, ImportDeclaration<'a>>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, WithClause<'a>>>>,
    {
        ImportDeclaration::boxed(span, specifiers, source, phase, with_clause, import_kind, self)
    }
    fn alloc_import_specifier(
        &self,
        span: Span,
        imported: ModuleExportName<'a>,
        local: BindingIdentifier<'a>,
        import_kind: ImportOrExportKind,
    ) -> ArenaBox<'a, ImportSpecifier<'a>> {
        ImportSpecifier::boxed(span, imported, local, import_kind, self)
    }
    fn alloc_export_named_declaration<T1>(
        &self,
        span: Span,
        declaration: Option<Declaration<'a>>,
        specifiers: ArenaVec<'a, ExportSpecifier<'a>>,
        source: Option<StringLiteral<'a>>,
        export_kind: ImportOrExportKind,
        with_clause: T1,
    ) -> ArenaBox<'a, ExportNamedDeclaration<'a>>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, WithClause<'a>>>>,
    {
        ExportNamedDeclaration::boxed(
            span,
            declaration,
            specifiers,
            source,
            export_kind,
            with_clause,
            self,
        )
    }
    fn alloc_export_default_declaration(
        &self,
        span: Span,
        declaration: ExportDefaultDeclarationKind<'a>,
    ) -> ArenaBox<'a, ExportDefaultDeclaration<'a>> {
        ExportDefaultDeclaration::boxed(span, declaration, self)
    }
}
