#![allow(
    clippy::fn_params_excessive_bools,
    clippy::must_use_candidate, // must_use_candidate is too annoying for this file
    clippy::too_many_arguments,
    clippy::unused_self,
)]

use std::mem;

use oxc_allocator::{Allocator, Box, String, Vec};
use oxc_span::{Atom, GetSpan, SourceType, Span, SPAN};
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

    #[inline]
    pub fn copy<T>(self, src: &T) -> T {
        // SAFETY:
        // This should be safe as long as `src` is an reference from the allocator.
        // But honestly, I'm not really sure if this is safe.
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
        mem::replace(stmt, empty_stmt)
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
            Modifiers::empty(),
        );
        let empty_decl = Declaration::VariableDeclaration(empty_decl);
        mem::replace(decl, empty_decl)
    }

    #[inline]
    pub fn program(
        self,
        span: Span,
        source_type: SourceType,
        directives: Vec<'a, Directive<'a>>,
        hashbang: Option<Hashbang<'a>>,
        body: Vec<'a, Statement<'a>>,
    ) -> Program<'a> {
        Program::new(span, source_type, directives, hashbang, body)
    }

    /* ---------- Constructors ---------- */

    /// `void 0`
    #[inline]
    pub fn void_0(self) -> Expression<'a> {
        let left = self.number_literal(Span::default(), 0.0, "0", NumberBase::Decimal);
        let num = self.literal_number_expression(left);
        self.unary_expression(Span::default(), UnaryOperator::Void, num)
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
    pub fn boolean_literal(self, span: Span, value: bool) -> BooleanLiteral {
        BooleanLiteral { span, value }
    }

    #[inline]
    pub fn string_literal(self, span: Span, name: &str) -> StringLiteral<'a> {
        StringLiteral::new(span, self.new_atom(name))
    }

    #[inline]
    pub fn bigint_literal(self, span: Span, raw: Atom<'a>, base: BigintBase) -> BigIntLiteral<'a> {
        BigIntLiteral { span, raw, base }
    }

    #[inline]
    pub fn template_literal(
        self,
        span: Span,
        quasis: Vec<'a, TemplateElement<'a>>,
        expressions: Vec<'a, Expression<'a>>,
    ) -> TemplateLiteral<'a> {
        TemplateLiteral { span, quasis, expressions }
    }

    #[inline]
    pub fn template_element(
        self,
        span: Span,
        tail: bool,
        value: TemplateElementValue<'a>,
    ) -> TemplateElement<'a> {
        TemplateElement { span, tail, value }
    }

    #[inline]
    pub fn template_element_value(
        self,
        raw: Atom<'a>,
        cooked: Option<Atom<'a>>,
    ) -> TemplateElementValue<'a> {
        TemplateElementValue { raw, cooked }
    }

    #[inline]
    pub fn reg_exp_literal(
        self,
        span: Span,
        pattern: &'a str,
        flags: RegExpFlags,
    ) -> RegExpLiteral<'a> {
        RegExpLiteral { span, value: EmptyObject, regex: RegExp { pattern: pattern.into(), flags } }
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
        Expression::BigintLiteral(self.alloc(literal))
    }

    #[inline]
    pub fn literal_template_expression(self, literal: TemplateLiteral<'a>) -> Expression<'a> {
        Expression::TemplateLiteral(self.alloc(literal))
    }

    #[inline]
    pub fn identifier_name(self, span: Span, name: &str) -> IdentifierName<'a> {
        IdentifierName::new(span, self.new_atom(name))
    }

    #[inline]
    pub fn identifier_reference(self, span: Span, name: &str) -> IdentifierReference<'a> {
        IdentifierReference::new(span, self.new_atom(name))
    }

    #[inline]
    pub fn identifier_reference_expression(self, ident: IdentifierReference<'a>) -> Expression<'a> {
        Expression::Identifier(self.alloc(ident))
    }

    /* ---------- Statements ---------- */

    #[inline]
    pub fn directive(
        self,
        span: Span,
        expression: StringLiteral<'a>,
        directive: Atom<'a>,
    ) -> Directive<'a> {
        Directive { span, expression, directive }
    }

    #[inline]
    pub fn hashbang(self, span: Span, value: Atom<'a>) -> Hashbang<'a> {
        Hashbang { span, value }
    }

    #[inline]
    pub fn block(self, span: Span, body: Vec<'a, Statement<'a>>) -> Box<'a, BlockStatement<'a>> {
        self.alloc(BlockStatement::new(span, body))
    }

    #[inline]
    pub fn block_statement(self, block: Box<'a, BlockStatement<'a>>) -> Statement<'a> {
        Statement::BlockStatement(self.block(block.span, block.unbox().body))
    }

    #[inline]
    pub fn break_statement(self, span: Span, label: Option<LabelIdentifier<'a>>) -> Statement<'a> {
        Statement::BreakStatement(self.alloc(BreakStatement { span, label }))
    }

    #[inline]
    pub fn continue_statement(
        self,
        span: Span,
        label: Option<LabelIdentifier<'a>>,
    ) -> Statement<'a> {
        Statement::ContinueStatement(self.alloc(ContinueStatement { span, label }))
    }

    #[inline]
    pub fn debugger_statement(self, span: Span) -> Statement<'a> {
        Statement::DebuggerStatement(self.alloc(DebuggerStatement { span }))
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

    #[inline]
    pub fn do_while_statement(
        self,
        span: Span,
        body: Statement<'a>,
        test: Expression<'a>,
    ) -> Statement<'a> {
        Statement::DoWhileStatement(self.alloc(DoWhileStatement { span, body, test }))
    }

    #[inline]
    pub fn empty_statement(self, span: Span) -> Statement<'a> {
        Statement::EmptyStatement(self.alloc(EmptyStatement { span }))
    }

    #[inline]
    pub fn expression_statement(self, span: Span, expression: Expression<'a>) -> Statement<'a> {
        Statement::ExpressionStatement(self.alloc(ExpressionStatement { span, expression }))
    }

    #[inline]
    pub fn for_in_statement(
        self,
        span: Span,
        left: ForStatementLeft<'a>,
        right: Expression<'a>,
        body: Statement<'a>,
    ) -> Statement<'a> {
        Statement::ForInStatement(self.alloc(ForInStatement::new(span, left, right, body)))
    }

    #[inline]
    pub fn for_of_statement(
        self,
        span: Span,
        r#await: bool,
        left: ForStatementLeft<'a>,
        right: Expression<'a>,
        body: Statement<'a>,
    ) -> Statement<'a> {
        Statement::ForOfStatement(self.alloc(ForOfStatement::new(span, r#await, left, right, body)))
    }

    #[inline]
    pub fn for_statement(
        self,
        span: Span,
        init: Option<ForStatementInit<'a>>,
        test: Option<Expression<'a>>,
        update: Option<Expression<'a>>,
        body: Statement<'a>,
    ) -> Statement<'a> {
        Statement::ForStatement(self.alloc(ForStatement::new(span, init, test, update, body)))
    }

    #[inline]
    pub fn if_statement(
        self,
        span: Span,
        test: Expression<'a>,
        consequent: Statement<'a>,
        alternate: Option<Statement<'a>>,
    ) -> Statement<'a> {
        Statement::IfStatement(self.alloc(IfStatement { span, test, consequent, alternate }))
    }

    #[inline]
    pub fn labeled_statement(
        self,
        span: Span,
        label: LabelIdentifier<'a>,
        body: Statement<'a>,
    ) -> Statement<'a> {
        Statement::LabeledStatement(self.alloc(LabeledStatement { span, label, body }))
    }

    #[inline]
    pub fn return_statement(self, span: Span, argument: Option<Expression<'a>>) -> Statement<'a> {
        Statement::ReturnStatement(self.alloc(ReturnStatement { span, argument }))
    }

    #[inline]
    pub fn switch_statement(
        self,
        span: Span,
        discriminant: Expression<'a>,
        cases: Vec<'a, SwitchCase<'a>>,
    ) -> Statement<'a> {
        Statement::SwitchStatement(self.alloc(SwitchStatement::new(span, discriminant, cases)))
    }

    #[inline]
    pub fn switch_case(
        self,
        span: Span,
        test: Option<Expression<'a>>,
        consequent: Vec<'a, Statement<'a>>,
    ) -> SwitchCase<'a> {
        SwitchCase { span, test, consequent }
    }

    #[inline]
    pub fn throw_statement(self, span: Span, argument: Expression<'a>) -> Statement<'a> {
        Statement::ThrowStatement(self.alloc(ThrowStatement { span, argument }))
    }

    #[inline]
    pub fn try_statement(
        self,
        span: Span,
        block: Box<'a, BlockStatement<'a>>,
        handler: Option<Box<'a, CatchClause<'a>>>,
        finalizer: Option<Box<'a, BlockStatement<'a>>>,
    ) -> Statement<'a> {
        Statement::TryStatement(self.alloc(TryStatement { span, block, handler, finalizer }))
    }

    #[inline]
    pub fn catch_clause(
        self,
        span: Span,
        param: Option<CatchParameter<'a>>,
        body: Box<'a, BlockStatement<'a>>,
    ) -> Box<'a, CatchClause<'a>> {
        self.alloc(CatchClause::new(span, param, body))
    }

    #[inline]
    pub fn catch_parameter(self, span: Span, pattern: BindingPattern<'a>) -> CatchParameter<'a> {
        CatchParameter { span, pattern }
    }

    #[inline]
    pub fn while_statement(
        self,
        span: Span,
        test: Expression<'a>,
        body: Statement<'a>,
    ) -> Statement<'a> {
        Statement::WhileStatement(self.alloc(WhileStatement { span, test, body }))
    }

    #[inline]
    pub fn with_statement(
        self,
        span: Span,
        object: Expression<'a>,
        body: Statement<'a>,
    ) -> Statement<'a> {
        Statement::WithStatement(self.alloc(WithStatement { span, object, body }))
    }

    /* ---------- Expressions ---------- */

    #[inline]
    pub fn super_(self, span: Span) -> Expression<'a> {
        Expression::Super(self.alloc(Super { span }))
    }

    #[inline]
    pub fn meta_property(
        self,
        span: Span,
        meta: IdentifierName<'a>,
        property: IdentifierName<'a>,
    ) -> Expression<'a> {
        Expression::MetaProperty(self.alloc(MetaProperty { span, meta, property }))
    }

    #[inline]
    pub fn array_expression(
        self,
        span: Span,
        elements: Vec<'a, ArrayExpressionElement<'a>>,
        trailing_comma: Option<Span>,
    ) -> Expression<'a> {
        Expression::ArrayExpression(self.alloc(ArrayExpression { span, elements, trailing_comma }))
    }

    #[inline]
    pub fn arrow_function_expression(
        self,
        span: Span,
        expression: bool,
        r#async: bool,
        params: Box<'a, FormalParameters<'a>>,
        body: Box<'a, FunctionBody<'a>>,
        type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
        return_type: Option<Box<'a, TSTypeAnnotation<'a>>>,
    ) -> Expression<'a> {
        Expression::ArrowFunctionExpression(self.alloc(ArrowFunctionExpression::new(
            span,
            expression,
            r#async,
            params,
            body,
            type_parameters,
            return_type,
        )))
    }

    #[inline]
    pub fn assignment_expression(
        self,
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

    #[inline]
    pub fn array_assignment_target(self, array: ArrayAssignmentTarget<'a>) -> AssignmentTarget<'a> {
        AssignmentTarget::ArrayAssignmentTarget(self.alloc(array))
    }

    #[inline]
    pub fn array_assignment_target_maybe_default(
        self,
        array: ArrayAssignmentTarget<'a>,
    ) -> AssignmentTargetMaybeDefault<'a> {
        AssignmentTargetMaybeDefault::ArrayAssignmentTarget(self.alloc(array))
    }

    #[inline]
    pub fn object_assignment_target(
        self,
        array: ObjectAssignmentTarget<'a>,
    ) -> AssignmentTarget<'a> {
        AssignmentTarget::ObjectAssignmentTarget(self.alloc(array))
    }

    #[inline]
    pub fn assignment_target_property_property(
        self,
        span: Span,
        name: PropertyKey<'a>,
        binding: AssignmentTargetMaybeDefault<'a>,
    ) -> AssignmentTargetProperty<'a> {
        AssignmentTargetProperty::AssignmentTargetPropertyProperty(
            self.alloc(AssignmentTargetPropertyProperty { span, name, binding }),
        )
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
        AssignmentTarget::from(SimpleAssignmentTarget::from(expr))
    }

    #[inline]
    pub fn await_expression(self, span: Span, argument: Expression<'a>) -> Expression<'a> {
        Expression::AwaitExpression(self.alloc(AwaitExpression { span, argument }))
    }

    #[inline]
    pub fn binary_expression(
        self,
        span: Span,
        left: Expression<'a>,
        operator: BinaryOperator,
        right: Expression<'a>,
    ) -> Expression<'a> {
        Expression::BinaryExpression(self.alloc(BinaryExpression { span, left, operator, right }))
    }

    #[inline]
    pub fn call_expression(
        self,
        span: Span,
        callee: Expression<'a>,
        arguments: Vec<'a, Argument<'a>>,
        optional: bool, // for optional chaining
        type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
    ) -> Expression<'a> {
        Expression::CallExpression(self.alloc(CallExpression {
            span,
            callee,
            arguments,
            optional,
            type_parameters,
        }))
    }

    #[inline]
    pub fn chain_expression(self, span: Span, expression: ChainElement<'a>) -> Expression<'a> {
        Expression::ChainExpression(self.alloc(ChainExpression { span, expression }))
    }

    #[inline]
    pub fn class_expression(self, class: Box<'a, Class<'a>>) -> Expression<'a> {
        Expression::ClassExpression(class)
    }

    #[inline]
    pub fn conditional_expression(
        self,
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

    #[inline]
    pub fn function_expression(self, function: Box<'a, Function<'a>>) -> Expression<'a> {
        Expression::FunctionExpression(function)
    }

    #[inline]
    pub fn import_expression(
        self,
        span: Span,
        source: Expression<'a>,
        arguments: Vec<'a, Expression<'a>>,
    ) -> Expression<'a> {
        Expression::ImportExpression(self.alloc(ImportExpression { span, source, arguments }))
    }

    #[inline]
    pub fn logical_expression(
        self,
        span: Span,
        left: Expression<'a>,
        operator: LogicalOperator,
        right: Expression<'a>,
    ) -> Expression<'a> {
        Expression::LogicalExpression(self.alloc(LogicalExpression { span, left, operator, right }))
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
    pub fn computed_member_expression(
        self,
        span: Span,
        object: Expression<'a>,
        expression: Expression<'a>,
        optional: bool, // for optional chaining
    ) -> Expression<'a> {
        self.member_expression(self.computed_member(span, object, expression, optional))
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
    pub fn static_member_expression(
        self,
        span: Span,
        object: Expression<'a>,
        property: IdentifierName<'a>,
        optional: bool, // for optional chaining
    ) -> Expression<'a> {
        self.member_expression(self.static_member(span, object, property, optional))
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
    pub fn private_in_expression(
        self,
        span: Span,
        left: PrivateIdentifier<'a>,
        right: Expression<'a>,
    ) -> Expression<'a> {
        Expression::PrivateInExpression(self.alloc(PrivateInExpression {
            span,
            left,
            operator: BinaryOperator::In,
            right,
        }))
    }

    #[inline]
    pub fn private_field_expression(
        self,
        span: Span,
        object: Expression<'a>,
        field: PrivateIdentifier<'a>,
        optional: bool,
    ) -> Expression<'a> {
        self.member_expression(self.private_field(span, object, field, optional))
    }

    #[inline]
    pub fn new_expression(
        self,
        span: Span,
        callee: Expression<'a>,
        arguments: Vec<'a, Argument<'a>>,
        type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
    ) -> Expression<'a> {
        Expression::NewExpression(self.alloc(NewExpression {
            span,
            callee,
            arguments,
            type_parameters,
        }))
    }

    #[inline]
    pub fn object_expression(
        self,
        span: Span,
        properties: Vec<'a, ObjectPropertyKind<'a>>,
        trailing_comma: Option<Span>,
    ) -> Expression<'a> {
        Expression::ObjectExpression(self.alloc(ObjectExpression {
            span,
            properties,
            trailing_comma,
        }))
    }

    #[inline]
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

    #[inline]
    pub fn parenthesized_expression(
        self,
        span: Span,
        expression: Expression<'a>,
    ) -> Expression<'a> {
        Expression::ParenthesizedExpression(
            self.alloc(ParenthesizedExpression { span, expression }),
        )
    }

    #[inline]
    pub fn sequence_expression(
        self,
        span: Span,
        expressions: Vec<'a, Expression<'a>>,
    ) -> Expression<'a> {
        Expression::SequenceExpression(self.alloc(SequenceExpression { span, expressions }))
    }

    #[inline]
    pub fn tagged_template_expression(
        self,
        span: Span,
        tag: Expression<'a>,
        quasi: TemplateLiteral<'a>,
        type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
    ) -> Expression<'a> {
        Expression::TaggedTemplateExpression(self.alloc(TaggedTemplateExpression {
            span,
            tag,
            quasi,
            type_parameters,
        }))
    }

    #[inline]
    pub fn template_literal_expression(
        self,
        template_literal: TemplateLiteral<'a>,
    ) -> Expression<'a> {
        Expression::TemplateLiteral(self.alloc(template_literal))
    }

    #[inline]
    pub fn this_expression(self, span: Span) -> Expression<'a> {
        Expression::ThisExpression(self.alloc(ThisExpression { span }))
    }

    #[inline]
    pub fn unary_expression(
        self,
        span: Span,
        operator: UnaryOperator,
        argument: Expression<'a>,
    ) -> Expression<'a> {
        Expression::UnaryExpression(self.alloc(UnaryExpression { span, operator, argument }))
    }

    #[inline]
    pub fn update_expression(
        self,
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

    #[inline]
    pub fn yield_expression(
        self,
        span: Span,
        delegate: bool,
        argument: Option<Expression<'a>>,
    ) -> Expression<'a> {
        Expression::YieldExpression(self.alloc(YieldExpression { span, delegate, argument }))
    }

    /* ---------- Functions ---------- */
    #[inline]
    pub fn function_declaration(self, func: Box<'a, Function<'a>>) -> Statement<'a> {
        Statement::FunctionDeclaration(func)
    }

    #[inline]
    pub fn formal_parameters(
        self,
        span: Span,
        kind: FormalParameterKind,
        items: Vec<'a, FormalParameter<'a>>,
        rest: Option<Box<'a, BindingRestElement<'a>>>,
    ) -> Box<'a, FormalParameters<'a>> {
        self.alloc(FormalParameters { span, kind, items, rest })
    }

    #[inline]
    pub fn plain_formal_parameter(
        self,
        span: Span,
        pattern: BindingPattern<'a>,
    ) -> FormalParameter<'a> {
        self.formal_parameter(span, pattern, None, false, false, self.new_vec())
    }

    #[inline]
    pub fn formal_parameter(
        self,
        span: Span,
        pattern: BindingPattern<'a>,
        accessibility: Option<TSAccessibility>,
        readonly: bool,
        r#override: bool,
        decorators: Vec<'a, Decorator<'a>>,
    ) -> FormalParameter<'a> {
        FormalParameter { span, pattern, accessibility, readonly, r#override, decorators }
    }

    #[inline]
    pub fn ts_this_parameter(
        self,
        span: Span,
        this: IdentifierName<'a>,
        type_annotation: Option<Box<'a, TSTypeAnnotation<'a>>>,
    ) -> TSThisParameter<'a> {
        TSThisParameter { span, this, type_annotation }
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
        self.function(
            r#type,
            span,
            id,
            false,
            false,
            None,
            params,
            body,
            None,
            None,
            Modifiers::empty(),
        )
    }

    #[inline]
    pub fn function(
        self,
        r#type: FunctionType,
        span: Span,
        id: Option<BindingIdentifier<'a>>,
        generator: bool,
        r#async: bool,
        this_param: Option<TSThisParameter<'a>>,
        params: Box<'a, FormalParameters<'a>>,
        body: Option<Box<'a, FunctionBody<'a>>>,
        type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
        return_type: Option<Box<'a, TSTypeAnnotation<'a>>>,
        modifiers: Modifiers<'a>,
    ) -> Box<'a, Function<'a>> {
        self.alloc(Function::new(
            r#type,
            span,
            id,
            generator,
            r#async,
            this_param,
            params,
            body,
            type_parameters,
            return_type,
            modifiers,
        ))
    }

    #[inline]
    pub fn function_body(
        self,
        span: Span,
        directives: Vec<'a, Directive<'a>>,
        statements: Vec<'a, Statement<'a>>,
    ) -> Box<'a, FunctionBody<'a>> {
        self.alloc(FunctionBody { span, directives, statements })
    }

    /* ---------- Class ---------- */

    #[inline]
    pub fn class(
        self,
        r#type: ClassType,
        span: Span,
        id: Option<BindingIdentifier<'a>>,
        super_class: Option<Expression<'a>>,
        body: Box<'a, ClassBody<'a>>,
        type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
        super_type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
        implements: Option<Vec<'a, TSClassImplements<'a>>>,
        decorators: Vec<'a, Decorator<'a>>,
        modifiers: Modifiers<'a>,
    ) -> Box<'a, Class<'a>> {
        self.alloc(Class::new(
            r#type,
            span,
            decorators,
            id,
            super_class,
            body,
            type_parameters,
            super_type_parameters,
            implements,
            modifiers,
        ))
    }

    #[inline]
    pub fn class_body(self, span: Span, body: Vec<'a, ClassElement<'a>>) -> Box<'a, ClassBody<'a>> {
        self.alloc(ClassBody { span, body })
    }

    #[inline]
    pub fn class_declaration(self, class: Box<'a, Class<'a>>) -> Statement<'a> {
        Statement::ClassDeclaration(class)
    }

    #[inline]
    pub fn static_block(self, span: Span, body: Vec<'a, Statement<'a>>) -> ClassElement<'a> {
        ClassElement::StaticBlock(self.alloc(StaticBlock::new(span, body)))
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
        decorators: Vec<'a, Decorator<'a>>,
    ) -> ClassElement<'a> {
        ClassElement::PropertyDefinition(self.alloc(PropertyDefinition {
            r#type,
            span,
            key,
            value,
            computed,
            r#static,
            declare: false,
            r#override: false,
            optional: false,
            definite: false,
            readonly: false,
            type_annotation: None,
            accessibility: None,
            decorators,
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

    #[inline]
    pub fn accessor_property(
        self,
        r#type: AccessorPropertyType,
        span: Span,
        key: PropertyKey<'a>,
        value: Option<Expression<'a>>,
        computed: bool,
        r#static: bool,
        decorators: Vec<'a, Decorator<'a>>,
    ) -> ClassElement<'a> {
        ClassElement::AccessorProperty(self.alloc(AccessorProperty {
            r#type,
            span,
            key,
            value,
            computed,
            r#static,
            decorators,
        }))
    }

    /* ---------- Declarations ---------- */

    #[inline]
    pub fn variable_declaration(
        self,
        span: Span,
        kind: VariableDeclarationKind,
        declarations: Vec<'a, VariableDeclarator<'a>>,
        modifiers: Modifiers<'a>,
    ) -> Box<'a, VariableDeclaration<'a>> {
        self.alloc(VariableDeclaration { span, kind, declarations, modifiers })
    }

    #[inline]
    pub fn variable_declarator(
        self,
        span: Span,
        kind: VariableDeclarationKind,
        id: BindingPattern<'a>,
        init: Option<Expression<'a>>,
        definite: bool,
    ) -> VariableDeclarator<'a> {
        VariableDeclarator { span, kind, id, init, definite }
    }

    #[inline]
    pub fn using_declaration(
        self,
        span: Span,
        declarations: Vec<'a, VariableDeclarator<'a>>,
        is_await: bool,
    ) -> UsingDeclaration<'a> {
        UsingDeclaration { span, is_await, declarations }
    }

    /* ---------- Patterns ---------- */

    #[inline]
    pub fn binding_pattern(
        self,
        kind: BindingPatternKind<'a>,
        type_annotation: Option<Box<'a, TSTypeAnnotation<'a>>>,
        optional: bool,
    ) -> BindingPattern<'a> {
        BindingPattern { kind, type_annotation, optional }
    }

    #[inline]
    pub fn binding_pattern_identifier(
        self,
        identifier: BindingIdentifier<'a>,
    ) -> BindingPatternKind<'a> {
        BindingPatternKind::BindingIdentifier(self.alloc(identifier))
    }

    #[inline]
    pub fn object_pattern(
        self,
        span: Span,
        properties: Vec<'a, BindingProperty<'a>>,
        rest: Option<Box<'a, BindingRestElement<'a>>>,
    ) -> BindingPatternKind<'a> {
        BindingPatternKind::ObjectPattern(self.alloc(ObjectPattern { span, properties, rest }))
    }

    #[inline]
    pub fn binding_property(
        self,
        span: Span,
        key: PropertyKey<'a>,
        value: BindingPattern<'a>,
        shorthand: bool,
        computed: bool,
    ) -> BindingProperty<'a> {
        BindingProperty { span, key, value, shorthand, computed }
    }

    #[inline]
    pub fn spread_element(
        self,
        span: Span,
        argument: Expression<'a>,
    ) -> Box<'a, SpreadElement<'a>> {
        self.alloc(SpreadElement { span, argument })
    }

    #[inline]
    pub fn array_pattern(
        self,
        span: Span,
        elements: Vec<'a, Option<BindingPattern<'a>>>,
        rest: Option<Box<'a, BindingRestElement<'a>>>,
    ) -> BindingPatternKind<'a> {
        BindingPatternKind::ArrayPattern(self.alloc(ArrayPattern { span, elements, rest }))
    }

    #[inline]
    pub fn assignment_pattern(
        self,
        span: Span,
        left: BindingPattern<'a>,
        right: Expression<'a>,
    ) -> BindingPattern<'a> {
        let pattern = self.alloc(AssignmentPattern { span, left, right });
        BindingPattern {
            kind: BindingPatternKind::AssignmentPattern(pattern),
            type_annotation: None,
            optional: false,
        }
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
    pub fn property_key_expression(self, expr: Expression<'a>) -> PropertyKey<'a> {
        PropertyKey::from(expr)
    }

    /* ---------- Modules ---------- */

    #[inline]
    pub fn module_declaration(self, decl: ModuleDeclaration<'a>) -> Statement<'a> {
        Statement::from(decl)
    }

    #[inline]
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

    #[inline]
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

    #[inline]
    pub fn export_default_declaration(
        self,
        span: Span,
        declaration: ExportDefaultDeclarationKind<'a>,
        exported: ModuleExportName<'a>,
    ) -> Box<'a, ExportDefaultDeclaration<'a>> {
        self.alloc(ExportDefaultDeclaration { span, declaration, exported })
    }

    #[inline]
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

    /* ---------- JSX ----------------- */
    #[inline]
    pub fn jsx_element(
        self,
        span: Span,
        opening_element: Box<'a, JSXOpeningElement<'a>>,
        closing_element: Option<Box<'a, JSXClosingElement<'a>>>,
        children: Vec<'a, JSXChild<'a>>,
    ) -> Box<'a, JSXElement<'a>> {
        self.alloc(JSXElement { span, opening_element, closing_element, children })
    }

    #[inline]
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

    #[inline]
    pub fn jsx_closing_element(
        self,
        span: Span,
        name: JSXElementName<'a>,
    ) -> Box<'a, JSXClosingElement<'a>> {
        self.alloc(JSXClosingElement { span, name })
    }

    #[inline]
    pub fn jsx_fragment(
        self,
        span: Span,
        opening_fragment: JSXOpeningFragment,
        closing_fragment: JSXClosingFragment,
        children: Vec<'a, JSXChild<'a>>,
    ) -> Box<'a, JSXFragment<'a>> {
        self.alloc(JSXFragment { span, opening_fragment, closing_fragment, children })
    }

    #[inline]
    pub fn jsx_opening_fragment(self, span: Span) -> JSXOpeningFragment {
        JSXOpeningFragment { span }
    }

    #[inline]
    pub fn jsx_closing_fragment(self, span: Span) -> JSXClosingFragment {
        JSXClosingFragment { span }
    }

    #[inline]
    pub fn jsx_namespaced_name(
        self,
        span: Span,
        namespace: JSXIdentifier<'a>,
        property: JSXIdentifier<'a>,
    ) -> Box<'a, JSXNamespacedName<'a>> {
        self.alloc(JSXNamespacedName { span, namespace, property })
    }

    #[inline]
    pub fn jsx_member_expression(
        self,
        span: Span,
        object: JSXMemberExpressionObject<'a>,
        property: JSXIdentifier<'a>,
    ) -> Box<'a, JSXMemberExpression<'a>> {
        self.alloc(JSXMemberExpression { span, object, property })
    }

    #[inline]
    pub fn jsx_expression_container(
        self,
        span: Span,
        expression: JSXExpression<'a>,
    ) -> Box<'a, JSXExpressionContainer<'a>> {
        self.alloc(JSXExpressionContainer { span, expression })
    }

    #[inline]
    pub fn jsx_spread_child(
        self,
        span: Span,
        expression: Expression<'a>,
    ) -> Box<'a, JSXSpreadChild<'a>> {
        self.alloc(JSXSpreadChild { span, expression })
    }

    #[inline]
    pub fn jsx_empty_expression(self, span: Span) -> JSXEmptyExpression {
        JSXEmptyExpression { span }
    }

    #[inline]
    pub fn jsx_attribute(
        self,
        span: Span,
        name: JSXAttributeName<'a>,
        value: Option<JSXAttributeValue<'a>>,
    ) -> Box<'a, JSXAttribute<'a>> {
        self.alloc(JSXAttribute { span, name, value })
    }

    #[inline]
    pub fn jsx_spread_attribute(
        self,
        span: Span,
        argument: Expression<'a>,
    ) -> Box<'a, JSXSpreadAttribute<'a>> {
        self.alloc(JSXSpreadAttribute { span, argument })
    }

    #[inline]
    pub fn jsx_identifier(self, span: Span, name: Atom<'a>) -> JSXIdentifier<'a> {
        JSXIdentifier { span, name }
    }

    #[inline]
    pub fn jsx_text(self, span: Span, value: Atom<'a>) -> Box<'a, JSXText<'a>> {
        self.alloc(JSXText { span, value })
    }

    /* ---------- TypeScript ---------- */
    #[inline]
    pub fn ts_module_declaration(
        self,
        span: Span,
        id: TSModuleDeclarationName<'a>,
        body: Option<TSModuleDeclarationBody<'a>>,
        kind: TSModuleDeclarationKind,
        modifiers: Modifiers<'a>,
    ) -> Box<'a, TSModuleDeclaration<'a>> {
        self.alloc(TSModuleDeclaration::new(span, id, body, kind, modifiers))
    }

    #[inline]
    pub fn ts_type_annotation(
        self,
        span: Span,
        type_annotation: TSType<'a>,
    ) -> Box<'a, TSTypeAnnotation<'a>> {
        self.alloc(TSTypeAnnotation { span, type_annotation })
    }

    #[inline]
    pub fn ts_literal_type(self, span: Span, literal: TSLiteral<'a>) -> TSType<'a> {
        TSType::TSLiteralType(self.alloc(TSLiteralType { span, literal }))
    }

    #[inline]
    pub fn ts_union_type(self, span: Span, types: Vec<'a, TSType<'a>>) -> TSType<'a> {
        TSType::TSUnionType(self.alloc(TSUnionType { span, types }))
    }

    #[inline]
    pub fn ts_intersection_type(self, span: Span, types: Vec<'a, TSType<'a>>) -> TSType<'a> {
        TSType::TSIntersectionType(self.alloc(TSIntersectionType { span, types }))
    }

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
    pub fn ts_array_type(self, span: Span, element_type: TSType<'a>) -> TSType<'a> {
        TSType::TSArrayType(self.alloc(TSArrayType { span, element_type }))
    }

    #[inline]
    pub fn ts_indexed_access_type(
        self,
        span: Span,
        object_type: TSType<'a>,
        index_type: TSType<'a>,
    ) -> TSType<'a> {
        TSType::TSIndexedAccessType(self.alloc(TSIndexedAccessType {
            span,
            object_type,
            index_type,
        }))
    }

    #[inline]
    pub fn ts_tuple_type(
        self,
        span: Span,
        element_types: Vec<'a, TSTupleElement<'a>>,
    ) -> TSType<'a> {
        TSType::TSTupleType(self.alloc(TSTupleType { span, element_types }))
    }

    #[inline]
    pub fn ts_type_reference(
        self,
        span: Span,
        type_name: TSTypeName<'a>,
        type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
    ) -> TSType<'a> {
        TSType::TSTypeReference(self.alloc(TSTypeReference { span, type_name, type_parameters }))
    }

    #[inline]
    pub fn ts_type_literal(self, span: Span, members: Vec<'a, TSSignature<'a>>) -> TSType<'a> {
        TSType::TSTypeLiteral(self.alloc(TSTypeLiteral { span, members }))
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
    pub fn ts_type_parameter(
        self,
        span: Span,
        name: BindingIdentifier<'a>,
        constraint: Option<TSType<'a>>,
        default: Option<TSType<'a>>,
        r#in: bool,
        out: bool,
        r#const: bool,
    ) -> TSTypeParameter<'a> {
        TSTypeParameter::new(span, name, constraint, default, r#in, out, r#const)
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
    pub fn ts_interface_body(
        self,
        span: Span,
        body: Vec<'a, TSSignature<'a>>,
    ) -> Box<'a, TSInterfaceBody<'a>> {
        self.alloc(TSInterfaceBody { span, body })
    }

    #[inline]
    pub fn ts_index_signature(
        self,
        span: Span,
        parameters: Vec<'a, TSIndexSignatureName<'a>>,
        type_annotation: Box<'a, TSTypeAnnotation<'a>>,
        readonly: bool,
    ) -> TSSignature<'a> {
        TSSignature::TSIndexSignature(self.alloc(TSIndexSignature {
            span,
            parameters,
            type_annotation,
            readonly,
        }))
    }

    #[inline]
    pub fn ts_property_signature(
        self,
        span: Span,
        computed: bool,
        optional: bool,
        readonly: bool,
        key: PropertyKey<'a>,
        type_annotation: Option<Box<'a, TSTypeAnnotation<'a>>>,
    ) -> TSSignature<'a> {
        TSSignature::TSPropertySignature(self.alloc(TSPropertySignature {
            span,
            computed,
            optional,
            readonly,
            key,
            type_annotation,
        }))
    }

    #[inline]
    pub fn ts_call_signature_declaration(
        self,
        span: Span,
        this_param: Option<TSThisParameter<'a>>,
        params: Box<'a, FormalParameters<'a>>,
        return_type: Option<Box<'a, TSTypeAnnotation<'a>>>,
        type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
    ) -> TSSignature<'a> {
        TSSignature::TSCallSignatureDeclaration(self.alloc(TSCallSignatureDeclaration {
            span,
            this_param,
            params,
            return_type,
            type_parameters,
        }))
    }

    #[inline]
    pub fn ts_construct_signature_declaration(
        self,
        span: Span,
        params: Box<'a, FormalParameters<'a>>,
        return_type: Option<Box<'a, TSTypeAnnotation<'a>>>,
        type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
    ) -> TSSignature<'a> {
        TSSignature::TSConstructSignatureDeclaration(self.alloc(TSConstructSignatureDeclaration {
            span,
            params,
            return_type,
            type_parameters,
        }))
    }

    #[inline]
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
    ) -> TSSignature<'a> {
        TSSignature::TSMethodSignature(self.alloc(TSMethodSignature {
            span,
            key,
            computed,
            optional,
            kind,
            this_param,
            params,
            return_type,
            type_parameters,
        }))
    }

    #[inline]
    pub fn ts_module_block(
        self,
        span: Span,
        body: Vec<'a, Statement<'a>>,
    ) -> Box<'a, TSModuleBlock<'a>> {
        self.alloc(TSModuleBlock { span, body })
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
    pub fn ts_as_expression(
        self,
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
    ) -> Expression<'a> {
        Expression::TSAsExpression(self.alloc(TSAsExpression { span, expression, type_annotation }))
    }

    #[inline]
    pub fn ts_satisfies_expression(
        self,
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
    ) -> Expression<'a> {
        Expression::TSSatisfiesExpression(self.alloc(TSSatisfiesExpression {
            span,
            expression,
            type_annotation,
        }))
    }

    #[inline]
    pub fn ts_instantiation_expression(
        self,
        span: Span,
        expression: Expression<'a>,
        type_parameters: Box<'a, TSTypeParameterInstantiation<'a>>,
    ) -> Expression<'a> {
        Expression::TSInstantiationExpression(self.alloc(TSInstantiationExpression {
            span,
            expression,
            type_parameters,
        }))
    }

    #[inline]
    pub fn ts_non_null_expression(self, span: Span, expression: Expression<'a>) -> Expression<'a> {
        Expression::TSNonNullExpression(self.alloc(TSNonNullExpression { span, expression }))
    }

    #[inline]
    pub fn ts_type_assertion(
        self,
        span: Span,
        type_annotation: TSType<'a>,
        expression: Expression<'a>,
    ) -> Expression<'a> {
        Expression::TSTypeAssertion(self.alloc(TSTypeAssertion {
            span,
            expression,
            type_annotation,
        }))
    }

    #[inline]
    pub fn ts_import_equals_declaration(
        self,
        span: Span,
        id: BindingIdentifier<'a>,
        module_reference: TSModuleReference<'a>,
        import_kind: ImportOrExportKind,
    ) -> Declaration<'a> {
        Declaration::TSImportEqualsDeclaration(self.alloc(TSImportEqualsDeclaration {
            span,
            id,
            module_reference,
            import_kind,
        }))
    }

    #[inline]
    pub fn ts_interface_declaration(
        self,
        span: Span,
        id: BindingIdentifier<'a>,
        body: Box<'a, TSInterfaceBody<'a>>,
        type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
        extends: Option<Vec<'a, TSInterfaceHeritage<'a>>>,
        modifiers: Modifiers<'a>,
    ) -> Declaration<'a> {
        Declaration::TSInterfaceDeclaration(self.alloc(TSInterfaceDeclaration {
            span,
            id,
            body,
            type_parameters,
            extends,
            modifiers,
        }))
    }

    #[inline]
    pub fn ts_type_alias_declaration(
        self,
        span: Span,
        id: BindingIdentifier<'a>,
        type_annotation: TSType<'a>,
        type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
        modifiers: Modifiers<'a>,
    ) -> Declaration<'a> {
        Declaration::TSTypeAliasDeclaration(self.alloc(TSTypeAliasDeclaration {
            span,
            id,
            type_annotation,
            type_parameters,
            modifiers,
        }))
    }

    #[inline]
    pub fn ts_enum_declaration(
        self,
        span: Span,
        id: BindingIdentifier<'a>,
        members: Vec<'a, TSEnumMember<'a>>,
        modifiers: Modifiers<'a>,
    ) -> Declaration<'a> {
        Declaration::TSEnumDeclaration(
            self.alloc(TSEnumDeclaration::new(span, id, members, modifiers)),
        )
    }

    #[inline]
    pub fn decorator(self, span: Span, expression: Expression<'a>) -> Decorator<'a> {
        Decorator { span, expression }
    }

    #[inline]
    pub fn ts_void_keyword(self, span: Span) -> TSType<'a> {
        TSType::TSVoidKeyword(self.alloc(TSVoidKeyword { span }))
    }

    #[inline]
    pub fn ts_this_keyword(self, span: Span) -> TSType<'a> {
        TSType::TSThisType(self.alloc(TSThisType { span }))
    }

    #[inline]
    pub fn ts_any_keyword(self, span: Span) -> TSType<'a> {
        TSType::TSAnyKeyword(self.alloc(TSAnyKeyword { span }))
    }

    #[inline]
    pub fn ts_unknown_keyword(self, span: Span) -> TSType<'a> {
        TSType::TSUnknownKeyword(self.alloc(TSUnknownKeyword { span }))
    }

    #[inline]
    pub fn ts_number_keyword(self, span: Span) -> TSType<'a> {
        TSType::TSNumberKeyword(self.alloc(TSNumberKeyword { span }))
    }

    #[inline]
    pub fn ts_boolean_keyword(self, span: Span) -> TSType<'a> {
        TSType::TSBooleanKeyword(self.alloc(TSBooleanKeyword { span }))
    }

    #[inline]
    pub fn ts_object_keyword(self, span: Span) -> TSType<'a> {
        TSType::TSObjectKeyword(self.alloc(TSObjectKeyword { span }))
    }

    #[inline]
    pub fn ts_string_keyword(self, span: Span) -> TSType<'a> {
        TSType::TSStringKeyword(self.alloc(TSStringKeyword { span }))
    }

    #[inline]
    pub fn ts_bigint_keyword(self, span: Span) -> TSType<'a> {
        TSType::TSBigIntKeyword(self.alloc(TSBigIntKeyword { span }))
    }

    #[inline]
    pub fn ts_symbol_keyword(self, span: Span) -> TSType<'a> {
        TSType::TSSymbolKeyword(self.alloc(TSSymbolKeyword { span }))
    }

    #[inline]
    pub fn ts_null_keyword(self, span: Span) -> TSType<'a> {
        TSType::TSNullKeyword(self.alloc(TSNullKeyword { span }))
    }

    #[inline]
    pub fn ts_undefined_keyword(self, span: Span) -> TSType<'a> {
        TSType::TSUndefinedKeyword(self.alloc(TSUndefinedKeyword { span }))
    }

    #[inline]
    pub fn ts_never_keyword(self, span: Span) -> TSType<'a> {
        TSType::TSNeverKeyword(self.alloc(TSNeverKeyword { span }))
    }

    #[inline]
    pub fn ts_template_literal_type(
        self,
        span: Span,
        quasis: Vec<'a, TemplateElement<'a>>,
        types: Vec<'a, TSType<'a>>,
    ) -> TSType<'a> {
        TSType::TSTemplateLiteralType(self.alloc(TSTemplateLiteralType { span, quasis, types }))
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
    pub fn ts_conditional_type(
        self,
        span: Span,
        check_type: TSType<'a>,
        extends_type: TSType<'a>,
        true_type: TSType<'a>,
        false_type: TSType<'a>,
    ) -> TSType<'a> {
        TSType::TSConditionalType(self.alloc(TSConditionalType {
            span,
            check_type,
            extends_type,
            true_type,
            false_type,
        }))
    }

    #[inline]
    pub fn ts_mapped_type(
        self,
        span: Span,
        type_parameter: Box<'a, TSTypeParameter<'a>>,
        name_type: Option<TSType<'a>>,
        type_annotation: Option<TSType<'a>>,
        optional: TSMappedTypeModifierOperator,
        readonly: TSMappedTypeModifierOperator,
    ) -> TSType<'a> {
        TSType::TSMappedType(self.alloc(TSMappedType {
            span,
            type_parameter,
            name_type,
            type_annotation,
            optional,
            readonly,
        }))
    }

    #[inline]
    pub fn ts_import_type(
        self,
        span: Span,
        argument: TSType<'a>,
        qualifier: Option<TSTypeName<'a>>,
        attributes: Option<TSImportAttributes<'a>>,
        type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
    ) -> TSType<'a> {
        TSType::TSImportType(self.alloc(TSImportType {
            span,
            argument,
            qualifier,
            attributes,
            type_parameters,
        }))
    }

    #[inline]
    pub fn ts_constructor_type(
        self,
        span: Span,
        r#abstract: bool,
        params: Box<'a, FormalParameters<'a>>,
        return_type: Box<'a, TSTypeAnnotation<'a>>,
        type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
    ) -> TSType<'a> {
        TSType::TSConstructorType(self.alloc(TSConstructorType {
            span,
            r#abstract,
            params,
            return_type,
            type_parameters,
        }))
    }

    #[inline]
    pub fn ts_function_type(
        self,
        span: Span,
        this_param: Option<TSThisParameter<'a>>,
        params: Box<'a, FormalParameters<'a>>,
        return_type: Box<'a, TSTypeAnnotation<'a>>,
        type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
    ) -> TSType<'a> {
        TSType::TSFunctionType(self.alloc(TSFunctionType {
            span,
            this_param,
            params,
            return_type,
            type_parameters,
        }))
    }

    #[inline]
    pub fn ts_infer_type(
        self,
        span: Span,
        type_parameter: Box<'a, TSTypeParameter<'a>>,
    ) -> TSType<'a> {
        TSType::TSInferType(self.alloc(TSInferType { span, type_parameter }))
    }

    #[inline]
    pub fn ts_type_predicate(
        self,
        span: Span,
        parameter_name: TSTypePredicateName<'a>,
        asserts: bool,
        type_annotation: Option<Box<'a, TSTypeAnnotation<'a>>>,
    ) -> TSType<'a> {
        TSType::TSTypePredicate(self.alloc(TSTypePredicate {
            span,
            parameter_name,
            asserts,
            type_annotation,
        }))
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

    /* JSDoc */
    #[inline]
    pub fn js_doc_nullable_type(
        self,
        span: Span,
        type_annotation: TSType<'a>,
        postfix: bool,
    ) -> TSType<'a> {
        TSType::JSDocNullableType(self.alloc(JSDocNullableType { span, type_annotation, postfix }))
    }

    #[inline]
    pub fn js_doc_unknown_type(self, span: Span) -> TSType<'a> {
        TSType::JSDocUnknownType(self.alloc(JSDocUnknownType { span }))
    }
}
