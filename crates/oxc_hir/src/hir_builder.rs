#![allow(
    clippy::fn_params_excessive_bools,
    clippy::must_use_candidate, // must_use_candidate is too annoying for this file
    clippy::too_many_arguments,
    clippy::unused_self,
)]

use num_bigint::BigUint;
use ordered_float::NotNan;
use oxc_allocator::{Allocator, Box, String, Vec};
use oxc_index::Idx;
use oxc_span::{Atom, Span};
use oxc_syntax::operator::{
    AssignmentOperator, BinaryOperator, LogicalOperator, UnaryOperator, UpdateOperator,
};

#[allow(clippy::wildcard_imports)]
use crate::hir::*;
use crate::HirId;

pub struct HirBuilder<'a> {
    allocator: &'a Allocator,
    hir_id: HirId,
}

impl<'a> HirBuilder<'a> {
    pub fn new(allocator: &'a Allocator) -> Self {
        Self { allocator, hir_id: HirId::new(0) }
    }

    fn next_id(&mut self) -> HirId {
        self.hir_id.increment();
        self.hir_id
    }

    #[inline]
    pub fn alloc<T>(&self, value: T) -> Box<'a, T> {
        Box(self.allocator.alloc(value))
    }

    #[inline]
    pub fn new_vec<T>(&self) -> Vec<'a, T> {
        Vec::new_in(self.allocator)
    }

    #[inline]
    pub fn new_vec_with_capacity<T>(&self, capacity: usize) -> Vec<'a, T> {
        Vec::with_capacity_in(capacity, self.allocator)
    }

    #[inline]
    pub fn new_vec_single<T>(&self, value: T) -> Vec<'a, T> {
        let mut vec = self.new_vec_with_capacity(1);
        vec.push(value);
        vec
    }

    #[inline]
    pub fn new_str(&self, value: &str) -> &'a str {
        String::from_str_in(value, self.allocator).into_bump_str()
    }

    pub fn program(
        &mut self,
        span: Span,
        directives: Vec<'a, Directive>,
        body: Vec<'a, Statement<'a>>,
    ) -> Program<'a> {
        Program { span, directives, body }
    }

    /* ---------- Literals ---------- */

    pub fn string_literal(&mut self, span: Span, value: Atom) -> StringLiteral {
        StringLiteral { span, value }
    }

    pub fn number_literal(
        &mut self,
        span: Span,
        value: NotNan<f64>,
        raw: &'a str,
        base: NumberBase,
    ) -> NumberLiteral<'a> {
        NumberLiteral { span, value, raw, base }
    }

    pub fn boolean_literal(&mut self, span: Span, value: bool) -> BooleanLiteral {
        BooleanLiteral { span, value }
    }

    pub fn null_literal(&mut self, span: Span) -> NullLiteral {
        NullLiteral { span }
    }

    pub fn bigint_literal(&mut self, span: Span, value: BigUint) -> BigintLiteral {
        BigintLiteral { span, value }
    }

    pub fn template_literal(
        &mut self,
        span: Span,
        quasis: Vec<'a, TemplateElement>,
        expressions: Vec<'a, Expression<'a>>,
    ) -> TemplateLiteral<'a> {
        TemplateLiteral { span, quasis, expressions }
    }

    pub fn template_element(
        &mut self,
        span: Span,
        tail: bool,
        value: TemplateElementValue,
    ) -> TemplateElement {
        TemplateElement { span, tail, value }
    }

    pub fn template_element_value(
        &mut self,
        raw: Atom,
        cooked: Option<Atom>,
    ) -> TemplateElementValue {
        TemplateElementValue { raw, cooked }
    }

    pub fn reg_exp_literal(
        &mut self,
        span: Span,
        pattern: Atom,
        flags: RegExpFlags,
    ) -> RegExpLiteral {
        RegExpLiteral { span, value: EmptyObject, regex: RegExp { pattern, flags } }
    }

    pub fn literal_string_expression(&mut self, literal: StringLiteral) -> Expression<'a> {
        Expression::StringLiteral(self.alloc(literal))
    }

    pub fn literal_boolean_expression(&mut self, literal: BooleanLiteral) -> Expression<'a> {
        Expression::BooleanLiteral(self.alloc(literal))
    }

    pub fn literal_null_expression(&mut self, literal: NullLiteral) -> Expression<'a> {
        Expression::NullLiteral(self.alloc(literal))
    }

    pub fn literal_regexp_expression(&mut self, literal: RegExpLiteral) -> Expression<'a> {
        Expression::RegExpLiteral(self.alloc(literal))
    }

    pub fn literal_number_expression(&mut self, literal: NumberLiteral<'a>) -> Expression<'a> {
        Expression::NumberLiteral(self.alloc(literal))
    }

    pub fn literal_bigint_expression(&mut self, literal: BigintLiteral) -> Expression<'a> {
        Expression::BigintLiteral(self.alloc(literal))
    }

    pub fn literal_template_expression(&mut self, literal: TemplateLiteral<'a>) -> Expression<'a> {
        Expression::TemplateLiteral(self.alloc(literal))
    }

    pub fn identifier_reference_expression(
        &mut self,
        ident: IdentifierReference,
    ) -> Expression<'a> {
        Expression::Identifier(self.alloc(ident))
    }

    /* ---------- Identifiers ---------- */

    pub fn identifier_name(&mut self, span: Span, name: Atom) -> IdentifierName {
        IdentifierName { span, name }
    }

    pub fn identifier_reference(&mut self, span: Span, name: Atom) -> IdentifierReference {
        IdentifierReference { span, name }
    }

    pub fn binding_identifier(&mut self, span: Span, name: Atom) -> BindingIdentifier {
        BindingIdentifier { span, name }
    }

    pub fn label_identifier(&mut self, span: Span, name: Atom) -> LabelIdentifier {
        LabelIdentifier { span, name }
    }

    pub fn private_identifier(&mut self, span: Span, name: Atom) -> PrivateIdentifier {
        PrivateIdentifier { span, name }
    }

    /* ---------- Statements ---------- */

    pub fn directive(
        &mut self,
        span: Span,
        expression: StringLiteral,
        directive: &'a str,
    ) -> Directive<'a> {
        Directive { hir_id: self.next_id(), span, expression, directive }
    }

    pub fn block(
        &mut self,
        span: Span,
        body: Vec<'a, Statement<'a>>,
    ) -> Box<'a, BlockStatement<'a>> {
        self.alloc(BlockStatement { span, body })
    }

    pub fn block_statement(&mut self, span: Span, body: Vec<'a, Statement<'a>>) -> Statement<'a> {
        Statement::BlockStatement(self.block(span, body))
    }

    pub fn break_statement(&mut self, span: Span, label: Option<LabelIdentifier>) -> Statement<'a> {
        Statement::BreakStatement(self.alloc(BreakStatement { span, label }))
    }

    pub fn continue_statement(
        &mut self,
        span: Span,
        label: Option<LabelIdentifier>,
    ) -> Statement<'a> {
        Statement::ContinueStatement(self.alloc(ContinueStatement { span, label }))
    }

    pub fn debugger_statement(&mut self, span: Span) -> Statement<'a> {
        Statement::DebuggerStatement(self.alloc(DebuggerStatement { span }))
    }

    pub fn do_while_statement(
        &mut self,
        span: Span,
        body: Statement<'a>,
        test: Expression<'a>,
    ) -> Statement<'a> {
        Statement::DoWhileStatement(self.alloc(DoWhileStatement { span, body, test }))
    }

    pub fn empty_statement(&mut self, span: Span) -> Statement<'a> {
        Statement::EmptyStatement(self.alloc(EmptyStatement { span }))
    }

    pub fn expression_statement(
        &mut self,
        span: Span,
        expression: Expression<'a>,
    ) -> Statement<'a> {
        Statement::ExpressionStatement(self.alloc(ExpressionStatement { span, expression }))
    }

    pub fn for_in_statement(
        &mut self,
        span: Span,
        left: ForStatementLeft<'a>,
        right: Expression<'a>,
        body: Statement<'a>,
    ) -> Statement<'a> {
        Statement::ForInStatement(self.alloc(ForInStatement { span, left, right, body }))
    }

    pub fn for_of_statement(
        &mut self,
        span: Span,
        r#await: bool,
        left: ForStatementLeft<'a>,
        right: Expression<'a>,
        body: Statement<'a>,
    ) -> Statement<'a> {
        Statement::ForOfStatement(self.alloc(ForOfStatement { span, r#await, left, right, body }))
    }

    pub fn for_statement(
        &mut self,
        span: Span,
        init: Option<ForStatementInit<'a>>,
        test: Option<Expression<'a>>,
        update: Option<Expression<'a>>,
        body: Statement<'a>,
    ) -> Statement<'a> {
        Statement::ForStatement(self.alloc(ForStatement { span, init, test, update, body }))
    }

    pub fn if_statement(
        &mut self,
        span: Span,
        test: Expression<'a>,
        consequent: Statement<'a>,
        alternate: Option<Statement<'a>>,
    ) -> Statement<'a> {
        Statement::IfStatement(self.alloc(IfStatement { span, test, consequent, alternate }))
    }

    pub fn labeled_statement(
        &mut self,
        span: Span,
        label: LabelIdentifier,
        body: Statement<'a>,
    ) -> Statement<'a> {
        Statement::LabeledStatement(self.alloc(LabeledStatement { span, label, body }))
    }

    pub fn return_statement(
        &mut self,
        span: Span,
        argument: Option<Expression<'a>>,
    ) -> Statement<'a> {
        Statement::ReturnStatement(self.alloc(ReturnStatement { span, argument }))
    }

    pub fn switch_statement(
        &mut self,
        span: Span,
        discriminant: Expression<'a>,
        cases: Vec<'a, SwitchCase<'a>>,
    ) -> Statement<'a> {
        Statement::SwitchStatement(self.alloc(SwitchStatement { span, discriminant, cases }))
    }

    pub fn switch_case(
        &mut self,
        span: Span,
        test: Option<Expression<'a>>,
        consequent: Vec<'a, Statement<'a>>,
    ) -> SwitchCase<'a> {
        SwitchCase { span, test, consequent }
    }

    pub fn throw_statement(&mut self, span: Span, argument: Expression<'a>) -> Statement<'a> {
        Statement::ThrowStatement(self.alloc(ThrowStatement { span, argument }))
    }

    pub fn try_statement(
        &mut self,
        span: Span,
        block: Box<'a, BlockStatement<'a>>,
        handler: Option<Box<'a, CatchClause<'a>>>,
        finalizer: Option<Box<'a, BlockStatement<'a>>>,
    ) -> Statement<'a> {
        Statement::TryStatement(self.alloc(TryStatement { span, block, handler, finalizer }))
    }

    pub fn catch_clause(
        &mut self,
        span: Span,
        param: Option<BindingPattern<'a>>,
        body: Box<'a, BlockStatement<'a>>,
    ) -> Box<'a, CatchClause<'a>> {
        self.alloc(CatchClause { span, param, body })
    }

    pub fn while_statement(
        &mut self,
        span: Span,
        test: Expression<'a>,
        body: Statement<'a>,
    ) -> Statement<'a> {
        Statement::WhileStatement(self.alloc(WhileStatement { span, test, body }))
    }

    pub fn with_statement(
        &mut self,
        span: Span,
        object: Expression<'a>,
        body: Statement<'a>,
    ) -> Statement<'a> {
        Statement::WithStatement(self.alloc(WithStatement { span, object, body }))
    }

    /* ---------- Expressions ---------- */

    pub fn super_expression(&mut self, span: Span) -> Expression<'a> {
        Expression::Super(self.alloc(Super { span }))
    }

    pub fn meta_property(
        &mut self,
        span: Span,
        meta: IdentifierName,
        property: IdentifierName,
    ) -> Expression<'a> {
        Expression::MetaProperty(self.alloc(MetaProperty { span, meta, property }))
    }

    pub fn array_expression(
        &mut self,
        span: Span,
        elements: Vec<'a, ArrayExpressionElement<'a>>,
        trailing_comma: Option<Span>,
    ) -> Expression<'a> {
        Expression::ArrayExpression(self.alloc(ArrayExpression { span, elements, trailing_comma }))
    }

    pub fn arrow_expression(
        &mut self,
        span: Span,
        expression: bool,
        generator: bool,
        r#async: bool,
        params: Box<'a, FormalParameters<'a>>,
        body: Box<'a, FunctionBody<'a>>,
    ) -> Expression<'a> {
        Expression::ArrowFunctionExpression(self.alloc(ArrowExpression {
            span,
            expression,
            generator,
            r#async,
            params,
            body,
        }))
    }

    pub fn assignment_expression(
        &mut self,
        span: Span,
        operator: AssignmentOperator,
        left: AssignmentTarget<'a>,
        right: Expression<'a>,
    ) -> Expression<'a> {
        Expression::AssignmentExpression(self.alloc(AssignmentExpression {
            span,
            operator,
            left,
            right,
        }))
    }

    pub fn await_expression(&mut self, span: Span, argument: Expression<'a>) -> Expression<'a> {
        Expression::AwaitExpression(self.alloc(AwaitExpression { span, argument }))
    }

    pub fn binary_expression(
        &mut self,
        span: Span,
        left: Expression<'a>,
        operator: BinaryOperator,
        right: Expression<'a>,
    ) -> Expression<'a> {
        Expression::BinaryExpression(self.alloc(BinaryExpression { span, left, operator, right }))
    }

    pub fn call_expression(
        &mut self,
        span: Span,
        callee: Expression<'a>,
        arguments: Vec<'a, Argument<'a>>,
        optional: bool, // for optional chaining
    ) -> Expression<'a> {
        Expression::CallExpression(self.alloc(CallExpression { span, callee, arguments, optional }))
    }

    pub fn chain_expression(&mut self, span: Span, expression: ChainElement<'a>) -> Expression<'a> {
        Expression::ChainExpression(self.alloc(ChainExpression { span, expression }))
    }

    pub fn class_expression(&mut self, class: Box<'a, Class<'a>>) -> Expression<'a> {
        Expression::ClassExpression(class)
    }

    pub fn conditional_expression(
        &mut self,
        span: Span,
        test: Expression<'a>,
        consequent: Expression<'a>,
        alternate: Expression<'a>,
    ) -> Expression<'a> {
        Expression::ConditionalExpression(self.alloc(ConditionalExpression {
            span,
            test,
            consequent,
            alternate,
        }))
    }

    pub fn function_expression(&mut self, function: Box<'a, Function<'a>>) -> Expression<'a> {
        Expression::FunctionExpression(function)
    }

    pub fn import_expression(
        &mut self,
        span: Span,
        source: Expression<'a>,
        arguments: Vec<'a, Expression<'a>>,
    ) -> Expression<'a> {
        Expression::ImportExpression(self.alloc(ImportExpression { span, source, arguments }))
    }

    pub fn logical_expression(
        &mut self,
        span: Span,
        left: Expression<'a>,
        operator: LogicalOperator,
        right: Expression<'a>,
    ) -> Expression<'a> {
        Expression::LogicalExpression(self.alloc(LogicalExpression { span, left, operator, right }))
    }

    pub fn computed_member_expression(
        &mut self,
        span: Span,
        object: Expression<'a>,
        expression: Expression<'a>,
        optional: bool, // for optional chaining
    ) -> MemberExpression<'a> {
        MemberExpression::ComputedMemberExpression(ComputedMemberExpression {
            span,
            object,
            expression,
            optional,
        })
    }

    pub fn static_member_expression(
        &mut self,
        span: Span,
        object: Expression<'a>,
        property: IdentifierName,
        optional: bool, // for optional chaining
    ) -> MemberExpression<'a> {
        MemberExpression::StaticMemberExpression(StaticMemberExpression {
            span,
            object,
            property,
            optional,
        })
    }

    pub fn private_field_expression(
        &mut self,
        span: Span,
        object: Expression<'a>,
        field: PrivateIdentifier,
        optional: bool,
    ) -> MemberExpression<'a> {
        MemberExpression::PrivateFieldExpression(PrivateFieldExpression {
            span,
            object,
            field,
            optional,
        })
    }

    pub fn member_expression(&mut self, member_expr: MemberExpression<'a>) -> Expression<'a> {
        Expression::MemberExpression(self.alloc(member_expr))
    }

    pub fn new_expression(
        &mut self,
        span: Span,
        callee: Expression<'a>,
        arguments: Vec<'a, Argument<'a>>,
    ) -> Expression<'a> {
        Expression::NewExpression(self.alloc(NewExpression { span, callee, arguments }))
    }

    pub fn object_expression(
        &mut self,
        span: Span,
        properties: Vec<'a, ObjectProperty<'a>>,
        trailing_comma: Option<Span>,
    ) -> Expression<'a> {
        Expression::ObjectExpression(self.alloc(ObjectExpression {
            span,
            properties,
            trailing_comma,
        }))
    }

    pub fn object_property_property(&mut self, property: Property<'a>) -> ObjectProperty<'a> {
        ObjectProperty::Property(self.alloc(property))
    }

    pub fn private_in_expression(
        &mut self,
        span: Span,
        left: PrivateIdentifier,
        right: Expression<'a>,
    ) -> Expression<'a> {
        Expression::PrivateInExpression(self.alloc(PrivateInExpression { span, left, right }))
    }

    pub fn sequence_expression(
        &mut self,
        span: Span,
        expressions: Vec<'a, Expression<'a>>,
    ) -> Expression<'a> {
        Expression::SequenceExpression(self.alloc(SequenceExpression { span, expressions }))
    }

    pub fn tagged_template_expression(
        &mut self,
        span: Span,
        tag: Expression<'a>,
        quasi: TemplateLiteral<'a>,
    ) -> Expression<'a> {
        Expression::TaggedTemplateExpression(self.alloc(TaggedTemplateExpression {
            span,
            tag,
            quasi,
        }))
    }

    pub fn template_literal_expression(
        &mut self,
        template_literal: TemplateLiteral<'a>,
    ) -> Expression<'a> {
        Expression::TemplateLiteral(self.alloc(template_literal))
    }

    pub fn this_expression(&mut self, span: Span) -> Expression<'a> {
        Expression::ThisExpression(self.alloc(ThisExpression { span }))
    }

    pub fn unary_expression(
        &mut self,
        span: Span,
        operator: UnaryOperator,
        prefix: bool,
        argument: Expression<'a>,
    ) -> Expression<'a> {
        Expression::UnaryExpression(self.alloc(UnaryExpression {
            span,
            operator,
            prefix,
            argument,
        }))
    }

    pub fn update_expression(
        &mut self,
        span: Span,
        operator: UpdateOperator,
        prefix: bool,
        argument: SimpleAssignmentTarget<'a>,
    ) -> Expression<'a> {
        Expression::UpdateExpression(self.alloc(UpdateExpression {
            span,
            operator,
            prefix,
            argument,
        }))
    }

    pub fn yield_expression(
        &mut self,
        span: Span,
        delegate: bool,
        argument: Option<Expression<'a>>,
    ) -> Expression<'a> {
        Expression::YieldExpression(self.alloc(YieldExpression { span, delegate, argument }))
    }

    pub fn assignment_target_identifier(
        &mut self,
        ident: IdentifierReference,
    ) -> SimpleAssignmentTarget<'a> {
        SimpleAssignmentTarget::AssignmentTargetIdentifier(self.alloc(ident))
    }

    pub fn member_assignment_target(
        &mut self,
        member_expr: MemberExpression<'a>,
    ) -> SimpleAssignmentTarget<'a> {
        SimpleAssignmentTarget::MemberAssignmentTarget(self.alloc(member_expr))
    }

    pub fn array_assignment_target(
        &mut self,
        span: Span,
        elements: Vec<'a, Option<AssignmentTargetMaybeDefault<'a>>>,
        rest: Option<AssignmentTarget<'a>>,
        trailing_comma: Option<Span>,
    ) -> Box<'a, ArrayAssignmentTarget<'a>> {
        self.alloc(ArrayAssignmentTarget { span, elements, rest, trailing_comma })
    }

    pub fn object_assignment_target(
        &mut self,
        span: Span,
        properties: Vec<'a, AssignmentTargetProperty<'a>>,
        rest: Option<AssignmentTarget<'a>>,
    ) -> Box<'a, ObjectAssignmentTarget<'a>> {
        self.alloc(ObjectAssignmentTarget { span, properties, rest })
    }

    pub fn assignment_target_with_default(
        &mut self,
        span: Span,
        binding: AssignmentTarget<'a>,
        init: Expression<'a>,
    ) -> Box<'a, AssignmentTargetWithDefault<'a>> {
        self.alloc(AssignmentTargetWithDefault { span, binding, init })
    }

    pub fn assignment_target_property_identifier(
        &mut self,
        span: Span,
        binding: IdentifierReference,
        init: Option<Expression<'a>>,
    ) -> Box<'a, AssignmentTargetPropertyIdentifier<'a>> {
        self.alloc(AssignmentTargetPropertyIdentifier { span, binding, init })
    }

    pub fn assignment_target_property_property(
        &mut self,
        span: Span,
        name: PropertyKey<'a>,
        binding: AssignmentTargetMaybeDefault<'a>,
    ) -> Box<'a, AssignmentTargetPropertyProperty<'a>> {
        self.alloc(AssignmentTargetPropertyProperty { span, name, binding })
    }

    /* ---------- Functions ---------- */
    pub fn function_declaration(&mut self, func: Box<'a, Function<'a>>) -> Statement<'a> {
        Statement::Declaration(Declaration::FunctionDeclaration(func))
    }

    pub fn formal_parameters(
        &mut self,
        span: Span,
        kind: FormalParameterKind,
        items: Vec<'a, FormalParameter<'a>>,
    ) -> Box<'a, FormalParameters<'a>> {
        self.alloc(FormalParameters { span, kind, items })
    }

    pub fn formal_parameter(
        &mut self,
        span: Span,
        pattern: BindingPattern<'a>,
        decorators: Vec<'a, Decorator<'a>>,
    ) -> FormalParameter<'a> {
        FormalParameter { span, pattern, decorators }
    }

    #[allow(clippy::fn_params_excessive_bools)]
    pub fn function(
        &mut self,
        r#type: FunctionType,
        span: Span,
        id: Option<BindingIdentifier>,
        expression: bool,
        generator: bool,
        r#async: bool,
        params: Box<'a, FormalParameters<'a>>,
        body: Option<Box<'a, FunctionBody<'a>>>,
    ) -> Box<'a, Function<'a>> {
        self.alloc(Function { r#type, span, id, expression, generator, r#async, params, body })
    }

    pub fn function_body(
        &mut self,
        span: Span,
        directives: Vec<'a, Directive>,
        statements: Vec<'a, Statement<'a>>,
    ) -> Box<'a, FunctionBody<'a>> {
        self.alloc(FunctionBody { span, directives, statements })
    }

    /* ---------- Class ---------- */

    pub fn class(
        &mut self,
        r#type: ClassType,
        span: Span,
        id: Option<BindingIdentifier>,
        super_class: Option<Expression<'a>>,
        body: Box<'a, ClassBody<'a>>,
        decorators: Vec<'a, Decorator<'a>>,
    ) -> Box<'a, Class<'a>> {
        self.alloc(Class { r#type, span, id, super_class, body, decorators })
    }

    pub fn class_body(
        &mut self,
        span: Span,
        body: Vec<'a, ClassElement<'a>>,
    ) -> Box<'a, ClassBody<'a>> {
        self.alloc(ClassBody { span, body })
    }

    pub fn static_block(
        &mut self,
        span: Span,
        body: Vec<'a, Statement<'a>>,
    ) -> Box<'a, StaticBlock<'a>> {
        self.alloc(StaticBlock { span, body })
    }

    pub fn method_definition(
        &mut self,
        span: Span,
        key: PropertyKey<'a>,
        value: Box<'a, Function<'a>>, // FunctionExpression
        kind: MethodDefinitionKind,
        computed: bool,
        r#static: bool,
        r#override: bool,
        optional: bool,
        decorators: Vec<'a, Decorator<'a>>,
    ) -> Box<'a, MethodDefinition<'a>> {
        self.alloc(MethodDefinition {
            span,
            key,
            value,
            kind,
            computed,
            r#static,
            r#override,
            optional,
            decorators,
        })
    }

    pub fn property_definition(
        &mut self,
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
        decorators: Vec<'a, Decorator<'a>>,
    ) -> Box<'a, PropertyDefinition<'a>> {
        self.alloc(PropertyDefinition {
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
            decorators,
        })
    }

    pub fn accessor_property(
        &mut self,
        span: Span,
        key: PropertyKey<'a>,
        value: Option<Expression<'a>>,
        computed: bool,
        r#static: bool,
    ) -> Box<'a, AccessorProperty<'a>> {
        self.alloc(AccessorProperty { span, key, value, computed, r#static })
    }

    /* ---------- Declarations ---------- */

    pub fn variable_declaration(
        &mut self,
        span: Span,
        kind: VariableDeclarationKind,
        declarations: Vec<'a, VariableDeclarator<'a>>,
    ) -> Box<'a, VariableDeclaration<'a>> {
        self.alloc(VariableDeclaration { span, kind, declarations })
    }

    pub fn variable_declarator(
        &mut self,
        span: Span,
        kind: VariableDeclarationKind,
        id: BindingPattern<'a>,
        init: Option<Expression<'a>>,
        definite: bool,
    ) -> VariableDeclarator<'a> {
        VariableDeclarator { span, kind, id, init, definite }
    }

    /* ---------- Patterns ---------- */

    pub fn binding_identifier_pattern(
        &mut self,
        identifier: BindingIdentifier,
    ) -> BindingPattern<'a> {
        BindingPattern::BindingIdentifier(self.alloc(identifier))
    }

    pub fn object_pattern(
        &mut self,
        span: Span,
        properties: Vec<'a, ObjectPatternProperty<'a>>,
    ) -> BindingPattern<'a> {
        BindingPattern::ObjectPattern(self.alloc(ObjectPattern { span, properties }))
    }

    pub fn spread_element(
        &mut self,
        span: Span,
        argument: Expression<'a>,
    ) -> Box<'a, SpreadElement<'a>> {
        self.alloc(SpreadElement { span, argument })
    }

    pub fn property(
        &mut self,
        span: Span,
        kind: PropertyKind,
        key: PropertyKey<'a>,
        value: PropertyValue<'a>,
        method: bool,
        shorthand: bool,
        computed: bool,
    ) -> Box<'a, Property<'a>> {
        self.alloc(Property { span, kind, key, value, method, shorthand, computed })
    }

    pub fn property_key_identifier(&mut self, ident: IdentifierName) -> PropertyKey<'a> {
        PropertyKey::Identifier(self.alloc(ident))
    }

    pub fn property_key_private_identifier(&mut self, ident: PrivateIdentifier) -> PropertyKey<'a> {
        PropertyKey::PrivateIdentifier(self.alloc(ident))
    }

    pub fn array_pattern(
        &mut self,
        span: Span,
        elements: Vec<'a, Option<BindingPattern<'a>>>,
    ) -> BindingPattern<'a> {
        BindingPattern::ArrayPattern(self.alloc(ArrayPattern { span, elements }))
    }

    pub fn assignment_pattern(
        &mut self,
        span: Span,
        left: BindingPattern<'a>,
        right: Expression<'a>,
    ) -> BindingPattern<'a> {
        let pattern = self.alloc(AssignmentPattern { span, left, right });
        BindingPattern::AssignmentPattern(pattern)
    }

    pub fn rest_element(
        &mut self,
        span: Span,
        argument: BindingPattern<'a>,
    ) -> Box<'a, RestElement<'a>> {
        self.alloc(RestElement { span, argument })
    }

    pub fn rest_element_pattern(&mut self, elem: Box<'a, RestElement<'a>>) -> BindingPattern<'a> {
        BindingPattern::RestElement(elem)
    }

    /* ---------- Modules ---------- */

    pub fn module_declaration(&mut self, decl: ModuleDeclaration<'a>) -> Statement<'a> {
        Statement::ModuleDeclaration(self.alloc(decl))
    }

    pub fn import_declaration(
        &mut self,
        span: Span,
        specifiers: Vec<'a, ImportDeclarationSpecifier>,
        source: StringLiteral,
        assertions: Option<Vec<'a, ImportAttribute>>,
        import_kind: ImportOrExportKind,
    ) -> Box<'a, ImportDeclaration<'a>> {
        self.alloc(ImportDeclaration { span, specifiers, source, assertions, import_kind })
    }

    pub fn import_attribute(
        &mut self,
        span: Span,
        key: ImportAttributeKey,
        value: StringLiteral,
    ) -> ImportAttribute {
        ImportAttribute { span, key, value }
    }

    pub fn import_specifier(
        &mut self,
        span: Span,
        imported: ModuleExportName,
        local: BindingIdentifier,
    ) -> ImportSpecifier {
        ImportSpecifier { span, imported, local }
    }

    pub fn import_default_specifier(
        &mut self,
        span: Span,
        local: BindingIdentifier,
    ) -> ImportDefaultSpecifier {
        ImportDefaultSpecifier { span, local }
    }

    pub fn import_namespace_specifier(
        &mut self,
        span: Span,
        local: BindingIdentifier,
    ) -> ImportNamespaceSpecifier {
        ImportNamespaceSpecifier { span, local }
    }

    pub fn export_all_declaration(
        &mut self,
        span: Span,
        exported: Option<ModuleExportName>,
        source: StringLiteral,
        assertions: Option<Vec<'a, ImportAttribute>>, // Some(vec![]) for empty assertion
        export_kind: ImportOrExportKind,
    ) -> Box<'a, ExportAllDeclaration<'a>> {
        self.alloc(ExportAllDeclaration { span, exported, source, assertions, export_kind })
    }

    pub fn export_default_declaration(
        &mut self,
        span: Span,
        declaration: ExportDefaultDeclarationKind<'a>,
        exported: ModuleExportName,
    ) -> Box<'a, ExportDefaultDeclaration<'a>> {
        self.alloc(ExportDefaultDeclaration { span, declaration, exported })
    }

    pub fn export_named_declaration(
        &mut self,
        span: Span,
        declaration: Option<Declaration<'a>>,
        specifiers: Vec<'a, ExportSpecifier>,
        source: Option<StringLiteral>,
        export_kind: ImportOrExportKind, // `export type { foo }`
    ) -> Box<'a, ExportNamedDeclaration<'a>> {
        self.alloc(ExportNamedDeclaration { span, declaration, specifiers, source, export_kind })
    }

    pub fn export_specifier(
        &mut self,
        span: Span,
        local: ModuleExportName,
        exported: ModuleExportName,
    ) -> ExportSpecifier {
        ExportSpecifier { span, local, exported }
    }

    /* ---------- JSX ----------------- */
    pub fn jsx_element(
        &mut self,
        span: Span,
        opening_element: Box<'a, JSXOpeningElement<'a>>,
        closing_element: Option<Box<'a, JSXClosingElement<'a>>>,
        children: Vec<'a, JSXChild<'a>>,
    ) -> Box<'a, JSXElement<'a>> {
        self.alloc(JSXElement { span, opening_element, closing_element, children })
    }

    pub fn jsx_opening_element(
        &mut self,
        span: Span,
        self_closing: bool,
        name: JSXElementName<'a>,
        attributes: Vec<'a, JSXAttributeItem<'a>>,
    ) -> Box<'a, JSXOpeningElement<'a>> {
        self.alloc(JSXOpeningElement { span, self_closing, name, attributes })
    }

    pub fn jsx_closing_element(
        &mut self,
        span: Span,
        name: JSXElementName<'a>,
    ) -> Box<'a, JSXClosingElement<'a>> {
        self.alloc(JSXClosingElement { span, name })
    }

    pub fn jsx_fragment(
        &mut self,
        span: Span,
        opening_fragment: JSXOpeningFragment,
        closing_fragment: JSXClosingFragment,
        children: Vec<'a, JSXChild<'a>>,
    ) -> Box<'a, JSXFragment<'a>> {
        self.alloc(JSXFragment { span, opening_fragment, closing_fragment, children })
    }

    pub fn jsx_opening_fragment(&mut self, span: Span) -> JSXOpeningFragment {
        JSXOpeningFragment { span }
    }

    pub fn jsx_closing_fragment(&mut self, span: Span) -> JSXClosingFragment {
        JSXClosingFragment { span }
    }

    pub fn jsx_namespaced_name(
        &mut self,
        span: Span,
        namespace: JSXIdentifier,
        property: JSXIdentifier,
    ) -> Box<'a, JSXNamespacedName> {
        self.alloc(JSXNamespacedName { span, namespace, property })
    }

    pub fn jsx_member_expression(
        &mut self,
        span: Span,
        object: JSXMemberExpressionObject<'a>,
        property: JSXIdentifier,
    ) -> Box<'a, JSXMemberExpression<'a>> {
        self.alloc(JSXMemberExpression { span, object, property })
    }

    pub fn jsx_expression_container(
        &mut self,
        span: Span,
        expression: JSXExpression<'a>,
    ) -> JSXExpressionContainer<'a> {
        JSXExpressionContainer { span, expression }
    }

    pub fn jsx_spread_child(
        &mut self,
        span: Span,
        expression: Expression<'a>,
    ) -> JSXSpreadChild<'a> {
        JSXSpreadChild { span, expression }
    }

    pub fn jsx_empty_expression(&mut self, span: Span) -> JSXEmptyExpression {
        JSXEmptyExpression { span }
    }

    pub fn jsx_attribute(
        &mut self,
        span: Span,
        name: JSXAttributeName<'a>,
        value: Option<JSXAttributeValue<'a>>,
    ) -> Box<'a, JSXAttribute<'a>> {
        self.alloc(JSXAttribute { span, name, value })
    }

    pub fn jsx_spread_attribute(
        &mut self,
        span: Span,
        argument: Expression<'a>,
    ) -> Box<'a, JSXSpreadAttribute<'a>> {
        self.alloc(JSXSpreadAttribute { span, argument })
    }

    pub fn jsx_identifier(&mut self, span: Span, name: Atom) -> JSXIdentifier {
        JSXIdentifier { span, name }
    }

    pub fn jsx_text(&mut self, span: Span, value: Atom) -> JSXText {
        JSXText { span, value }
    }

    pub fn ts_enum_declaration(
        &mut self,
        span: Span,
        id: BindingIdentifier,
        members: Vec<'a, TSEnumMember<'a>>,
    ) -> Declaration<'a> {
        Declaration::TSEnumDeclaration(self.alloc(TSEnumDeclaration { span, id, members }))
    }

    pub fn decorator(&mut self, span: Span, expression: Expression<'a>) -> Decorator<'a> {
        Decorator { span, expression }
    }
}
