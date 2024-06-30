// NB: `#[visited_node]` attribute on AST nodes does not do anything to the code in this file.
// It is purely a marker for codegen used in `oxc_traverse`. See docs in that crate.

use crate::ast::*;

use std::{cell::Cell, fmt, hash::Hash};

use oxc_allocator::{Box, Vec};
use oxc_span::{Atom, CompactStr, SourceType, Span};
use oxc_syntax::{
    operator::UnaryOperator,
    reference::{ReferenceFlag, ReferenceId},
    scope::ScopeFlags,
};

#[cfg(feature = "serialize")]
#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
export interface BindingIdentifier extends Span { type: "Identifier", name: Atom }
export interface IdentifierReference extends Span { type: "Identifier", name: Atom }
export interface IdentifierName extends Span { type: "Identifier", name: Atom }
export interface LabelIdentifier extends Span { type: "Identifier", name: Atom }
export interface AssignmentTargetRest extends Span { type: "RestElement", argument: AssignmentTarget }
export interface BindingRestElement extends Span { type: "RestElement", argument: BindingPattern }
export interface FormalParameterRest extends Span {
    type: "RestElement",
    argument: BindingPatternKind,
    typeAnnotation?: TSTypeAnnotation,
    optional: boolean,
}
"#;

impl<'a> Program<'a> {
    pub fn new(
        span: Span,
        source_type: SourceType,
        directives: Vec<'a, Directive<'a>>,
        hashbang: Option<Hashbang<'a>>,
        body: Vec<'a, Statement<'a>>,
    ) -> Self {
        Self { span, source_type, directives, hashbang, body, scope_id: Cell::default() }
    }
}

impl<'a> Hash for Program<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.source_type.hash(state);
        self.directives.hash(state);
        self.hashbang.hash(state);
        self.body.hash(state);
    }
}

impl<'a> Program<'a> {
    pub fn is_empty(&self) -> bool {
        self.body.is_empty() && self.directives.is_empty()
    }

    pub fn is_strict(&self) -> bool {
        self.source_type.is_strict() || self.directives.iter().any(Directive::is_use_strict)
    }
}

impl<'a> Expression<'a> {
    pub fn is_typescript_syntax(&self) -> bool {
        matches!(
            self,
            Self::TSAsExpression(_)
                | Self::TSSatisfiesExpression(_)
                | Self::TSTypeAssertion(_)
                | Self::TSNonNullExpression(_)
                | Self::TSInstantiationExpression(_)
        )
    }

    pub fn is_primary_expression(&self) -> bool {
        self.is_literal()
            || matches!(
                self,
                Self::Identifier(_)
                    | Self::ThisExpression(_)
                    | Self::FunctionExpression(_)
                    | Self::ClassExpression(_)
                    | Self::ParenthesizedExpression(_)
                    | Self::ArrayExpression(_)
                    | Self::ObjectExpression(_)
            )
    }

    pub fn is_literal(&self) -> bool {
        // Note: TemplateLiteral is not `Literal`
        matches!(
            self,
            Self::BooleanLiteral(_)
                | Self::NullLiteral(_)
                | Self::NumericLiteral(_)
                | Self::BigIntLiteral(_)
                | Self::RegExpLiteral(_)
                | Self::StringLiteral(_)
        )
    }

    pub fn is_string_literal(&self) -> bool {
        matches!(self, Self::StringLiteral(_) | Self::TemplateLiteral(_))
    }

    pub fn is_number_literal(&self) -> bool {
        matches!(self, Self::NumericLiteral(_) | Self::BigIntLiteral(_))
    }

    pub fn is_specific_string_literal(&self, string: &str) -> bool {
        match self {
            Self::StringLiteral(s) => s.value == string,
            _ => false,
        }
    }

    /// Determines whether the given expr is a `null` literal
    pub fn is_null(&self) -> bool {
        matches!(self, Expression::NullLiteral(_))
    }

    /// Determines whether the given expr is a `undefined` literal
    pub fn is_undefined(&self) -> bool {
        matches!(self, Self::Identifier(ident) if ident.name == "undefined")
    }

    /// Determines whether the given expr is a `void expr`
    pub fn is_void(&self) -> bool {
        matches!(self, Self::UnaryExpression(expr) if expr.operator == UnaryOperator::Void)
    }

    /// Determines whether the given expr is a `void 0`
    pub fn is_void_0(&self) -> bool {
        match self {
            Self::UnaryExpression(expr) if expr.operator == UnaryOperator::Void => {
                matches!(&expr.argument, Self::NumericLiteral(lit) if lit.value == 0.0)
            }
            _ => false,
        }
    }

    /// Determines whether the given expr is a `0`
    pub fn is_number_0(&self) -> bool {
        matches!(self, Self::NumericLiteral(lit) if lit.value == 0.0)
    }

    pub fn is_number(&self, val: f64) -> bool {
        matches!(self, Self::NumericLiteral(lit) if (lit.value - val).abs() < f64::EPSILON)
    }

    /// Determines whether the given numeral literal's raw value is exactly val
    pub fn is_specific_raw_number_literal(&self, val: &str) -> bool {
        matches!(self, Self::NumericLiteral(lit) if lit.raw == val)
    }

    /// Determines whether the given expr evaluate to `undefined`
    pub fn evaluate_to_undefined(&self) -> bool {
        self.is_undefined() || self.is_void()
    }

    /// Determines whether the given expr is a `null` or `undefined` or `void 0`
    pub fn is_null_or_undefined(&self) -> bool {
        self.is_null() || self.evaluate_to_undefined()
    }

    /// Determines whether the given expr is a `NaN` literal
    pub fn is_nan(&self) -> bool {
        matches!(self, Self::Identifier(ident) if ident.name == "NaN")
    }

