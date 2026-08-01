//! Static ETS grammar shared by top-level, class, interface, and struct parsers.

use oxc_allocator::{ArenaBox, ArenaVec};
use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{
    ParserConfig as Config, ParserImpl,
    lexer::Kind,
    modifiers::{ModifierKind, Modifiers},
};

impl<'a, C: Config> ParserImpl<'a, C> {
    pub(crate) fn alloc_ets_decorators(
        &self,
        decorators: ArenaVec<'a, Decorator<'a>>,
    ) -> Option<ArenaBox<'a, ArenaVec<'a, Decorator<'a>>>> {
        (!decorators.is_empty()).then(|| self.alloc(decorators))
    }

    pub(crate) fn register_ets_type_name(&mut self, name: oxc_str::Ident<'a>) {
        if self.source_type.is_ets_static() {
            self.state.ets_type_names.insert(name);
        }
    }

    pub(crate) fn ets_identifier_is_type(&self, expression: &Expression<'a>) -> bool {
        self.source_type.is_ets_static()
            && matches!(
                expression,
                Expression::Identifier(identifier)
                    if self.state.ets_type_names.contains(&identifier.name)
            )
    }

    pub(crate) fn check_ets_type_value(&mut self, expression: &Expression<'a>) {
        if self.ets_identifier_is_type(expression) {
            self.error(crate::diagnostics::ets_unsupported_syntax(
                "Using a type declaration as a runtime object",
                expression.span(),
            ));
        }
    }

    pub(crate) fn check_ets_annotation_usage(&mut self, expression: &Expression<'a>) {
        let Expression::CallExpression(call) = expression else { return };

        if call.arguments.len() > 1 {
            self.error(crate::diagnostics::ets_annotation_single_argument(
                call.arguments[1].span(),
            ));
        }

        let Some(argument) = call.arguments.first() else { return };
        let Some(argument) = argument.as_expression() else {
            self.error(crate::diagnostics::ets_invalid_annotation_value(argument.span()));
            return;
        };

        if let Expression::ObjectExpression(object) = argument {
            for property in &object.properties {
                let ObjectPropertyKind::ObjectProperty(property) = property else {
                    self.error(crate::diagnostics::ets_invalid_annotation_value(property.span()));
                    continue;
                };
                if property.shorthand
                    || property.method
                    || !matches!(property.kind, PropertyKind::Init | PropertyKind::EtsEquals)
                {
                    self.error(crate::diagnostics::ets_annotation_argument_requires_initializer(
                        property.span,
                    ));
                    continue;
                }
                self.check_ets_annotation_value(&property.value);
            }
        } else {
            self.check_ets_annotation_value(argument);
        }
    }

    pub(crate) fn check_ets_annotation_value(&mut self, expression: &Expression<'a>) {
        self.check_ets_annotation_type_value(expression);
        if !Self::ets_annotation_value_is_valid(expression) {
            self.error(crate::diagnostics::ets_invalid_annotation_value(expression.span()));
        }
    }

    fn check_ets_annotation_type_value(&mut self, expression: &Expression<'a>) {
        self.check_ets_type_value(expression);
        match expression {
            Expression::ArrayExpression(array) => {
                for element in &array.elements {
                    if let Some(element) = element.as_expression() {
                        self.check_ets_annotation_type_value(element);
                    }
                }
            }
            Expression::BinaryExpression(binary) => {
                self.check_ets_annotation_type_value(&binary.left);
                self.check_ets_annotation_type_value(&binary.right);
            }
            Expression::LogicalExpression(logical) => {
                self.check_ets_annotation_type_value(&logical.left);
                self.check_ets_annotation_type_value(&logical.right);
            }
            Expression::UnaryExpression(unary) => {
                self.check_ets_annotation_type_value(&unary.argument);
            }
            Expression::ConditionalExpression(conditional) => {
                self.check_ets_annotation_type_value(&conditional.test);
                self.check_ets_annotation_type_value(&conditional.consequent);
                self.check_ets_annotation_type_value(&conditional.alternate);
            }
            Expression::TSAsExpression(as_expression) => {
                self.check_ets_annotation_type_value(&as_expression.expression);
            }
            Expression::ParenthesizedExpression(parenthesized) => {
                self.check_ets_annotation_type_value(&parenthesized.expression);
            }
            _ => {}
        }
    }

    fn ets_annotation_value_is_valid(expression: &Expression<'a>) -> bool {
        match expression {
            Expression::ArrayExpression(array) => array.elements.iter().all(|element| {
                element.as_expression().is_some_and(Self::ets_annotation_value_is_valid)
            }),
            Expression::StringLiteral(_)
            | Expression::NumericLiteral(_)
            | Expression::BooleanLiteral(_)
            | Expression::Identifier(_)
            | Expression::BinaryExpression(_)
            | Expression::LogicalExpression(_)
            | Expression::UnaryExpression(_)
            | Expression::ConditionalExpression(_)
            | Expression::TSAsExpression(_)
            | Expression::ETSInstanceOfExpression(_) => true,
            expression if expression.is_member_expression() => true,
            Expression::ParenthesizedExpression(parenthesized) => {
                Self::ets_annotation_value_is_valid(&parenthesized.expression)
            }
            _ => false,
        }
    }

    pub(crate) fn check_ets_binding_name(&mut self, name: &str, span: oxc_span::Span) {
        if !self.source_type.is_ets_static() {
            return;
        }
        if matches!(
            name,
            "Any"
                | "bigint"
                | "BigInt"
                | "boolean"
                | "Boolean"
                | "byte"
                | "Byte"
                | "char"
                | "Char"
                | "double"
                | "Double"
                | "float"
                | "Float"
                | "int"
                | "Int"
                | "long"
                | "Long"
                | "number"
                | "Number"
                | "object"
                | "Object"
                | "short"
                | "Short"
                | "string"
                | "String"
                | "void"
                | "Partial"
                | "Readonly"
                | "Required"
                | "Awaited"
                | "ReturnType"
                | "any"
                | "is"
                | "namespace"
                | "NonNullable"
                | "undefined"
                | "var"
                | "yield"
        ) {
            self.error(crate::diagnostics::ets_reserved_identifier(name, span));
        }
    }

    pub(crate) fn parse_ets_package_declaration(&mut self) -> Statement<'a> {
        let span = self.start_span();
        if !self.ctx.has_top_level() || !self.state.ets_in_declaration_scope {
            self.error(crate::diagnostics::ets_nested_declaration(
                "Package",
                self.cur_token().span(),
            ));
        }
        self.expect(Kind::Package);
        let mut name = ArenaVec::with_capacity_in(1, self);
        name.push(self.parse_identifier_name());
        while self.eat(Kind::Dot) {
            name.push(self.parse_identifier_name());
        }
        self.asi();
        Statement::ETSPackageDeclaration(ETSPackageDeclaration::boxed(
            self.end_span(span),
            name,
            self,
        ))
    }

    /// Parse an ETS managed overload declaration.
    ///
    /// The es2panda frontend represents the same node in all four declaration
    /// contexts and records the context as an overload flag. Oxc mirrors that
    /// shape with [`ETSOverloadDeclarationKind`].
    pub(crate) fn parse_ets_overload_declaration(
        &mut self,
        start: u32,
        decorators: ArenaVec<'a, Decorator<'a>>,
        modifiers: &Modifiers,
        kind: ETSOverloadDeclarationKind,
    ) -> ArenaBox<'a, ETSOverloadDeclaration<'a>> {
        debug_assert!(self.source_type.is_ets_static());
        self.expect(Kind::Overload);

        let key = if self.at(Kind::Constructor) {
            let identifier = self.parse_identifier_name();
            PropertyKey::StaticIdentifier(self.alloc(identifier))
        } else {
            let (key, _) = self.parse_property_name();
            key
        };

        self.expect(Kind::LCurly);
        let mut overloads = ArenaVec::new_in(self);
        while !self.at(Kind::RCurly) && !self.at(Kind::Eof) {
            overloads.push(self.parse_lhs_expression_or_higher());
            if !self.eat(Kind::Comma) {
                break;
            }
        }
        self.expect(Kind::RCurly);
        self.asi();

        ETSOverloadDeclaration::boxed(
            self.end_span(start),
            decorators,
            key,
            overloads,
            kind,
            modifiers.accessibility(),
            modifiers.contains(ModifierKind::Static),
            modifiers.contains(ModifierKind::Abstract),
            modifiers.contains(ModifierKind::Final),
            modifiers.contains(ModifierKind::Native),
            modifiers.contains(ModifierKind::Declare),
            self,
        )
    }
}
