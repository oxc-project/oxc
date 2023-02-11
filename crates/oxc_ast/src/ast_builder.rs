//! AST builder for creating AST nodes

#![allow(clippy::unused_self, clippy::missing_const_for_fn, clippy::too_many_arguments)]

use oxc_allocator::{Allocator, Box, String, Vec};

#[allow(clippy::wildcard_imports)]
use crate::{ast::*, Atom, Node, SourceType};

pub struct AstBuilder<'a> {
    pub allocator: &'a Allocator,
}

impl<'a> AstBuilder<'a> {
    pub fn new(allocator: &'a Allocator) -> Self {
        Self { allocator }
    }

    pub fn alloc<T>(&self, value: T) -> Box<'a, T> {
        Box(self.allocator.alloc(value))
    }

    #[must_use]
    #[inline]
    pub fn new_vec<T>(&self) -> Vec<'a, T> {
        Vec::new_in(self.allocator)
    }

    #[must_use]
    pub fn new_vec_with_capacity<T>(&self, capacity: usize) -> Vec<'a, T> {
        Vec::with_capacity_in(capacity, self.allocator)
    }

    #[must_use]
    pub fn new_vec_single<T>(&self, value: T) -> Vec<'a, T> {
        let mut vec = self.new_vec_with_capacity(1);
        vec.push(value);
        vec
    }

    #[must_use]
    #[inline]
    pub fn new_str(&self, value: &str) -> &'a str {
        String::from_str_in(value, self.allocator).into_bump_str()
    }

    #[must_use]
    #[inline]
    pub fn program(
        &self,
        node: Node,
        directives: Vec<'a, Directive>,
        body: Vec<'a, Statement<'a>>,
        source_type: SourceType,
    ) -> Program<'a> {
        Program { node, directives, body, source_type }
    }

    /* ---------- Literals ---------- */

    #[must_use]
    #[inline]
    pub fn literal_string_expression(&self, literal: StringLiteral) -> Expression<'a> {
        Expression::StringLiteral(self.alloc(literal))
    }

    #[must_use]
    #[inline]
    pub fn literal_boolean_expression(&self, literal: BooleanLiteral) -> Expression<'a> {
        Expression::BooleanLiteral(self.alloc(literal))
    }

    #[must_use]
    #[inline]
    pub fn literal_null_expression(&self, literal: NullLiteral) -> Expression<'a> {
        Expression::NullLiteral(self.alloc(literal))
    }

    #[must_use]
    #[inline]
    pub fn literal_regexp_expression(&self, literal: RegExpLiteral) -> Expression<'a> {
        Expression::RegExpLiteral(self.alloc(literal))
    }

    #[must_use]
    #[inline]
    pub fn literal_number_expression(&self, literal: NumberLiteral<'a>) -> Expression<'a> {
        Expression::NumberLiteral(self.alloc(literal))
    }

    #[must_use]
    #[inline]
    pub fn literal_bigint_expression(&self, literal: BigintLiteral) -> Expression<'a> {
        Expression::BigintLiteral(self.alloc(literal))
    }

    /* ---------- Identifiers ---------- */

    #[must_use]
    #[inline]
    pub fn identifier_expression(&self, identifier: IdentifierReference) -> Expression<'a> {
        Expression::Identifier(self.alloc(identifier))
    }

    /* ---------- Statements ---------- */

    #[must_use]
    #[inline]
    pub fn directive(
        &self,
        node: Node,
        expression: StringLiteral,
        directive: &'a str,
    ) -> Directive<'a> {
        Directive { node, expression, directive }
    }

    #[must_use]
    #[inline]
    pub fn block(&self, node: Node, body: Vec<'a, Statement<'a>>) -> Box<'a, BlockStatement<'a>> {
        self.alloc(BlockStatement { node, body })
    }

    #[must_use]
    #[inline]
    pub fn block_statement(&self, block: Box<'a, BlockStatement<'a>>) -> Statement<'a> {
        Statement::BlockStatement(
            self.alloc(BlockStatement { node: block.node, body: block.unbox().body }),
        )
    }

    #[must_use]
    #[inline]
    pub fn break_statement(&self, node: Node, label: Option<LabelIdentifier>) -> Statement<'a> {
        Statement::BreakStatement(self.alloc(BreakStatement { node, label }))
    }

    #[must_use]
    #[inline]
    pub fn continue_statement(&self, node: Node, label: Option<LabelIdentifier>) -> Statement<'a> {
        Statement::ContinueStatement(self.alloc(ContinueStatement { node, label }))
    }

    #[must_use]
    #[inline]
    pub fn debugger_statement(&self, node: Node) -> Statement<'a> {
        Statement::DebuggerStatement(self.alloc(DebuggerStatement { node }))
    }

    #[must_use]
    #[inline]
    pub fn do_while_statement(
        &self,
        node: Node,
        body: Statement<'a>,
        test: Expression<'a>,
    ) -> Statement<'a> {
        Statement::DoWhileStatement(self.alloc(DoWhileStatement { node, body, test }))
    }

    #[must_use]
    #[inline]
    pub fn empty_statement(&self, node: Node) -> Statement<'a> {
        Statement::EmptyStatement(self.alloc(EmptyStatement { node }))
    }

    #[must_use]
    #[inline]
    pub fn expression_statement(&self, node: Node, expression: Expression<'a>) -> Statement<'a> {
        Statement::ExpressionStatement(self.alloc(ExpressionStatement { node, expression }))
    }

    #[must_use]
    #[inline]
    pub fn for_in_statement(
        &self,
        node: Node,
        left: ForStatementLeft<'a>,
        right: Expression<'a>,
        body: Statement<'a>,
    ) -> Statement<'a> {
        Statement::ForInStatement(self.alloc(ForInStatement { node, left, right, body }))
    }

    #[must_use]
    #[inline]
    pub fn for_of_statement(
        &self,
        node: Node,
        r#await: bool,
        left: ForStatementLeft<'a>,
        right: Expression<'a>,
        body: Statement<'a>,
    ) -> Statement<'a> {
        Statement::ForOfStatement(self.alloc(ForOfStatement { node, r#await, left, right, body }))
    }

    #[must_use]
    #[inline]
    pub fn for_statement(
        &self,
        node: Node,
        init: Option<ForStatementInit<'a>>,
        test: Option<Expression<'a>>,
        update: Option<Expression<'a>>,
        body: Statement<'a>,
    ) -> Statement<'a> {
        Statement::ForStatement(self.alloc(ForStatement { node, init, test, update, body }))
    }

    #[must_use]
    #[inline]
    pub fn if_statement(
        &self,
        node: Node,
        test: Expression<'a>,
        consequent: Statement<'a>,
        alternate: Option<Statement<'a>>,
    ) -> Statement<'a> {
        Statement::IfStatement(self.alloc(IfStatement { node, test, consequent, alternate }))
    }

    #[must_use]
    #[inline]
    pub fn labeled_statement(
        &self,
        node: Node,
        label: LabelIdentifier,
        body: Statement<'a>,
    ) -> Statement<'a> {
        Statement::LabeledStatement(self.alloc(LabeledStatement { node, label, body }))
    }

    #[must_use]
    #[inline]
    pub fn return_statement(&self, node: Node, argument: Option<Expression<'a>>) -> Statement<'a> {
        Statement::ReturnStatement(self.alloc(ReturnStatement { node, argument }))
    }

    #[must_use]
    #[inline]
    pub fn switch_statement(
        &self,
        node: Node,
        discriminant: Expression<'a>,
        cases: Vec<'a, SwitchCase<'a>>,
    ) -> Statement<'a> {
        Statement::SwitchStatement(self.alloc(SwitchStatement { node, discriminant, cases }))
    }

    #[must_use]
    #[inline]
    pub fn switch_case(
        &self,
        node: Node,
        test: Option<Expression<'a>>,
        consequent: Vec<'a, Statement<'a>>,
    ) -> SwitchCase<'a> {
        SwitchCase { node, test, consequent }
    }

    #[must_use]
    #[inline]
    pub fn throw_statement(&self, node: Node, argument: Expression<'a>) -> Statement<'a> {
        Statement::ThrowStatement(self.alloc(ThrowStatement { node, argument }))
    }

    #[must_use]
    #[inline]
    pub fn try_statement(
        &self,
        node: Node,
        block: Box<'a, BlockStatement<'a>>,
        handler: Option<Box<'a, CatchClause<'a>>>,
        finalizer: Option<Box<'a, BlockStatement<'a>>>,
    ) -> Statement<'a> {
        Statement::TryStatement(self.alloc(TryStatement { node, block, handler, finalizer }))
    }

    #[must_use]
    #[inline]
    pub fn catch_clause(
        &self,
        node: Node,
        param: Option<BindingPattern<'a>>,
        body: Box<'a, BlockStatement<'a>>,
    ) -> Box<'a, CatchClause<'a>> {
        self.alloc(CatchClause { node, param, body })
    }

    #[must_use]
    #[inline]
    pub fn while_statement(
        &self,
        node: Node,
        test: Expression<'a>,
        body: Statement<'a>,
    ) -> Statement<'a> {
        Statement::WhileStatement(self.alloc(WhileStatement { node, test, body }))
    }

    #[must_use]
    #[inline]
    pub fn with_statement(
        &self,
        node: Node,
        object: Expression<'a>,
        body: Statement<'a>,
    ) -> Statement<'a> {
        Statement::WithStatement(self.alloc(WithStatement { node, object, body }))
    }

    /* ---------- Expressions ---------- */

    #[must_use]
    #[inline]
    pub fn super_(&self, node: Node) -> Expression<'a> {
        Expression::Super(self.alloc(Super { node }))
    }

    #[must_use]
    #[inline]
    pub fn meta_property(
        &self,
        node: Node,
        meta: IdentifierName,
        property: IdentifierName,
    ) -> Expression<'a> {
        Expression::MetaProperty(self.alloc(MetaProperty { node, meta, property }))
    }

    #[must_use]
    #[inline]
    pub fn array_expression(
        &self,
        node: Node,
        elements: Vec<'a, Option<Argument<'a>>>,
        trailing_comma: Option<Node>,
    ) -> Expression<'a> {
        Expression::ArrayExpression(self.alloc(ArrayExpression { node, elements, trailing_comma }))
    }

    #[must_use]
    #[inline]
    pub fn arrow_expression(
        &self,
        node: Node,
        expression: bool,
        generator: bool,
        r#async: bool,
        params: FormalParameters<'a>,
        body: Box<'a, FunctionBody<'a>>,
        type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
        return_type: Option<TSTypeAnnotation<'a>>,
    ) -> Expression<'a> {
        Expression::ArrowFunctionExpression(self.alloc(ArrowExpression {
            node,
            expression,
            generator,
            r#async,
            params,
            body,
            type_parameters,
            return_type,
        }))
    }

    #[must_use]
    #[inline]
    pub fn assignment_expression(
        &self,
        node: Node,
        operator: AssignmentOperator,
        left: AssignmentTarget<'a>,
        right: Expression<'a>,
    ) -> Expression<'a> {
        Expression::AssignmentExpression(self.alloc(AssignmentExpression {
            node,
            operator,
            left,
            right,
        }))
    }

    #[must_use]
    #[inline]
    pub fn await_expression(&self, node: Node, argument: Expression<'a>) -> Expression<'a> {
        Expression::AwaitExpression(self.alloc(AwaitExpression { node, argument }))
    }

    #[must_use]
    #[inline]
    pub fn binary_expression(
        &self,
        node: Node,
        left: Expression<'a>,
        operator: BinaryOperator,
        right: Expression<'a>,
    ) -> Expression<'a> {
        Expression::BinaryExpression(self.alloc(BinaryExpression { node, left, operator, right }))
    }

    #[must_use]
    #[inline]
    pub fn call_expression(
        &self,
        node: Node,
        callee: Expression<'a>,
        arguments: Vec<'a, Argument<'a>>,
        optional: bool, // for optional chaining
        type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
    ) -> Expression<'a> {
        Expression::CallExpression(self.alloc(CallExpression {
            node,
            callee,
            arguments,
            optional,
            type_parameters,
        }))
    }

    #[must_use]
    #[inline]
    pub fn chain_expression(&self, node: Node, expression: ChainElement<'a>) -> Expression<'a> {
        Expression::ChainExpression(self.alloc(ChainExpression { node, expression }))
    }

    #[must_use]
    #[inline]
    pub fn class_expression(&self, class: Box<'a, Class<'a>>) -> Expression<'a> {
        Expression::ClassExpression(class)
    }

    #[must_use]
    #[inline]
    pub fn conditional_expression(
        &self,
        node: Node,
        test: Expression<'a>,
        consequent: Expression<'a>,
        alternate: Expression<'a>,
    ) -> Expression<'a> {
        Expression::ConditionalExpression(self.alloc(ConditionalExpression {
            node,
            test,
            consequent,
            alternate,
        }))
    }

    #[must_use]
    #[inline]
    pub fn function_expression(&self, function: Box<'a, Function<'a>>) -> Expression<'a> {
        Expression::FunctionExpression(function)
    }

    #[must_use]
    #[inline]
    pub fn import_expression(
        &self,
        node: Node,
        source: Expression<'a>,
        arguments: Vec<'a, Expression<'a>>,
    ) -> Expression<'a> {
        Expression::ImportExpression(self.alloc(ImportExpression { node, source, arguments }))
    }

    #[must_use]
    #[inline]
    pub fn logical_expression(
        &self,
        node: Node,
        left: Expression<'a>,
        operator: LogicalOperator,
        right: Expression<'a>,
    ) -> Expression<'a> {
        Expression::LogicalExpression(self.alloc(LogicalExpression { node, left, operator, right }))
    }

    #[must_use]
    #[inline]
    pub fn computed_member_expression(
        &self,
        node: Node,
        object: Expression<'a>,
        expression: Expression<'a>,
        optional: bool, // for optional chaining
    ) -> Expression<'a> {
        Expression::MemberExpression(self.alloc({
            MemberExpression::ComputedMemberExpression(ComputedMemberExpression {
                node,
                object,
                expression,
                optional,
            })
        }))
    }

    #[must_use]
    #[inline]
    pub fn static_member_expression(
        &self,
        node: Node,
        object: Expression<'a>,
        property: IdentifierName,
        optional: bool, // for optional chaining
    ) -> Expression<'a> {
        Expression::MemberExpression(self.alloc({
            MemberExpression::StaticMemberExpression(StaticMemberExpression {
                node,
                object,
                property,
                optional,
            })
        }))
    }

    #[must_use]
    #[inline]
    pub fn private_field_expression(
        &self,
        node: Node,
        object: Expression<'a>,
        field: PrivateIdentifier,
        optional: bool,
    ) -> Expression<'a> {
        Expression::MemberExpression(self.alloc({
            MemberExpression::PrivateFieldExpression(PrivateFieldExpression {
                node,
                object,
                field,
                optional,
            })
        }))
    }

    #[must_use]
    #[inline]
    pub fn new_expression(
        &self,
        node: Node,
        callee: Expression<'a>,
        arguments: Vec<'a, Argument<'a>>,
        type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
    ) -> Expression<'a> {
        Expression::NewExpression(self.alloc(NewExpression {
            node,
            callee,
            arguments,
            type_parameters,
        }))
    }

    #[must_use]
    #[inline]
    pub fn object_expression(
        &self,
        node: Node,
        properties: Vec<'a, ObjectProperty<'a>>,
        trailing_comma: Option<Node>,
    ) -> Expression<'a> {
        Expression::ObjectExpression(self.alloc(ObjectExpression {
            node,
            properties,
            trailing_comma,
        }))
    }

    #[must_use]
    #[inline]
    pub fn parenthesized_expression(
        &self,
        node: Node,
        expression: Expression<'a>,
    ) -> Expression<'a> {
        Expression::ParenthesizedExpression(
            self.alloc(ParenthesizedExpression { node, expression }),
        )
    }

    #[must_use]
    #[inline]
    pub fn sequence_expression(
        &self,
        node: Node,
        expressions: Vec<'a, Expression<'a>>,
    ) -> Expression<'a> {
        Expression::SequenceExpression(self.alloc(SequenceExpression { node, expressions }))
    }

    #[must_use]
    #[inline]
    pub fn tagged_template_expression(
        &self,
        node: Node,
        tag: Expression<'a>,
        quasi: TemplateLiteral<'a>,
        type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
    ) -> Expression<'a> {
        Expression::TaggedTemplateExpression(self.alloc(TaggedTemplateExpression {
            node,
            tag,
            quasi,
            type_parameters,
        }))
    }

    #[must_use]
    #[inline]
    pub fn template_literal_expression(
        &self,
        template_literal: TemplateLiteral<'a>,
    ) -> Expression<'a> {
        Expression::TemplateLiteral(self.alloc(template_literal))
    }

    #[must_use]
    #[inline]
    pub fn this_expression(&self, node: Node) -> Expression<'a> {
        Expression::ThisExpression(self.alloc(ThisExpression { node }))
    }

    #[must_use]
    #[inline]
    pub fn unary_expression(
        &self,
        node: Node,
        operator: UnaryOperator,
        prefix: bool,
        argument: Expression<'a>,
    ) -> Expression<'a> {
        Expression::UnaryExpression(self.alloc(UnaryExpression {
            node,
            operator,
            prefix,
            argument,
        }))
    }

    #[must_use]
    #[inline]
    pub fn update_expression(
        &self,
        node: Node,
        operator: UpdateOperator,
        prefix: bool,
        argument: SimpleAssignmentTarget<'a>,
    ) -> Expression<'a> {
        Expression::UpdateExpression(self.alloc(UpdateExpression {
            node,
            operator,
            prefix,
            argument,
        }))
    }

    #[must_use]
    #[inline]
    pub fn yield_expression(
        &self,
        node: Node,
        delegate: bool,
        argument: Option<Expression<'a>>,
    ) -> Expression<'a> {
        Expression::YieldExpression(self.alloc(YieldExpression { node, delegate, argument }))
    }

    /* ---------- Functions ---------- */
    #[must_use]
    #[inline]
    pub fn function_declaration(&self, func: Box<'a, Function<'a>>) -> Statement<'a> {
        Statement::Declaration(Declaration::FunctionDeclaration(func))
    }

    #[must_use]
    #[inline]
    pub fn formal_parameters(
        &self,
        node: Node,
        kind: FormalParameterKind,
        items: Vec<'a, FormalParameter<'a>>,
    ) -> FormalParameters<'a> {
        FormalParameters { node, kind, items }
    }

    #[must_use]
    #[inline]
    pub fn formal_parameter(
        &self,
        node: Node,
        pattern: BindingPattern<'a>,
        accessibility: Option<TSAccessibility>,
        readonly: bool,
        decorators: Option<Vec<'a, Decorator<'a>>>,
    ) -> FormalParameter<'a> {
        FormalParameter { node, pattern, accessibility, readonly, decorators }
    }

    #[must_use]
    #[inline]
    #[allow(clippy::fn_params_excessive_bools)]
    pub fn function(
        &self,
        r#type: FunctionType,
        node: Node,
        id: Option<BindingIdentifier>,
        expression: bool,
        generator: bool,
        r#async: bool,
        params: FormalParameters<'a>,
        body: Option<Box<'a, FunctionBody<'a>>>,
        type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
        return_type: Option<TSTypeAnnotation<'a>>,
        declare: bool,
    ) -> Box<'a, Function<'a>> {
        self.alloc(Function {
            r#type,
            node,
            id,
            expression,
            generator,
            r#async,
            params,
            body,
            declare,
            type_parameters,
            return_type,
        })
    }

    #[must_use]
    #[inline]
    pub fn function_body(
        &self,
        node: Node,
        directives: Vec<'a, Directive>,
        statements: Vec<'a, Statement<'a>>,
    ) -> Box<'a, FunctionBody<'a>> {
        self.alloc(FunctionBody { node, directives, statements })
    }

    /* ---------- Class ---------- */

    #[must_use]
    #[inline]
    pub fn class(
        &self,
        r#type: ClassType,
        node: Node,
        id: Option<BindingIdentifier>,
        super_class: Option<Expression<'a>>,
        body: ClassBody<'a>,
        type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
        super_type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
        implements: Option<Vec<'a, Box<'a, TSClassImplements<'a>>>>,
        r#abstract: bool,
        decorators: Option<Vec<'a, Decorator<'a>>>,
        declare: bool,
    ) -> Box<'a, Class<'a>> {
        self.alloc(Class {
            r#type,
            node,
            id,
            super_class,
            body,
            type_parameters,
            super_type_parameters,
            implements,
            r#abstract,
            decorators,
            declare,
        })
    }

    #[must_use]
    #[inline]
    pub fn class_declaration(&self, class: Box<'a, Class<'a>>) -> Statement<'a> {
        Statement::Declaration(Declaration::ClassDeclaration(class))
    }

    #[must_use]
    #[inline]
    pub fn static_block(&self, node: Node, body: Vec<'a, Statement<'a>>) -> ClassElement<'a> {
        ClassElement::StaticBlock(self.alloc(StaticBlock { node, body }))
    }

    #[must_use]
    #[inline]
    pub fn accessor_property(
        &self,
        node: Node,
        key: PropertyKey<'a>,
        value: Option<Expression<'a>>,
        computed: bool,
        r#static: bool,
    ) -> ClassElement<'a> {
        ClassElement::AccessorProperty(self.alloc(AccessorProperty {
            node,
            key,
            value,
            computed,
            r#static,
        }))
    }

    /* ---------- Declarations ---------- */

    #[must_use]
    #[inline]
    pub fn variable_declaration(
        &self,
        node: Node,
        kind: VariableDeclarationKind,
        declarations: Vec<'a, VariableDeclarator<'a>>,
    ) -> Box<'a, VariableDeclaration<'a>> {
        self.alloc(VariableDeclaration { node, kind, declarations })
    }

    #[must_use]
    #[inline]
    pub fn variable_declarator(
        &self,
        node: Node,
        kind: VariableDeclarationKind,
        id: BindingPattern<'a>,
        init: Option<Expression<'a>>,
        definite: bool,
    ) -> VariableDeclarator<'a> {
        VariableDeclarator { node, kind, id, init, definite }
    }

    /* ---------- Patterns ---------- */

    #[must_use]
    #[inline]
    pub fn binding_pattern(
        &self,
        kind: BindingPatternKind<'a>,
        type_annotation: Option<TSTypeAnnotation<'a>>,
        optional: bool,
    ) -> BindingPattern<'a> {
        BindingPattern { kind, type_annotation, optional }
    }

    #[must_use]
    #[inline]
    pub fn binding_identifier(&self, identifier: BindingIdentifier) -> BindingPatternKind<'a> {
        BindingPatternKind::BindingIdentifier(self.alloc(identifier))
    }

    #[must_use]
    #[inline]
    pub fn object_pattern(
        &self,
        node: Node,
        properties: Vec<'a, ObjectPatternProperty<'a>>,
    ) -> BindingPatternKind<'a> {
        BindingPatternKind::ObjectPattern(self.alloc(ObjectPattern { node, properties }))
    }

    #[must_use]
    #[inline]
    pub fn spread_element(
        &self,
        node: Node,
        argument: Expression<'a>,
    ) -> Box<'a, SpreadElement<'a>> {
        self.alloc(SpreadElement { node, argument })
    }

    #[must_use]
    #[inline]
    pub fn property(
        &self,
        node: Node,
        kind: PropertyKind,
        key: PropertyKey<'a>,
        value: PropertyValue<'a>,
        method: bool,
        shorthand: bool,
        computed: bool,
    ) -> Box<'a, Property<'a>> {
        self.alloc(Property { node, kind, key, value, method, shorthand, computed })
    }

    #[must_use]
    #[inline]
    pub fn array_pattern(
        &self,
        node: Node,
        elements: Vec<'a, Option<BindingPattern<'a>>>,
    ) -> BindingPatternKind<'a> {
        BindingPatternKind::ArrayPattern(self.alloc(ArrayPattern { node, elements }))
    }

    #[must_use]
    #[inline]
    pub fn assignment_pattern(
        &self,
        node: Node,
        left: BindingPattern<'a>,
        right: Expression<'a>,
    ) -> BindingPattern<'a> {
        let pattern = self.alloc(AssignmentPattern { node, left, right });
        BindingPattern {
            kind: BindingPatternKind::AssignmentPattern(pattern),
            type_annotation: None,
            optional: false,
        }
    }

    #[must_use]
    #[inline]
    pub fn rest_element(
        &self,
        node: Node,
        argument: BindingPattern<'a>,
    ) -> Box<'a, RestElement<'a>> {
        self.alloc(RestElement { node, argument })
    }

    #[must_use]
    #[inline]
    pub fn rest_element_pattern(&self, elem: Box<'a, RestElement<'a>>) -> BindingPattern<'a> {
        BindingPattern {
            kind: BindingPatternKind::RestElement(elem),
            type_annotation: None,
            optional: false,
        }
    }

    /* ---------- Modules ---------- */

    #[must_use]
    #[inline]
    pub fn module_declaration(&self, node: Node, kind: ModuleDeclarationKind<'a>) -> Statement<'a> {
        Statement::ModuleDeclaration(self.alloc(ModuleDeclaration { node, kind }))
    }

    #[must_use]
    #[inline]
    pub fn import_declaration(
        &self,
        specifiers: Vec<'a, ImportDeclarationSpecifier>,
        source: StringLiteral,
        assertions: Option<Vec<'a, ImportAttribute>>,
        import_kind: Option<ImportOrExportKind>,
    ) -> Box<'a, ImportDeclaration<'a>> {
        self.alloc(ImportDeclaration { specifiers, source, assertions, import_kind })
    }

    #[must_use]
    #[inline]
    pub fn export_all_declaration(
        &self,
        exported: Option<ModuleExportName>,
        source: StringLiteral,
        assertions: Option<Vec<'a, ImportAttribute>>, // Some(vec![]) for empty assertion
    ) -> Box<'a, ExportAllDeclaration<'a>> {
        self.alloc(ExportAllDeclaration { exported, source, assertions })
    }

    #[must_use]
    #[inline]
    pub fn export_default_declaration(
        &self,
        declaration: ExportDefaultDeclarationKind<'a>,
        exported: ModuleExportName,
    ) -> Box<'a, ExportDefaultDeclaration<'a>> {
        self.alloc(ExportDefaultDeclaration { declaration, exported })
    }

    #[must_use]
    #[inline]
    pub fn export_named_declaration(
        &self,
        declaration: Option<Declaration<'a>>,
        specifiers: Vec<'a, ExportSpecifier>,
        source: Option<StringLiteral>,
        export_kind: Option<ImportOrExportKind>, // `export type { foo }`
    ) -> Box<'a, ExportNamedDeclaration<'a>> {
        self.alloc(ExportNamedDeclaration { declaration, specifiers, source, export_kind })
    }

    /* ---------- JSX ----------------- */
    #[must_use]
    #[inline]
    pub fn jsx_element(
        &self,
        node: Node,
        opening_element: Box<'a, JSXOpeningElement<'a>>,
        closing_element: Option<Box<'a, JSXClosingElement<'a>>>,
        children: Vec<'a, JSXChild<'a>>,
    ) -> Box<'a, JSXElement<'a>> {
        self.alloc(JSXElement { node, opening_element, closing_element, children })
    }

    #[must_use]
    #[inline]
    pub fn jsx_opening_element(
        &self,
        node: Node,
        self_closing: bool,
        name: JSXElementName<'a>,
        attributes: Vec<'a, JSXAttributeItem<'a>>,
        type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
    ) -> Box<'a, JSXOpeningElement<'a>> {
        self.alloc(JSXOpeningElement { node, self_closing, name, attributes, type_parameters })
    }

    #[must_use]
    #[inline]
    pub fn jsx_closing_element(
        &self,
        node: Node,
        name: JSXElementName<'a>,
    ) -> Box<'a, JSXClosingElement<'a>> {
        self.alloc(JSXClosingElement { node, name })
    }

    #[must_use]
    #[inline]
    pub fn jsx_fragment(
        &self,
        node: Node,
        opening_fragment: JSXOpeningFragment,
        closing_fragment: JSXClosingFragment,
        children: Vec<'a, JSXChild<'a>>,
    ) -> Box<'a, JSXFragment<'a>> {
        self.alloc(JSXFragment { node, opening_fragment, closing_fragment, children })
    }

    #[must_use]
    #[inline]
    pub fn jsx_opening_fragment(&self, node: Node) -> JSXOpeningFragment {
        JSXOpeningFragment { node }
    }

    #[must_use]
    #[inline]
    pub fn jsx_closing_fragment(&self, node: Node) -> JSXClosingFragment {
        JSXClosingFragment { node }
    }

    #[must_use]
    #[inline]
    pub fn jsx_namespaced_name(
        &self,
        node: Node,
        namespace: JSXIdentifier,
        property: JSXIdentifier,
    ) -> Box<'a, JSXNamespacedName> {
        self.alloc(JSXNamespacedName { node, namespace, property })
    }

    #[must_use]
    #[inline]
    pub fn jsx_member_expression(
        &self,
        node: Node,
        object: JSXMemberExpressionObject<'a>,
        property: JSXIdentifier,
    ) -> Box<'a, JSXMemberExpression<'a>> {
        self.alloc(JSXMemberExpression { node, object, property })
    }

    #[must_use]
    #[inline]
    pub fn jsx_expression_container(
        &self,
        node: Node,
        expression: JSXExpression<'a>,
    ) -> JSXExpressionContainer<'a> {
        JSXExpressionContainer { node, expression }
    }

    #[must_use]
    #[inline]
    pub fn jsx_spread_child(&self, node: Node, expression: Expression<'a>) -> JSXSpreadChild<'a> {
        JSXSpreadChild { node, expression }
    }

    #[must_use]
    #[inline]
    pub fn jsx_empty_expression(&self, node: Node) -> JSXEmptyExpression {
        JSXEmptyExpression { node }
    }

    #[must_use]
    #[inline]
    pub fn jsx_attribute(
        &self,
        node: Node,
        name: JSXAttributeName<'a>,
        value: Option<JSXAttributeValue<'a>>,
    ) -> Box<'a, JSXAttribute<'a>> {
        self.alloc(JSXAttribute { node, name, value })
    }

    #[must_use]
    #[inline]
    pub fn jsx_spread_attribute(
        &self,
        node: Node,
        argument: Expression<'a>,
    ) -> Box<'a, JSXSpreadAttribute<'a>> {
        self.alloc(JSXSpreadAttribute { node, argument })
    }

    #[must_use]
    #[inline]
    pub fn jsx_identifier(&self, node: Node, name: Atom) -> JSXIdentifier {
        JSXIdentifier { node, name }
    }

    #[must_use]
    #[inline]
    pub fn jsx_text(&self, node: Node, value: Atom) -> JSXText {
        JSXText { node, value }
    }

    /* ---------- TypeScript ---------- */
    #[must_use]
    #[inline]
    pub fn ts_module_declaration(
        &self,
        node: Node,
        id: TSModuleDeclarationName,
        body: TSModuleDeclarationBody<'a>,
        declare: bool,
    ) -> Box<'a, TSModuleDeclaration<'a>> {
        self.alloc(TSModuleDeclaration { node, id, body, declare })
    }

    #[must_use]
    #[inline]
    pub fn ts_type_annotation(
        &self,
        node: Node,
        type_annotation: TSType<'a>,
    ) -> TSTypeAnnotation<'a> {
        TSTypeAnnotation { node, type_annotation }
    }

    #[must_use]
    #[inline]
    pub fn ts_literal_type(&self, node: Node, literal: TSLiteral<'a>) -> TSType<'a> {
        TSType::TSLiteralType(self.alloc(TSLiteralType { node, literal }))
    }

    #[must_use]
    #[inline]
    pub fn ts_union_type(&self, node: Node, types: Vec<'a, TSType<'a>>) -> TSType<'a> {
        TSType::TSUnionType(self.alloc(TSUnionType { node, types }))
    }

    #[must_use]
    #[inline]
    pub fn ts_intersection_type(&self, node: Node, types: Vec<'a, TSType<'a>>) -> TSType<'a> {
        TSType::TSIntersectionType(self.alloc(TSIntersectionType { node, types }))
    }

    #[must_use]
    #[inline]
    pub fn ts_type_operator_type(
        &self,
        node: Node,
        operator: TSTypeOperator,
        type_annotation: TSType<'a>,
    ) -> TSType<'a> {
        TSType::TSTypeOperatorType(self.alloc(TSTypeOperatorType {
            node,
            operator,
            type_annotation,
        }))
    }

    #[must_use]
    #[inline]
    pub fn ts_array_type(&self, node: Node, element_type: TSType<'a>) -> TSType<'a> {
        TSType::TSArrayType(self.alloc(TSArrayType { node, element_type }))
    }

    #[must_use]
    #[inline]
    pub fn ts_indexed_access_type(
        &self,
        node: Node,
        object_type: TSType<'a>,
        index_type: TSType<'a>,
    ) -> TSType<'a> {
        TSType::TSIndexedAccessType(self.alloc(TSIndexedAccessType {
            node,
            object_type,
            index_type,
        }))
    }

    #[must_use]
    #[inline]
    pub fn ts_tuple_type(
        &self,
        node: Node,
        element_types: Vec<'a, TSTupleElement<'a>>,
    ) -> TSType<'a> {
        TSType::TSTupleType(self.alloc(TSTupleType { node, element_types }))
    }

    #[must_use]
    #[inline]
    pub fn ts_type_reference(
        &self,
        node: Node,
        type_name: TSTypeName<'a>,
        type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
    ) -> TSType<'a> {
        TSType::TSTypeReference(self.alloc(TSTypeReference { node, type_name, type_parameters }))
    }

    #[must_use]
    #[inline]
    pub fn ts_type_literal(&self, node: Node, members: Vec<'a, TSSignature<'a>>) -> TSType<'a> {
        TSType::TSTypeLiteral(self.alloc(TSTypeLiteral { node, members }))
    }

    #[must_use]
    #[inline]
    pub fn ts_type_implement(
        &self,
        node: Node,
        expression: TSTypeName<'a>,
        type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
    ) -> Box<'a, TSClassImplements<'a>> {
        self.alloc(TSClassImplements { node, expression, type_parameters })
    }

    #[must_use]
    #[inline]
    pub fn ts_type_parameter(
        &self,
        node: Node,
        name: BindingIdentifier,
        constraint: Option<TSType<'a>>,
        default: Option<TSType<'a>>,
        r#in: bool,
        out: bool,
    ) -> Box<'a, TSTypeParameter<'a>> {
        self.alloc(TSTypeParameter { node, name, constraint, default, r#in, out })
    }

    #[must_use]
    #[inline]
    pub fn ts_type_parameters(
        &self,
        node: Node,
        params: Vec<'a, Box<'a, TSTypeParameter<'a>>>,
    ) -> Box<'a, TSTypeParameterDeclaration<'a>> {
        self.alloc(TSTypeParameterDeclaration { node, params })
    }

    #[must_use]
    #[inline]
    pub fn ts_interface_heritages(
        &self,
        extends: Vec<'a, (Expression<'a>, Option<Box<'a, TSTypeParameterInstantiation<'a>>>, Node)>,
    ) -> Vec<'a, Box<'a, TSInterfaceHeritage<'a>>> {
        Vec::from_iter_in(
            extends.into_iter().map(|(expression, type_parameters, node)| {
                self.alloc(TSInterfaceHeritage { node, expression, type_parameters })
            }),
            self.allocator,
        )
    }

    #[must_use]
    #[inline]
    pub fn ts_interface_body(
        &self,
        node: Node,
        body: Vec<'a, TSSignature<'a>>,
    ) -> Box<'a, TSInterfaceBody<'a>> {
        self.alloc(TSInterfaceBody { node, body })
    }

    #[must_use]
    #[inline]
    pub fn ts_index_signature(
        &self,
        node: Node,
        parameters: Vec<'a, Box<'a, TSIndexSignatureName<'a>>>,
        type_annotation: TSTypeAnnotation<'a>,
    ) -> TSSignature<'a> {
        TSSignature::TSIndexSignature(self.alloc(TSIndexSignature {
            node,
            parameters,
            type_annotation,
        }))
    }

    #[must_use]
    #[inline]
    pub fn ts_property_signature(
        &self,
        node: Node,
        computed: bool,
        optional: bool,
        readonly: bool,
        key: PropertyKey<'a>,
        type_annotation: Option<TSTypeAnnotation<'a>>,
    ) -> TSSignature<'a> {
        TSSignature::TSPropertySignature(self.alloc(TSPropertySignature {
            node,
            computed,
            optional,
            readonly,
            key,
            type_annotation,
        }))
    }

    #[must_use]
    #[inline]
    pub fn ts_call_signature_declaration(
        &self,
        node: Node,
        params: FormalParameters<'a>,
        return_type: Option<TSTypeAnnotation<'a>>,
        type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
    ) -> TSSignature<'a> {
        TSSignature::TSCallSignatureDeclaration(self.alloc(TSCallSignatureDeclaration {
            node,
            params,
            return_type,
            type_parameters,
        }))
    }

    #[must_use]
    #[inline]
    pub fn ts_construct_signature_declaration(
        &self,
        node: Node,
        params: FormalParameters<'a>,
        return_type: Option<TSTypeAnnotation<'a>>,
        type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
    ) -> TSSignature<'a> {
        TSSignature::TSConstructSignatureDeclaration(self.alloc(TSConstructSignatureDeclaration {
            node,
            params,
            return_type,
            type_parameters,
        }))
    }

    #[must_use]
    #[inline]
    pub fn ts_method_signature(
        &self,
        node: Node,
        key: PropertyKey<'a>,
        computed: bool,
        optional: bool,
        kind: TSMethodSignatureKind,
        params: FormalParameters<'a>,
        return_type: Option<TSTypeAnnotation<'a>>,
        type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
    ) -> TSSignature<'a> {
        TSSignature::TSMethodSignature(self.alloc(TSMethodSignature {
            node,
            key,
            computed,
            optional,
            kind,
            params,
            return_type,
            type_parameters,
        }))
    }

    #[must_use]
    #[inline]
    pub fn ts_module_block(
        &self,
        node: Node,
        body: Vec<'a, Statement<'a>>,
    ) -> Box<'a, TSModuleBlock<'a>> {
        self.alloc(TSModuleBlock { node, body })
    }

    #[must_use]
    #[inline]
    pub fn ts_type_arguments(
        &self,
        node: Node,
        params: Vec<'a, TSType<'a>>,
    ) -> Box<'a, TSTypeParameterInstantiation<'a>> {
        self.alloc(TSTypeParameterInstantiation { node, params })
    }

    #[must_use]
    #[inline]
    pub fn ts_non_null_expression(&self, node: Node, expression: Expression<'a>) -> Expression<'a> {
        Expression::TSNonNullExpression(self.alloc(TSNonNullExpression { node, expression }))
    }

    #[must_use]
    #[inline]
    pub fn ts_type_assertion(
        &self,
        node: Node,
        type_annotation: TSType<'a>,
        expression: Expression<'a>,
    ) -> Expression<'a> {
        Expression::TSTypeAssertion(self.alloc(TSTypeAssertion {
            node,
            type_annotation,
            expression,
        }))
    }

    #[must_use]
    #[inline]
    pub fn ts_import_equals_declaration(
        &self,
        node: Node,
        id: BindingIdentifier,
        module_reference: TSModuleReference<'a>,
        is_export: bool,
        import_kind: ImportOrExportKind,
    ) -> Declaration<'a> {
        Declaration::TSImportEqualsDeclaration(self.alloc(TSImportEqualsDeclaration {
            node,
            id,
            module_reference: self.alloc(module_reference),
            is_export,
            import_kind,
        }))
    }

    #[must_use]
    #[inline]
    pub fn ts_interface_declaration(
        &self,
        node: Node,
        id: BindingIdentifier,
        body: Box<'a, TSInterfaceBody<'a>>,
        type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
        extends: Option<Vec<'a, Box<'a, TSInterfaceHeritage<'a>>>>,
        declare: bool,
    ) -> Declaration<'a> {
        Declaration::TSInterfaceDeclaration(self.alloc(TSInterfaceDeclaration {
            node,
            id,
            body,
            type_parameters,
            extends,
            declare,
        }))
    }

    #[must_use]
    #[inline]
    pub fn ts_type_alias_declaration(
        &self,
        node: Node,
        id: BindingIdentifier,
        type_annotation: TSType<'a>,
        type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
        declare: bool,
    ) -> Declaration<'a> {
        Declaration::TSTypeAliasDeclaration(self.alloc(TSTypeAliasDeclaration {
            node,
            id,
            type_annotation,
            type_parameters,
            declare,
        }))
    }

    #[must_use]
    #[inline]
    pub fn ts_enum_declaration(
        &self,
        node: Node,
        id: BindingIdentifier,
        members: Vec<'a, TSEnumMember<'a>>,
        declare: bool,
        r#const: bool,
    ) -> Declaration<'a> {
        Declaration::TSEnumDeclaration(self.alloc(TSEnumDeclaration {
            node,
            id,
            members,
            declare,
            r#const,
        }))
    }

    #[must_use]
    #[inline]
    pub fn decorator(&self, node: Node, expression: Expression<'a>) -> Decorator<'a> {
        Decorator { node, expression }
    }

    #[must_use]
    #[inline]
    pub fn ts_void_keyword(&self, node: Node) -> TSType<'a> {
        TSType::TSVoidKeyword(self.alloc(TSVoidKeyword { node }))
    }

    #[must_use]
    #[inline]
    pub fn ts_this_keyword(&self, node: Node) -> TSType<'a> {
        TSType::TSThisKeyword(self.alloc(TSThisKeyword { node }))
    }

    #[must_use]
    #[inline]
    pub fn ts_any_keyword(&self, node: Node) -> TSType<'a> {
        TSType::TSAnyKeyword(self.alloc(TSAnyKeyword { node }))
    }

    #[must_use]
    #[inline]
    pub fn ts_unknown_keyword(&self, node: Node) -> TSType<'a> {
        TSType::TSUnknownKeyword(self.alloc(TSUnknownKeyword { node }))
    }

    #[must_use]
    #[inline]
    pub fn ts_number_keyword(&self, node: Node) -> TSType<'a> {
        TSType::TSNumberKeyword(self.alloc(TSNumberKeyword { node }))
    }

    #[must_use]
    #[inline]
    pub fn ts_boolean_keyword(&self, node: Node) -> TSType<'a> {
        TSType::TSBooleanKeyword(self.alloc(TSBooleanKeyword { node }))
    }

    #[must_use]
    #[inline]
    pub fn ts_object_keyword(&self, node: Node) -> TSType<'a> {
        TSType::TSObjectKeyword(self.alloc(TSObjectKeyword { node }))
    }

    #[must_use]
    #[inline]
    pub fn ts_string_keyword(&self, node: Node) -> TSType<'a> {
        TSType::TSStringKeyword(self.alloc(TSStringKeyword { node }))
    }

    #[must_use]
    #[inline]
    pub fn ts_bigint_keyword(&self, node: Node) -> TSType<'a> {
        TSType::TSBigIntKeyword(self.alloc(TSBigIntKeyword { node }))
    }

    #[must_use]
    #[inline]
    pub fn ts_symbol_keyword(&self, node: Node) -> TSType<'a> {
        TSType::TSSymbolKeyword(self.alloc(TSSymbolKeyword { node }))
    }

    #[must_use]
    #[inline]
    pub fn ts_null_keyword(&self, node: Node) -> TSType<'a> {
        TSType::TSNullKeyword(self.alloc(TSNullKeyword { node }))
    }

    #[must_use]
    #[inline]
    pub fn ts_undefined_keyword(&self, node: Node) -> TSType<'a> {
        TSType::TSUndefinedKeyword(self.alloc(TSUndefinedKeyword { node }))
    }

    #[must_use]
    #[inline]
    pub fn ts_never_keyword(&self, node: Node) -> TSType<'a> {
        TSType::TSNeverKeyword(self.alloc(TSNeverKeyword { node }))
    }

    #[must_use]
    #[inline]
    pub fn ts_template_literal_type(
        &self,
        node: Node,
        quasis: Vec<'a, TemplateElement>,
        types: Vec<'a, TSType<'a>>,
    ) -> TSType<'a> {
        TSType::TSTemplateLiteralType(self.alloc(TSTemplateLiteralType { node, quasis, types }))
    }

    #[must_use]
    #[inline]
    pub fn ts_type_query_type(
        &self,
        node: Node,
        expr_name: TSTypeName<'a>,
        type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
    ) -> TSType<'a> {
        TSType::TSTypeQuery(self.alloc(TSTypeQuery { node, expr_name, type_parameters }))
    }

    #[must_use]
    #[inline]
    pub fn ts_conditional_type(
        &self,
        node: Node,
        check_type: TSType<'a>,
        extends_type: TSType<'a>,
        true_type: TSType<'a>,
        false_type: TSType<'a>,
    ) -> TSType<'a> {
        TSType::TSConditionalType(self.alloc(TSConditionalType {
            node,
            check_type,
            extends_type,
            true_type,
            false_type,
        }))
    }

    #[must_use]
    #[inline]
    pub fn ts_mapped_type(
        &self,
        node: Node,
        type_parameter: Box<'a, TSTypeParameter<'a>>,
        name_type: Option<TSType<'a>>,
        type_annotation: TSType<'a>,
        optional: TSMappedTypeModifierOperator,
        readonly: TSMappedTypeModifierOperator,
    ) -> TSType<'a> {
        TSType::TSMappedType(self.alloc(TSMappedType {
            node,
            type_parameter,
            name_type,
            type_annotation,
            optional,
            readonly,
        }))
    }

    #[must_use]
    #[inline]
    pub fn ts_import_type(
        &self,
        node: Node,
        is_type_of: bool,
        parameter: TSType<'a>,
        qualifier: Option<TSTypeName<'a>>,
        type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
    ) -> TSType<'a> {
        TSType::TSImportType(self.alloc(TSImportType {
            node,
            is_type_of,
            parameter,
            qualifier,
            type_parameters,
        }))
    }

    #[must_use]
    #[inline]
    pub fn ts_constructor_type(
        &self,
        node: Node,
        r#abstract: bool,
        params: FormalParameters<'a>,
        return_type: TSTypeAnnotation<'a>,
        type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
    ) -> TSType<'a> {
        TSType::TSConstructorType(self.alloc(TSConstructorType {
            node,
            r#abstract,
            params,
            return_type,
            type_parameters,
        }))
    }

    #[must_use]
    #[inline]
    pub fn ts_function_type(
        &self,
        node: Node,
        params: FormalParameters<'a>,
        return_type: TSTypeAnnotation<'a>,
        type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
    ) -> TSType<'a> {
        TSType::TSFunctionType(self.alloc(TSFunctionType {
            node,
            params,
            return_type,
            type_parameters,
        }))
    }

    #[must_use]
    #[inline]
    pub fn ts_infer_type(
        &self,
        node: Node,
        type_parameter: Box<'a, TSTypeParameter<'a>>,
    ) -> TSType<'a> {
        TSType::TSInferType(self.alloc(TSInferType { node, type_parameter }))
    }

    #[must_use]
    #[inline]
    pub fn ts_type_predicate(
        &self,
        node: Node,
        parameter_name: TSTypePredicateName,
        asserts: bool,
        type_annotation: Option<TSTypeAnnotation<'a>>,
    ) -> TSType<'a> {
        TSType::TSTypePredicate(self.alloc(TSTypePredicate {
            node,
            parameter_name,
            asserts,
            type_annotation,
        }))
    }

    /* JSDoc */
    #[must_use]
    #[inline]
    pub fn js_doc_nullable_type(
        &self,
        node: Node,
        type_annotation: TSType<'a>,
        postfix: bool,
    ) -> TSType<'a> {
        TSType::JSDocNullableType(self.alloc(JSDocNullableType { node, type_annotation, postfix }))
    }

    #[must_use]
    #[inline]
    pub fn js_doc_unknown_type(&self, node: Node) -> TSType<'a> {
        TSType::JSDocUnknownType(self.alloc(JSDocUnknownType { node }))
    }
}