    /// Remove nested parentheses from this expression.
    pub fn without_parenthesized(&self) -> &Self {
        match self {
            Expression::ParenthesizedExpression(expr) => expr.expression.without_parenthesized(),
            _ => self,
        }
    }

    pub fn is_specific_id(&self, name: &str) -> bool {
        match self.get_inner_expression() {
            Expression::Identifier(ident) => ident.name == name,
            _ => false,
        }
    }

    pub fn is_specific_member_access(&self, object: &str, property: &str) -> bool {
        match self.get_inner_expression() {
            expr if expr.is_member_expression() => {
                expr.to_member_expression().is_specific_member_access(object, property)
            }
            Expression::ChainExpression(chain) => {
                let Some(expr) = chain.expression.as_member_expression() else {
                    return false;
                };
                expr.is_specific_member_access(object, property)
            }
            _ => false,
        }
    }

    pub fn get_inner_expression(&self) -> &Expression<'a> {
        match self {
            Expression::ParenthesizedExpression(expr) => expr.expression.get_inner_expression(),
            Expression::TSAsExpression(expr) => expr.expression.get_inner_expression(),
            Expression::TSSatisfiesExpression(expr) => expr.expression.get_inner_expression(),
            Expression::TSInstantiationExpression(expr) => expr.expression.get_inner_expression(),
            Expression::TSNonNullExpression(expr) => expr.expression.get_inner_expression(),
            Expression::TSTypeAssertion(expr) => expr.expression.get_inner_expression(),
            _ => self,
        }
    }

    pub fn is_identifier_reference(&self) -> bool {
        matches!(self, Expression::Identifier(_))
    }

    pub fn get_identifier_reference(&self) -> Option<&IdentifierReference<'a>> {
        match self.get_inner_expression() {
            Expression::Identifier(ident) => Some(ident),
            _ => None,
        }
    }

    pub fn is_function(&self) -> bool {
        matches!(self, Expression::FunctionExpression(_) | Expression::ArrowFunctionExpression(_))
    }

    pub fn is_call_expression(&self) -> bool {
        matches!(self, Expression::CallExpression(_))
    }

    pub fn is_super_call_expression(&self) -> bool {
        matches!(self, Expression::CallExpression(expr) if matches!(&expr.callee, Expression::Super(_)))
    }

    pub fn is_call_like_expression(&self) -> bool {
        self.is_call_expression()
            && matches!(self, Expression::NewExpression(_) | Expression::ImportExpression(_))
    }

    pub fn is_binaryish(&self) -> bool {
        matches!(self, Expression::BinaryExpression(_) | Expression::LogicalExpression(_))
    }

    /// Returns literal's value converted to the Boolean type
    /// returns `true` when node is truthy, `false` when node is falsy, `None` when it cannot be determined.
    pub fn get_boolean_value(&self) -> Option<bool> {
        match self {
            Self::BooleanLiteral(lit) => Some(lit.value),
            Self::NullLiteral(_) => Some(false),
            Self::NumericLiteral(lit) => Some(lit.value != 0.0),
            Self::BigIntLiteral(lit) => Some(!lit.is_zero()),
            Self::RegExpLiteral(_) => Some(true),
            Self::StringLiteral(lit) => Some(!lit.value.is_empty()),
            _ => None,
        }
    }

    pub fn get_member_expr(&self) -> Option<&MemberExpression<'a>> {
        match self.get_inner_expression() {
            Expression::ChainExpression(chain_expr) => chain_expr.expression.as_member_expression(),
            expr => expr.as_member_expression(),
        }
    }

    pub fn is_immutable_value(&self) -> bool {
        match self {
            Self::BooleanLiteral(_)
            | Self::NullLiteral(_)
            | Self::NumericLiteral(_)
            | Self::BigIntLiteral(_)
            | Self::RegExpLiteral(_)
            | Self::StringLiteral(_) => true,
            Self::TemplateLiteral(lit) if lit.is_no_substitution_template() => true,
            Self::UnaryExpression(unary_expr) => unary_expr.argument.is_immutable_value(),
            Self::Identifier(ident) => {
                matches!(ident.name.as_str(), "undefined" | "Infinity" | "NaN")
            }
            _ => false,
        }
    }
}

impl<'a> IdentifierName<'a> {
    pub fn new(span: Span, name: Atom<'a>) -> Self {
        Self { span, name }
    }
}

impl<'a> Hash for IdentifierReference<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl<'a> IdentifierReference<'a> {
    pub fn new(span: Span, name: Atom<'a>) -> Self {
        Self { span, name, reference_id: Cell::default(), reference_flag: ReferenceFlag::default() }
    }

    pub fn new_read(span: Span, name: Atom<'a>, reference_id: Option<ReferenceId>) -> Self {
        Self {
            span,
            name,
            reference_id: Cell::new(reference_id),
            reference_flag: ReferenceFlag::Read,
        }
    }
}

impl<'a> Hash for BindingIdentifier<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl<'a> BindingIdentifier<'a> {
    pub fn new(span: Span, name: Atom<'a>) -> Self {
        Self { span, name, symbol_id: Cell::default() }
    }
}

impl<'a> ArrayExpressionElement<'a> {
    pub fn is_elision(&self) -> bool {
        matches!(self, Self::Elision(_))
    }
}

impl<'a> ObjectExpression<'a> {
    pub fn has_proto(&self) -> bool {
        use crate::syntax_directed_operations::PropName;
        self.properties.iter().any(|p| p.prop_name().is_some_and(|name| name.0 == "__proto__"))
    }
}

