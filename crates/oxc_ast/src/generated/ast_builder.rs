use oxc_allocator::{Allocator, Box, Vec};
use oxc_span::{Atom, SourceType, Span};
use oxc_syntax::{
    number::{BigintBase, NumberBase},
    operator::{
        AssignmentOperator, BinaryOperator, LogicalOperator, UnaryOperator, UpdateOperator,
    },
};

#[allow(clippy::wildcard_imports)]
use crate::ast::*;

/// AST builder for creating AST nodes
#[derive(Clone, Copy)]
pub struct AstBuilder<'a> {
    pub allocator: &'a Allocator,
}

impl<'a> AstBuilder<'a> {
    pub fn boolean_literal(self, span: Span, value: bool) -> Box<'a, BooleanLiteral> {
        self.alloc(BooleanLiteral { span, value })
    }
    pub fn null_literal(self, span: Span) -> Box<'a, NullLiteral> {
        self.alloc(NullLiteral { span })
    }
    pub fn numeric_literal(
        self,
        span: Span,
        value: f64,
        raw: &'a str,
        base: NumberBase,
    ) -> Box<'a, NumericLiteral<'a>> {
        self.alloc(NumericLiteral { span, value, raw, base })
    }
    pub fn big_int_literal(
        self,
        span: Span,
        raw: Atom<'a>,
        base: BigintBase,
    ) -> Box<'a, BigIntLiteral<'a>> {
        self.alloc(BigIntLiteral { span, raw, base })
    }
    pub fn reg_exp_literal(
        self,
        span: Span,
        value: EmptyObject,
        regex: RegExp<'a>,
    ) -> Box<'a, RegExpLiteral<'a>> {
        self.alloc(RegExpLiteral { span, value, regex })
    }
    pub fn reg_exp(self, pattern: Atom<'a>, flags: RegExpFlags) -> Box<'a, RegExp<'a>> {
        self.alloc(RegExp { pattern, flags })
    }
    pub fn empty_object(self) -> Box<'a, EmptyObject> {
        self.alloc(EmptyObject {})
    }
    pub fn string_literal(self, span: Span, value: Atom<'a>) -> Box<'a, StringLiteral<'a>> {
        self.alloc(StringLiteral { span, value })
    }
    pub fn program(
        self,
        span: Span,
        source_type: SourceType,
        directives: Vec<'a, Directive<'a>>,
        hashbang: Option<Hashbang<'a>>,
        body: Vec<'a, Statement<'a>>,
    ) -> Box<'a, Program<'a>> {
        self.alloc(Program {
            span,
            source_type,
            directives,
            hashbang,
            body,
            scope_id: Default::default(),
        })
    }
    pub fn identifier_name(self, span: Span, name: Atom<'a>) -> Box<'a, IdentifierName<'a>> {
        self.alloc(IdentifierName { span, name })
    }
    pub fn identifier_reference(
        self,
        span: Span,
        name: Atom<'a>,
    ) -> Box<'a, IdentifierReference<'a>> {
        self.alloc(IdentifierReference {
            span,
            name,
            reference_id: Default::default(),
            reference_flag: Default::default(),
        })
    }
    pub fn binding_identifier(self, span: Span, name: Atom<'a>) -> Box<'a, BindingIdentifier<'a>> {
        self.alloc(BindingIdentifier { span, name, symbol_id: Default::default() })
    }
    pub fn label_identifier(self, span: Span, name: Atom<'a>) -> Box<'a, LabelIdentifier<'a>> {
        self.alloc(LabelIdentifier { span, name })
    }
    pub fn this_expression(self, span: Span) -> Box<'a, ThisExpression> {
        self.alloc(ThisExpression { span })
    }
    pub fn array_expression(
        self,
        span: Span,
        elements: Vec<'a, ArrayExpressionElement<'a>>,
        trailing_comma: Option<Span>,
    ) -> Box<'a, ArrayExpression<'a>> {
        self.alloc(ArrayExpression { span, elements, trailing_comma })
    }
    pub fn elision(self, span: Span) -> Box<'a, Elision> {
        self.alloc(Elision { span })
    }
    pub fn object_expression(
        self,
        span: Span,
        properties: Vec<'a, ObjectPropertyKind<'a>>,
        trailing_comma: Option<Span>,
    ) -> Box<'a, ObjectExpression<'a>> {
        self.alloc(ObjectExpression { span, properties, trailing_comma })
    }
    pub fn object_property(
        self,
        span: Span,
        kind: PropertyKind,
        key: PropertyKey<'a>,
        value: Expression<'a>,
        init: Option<Expression<'a>>,
        method: bool,
        shorthand: bool,
        computed: bool,
    ) -> Box<'a, ObjectProperty<'a>> {
        self.alloc(ObjectProperty { span, kind, key, value, init, method, shorthand, computed })
    }
    pub fn template_literal(
        self,
        span: Span,
        quasis: Vec<'a, TemplateElement<'a>>,
        expressions: Vec<'a, Expression<'a>>,
    ) -> Box<'a, TemplateLiteral<'a>> {
        self.alloc(TemplateLiteral { span, quasis, expressions })
    }
    pub fn tagged_template_expression(
        self,
        span: Span,
        tag: Expression<'a>,
        quasi: TemplateLiteral<'a>,
        type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
    ) -> Box<'a, TaggedTemplateExpression<'a>> {
        self.alloc(TaggedTemplateExpression { span, tag, quasi, type_parameters })
    }
    pub fn template_element(
        self,
        span: Span,
        tail: bool,
        value: TemplateElementValue<'a>,
    ) -> Box<'a, TemplateElement<'a>> {
        self.alloc(TemplateElement { span, tail, value })
    }
    pub fn template_element_value(
        self,
        raw: Atom<'a>,
        cooked: Option<Atom<'a>>,
    ) -> Box<'a, TemplateElementValue<'a>> {
        self.alloc(TemplateElementValue { raw, cooked })
    }
    pub fn computed_member_expression(
        self,
        span: Span,
        object: Expression<'a>,
        expression: Expression<'a>,
        optional: bool,
    ) -> Box<'a, ComputedMemberExpression<'a>> {
        self.alloc(ComputedMemberExpression { span, object, expression, optional })
    }
    pub fn static_member_expression(
        self,
        span: Span,
        object: Expression<'a>,
        property: IdentifierName<'a>,
        optional: bool,
    ) -> Box<'a, StaticMemberExpression<'a>> {
        self.alloc(StaticMemberExpression { span, object, property, optional })
    }
    pub fn private_field_expression(
        self,
        span: Span,
        object: Expression<'a>,
        field: PrivateIdentifier<'a>,
        optional: bool,
    ) -> Box<'a, PrivateFieldExpression<'a>> {
        self.alloc(PrivateFieldExpression { span, object, field, optional })
    }
    pub fn call_expression(
        self,
        span: Span,
        arguments: Vec<'a, Argument<'a>>,
        callee: Expression<'a>,
        type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
        optional: bool,
    ) -> Box<'a, CallExpression<'a>> {
        self.alloc(CallExpression { span, arguments, callee, type_parameters, optional })
    }
    pub fn new_expression(
        self,
        span: Span,
        callee: Expression<'a>,
        arguments: Vec<'a, Argument<'a>>,
        type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
    ) -> Box<'a, NewExpression<'a>> {
        self.alloc(NewExpression { span, callee, arguments, type_parameters })
    }
    pub fn meta_property(
        self,
        span: Span,
        meta: IdentifierName<'a>,
        property: IdentifierName<'a>,
    ) -> Box<'a, MetaProperty<'a>> {
        self.alloc(MetaProperty { span, meta, property })
    }
    pub fn spread_element(
        self,
        span: Span,
        argument: Expression<'a>,
    ) -> Box<'a, SpreadElement<'a>> {
        self.alloc(SpreadElement { span, argument })
    }
    pub fn update_expression(
        self,
        span: Span,
        operator: UpdateOperator,
        prefix: bool,
        argument: SimpleAssignmentTarget<'a>,
    ) -> Box<'a, UpdateExpression<'a>> {
        self.alloc(UpdateExpression { span, operator, prefix, argument })
    }
    pub fn unary_expression(
        self,
        span: Span,
        operator: UnaryOperator,
        argument: Expression<'a>,
    ) -> Box<'a, UnaryExpression<'a>> {
        self.alloc(UnaryExpression { span, operator, argument })
    }
    pub fn binary_expression(
        self,
        span: Span,
        left: Expression<'a>,
        operator: BinaryOperator,
        right: Expression<'a>,
    ) -> Box<'a, BinaryExpression<'a>> {
        self.alloc(BinaryExpression { span, left, operator, right })
    }
    pub fn private_in_expression(
        self,
        span: Span,
        left: PrivateIdentifier<'a>,
        operator: BinaryOperator,
        right: Expression<'a>,
    ) -> Box<'a, PrivateInExpression<'a>> {
        self.alloc(PrivateInExpression { span, left, operator, right })
    }
    pub fn logical_expression(
        self,
        span: Span,
        left: Expression<'a>,
        operator: LogicalOperator,
        right: Expression<'a>,
    ) -> Box<'a, LogicalExpression<'a>> {
        self.alloc(LogicalExpression { span, left, operator, right })
    }
    pub fn conditional_expression(
        self,
        span: Span,
        test: Expression<'a>,
        consequent: Expression<'a>,
        alternate: Expression<'a>,
    ) -> Box<'a, ConditionalExpression<'a>> {
        self.alloc(ConditionalExpression { span, test, consequent, alternate })
    }
    pub fn assignment_expression(
        self,
        span: Span,
        operator: AssignmentOperator,
        left: AssignmentTarget<'a>,
        right: Expression<'a>,
    ) -> Box<'a, AssignmentExpression<'a>> {
        self.alloc(AssignmentExpression { span, operator, left, right })
    }
    pub fn array_assignment_target(
        self,
        span: Span,
        elements: Vec<'a, Option<AssignmentTargetMaybeDefault<'a>>>,
        rest: Option<AssignmentTargetRest<'a>>,
        trailing_comma: Option<Span>,
    ) -> Box<'a, ArrayAssignmentTarget<'a>> {
        self.alloc(ArrayAssignmentTarget { span, elements, rest, trailing_comma })
    }
    pub fn object_assignment_target(
        self,
        span: Span,
        properties: Vec<'a, AssignmentTargetProperty<'a>>,
        rest: Option<AssignmentTargetRest<'a>>,
    ) -> Box<'a, ObjectAssignmentTarget<'a>> {
        self.alloc(ObjectAssignmentTarget { span, properties, rest })
    }
    pub fn assignment_target_rest(
        self,
        span: Span,
        target: AssignmentTarget<'a>,
    ) -> Box<'a, AssignmentTargetRest<'a>> {
        self.alloc(AssignmentTargetRest { span, target })
    }
    pub fn assignment_target_with_default(
        self,
        span: Span,
        binding: AssignmentTarget<'a>,
        init: Expression<'a>,
    ) -> Box<'a, AssignmentTargetWithDefault<'a>> {
        self.alloc(AssignmentTargetWithDefault { span, binding, init })
    }
    pub fn assignment_target_property_identifier(
        self,
        span: Span,
        binding: IdentifierReference<'a>,
        init: Option<Expression<'a>>,
    ) -> Box<'a, AssignmentTargetPropertyIdentifier<'a>> {
        self.alloc(AssignmentTargetPropertyIdentifier { span, binding, init })
    }
    pub fn assignment_target_property_property(
        self,
        span: Span,
        name: PropertyKey<'a>,
        binding: AssignmentTargetMaybeDefault<'a>,
    ) -> Box<'a, AssignmentTargetPropertyProperty<'a>> {
        self.alloc(AssignmentTargetPropertyProperty { span, name, binding })
    }
    pub fn sequence_expression(
        self,
        span: Span,
        expressions: Vec<'a, Expression<'a>>,
    ) -> Box<'a, SequenceExpression<'a>> {
        self.alloc(SequenceExpression { span, expressions })
    }
    pub fn super_(self, span: Span) -> Box<'a, Super> {
        self.alloc(Super { span })
    }
    pub fn await_expression(
        self,
        span: Span,
        argument: Expression<'a>,
    ) -> Box<'a, AwaitExpression<'a>> {
        self.alloc(AwaitExpression { span, argument })
    }
    pub fn chain_expression(
        self,
        span: Span,
        expression: ChainElement<'a>,
    ) -> Box<'a, ChainExpression<'a>> {
        self.alloc(ChainExpression { span, expression })
    }
    pub fn parenthesized_expression(
        self,
        span: Span,
        expression: Expression<'a>,
    ) -> Box<'a, ParenthesizedExpression<'a>> {
        self.alloc(ParenthesizedExpression { span, expression })
    }
    pub fn directive(
        self,
        span: Span,
        expression: StringLiteral<'a>,
        directive: Atom<'a>,
    ) -> Box<'a, Directive<'a>> {
        self.alloc(Directive { span, expression, directive })
    }
    pub fn hashbang(self, span: Span, value: Atom<'a>) -> Box<'a, Hashbang<'a>> {
        self.alloc(Hashbang { span, value })
    }
    pub fn block_statement(
        self,
        span: Span,
        body: Vec<'a, Statement<'a>>,
    ) -> Box<'a, BlockStatement<'a>> {
        self.alloc(BlockStatement { span, body, scope_id: Default::default() })
    }
    pub fn variable_declaration(
        self,
        span: Span,
        kind: VariableDeclarationKind,
        declarations: Vec<'a, VariableDeclarator<'a>>,
        declare: bool,
    ) -> Box<'a, VariableDeclaration<'a>> {
        self.alloc(VariableDeclaration { span, kind, declarations, declare })
    }
    pub fn variable_declarator(
        self,
        span: Span,
        kind: VariableDeclarationKind,
        id: BindingPattern<'a>,
        init: Option<Expression<'a>>,
        definite: bool,
    ) -> Box<'a, VariableDeclarator<'a>> {
        self.alloc(VariableDeclarator { span, kind, id, init, definite })
    }
    pub fn using_declaration(
        self,
        span: Span,
        is_await: bool,
        declarations: Vec<'a, VariableDeclarator<'a>>,
    ) -> Box<'a, UsingDeclaration<'a>> {
        self.alloc(UsingDeclaration { span, is_await, declarations })
    }
    pub fn empty_statement(self, span: Span) -> Box<'a, EmptyStatement> {
        self.alloc(EmptyStatement { span })
    }
    pub fn expression_statement(
        self,
        span: Span,
        expression: Expression<'a>,
    ) -> Box<'a, ExpressionStatement<'a>> {
        self.alloc(ExpressionStatement { span, expression })
    }
    pub fn if_statement(
        self,
        span: Span,
        test: Expression<'a>,
        consequent: Statement<'a>,
        alternate: Option<Statement<'a>>,
    ) -> Box<'a, IfStatement<'a>> {
        self.alloc(IfStatement { span, test, consequent, alternate })
    }
    pub fn do_while_statement(
        self,
        span: Span,
        body: Statement<'a>,
        test: Expression<'a>,
    ) -> Box<'a, DoWhileStatement<'a>> {
        self.alloc(DoWhileStatement { span, body, test })
    }
    pub fn while_statement(
        self,
        span: Span,
        test: Expression<'a>,
        body: Statement<'a>,
    ) -> Box<'a, WhileStatement<'a>> {
        self.alloc(WhileStatement { span, test, body })
    }
    pub fn for_statement(
        self,
        span: Span,
        init: Option<ForStatementInit<'a>>,
        test: Option<Expression<'a>>,
        update: Option<Expression<'a>>,
        body: Statement<'a>,
    ) -> Box<'a, ForStatement<'a>> {
        self.alloc(ForStatement { span, init, test, update, body, scope_id: Default::default() })
    }
    pub fn for_in_statement(
        self,
        span: Span,
        left: ForStatementLeft<'a>,
        right: Expression<'a>,
        body: Statement<'a>,
    ) -> Box<'a, ForInStatement<'a>> {
        self.alloc(ForInStatement { span, left, right, body, scope_id: Default::default() })
    }
    pub fn for_of_statement(
        self,
        span: Span,
        r#await: bool,
        left: ForStatementLeft<'a>,
        right: Expression<'a>,
        body: Statement<'a>,
    ) -> Box<'a, ForOfStatement<'a>> {
        self.alloc(ForOfStatement {
            span,
            r#await,
            left,
            right,
            body,
            scope_id: Default::default(),
        })
    }
    pub fn continue_statement(
        self,
        span: Span,
        label: Option<LabelIdentifier<'a>>,
    ) -> Box<'a, ContinueStatement<'a>> {
        self.alloc(ContinueStatement { span, label })
    }
    pub fn break_statement(
        self,
        span: Span,
        label: Option<LabelIdentifier<'a>>,
    ) -> Box<'a, BreakStatement<'a>> {
        self.alloc(BreakStatement { span, label })
    }
    pub fn return_statement(
        self,
        span: Span,
        argument: Option<Expression<'a>>,
    ) -> Box<'a, ReturnStatement<'a>> {
        self.alloc(ReturnStatement { span, argument })
    }
    pub fn with_statement(
        self,
        span: Span,
        object: Expression<'a>,
        body: Statement<'a>,
    ) -> Box<'a, WithStatement<'a>> {
        self.alloc(WithStatement { span, object, body })
    }
    pub fn switch_statement(
        self,
        span: Span,
        discriminant: Expression<'a>,
        cases: Vec<'a, SwitchCase<'a>>,
    ) -> Box<'a, SwitchStatement<'a>> {
        self.alloc(SwitchStatement { span, discriminant, cases, scope_id: Default::default() })
    }
    pub fn switch_case(
        self,
        span: Span,
        test: Option<Expression<'a>>,
        consequent: Vec<'a, Statement<'a>>,
    ) -> Box<'a, SwitchCase<'a>> {
        self.alloc(SwitchCase { span, test, consequent })
    }
    pub fn labeled_statement(
        self,
        span: Span,
        label: LabelIdentifier<'a>,
        body: Statement<'a>,
    ) -> Box<'a, LabeledStatement<'a>> {
        self.alloc(LabeledStatement { span, label, body })
    }
    pub fn throw_statement(
        self,
        span: Span,
        argument: Expression<'a>,
    ) -> Box<'a, ThrowStatement<'a>> {
        self.alloc(ThrowStatement { span, argument })
    }
    pub fn try_statement(
        self,
        span: Span,
        block: Box<'a, BlockStatement<'a>>,
        handler: Option<Box<'a, CatchClause<'a>>>,
        finalizer: Option<Box<'a, BlockStatement<'a>>>,
    ) -> Box<'a, TryStatement<'a>> {
        self.alloc(TryStatement { span, block, handler, finalizer })
    }
    pub fn catch_clause(
        self,
        span: Span,
        param: Option<CatchParameter<'a>>,
        body: Box<'a, BlockStatement<'a>>,
    ) -> Box<'a, CatchClause<'a>> {
        self.alloc(CatchClause { span, param, body, scope_id: Default::default() })
    }
    pub fn catch_parameter(
        self,
        span: Span,
        pattern: BindingPattern<'a>,
    ) -> Box<'a, CatchParameter<'a>> {
        self.alloc(CatchParameter { span, pattern })
    }
    pub fn debugger_statement(self, span: Span) -> Box<'a, DebuggerStatement> {
        self.alloc(DebuggerStatement { span })
    }
    pub fn binding_pattern(
        self,
        kind: BindingPatternKind<'a>,
        type_annotation: Option<Box<'a, TSTypeAnnotation<'a>>>,
        optional: bool,
    ) -> Box<'a, BindingPattern<'a>> {
        self.alloc(BindingPattern { kind, type_annotation, optional })
    }
    pub fn assignment_pattern(
        self,
        span: Span,
        left: BindingPattern<'a>,
        right: Expression<'a>,
    ) -> Box<'a, AssignmentPattern<'a>> {
        self.alloc(AssignmentPattern { span, left, right })
    }
    pub fn object_pattern(
        self,
        span: Span,
        properties: Vec<'a, BindingProperty<'a>>,
        rest: Option<Box<'a, BindingRestElement<'a>>>,
    ) -> Box<'a, ObjectPattern<'a>> {
        self.alloc(ObjectPattern { span, properties, rest })
    }
    pub fn binding_property(
        self,
        span: Span,
        key: PropertyKey<'a>,
        value: BindingPattern<'a>,
        shorthand: bool,
        computed: bool,
    ) -> Box<'a, BindingProperty<'a>> {
        self.alloc(BindingProperty { span, key, value, shorthand, computed })
    }
    pub fn array_pattern(
        self,
        span: Span,
        elements: Vec<'a, Option<BindingPattern<'a>>>,
        rest: Option<Box<'a, BindingRestElement<'a>>>,
    ) -> Box<'a, ArrayPattern<'a>> {
        self.alloc(ArrayPattern { span, elements, rest })
    }
    pub fn binding_rest_element(
        self,
        span: Span,
        argument: BindingPattern<'a>,
    ) -> Box<'a, BindingRestElement<'a>> {
        self.alloc(BindingRestElement { span, argument })
    }
    pub fn function(
        self,
        r#type: FunctionType,
        span: Span,
        id: Option<BindingIdentifier<'a>>,
        generator: bool,
        r#async: bool,
        declare: bool,
        type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
        this_param: Option<TSThisParameter<'a>>,
        params: Box<'a, FormalParameters<'a>>,
        body: Option<Box<'a, FunctionBody<'a>>>,
        return_type: Option<Box<'a, TSTypeAnnotation<'a>>>,
    ) -> Box<'a, Function<'a>> {
        self.alloc(Function {
            r#type,
            span,
            id,
            generator,
            r#async,
            declare,
            type_parameters,
            this_param,
            params,
            body,
            return_type,
            scope_id: Default::default(),
        })
    }
    pub fn formal_parameters(
        self,
        span: Span,
        kind: FormalParameterKind,
        items: Vec<'a, FormalParameter<'a>>,
        rest: Option<Box<'a, BindingRestElement<'a>>>,
    ) -> Box<'a, FormalParameters<'a>> {
        self.alloc(FormalParameters { span, kind, items, rest })
    }
    pub fn formal_parameter(
        self,
        span: Span,
        pattern: BindingPattern<'a>,
        accessibility: Option<TSAccessibility>,
        readonly: bool,
        r#override: bool,
        decorators: Vec<'a, Decorator<'a>>,
    ) -> Box<'a, FormalParameter<'a>> {
        self.alloc(FormalParameter {
            span,
            pattern,
            accessibility,
            readonly,
            r#override,
            decorators,
        })
    }
    pub fn function_body(
        self,
        span: Span,
        directives: Vec<'a, Directive<'a>>,
        statements: Vec<'a, Statement<'a>>,
    ) -> Box<'a, FunctionBody<'a>> {
        self.alloc(FunctionBody { span, directives, statements })
    }
    pub fn arrow_function_expression(
        self,
        span: Span,
        expression: bool,
        r#async: bool,
        params: Box<'a, FormalParameters<'a>>,
        body: Box<'a, FunctionBody<'a>>,
        type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
        return_type: Option<Box<'a, TSTypeAnnotation<'a>>>,
    ) -> Box<'a, ArrowFunctionExpression<'a>> {
        self.alloc(ArrowFunctionExpression {
            span,
            expression,
            r#async,
            params,
            body,
            type_parameters,
            return_type,
            scope_id: Default::default(),
        })
    }
    pub fn yield_expression(
        self,
        span: Span,
        delegate: bool,
        argument: Option<Expression<'a>>,
    ) -> Box<'a, YieldExpression<'a>> {
        self.alloc(YieldExpression { span, delegate, argument })
    }
    pub fn class(
        self,
        r#type: ClassType,
        span: Span,
        decorators: Vec<'a, Decorator<'a>>,
        id: Option<BindingIdentifier<'a>>,
        super_class: Option<Expression<'a>>,
        body: Box<'a, ClassBody<'a>>,
        type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
        super_type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
        implements: Option<Vec<'a, TSClassImplements<'a>>>,
        r#abstract: bool,
        declare: bool,
    ) -> Box<'a, Class<'a>> {
        self.alloc(Class {
            r#type,
            span,
            decorators,
            id,
            super_class,
            body,
            type_parameters,
            super_type_parameters,
            implements,
            r#abstract,
            declare,
            scope_id: Default::default(),
        })
    }
    pub fn class_body(self, span: Span, body: Vec<'a, ClassElement<'a>>) -> Box<'a, ClassBody<'a>> {
        self.alloc(ClassBody { span, body })
    }
    pub fn method_definition(
        self,
        r#type: MethodDefinitionType,
        span: Span,
        decorators: Vec<'a, Decorator<'a>>,
        key: PropertyKey<'a>,
        value: Box<'a, Function<'a>>,
        kind: MethodDefinitionKind,
        computed: bool,
        r#static: bool,
        r#override: bool,
        optional: bool,
        accessibility: Option<TSAccessibility>,
    ) -> Box<'a, MethodDefinition<'a>> {
        self.alloc(MethodDefinition {
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
        })
    }
    pub fn property_definition(
        self,
        r#type: PropertyDefinitionType,
        span: Span,
        decorators: Vec<'a, Decorator<'a>>,
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
    ) -> Box<'a, PropertyDefinition<'a>> {
        self.alloc(PropertyDefinition {
            r#type,
            span,
            decorators,
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
        })
    }
    pub fn private_identifier(self, span: Span, name: Atom<'a>) -> Box<'a, PrivateIdentifier<'a>> {
        self.alloc(PrivateIdentifier { span, name })
    }
    pub fn static_block(
        self,
        span: Span,
        body: Vec<'a, Statement<'a>>,
    ) -> Box<'a, StaticBlock<'a>> {
        self.alloc(StaticBlock { span, body, scope_id: Default::default() })
    }
    pub fn accessor_property(
        self,
        r#type: AccessorPropertyType,
        span: Span,
        key: PropertyKey<'a>,
        value: Option<Expression<'a>>,
        computed: bool,
        r#static: bool,
        decorators: Vec<'a, Decorator<'a>>,
    ) -> Box<'a, AccessorProperty<'a>> {
        self.alloc(AccessorProperty { r#type, span, key, value, computed, r#static, decorators })
    }
    pub fn import_expression(
        self,
        span: Span,
        source: Expression<'a>,
        arguments: Vec<'a, Expression<'a>>,
    ) -> Box<'a, ImportExpression<'a>> {
        self.alloc(ImportExpression { span, source, arguments })
    }
    pub fn import_declaration(
        self,
        span: Span,
        specifiers: Option<Vec<'a, ImportDeclarationSpecifier<'a>>>,
        source: StringLiteral<'a>,
        with_clause: Option<WithClause<'a>>,
        import_kind: ImportOrExportKind,
    ) -> Box<'a, ImportDeclaration<'a>> {
        self.alloc(ImportDeclaration { span, specifiers, source, with_clause, import_kind })
    }
    pub fn import_specifier(
        self,
        span: Span,
        imported: ModuleExportName<'a>,
        local: BindingIdentifier<'a>,
        import_kind: ImportOrExportKind,
    ) -> Box<'a, ImportSpecifier<'a>> {
        self.alloc(ImportSpecifier { span, imported, local, import_kind })
    }
    pub fn import_default_specifier(
        self,
        span: Span,
        local: BindingIdentifier<'a>,
    ) -> Box<'a, ImportDefaultSpecifier<'a>> {
        self.alloc(ImportDefaultSpecifier { span, local })
    }
    pub fn import_namespace_specifier(
        self,
        span: Span,
        local: BindingIdentifier<'a>,
    ) -> Box<'a, ImportNamespaceSpecifier<'a>> {
        self.alloc(ImportNamespaceSpecifier { span, local })
    }
    pub fn with_clause(
        self,
        span: Span,
        attributes_keyword: IdentifierName<'a>,
        with_entries: Vec<'a, ImportAttribute<'a>>,
    ) -> Box<'a, WithClause<'a>> {
        self.alloc(WithClause { span, attributes_keyword, with_entries })
    }
    pub fn import_attribute(
        self,
        span: Span,
        key: ImportAttributeKey<'a>,
        value: StringLiteral<'a>,
    ) -> Box<'a, ImportAttribute<'a>> {
        self.alloc(ImportAttribute { span, key, value })
    }
    pub fn export_named_declaration(
        self,
        span: Span,
        declaration: Option<Declaration<'a>>,
        specifiers: Vec<'a, ExportSpecifier<'a>>,
        source: Option<StringLiteral<'a>>,
        export_kind: ImportOrExportKind,
        with_clause: Option<WithClause<'a>>,
    ) -> Box<'a, ExportNamedDeclaration<'a>> {
        self.alloc(ExportNamedDeclaration {
            span,
            declaration,
            specifiers,
            source,
            export_kind,
            with_clause,
        })
    }
    pub fn export_default_declaration(
        self,
        span: Span,
        declaration: ExportDefaultDeclarationKind<'a>,
        exported: ModuleExportName<'a>,
    ) -> Box<'a, ExportDefaultDeclaration<'a>> {
        self.alloc(ExportDefaultDeclaration { span, declaration, exported })
    }
    pub fn export_all_declaration(
        self,
        span: Span,
        exported: Option<ModuleExportName<'a>>,
        source: StringLiteral<'a>,
        with_clause: Option<WithClause<'a>>,
        export_kind: ImportOrExportKind,
    ) -> Box<'a, ExportAllDeclaration<'a>> {
        self.alloc(ExportAllDeclaration { span, exported, source, with_clause, export_kind })
    }
    pub fn export_specifier(
        self,
        span: Span,
        local: ModuleExportName<'a>,
        exported: ModuleExportName<'a>,
        export_kind: ImportOrExportKind,
    ) -> Box<'a, ExportSpecifier<'a>> {
        self.alloc(ExportSpecifier { span, local, exported, export_kind })
    }
    pub fn ts_this_parameter(
        self,
        span: Span,
        this: IdentifierName<'a>,
        type_annotation: Option<Box<'a, TSTypeAnnotation<'a>>>,
    ) -> Box<'a, TSThisParameter<'a>> {
        self.alloc(TSThisParameter { span, this, type_annotation })
    }
    pub fn ts_enum_declaration(
        self,
        span: Span,
        id: BindingIdentifier<'a>,
        members: Vec<'a, TSEnumMember<'a>>,
        r#const: bool,
        declare: bool,
    ) -> Box<'a, TSEnumDeclaration<'a>> {
        self.alloc(TSEnumDeclaration {
            span,
            id,
            members,
            r#const,
            declare,
            scope_id: Default::default(),
        })
    }
    pub fn ts_enum_member(
        self,
        span: Span,
        id: TSEnumMemberName<'a>,
        initializer: Option<Expression<'a>>,
    ) -> Box<'a, TSEnumMember<'a>> {
        self.alloc(TSEnumMember { span, id, initializer })
    }
    pub fn ts_type_annotation(
        self,
        span: Span,
        type_annotation: TSType<'a>,
    ) -> Box<'a, TSTypeAnnotation<'a>> {
        self.alloc(TSTypeAnnotation { span, type_annotation })
    }
    pub fn ts_literal_type(self, span: Span, literal: TSLiteral<'a>) -> Box<'a, TSLiteralType<'a>> {
        self.alloc(TSLiteralType { span, literal })
    }
    pub fn ts_conditional_type(
        self,
        span: Span,
        check_type: TSType<'a>,
        extends_type: TSType<'a>,
        true_type: TSType<'a>,
        false_type: TSType<'a>,
    ) -> Box<'a, TSConditionalType<'a>> {
        self.alloc(TSConditionalType { span, check_type, extends_type, true_type, false_type })
    }
    pub fn ts_union_type(self, span: Span, types: Vec<'a, TSType<'a>>) -> Box<'a, TSUnionType<'a>> {
        self.alloc(TSUnionType { span, types })
    }
    pub fn ts_intersection_type(
        self,
        span: Span,
        types: Vec<'a, TSType<'a>>,
    ) -> Box<'a, TSIntersectionType<'a>> {
        self.alloc(TSIntersectionType { span, types })
    }
    pub fn ts_parenthesized_type(
        self,
        span: Span,
        type_annotation: TSType<'a>,
    ) -> Box<'a, TSParenthesizedType<'a>> {
        self.alloc(TSParenthesizedType { span, type_annotation })
    }
    pub fn ts_type_operator(
        self,
        span: Span,
        operator: TSTypeOperatorOperator,
        type_annotation: TSType<'a>,
    ) -> Box<'a, TSTypeOperator<'a>> {
        self.alloc(TSTypeOperator { span, operator, type_annotation })
    }
    pub fn ts_array_type(self, span: Span, element_type: TSType<'a>) -> Box<'a, TSArrayType<'a>> {
        self.alloc(TSArrayType { span, element_type })
    }
    pub fn ts_indexed_access_type(
        self,
        span: Span,
        object_type: TSType<'a>,
        index_type: TSType<'a>,
    ) -> Box<'a, TSIndexedAccessType<'a>> {
        self.alloc(TSIndexedAccessType { span, object_type, index_type })
    }
    pub fn ts_tuple_type(
        self,
        span: Span,
        element_types: Vec<'a, TSTupleElement<'a>>,
    ) -> Box<'a, TSTupleType<'a>> {
        self.alloc(TSTupleType { span, element_types })
    }
    pub fn ts_named_tuple_member(
        self,
        span: Span,
        element_type: TSTupleElement<'a>,
        label: IdentifierName<'a>,
        optional: bool,
    ) -> Box<'a, TSNamedTupleMember<'a>> {
        self.alloc(TSNamedTupleMember { span, element_type, label, optional })
    }
    pub fn ts_optional_type(
        self,
        span: Span,
        type_annotation: TSType<'a>,
    ) -> Box<'a, TSOptionalType<'a>> {
        self.alloc(TSOptionalType { span, type_annotation })
    }
    pub fn ts_rest_type(self, span: Span, type_annotation: TSType<'a>) -> Box<'a, TSRestType<'a>> {
        self.alloc(TSRestType { span, type_annotation })
    }
    pub fn ts_any_keyword(self, span: Span) -> Box<'a, TSAnyKeyword> {
        self.alloc(TSAnyKeyword { span })
    }
    pub fn ts_string_keyword(self, span: Span) -> Box<'a, TSStringKeyword> {
        self.alloc(TSStringKeyword { span })
    }
    pub fn ts_boolean_keyword(self, span: Span) -> Box<'a, TSBooleanKeyword> {
        self.alloc(TSBooleanKeyword { span })
    }
    pub fn ts_number_keyword(self, span: Span) -> Box<'a, TSNumberKeyword> {
        self.alloc(TSNumberKeyword { span })
    }
    pub fn ts_never_keyword(self, span: Span) -> Box<'a, TSNeverKeyword> {
        self.alloc(TSNeverKeyword { span })
    }
    pub fn ts_intrinsic_keyword(self, span: Span) -> Box<'a, TSIntrinsicKeyword> {
        self.alloc(TSIntrinsicKeyword { span })
    }
    pub fn ts_unknown_keyword(self, span: Span) -> Box<'a, TSUnknownKeyword> {
        self.alloc(TSUnknownKeyword { span })
    }
    pub fn ts_null_keyword(self, span: Span) -> Box<'a, TSNullKeyword> {
        self.alloc(TSNullKeyword { span })
    }
    pub fn ts_undefined_keyword(self, span: Span) -> Box<'a, TSUndefinedKeyword> {
        self.alloc(TSUndefinedKeyword { span })
    }
    pub fn ts_void_keyword(self, span: Span) -> Box<'a, TSVoidKeyword> {
        self.alloc(TSVoidKeyword { span })
    }
    pub fn ts_symbol_keyword(self, span: Span) -> Box<'a, TSSymbolKeyword> {
        self.alloc(TSSymbolKeyword { span })
    }
    pub fn ts_this_type(self, span: Span) -> Box<'a, TSThisType> {
        self.alloc(TSThisType { span })
    }
    pub fn ts_object_keyword(self, span: Span) -> Box<'a, TSObjectKeyword> {
        self.alloc(TSObjectKeyword { span })
    }
    pub fn ts_big_int_keyword(self, span: Span) -> Box<'a, TSBigIntKeyword> {
        self.alloc(TSBigIntKeyword { span })
    }
    pub fn ts_type_reference(
        self,
        span: Span,
        type_name: TSTypeName<'a>,
        type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
    ) -> Box<'a, TSTypeReference<'a>> {
        self.alloc(TSTypeReference { span, type_name, type_parameters })
    }
    pub fn ts_qualified_name(
        self,
        span: Span,
        left: TSTypeName<'a>,
        right: IdentifierName<'a>,
    ) -> Box<'a, TSQualifiedName<'a>> {
        self.alloc(TSQualifiedName { span, left, right })
    }
    pub fn ts_type_parameter_instantiation(
        self,
        span: Span,
        params: Vec<'a, TSType<'a>>,
    ) -> Box<'a, TSTypeParameterInstantiation<'a>> {
        self.alloc(TSTypeParameterInstantiation { span, params })
    }
    pub fn ts_type_parameter(
        self,
        span: Span,
        name: BindingIdentifier<'a>,
        constraint: Option<TSType<'a>>,
        default: Option<TSType<'a>>,
        r#in: bool,
        out: bool,
        r#const: bool,
    ) -> Box<'a, TSTypeParameter<'a>> {
        self.alloc(TSTypeParameter {
            span,
            name,
            constraint,
            default,
            r#in,
            out,
            r#const,
            scope_id: Default::default(),
        })
    }
    pub fn ts_type_parameter_declaration(
        self,
        span: Span,
        params: Vec<'a, TSTypeParameter<'a>>,
    ) -> Box<'a, TSTypeParameterDeclaration<'a>> {
        self.alloc(TSTypeParameterDeclaration { span, params })
    }
    pub fn ts_type_alias_declaration(
        self,
        span: Span,
        id: BindingIdentifier<'a>,
        type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
        type_annotation: TSType<'a>,
        declare: bool,
    ) -> Box<'a, TSTypeAliasDeclaration<'a>> {
        self.alloc(TSTypeAliasDeclaration { span, id, type_parameters, type_annotation, declare })
    }
    pub fn ts_class_implements(
        self,
        span: Span,
        expression: TSTypeName<'a>,
        type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
    ) -> Box<'a, TSClassImplements<'a>> {
        self.alloc(TSClassImplements { span, expression, type_parameters })
    }
    pub fn ts_interface_declaration(
        self,
        span: Span,
        id: BindingIdentifier<'a>,
        extends: Option<Vec<'a, TSInterfaceHeritage<'a>>>,
        type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
        body: Box<'a, TSInterfaceBody<'a>>,
        declare: bool,
    ) -> Box<'a, TSInterfaceDeclaration<'a>> {
        self.alloc(TSInterfaceDeclaration { span, id, extends, type_parameters, body, declare })
    }
    pub fn ts_interface_body(
        self,
        span: Span,
        body: Vec<'a, TSSignature<'a>>,
    ) -> Box<'a, TSInterfaceBody<'a>> {
        self.alloc(TSInterfaceBody { span, body })
    }
    pub fn ts_property_signature(
        self,
        span: Span,
        computed: bool,
        optional: bool,
        readonly: bool,
        key: PropertyKey<'a>,
        type_annotation: Option<Box<'a, TSTypeAnnotation<'a>>>,
    ) -> Box<'a, TSPropertySignature<'a>> {
        self.alloc(TSPropertySignature { span, computed, optional, readonly, key, type_annotation })
    }
    pub fn ts_index_signature(
        self,
        span: Span,
        parameters: Vec<'a, TSIndexSignatureName<'a>>,
        type_annotation: Box<'a, TSTypeAnnotation<'a>>,
        readonly: bool,
    ) -> Box<'a, TSIndexSignature<'a>> {
        self.alloc(TSIndexSignature { span, parameters, type_annotation, readonly })
    }
    pub fn ts_call_signature_declaration(
        self,
        span: Span,
        this_param: Option<TSThisParameter<'a>>,
        params: Box<'a, FormalParameters<'a>>,
        return_type: Option<Box<'a, TSTypeAnnotation<'a>>>,
        type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
    ) -> Box<'a, TSCallSignatureDeclaration<'a>> {
        self.alloc(TSCallSignatureDeclaration {
            span,
            this_param,
            params,
            return_type,
            type_parameters,
        })
    }
    pub fn ts_method_signature(
        self,
        span: Span,
        key: PropertyKey<'a>,
        computed: bool,
        optional: bool,
        kind: TSMethodSignatureKind,
        this_param: Option<TSThisParameter<'a>>,
        params: Box<'a, FormalParameters<'a>>,
        return_type: Option<Box<'a, TSTypeAnnotation<'a>>>,
        type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
    ) -> Box<'a, TSMethodSignature<'a>> {
        self.alloc(TSMethodSignature {
            span,
            key,
            computed,
            optional,
            kind,
            this_param,
            params,
            return_type,
            type_parameters,
        })
    }
    pub fn ts_construct_signature_declaration(
        self,
        span: Span,
        params: Box<'a, FormalParameters<'a>>,
        return_type: Option<Box<'a, TSTypeAnnotation<'a>>>,
        type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
    ) -> Box<'a, TSConstructSignatureDeclaration<'a>> {
        self.alloc(TSConstructSignatureDeclaration { span, params, return_type, type_parameters })
    }
    pub fn ts_index_signature_name(
        self,
        span: Span,
        name: Atom<'a>,
        type_annotation: Box<'a, TSTypeAnnotation<'a>>,
    ) -> Box<'a, TSIndexSignatureName<'a>> {
        self.alloc(TSIndexSignatureName { span, name, type_annotation })
    }
    pub fn ts_interface_heritage(
        self,
        span: Span,
        expression: Expression<'a>,
        type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
    ) -> Box<'a, TSInterfaceHeritage<'a>> {
        self.alloc(TSInterfaceHeritage { span, expression, type_parameters })
    }
    pub fn ts_type_predicate(
        self,
        span: Span,
        parameter_name: TSTypePredicateName<'a>,
        asserts: bool,
        type_annotation: Option<Box<'a, TSTypeAnnotation<'a>>>,
    ) -> Box<'a, TSTypePredicate<'a>> {
        self.alloc(TSTypePredicate { span, parameter_name, asserts, type_annotation })
    }
    pub fn ts_module_declaration(
        self,
        span: Span,
        id: TSModuleDeclarationName<'a>,
        body: Option<TSModuleDeclarationBody<'a>>,
        kind: TSModuleDeclarationKind,
        declare: bool,
    ) -> Box<'a, TSModuleDeclaration<'a>> {
        self.alloc(TSModuleDeclaration {
            span,
            id,
            body,
            kind,
            declare,
            scope_id: Default::default(),
        })
    }
    pub fn ts_module_block(
        self,
        span: Span,
        directives: Vec<'a, Directive<'a>>,
        body: Vec<'a, Statement<'a>>,
    ) -> Box<'a, TSModuleBlock<'a>> {
        self.alloc(TSModuleBlock { span, directives, body })
    }
    pub fn ts_type_literal(
        self,
        span: Span,
        members: Vec<'a, TSSignature<'a>>,
    ) -> Box<'a, TSTypeLiteral<'a>> {
        self.alloc(TSTypeLiteral { span, members })
    }
    pub fn ts_infer_type(
        self,
        span: Span,
        type_parameter: Box<'a, TSTypeParameter<'a>>,
    ) -> Box<'a, TSInferType<'a>> {
        self.alloc(TSInferType { span, type_parameter })
    }
    pub fn ts_type_query(
        self,
        span: Span,
        expr_name: TSTypeQueryExprName<'a>,
        type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
    ) -> Box<'a, TSTypeQuery<'a>> {
        self.alloc(TSTypeQuery { span, expr_name, type_parameters })
    }
    pub fn ts_import_type(
        self,
        span: Span,
        is_type_of: bool,
        parameter: TSType<'a>,
        qualifier: Option<TSTypeName<'a>>,
        attributes: Option<TSImportAttributes<'a>>,
        type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
    ) -> Box<'a, TSImportType<'a>> {
        self.alloc(TSImportType {
            span,
            is_type_of,
            parameter,
            qualifier,
            attributes,
            type_parameters,
        })
    }
    pub fn ts_import_attributes(
        self,
        span: Span,
        elements: Vec<'a, TSImportAttribute<'a>>,
    ) -> Box<'a, TSImportAttributes<'a>> {
        self.alloc(TSImportAttributes { span, elements })
    }
    pub fn ts_import_attribute(
        self,
        span: Span,
        name: TSImportAttributeName<'a>,
        value: Expression<'a>,
    ) -> Box<'a, TSImportAttribute<'a>> {
        self.alloc(TSImportAttribute { span, name, value })
    }
    pub fn ts_function_type(
        self,
        span: Span,
        this_param: Option<TSThisParameter<'a>>,
        params: Box<'a, FormalParameters<'a>>,
        return_type: Box<'a, TSTypeAnnotation<'a>>,
        type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
    ) -> Box<'a, TSFunctionType<'a>> {
        self.alloc(TSFunctionType { span, this_param, params, return_type, type_parameters })
    }
    pub fn ts_constructor_type(
        self,
        span: Span,
        r#abstract: bool,
        params: Box<'a, FormalParameters<'a>>,
        return_type: Box<'a, TSTypeAnnotation<'a>>,
        type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
    ) -> Box<'a, TSConstructorType<'a>> {
        self.alloc(TSConstructorType { span, r#abstract, params, return_type, type_parameters })
    }
    pub fn ts_mapped_type(
        self,
        span: Span,
        type_parameter: Box<'a, TSTypeParameter<'a>>,
        name_type: Option<TSType<'a>>,
        type_annotation: Option<TSType<'a>>,
        optional: TSMappedTypeModifierOperator,
        readonly: TSMappedTypeModifierOperator,
    ) -> Box<'a, TSMappedType<'a>> {
        self.alloc(TSMappedType {
            span,
            type_parameter,
            name_type,
            type_annotation,
            optional,
            readonly,
        })
    }
    pub fn ts_template_literal_type(
        self,
        span: Span,
        quasis: Vec<'a, TemplateElement<'a>>,
        types: Vec<'a, TSType<'a>>,
    ) -> Box<'a, TSTemplateLiteralType<'a>> {
        self.alloc(TSTemplateLiteralType { span, quasis, types })
    }
    pub fn ts_as_expression(
        self,
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
    ) -> Box<'a, TSAsExpression<'a>> {
        self.alloc(TSAsExpression { span, expression, type_annotation })
    }
    pub fn ts_satisfies_expression(
        self,
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
    ) -> Box<'a, TSSatisfiesExpression<'a>> {
        self.alloc(TSSatisfiesExpression { span, expression, type_annotation })
    }
    pub fn ts_type_assertion(
        self,
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
    ) -> Box<'a, TSTypeAssertion<'a>> {
        self.alloc(TSTypeAssertion { span, expression, type_annotation })
    }
    pub fn ts_import_equals_declaration(
        self,
        span: Span,
        id: BindingIdentifier<'a>,
        module_reference: TSModuleReference<'a>,
        import_kind: ImportOrExportKind,
    ) -> Box<'a, TSImportEqualsDeclaration<'a>> {
        self.alloc(TSImportEqualsDeclaration { span, id, module_reference, import_kind })
    }
    pub fn ts_external_module_reference(
        self,
        span: Span,
        expression: StringLiteral<'a>,
    ) -> Box<'a, TSExternalModuleReference<'a>> {
        self.alloc(TSExternalModuleReference { span, expression })
    }
    pub fn ts_non_null_expression(
        self,
        span: Span,
        expression: Expression<'a>,
    ) -> Box<'a, TSNonNullExpression<'a>> {
        self.alloc(TSNonNullExpression { span, expression })
    }
    pub fn decorator(self, span: Span, expression: Expression<'a>) -> Box<'a, Decorator<'a>> {
        self.alloc(Decorator { span, expression })
    }
    pub fn ts_export_assignment(
        self,
        span: Span,
        expression: Expression<'a>,
    ) -> Box<'a, TSExportAssignment<'a>> {
        self.alloc(TSExportAssignment { span, expression })
    }
    pub fn ts_namespace_export_declaration(
        self,
        span: Span,
        id: IdentifierName<'a>,
    ) -> Box<'a, TSNamespaceExportDeclaration<'a>> {
        self.alloc(TSNamespaceExportDeclaration { span, id })
    }
    pub fn ts_instantiation_expression(
        self,
        span: Span,
        expression: Expression<'a>,
        type_parameters: Box<'a, TSTypeParameterInstantiation<'a>>,
    ) -> Box<'a, TSInstantiationExpression<'a>> {
        self.alloc(TSInstantiationExpression { span, expression, type_parameters })
    }
    pub fn js_doc_nullable_type(
        self,
        span: Span,
        type_annotation: TSType<'a>,
        postfix: bool,
    ) -> Box<'a, JSDocNullableType<'a>> {
        self.alloc(JSDocNullableType { span, type_annotation, postfix })
    }
    pub fn js_doc_non_nullable_type(
        self,
        span: Span,
        type_annotation: TSType<'a>,
        postfix: bool,
    ) -> Box<'a, JSDocNonNullableType<'a>> {
        self.alloc(JSDocNonNullableType { span, type_annotation, postfix })
    }
    pub fn js_doc_unknown_type(self, span: Span) -> Box<'a, JSDocUnknownType> {
        self.alloc(JSDocUnknownType { span })
    }
    pub fn jsx_element(
        self,
        span: Span,
        opening_element: Box<'a, JSXOpeningElement<'a>>,
        closing_element: Option<Box<'a, JSXClosingElement<'a>>>,
        children: Vec<'a, JSXChild<'a>>,
    ) -> Box<'a, JSXElement<'a>> {
        self.alloc(JSXElement { span, opening_element, closing_element, children })
    }
    pub fn jsx_opening_element(
        self,
        span: Span,
        self_closing: bool,
        name: JSXElementName<'a>,
        attributes: Vec<'a, JSXAttributeItem<'a>>,
        type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
    ) -> Box<'a, JSXOpeningElement<'a>> {
        self.alloc(JSXOpeningElement { span, self_closing, name, attributes, type_parameters })
    }
    pub fn jsx_closing_element(
        self,
        span: Span,
        name: JSXElementName<'a>,
    ) -> Box<'a, JSXClosingElement<'a>> {
        self.alloc(JSXClosingElement { span, name })
    }
    pub fn jsx_fragment(
        self,
        span: Span,
        opening_fragment: JSXOpeningFragment,
        closing_fragment: JSXClosingFragment,
        children: Vec<'a, JSXChild<'a>>,
    ) -> Box<'a, JSXFragment<'a>> {
        self.alloc(JSXFragment { span, opening_fragment, closing_fragment, children })
    }
    pub fn jsx_opening_fragment(self, span: Span) -> Box<'a, JSXOpeningFragment> {
        self.alloc(JSXOpeningFragment { span })
    }
    pub fn jsx_closing_fragment(self, span: Span) -> Box<'a, JSXClosingFragment> {
        self.alloc(JSXClosingFragment { span })
    }
    pub fn jsx_namespaced_name(
        self,
        span: Span,
        namespace: JSXIdentifier<'a>,
        property: JSXIdentifier<'a>,
    ) -> Box<'a, JSXNamespacedName<'a>> {
        self.alloc(JSXNamespacedName { span, namespace, property })
    }
    pub fn jsx_member_expression(
        self,
        span: Span,
        object: JSXMemberExpressionObject<'a>,
        property: JSXIdentifier<'a>,
    ) -> Box<'a, JSXMemberExpression<'a>> {
        self.alloc(JSXMemberExpression { span, object, property })
    }
    pub fn jsx_expression_container(
        self,
        span: Span,
        expression: JSXExpression<'a>,
    ) -> Box<'a, JSXExpressionContainer<'a>> {
        self.alloc(JSXExpressionContainer { span, expression })
    }
    pub fn jsx_empty_expression(self, span: Span) -> Box<'a, JSXEmptyExpression> {
        self.alloc(JSXEmptyExpression { span })
    }
    pub fn jsx_attribute(
        self,
        span: Span,
        name: JSXAttributeName<'a>,
        value: Option<JSXAttributeValue<'a>>,
    ) -> Box<'a, JSXAttribute<'a>> {
        self.alloc(JSXAttribute { span, name, value })
    }
    pub fn jsx_spread_attribute(
        self,
        span: Span,
        argument: Expression<'a>,
    ) -> Box<'a, JSXSpreadAttribute<'a>> {
        self.alloc(JSXSpreadAttribute { span, argument })
    }
    pub fn jsx_identifier(self, span: Span, name: Atom<'a>) -> Box<'a, JSXIdentifier<'a>> {
        self.alloc(JSXIdentifier { span, name })
    }
    pub fn jsx_spread_child(
        self,
        span: Span,
        expression: Expression<'a>,
    ) -> Box<'a, JSXSpreadChild<'a>> {
        self.alloc(JSXSpreadChild { span, expression })
    }
    pub fn jsx_text(self, span: Span, value: Atom<'a>) -> Box<'a, JSXText<'a>> {
        self.alloc(JSXText { span, value })
    }
}
