use oxc_allocator::{Allocator, Box, Vec};
use oxc_span::{Atom, SourceType, Span};
use oxc_syntax::{
    number::{BigintBase, NumberBase},
    operator::{
        AssignmentOperator, BinaryOperator, LogicalOperator, UnaryOperator,
        UpdateOperator,
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
    fn boolean_literal(self, span: Span, value: bool) -> BooleanLiteral {
        BooleanLiteral { span, value }
    }
    fn null_literal(self, span: Span) -> NullLiteral {
        NullLiteral { span }
    }
    fn numeric_literal(
        self,
        span: Span,
        value: f64,
        raw: &'a str,
        base: NumberBase,
    ) -> NumericLiteral<'a> {
        NumericLiteral {
            span,
            value,
            raw,
            base,
        }
    }
    fn big_int_literal(
        self,
        span: Span,
        raw: Atom<'a>,
        base: BigintBase,
    ) -> BigIntLiteral<'a> {
        BigIntLiteral { span, raw, base }
    }
    fn reg_exp_literal(
        self,
        span: Span,
        value: EmptyObject,
        regex: RegExp<'a>,
    ) -> RegExpLiteral<'a> {
        RegExpLiteral {
            span,
            value,
            regex,
        }
    }
    fn reg_exp(self, pattern: Atom<'a>, flags: RegExpFlags) -> RegExp<'a> {
        RegExp { pattern, flags }
    }
    fn empty_object(self) -> EmptyObject {
        EmptyObject {}
    }
    fn string_literal(self, span: Span, value: Atom<'a>) -> StringLiteral<'a> {
        StringLiteral { span, value }
    }
    fn program(
        self,
        span: Span,
        source_type: SourceType,
        directives: Vec<'a, Directive<'a>>,
        hashbang: Option<Hashbang<'a>>,
        body: Vec<'a, Statement<'a>>,
    ) -> Program<'a> {
        Program {
            span,
            source_type,
            directives,
            hashbang,
            body,
            scope_id: Default::default(),
        }
    }
    fn identifier_name(self, span: Span, name: Atom<'a>) -> IdentifierName<'a> {
        IdentifierName { span, name }
    }
    fn identifier_reference(
        self,
        span: Span,
        name: Atom<'a>,
    ) -> IdentifierReference<'a> {
        IdentifierReference {
            span,
            name,
            reference_id: Default::default(),
            reference_flag: Default::default(),
        }
    }
    fn binding_identifier(self, span: Span, name: Atom<'a>) -> BindingIdentifier<'a> {
        BindingIdentifier {
            span,
            name,
            symbol_id: Default::default(),
        }
    }
    fn label_identifier(self, span: Span, name: Atom<'a>) -> LabelIdentifier<'a> {
        LabelIdentifier { span, name }
    }
    fn this_expression(self, span: Span) -> ThisExpression {
        ThisExpression { span }
    }
    fn array_expression(
        self,
        span: Span,
        elements: Vec<'a, ArrayExpressionElement<'a>>,
        trailing_comma: Option<Span>,
    ) -> ArrayExpression<'a> {
        ArrayExpression {
            span,
            elements,
            trailing_comma,
        }
    }
    fn elision(self, span: Span) -> Elision {
        Elision { span }
    }
    fn object_expression(
        self,
        span: Span,
        properties: Vec<'a, ObjectPropertyKind<'a>>,
        trailing_comma: Option<Span>,
    ) -> ObjectExpression<'a> {
        ObjectExpression {
            span,
            properties,
            trailing_comma,
        }
    }
    fn object_property(
        self,
        span: Span,
        kind: PropertyKind,
        key: PropertyKey<'a>,
        value: Expression<'a>,
        init: Option<Expression<'a>>,
        method: bool,
        shorthand: bool,
        computed: bool,
    ) -> ObjectProperty<'a> {
        ObjectProperty {
            span,
            kind,
            key,
            value,
            init,
            method,
            shorthand,
            computed,
        }
    }
    fn template_literal(
        self,
        span: Span,
        quasis: Vec<'a, TemplateElement<'a>>,
        expressions: Vec<'a, Expression<'a>>,
    ) -> TemplateLiteral<'a> {
        TemplateLiteral {
            span,
            quasis,
            expressions,
        }
    }
    fn tagged_template_expression(
        self,
        span: Span,
        tag: Expression<'a>,
        quasi: TemplateLiteral<'a>,
        type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
    ) -> TaggedTemplateExpression<'a> {
        TaggedTemplateExpression {
            span,
            tag,
            quasi,
            type_parameters,
        }
    }
    fn template_element(
        self,
        span: Span,
        tail: bool,
        value: TemplateElementValue<'a>,
    ) -> TemplateElement<'a> {
        TemplateElement {
            span,
            tail,
            value,
        }
    }
    fn template_element_value(
        self,
        raw: Atom<'a>,
        cooked: Option<Atom<'a>>,
    ) -> TemplateElementValue<'a> {
        TemplateElementValue {
            raw,
            cooked,
        }
    }
    fn computed_member_expression(
        self,
        span: Span,
        object: Expression<'a>,
        expression: Expression<'a>,
        optional: bool,
    ) -> ComputedMemberExpression<'a> {
        ComputedMemberExpression {
            span,
            object,
            expression,
            optional,
        }
    }
    fn static_member_expression(
        self,
        span: Span,
        object: Expression<'a>,
        property: IdentifierName<'a>,
        optional: bool,
    ) -> StaticMemberExpression<'a> {
        StaticMemberExpression {
            span,
            object,
            property,
            optional,
        }
    }
    fn private_field_expression(
        self,
        span: Span,
        object: Expression<'a>,
        field: PrivateIdentifier<'a>,
        optional: bool,
    ) -> PrivateFieldExpression<'a> {
        PrivateFieldExpression {
            span,
            object,
            field,
            optional,
        }
    }
    fn call_expression(
        self,
        span: Span,
        arguments: Vec<'a, Argument<'a>>,
        callee: Expression<'a>,
        type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
    ) -> CallExpression<'a> {
        CallExpression {
            span,
            callee,
            arguments,
            optional,
            type_parameters,
        }
    }
    fn new_expression(
        self,
        span: Span,
        callee: Expression<'a>,
        arguments: Vec<'a, Argument<'a>>,
        type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
    ) -> NewExpression<'a> {
        NewExpression {
            span,
            callee,
            arguments,
            type_parameters,
        }
    }
    fn meta_property(
        self,
        span: Span,
        meta: IdentifierName<'a>,
        property: IdentifierName<'a>,
    ) -> MetaProperty<'a> {
        MetaProperty {
            span,
            meta,
            property,
        }
    }
    fn spread_element(self, span: Span, argument: Expression<'a>) -> SpreadElement<'a> {
        SpreadElement { span, argument }
    }
    fn update_expression(
        self,
        span: Span,
        operator: UpdateOperator,
        prefix: bool,
        argument: SimpleAssignmentTarget<'a>,
    ) -> UpdateExpression<'a> {
        UpdateExpression {
            span,
            operator,
            prefix,
            argument,
        }
    }
    fn unary_expression(
        self,
        span: Span,
        operator: UnaryOperator,
        argument: Expression<'a>,
    ) -> UnaryExpression<'a> {
        UnaryExpression {
            span,
            operator,
            argument,
        }
    }
    fn binary_expression(
        self,
        span: Span,
        left: Expression<'a>,
        operator: BinaryOperator,
        right: Expression<'a>,
    ) -> BinaryExpression<'a> {
        BinaryExpression {
            span,
            left,
            operator,
            right,
        }
    }
    fn private_in_expression(
        self,
        span: Span,
        left: PrivateIdentifier<'a>,
        operator: BinaryOperator,
        right: Expression<'a>,
    ) -> PrivateInExpression<'a> {
        PrivateInExpression {
            span,
            left,
            operator,
            right,
        }
    }
    fn logical_expression(
        self,
        span: Span,
        left: Expression<'a>,
        operator: LogicalOperator,
        right: Expression<'a>,
    ) -> LogicalExpression<'a> {
        LogicalExpression {
            span,
            left,
            operator,
            right,
        }
    }
    fn conditional_expression(
        self,
        span: Span,
        test: Expression<'a>,
        consequent: Expression<'a>,
        alternate: Expression<'a>,
    ) -> ConditionalExpression<'a> {
        ConditionalExpression {
            span,
            test,
            consequent,
            alternate,
        }
    }
    fn assignment_expression(
        self,
        span: Span,
        operator: AssignmentOperator,
        left: AssignmentTarget<'a>,
        right: Expression<'a>,
    ) -> AssignmentExpression<'a> {
        AssignmentExpression {
            span,
            operator,
            left,
            right,
        }
    }
    fn array_assignment_target(
        self,
        span: Span,
        elements: Vec<'a, Option<AssignmentTargetMaybeDefault<'a>>>,
        rest: Option<AssignmentTargetRest<'a>>,
        trailing_comma: Option<Span>,
    ) -> ArrayAssignmentTarget<'a> {
        ArrayAssignmentTarget {
            span,
            elements,
            rest,
            trailing_comma,
        }
    }
    fn object_assignment_target(
        self,
        span: Span,
        properties: Vec<'a, AssignmentTargetProperty<'a>>,
        rest: Option<AssignmentTargetRest<'a>>,
    ) -> ObjectAssignmentTarget<'a> {
        ObjectAssignmentTarget {
            span,
            properties,
            rest,
        }
    }
    fn assignment_target_rest(
        self,
        span: Span,
        target: AssignmentTarget<'a>,
    ) -> AssignmentTargetRest<'a> {
        AssignmentTargetRest {
            span,
            target,
        }
    }
    fn assignment_target_with_default(
        self,
        span: Span,
        binding: AssignmentTarget<'a>,
        init: Expression<'a>,
    ) -> AssignmentTargetWithDefault<'a> {
        AssignmentTargetWithDefault {
            span,
            binding,
            init,
        }
    }
    fn assignment_target_property_identifier(
        self,
        span: Span,
        binding: IdentifierReference<'a>,
        init: Option<Expression<'a>>,
    ) -> AssignmentTargetPropertyIdentifier<'a> {
        AssignmentTargetPropertyIdentifier {
            span,
            binding,
            init,
        }
    }
    fn assignment_target_property_property(
        self,
        span: Span,
        name: PropertyKey<'a>,
        binding: AssignmentTargetMaybeDefault<'a>,
    ) -> AssignmentTargetPropertyProperty<'a> {
        AssignmentTargetPropertyProperty {
            span,
            name,
            binding,
        }
    }
    fn sequence_expression(
        self,
        span: Span,
        expressions: Vec<'a, Expression<'a>>,
    ) -> SequenceExpression<'a> {
        SequenceExpression {
            span,
            expressions,
        }
    }
    fn super_(self, span: Span) -> Super {
        Super { span }
    }
    fn await_expression(
        self,
        span: Span,
        argument: Expression<'a>,
    ) -> AwaitExpression<'a> {
        AwaitExpression { span, argument }
    }
    fn chain_expression(
        self,
        span: Span,
        expression: ChainElement<'a>,
    ) -> ChainExpression<'a> {
        ChainExpression {
            span,
            expression,
        }
    }
    fn parenthesized_expression(
        self,
        span: Span,
        expression: Expression<'a>,
    ) -> ParenthesizedExpression<'a> {
        ParenthesizedExpression {
            span,
            expression,
        }
    }
    fn directive(
        self,
        span: Span,
        expression: StringLiteral<'a>,
        directive: Atom<'a>,
    ) -> Directive<'a> {
        Directive {
            span,
            expression,
            directive,
        }
    }
    fn hashbang(self, span: Span, value: Atom<'a>) -> Hashbang<'a> {
        Hashbang { span, value }
    }
    fn block_statement(
        self,
        span: Span,
        body: Vec<'a, Statement<'a>>,
    ) -> BlockStatement<'a> {
        BlockStatement {
            span,
            body,
            scope_id: Default::default(),
        }
    }
    fn variable_declaration(
        self,
        span: Span,
        kind: VariableDeclarationKind,
        declarations: Vec<'a, VariableDeclarator<'a>>,
        declare: bool,
    ) -> VariableDeclaration<'a> {
        VariableDeclaration {
            span,
            kind,
            declarations,
            declare,
        }
    }
    fn variable_declarator(
        self,
        span: Span,
        kind: VariableDeclarationKind,
        id: BindingPattern<'a>,
        init: Option<Expression<'a>>,
        definite: bool,
    ) -> VariableDeclarator<'a> {
        VariableDeclarator {
            span,
            kind,
            id,
            init,
            definite,
        }
    }
    fn using_declaration(
        self,
        span: Span,
        is_await: bool,
        declarations: Vec<'a, VariableDeclarator<'a>>,
    ) -> UsingDeclaration<'a> {
        UsingDeclaration {
            span,
            is_await,
            declarations,
        }
    }
    fn empty_statement(self, span: Span) -> EmptyStatement {
        EmptyStatement { span }
    }
    fn expression_statement(
        self,
        span: Span,
        expression: Expression<'a>,
    ) -> ExpressionStatement<'a> {
        ExpressionStatement {
            span,
            expression,
        }
    }
    fn if_statement(
        self,
        span: Span,
        test: Expression<'a>,
        consequent: Statement<'a>,
        alternate: Option<Statement<'a>>,
    ) -> IfStatement<'a> {
        IfStatement {
            span,
            test,
            consequent,
            alternate,
        }
    }
    fn do_while_statement(
        self,
        span: Span,
        body: Statement<'a>,
        test: Expression<'a>,
    ) -> DoWhileStatement<'a> {
        DoWhileStatement {
            span,
            body,
            test,
        }
    }
    fn while_statement(
        self,
        span: Span,
        test: Expression<'a>,
        body: Statement<'a>,
    ) -> WhileStatement<'a> {
        WhileStatement { span, test, body }
    }
    fn for_statement(
        self,
        span: Span,
        init: Option<ForStatementInit<'a>>,
        test: Option<Expression<'a>>,
        update: Option<Expression<'a>>,
        body: Statement<'a>,
    ) -> ForStatement<'a> {
        ForStatement {
            span,
            init,
            test,
            update,
            body,
            scope_id: Default::default(),
        }
    }
    fn for_in_statement(
        self,
        span: Span,
        left: ForStatementLeft<'a>,
        right: Expression<'a>,
        body: Statement<'a>,
    ) -> ForInStatement<'a> {
        ForInStatement {
            span,
            left,
            right,
            body,
            scope_id: Default::default(),
        }
    }
    fn for_of_statement(
        self,
        span: Span,
        r#await: bool,
        left: ForStatementLeft<'a>,
        right: Expression<'a>,
        body: Statement<'a>,
    ) -> ForOfStatement<'a> {
        ForOfStatement {
            span,
            r#await,
            left,
            right,
            body,
            scope_id: Default::default(),
        }
    }
    fn continue_statement(
        self,
        span: Span,
        label: Option<LabelIdentifier<'a>>,
    ) -> ContinueStatement<'a> {
        ContinueStatement { span, label }
    }
    fn break_statement(
        self,
        span: Span,
        label: Option<LabelIdentifier<'a>>,
    ) -> BreakStatement<'a> {
        BreakStatement { span, label }
    }
    fn return_statement(
        self,
        span: Span,
        argument: Option<Expression<'a>>,
    ) -> ReturnStatement<'a> {
        ReturnStatement { span, argument }
    }
    fn with_statement(
        self,
        span: Span,
        object: Expression<'a>,
        body: Statement<'a>,
    ) -> WithStatement<'a> {
        WithStatement {
            span,
            object,
            body,
        }
    }
    fn switch_statement(
        self,
        span: Span,
        discriminant: Expression<'a>,
        cases: Vec<'a, SwitchCase<'a>>,
    ) -> SwitchStatement<'a> {
        SwitchStatement {
            span,
            discriminant,
            cases,
            scope_id: Default::default(),
        }
    }
    fn switch_case(
        self,
        span: Span,
        test: Option<Expression<'a>>,
        consequent: Vec<'a, Statement<'a>>,
    ) -> SwitchCase<'a> {
        SwitchCase {
            span,
            test,
            consequent,
        }
    }
    fn labeled_statement(
        self,
        span: Span,
        label: LabelIdentifier<'a>,
        body: Statement<'a>,
    ) -> LabeledStatement<'a> {
        LabeledStatement {
            span,
            label,
            body,
        }
    }
    fn throw_statement(
        self,
        span: Span,
        argument: Expression<'a>,
    ) -> ThrowStatement<'a> {
        ThrowStatement { span, argument }
    }
    fn try_statement(
        self,
        span: Span,
        block: Box<'a, BlockStatement<'a>>,
        handler: Option<Box<'a, CatchClause<'a>>>,
        finalizer: Option<Box<'a, BlockStatement<'a>>>,
    ) -> TryStatement<'a> {
        TryStatement {
            span,
            block,
            handler,
            finalizer,
        }
    }
    fn catch_clause(
        self,
        span: Span,
        param: Option<CatchParameter<'a>>,
        body: Box<'a, BlockStatement<'a>>,
    ) -> CatchClause<'a> {
        CatchClause {
            span,
            param,
            body,
            scope_id: Default::default(),
        }
    }
    fn catch_parameter(
        self,
        span: Span,
        pattern: BindingPattern<'a>,
    ) -> CatchParameter<'a> {
        CatchParameter { span, pattern }
    }
    fn debugger_statement(self, span: Span) -> DebuggerStatement {
        DebuggerStatement { span }
    }
    fn binding_pattern(
        self,
        kind: BindingPatternKind<'a>,
        type_annotation: Option<Box<'a, TSTypeAnnotation<'a>>>,
        optional: bool,
    ) -> BindingPattern<'a> {
        BindingPattern {
            kind,
            type_annotation,
            optional,
        }
    }
    fn assignment_pattern(
        self,
        span: Span,
        left: BindingPattern<'a>,
        right: Expression<'a>,
    ) -> AssignmentPattern<'a> {
        AssignmentPattern {
            span,
            left,
            right,
        }
    }
    fn object_pattern(
        self,
        span: Span,
        properties: Vec<'a, BindingProperty<'a>>,
        rest: Option<Box<'a, BindingRestElement<'a>>>,
    ) -> ObjectPattern<'a> {
        ObjectPattern {
            span,
            properties,
            rest,
        }
    }
    fn binding_property(
        self,
        span: Span,
        key: PropertyKey<'a>,
        value: BindingPattern<'a>,
        shorthand: bool,
        computed: bool,
    ) -> BindingProperty<'a> {
        BindingProperty {
            span,
            key,
            value,
            shorthand,
            computed,
        }
    }
    fn array_pattern(
        self,
        span: Span,
        elements: Vec<'a, Option<BindingPattern<'a>>>,
        rest: Option<Box<'a, BindingRestElement<'a>>>,
    ) -> ArrayPattern<'a> {
        ArrayPattern {
            span,
            elements,
            rest,
        }
    }
    fn binding_rest_element(
        self,
        span: Span,
        argument: BindingPattern<'a>,
    ) -> BindingRestElement<'a> {
        BindingRestElement {
            span,
            argument,
        }
    }
    fn function(
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
    ) -> Function<'a> {
        Function {
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
        }
    }
    fn formal_parameters(
        self,
        span: Span,
        kind: FormalParameterKind,
        items: Vec<'a, FormalParameter<'a>>,
        rest: Option<Box<'a, BindingRestElement<'a>>>,
    ) -> FormalParameters<'a> {
        FormalParameters {
            span,
            kind,
            items,
            rest,
        }
    }
    fn formal_parameter(
        self,
        span: Span,
        pattern: BindingPattern<'a>,
        accessibility: Option<TSAccessibility>,
        readonly: bool,
        r#override: bool,
        decorators: Vec<'a, Decorator<'a>>,
    ) -> FormalParameter<'a> {
        FormalParameter {
            span,
            pattern,
            accessibility,
            readonly,
            r#override,
            decorators,
        }
    }
    fn function_body(
        self,
        span: Span,
        directives: Vec<'a, Directive<'a>>,
        statements: Vec<'a, Statement<'a>>,
    ) -> FunctionBody<'a> {
        FunctionBody {
            span,
            directives,
            statements,
        }
    }
    fn arrow_function_expression(
        self,
        span: Span,
        expression: bool,
        r#async: bool,
        params: Box<'a, FormalParameters<'a>>,
        body: Box<'a, FunctionBody<'a>>,
        type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
        return_type: Option<Box<'a, TSTypeAnnotation<'a>>>,
    ) -> ArrowFunctionExpression<'a> {
        ArrowFunctionExpression {
            span,
            expression,
            r#async,
            params,
            body,
            type_parameters,
            return_type,
            scope_id: Default::default(),
        }
    }
    fn yield_expression(
        self,
        span: Span,
        delegate: bool,
        argument: Option<Expression<'a>>,
    ) -> YieldExpression<'a> {
        YieldExpression {
            span,
            delegate,
            argument,
        }
    }
    fn class(
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
    ) -> Class<'a> {
        Class {
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
        }
    }
    fn class_body(self, span: Span, body: Vec<'a, ClassElement<'a>>) -> ClassBody<'a> {
        ClassBody { span, body }
    }
    fn method_definition(
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
    ) -> MethodDefinition<'a> {
        MethodDefinition {
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
        }
    }
    fn property_definition(
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
        decorators: Vec<'a, Decorator<'a>>,
    ) -> PropertyDefinition<'a> {
        PropertyDefinition {
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
        }
    }
    fn private_identifier(self, span: Span, name: Atom<'a>) -> PrivateIdentifier<'a> {
        PrivateIdentifier { span, name }
    }
    fn static_block(self, span: Span, body: Vec<'a, Statement<'a>>) -> StaticBlock<'a> {
        StaticBlock {
            span,
            body,
            scope_id: Default::default(),
        }
    }
    fn accessor_property(
        self,
        r#type: AccessorPropertyType,
        span: Span,
        key: PropertyKey<'a>,
        value: Option<Expression<'a>>,
        computed: bool,
        r#static: bool,
        decorators: Vec<'a, Decorator<'a>>,
    ) -> AccessorProperty<'a> {
        AccessorProperty {
            r#type,
            span,
            key,
            value,
            computed,
            r#static,
            decorators,
        }
    }
    fn import_expression(
        self,
        span: Span,
        source: Expression<'a>,
        arguments: Vec<'a, Expression<'a>>,
    ) -> ImportExpression<'a> {
        ImportExpression {
            span,
            source,
            arguments,
        }
    }
    fn import_declaration(
        self,
        span: Span,
        specifiers: Option<Vec<'a, ImportDeclarationSpecifier<'a>>>,
        source: StringLiteral<'a>,
        with_clause: Option<WithClause<'a>>,
        import_kind: ImportOrExportKind,
    ) -> ImportDeclaration<'a> {
        ImportDeclaration {
            span,
            specifiers,
            source,
            with_clause,
            import_kind,
        }
    }
    fn import_specifier(
        self,
        span: Span,
        imported: ModuleExportName<'a>,
        local: BindingIdentifier<'a>,
        import_kind: ImportOrExportKind,
    ) -> ImportSpecifier<'a> {
        ImportSpecifier {
            span,
            imported,
            local,
            import_kind,
        }
    }
    fn import_default_specifier(
        self,
        span: Span,
        local: BindingIdentifier<'a>,
    ) -> ImportDefaultSpecifier<'a> {
        ImportDefaultSpecifier {
            span,
            local,
        }
    }
    fn import_namespace_specifier(
        self,
        span: Span,
        local: BindingIdentifier<'a>,
    ) -> ImportNamespaceSpecifier<'a> {
        ImportNamespaceSpecifier {
            span,
            local,
        }
    }
    fn with_clause(
        self,
        span: Span,
        attributes_keyword: IdentifierName<'a>,
        with_entries: Vec<'a, ImportAttribute<'a>>,
    ) -> WithClause<'a> {
        WithClause {
            span,
            attributes_keyword,
            with_entries,
        }
    }
    fn import_attribute(
        self,
        span: Span,
        key: ImportAttributeKey<'a>,
        value: StringLiteral<'a>,
    ) -> ImportAttribute<'a> {
        ImportAttribute {
            span,
            key,
            value,
        }
    }
    fn export_named_declaration(
        self,
        span: Span,
        declaration: Option<Declaration<'a>>,
        specifiers: Vec<'a, ExportSpecifier<'a>>,
        source: Option<StringLiteral<'a>>,
        export_kind: ImportOrExportKind,
        with_clause: Option<WithClause<'a>>,
    ) -> ExportNamedDeclaration<'a> {
        ExportNamedDeclaration {
            span,
            declaration,
            specifiers,
            source,
            export_kind,
            with_clause,
        }
    }
    fn export_default_declaration(
        self,
        span: Span,
        declaration: ExportDefaultDeclarationKind<'a>,
        exported: ModuleExportName<'a>,
    ) -> ExportDefaultDeclaration<'a> {
        ExportDefaultDeclaration {
            span,
            declaration,
            exported,
        }
    }
    fn export_all_declaration(
        self,
        span: Span,
        exported: Option<ModuleExportName<'a>>,
        source: StringLiteral<'a>,
        with_clause: Option<WithClause<'a>>,
        export_kind: ImportOrExportKind,
    ) -> ExportAllDeclaration<'a> {
        ExportAllDeclaration {
            span,
            exported,
            source,
            with_clause,
            export_kind,
        }
    }
    fn export_specifier(
        self,
        span: Span,
        local: ModuleExportName<'a>,
        exported: ModuleExportName<'a>,
        export_kind: ImportOrExportKind,
    ) -> ExportSpecifier<'a> {
        ExportSpecifier {
            span,
            local,
            exported,
            export_kind,
        }
    }
    fn ts_this_parameter(
        self,
        span: Span,
        this: IdentifierName<'a>,
        type_annotation: Option<Box<'a, TSTypeAnnotation<'a>>>,
    ) -> TSThisParameter<'a> {
        TSThisParameter {
            span,
            this,
            type_annotation,
        }
    }
    fn ts_enum_declaration(
        self,
        span: Span,
        id: BindingIdentifier<'a>,
        members: Vec<'a, TSEnumMember<'a>>,
        r#const: bool,
        declare: bool,
    ) -> TSEnumDeclaration<'a> {
        TSEnumDeclaration {
            span,
            id,
            members,
            r#const,
            declare,
            scope_id: Default::default(),
        }
    }
    fn ts_enum_member(
        self,
        span: Span,
        id: TSEnumMemberName<'a>,
        initializer: Option<Expression<'a>>,
    ) -> TSEnumMember<'a> {
        TSEnumMember {
            span,
            id,
            initializer,
        }
    }
    fn ts_type_annotation(
        self,
        span: Span,
        type_annotation: TSType<'a>,
    ) -> TSTypeAnnotation<'a> {
        TSTypeAnnotation {
            span,
            type_annotation,
        }
    }
    fn ts_literal_type(self, span: Span, literal: TSLiteral<'a>) -> TSLiteralType<'a> {
        TSLiteralType { span, literal }
    }
    fn ts_conditional_type(
        self,
        span: Span,
        check_type: TSType<'a>,
        extends_type: TSType<'a>,
        true_type: TSType<'a>,
        false_type: TSType<'a>,
    ) -> TSConditionalType<'a> {
        TSConditionalType {
            span,
            check_type,
            extends_type,
            true_type,
            false_type,
        }
    }
    fn ts_union_type(self, span: Span, types: Vec<'a, TSType<'a>>) -> TSUnionType<'a> {
        TSUnionType { span, types }
    }
    fn ts_intersection_type(
        self,
        span: Span,
        types: Vec<'a, TSType<'a>>,
    ) -> TSIntersectionType<'a> {
        TSIntersectionType { span, types }
    }
    fn ts_parenthesized_type(self, span: Span, type_annotation: TSType<'a>) -> TSParenthesizedType {
        TSParenthesizedType { span, type_annotation }
    }
    fn ts_type_operator(
        self,
        span: Span,
        operator: TSTypeOperatorOperator,
        type_annotation: TSType<'a>,
    ) -> TSTypeOperator<'a> {
        TSTypeOperator {
            span,
            operator,
            type_annotation,
        }
    }
    fn ts_array_type(self, span: Span, element_type: TSType<'a>) -> TSArrayType<'a> {
        TSArrayType { span, element_type }
    }
    fn ts_indexed_access_type(
        self,
        span: Span,
        object_type: TSType<'a>,
        index_type: TSType<'a>,
    ) -> TSIndexedAccessType<'a> {
        TSIndexedAccessType {
            span,
            object_type,
            index_type,
        }
    }
    fn ts_tuple_type(
        self,
        span: Span,
        element_types: Vec<'a, TSTupleElement<'a>>,
    ) -> TSTupleType<'a> {
        TSTupleType { span, element_types }
    }
    fn ts_named_tuple_member(
        self,
        span: Span,
        element_type: TSTupleElement<'a>,
        label: IdentifierName<'a>,
        optional: bool,
    ) -> TSNamedTupleMember<'a> {
        TSNamedTupleMember {
            span,
            element_type,
            label,
            optional,
        }
    }
    fn ts_optional_type(
        self,
        span: Span,
        type_annotation: TSType<'a>,
    ) -> TSOptionalType<'a> {
        TSOptionalType {
            span,
            type_annotation,
        }
    }
    fn ts_rest_type(self, span: Span, type_annotation: TSType<'a>) -> TSRestType<'a> {
        TSRestType {
            span,
            type_annotation,
        }
    }
    fn ts_any_keyword(self, span: Span) -> TSAnyKeyword {
        TSAnyKeyword { span }
    }
    fn ts_string_keyword(self, span: Span) -> TSStringKeyword {
        TSStringKeyword { span }
    }
    fn ts_boolean_keyword(self, span: Span) -> TSBooleanKeyword {
        TSBooleanKeyword { span }
    }
    fn ts_number_keyword(self, span: Span) -> TSNumberKeyword {
        TSNumberKeyword { span }
    }
    fn ts_never_keyword(self, span: Span) -> TSNeverKeyword {
        TSNeverKeyword { span }
    }
    fn ts_intrinsic_keyword(self, span: Span) -> TSIntrinsicKeyword {
        TSIntrinsicKeyword { span }
    }
    fn ts_unknown_keyword(self, span: Span) -> TSUnknownKeyword {
        TSUnknownKeyword { span }
    }
    fn ts_null_keyword(self, span: Span) -> TSNullKeyword {
        TSNullKeyword { span }
    }
    fn ts_undefined_keyword(self, span: Span) -> TSUndefinedKeyword {
        TSUndefinedKeyword { span }
    }
    fn ts_void_keyword(self, span: Span) -> TSVoidKeyword {
        TSVoidKeyword { span }
    }
    fn ts_symbol_keyword(self, span: Span) -> TSSymbolKeyword {
        TSSymbolKeyword { span }
    }
    fn ts_this_type(self, span: Span) -> TSThisType {
        TSThisType { span }
    }
    fn ts_object_keyword(self, span: Span) -> TSObjectKeyword {
        TSObjectKeyword { span }
    }
    fn ts_big_int_keyword(self, span: Span) -> TSBigIntKeyword {
        TSBigIntKeyword { span }
    }
    fn ts_type_reference(
        self,
        span: Span,
        type_name: TSTypeName<'a>,
        type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
    ) -> TSTypeReference<'a> {
        TSTypeReference {
            span,
            type_name,
            type_parameters,
        }
    }
    fn ts_qualified_name(
        self,
        span: Span,
        left: TSTypeName<'a>,
        right: IdentifierName<'a>,
    ) -> TSQualifiedName<'a> {
        TSQualifiedName {
            span,
            left,
            right,
        }
    }
    fn ts_type_parameter_instantiation(
        self,
        span: Span,
        params: Vec<'a, TSType<'a>>,
    ) -> TSTypeParameterInstantiation<'a> {
        TSTypeParameterInstantiation {
            span,
            params,
        }
    }
    fn ts_type_parameter(
        self,
        span: Span,
        name: BindingIdentifier<'a>,
        constraint: Option<TSType<'a>>,
        default: Option<TSType<'a>>,
        r#in: bool,
        out: bool,
        r#const: bool,
    ) -> TSTypeParameter<'a> {
        TSTypeParameter {
            span,
            name,
            constraint,
            default,
            r#in,
            out,
            r#const,
            scope_id: Default::default(),
        }
    }
    fn ts_type_parameter_declaration(
        self,
        span: Span,
        params: Vec<'a, TSTypeParameter<'a>>,
    ) -> TSTypeParameterDeclaration<'a> {
        TSTypeParameterDeclaration {
            span,
            params,
        }
    }
    fn ts_type_alias_declaration(
        self,
        span: Span,
        id: BindingIdentifier<'a>,
        type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
        type_annotation: TSType<'a>,
        declare: bool,
    ) -> TSTypeAliasDeclaration<'a> {
        TSTypeAliasDeclaration {
            span,
            id,
            type_annotation,
            type_parameters,
            declare,
        }
    }
    fn ts_class_implements(
        self,
        span: Span,
        expression: TSTypeName<'a>,
        type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
    ) -> TSClassImplements<'a> {
        TSClassImplements {
            span,
            expression,
            type_parameters,
        }
    }
    fn ts_interface_declaration(
        self,
        span: Span,
        id: BindingIdentifier<'a>,
        extends: Option<Vec<'a, TSInterfaceHeritage<'a>>>,
        type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
        body: Box<'a, TSInterfaceBody<'a>>,
        declare: bool,
    ) -> TSInterfaceDeclaration<'a> {
        TSInterfaceDeclaration {
            span,
            id,
            body,
            type_parameters,
            extends,
            declare,
        }
    }
    fn ts_interface_body(
        self,
        span: Span,
        body: Vec<'a, TSSignature<'a>>,
    ) -> TSInterfaceBody<'a> {
        TSInterfaceBody { span, body }
    }
    fn ts_property_signature(
        self,
        span: Span,
        computed: bool,
        optional: bool,
        readonly: bool,
        key: PropertyKey<'a>,
        type_annotation: Option<Box<'a, TSTypeAnnotation<'a>>>,
    ) -> TSPropertySignature<'a> {
        TSPropertySignature {
            span,
            computed,
            optional,
            readonly,
            key,
            type_annotation,
        }
    }
    fn ts_index_signature(
        self,
        span: Span,
        parameters: Vec<'a, TSIndexSignatureName<'a>>,
        type_annotation: Box<'a, TSTypeAnnotation<'a>>,
        readonly: bool,
    ) -> TSIndexSignature<'a> {
        TSIndexSignature {
            span,
            parameters,
            type_annotation,
            readonly,
        }
    }
    fn ts_call_signature_declaration(
        self,
        span: Span,
        this_param: Option<TSThisParameter<'a>>,
        params: Box<'a, FormalParameters<'a>>,
        return_type: Option<Box<'a, TSTypeAnnotation<'a>>>,
        type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
    ) -> TSCallSignatureDeclaration<'a> {
        TSCallSignatureDeclaration {
            span,
            this_param,
            params,
            return_type,
            type_parameters,
        }
    }
    fn ts_method_signature(
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
    ) -> TSMethodSignature<'a> {
        TSMethodSignature {
            span,
            key,
            computed,
            optional,
            kind,
            this_param,
            params,
            return_type,
            type_parameters,
        }
    }
    fn ts_construct_signature_declaration(
        self,
        span: Span,
        params: Box<'a, FormalParameters<'a>>,
        return_type: Option<Box<'a, TSTypeAnnotation<'a>>>,
        type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
    ) -> TSConstructSignatureDeclaration<'a> {
        TSConstructSignatureDeclaration {
            span,
            params,
            return_type,
            type_parameters,
        }
    }
    fn ts_index_signature_name(
        self,
        span: Span,
        name: Atom<'a>,
        type_annotation: Box<'a, TSTypeAnnotation<'a>>,
    ) -> TSIndexSignatureName<'a> {
        TSIndexSignatureName {
            span,
            name,
            type_annotation,
        }
    }
    fn ts_interface_heritage(
        self,
        span: Span,
        expression: Expression<'a>,
        type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
    ) -> TSInterfaceHeritage<'a> {
        TSInterfaceHeritage {
            span,
            expression,
            type_parameters,
        }
    }
    fn ts_type_predicate(
        self,
        span: Span,
        parameter_name: TSTypePredicateName<'a>,
        asserts: bool,
        type_annotation: Option<Box<'a, TSTypeAnnotation<'a>>>,
    ) -> TSTypePredicate<'a> {
        TSTypePredicate {
            span,
            parameter_name,
            asserts,
            type_annotation,
        }
    }
    fn ts_module_declaration(
        self,
        span: Span,
        id: TSModuleDeclarationName<'a>,
        body: Option<TSModuleDeclarationBody<'a>>,
        kind: TSModuleDeclarationKind,
        declare: bool,
    ) -> TSModuleDeclaration<'a> {
        TSModuleDeclaration {
            span,
            id,
            body,
            kind,
            declare,
            scope_id: Default::default(),
        }
    }
    fn ts_module_block(
        self,
        span: Span,
        directives: Vec<'a, Directive<'a>>,
        body: Vec<'a, Statement<'a>>,
    ) -> TSModuleBlock<'a> {
        TSModuleBlock {
            span,
            directives,
            body,
        }
    }
    fn ts_type_literal(
        self,
        span: Span,
        members: Vec<'a, TSSignature<'a>>,
    ) -> TSTypeLiteral<'a> {
        TSTypeLiteral { span, members }
    }
    fn ts_infer_type(
        self,
        span: Span,
        type_parameter: Box<'a, TSTypeParameter<'a>>,
    ) -> TSInferType<'a> {
        TSInferType {
            span,
            type_parameter,
        }
    }
    fn ts_type_query(
        self,
        span: Span,
        expr_name: TSTypeQueryExprName<'a>,
        type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
    ) -> TSTypeQuery<'a> {
        TSTypeQuery {
            span,
            expr_name,
            type_parameters,
        }
    }
    fn ts_import_type(
        self,
        span: Span,
        is_type_of: bool,
        parameter: TSType<'a>,
        qualifier: Option<TSTypeName<'a>>,
        attributes: Option<TSImportAttributes<'a>>,
        type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
    ) -> TSImportType<'a> {
        TSImportType {
            span,
            argument,
            qualifier,
            attributes,
            type_parameters,
        }
    }
    fn ts_import_attributes(
        self,
        span: Span,
        elements: Vec<'a, TSImportAttribute<'a>>,
    ) -> TSImportAttributes<'a> {
        TSImportAttributes {
            span,
            elements,
        }
    }
    fn ts_import_attribute(
        self,
        span: Span,
        name: TSImportAttributeName<'a>,
        value: Expression<'a>,
    ) -> TSImportAttribute<'a> {
        TSImportAttribute {
            span,
            name,
            value,
        }
    }
    fn ts_function_type(
        self,
        span: Span,
        this_param: Option<TSThisParameter<'a>>,
        params: Box<'a, FormalParameters<'a>>,
        return_type: Box<'a, TSTypeAnnotation<'a>>,
        type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
    ) -> TSFunctionType<'a> {
        TSFunctionType {
            span,
            this_param,
            params,
            return_type,
            type_parameters,
        }
    }
    fn ts_constructor_type(
        self,
        span: Span,
        r#abstract: bool,
        params: Box<'a, FormalParameters<'a>>,
        return_type: Box<'a, TSTypeAnnotation<'a>>,
        type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
    ) -> TSConstructorType<'a> {
        TSConstructorType {
            span,
            r#abstract,
            params,
            return_type,
            type_parameters,
        }
    }
    fn ts_mapped_type(
        self,
        span: Span,
        type_parameter: Box<'a, TSTypeParameter<'a>>,
        name_type: Option<TSType<'a>>,
        type_annotation: Option<TSType<'a>>,
        optional: TSMappedTypeModifierOperator,
        readonly: TSMappedTypeModifierOperator,
    ) -> TSMappedType<'a> {
        TSMappedType {
            span,
            type_parameter,
            name_type,
            type_annotation,
            optional,
            readonly,
        }
    }
    fn ts_template_literal_type(
        self,
        span: Span,
        quasis: Vec<'a, TemplateElement<'a>>,
        types: Vec<'a, TSType<'a>>,
    ) -> TSTemplateLiteralType<'a> {
        TSTemplateLiteralType {
            span,
            quasis,
            types,
        }
    }
    fn ts_as_expression(
        self,
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
    ) -> TSAsExpression<'a> {
        TSAsExpression {
            span,
            expression,
            type_annotation,
        }
    }
    fn ts_satisfies_expression(
        self,
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
    ) -> TSSatisfiesExpression<'a> {
        TSSatisfiesExpression {
            span,
            expression,
            type_annotation,
        }
    }
    fn ts_type_assertion(
        self,
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
    ) -> TSTypeAssertion<'a> {
        TSTypeAssertion {
            span,
            expression,
            type_annotation,
        }
    }
    fn ts_import_equals_declaration(
        self,
        span: Span,
        id: BindingIdentifier<'a>,
        module_reference: TSModuleReference<'a>,
        import_kind: ImportOrExportKind,
    ) -> TSImportEqualsDeclaration<'a> {
        TSImportEqualsDeclaration {
            span,
            id,
            module_reference,
            import_kind,
        }
    }
    fn ts_external_module_reference(
        self,
        span: Span,
        expression: StringLiteral<'a>,
    ) -> TSExternalModuleReference<'a> {
        TSExternalModuleReference {
            span,
            expression,
        }
    }
    fn ts_non_null_expression(
        self,
        span: Span,
        expression: Expression<'a>,
    ) -> TSNonNullExpression<'a> {
        TSNonNullExpression {
            span,
            expression,
        }
    }
    fn decorator(self, span: Span, expression: Expression<'a>) -> Decorator<'a> {
        Decorator { span, expression }
    }
    fn ts_export_assignment(
        self,
        span: Span,
        expression: Expression<'a>,
    ) -> TSExportAssignment<'a> {
        TSExportAssignment {
            span,
            expression,
        }
    }
    fn ts_namespace_export_declaration(
        self,
        span: Span,
        id: IdentifierName<'a>,
    ) -> TSNamespaceExportDeclaration<'a> {
        TSNamespaceExportDeclaration {
            span,
            id,
        }
    }
    fn ts_instantiation_expression(
        self,
        span: Span,
        expression: Expression<'a>,
        type_parameters: Box<'a, TSTypeParameterInstantiation<'a>>,
    ) -> TSInstantiationExpression<'a> {
        TSInstantiationExpression {
            span,
            expression,
            type_parameters,
        }
    }
    fn js_doc_nullable_type(
        self,
        span: Span,
        type_annotation: TSType<'a>,
        postfix: bool,
    ) -> JSDocNullableType<'a> {
        JSDocNullableType {
            span,
            type_annotation,
            postfix,
        }
    }
    fn js_doc_unknown_type(self, span: Span) -> JSDocUnknownType {
        JSDocUnknownType { span }
    }
    fn jsx_element(
        self,
        span: Span,
        opening_element: Box<'a, JSXOpeningElement<'a>>,
        closing_element: Option<Box<'a, JSXClosingElement<'a>>>,
        children: Vec<'a, JSXChild<'a>>,
    ) -> JSXElement<'a> {
        JSXElement {
            span,
            opening_element,
            closing_element,
            children,
        }
    }
    fn jsx_opening_element(
        self,
        span: Span,
        self_closing: bool,
        name: JSXElementName<'a>,
        attributes: Vec<'a, JSXAttributeItem<'a>>,
        type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
    ) -> JSXOpeningElement<'a> {
        JSXOpeningElement {
            span,
            self_closing,
            name,
            attributes,
            type_parameters,
        }
    }
    fn jsx_closing_element(
        self,
        span: Span,
        name: JSXElementName<'a>,
    ) -> JSXClosingElement<'a> {
        JSXClosingElement { span, name }
    }
    fn jsx_fragment(
        self,
        span: Span,
        opening_fragment: JSXOpeningFragment,
        closing_fragment: JSXClosingFragment,
        children: Vec<'a, JSXChild<'a>>,
    ) -> JSXFragment<'a> {
        JSXFragment {
            span,
            opening_fragment,
            closing_fragment,
            children,
        }
    }
    fn jsx_opening_fragment(self, span: Span) -> JSXOpeningFragment {
        JSXOpeningFragment { span }
    }
    fn jsx_closing_fragment(self, span: Span) -> JSXClosingFragment {
        JSXClosingFragment { span }
    }
    fn jsx_namespaced_name(
        self,
        span: Span,
        namespace: JSXIdentifier<'a>,
        property: JSXIdentifier<'a>,
    ) -> JSXNamespacedName<'a> {
        JSXNamespacedName {
            span,
            namespace,
            property,
        }
    }
    fn jsx_member_expression(
        self,
        span: Span,
        object: JSXMemberExpressionObject<'a>,
        property: JSXIdentifier<'a>,
    ) -> JSXMemberExpression<'a> {
        JSXMemberExpression {
            span,
            object,
            property,
        }
    }
    fn jsx_expression_container(
        self,
        span: Span,
        expression: JSXExpression<'a>,
    ) -> JSXExpressionContainer<'a> {
        JSXExpressionContainer {
            span,
            expression,
        }
    }
    fn jsx_empty_expression(self, span: Span) -> JSXEmptyExpression {
        JSXEmptyExpression { span }
    }
    fn jsx_attribute(
        self,
        span: Span,
        name: JSXAttributeName<'a>,
        value: Option<JSXAttributeValue<'a>>,
    ) -> JSXAttribute<'a> {
        JSXAttribute { span, name, value }
    }
    fn jsx_spread_attribute(
        self,
        span: Span,
        argument: Expression<'a>,
    ) -> JSXSpreadAttribute<'a> {
        JSXSpreadAttribute {
            span,
            argument,
        }
    }
    fn jsx_identifier(self, span: Span, name: Atom<'a>) -> JSXIdentifier<'a> {
        JSXIdentifier { span, name }
    }
    fn jsx_spread_child(
        self,
        span: Span,
        expression: Expression<'a>,
    ) -> JSXSpreadChild<'a> {
        JSXSpreadChild { span, expression }
    }
    fn jsx_text(self, span: Span, value: Atom<'a>) -> JSXText<'a> {
        JSXText { span, value }
    }
}