impl<'a> PropertyKey<'a> {
    pub fn static_name(&self) -> Option<CompactStr> {
        match self {
            Self::StaticIdentifier(ident) => Some(ident.name.to_compact_str()),
            Self::StringLiteral(lit) => Some(lit.value.to_compact_str()),
            Self::RegExpLiteral(lit) => Some(lit.regex.to_string().into()),
            Self::NumericLiteral(lit) => Some(lit.value.to_string().into()),
            Self::BigIntLiteral(lit) => Some(lit.raw.to_compact_str()),
            Self::NullLiteral(_) => Some("null".into()),
            Self::TemplateLiteral(lit) => lit
                .expressions
                .is_empty()
                .then(|| lit.quasi())
                .flatten()
                .map(|quasi| quasi.to_compact_str()),
            _ => None,
        }
    }

    pub fn is_specific_static_name(&self, name: &str) -> bool {
        self.static_name().is_some_and(|n| n == name)
    }

    pub fn is_identifier(&self) -> bool {
        matches!(self, Self::PrivateIdentifier(_) | Self::StaticIdentifier(_))
    }

    pub fn is_private_identifier(&self) -> bool {
        matches!(self, Self::PrivateIdentifier(_))
    }

    pub fn private_name(&self) -> Option<Atom<'a>> {
        match self {
            Self::PrivateIdentifier(ident) => Some(ident.name.clone()),
            _ => None,
        }
    }

    pub fn name(&self) -> Option<CompactStr> {
        if self.is_private_identifier() {
            self.private_name().map(|name| name.to_compact_str())
        } else {
            self.static_name()
        }
    }

    pub fn is_specific_id(&self, name: &str) -> bool {
        match self {
            PropertyKey::StaticIdentifier(ident) => ident.name == name,
            _ => false,
        }
    }

    pub fn is_specific_string_literal(&self, string: &str) -> bool {
        matches!(self, Self::StringLiteral(s) if s.value == string)
    }
}

impl<'a> TemplateLiteral<'a> {
    pub fn is_no_substitution_template(&self) -> bool {
        self.expressions.is_empty() && self.quasis.len() == 1
    }

    /// Get single quasi from `template`
    pub fn quasi(&self) -> Option<Atom<'a>> {
        self.quasis.first().and_then(|quasi| quasi.value.cooked.clone())
    }
}

impl<'a> MemberExpression<'a> {
    pub fn is_computed(&self) -> bool {
        matches!(self, MemberExpression::ComputedMemberExpression(_))
    }

    pub fn optional(&self) -> bool {
        match self {
            MemberExpression::ComputedMemberExpression(expr) => expr.optional,
            MemberExpression::StaticMemberExpression(expr) => expr.optional,
            MemberExpression::PrivateFieldExpression(expr) => expr.optional,
        }
    }

    pub fn object(&self) -> &Expression<'a> {
        match self {
            MemberExpression::ComputedMemberExpression(expr) => &expr.object,
            MemberExpression::StaticMemberExpression(expr) => &expr.object,
            MemberExpression::PrivateFieldExpression(expr) => &expr.object,
        }
    }

    pub fn static_property_name(&self) -> Option<&str> {
        match self {
            MemberExpression::ComputedMemberExpression(expr) => {
                expr.static_property_name().map(|name| name.as_str())
            }
            MemberExpression::StaticMemberExpression(expr) => Some(expr.property.name.as_str()),
            MemberExpression::PrivateFieldExpression(_) => None,
        }
    }

    pub fn static_property_info(&self) -> Option<(Span, &str)> {
        match self {
            MemberExpression::ComputedMemberExpression(expr) => match &expr.expression {
                Expression::StringLiteral(lit) => Some((lit.span, &lit.value)),
                Expression::TemplateLiteral(lit) => {
                    if lit.expressions.is_empty() && lit.quasis.len() == 1 {
                        Some((lit.span, &lit.quasis[0].value.raw))
                    } else {
                        None
                    }
                }
                _ => None,
            },
            MemberExpression::StaticMemberExpression(expr) => {
                Some((expr.property.span, &expr.property.name))
            }
            MemberExpression::PrivateFieldExpression(_) => None,
        }
    }

    pub fn through_optional_is_specific_member_access(&self, object: &str, property: &str) -> bool {
        let object_matches = match self.object().without_parenthesized() {
            Expression::ChainExpression(x) => match &x.expression {
                ChainElement::CallExpression(_) => false,
                match_member_expression!(ChainElement) => {
                    let member_expr = x.expression.to_member_expression();
                    member_expr.object().without_parenthesized().is_specific_id(object)
                }
            },
            x => x.is_specific_id(object),
        };

        let property_matches = self.static_property_name().is_some_and(|p| p == property);

        object_matches && property_matches
    }

    /// Whether it is a static member access `object.property`
    pub fn is_specific_member_access(&self, object: &str, property: &str) -> bool {
        self.object().is_specific_id(object)
            && self.static_property_name().is_some_and(|p| p == property)
    }
}

impl<'a> ComputedMemberExpression<'a> {
    pub fn static_property_name(&self) -> Option<Atom<'a>> {
        match &self.expression {
            Expression::StringLiteral(lit) => Some(lit.value.clone()),
            Expression::TemplateLiteral(lit)
                if lit.expressions.is_empty() && lit.quasis.len() == 1 =>
            {
                Some(lit.quasis[0].value.raw.clone())
            }
            _ => None,
        }
    }
}

impl<'a> StaticMemberExpression<'a> {
    pub fn get_first_object(&self) -> &Expression<'a> {
        match &self.object {
            Expression::StaticMemberExpression(member) => {
                if let Expression::StaticMemberExpression(expr) = &member.object {
                    expr.get_first_object()
                } else {
                    &self.object
                }
            }
            Expression::ChainExpression(chain) => {
                if let ChainElement::StaticMemberExpression(expr) = &chain.expression {
                    expr.get_first_object()
                } else {
                    &self.object
                }
            }
            _ => &self.object,
        }
    }
}

impl<'a> CallExpression<'a> {
    pub fn callee_name(&self) -> Option<&str> {
        match &self.callee {
            Expression::Identifier(ident) => Some(ident.name.as_str()),
            expr => expr.as_member_expression().and_then(MemberExpression::static_property_name),
        }
    }

    pub fn is_require_call(&self) -> bool {
        if self.arguments.len() != 1 {
            return false;
        }
        if let Expression::Identifier(id) = &self.callee {
            id.name == "require"
                && matches!(
                    self.arguments.first(),
                    Some(Argument::StringLiteral(_) | Argument::TemplateLiteral(_)),
                )
        } else {
            false
        }
    }

    pub fn is_symbol_or_symbol_for_call(&self) -> bool {
        // TODO: is 'Symbol' reference to global object
        match &self.callee {
            Expression::Identifier(id) => id.name == "Symbol",
            expr => match expr.as_member_expression() {
                Some(member) => {
                    matches!(member.object(), Expression::Identifier(id) if id.name == "Symbol")
                        && member.static_property_name() == Some("for")
                }
                None => false,
            },
        }
    }

    pub fn common_js_require(&self) -> Option<&StringLiteral> {
        if !(self.callee.is_specific_id("require") && self.arguments.len() == 1) {
            return None;
        }
        match &self.arguments[0] {
            Argument::StringLiteral(str_literal) => Some(str_literal),
            _ => None,
        }
    }
}

impl Argument<'_> {
    pub fn is_spread(&self) -> bool {
        matches!(self, Self::SpreadElement(_))
    }
}

impl<'a> AssignmentTarget<'a> {
    pub fn get_identifier(&self) -> Option<&str> {
        self.as_simple_assignment_target().and_then(|it| it.get_identifier())
    }

    pub fn get_expression(&self) -> Option<&Expression<'a>> {
        self.as_simple_assignment_target().and_then(|it| it.get_expression())
    }
}

impl<'a> SimpleAssignmentTarget<'a> {
    pub fn get_identifier(&self) -> Option<&str> {
        match self {
            Self::AssignmentTargetIdentifier(ident) => Some(ident.name.as_str()),
            match_member_expression!(Self) => self.to_member_expression().static_property_name(),
            _ => None,
        }
    }

    pub fn get_expression(&self) -> Option<&Expression<'a>> {
        match self {
            Self::TSAsExpression(expr) => Some(&expr.expression),
            Self::TSSatisfiesExpression(expr) => Some(&expr.expression),
            Self::TSNonNullExpression(expr) => Some(&expr.expression),
            Self::TSTypeAssertion(expr) => Some(&expr.expression),
            _ => None,
        }
    }
}

impl<'a> ArrayAssignmentTarget<'a> {
    pub fn new_with_elements(
        span: Span,
        elements: Vec<'a, Option<AssignmentTargetMaybeDefault<'a>>>,
    ) -> Self {
        Self { span, elements, rest: None, trailing_comma: None }
    }
}

impl<'a> ObjectAssignmentTarget<'a> {
    pub fn new_with_properties(
        span: Span,
        properties: Vec<'a, AssignmentTargetProperty<'a>>,
    ) -> Self {
        Self { span, properties, rest: None }
    }

    pub fn is_empty(&self) -> bool {
        self.properties.is_empty() && self.rest.is_none()
    }

    pub fn len(&self) -> usize {
        self.properties.len() + usize::from(self.rest.is_some())
    }
}

impl<'a> AssignmentTargetMaybeDefault<'a> {
    pub fn name(&self) -> Option<Atom> {
        match self {
            AssignmentTargetMaybeDefault::AssignmentTargetIdentifier(id) => Some(id.name.clone()),
            Self::AssignmentTargetWithDefault(target) => {
                if let AssignmentTarget::AssignmentTargetIdentifier(id) = &target.binding {
                    Some(id.name.clone())
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl<'a> Statement<'a> {
    pub fn is_typescript_syntax(&self) -> bool {
        match self {
            match_declaration!(Self) => {
                self.as_declaration().is_some_and(Declaration::is_typescript_syntax)
            }
            match_module_declaration!(Self) => {
                self.as_module_declaration().is_some_and(ModuleDeclaration::is_typescript_syntax)
            }
            _ => false,
        }
    }

    pub fn is_iteration_statement(&self) -> bool {
        matches!(
            self,
            Statement::DoWhileStatement(_)
                | Statement::ForInStatement(_)
                | Statement::ForOfStatement(_)
                | Statement::ForStatement(_)
                | Statement::WhileStatement(_)
        )
    }
}

impl<'a> Directive<'a> {
    /// A Use Strict Directive is an ExpressionStatement in a Directive Prologue whose StringLiteral is either of the exact code point sequences "use strict" or 'use strict'.
    /// A Use Strict Directive may not contain an EscapeSequence or LineContinuation.
    /// <https://tc39.es/ecma262/#sec-directive-prologues-and-the-use-strict-directive>
    pub fn is_use_strict(&self) -> bool {
        self.directive == "use strict"
    }
}

impl<'a> BlockStatement<'a> {
    pub fn new(span: Span, body: Vec<'a, Statement<'a>>) -> Self {
        Self { span, body, scope_id: Cell::default() }
    }
}

impl<'a> Hash for BlockStatement<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.body.hash(state);
    }
}

impl<'a> Declaration<'a> {
    pub fn is_typescript_syntax(&self) -> bool {
        match self {
            Self::VariableDeclaration(decl) => decl.is_typescript_syntax(),
            Self::FunctionDeclaration(func) => func.is_typescript_syntax(),
            Self::ClassDeclaration(class) => class.is_typescript_syntax(),
            Self::UsingDeclaration(_) => false,
            _ => true,
        }
    }

    pub fn id(&self) -> Option<&BindingIdentifier<'a>> {
        match self {
            Declaration::FunctionDeclaration(decl) => decl.id.as_ref(),
            Declaration::ClassDeclaration(decl) => decl.id.as_ref(),
            Declaration::TSTypeAliasDeclaration(decl) => Some(&decl.id),
            Declaration::TSInterfaceDeclaration(decl) => Some(&decl.id),
            Declaration::TSEnumDeclaration(decl) => Some(&decl.id),
            Declaration::TSImportEqualsDeclaration(decl) => Some(&decl.id),
            _ => None,
        }
    }

    pub fn declare(&self) -> bool {
        match self {
            Declaration::VariableDeclaration(decl) => decl.declare,
            Declaration::FunctionDeclaration(decl) => decl.declare,
            Declaration::ClassDeclaration(decl) => decl.declare,
            Declaration::TSEnumDeclaration(decl) => decl.declare,
            Declaration::TSTypeAliasDeclaration(decl) => decl.declare,
            Declaration::TSModuleDeclaration(decl) => decl.declare,
            Declaration::TSInterfaceDeclaration(decl) => decl.declare,
            _ => false,
        }
    }
}

impl<'a> VariableDeclaration<'a> {
    pub fn is_typescript_syntax(&self) -> bool {
        self.declare
    }

    pub fn has_init(&self) -> bool {
        self.declarations.iter().any(|decl| decl.init.is_some())
    }
}

impl VariableDeclarationKind {
    pub fn is_var(&self) -> bool {
        matches!(self, Self::Var)
    }

    pub fn is_const(&self) -> bool {
        matches!(self, Self::Const)
    }

    pub fn is_lexical(&self) -> bool {
        matches!(self, Self::Const | Self::Let)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Var => "var",
            Self::Const => "const",
            Self::Let => "let",
        }
    }
}

impl fmt::Display for VariableDeclarationKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = self.as_str();
        write!(f, "{s}")
    }
}

impl<'a> ForStatement<'a> {
    pub fn new(
        span: Span,
        init: Option<ForStatementInit<'a>>,
        test: Option<Expression<'a>>,
        update: Option<Expression<'a>>,
        body: Statement<'a>,
    ) -> Self {
        Self { span, init, test, update, body, scope_id: Cell::default() }
    }
}

impl<'a> Hash for ForStatement<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.init.hash(state);
        self.test.hash(state);
        self.update.hash(state);
        self.body.hash(state);
    }
}

impl<'a> ForStatementInit<'a> {
    /// LexicalDeclaration[In, Yield, Await] :
    ///   LetOrConst BindingList[?In, ?Yield, ?Await] ;
    pub fn is_lexical_declaration(&self) -> bool {
        matches!(self, Self::VariableDeclaration(decl) if decl.kind.is_lexical())
    }
}

impl<'a> ForInStatement<'a> {
    pub fn new(
        span: Span,
        left: ForStatementLeft<'a>,
        right: Expression<'a>,
        body: Statement<'a>,
    ) -> Self {
        Self { span, left, right, body, scope_id: Cell::default() }
    }
}

impl<'a> Hash for ForInStatement<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.left.hash(state);
        self.right.hash(state);
        self.body.hash(state);
    }
}

impl<'a> ForOfStatement<'a> {
    pub fn new(
        span: Span,
        r#await: bool,
        left: ForStatementLeft<'a>,
        right: Expression<'a>,
        body: Statement<'a>,
    ) -> Self {
        Self { span, r#await, left, right, body, scope_id: Cell::default() }
    }
}

impl<'a> Hash for ForOfStatement<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.r#await.hash(state);
        self.left.hash(state);
        self.right.hash(state);
        self.body.hash(state);
    }
}

impl<'a> ForStatementLeft<'a> {
    /// LexicalDeclaration[In, Yield, Await] :
    ///   LetOrConst BindingList[?In, ?Yield, ?Await] ;
    pub fn is_lexical_declaration(&self) -> bool {
        matches!(self, Self::VariableDeclaration(decl) if decl.kind.is_lexical())
    }
}

impl<'a> SwitchStatement<'a> {
    pub fn new(span: Span, discriminant: Expression<'a>, cases: Vec<'a, SwitchCase<'a>>) -> Self {
        Self { span, discriminant, cases, scope_id: Cell::default() }
    }
}

impl<'a> Hash for SwitchStatement<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.discriminant.hash(state);
        self.cases.hash(state);
    }
}

impl<'a> SwitchCase<'a> {
    pub fn is_default_case(&self) -> bool {
        self.test.is_none()
    }
}

impl<'a> CatchClause<'a> {
    pub fn new(
        span: Span,
        param: Option<CatchParameter<'a>>,
        body: Box<'a, BlockStatement<'a>>,
    ) -> Self {
        Self { span, param, body, scope_id: Cell::default() }
    }
}

impl<'a> Hash for CatchClause<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.param.hash(state);
        self.body.hash(state);
    }
}

impl<'a> BindingPattern<'a> {
    pub fn new_with_kind(kind: BindingPatternKind<'a>) -> Self {
        Self { kind, type_annotation: None, optional: false }
    }

    pub fn get_identifier(&self) -> Option<Atom<'a>> {
        self.kind.get_identifier()
    }

    pub fn get_binding_identifier(&self) -> Option<&BindingIdentifier<'a>> {
        self.kind.get_binding_identifier()
    }
}

impl<'a> BindingPatternKind<'a> {
    pub fn get_identifier(&self) -> Option<Atom<'a>> {
        match self {
            Self::BindingIdentifier(ident) => Some(ident.name.clone()),
            Self::AssignmentPattern(assign) => assign.left.get_identifier(),
            _ => None,
        }
    }

    pub fn get_binding_identifier(&self) -> Option<&BindingIdentifier<'a>> {
        match self {
            Self::BindingIdentifier(ident) => Some(ident),
            Self::AssignmentPattern(assign) => assign.left.get_binding_identifier(),
            _ => None,
        }
    }

    pub fn is_destructuring_pattern(&self) -> bool {
        match self {
            Self::ObjectPattern(_) | Self::ArrayPattern(_) => true,
            Self::AssignmentPattern(pattern) => pattern.left.kind.is_destructuring_pattern(),
            Self::BindingIdentifier(_) => false,
        }
    }

    pub fn is_binding_identifier(&self) -> bool {
        matches!(self, Self::BindingIdentifier(_))
    }

    pub fn is_assignment_pattern(&self) -> bool {
        matches!(self, Self::AssignmentPattern(_))
    }
}

impl<'a> ObjectPattern<'a> {
    pub fn is_empty(&self) -> bool {
        self.properties.is_empty() && self.rest.is_none()
    }

    pub fn len(&self) -> usize {
        self.properties.len() + usize::from(self.rest.is_some())
    }
}

impl<'a> ArrayPattern<'a> {
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty() && self.rest.is_none()
    }

    pub fn len(&self) -> usize {
        self.elements.len() + usize::from(self.rest.is_some())
    }
}

impl<'a> Function<'a> {
    #![allow(clippy::too_many_arguments)]
    pub fn new(
        r#type: FunctionType,
        span: Span,
        id: Option<BindingIdentifier<'a>>,
        generator: bool,
        r#async: bool,
        declare: bool,
        this_param: Option<TSThisParameter<'a>>,
        params: Box<'a, FormalParameters<'a>>,
        body: Option<Box<'a, FunctionBody<'a>>>,
        type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
        return_type: Option<Box<'a, TSTypeAnnotation<'a>>>,
    ) -> Self {
        Self {
            r#type,
            span,
            id,
            generator,
            r#async,
            declare,
            this_param,
            params,
            body,
            type_parameters,
            return_type,
            scope_id: Cell::default(),
        }
    }

    pub fn is_typescript_syntax(&self) -> bool {
        matches!(
            self.r#type,
            FunctionType::TSDeclareFunction | FunctionType::TSEmptyBodyFunctionExpression
        ) || self.body.is_none()
            || self.declare
    }

    pub fn is_expression(&self) -> bool {
        self.r#type == FunctionType::FunctionExpression
    }

    pub fn is_function_declaration(&self) -> bool {
        matches!(self.r#type, FunctionType::FunctionDeclaration)
    }

    pub fn is_ts_declare_function(&self) -> bool {
        matches!(self.r#type, FunctionType::TSDeclareFunction)
    }

    pub fn is_declaration(&self) -> bool {
        matches!(self.r#type, FunctionType::FunctionDeclaration | FunctionType::TSDeclareFunction)
    }

    pub fn is_strict(&self) -> bool {
        self.body.as_ref().is_some_and(|body| body.has_use_strict_directive())
    }
}

impl<'a> Hash for Function<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.r#type.hash(state);
        self.id.hash(state);
        self.generator.hash(state);
        self.r#async.hash(state);
        self.declare.hash(state);
        self.this_param.hash(state);
        self.params.hash(state);
        self.body.hash(state);
        self.type_parameters.hash(state);
        self.return_type.hash(state);
    }
}

impl<'a> FormalParameters<'a> {
    pub fn parameters_count(&self) -> usize {
        self.items.len() + self.rest.as_ref().map_or(0, |_| 1)
    }

    /// Iterates over all bound parameters, including rest parameters.
    pub fn iter_bindings(&self) -> impl Iterator<Item = &BindingPattern<'a>> + '_ {
        self.items
            .iter()
            .map(|param| &param.pattern)
            .chain(self.rest.iter().map(|rest| &rest.argument))
    }
}

impl<'a> FormalParameter<'a> {
    pub fn is_public(&self) -> bool {
        matches!(self.accessibility, Some(TSAccessibility::Public))
    }
}

impl FormalParameterKind {
    pub fn is_signature(&self) -> bool {
        matches!(self, Self::Signature)
    }
}

impl<'a> FormalParameters<'a> {
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

impl<'a> FunctionBody<'a> {
    pub fn is_empty(&self) -> bool {
        self.directives.is_empty() && self.statements.is_empty()
    }

    pub fn has_use_strict_directive(&self) -> bool {
        self.directives.iter().any(Directive::is_use_strict)
    }
}

impl<'a> ArrowFunctionExpression<'a> {
    pub fn new(
        span: Span,
        expression: bool,
        r#async: bool,
        params: Box<'a, FormalParameters<'a>>,
        body: Box<'a, FunctionBody<'a>>,
        type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
        return_type: Option<Box<'a, TSTypeAnnotation<'a>>>,
    ) -> Self {
        Self {
            span,
            expression,
            r#async,
            params,
            body,
            type_parameters,
            return_type,
            scope_id: Cell::default(),
        }
    }

    /// Get expression part of `ArrowFunctionExpression`: `() => expression_part`.
    pub fn get_expression(&self) -> Option<&Expression<'a>> {
        if self.expression {
            if let Statement::ExpressionStatement(expr_stmt) = &self.body.statements[0] {
                return Some(&expr_stmt.expression);
            }
        }
        None
    }
}

impl<'a> Hash for ArrowFunctionExpression<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.expression.hash(state);
        self.r#async.hash(state);
        self.params.hash(state);
        self.body.hash(state);
        self.type_parameters.hash(state);
        self.return_type.hash(state);
    }
}

impl<'a> Class<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
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
    ) -> Self {
        Self {
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
            scope_id: Cell::default(),
        }
    }

    /// `true` if this [`Class`] is an expression.
    ///
    /// For example,
    /// ```ts
    /// var Foo = class { /* ... */ }
    /// ```
    pub fn is_expression(&self) -> bool {
        self.r#type == ClassType::ClassExpression
    }

    /// `true` if this [`Class`] is a declaration statement.
    ///
    /// For example,
    /// ```ts
    /// class Foo {
    ///   // ...
    /// }
    /// ```
    pub fn is_declaration(&self) -> bool {
        self.r#type == ClassType::ClassDeclaration
    }

    pub fn is_typescript_syntax(&self) -> bool {
        self.declare || self.r#abstract
    }
}

impl<'a> Hash for Class<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.r#type.hash(state);
        self.decorators.hash(state);
        self.id.hash(state);
        self.super_class.hash(state);
        self.body.hash(state);
        self.type_parameters.hash(state);
        self.super_type_parameters.hash(state);
        self.implements.hash(state);
        self.r#abstract.hash(state);
        self.declare.hash(state);
    }
}

impl<'a> ClassElement<'a> {
    pub fn r#static(&self) -> bool {
        match self {
            Self::TSIndexSignature(_) | Self::StaticBlock(_) => false,
            Self::MethodDefinition(def) => def.r#static,
            Self::PropertyDefinition(def) => def.r#static,
            Self::AccessorProperty(def) => def.r#static,
        }
    }

    pub fn computed(&self) -> bool {
        match self {
            Self::TSIndexSignature(_) | Self::StaticBlock(_) => false,
            Self::MethodDefinition(def) => def.computed,
            Self::PropertyDefinition(def) => def.computed,
            Self::AccessorProperty(def) => def.computed,
        }
    }

    pub fn accessibility(&self) -> Option<TSAccessibility> {
        match self {
            Self::StaticBlock(_) | Self::TSIndexSignature(_) | Self::AccessorProperty(_) => None,
            Self::MethodDefinition(def) => def.accessibility,
            Self::PropertyDefinition(def) => def.accessibility,
        }
    }

    pub fn method_definition_kind(&self) -> Option<MethodDefinitionKind> {
        match self {
            Self::TSIndexSignature(_)
            | Self::StaticBlock(_)
            | Self::PropertyDefinition(_)
            | Self::AccessorProperty(_) => None,
            Self::MethodDefinition(def) => Some(def.kind),
        }
    }

    pub fn property_key(&self) -> Option<&PropertyKey<'a>> {
        match self {
            Self::TSIndexSignature(_) | Self::StaticBlock(_) => None,
            Self::MethodDefinition(def) => Some(&def.key),
            Self::PropertyDefinition(def) => Some(&def.key),
            Self::AccessorProperty(def) => Some(&def.key),
        }
    }

    pub fn static_name(&self) -> Option<CompactStr> {
        match self {
            Self::TSIndexSignature(_) | Self::StaticBlock(_) => None,
            Self::MethodDefinition(def) => def.key.static_name(),
            Self::PropertyDefinition(def) => def.key.static_name(),
            Self::AccessorProperty(def) => def.key.static_name(),
        }
    }

    pub fn is_property(&self) -> bool {
        matches!(self, Self::PropertyDefinition(_) | Self::AccessorProperty(_))
    }

    pub fn is_ts_empty_body_function(&self) -> bool {
        match self {
            Self::PropertyDefinition(_)
            | Self::StaticBlock(_)
            | Self::AccessorProperty(_)
            | Self::TSIndexSignature(_) => false,
            Self::MethodDefinition(method) => method.value.body.is_none(),
        }
    }

    pub fn is_typescript_syntax(&self) -> bool {
        match self {
            Self::TSIndexSignature(_) => true,
            Self::MethodDefinition(method) => method.value.is_typescript_syntax(),
            Self::PropertyDefinition(property) => {
                property.r#type == PropertyDefinitionType::TSAbstractPropertyDefinition
            }
            Self::AccessorProperty(property) => property.r#type.is_abstract(),
            Self::StaticBlock(_) => false,
        }
    }

    pub fn has_decorator(&self) -> bool {
        match self {
            Self::MethodDefinition(method) => !method.decorators.is_empty(),
            Self::PropertyDefinition(property) => !property.decorators.is_empty(),
            Self::AccessorProperty(property) => !property.decorators.is_empty(),
            Self::StaticBlock(_) | Self::TSIndexSignature(_) => false,
        }
    }
}

impl MethodDefinitionKind {
    pub fn is_constructor(&self) -> bool {
        matches!(self, Self::Constructor)
    }

    pub fn is_method(&self) -> bool {
        matches!(self, Self::Method)
    }

    pub fn is_set(&self) -> bool {
        matches!(self, Self::Set)
    }

    pub fn is_get(&self) -> bool {
        matches!(self, Self::Get)
    }

    pub fn scope_flags(self) -> ScopeFlags {
        match self {
            Self::Constructor => ScopeFlags::Constructor | ScopeFlags::Function,
            Self::Method => ScopeFlags::Function,
            Self::Get => ScopeFlags::GetAccessor | ScopeFlags::Function,
            Self::Set => ScopeFlags::SetAccessor | ScopeFlags::Function,
        }
    }
}

impl<'a> PrivateIdentifier<'a> {
    pub fn new(span: Span, name: Atom<'a>) -> Self {
        Self { span, name }
    }
}

impl<'a> StaticBlock<'a> {
    pub fn new(span: Span, body: Vec<'a, Statement<'a>>) -> Self {
        Self { span, body, scope_id: Cell::default() }
    }
}

impl<'a> Hash for StaticBlock<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.body.hash(state);
    }
}

impl<'a> ModuleDeclaration<'a> {
    pub fn is_typescript_syntax(&self) -> bool {
        match self {
            ModuleDeclaration::ImportDeclaration(_) => false,
            ModuleDeclaration::ExportDefaultDeclaration(decl) => decl.is_typescript_syntax(),
            ModuleDeclaration::ExportNamedDeclaration(decl) => decl.is_typescript_syntax(),
            ModuleDeclaration::ExportAllDeclaration(decl) => decl.is_typescript_syntax(),
            ModuleDeclaration::TSNamespaceExportDeclaration(_)
            | ModuleDeclaration::TSExportAssignment(_) => true,
        }
    }

    pub fn is_import(&self) -> bool {
        matches!(self, Self::ImportDeclaration(_))
    }

    pub fn is_export(&self) -> bool {
        matches!(
            self,
            Self::ExportAllDeclaration(_)
                | Self::ExportDefaultDeclaration(_)
                | Self::ExportNamedDeclaration(_)
                | Self::TSExportAssignment(_)
                | Self::TSNamespaceExportDeclaration(_)
        )
    }

    pub fn is_default_export(&self) -> bool {
        matches!(self, Self::ExportDefaultDeclaration(_))
    }

    pub fn source(&self) -> Option<&StringLiteral<'a>> {
        match self {
            Self::ImportDeclaration(decl) => Some(&decl.source),
            Self::ExportAllDeclaration(decl) => Some(&decl.source),
            Self::ExportNamedDeclaration(decl) => decl.source.as_ref(),
            Self::ExportDefaultDeclaration(_)
            | Self::TSExportAssignment(_)
            | Self::TSNamespaceExportDeclaration(_) => None,
        }
    }

    pub fn with_clause(&self) -> Option<&WithClause<'a>> {
        match self {
            Self::ImportDeclaration(decl) => decl.with_clause.as_ref(),
            Self::ExportAllDeclaration(decl) => decl.with_clause.as_ref(),
            Self::ExportNamedDeclaration(decl) => decl.with_clause.as_ref(),
            Self::ExportDefaultDeclaration(_)
            | Self::TSExportAssignment(_)
            | Self::TSNamespaceExportDeclaration(_) => None,
        }
    }
}

impl AccessorPropertyType {
    pub fn is_abstract(&self) -> bool {
        matches!(self, Self::TSAbstractAccessorProperty)
    }
}

impl<'a> ImportDeclarationSpecifier<'a> {
    pub fn local(&self) -> &BindingIdentifier<'a> {
        match self {
            ImportDeclarationSpecifier::ImportSpecifier(specifier) => &specifier.local,
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => &specifier.local,
            ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => &specifier.local,
        }
    }
    pub fn name(&self) -> CompactStr {
        self.local().name.to_compact_str()
    }
}

impl<'a> ImportAttributeKey<'a> {
    pub fn as_atom(&self) -> Atom<'a> {
        match self {
            Self::Identifier(identifier) => identifier.name.clone(),
            Self::StringLiteral(literal) => literal.value.clone(),
        }
    }
}

impl<'a> ExportNamedDeclaration<'a> {
    pub fn is_typescript_syntax(&self) -> bool {
        self.export_kind == ImportOrExportKind::Type
            || self.declaration.as_ref().map_or(false, Declaration::is_typescript_syntax)
    }
}

impl<'a> ExportDefaultDeclaration<'a> {
    pub fn is_typescript_syntax(&self) -> bool {
        self.declaration.is_typescript_syntax()
    }
}

impl<'a> ExportAllDeclaration<'a> {
    pub fn is_typescript_syntax(&self) -> bool {
        self.export_kind.is_type()
    }
}

impl<'a> ExportSpecifier<'a> {
    pub fn new(span: Span, local: ModuleExportName<'a>, exported: ModuleExportName<'a>) -> Self {
        Self { span, local, exported, export_kind: ImportOrExportKind::Value }
    }
}

impl<'a> ExportDefaultDeclarationKind<'a> {
    #[inline]
    pub fn is_typescript_syntax(&self) -> bool {
        match self {
            Self::FunctionDeclaration(func) => func.is_typescript_syntax(),
            Self::ClassDeclaration(class) => class.is_typescript_syntax(),
            Self::TSInterfaceDeclaration(_) => true,
            _ => false,
        }
    }
}

impl<'a> fmt::Display for ModuleExportName<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Self::IdentifierName(identifier) => identifier.name.to_string(),
            Self::IdentifierReference(identifier) => identifier.name.to_string(),
            Self::StringLiteral(literal) => format!(r#""{}""#, literal.value),
        };
        write!(f, "{s}")
    }
}

impl<'a> ModuleExportName<'a> {
    pub fn name(&self) -> Atom<'a> {
        match self {
            Self::IdentifierName(identifier) => identifier.name.clone(),
            Self::IdentifierReference(identifier) => identifier.name.clone(),
            Self::StringLiteral(literal) => literal.value.clone(),
        }
    }

    pub fn identifier_name(&self) -> Option<Atom<'a>> {
        match self {
            Self::IdentifierName(identifier) => Some(identifier.name.clone()),
            Self::IdentifierReference(identifier) => Some(identifier.name.clone()),
            Self::StringLiteral(_) => None,
        }
    }
}
