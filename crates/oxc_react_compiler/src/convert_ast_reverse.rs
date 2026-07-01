// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

//! Reverse AST converter: react_compiler_ast (Babel format) → OXC AST.
//!
//! This is the inverse of `convert_ast.rs`. It takes a `crate::react_compiler_ast::File`
//! (which represents the compiler's Babel-compatible output) and produces OXC AST
//! nodes allocated in an OXC arena, suitable for code generation via `oxc_codegen`.

use crate::react_compiler_ast::File;
use crate::react_compiler_ast::Program;
use crate::react_compiler_ast::SourceType as AstSourceType;
use crate::react_compiler_ast::common::BaseNode;
use crate::react_compiler_ast::declarations::*;
use crate::react_compiler_ast::expressions::*;
use crate::react_compiler_ast::jsx::*;
use crate::react_compiler_ast::operators::*;
use crate::react_compiler_ast::patterns::*;
use crate::react_compiler_ast::statements::*;
use oxc_allocator::{Allocator, ArenaBox, ArenaStringBuilder, ArenaVec, GetAllocator};
use oxc_ast::ast as oxc;
use oxc_ast::builder::{AstBuilder, GetAstBuilder};
use oxc_ast_visit::VisitMut;
use oxc_parser::Parser;
use oxc_span::GetSpanMut;
use oxc_span::SPAN;
use oxc_span::SourceType;
use oxc_span::Span;
use oxc_str::Str;
use oxc_syntax::identifier::is_identifier_name;
use oxc_syntax::operator::{
    AssignmentOperator as OxcAssOp, BinaryOperator as OxcBinOp, LogicalOperator as OxcLogOp,
    UnaryOperator as OxcUnOp, UpdateOperator as OxcUpOp,
};
use serde_json::Value;
use std::iter::once;

fn set_statement_span(stmt: &mut oxc::Statement<'_>, span: Span) {
    match stmt {
        oxc::Statement::ImportDeclaration(d) => *d.span_mut() = span,
        oxc::Statement::VariableDeclaration(d) => *d.span_mut() = span,
        oxc::Statement::FunctionDeclaration(d) => *d.span_mut() = span,
        oxc::Statement::ExportNamedDeclaration(d) => *d.span_mut() = span,
        oxc::Statement::ExportDefaultDeclaration(d) => *d.span_mut() = span,
        oxc::Statement::ExportAllDeclaration(d) => *d.span_mut() = span,
        oxc::Statement::ExpressionStatement(s) => *s.span_mut() = span,
        oxc::Statement::IfStatement(s) => *s.span_mut() = span,
        oxc::Statement::ForStatement(s) => *s.span_mut() = span,
        oxc::Statement::WhileStatement(s) => *s.span_mut() = span,
        oxc::Statement::DoWhileStatement(s) => *s.span_mut() = span,
        oxc::Statement::ForInStatement(s) => *s.span_mut() = span,
        oxc::Statement::ForOfStatement(s) => *s.span_mut() = span,
        oxc::Statement::SwitchStatement(s) => *s.span_mut() = span,
        oxc::Statement::ThrowStatement(s) => *s.span_mut() = span,
        oxc::Statement::TryStatement(s) => *s.span_mut() = span,
        oxc::Statement::BreakStatement(s) => *s.span_mut() = span,
        oxc::Statement::ContinueStatement(s) => *s.span_mut() = span,
        oxc::Statement::LabeledStatement(s) => *s.span_mut() = span,
        oxc::Statement::BlockStatement(s) => *s.span_mut() = span,
        oxc::Statement::ReturnStatement(s) => *s.span_mut() = span,
        oxc::Statement::WithStatement(s) => *s.span_mut() = span,
        oxc::Statement::EmptyStatement(s) => *s.span_mut() = span,
        oxc::Statement::DebuggerStatement(s) => *s.span_mut() = span,
        _ => {} // ClassDeclaration etc. - leave as-is
    }
}

fn encode_jsx_text(raw: &str) -> String {
    let mut escaped = String::with_capacity(raw.len());
    for ch in raw.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '{' => escaped.push_str("&#123;"),
            '}' => escaped.push_str("&#125;"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

struct SpanShift {
    offset: u32,
}

impl VisitMut<'_> for SpanShift {
    fn visit_span(&mut self, span: &mut Span) {
        span.start = span.start.saturating_add(self.offset);
        span.end = span.end.saturating_add(self.offset);
    }
}

/// Convert a `crate::react_compiler_ast::File` into an OXC `Program` allocated in the given arena.
pub fn convert_program_to_oxc<'a>(file: &File, allocator: &'a Allocator) -> oxc::Program<'a> {
    let ctx = ReverseCtx::new(allocator, None);
    ctx.convert_program(&file.program)
}

/// Convert with source text available for extracting TS declarations.
pub fn convert_program_to_oxc_with_source<'a>(
    file: &File,
    allocator: &'a Allocator,
    source_text: &str,
) -> oxc::Program<'a> {
    let ctx = ReverseCtx::new(allocator, Some(source_text.to_string()));
    ctx.convert_program(&file.program)
}

struct ReverseCtx<'a> {
    allocator: &'a Allocator,
    builder: AstBuilder<'a>,
    source_text: Option<String>,
}

impl<'a> ReverseCtx<'a> {
    fn new(allocator: &'a Allocator, source_text: Option<String>) -> Self {
        Self { allocator, builder: AstBuilder::new(allocator), source_text }
    }

    /// Extract a statement from the original source text using the base node's
    /// start/end positions. Re-parses the snippet with OXC to get a proper AST node.
    fn extract_source_stmt(&self, base: &BaseNode) -> Option<oxc::Statement<'a>> {
        let text = self.source_text_for_base(base)?;
        self.parse_source_stmt_text_at(text, base.start? as usize)
    }

    fn parse_source_stmt_text_at(
        &self,
        text: &str,
        original_start: usize,
    ) -> Option<oxc::Statement<'a>> {
        let text_ref = self.copy_source_text_to_allocator(text);
        let parsed = Parser::new(self.allocator, text_ref, SourceType::tsx()).parse();
        if parsed.panicked || parsed.program.body.is_empty() {
            return None;
        }
        let mut stmt = parsed.program.body.into_iter().next()?;
        if original_start > 0 {
            let mut shifter = SpanShift { offset: original_start as u32 };
            shifter.visit_statement(&mut stmt);
        }
        Some(stmt)
    }

    /// Extract an expression from the original source text using the base
    /// node's start/end positions.
    fn extract_source_expr(&self, base: &BaseNode) -> Option<oxc::Expression<'a>> {
        let text = self.source_text_for_base(base)?;
        self.parse_source_expr_text_at(text, base.start? as usize)
    }

    fn parse_source_expr_text_at(
        &self,
        text: &str,
        original_start: usize,
    ) -> Option<oxc::Expression<'a>> {
        let text_ref = self.copy_source_text_to_allocator(text);
        let mut expr =
            Parser::new(self.allocator, text_ref, SourceType::tsx()).parse_expression().ok()?;
        if original_start > 0 {
            let mut shifter = SpanShift { offset: original_start as u32 };
            shifter.visit_expression(&mut expr);
        }
        Some(expr)
    }

    fn parse_source_ts_type_text_at(
        &self,
        text: &str,
        original_start: usize,
    ) -> Option<oxc::TSType<'a>> {
        const PREFIX: &str = "let __oxc_type = __oxc_value as ";
        let wrapped = format!("{PREFIX}{text};");
        let stmt =
            self.parse_source_stmt_text_at(&wrapped, original_start.saturating_sub(PREFIX.len()))?;
        let oxc::Statement::VariableDeclaration(decl) = stmt else { return None };
        let decl = decl.unbox();
        let init = decl.declarations.into_iter().next()?.init?;
        let oxc::Expression::TSAsExpression(ts_as) = init else { return None };
        Some(ts_as.unbox().type_annotation)
    }

    fn copy_source_text_to_allocator(&self, text: &str) -> &'a str {
        ArenaStringBuilder::from_str_in(text, self.allocator).into_str()
    }

    fn extract_source_class_expression(
        &self,
        class: &ClassExpression,
    ) -> Option<oxc::Expression<'a>> {
        let expr = self.extract_source_expr(&class.base)?;
        if matches!(expr, oxc::Expression::ClassExpression(_)) { Some(expr) } else { None }
    }

    fn extract_source_call_type_arguments(
        &self,
        base: &BaseNode,
    ) -> Option<ArenaBox<'a, oxc::TSTypeParameterInstantiation<'a>>> {
        match self.extract_source_expr(base)? {
            oxc::Expression::CallExpression(call) => call.unbox().type_arguments,
            oxc::Expression::ChainExpression(chain) => match chain.unbox().expression {
                oxc::ChainElement::CallExpression(call) => call.unbox().type_arguments,
                _ => None,
            },
            _ => None,
        }
    }

    fn extract_source_call_arguments(&self, base: &BaseNode) -> Option<Vec<oxc::Argument<'a>>> {
        match self.extract_source_expr(base)? {
            oxc::Expression::CallExpression(call) => {
                Some(call.unbox().arguments.into_iter().collect())
            }
            oxc::Expression::ChainExpression(chain) => match chain.unbox().expression {
                oxc::ChainElement::CallExpression(call) => {
                    Some(call.unbox().arguments.into_iter().collect())
                }
                _ => None,
            },
            _ => None,
        }
    }

    fn extract_source_new_type_arguments(
        &self,
        base: &BaseNode,
    ) -> Option<ArenaBox<'a, oxc::TSTypeParameterInstantiation<'a>>> {
        match self.extract_source_expr(base)? {
            oxc::Expression::NewExpression(new) => new.unbox().type_arguments,
            _ => None,
        }
    }

    fn extract_source_new_arguments(&self, base: &BaseNode) -> Option<Vec<oxc::Argument<'a>>> {
        match self.extract_source_expr(base)? {
            oxc::Expression::NewExpression(new) => {
                Some(new.unbox().arguments.into_iter().collect())
            }
            _ => None,
        }
    }

    fn extract_source_tagged_template_type_arguments(
        &self,
        base: &BaseNode,
    ) -> Option<ArenaBox<'a, oxc::TSTypeParameterInstantiation<'a>>> {
        match self.extract_source_expr(base)? {
            oxc::Expression::TaggedTemplateExpression(tagged) => tagged.unbox().type_arguments,
            _ => None,
        }
    }

    fn extract_source_jsx_type_arguments(
        &self,
        base: &BaseNode,
    ) -> Option<ArenaBox<'a, oxc::TSTypeParameterInstantiation<'a>>> {
        match self.extract_source_expr(base)? {
            oxc::Expression::JSXElement(element) => {
                element.unbox().opening_element.unbox().type_arguments
            }
            _ => None,
        }
    }

    fn extract_source_ts_as_type(&self, base: &BaseNode) -> Option<oxc::TSType<'a>> {
        match self.extract_source_expr(base)? {
            oxc::Expression::TSAsExpression(expr) => Some(expr.unbox().type_annotation),
            _ => None,
        }
    }

    fn extract_source_ts_type_from_json(&self, value: &Value) -> Option<oxc::TSType<'a>> {
        let value = if value.get("type").and_then(Value::as_str) == Some("TSTypeAnnotation") {
            value.get("typeAnnotation")?
        } else {
            value
        };
        let source = self.source_text.as_deref()?;
        let start = value.get("start")?.as_u64()? as usize;
        let end = value.get("end")?.as_u64()? as usize;
        if start >= source.len() || end > source.len() || start >= end {
            return None;
        }
        self.parse_source_ts_type_text_at(&source[start..end], start)
    }

    fn span_from_json_value(&self, value: &Value) -> Span {
        let start = value.get("start").and_then(Value::as_u64);
        let end = value.get("end").and_then(Value::as_u64);
        match (start, end) {
            (Some(start), Some(end)) => Span::new(start as u32, end as u32),
            (Some(start), None) => Span::new(start as u32, start as u32),
            _ => SPAN,
        }
    }

    fn convert_ts_type_annotation_from_json(
        &self,
        value: &Value,
    ) -> Option<ArenaBox<'a, oxc::TSTypeAnnotation<'a>>> {
        let ty = if value.get("type").and_then(Value::as_str) == Some("TSTypeAnnotation") {
            let type_annotation = value.get("typeAnnotation")?;
            self.convert_ts_type_from_json(type_annotation).or_else(|| {
                if Self::ts_type_json_contains_type_query(type_annotation) {
                    None
                } else {
                    self.extract_source_ts_type_from_json(value)
                }
            })?
        } else {
            self.convert_ts_type_from_json(value)?
        };
        Some(oxc::TSTypeAnnotation::boxed(SPAN, ty, self))
    }

    fn convert_ts_type_from_json(&self, value: &Value) -> Option<oxc::TSType<'a>> {
        if !Self::ts_type_json_contains_type_query(value)
            && let Some(ty) = self.extract_source_ts_type_from_json(value)
        {
            return Some(ty);
        }

        match value.get("type")?.as_str()? {
            "TSAnyKeyword" => Some(oxc::TSType::new_ts_any_keyword(SPAN, self)),
            "TSBigIntKeyword" => Some(oxc::TSType::new_ts_big_int_keyword(SPAN, self)),
            "TSBooleanKeyword" => Some(oxc::TSType::new_ts_boolean_keyword(SPAN, self)),
            "TSIntrinsicKeyword" => Some(oxc::TSType::new_ts_intrinsic_keyword(SPAN, self)),
            "TSNeverKeyword" => Some(oxc::TSType::new_ts_never_keyword(SPAN, self)),
            "TSNullKeyword" => Some(oxc::TSType::new_ts_null_keyword(SPAN, self)),
            "TSNumberKeyword" => Some(oxc::TSType::new_ts_number_keyword(SPAN, self)),
            "TSObjectKeyword" => Some(oxc::TSType::new_ts_object_keyword(SPAN, self)),
            "TSStringKeyword" => Some(oxc::TSType::new_ts_string_keyword(SPAN, self)),
            "TSSymbolKeyword" => Some(oxc::TSType::new_ts_symbol_keyword(SPAN, self)),
            "TSThisType" => Some(oxc::TSType::new_ts_this_type(SPAN, self)),
            "TSUndefinedKeyword" => Some(oxc::TSType::new_ts_undefined_keyword(SPAN, self)),
            "TSUnknownKeyword" => Some(oxc::TSType::new_ts_unknown_keyword(SPAN, self)),
            "TSVoidKeyword" => Some(oxc::TSType::new_ts_void_keyword(SPAN, self)),
            "TSArrayType" => {
                let element_type = self.convert_ts_type_from_json(value.get("elementType")?)?;
                Some(oxc::TSType::new_ts_array_type(SPAN, element_type, self))
            }
            "TSUnionType" => {
                let types = value.get("types")?.as_array()?;
                let types = ArenaVec::from_iter_in(
                    types.iter().filter_map(|ty| self.convert_ts_type_from_json(ty)),
                    self,
                );
                Some(oxc::TSType::new_ts_union_type(SPAN, types, self))
            }
            "TSParenthesizedType" => {
                let type_annotation =
                    self.convert_ts_type_from_json(value.get("typeAnnotation")?)?;
                Some(oxc::TSType::new_ts_parenthesized_type(SPAN, type_annotation, self))
            }
            "TSTypeOperator" => {
                let operator = match value.get("operator")?.as_str()? {
                    "keyof" => oxc::TSTypeOperatorOperator::Keyof,
                    "unique" => oxc::TSTypeOperatorOperator::Unique,
                    "readonly" => oxc::TSTypeOperatorOperator::Readonly,
                    _ => return None,
                };
                let type_annotation =
                    self.convert_ts_type_from_json(value.get("typeAnnotation")?)?;
                Some(oxc::TSType::new_ts_type_operator_type(SPAN, operator, type_annotation, self))
            }
            "TSTypeReference" => {
                let type_name = self.convert_ts_type_name_from_json(value.get("typeName")?)?;
                let type_arguments =
                    value.get("typeParameters").or_else(|| value.get("typeArguments")).and_then(
                        |value| self.convert_ts_type_parameter_instantiation_from_json(value),
                    );
                Some(oxc::TSType::new_ts_type_reference(SPAN, type_name, type_arguments, self))
            }
            "TSTypeQuery" => {
                let expr_name =
                    self.convert_ts_type_query_expr_name_from_json(value.get("exprName")?)?;
                let type_arguments =
                    value.get("typeParameters").or_else(|| value.get("typeArguments")).and_then(
                        |value| self.convert_ts_type_parameter_instantiation_from_json(value),
                    );
                Some(oxc::TSType::new_ts_type_query(
                    self.span_from_json_value(value),
                    expr_name,
                    type_arguments,
                    self,
                ))
            }
            "TSIndexedAccessType" => {
                let object_type = self.convert_ts_type_from_json(value.get("objectType")?)?;
                let index_type = self.convert_ts_type_from_json(value.get("indexType")?)?;
                Some(oxc::TSType::new_ts_indexed_access_type(
                    self.span_from_json_value(value),
                    object_type,
                    index_type,
                    self,
                ))
            }
            "TSLiteralType" => {
                let literal = self.convert_ts_literal_from_json(value.get("literal")?)?;
                Some(oxc::TSType::new_ts_literal_type(SPAN, literal, self))
            }
            _ => None,
        }
    }

    fn convert_ts_type_name_from_json(&self, value: &Value) -> Option<oxc::TSTypeName<'a>> {
        match value.get("type")?.as_str()? {
            "Identifier" => Some(oxc::TSTypeName::new_identifier_reference(
                self.span_from_json_value(value),
                self.str(value.get("name")?.as_str()?),
                self,
            )),
            "TSQualifiedName" => {
                let left = self.convert_ts_type_name_from_json(value.get("left")?)?;
                let right_value = value.get("right")?;
                let right = oxc::IdentifierName::new(
                    self.span_from_json_value(right_value),
                    self.str(right_value.get("name")?.as_str()?),
                    self,
                );
                Some(oxc::TSTypeName::new_qualified_name(
                    self.span_from_json_value(value),
                    left,
                    right,
                    self,
                ))
            }
            "TSThisType" | "ThisExpression" => {
                Some(oxc::TSTypeName::new_this_expression(self.span_from_json_value(value), self))
            }
            _ => None,
        }
    }

    fn convert_ts_type_query_expr_name_from_json(
        &self,
        value: &Value,
    ) -> Option<oxc::TSTypeQueryExprName<'a>> {
        match self.convert_ts_type_name_from_json(value)? {
            oxc::TSTypeName::IdentifierReference(identifier) => {
                Some(oxc::TSTypeQueryExprName::IdentifierReference(identifier))
            }
            oxc::TSTypeName::QualifiedName(qualified) => {
                Some(oxc::TSTypeQueryExprName::QualifiedName(qualified))
            }
            oxc::TSTypeName::ThisExpression(this) => {
                Some(oxc::TSTypeQueryExprName::ThisExpression(this))
            }
        }
    }

    fn convert_ts_type_parameter_instantiation_from_json(
        &self,
        value: &Value,
    ) -> Option<ArenaBox<'a, oxc::TSTypeParameterInstantiation<'a>>> {
        let params = value.get("params")?.as_array()?;
        let params = ArenaVec::from_iter_in(
            params.iter().filter_map(|ty| self.convert_ts_type_from_json(ty)),
            self,
        );
        Some(oxc::TSTypeParameterInstantiation::boxed(SPAN, params, self))
    }

    fn convert_ts_literal_from_json(&self, value: &Value) -> Option<oxc::TSLiteral<'a>> {
        match value.get("type")?.as_str()? {
            "BooleanLiteral" => Some(oxc::TSLiteral::new_boolean_literal(
                SPAN,
                value.get("value")?.as_bool()?,
                self,
            )),
            "NumericLiteral" => Some(oxc::TSLiteral::new_numeric_literal(
                SPAN,
                value.get("value")?.as_f64()?,
                None,
                oxc::NumberBase::Decimal,
                self,
            )),
            "StringLiteral" => Some(oxc::TSLiteral::new_string_literal(
                SPAN,
                self.str(value.get("value")?.as_str()?),
                None,
                self,
            )),
            "BigIntLiteral" => Some(oxc::TSLiteral::new_big_int_literal(
                SPAN,
                self.str(value.get("value")?.as_str()?),
                None,
                oxc::BigintBase::Decimal,
                self,
            )),
            _ => None,
        }
    }

    fn ts_type_json_contains_type_query(value: &Value) -> bool {
        match value {
            Value::Object(obj) => {
                let is_rename_sensitive_type_query = obj.get("type").and_then(Value::as_str)
                    == Some("TSTypeQuery")
                    && obj
                        .get("exprName")
                        .and_then(|expr| expr.get("type"))
                        .and_then(Value::as_str)
                        != Some("TSImportType");
                is_rename_sensitive_type_query
                    || obj.values().any(Self::ts_type_json_contains_type_query)
            }
            Value::Array(values) => values.iter().any(Self::ts_type_json_contains_type_query),
            _ => false,
        }
    }

    fn ts_type_contains_type_query(ty: &oxc::TSType<'a>) -> bool {
        match ty {
            oxc::TSType::TSTypeQuery(_) => true,
            oxc::TSType::TSArrayType(ty) => Self::ts_type_contains_type_query(&ty.element_type),
            oxc::TSType::TSUnionType(ty) => ty.types.iter().any(Self::ts_type_contains_type_query),
            oxc::TSType::TSIntersectionType(ty) => {
                ty.types.iter().any(Self::ts_type_contains_type_query)
            }
            oxc::TSType::TSParenthesizedType(ty) => {
                Self::ts_type_contains_type_query(&ty.type_annotation)
            }
            oxc::TSType::TSTypeOperatorType(ty) => {
                Self::ts_type_contains_type_query(&ty.type_annotation)
            }
            oxc::TSType::TSIndexedAccessType(ty) => {
                Self::ts_type_contains_type_query(&ty.object_type)
                    || Self::ts_type_contains_type_query(&ty.index_type)
            }
            oxc::TSType::TSTypeReference(ty) => {
                ty.type_arguments.as_ref().is_some_and(|type_arguments| {
                    Self::ts_type_arguments_contain_type_query(type_arguments)
                })
            }
            _ => false,
        }
    }

    fn ts_type_arguments_contain_type_query(
        type_arguments: &oxc::TSTypeParameterInstantiation<'a>,
    ) -> bool {
        type_arguments.params.iter().any(Self::ts_type_contains_type_query)
    }

    fn extract_source_ts_satisfies_type(&self, base: &BaseNode) -> Option<oxc::TSType<'a>> {
        match self.extract_source_expr(base)? {
            oxc::Expression::TSSatisfiesExpression(expr) => Some(expr.unbox().type_annotation),
            _ => None,
        }
    }

    fn extract_source_ts_type_assertion_type(&self, base: &BaseNode) -> Option<oxc::TSType<'a>> {
        match self.extract_source_expr(base)? {
            oxc::Expression::TSTypeAssertion(expr) => Some(expr.unbox().type_annotation),
            _ => None,
        }
    }

    fn extract_source_ts_instantiation_type_arguments(
        &self,
        base: &BaseNode,
    ) -> Option<ArenaBox<'a, oxc::TSTypeParameterInstantiation<'a>>> {
        match self.extract_source_expr(base)? {
            oxc::Expression::TSInstantiationExpression(expr) => Some(expr.unbox().type_arguments),
            _ => None,
        }
    }

    fn wrap_expression_with_source_ts(
        &self,
        base: &BaseNode,
        expression: oxc::Expression<'a>,
    ) -> Result<oxc::Expression<'a>, oxc::Expression<'a>> {
        let Some(text) = self.source_text_for_base(base) else {
            return Err(expression);
        };
        if !text.contains(" as ") && !text.contains(" satisfies ") && !text.contains('<') {
            return Err(expression);
        }

        let Some(start) = base.start else {
            return Err(expression);
        };
        match self.parse_source_expr_text_at(text, start as usize) {
            Some(source_expr) => self.wrap_expression_with_source_ts_expr(source_expr, expression),
            None => Err(expression),
        }
    }

    fn wrap_expression_with_source_ts_expr(
        &self,
        source_expr: oxc::Expression<'a>,
        expression: oxc::Expression<'a>,
    ) -> Result<oxc::Expression<'a>, oxc::Expression<'a>> {
        if Self::is_ts_expression_wrapper(&expression) {
            return Err(expression);
        }
        match source_expr {
            oxc::Expression::TSAsExpression(expr) => {
                let expr = expr.unbox();
                if Self::ts_type_contains_type_query(&expr.type_annotation) {
                    return Err(expression);
                }
                Ok(oxc::Expression::new_ts_as_expression(
                    SPAN,
                    expression,
                    expr.type_annotation,
                    self,
                ))
            }
            oxc::Expression::TSSatisfiesExpression(expr) => {
                let expr = expr.unbox();
                if Self::ts_type_contains_type_query(&expr.type_annotation) {
                    return Err(expression);
                }
                Ok(oxc::Expression::new_ts_satisfies_expression(
                    SPAN,
                    expression,
                    expr.type_annotation,
                    self,
                ))
            }
            oxc::Expression::TSTypeAssertion(expr) => {
                let expr = expr.unbox();
                if Self::ts_type_contains_type_query(&expr.type_annotation) {
                    return Err(expression);
                }
                Ok(oxc::Expression::new_ts_type_assertion(
                    SPAN,
                    expr.type_annotation,
                    expression,
                    self,
                ))
            }
            oxc::Expression::TSInstantiationExpression(expr) => {
                let expr = expr.unbox();
                if Self::ts_type_arguments_contain_type_query(&expr.type_arguments) {
                    return Err(expression);
                }
                Ok(oxc::Expression::new_ts_instantiation_expression(
                    SPAN,
                    expression,
                    expr.type_arguments,
                    self,
                ))
            }
            _ => Err(expression),
        }
    }

    fn wrap_expression_with_source_argument_ts(
        &self,
        source_arg: oxc::Argument<'a>,
        expression: oxc::Expression<'a>,
    ) -> Result<oxc::Expression<'a>, oxc::Expression<'a>> {
        if Self::is_ts_expression_wrapper(&expression) {
            return Err(expression);
        }
        match source_arg {
            oxc::Argument::TSAsExpression(expr) => {
                let expr = expr.unbox();
                if Self::ts_type_contains_type_query(&expr.type_annotation) {
                    return Err(expression);
                }
                Ok(oxc::Expression::new_ts_as_expression(
                    SPAN,
                    expression,
                    expr.type_annotation,
                    self,
                ))
            }
            oxc::Argument::TSSatisfiesExpression(expr) => {
                let expr = expr.unbox();
                if Self::ts_type_contains_type_query(&expr.type_annotation) {
                    return Err(expression);
                }
                Ok(oxc::Expression::new_ts_satisfies_expression(
                    SPAN,
                    expression,
                    expr.type_annotation,
                    self,
                ))
            }
            oxc::Argument::TSTypeAssertion(expr) => {
                let expr = expr.unbox();
                if Self::ts_type_contains_type_query(&expr.type_annotation) {
                    return Err(expression);
                }
                Ok(oxc::Expression::new_ts_type_assertion(
                    SPAN,
                    expr.type_annotation,
                    expression,
                    self,
                ))
            }
            oxc::Argument::TSInstantiationExpression(expr) => {
                let expr = expr.unbox();
                if Self::ts_type_arguments_contain_type_query(&expr.type_arguments) {
                    return Err(expression);
                }
                Ok(oxc::Expression::new_ts_instantiation_expression(
                    SPAN,
                    expression,
                    expr.type_arguments,
                    self,
                ))
            }
            _ => Err(expression),
        }
    }

    fn is_ts_expression_wrapper(expression: &oxc::Expression<'a>) -> bool {
        matches!(
            expression,
            oxc::Expression::TSAsExpression(_)
                | oxc::Expression::TSSatisfiesExpression(_)
                | oxc::Expression::TSTypeAssertion(_)
                | oxc::Expression::TSInstantiationExpression(_)
        )
    }

    fn expression_base(expr: &Expression) -> Option<&BaseNode> {
        match expr {
            Expression::Identifier(expr) => Some(&expr.base),
            Expression::StringLiteral(expr) => Some(&expr.base),
            Expression::NumericLiteral(expr) => Some(&expr.base),
            Expression::BooleanLiteral(expr) => Some(&expr.base),
            Expression::NullLiteral(expr) => Some(&expr.base),
            Expression::BigIntLiteral(expr) => Some(&expr.base),
            Expression::RegExpLiteral(expr) => Some(&expr.base),
            Expression::CallExpression(expr) => Some(&expr.base),
            Expression::MemberExpression(expr) => Some(&expr.base),
            Expression::OptionalCallExpression(expr) => Some(&expr.base),
            Expression::OptionalMemberExpression(expr) => Some(&expr.base),
            Expression::BinaryExpression(expr) => Some(&expr.base),
            Expression::LogicalExpression(expr) => Some(&expr.base),
            Expression::UnaryExpression(expr) => Some(&expr.base),
            Expression::UpdateExpression(expr) => Some(&expr.base),
            Expression::ConditionalExpression(expr) => Some(&expr.base),
            Expression::AssignmentExpression(expr) => Some(&expr.base),
            Expression::SequenceExpression(expr) => Some(&expr.base),
            Expression::ArrowFunctionExpression(expr) => Some(&expr.base),
            Expression::FunctionExpression(expr) => Some(&expr.base),
            Expression::ObjectExpression(expr) => Some(&expr.base),
            Expression::ArrayExpression(expr) => Some(&expr.base),
            Expression::NewExpression(expr) => Some(&expr.base),
            Expression::TemplateLiteral(expr) => Some(&expr.base),
            Expression::TaggedTemplateExpression(expr) => Some(&expr.base),
            Expression::AwaitExpression(expr) => Some(&expr.base),
            Expression::YieldExpression(expr) => Some(&expr.base),
            Expression::SpreadElement(expr) => Some(&expr.base),
            Expression::MetaProperty(expr) => Some(&expr.base),
            Expression::ClassExpression(expr) => Some(&expr.base),
            Expression::PrivateName(expr) => Some(&expr.base),
            Expression::Super(expr) => Some(&expr.base),
            Expression::Import(expr) => Some(&expr.base),
            Expression::ThisExpression(expr) => Some(&expr.base),
            Expression::ParenthesizedExpression(expr) => Some(&expr.base),
            Expression::JSXElement(expr) => Some(&expr.base),
            Expression::JSXFragment(expr) => Some(&expr.base),
            Expression::AssignmentPattern(expr) => Some(&expr.base),
            Expression::TSAsExpression(_)
            | Expression::TSSatisfiesExpression(_)
            | Expression::TSNonNullExpression(_)
            | Expression::TSTypeAssertion(_)
            | Expression::TSInstantiationExpression(_)
            | Expression::TypeCastExpression(_) => None,
        }
    }

    fn extract_source_variable_declarator(
        &self,
        base: &BaseNode,
        kind: oxc::VariableDeclarationKind,
    ) -> Option<oxc::VariableDeclarator<'a>> {
        let text = self.source_text_for_base(base)?;
        if !text.contains(':') && !text.contains('!') {
            return None;
        }

        let keyword = match kind {
            oxc::VariableDeclarationKind::Var => "var",
            oxc::VariableDeclarationKind::Let => "let",
            oxc::VariableDeclarationKind::Const => "const",
            oxc::VariableDeclarationKind::Using | oxc::VariableDeclarationKind::AwaitUsing => {
                "using"
            }
        };
        let wrapped = format!("{keyword} {text};");
        let stmt = self.parse_source_stmt_text_at(
            &wrapped,
            (base.start? as usize).saturating_sub(keyword.len() + 1),
        )?;
        let oxc::Statement::VariableDeclaration(decl) = stmt else { return None };
        decl.unbox().declarations.into_iter().next()
    }

    fn extract_source_object_property_value(&self, base: &BaseNode) -> Option<oxc::Expression<'a>> {
        let text = self.source_text_for_base(base)?;
        let wrapped = format!("({{{text}}})");
        let expr =
            self.parse_source_expr_text_at(&wrapped, (base.start? as usize).saturating_sub(2))?;
        let oxc::Expression::ObjectExpression(obj) = expr else { return None };
        let mut properties = obj.unbox().properties.into_iter();
        let Some(oxc::ObjectPropertyKind::ObjectProperty(prop)) = properties.next() else {
            return None;
        };
        Some(prop.unbox().value)
    }

    fn extract_source_function_declaration(&self, base: &BaseNode) -> Option<oxc::Function<'a>> {
        let stmt = self.extract_source_stmt(base)?;
        let oxc::Statement::FunctionDeclaration(func) = stmt else { return None };
        Some(func.unbox())
    }

    fn extract_source_function_expression(&self, base: &BaseNode) -> Option<oxc::Function<'a>> {
        match self.extract_source_expr(base)? {
            oxc::Expression::FunctionExpression(func) => Some(func.unbox()),
            _ => None,
        }
    }

    fn extract_source_arrow_function(
        &self,
        base: &BaseNode,
    ) -> Option<oxc::ArrowFunctionExpression<'a>> {
        match self.extract_source_expr(base)? {
            oxc::Expression::ArrowFunctionExpression(arrow) => Some(arrow.unbox()),
            _ => None,
        }
    }

    fn extract_source_object_method_function(&self, base: &BaseNode) -> Option<oxc::Function<'a>> {
        let text = self.source_text_for_base(base)?;
        let wrapped = format!("({{{text}}})");
        let expr =
            self.parse_source_expr_text_at(&wrapped, (base.start? as usize).saturating_sub(2))?;
        let oxc::Expression::ObjectExpression(obj) = expr else { return None };
        let mut properties = obj.unbox().properties.into_iter();
        let Some(oxc::ObjectPropertyKind::ObjectProperty(prop)) = properties.next() else {
            return None;
        };
        let prop = prop.unbox();
        if !prop.method {
            return None;
        }
        let oxc::Expression::FunctionExpression(func) = prop.value else { return None };
        Some(func.unbox())
    }

    fn apply_function_signature_from_source(
        &self,
        func: &mut oxc::Function<'a>,
        source_func: oxc::Function<'a>,
    ) {
        func.type_parameters = source_func.type_parameters;
        func.this_param = source_func.this_param;
        func.return_type = source_func.return_type;
        self.apply_formal_parameters_ts_from_source(&mut func.params, source_func.params);
    }

    fn apply_arrow_signature_from_source(
        &self,
        arrow: &mut oxc::ArrowFunctionExpression<'a>,
        source_arrow: oxc::ArrowFunctionExpression<'a>,
    ) {
        arrow.type_parameters = source_arrow.type_parameters;
        arrow.return_type = source_arrow.return_type;
        self.apply_formal_parameters_ts_from_source(&mut arrow.params, source_arrow.params);
    }

    fn apply_formal_parameters_ts_from_source(
        &self,
        params: &mut ArenaBox<'a, oxc::FormalParameters<'a>>,
        source_params: ArenaBox<'a, oxc::FormalParameters<'a>>,
    ) {
        let source_params = source_params.unbox();
        for (param, source_param) in params.items.iter_mut().zip(source_params.items) {
            param.decorators = source_param.decorators;
            param.type_annotation = source_param.type_annotation;
            param.optional = source_param.optional;
            param.accessibility = source_param.accessibility;
            param.readonly = source_param.readonly;
            param.r#override = source_param.r#override;
        }

        if let (Some(rest), Some(source_rest)) = (&mut params.rest, source_params.rest) {
            let source_rest = source_rest.unbox();
            rest.decorators = source_rest.decorators;
            rest.type_annotation = source_rest.type_annotation;
        }
    }

    fn block_initializes_react_cache(&self, block: &BlockStatement) -> bool {
        block.body.iter().any(|stmt| {
            let Statement::VariableDeclaration(decl) = stmt else { return false };
            decl.declarations.iter().any(|declarator| {
                declarator.init.as_deref().is_some_and(Self::is_react_cache_call_expression)
            })
        })
    }

    fn is_react_cache_call_expression(expr: &Expression) -> bool {
        let Expression::CallExpression(call) = expr else { return false };
        matches!(call.callee.as_ref(), Expression::Identifier(id) if id.name == "_c")
    }

    fn source_text_for_base(&self, base: &BaseNode) -> Option<&str> {
        let source = self.source_text.as_deref()?;
        let start = base.start? as usize;
        let end = base.end? as usize;
        if start >= source.len() || end > source.len() || start >= end {
            return None;
        }
        Some(&source[start..end])
    }

    fn import_declaration_has_empty_named_specifiers(&self, decl: &ImportDeclaration) -> bool {
        if let Some(text) = self.source_text_for_base(&decl.base) {
            let Some(mut rest) = text.trim_start().strip_prefix("import").map(str::trim_start)
            else {
                return false;
            };
            if let Some(after_type) = rest.strip_prefix("type")
                && after_type.chars().next().is_some_and(char::is_whitespace)
            {
                rest = after_type.trim_start();
            }
            return rest.starts_with("{}");
        }

        matches!(decl.import_kind, Some(ImportKind::Type | ImportKind::Typeof))
    }

    fn export_named_needs_source_stmt(&self, decl: &ExportNamedDeclaration) -> bool {
        matches!(
            decl.declaration.as_deref(),
            Some(
                Declaration::ClassDeclaration(_)
                    | Declaration::TSTypeAliasDeclaration(_)
                    | Declaration::TSInterfaceDeclaration(_)
                    | Declaration::TSEnumDeclaration(_)
                    | Declaration::TSModuleDeclaration(_)
                    | Declaration::TSDeclareFunction(_)
                    | Declaration::TypeAlias(_)
                    | Declaration::OpaqueType(_)
                    | Declaration::InterfaceDeclaration(_)
                    | Declaration::EnumDeclaration(_)
            )
        )
    }

    fn export_default_needs_source_stmt(&self, decl: &ExportDefaultDeclaration) -> bool {
        match decl.declaration.as_ref() {
            ExportDefaultDecl::ClassDeclaration(_) => true,
            ExportDefaultDecl::Expression(expr) => self.is_export_default_interface(expr, decl),
            ExportDefaultDecl::FunctionDeclaration(_) | ExportDefaultDecl::EnumDeclaration(_) => {
                false
            }
        }
    }

    fn is_export_default_interface(
        &self,
        expr: &Expression,
        decl: &ExportDefaultDeclaration,
    ) -> bool {
        let Expression::NullLiteral(null) = expr else { return false };
        self.source_text_for_base(&decl.base)
            .is_some_and(|text| text.trim_start().starts_with("export default interface"))
            || self
                .source_text_for_base(&null.base)
                .is_some_and(|text| text.trim_start().starts_with("interface"))
    }

    /// Allocate a string in the arena and return a `Str<'a>`.
    ///
    /// `Str<'a>` converts to `Ident<'a>`, so can be passed to any `AstBuilder` method.
    #[inline]
    fn str(&self, s: &str) -> Str<'a> {
        Str::from_str_in(s, self)
    }

    /// Convert a BaseNode's start/end into an OXC Span.
    /// Returns SPAN (0,0) if the base has no position info.
    fn span_from_base(&self, base: &BaseNode) -> Span {
        match (base.start, base.end) {
            (Some(start), Some(end)) => Span::new(start, end),
            (Some(start), None) => Span::new(start, start),
            _ => SPAN,
        }
    }

    // ===== Program =====

    fn convert_program(&self, program: &Program) -> oxc::Program<'a> {
        let source_type = match program.source_type {
            AstSourceType::Module => SourceType::mjs(),
            AstSourceType::Script => SourceType::cjs(),
        };

        // Use convert_statements_with_spans for the top-level body so that
        // original source positions are preserved. This allows comments from
        // the original source to be correctly attached to statements.
        let body = self.convert_statements_with_spans(&program.body);
        let directives = self.convert_directives(&program.directives);
        let hashbang = program.interpreter.as_ref().map(|interpreter| {
            oxc::Hashbang::new(
                self.span_from_base(&interpreter.base),
                self.str(&interpreter.value),
                self,
            )
        });
        let comments = ArenaVec::new_in(self);

        oxc::Program::new(SPAN, source_type, "", comments, hashbang, directives, body, self)
    }

    // ===== Directives =====

    fn convert_directives(&self, directives: &[Directive]) -> ArenaVec<'a, oxc::Directive<'a>> {
        ArenaVec::from_iter_in(directives.iter().map(|d| self.convert_directive(d)), self)
    }

    fn convert_directive(&self, d: &Directive) -> oxc::Directive<'a> {
        let expression = oxc::StringLiteral::new(SPAN, self.str(&d.value.value), None, self);
        oxc::Directive::new(SPAN, expression, self.str(&d.value.value), self)
    }

    // ===== Statements =====

    /// Convert statements preserving span info from the Babel AST.
    /// This is used for top-level program body where span positions
    /// are needed for comment attachment.
    fn convert_statements_with_spans(
        &self,
        stmts: &[Statement],
    ) -> ArenaVec<'a, oxc::Statement<'a>> {
        ArenaVec::from_iter_in(
            stmts.iter().map(|s| {
                let span = self.get_statement_span(s);
                let mut oxc_stmt = self.convert_statement(s);
                if span != SPAN {
                    set_statement_span(&mut oxc_stmt, span);
                }
                oxc_stmt
            }),
            self,
        )
    }

    /// Extract the span from a Babel AST Statement's base node.
    fn get_statement_span(&self, stmt: &Statement) -> Span {
        let base = match stmt {
            Statement::BlockStatement(s) => &s.base,
            Statement::ReturnStatement(s) => &s.base,
            Statement::ExpressionStatement(s) => &s.base,
            Statement::IfStatement(s) => &s.base,
            Statement::ForStatement(s) => &s.base,
            Statement::WhileStatement(s) => &s.base,
            Statement::DoWhileStatement(s) => &s.base,
            Statement::ForInStatement(s) => &s.base,
            Statement::ForOfStatement(s) => &s.base,
            Statement::SwitchStatement(s) => &s.base,
            Statement::ThrowStatement(s) => &s.base,
            Statement::TryStatement(s) => &s.base,
            Statement::BreakStatement(s) => &s.base,
            Statement::ContinueStatement(s) => &s.base,
            Statement::LabeledStatement(s) => &s.base,
            Statement::EmptyStatement(s) => &s.base,
            Statement::DebuggerStatement(s) => &s.base,
            Statement::WithStatement(s) => &s.base,
            Statement::VariableDeclaration(d) => &d.base,
            Statement::FunctionDeclaration(f) => &f.base,
            Statement::ClassDeclaration(c) => &c.base,
            Statement::ImportDeclaration(d) => &d.base,
            Statement::ExportNamedDeclaration(d) => &d.base,
            Statement::ExportDefaultDeclaration(d) => &d.base,
            Statement::ExportAllDeclaration(d) => &d.base,
            _ => return SPAN,
        };
        self.span_from_base(base)
    }

    fn convert_statement(&self, stmt: &Statement) -> oxc::Statement<'a> {
        match stmt {
            Statement::BlockStatement(s) => {
                oxc::Statement::new_block_statement(SPAN, self.convert_statement_vec(&s.body), self)
            }
            Statement::ReturnStatement(s) => oxc::Statement::new_return_statement(
                SPAN,
                s.argument.as_ref().map(|a| self.convert_expression(a)),
                self,
            ),
            Statement::ExpressionStatement(s) => oxc::Statement::new_expression_statement(
                SPAN,
                self.convert_expression(&s.expression),
                self,
            ),
            Statement::IfStatement(s) => oxc::Statement::new_if_statement(
                SPAN,
                self.convert_expression(&s.test),
                self.convert_statement(&s.consequent),
                s.alternate.as_ref().map(|a| self.convert_statement(a)),
                self,
            ),
            Statement::ForStatement(s) => {
                let init = s.init.as_ref().map(|i| self.convert_for_init(i));
                let test = s.test.as_ref().map(|t| self.convert_expression(t));
                let update = s.update.as_ref().map(|u| self.convert_expression(u));
                let body = self.convert_statement(&s.body);
                oxc::Statement::new_for_statement(SPAN, init, test, update, body, self)
            }
            Statement::WhileStatement(s) => oxc::Statement::new_while_statement(
                SPAN,
                self.convert_expression(&s.test),
                self.convert_statement(&s.body),
                self,
            ),
            Statement::DoWhileStatement(s) => oxc::Statement::new_do_while_statement(
                SPAN,
                self.convert_statement(&s.body),
                self.convert_expression(&s.test),
                self,
            ),
            Statement::ForInStatement(s) => oxc::Statement::new_for_in_statement(
                SPAN,
                self.convert_for_in_of_left(&s.left),
                self.convert_expression(&s.right),
                self.convert_statement(&s.body),
                self,
            ),
            Statement::ForOfStatement(s) => oxc::Statement::new_for_of_statement(
                SPAN,
                s.is_await,
                self.convert_for_in_of_left(&s.left),
                self.convert_expression(&s.right),
                self.convert_statement(&s.body),
                self,
            ),
            Statement::SwitchStatement(s) => {
                let cases = ArenaVec::from_iter_in(
                    s.cases.iter().map(|c| {
                        oxc::SwitchCase::new(
                            SPAN,
                            c.test.as_ref().map(|t| self.convert_expression(t)),
                            self.convert_statement_vec(&c.consequent),
                            self,
                        )
                    }),
                    self,
                );
                oxc::Statement::new_switch_statement(
                    SPAN,
                    self.convert_expression(&s.discriminant),
                    cases,
                    self,
                )
            }
            Statement::ThrowStatement(s) => oxc::Statement::new_throw_statement(
                SPAN,
                self.convert_expression(&s.argument),
                self,
            ),
            Statement::TryStatement(s) => {
                let block = self.convert_block_statement(&s.block);
                let handler = s.handler.as_ref().map(|h| self.convert_catch_clause(h));
                let finalizer = s.finalizer.as_ref().map(|f| self.convert_block_statement(f));
                oxc::Statement::new_try_statement(SPAN, block, handler, finalizer, self)
            }
            Statement::BreakStatement(s) => {
                let label = s
                    .label
                    .as_ref()
                    .map(|l| oxc::LabelIdentifier::new(SPAN, self.str(&l.name), self));
                oxc::Statement::new_break_statement(SPAN, label, self)
            }
            Statement::ContinueStatement(s) => {
                let label = s
                    .label
                    .as_ref()
                    .map(|l| oxc::LabelIdentifier::new(SPAN, self.str(&l.name), self));
                oxc::Statement::new_continue_statement(SPAN, label, self)
            }
            Statement::LabeledStatement(s) => {
                let label = oxc::LabelIdentifier::new(SPAN, self.str(&s.label.name), self);
                oxc::Statement::new_labeled_statement(
                    SPAN,
                    label,
                    self.convert_statement(&s.body),
                    self,
                )
            }
            Statement::EmptyStatement(_) => oxc::Statement::new_empty_statement(SPAN, self),
            Statement::DebuggerStatement(_) => oxc::Statement::new_debugger_statement(SPAN, self),
            Statement::WithStatement(s) => oxc::Statement::new_with_statement(
                SPAN,
                self.convert_expression(&s.object),
                self.convert_statement(&s.body),
                self,
            ),
            Statement::VariableDeclaration(d) => {
                let decl = self.convert_variable_declaration(d);
                oxc::Statement::VariableDeclaration(ArenaBox::new_in(decl, self))
            }
            Statement::FunctionDeclaration(f) => {
                let func = self.convert_function_decl(f, oxc::FunctionType::FunctionDeclaration);
                oxc::Statement::FunctionDeclaration(ArenaBox::new_in(func, self))
            }
            // The compiler never compiles classes, so the converter stubs out the
            // class body (members -> null forward, empty reverse). Re-parse the
            // original class from source to preserve its members; fall back to the
            // structural (member-less) conversion only if the slice is unavailable.
            Statement::ClassDeclaration(c) => {
                self.extract_source_stmt(&c.base).unwrap_or_else(|| {
                    oxc::Statement::ClassDeclaration(ArenaBox::new_in(
                        self.convert_class_declaration(c),
                        self,
                    ))
                })
            }
            Statement::ImportDeclaration(d) => {
                if d.specifiers.is_empty()
                    && let Some(stmt) = self.extract_source_stmt(&d.base)
                {
                    return stmt;
                }
                let decl = self.convert_import_declaration(d);
                oxc::Statement::ImportDeclaration(ArenaBox::new_in(decl, self))
            }
            Statement::ExportNamedDeclaration(d) => {
                if self.export_named_needs_source_stmt(d) {
                    if let Some(stmt) = self.extract_source_stmt(&d.base) {
                        return stmt;
                    }
                }
                let decl = self.convert_export_named_declaration(d);
                oxc::Statement::ExportNamedDeclaration(ArenaBox::new_in(decl, self))
            }
            Statement::ExportDefaultDeclaration(d) => {
                if self.export_default_needs_source_stmt(d) {
                    return self.extract_source_stmt(&d.base).unwrap_or_else(|| {
                        if matches!(d.declaration.as_ref(), ExportDefaultDecl::Expression(_)) {
                            oxc::Statement::new_empty_statement(SPAN, self)
                        } else {
                            let decl = self.convert_export_default_declaration(d);
                            oxc::Statement::ExportDefaultDeclaration(ArenaBox::new_in(decl, self))
                        }
                    });
                }
                let decl = self.convert_export_default_declaration(d);
                oxc::Statement::ExportDefaultDeclaration(ArenaBox::new_in(decl, self))
            }
            Statement::ExportAllDeclaration(d) => {
                if let Some(stmt) = self.extract_source_stmt(&d.base) {
                    return stmt;
                }
                let decl = self.convert_export_all_declaration(d);
                oxc::Statement::ExportAllDeclaration(ArenaBox::new_in(decl, self))
            }
            // TS/Flow declarations - try to extract from source text, fall back to empty
            Statement::TSTypeAliasDeclaration(d) => self
                .extract_source_stmt(&d.base)
                .unwrap_or_else(|| oxc::Statement::new_empty_statement(SPAN, self)),
            Statement::TSInterfaceDeclaration(d) => self
                .extract_source_stmt(&d.base)
                .unwrap_or_else(|| oxc::Statement::new_empty_statement(SPAN, self)),
            Statement::TSEnumDeclaration(d) => self
                .extract_source_stmt(&d.base)
                .unwrap_or_else(|| oxc::Statement::new_empty_statement(SPAN, self)),
            Statement::TSModuleDeclaration(d) => self
                .extract_source_stmt(&d.base)
                .unwrap_or_else(|| oxc::Statement::new_empty_statement(SPAN, self)),
            Statement::TSDeclareFunction(d) => self
                .extract_source_stmt(&d.base)
                .unwrap_or_else(|| oxc::Statement::new_empty_statement(SPAN, self)),
            Statement::TypeAlias(d) => self
                .extract_source_stmt(&d.base)
                .unwrap_or_else(|| oxc::Statement::new_empty_statement(SPAN, self)),
            Statement::OpaqueType(d) => self
                .extract_source_stmt(&d.base)
                .unwrap_or_else(|| oxc::Statement::new_empty_statement(SPAN, self)),
            Statement::InterfaceDeclaration(d) => self
                .extract_source_stmt(&d.base)
                .unwrap_or_else(|| oxc::Statement::new_empty_statement(SPAN, self)),
            Statement::EnumDeclaration(d) => self
                .extract_source_stmt(&d.base)
                .unwrap_or_else(|| oxc::Statement::new_empty_statement(SPAN, self)),
            Statement::DeclareVariable(_)
            | Statement::DeclareFunction(_)
            | Statement::DeclareClass(_)
            | Statement::DeclareModule(_)
            | Statement::DeclareModuleExports(_)
            | Statement::DeclareExportDeclaration(_)
            | Statement::DeclareExportAllDeclaration(_)
            | Statement::DeclareInterface(_)
            | Statement::DeclareTypeAlias(_)
            | Statement::DeclareOpaqueType(_) => oxc::Statement::new_empty_statement(SPAN, self),
            Statement::Unknown(s) => self
                .extract_source_stmt(s.base())
                .unwrap_or_else(|| oxc::Statement::new_empty_statement(SPAN, self)),
        }
    }

    fn convert_statement_vec(&self, stmts: &[Statement]) -> ArenaVec<'a, oxc::Statement<'a>> {
        ArenaVec::from_iter_in(stmts.iter().map(|s| self.convert_statement(s)), self)
    }

    fn convert_block_statement(&self, block: &BlockStatement) -> oxc::BlockStatement<'a> {
        oxc::BlockStatement::new(SPAN, self.convert_statement_vec(&block.body), self)
    }

    fn convert_catch_clause(&self, clause: &CatchClause) -> oxc::CatchClause<'a> {
        let param = clause.param.as_ref().map(|p| {
            let pattern = self.convert_pattern_to_binding_pattern(p);
            let type_annotation = self.pattern_type_annotation(p);
            oxc::CatchParameter::new(SPAN, pattern, type_annotation, self)
        });
        oxc::CatchClause::new(SPAN, param, self.convert_block_statement(&clause.body), self)
    }

    fn convert_for_init(&self, init: &ForInit) -> oxc::ForStatementInit<'a> {
        match init {
            ForInit::VariableDeclaration(v) => {
                let decl = self.convert_variable_declaration(v);
                oxc::ForStatementInit::VariableDeclaration(ArenaBox::new_in(decl, self))
            }
            ForInit::Expression(e) => oxc::ForStatementInit::from(self.convert_expression(e)),
        }
    }

    fn convert_for_in_of_left(&self, left: &ForInOfLeft) -> oxc::ForStatementLeft<'a> {
        match left {
            ForInOfLeft::VariableDeclaration(v) => {
                let decl = self.convert_variable_declaration(v);
                oxc::ForStatementLeft::VariableDeclaration(ArenaBox::new_in(decl, self))
            }
            ForInOfLeft::Pattern(p) => {
                let target = self.convert_pattern_to_assignment_target(p);
                oxc::ForStatementLeft::from(target)
            }
        }
    }

    fn convert_variable_declaration(
        &self,
        decl: &VariableDeclaration,
    ) -> oxc::VariableDeclaration<'a> {
        let kind = match decl.kind {
            VariableDeclarationKind::Var => oxc::VariableDeclarationKind::Var,
            VariableDeclarationKind::Let => oxc::VariableDeclarationKind::Let,
            VariableDeclarationKind::Const => oxc::VariableDeclarationKind::Const,
            VariableDeclarationKind::Using => oxc::VariableDeclarationKind::Using,
        };
        let declarators = ArenaVec::from_iter_in(
            decl.declarations.iter().map(|d| self.convert_variable_declarator(d, kind)),
            self,
        );
        let declare = decl.declare.unwrap_or(false);
        oxc::VariableDeclaration::new(SPAN, kind, declarators, declare, self)
    }

    fn convert_variable_declarator(
        &self,
        d: &VariableDeclarator,
        kind: oxc::VariableDeclarationKind,
    ) -> oxc::VariableDeclarator<'a> {
        let id = self.convert_pattern_to_binding_pattern(&d.id);
        let definite = d.definite.unwrap_or(false);
        let source_declarator = self.extract_source_variable_declarator(&d.base, kind);
        let (type_annotation, source_init) = source_declarator.map_or((None, None), |d| {
            let oxc::VariableDeclarator { type_annotation, init, .. } = d;
            (type_annotation, init)
        });
        let init = d.init.as_ref().map(|e| {
            let converted = self.convert_expression(e);
            match source_init {
                Some(source_init) => {
                    match self.wrap_expression_with_source_ts_expr(source_init, converted) {
                        Ok(wrapped) => wrapped,
                        Err(converted) => converted,
                    }
                }
                None => converted,
            }
        });
        oxc::VariableDeclarator::new(SPAN, kind, id, type_annotation, init, definite, self)
    }

    // ===== Expressions =====

    fn convert_expression(&self, expr: &Expression) -> oxc::Expression<'a> {
        let converted = match expr {
            Expression::Identifier(id) => {
                oxc::Expression::new_identifier(SPAN, self.str(&id.name), self)
            }
            Expression::StringLiteral(lit) => oxc::Expression::new_string_literal(
                SPAN,
                self.str(&lit.value.to_string_lossy()),
                None,
                self,
            ),
            Expression::NumericLiteral(lit) => oxc::Expression::new_numeric_literal(
                SPAN,
                lit.value,
                None,
                oxc::NumberBase::Decimal,
                self,
            ),
            Expression::BooleanLiteral(lit) => {
                oxc::Expression::new_boolean_literal(SPAN, lit.value, self)
            }
            Expression::NullLiteral(_) => oxc::Expression::new_null_literal(SPAN, self),
            Expression::BigIntLiteral(lit) => oxc::Expression::new_big_int_literal(
                SPAN,
                self.str(lit.value.strip_suffix('n').unwrap_or(&lit.value)),
                None,
                oxc::BigintBase::Decimal,
                self,
            ),
            Expression::RegExpLiteral(lit) => {
                let flags = self.parse_regexp_flags(&lit.flags);
                let pattern = oxc::RegExpPattern { text: self.str(&lit.pattern), pattern: None };
                let regex = oxc::RegExp { pattern, flags };
                oxc::Expression::new_reg_exp_literal(SPAN, regex, None, self)
            }
            Expression::CallExpression(call) => {
                if let Some(import_expr) = self.convert_dynamic_import_call(call) {
                    return import_expr;
                }
                let callee = self.convert_expression(&call.callee);
                let args = self.convert_arguments_with_source(
                    &call.arguments,
                    self.extract_source_call_arguments(&call.base),
                );
                let type_arguments =
                    if call.type_parameters.is_some() || call.type_arguments.is_some() {
                        self.extract_source_call_type_arguments(&call.base)
                    } else {
                        None
                    };
                oxc::Expression::new_call_expression(
                    SPAN,
                    callee,
                    type_arguments,
                    args,
                    false,
                    self,
                )
            }
            Expression::MemberExpression(m) => self.convert_member_expression(m),
            Expression::OptionalCallExpression(call) => {
                let callee = self.convert_expression_for_chain(&call.callee);
                let args = self.convert_arguments_with_source(
                    &call.arguments,
                    self.extract_source_call_arguments(&call.base),
                );
                let type_arguments =
                    if call.type_parameters.is_some() || call.type_arguments.is_some() {
                        self.extract_source_call_type_arguments(&call.base)
                    } else {
                        None
                    };
                let chain_call = oxc::ChainElement::new_call_expression(
                    SPAN,
                    callee,
                    type_arguments,
                    args,
                    call.optional,
                    self,
                );
                oxc::Expression::new_chain_expression(SPAN, chain_call, self)
            }
            Expression::OptionalMemberExpression(m) => {
                let chain_elem = self.convert_optional_member_to_chain_element(m);
                oxc::Expression::new_chain_expression(SPAN, chain_elem, self)
            }
            Expression::BinaryExpression(bin) => {
                let op = self.convert_binary_operator(&bin.operator);
                oxc::Expression::new_binary_expression(
                    SPAN,
                    self.convert_expression(&bin.left),
                    op,
                    self.convert_expression(&bin.right),
                    self,
                )
            }
            Expression::LogicalExpression(log) => {
                let op = self.convert_logical_operator(&log.operator);
                oxc::Expression::new_logical_expression(
                    SPAN,
                    self.convert_expression(&log.left),
                    op,
                    self.convert_expression(&log.right),
                    self,
                )
            }
            Expression::UnaryExpression(un) => {
                let op = self.convert_unary_operator(&un.operator);
                oxc::Expression::new_unary_expression(
                    SPAN,
                    op,
                    self.convert_expression(&un.argument),
                    self,
                )
            }
            Expression::UpdateExpression(up) => {
                let op = self.convert_update_operator(&up.operator);
                let arg = self.convert_expression_to_simple_assignment_target(&up.argument);
                oxc::Expression::new_update_expression(SPAN, op, up.prefix, arg, self)
            }
            Expression::ConditionalExpression(cond) => oxc::Expression::new_conditional_expression(
                SPAN,
                self.convert_expression(&cond.test),
                self.convert_expression(&cond.consequent),
                self.convert_expression(&cond.alternate),
                self,
            ),
            Expression::AssignmentExpression(assign) => {
                let op = self.convert_assignment_operator(&assign.operator);
                let left = self.convert_pattern_to_assignment_target(&assign.left);
                oxc::Expression::new_assignment_expression(
                    SPAN,
                    op,
                    left,
                    self.convert_expression(&assign.right),
                    self,
                )
            }
            Expression::SequenceExpression(seq) => {
                let exprs = ArenaVec::from_iter_in(
                    seq.expressions.iter().map(|e| self.convert_expression(e)),
                    self,
                );
                oxc::Expression::new_sequence_expression(SPAN, exprs, self)
            }
            Expression::ArrowFunctionExpression(arrow) => self.convert_arrow_function(arrow),
            Expression::FunctionExpression(func) => {
                let f = self.convert_function_expr(func);
                oxc::Expression::FunctionExpression(ArenaBox::new_in(f, self))
            }
            Expression::ObjectExpression(obj) => {
                let properties = ArenaVec::from_iter_in(
                    obj.properties.iter().map(|p| self.convert_object_expression_property(p)),
                    self,
                );
                oxc::Expression::new_object_expression(SPAN, properties, self)
            }
            Expression::ArrayExpression(arr) => {
                let elements = ArenaVec::from_iter_in(
                    arr.elements.iter().map(|e| self.convert_array_element(e)),
                    self,
                );
                oxc::Expression::new_array_expression(SPAN, elements, self)
            }
            Expression::NewExpression(n) => {
                let callee = self.convert_expression(&n.callee);
                let args = self.convert_arguments_with_source(
                    &n.arguments,
                    self.extract_source_new_arguments(&n.base),
                );
                let type_arguments = if n.type_parameters.is_some() || n.type_arguments.is_some() {
                    self.extract_source_new_type_arguments(&n.base)
                } else {
                    None
                };
                oxc::Expression::new_new_expression(SPAN, callee, type_arguments, args, self)
            }
            Expression::TemplateLiteral(tl) => {
                let template = self.convert_template_literal(tl);
                oxc::Expression::TemplateLiteral(ArenaBox::new_in(template, self))
            }
            Expression::TaggedTemplateExpression(tag) => {
                let t = self.convert_expression(&tag.tag);
                let quasi = self.convert_template_literal(&tag.quasi);
                let type_arguments = if tag.type_parameters.is_some() {
                    self.extract_source_tagged_template_type_arguments(&tag.base)
                } else {
                    None
                };
                oxc::Expression::new_tagged_template_expression(
                    SPAN,
                    t,
                    type_arguments,
                    quasi,
                    self,
                )
            }
            Expression::AwaitExpression(a) => oxc::Expression::new_await_expression(
                SPAN,
                self.convert_expression(&a.argument),
                self,
            ),
            Expression::YieldExpression(y) => oxc::Expression::new_yield_expression(
                SPAN,
                y.delegate,
                y.argument.as_ref().map(|a| self.convert_expression(a)),
                self,
            ),
            Expression::SpreadElement(s) => {
                // SpreadElement can't be a standalone expression in OXC.
                // Return the argument directly as a fallback.
                self.convert_expression(&s.argument)
            }
            Expression::MetaProperty(mp) => {
                let meta = oxc::IdentifierName::new(SPAN, self.str(&mp.meta.name), self);
                let property = oxc::IdentifierName::new(SPAN, self.str(&mp.property.name), self);
                oxc::Expression::new_meta_property(SPAN, meta, property, self)
            }
            Expression::ClassExpression(c) => {
                if let Some(expr) = self.extract_source_class_expression(c) {
                    return expr;
                }
                let class = self.convert_class_to_oxc(c, oxc::ClassType::ClassExpression);
                oxc::Expression::ClassExpression(ArenaBox::new_in(class, self))
            }
            Expression::PrivateName(_) => {
                oxc::Expression::new_identifier(SPAN, self.str("__private__"), self)
            }
            Expression::Super(_) => oxc::Expression::new_super(SPAN, self),
            Expression::Import(_) => {
                oxc::Expression::new_identifier(SPAN, self.str("__import__"), self)
            }
            Expression::ThisExpression(_) => oxc::Expression::new_this_expression(SPAN, self),
            Expression::ParenthesizedExpression(p) => {
                oxc::Expression::new_parenthesized_expression(
                    SPAN,
                    self.convert_expression(&p.expression),
                    self,
                )
            }
            Expression::JSXElement(el) => {
                let element = self.convert_jsx_element(el);
                oxc::Expression::JSXElement(ArenaBox::new_in(element, self))
            }
            Expression::JSXFragment(frag) => {
                let fragment = self.convert_jsx_fragment(frag);
                oxc::Expression::JSXFragment(ArenaBox::new_in(fragment, self))
            }
            // TS expressions carry their actual type AST as `null` through the
            // React AST bridge. Rebuild the wrapper with the converted child
            // expression and recover only the type from the original source.
            Expression::TSAsExpression(e) => {
                let expression = self.convert_expression(&e.expression);
                let parsed_type = e.type_annotation.parse_value();
                let type_annotation = self.convert_ts_type_from_json(&parsed_type).or_else(|| {
                    if Self::ts_type_json_contains_type_query(&parsed_type) {
                        None
                    } else {
                        self.extract_source_ts_as_type(&e.base)
                    }
                });
                if let Some(type_annotation) = type_annotation {
                    oxc::Expression::new_ts_as_expression(SPAN, expression, type_annotation, self)
                } else {
                    expression
                }
            }
            Expression::TSSatisfiesExpression(e) => {
                let expression = self.convert_expression(&e.expression);
                let parsed_type = e.type_annotation.parse_value();
                let type_annotation = self.convert_ts_type_from_json(&parsed_type).or_else(|| {
                    if Self::ts_type_json_contains_type_query(&parsed_type) {
                        None
                    } else {
                        self.extract_source_ts_satisfies_type(&e.base)
                    }
                });
                if let Some(type_annotation) = type_annotation {
                    oxc::Expression::new_ts_satisfies_expression(
                        SPAN,
                        expression,
                        type_annotation,
                        self,
                    )
                } else {
                    expression
                }
            }
            Expression::TSNonNullExpression(e) => oxc::Expression::new_ts_non_null_expression(
                SPAN,
                self.convert_expression(&e.expression),
                self,
            ),
            Expression::TSTypeAssertion(e) => {
                let expression = self.convert_expression(&e.expression);
                let parsed_type = e.type_annotation.parse_value();
                let type_annotation = self.convert_ts_type_from_json(&parsed_type).or_else(|| {
                    if Self::ts_type_json_contains_type_query(&parsed_type) {
                        None
                    } else {
                        self.extract_source_ts_type_assertion_type(&e.base)
                    }
                });
                if let Some(type_annotation) = type_annotation {
                    oxc::Expression::new_ts_type_assertion(SPAN, type_annotation, expression, self)
                } else {
                    expression
                }
            }
            Expression::TSInstantiationExpression(e) => {
                let expression = self.convert_expression(&e.expression);
                if let Some(type_arguments) =
                    self.extract_source_ts_instantiation_type_arguments(&e.base)
                {
                    oxc::Expression::new_ts_instantiation_expression(
                        SPAN,
                        expression,
                        type_arguments,
                        self,
                    )
                } else {
                    expression
                }
            }
            Expression::TypeCastExpression(e) => self.convert_expression(&e.expression),
            Expression::AssignmentPattern(p) => {
                let left = self.convert_pattern_to_assignment_target(&p.left);
                oxc::Expression::new_assignment_expression(
                    SPAN,
                    OxcAssOp::Assign,
                    left,
                    self.convert_expression(&p.right),
                    self,
                )
            }
        };

        if let Some(base) = Self::expression_base(expr) {
            match self.wrap_expression_with_source_ts(base, converted) {
                Ok(wrapped) => wrapped,
                Err(converted) => converted,
            }
        } else {
            converted
        }
    }

    fn convert_dynamic_import_call(&self, call: &CallExpression) -> Option<oxc::Expression<'a>> {
        if !matches!(call.callee.as_ref(), Expression::Import(_)) {
            return None;
        }
        let source = call.arguments.first().map(|arg| self.convert_expression(arg))?;
        let options = call.arguments.get(1).map(|arg| self.convert_expression(arg));
        Some(oxc::Expression::new_import_expression(SPAN, source, options, None, self))
    }

    /// Convert an expression that may be used inside a chain (optional chaining).
    fn convert_expression_for_chain(&self, expr: &Expression) -> oxc::Expression<'a> {
        match expr {
            Expression::OptionalMemberExpression(m) => {
                self.convert_optional_member_to_expression(m)
            }
            Expression::OptionalCallExpression(call) => {
                let callee = self.convert_expression_for_chain(&call.callee);
                let args = self.convert_arguments_with_source(
                    &call.arguments,
                    self.extract_source_call_arguments(&call.base),
                );
                let type_arguments =
                    if call.type_parameters.is_some() || call.type_arguments.is_some() {
                        self.extract_source_call_type_arguments(&call.base)
                    } else {
                        None
                    };
                let call_expr = oxc::CallExpression::new(
                    SPAN,
                    callee,
                    type_arguments,
                    args,
                    call.optional,
                    self,
                );
                oxc::Expression::CallExpression(ArenaBox::new_in(call_expr, self))
            }
            _ => self.convert_expression(expr),
        }
    }

    fn convert_member_expression(&self, m: &MemberExpression) -> oxc::Expression<'a> {
        let object = self.convert_expression(&m.object);
        if m.computed {
            let property = self.convert_expression(&m.property);
            oxc::Expression::new_computed_member_expression(SPAN, object, property, false, self)
        } else {
            let prop_name = self.expression_to_identifier_name(&m.property);
            oxc::Expression::new_static_member_expression(SPAN, object, prop_name, false, self)
        }
    }

    fn convert_optional_member_to_chain_element(
        &self,
        m: &OptionalMemberExpression,
    ) -> oxc::ChainElement<'a> {
        let object = self.convert_expression_for_chain(&m.object);
        if m.computed {
            let property = self.convert_expression(&m.property);
            oxc::ChainElement::new_computed_member_expression(
                SPAN, object, property, m.optional, self,
            )
        } else {
            let prop_name = self.expression_to_identifier_name(&m.property);
            oxc::ChainElement::new_static_member_expression(
                SPAN, object, prop_name, m.optional, self,
            )
        }
    }

    fn convert_optional_member_to_expression(
        &self,
        m: &OptionalMemberExpression,
    ) -> oxc::Expression<'a> {
        let object = self.convert_expression_for_chain(&m.object);
        if m.computed {
            let property = self.convert_expression(&m.property);
            oxc::Expression::new_computed_member_expression(
                SPAN, object, property, m.optional, self,
            )
        } else {
            let prop_name = self.expression_to_identifier_name(&m.property);
            oxc::Expression::new_static_member_expression(SPAN, object, prop_name, m.optional, self)
        }
    }

    fn expression_to_identifier_name(&self, expr: &Expression) -> oxc::IdentifierName<'a> {
        match expr {
            Expression::Identifier(id) => oxc::IdentifierName::new(SPAN, self.str(&id.name), self),
            _ => oxc::IdentifierName::new(SPAN, self.str("__unknown__"), self),
        }
    }

    fn convert_arguments_with_source(
        &self,
        args: &[Expression],
        source_args: Option<Vec<oxc::Argument<'a>>>,
    ) -> ArenaVec<'a, oxc::Argument<'a>> {
        let mut source_args = source_args.map(Vec::into_iter);
        ArenaVec::from_iter_in(
            args.iter().map(|arg| {
                let source_arg = source_args.as_mut().and_then(Iterator::next);
                self.convert_argument_with_source(arg, source_arg)
            }),
            self,
        )
    }

    fn convert_argument_with_source(
        &self,
        arg: &Expression,
        source_arg: Option<oxc::Argument<'a>>,
    ) -> oxc::Argument<'a> {
        match arg {
            Expression::SpreadElement(s) => {
                let converted = self.convert_expression(&s.argument);
                let converted = match source_arg {
                    Some(oxc::Argument::SpreadElement(source_spread)) => {
                        match self.wrap_expression_with_source_ts_expr(
                            source_spread.unbox().argument,
                            converted,
                        ) {
                            Ok(wrapped) => wrapped,
                            Err(converted) => converted,
                        }
                    }
                    Some(source_arg) => {
                        match self.wrap_expression_with_source_argument_ts(source_arg, converted) {
                            Ok(wrapped) => wrapped,
                            Err(converted) => converted,
                        }
                    }
                    None => converted,
                };
                oxc::Argument::new_spread_element(SPAN, converted, self)
            }
            _ => {
                let converted = self.convert_expression(arg);
                let converted = match source_arg {
                    Some(source_arg) => {
                        match self.wrap_expression_with_source_argument_ts(source_arg, converted) {
                            Ok(wrapped) => wrapped,
                            Err(converted) => converted,
                        }
                    }
                    None => converted,
                };
                oxc::Argument::from(converted)
            }
        }
    }

    fn convert_array_element(&self, elem: &Option<Expression>) -> oxc::ArrayExpressionElement<'a> {
        match elem {
            None => oxc::ArrayExpressionElement::new_elision(SPAN, self),
            Some(Expression::SpreadElement(s)) => oxc::ArrayExpressionElement::new_spread_element(
                SPAN,
                self.convert_expression(&s.argument),
                self,
            ),
            Some(e) => oxc::ArrayExpressionElement::from(self.convert_expression(e)),
        }
    }

    fn convert_object_expression_property(
        &self,
        prop: &ObjectExpressionProperty,
    ) -> oxc::ObjectPropertyKind<'a> {
        match prop {
            ObjectExpressionProperty::ObjectProperty(p) => {
                let key = self.convert_expression_to_property_key(&p.key, p.computed);
                let value = self.convert_expression(&p.value);
                let value = match self.extract_source_object_property_value(&p.base) {
                    Some(source_value) => {
                        match self.wrap_expression_with_source_ts_expr(source_value, value) {
                            Ok(wrapped) => wrapped,
                            Err(value) => value,
                        }
                    }
                    None => value,
                };
                let method = p.method.unwrap_or(false);
                let obj_prop = oxc::ObjectProperty::new(
                    SPAN,
                    oxc::PropertyKind::Init,
                    key,
                    value,
                    method,
                    p.shorthand,
                    p.computed,
                    self,
                );
                oxc::ObjectPropertyKind::ObjectProperty(ArenaBox::new_in(obj_prop, self))
            }
            ObjectExpressionProperty::ObjectMethod(m) => {
                let kind = match m.kind {
                    ObjectMethodKind::Method => oxc::PropertyKind::Init,
                    ObjectMethodKind::Get => oxc::PropertyKind::Get,
                    ObjectMethodKind::Set => oxc::PropertyKind::Set,
                };
                let key = self.convert_expression_to_property_key(&m.key, m.computed);
                let func = self.convert_object_method_to_function(m);
                let func_expr = oxc::Expression::FunctionExpression(ArenaBox::new_in(func, self));
                let obj_prop = oxc::ObjectProperty::new(
                    SPAN, kind, key, func_expr, m.method, false, // shorthand
                    m.computed, self,
                );
                oxc::ObjectPropertyKind::ObjectProperty(ArenaBox::new_in(obj_prop, self))
            }
            ObjectExpressionProperty::SpreadElement(s) => {
                let spread =
                    oxc::SpreadElement::new(SPAN, self.convert_expression(&s.argument), self);
                oxc::ObjectPropertyKind::SpreadProperty(ArenaBox::new_in(spread, self))
            }
        }
    }

    fn convert_expression_to_property_key(
        &self,
        expr: &Expression,
        computed: bool,
    ) -> oxc::PropertyKey<'a> {
        // A computed key (`{ [expr]: … }`) is an arbitrary expression evaluated at
        // runtime, so its identifiers are *references* — `{ [CONST]: x }` reads the
        // variable `CONST`. Build it from the expression so semantic analysis links
        // those references; otherwise an imported `CONST` looks unused and import
        // elision drops it, leaving the emitted `[CONST]` dangling. Only a
        // non-computed key is a static property name or literal.
        if computed {
            return oxc::PropertyKey::from(self.convert_expression(expr));
        }
        match expr {
            Expression::Identifier(id) => {
                oxc::PropertyKey::new_static_identifier(SPAN, self.str(&id.name), self)
            }
            Expression::StringLiteral(s) => {
                let lit =
                    oxc::StringLiteral::new(SPAN, self.str(&s.value.to_string_lossy()), None, self);
                oxc::PropertyKey::StringLiteral(ArenaBox::new_in(lit, self))
            }
            Expression::NumericLiteral(n) => {
                let lit =
                    oxc::NumericLiteral::new(SPAN, n.value, None, oxc::NumberBase::Decimal, self);
                oxc::PropertyKey::NumericLiteral(ArenaBox::new_in(lit, self))
            }
            Expression::PrivateName(p) => {
                oxc::PropertyKey::new_private_identifier(SPAN, self.str(&p.id.name), self)
            }
            _ => oxc::PropertyKey::from(self.convert_expression(expr)),
        }
    }

    fn convert_template_literal(&self, tl: &TemplateLiteral) -> oxc::TemplateLiteral<'a> {
        let quasis = ArenaVec::from_iter_in(
            tl.quasis.iter().map(|q| {
                let raw = self.str(&q.value.raw);
                let cooked = q.value.cooked.as_ref().map(|c| self.str(c));
                let value = oxc::TemplateElementValue { raw, cooked };
                oxc::TemplateElement::new(SPAN, value, q.tail, self)
            }),
            self,
        );
        let expressions =
            ArenaVec::from_iter_in(tl.expressions.iter().map(|e| self.convert_expression(e)), self);
        oxc::TemplateLiteral::new(SPAN, quasis, expressions, self)
    }

    // ===== Functions =====

    fn convert_function_decl(
        &self,
        f: &FunctionDeclaration,
        fn_type: oxc::FunctionType,
    ) -> oxc::Function<'a> {
        let id =
            f.id.as_ref().map(|id| oxc::BindingIdentifier::new(SPAN, self.str(&id.name), self));
        let params = self.convert_params_to_formal_parameters(&f.params);
        let body = self.convert_block_to_function_body(&f.body);
        let mut func = oxc::Function::new(
            SPAN,
            fn_type,
            id,
            f.generator,
            f.is_async,
            f.declare.unwrap_or(false),
            None::<ArenaBox<'a, oxc::TSTypeParameterDeclaration<'a>>>,
            None::<ArenaBox<'a, oxc::TSThisParameter<'a>>>,
            params,
            None::<ArenaBox<'a, oxc::TSTypeAnnotation<'a>>>,
            Some(body),
            self,
        );
        if !self.block_initializes_react_cache(&f.body)
            && let Some(source_func) = self.extract_source_function_declaration(&f.base)
        {
            let source_has_no_body = source_func.body.is_none();
            self.apply_function_signature_from_source(&mut func, source_func);
            if source_has_no_body {
                func.body = None;
            }
        }
        func
    }

    fn convert_class_declaration(&self, c: &ClassDeclaration) -> oxc::Class<'a> {
        let id =
            c.id.as_ref().map(|id| oxc::BindingIdentifier::new(SPAN, self.str(&id.name), self));
        let super_class = c.super_class.as_ref().map(|s| self.convert_expression(s));
        let body = oxc::ClassBody::new(SPAN, ArenaVec::new_in(self), self);
        oxc::Class::new(
            SPAN,
            oxc::ClassType::ClassDeclaration,
            ArenaVec::new_in(self), // decorators
            id,
            None::<ArenaBox<'a, oxc::TSTypeParameterDeclaration<'a>>>,
            super_class,
            None::<ArenaBox<'a, oxc::TSTypeParameterInstantiation<'a>>>,
            ArenaVec::new_in(self), // implements
            body,
            c.is_abstract.unwrap_or(false),
            c.declare.unwrap_or(false),
            self,
        )
    }

    fn convert_class_to_oxc(
        &self,
        c: &ClassExpression,
        class_type: oxc::ClassType,
    ) -> oxc::Class<'a> {
        let id =
            c.id.as_ref().map(|id| oxc::BindingIdentifier::new(SPAN, self.str(&id.name), self));
        let super_class = c.super_class.as_ref().map(|s| self.convert_expression(s));
        let body = oxc::ClassBody::new(SPAN, ArenaVec::new_in(self), self);
        oxc::Class::new(
            SPAN,
            class_type,
            ArenaVec::new_in(self), // decorators
            id,
            None::<ArenaBox<'a, oxc::TSTypeParameterDeclaration<'a>>>,
            super_class,
            None::<ArenaBox<'a, oxc::TSTypeParameterInstantiation<'a>>>,
            ArenaVec::new_in(self), // implements
            body,
            false, // is_abstract
            false, /* declare */
            self,
        )
    }

    fn convert_function_expr(&self, f: &FunctionExpression) -> oxc::Function<'a> {
        let id =
            f.id.as_ref().map(|id| oxc::BindingIdentifier::new(SPAN, self.str(&id.name), self));
        let params = self.convert_params_to_formal_parameters(&f.params);
        let body = self.convert_block_to_function_body(&f.body);
        let initializes_react_cache = self.block_initializes_react_cache(&f.body);
        let return_type = if initializes_react_cache {
            None
        } else {
            f.return_type
                .as_ref()
                .and_then(|value| self.convert_ts_type_annotation_from_json(&value.parse_value()))
        };
        let mut func = oxc::Function::new(
            SPAN,
            oxc::FunctionType::FunctionExpression,
            id,
            f.generator,
            f.is_async,
            false,
            None::<ArenaBox<'a, oxc::TSTypeParameterDeclaration<'a>>>,
            None::<ArenaBox<'a, oxc::TSThisParameter<'a>>>,
            params,
            return_type,
            Some(body),
            self,
        );
        if !initializes_react_cache
            && let Some(source_func) = self.extract_source_function_expression(&f.base)
        {
            self.apply_function_signature_from_source(&mut func, source_func);
        }
        func
    }

    fn convert_object_method_to_function(&self, m: &ObjectMethod) -> oxc::Function<'a> {
        let params = self.convert_params_to_formal_parameters(&m.params);
        let body = self.convert_block_to_function_body(&m.body);
        let initializes_react_cache = self.block_initializes_react_cache(&m.body);
        let return_type = if initializes_react_cache {
            None
        } else {
            m.return_type
                .as_ref()
                .and_then(|value| self.convert_ts_type_annotation_from_json(&value.parse_value()))
        };
        let mut func = oxc::Function::new(
            SPAN,
            oxc::FunctionType::FunctionExpression,
            None,
            m.generator,
            m.is_async,
            false,
            None::<ArenaBox<'a, oxc::TSTypeParameterDeclaration<'a>>>,
            None::<ArenaBox<'a, oxc::TSThisParameter<'a>>>,
            params,
            return_type,
            Some(body),
            self,
        );
        if !initializes_react_cache
            && let Some(source_func) = self.extract_source_object_method_function(&m.base)
        {
            self.apply_function_signature_from_source(&mut func, source_func);
        }
        func
    }

    fn convert_arrow_function(&self, arrow: &ArrowFunctionExpression) -> oxc::Expression<'a> {
        let is_expression = arrow.expression.unwrap_or(false);
        let params = self.convert_params_to_formal_parameters(&arrow.params);
        let arrow_initializes_react_cache = match &*arrow.body {
            ArrowFunctionBody::BlockStatement(block) => self.block_initializes_react_cache(block),
            ArrowFunctionBody::Expression(_) => false,
        };

        let body = match &*arrow.body {
            ArrowFunctionBody::BlockStatement(block) => self.convert_block_to_function_body(block),
            ArrowFunctionBody::Expression(expr) => {
                let oxc_expr = self.convert_expression(expr);
                let stmt = oxc::Statement::new_expression_statement(SPAN, oxc_expr, self);
                let stmts = ArenaVec::from_iter_in(once(stmt), self);
                oxc::FunctionBody::new(SPAN, ArenaVec::new_in(self), stmts, self)
            }
        };

        let mut arrow_expr = oxc::ArrowFunctionExpression::new(
            SPAN,
            is_expression,
            arrow.is_async,
            None::<ArenaBox<'a, oxc::TSTypeParameterDeclaration<'a>>>,
            params,
            if arrow_initializes_react_cache {
                None
            } else {
                arrow.return_type.as_ref().and_then(|value| {
                    self.convert_ts_type_annotation_from_json(&value.parse_value())
                })
            },
            body,
            self,
        );
        if !arrow_initializes_react_cache
            && let Some(source_arrow) = self.extract_source_arrow_function(&arrow.base)
        {
            self.apply_arrow_signature_from_source(&mut arrow_expr, source_arrow);
        }
        oxc::Expression::ArrowFunctionExpression(ArenaBox::new_in(arrow_expr, self))
    }

    fn convert_block_to_function_body(&self, block: &BlockStatement) -> oxc::FunctionBody<'a> {
        let stmts = self.convert_statement_vec(&block.body);
        let directives = self.convert_directives(&block.directives);
        oxc::FunctionBody::new(SPAN, directives, stmts, self)
    }

    fn convert_params_to_formal_parameters(
        &self,
        params: &[PatternLike],
    ) -> oxc::FormalParameters<'a> {
        let mut items: Vec<oxc::FormalParameter<'a>> = Vec::new();
        let mut rest: Option<oxc::FormalParameterRest<'a>> = None;

        for param in params {
            match param {
                PatternLike::RestElement(r) => {
                    let arg = self.convert_pattern_to_binding_pattern(&r.argument);
                    let rest_elem = oxc::BindingRestElement::new(SPAN, arg, self);
                    let type_annotation = r.type_annotation.as_ref().and_then(|value| {
                        self.convert_ts_type_annotation_from_json(&value.parse_value())
                    });
                    rest = Some(oxc::FormalParameterRest::new(
                        SPAN,
                        ArenaVec::new_in(self),
                        rest_elem,
                        type_annotation,
                        self,
                    ));
                }
                PatternLike::AssignmentPattern(ap) => {
                    // OXC stores default parameter values in FormalParameter.initializer
                    // rather than using BindingPattern::AssignmentPattern (which OXC considers
                    // invalid in FormalParameter position).
                    let left = self.convert_pattern_to_binding_pattern(&ap.left);
                    let right = self.convert_expression(&ap.right);
                    let initializer = Some(ArenaBox::new_in(right, &self.allocator));
                    let type_annotation = self
                        .pattern_type_annotation(param)
                        .or_else(|| self.pattern_type_annotation(&ap.left));
                    let optional = self.pattern_optional(param) || self.pattern_optional(&ap.left);
                    let fp = oxc::FormalParameter::new(
                        SPAN,
                        ArenaVec::new_in(self), // decorators
                        left,
                        type_annotation,
                        initializer,
                        optional,
                        None,  // accessibility
                        false, // readonly
                        false, /* override */
                        self,
                    );
                    items.push(fp);
                }
                _ => {
                    let pattern = self.convert_pattern_to_binding_pattern(param);
                    let type_annotation = self.pattern_type_annotation(param);
                    let optional = self.pattern_optional(param);
                    let fp = oxc::FormalParameter::new(
                        SPAN,
                        ArenaVec::new_in(self), // decorators
                        pattern,
                        type_annotation,
                        None::<ArenaBox<'a, oxc::Expression<'a>>>,
                        optional,
                        None,  // accessibility
                        false, // readonly
                        false, /* override */
                        self,
                    );
                    items.push(fp);
                }
            }
        }

        let items_vec = ArenaVec::from_iter_in(items, self);
        oxc::FormalParameters::new(
            SPAN,
            oxc::FormalParameterKind::FormalParameter,
            items_vec,
            rest,
            self,
        )
    }

    fn pattern_optional(&self, pattern: &PatternLike) -> bool {
        match pattern {
            PatternLike::Identifier(id) => id.optional.unwrap_or(false),
            PatternLike::AssignmentPattern(assign) => self.pattern_optional(&assign.left),
            PatternLike::RestElement(rest) => self.pattern_optional(&rest.argument),
            PatternLike::ObjectPattern(_)
            | PatternLike::ArrayPattern(_)
            | PatternLike::MemberExpression(_)
            | PatternLike::TSAsExpression(_)
            | PatternLike::TSSatisfiesExpression(_)
            | PatternLike::TSNonNullExpression(_)
            | PatternLike::TSTypeAssertion(_)
            | PatternLike::TypeCastExpression(_) => false,
        }
    }

    fn pattern_type_annotation(
        &self,
        pattern: &PatternLike,
    ) -> Option<ArenaBox<'a, oxc::TSTypeAnnotation<'a>>> {
        let value = match pattern {
            PatternLike::Identifier(id) => id.type_annotation.as_ref(),
            PatternLike::ObjectPattern(obj) => obj.type_annotation.as_ref(),
            PatternLike::ArrayPattern(arr) => arr.type_annotation.as_ref(),
            PatternLike::AssignmentPattern(assign) => assign.type_annotation.as_ref(),
            PatternLike::RestElement(rest) => rest.type_annotation.as_ref(),
            PatternLike::MemberExpression(_)
            | PatternLike::TSAsExpression(_)
            | PatternLike::TSSatisfiesExpression(_)
            | PatternLike::TSNonNullExpression(_)
            | PatternLike::TSTypeAssertion(_)
            | PatternLike::TypeCastExpression(_) => None,
        }?;
        self.convert_ts_type_annotation_from_json(&value.parse_value())
    }

    // ===== Patterns → BindingPattern =====

    fn convert_pattern_to_binding_pattern(&self, pattern: &PatternLike) -> oxc::BindingPattern<'a> {
        match pattern {
            PatternLike::Identifier(id) => {
                oxc::BindingPattern::new_binding_identifier(SPAN, self.str(&id.name), self)
            }
            PatternLike::ObjectPattern(obj) => {
                let mut properties: Vec<oxc::BindingProperty<'a>> = Vec::new();
                let mut rest: Option<oxc::BindingRestElement<'a>> = None;

                for prop in &obj.properties {
                    match prop {
                        ObjectPatternProperty::ObjectProperty(p) => {
                            let key = self.convert_expression_to_property_key(&p.key, p.computed);
                            let value = self.convert_pattern_to_binding_pattern(&p.value);
                            let bp = oxc::BindingProperty::new(
                                SPAN,
                                key,
                                value,
                                p.shorthand,
                                p.computed,
                                self,
                            );
                            properties.push(bp);
                        }
                        ObjectPatternProperty::RestElement(r) => {
                            let arg = self.convert_pattern_to_binding_pattern(&r.argument);
                            rest = Some(oxc::BindingRestElement::new(SPAN, arg, self));
                        }
                    }
                }

                let props_vec = ArenaVec::from_iter_in(properties, self);
                oxc::BindingPattern::new_object_pattern(SPAN, props_vec, rest, self)
            }
            PatternLike::ArrayPattern(arr) => {
                let mut elements: Vec<Option<oxc::BindingPattern<'a>>> = Vec::new();
                let mut rest: Option<oxc::BindingRestElement<'a>> = None;

                for elem in &arr.elements {
                    match elem {
                        None => elements.push(None),
                        Some(PatternLike::RestElement(r)) => {
                            let arg = self.convert_pattern_to_binding_pattern(&r.argument);
                            rest = Some(oxc::BindingRestElement::new(SPAN, arg, self));
                        }
                        Some(p) => {
                            elements.push(Some(self.convert_pattern_to_binding_pattern(p)));
                        }
                    }
                }

                let elems_vec = ArenaVec::from_iter_in(elements, self);
                oxc::BindingPattern::new_array_pattern(SPAN, elems_vec, rest, self)
            }
            PatternLike::AssignmentPattern(ap) => {
                let left = self.convert_pattern_to_binding_pattern(&ap.left);
                let right = self.convert_expression(&ap.right);
                oxc::BindingPattern::new_assignment_pattern(SPAN, left, right, self)
            }
            PatternLike::RestElement(r) => self.convert_pattern_to_binding_pattern(&r.argument),
            PatternLike::MemberExpression(_)
            | PatternLike::TSAsExpression(_)
            | PatternLike::TSSatisfiesExpression(_)
            | PatternLike::TSNonNullExpression(_)
            | PatternLike::TSTypeAssertion(_)
            | PatternLike::TypeCastExpression(_) => oxc::BindingPattern::new_binding_identifier(
                SPAN,
                self.str("__member_pattern__"),
                self,
            ),
        }
    }

    // ===== Patterns → AssignmentTarget =====

    fn convert_pattern_to_assignment_target(
        &self,
        pattern: &PatternLike,
    ) -> oxc::AssignmentTarget<'a> {
        match pattern {
            PatternLike::Identifier(id) => oxc::AssignmentTarget::new_assignment_target_identifier(
                SPAN,
                self.str(&id.name),
                self,
            ),
            PatternLike::MemberExpression(m) => {
                let object = self.convert_expression(&m.object);
                if m.computed {
                    let property = self.convert_expression(&m.property);
                    let mem =
                        oxc::ComputedMemberExpression::new(SPAN, object, property, false, self);
                    oxc::AssignmentTarget::ComputedMemberExpression(ArenaBox::new_in(mem, self))
                } else {
                    let prop_name = self.expression_to_identifier_name(&m.property);
                    let mem =
                        oxc::StaticMemberExpression::new(SPAN, object, prop_name, false, self);
                    oxc::AssignmentTarget::StaticMemberExpression(ArenaBox::new_in(mem, self))
                }
            }
            PatternLike::ObjectPattern(obj) => {
                let mut properties: Vec<oxc::AssignmentTargetProperty<'a>> = Vec::new();
                let mut rest: Option<oxc::AssignmentTargetRest<'a>> = None;

                for prop in &obj.properties {
                    match prop {
                        ObjectPatternProperty::ObjectProperty(p) => {
                            if p.shorthand {
                                // Shorthand: { x } means { x: x }
                                // Use AssignmentTargetPropertyIdentifier
                                if let Expression::Identifier(id) = &*p.key {
                                    let binding = oxc::IdentifierReference::new(
                                        SPAN,
                                        self.str(&id.name),
                                        self,
                                    );
                                    let init = match &*p.value {
                                        PatternLike::AssignmentPattern(ap) => {
                                            Some(self.convert_expression(&ap.right))
                                        }
                                        _ => None,
                                    };
                                    let atp = oxc::AssignmentTargetProperty::new_assignment_target_property_identifier(SPAN, binding, init, self);
                                    properties.push(atp);
                                } else {
                                    // Fallback to non-shorthand
                                    let key =
                                        self.convert_expression_to_property_key(&p.key, p.computed);
                                    let binding = self
                                        .convert_pattern_to_assignment_target_maybe_default(
                                            &p.value,
                                        );
                                    let atp = oxc::AssignmentTargetProperty::new_assignment_target_property_property(SPAN, key, binding, p.computed, self);
                                    properties.push(atp);
                                }
                            } else {
                                let key =
                                    self.convert_expression_to_property_key(&p.key, p.computed);
                                let binding = self
                                    .convert_pattern_to_assignment_target_maybe_default(&p.value);
                                let atp = oxc::AssignmentTargetProperty::new_assignment_target_property_property(SPAN, key, binding, p.computed, self);
                                properties.push(atp);
                            }
                        }
                        ObjectPatternProperty::RestElement(r) => {
                            let target = self.convert_pattern_to_assignment_target(&r.argument);
                            rest = Some(oxc::AssignmentTargetRest::new(SPAN, target, self));
                        }
                    }
                }

                let props_vec = ArenaVec::from_iter_in(properties, self);
                oxc::AssignmentTarget::new_object_assignment_target(SPAN, props_vec, rest, self)
            }
            PatternLike::ArrayPattern(arr) => {
                let mut elements: Vec<Option<oxc::AssignmentTargetMaybeDefault<'a>>> = Vec::new();
                let mut rest: Option<oxc::AssignmentTargetRest<'a>> = None;

                for elem in &arr.elements {
                    match elem {
                        None => elements.push(None),
                        Some(PatternLike::RestElement(r)) => {
                            let target = self.convert_pattern_to_assignment_target(&r.argument);
                            rest = Some(oxc::AssignmentTargetRest::new(SPAN, target, self));
                        }
                        Some(p) => {
                            elements.push(Some(
                                self.convert_pattern_to_assignment_target_maybe_default(p),
                            ));
                        }
                    }
                }

                let elems_vec = ArenaVec::from_iter_in(elements, self);
                oxc::AssignmentTarget::new_array_assignment_target(SPAN, elems_vec, rest, self)
            }
            PatternLike::AssignmentPattern(ap) => {
                // For assignment LHS, use the left side
                self.convert_pattern_to_assignment_target(&ap.left)
            }
            PatternLike::RestElement(r) => self.convert_pattern_to_assignment_target(&r.argument),
            PatternLike::TSAsExpression(e) => {
                self.convert_ts_as_expression_to_simple_assignment_target(e).into()
            }
            PatternLike::TSSatisfiesExpression(e) => {
                self.convert_ts_satisfies_expression_to_simple_assignment_target(e).into()
            }
            PatternLike::TSNonNullExpression(e) => {
                self.convert_ts_non_null_expression_to_simple_assignment_target(e).into()
            }
            PatternLike::TSTypeAssertion(e) => {
                self.convert_ts_type_assertion_to_simple_assignment_target(e).into()
            }
            PatternLike::TypeCastExpression(_) => {
                oxc::AssignmentTarget::new_assignment_target_identifier(
                    SPAN,
                    self.str("__unknown__"),
                    self,
                )
            }
        }
    }

    fn convert_pattern_to_assignment_target_maybe_default(
        &self,
        pattern: &PatternLike,
    ) -> oxc::AssignmentTargetMaybeDefault<'a> {
        match pattern {
            PatternLike::AssignmentPattern(ap) => {
                let binding = self.convert_pattern_to_assignment_target(&ap.left);
                let init = self.convert_expression(&ap.right);
                oxc::AssignmentTargetMaybeDefault::new_assignment_target_with_default(
                    SPAN, binding, init, self,
                )
            }
            _ => {
                let target = self.convert_pattern_to_assignment_target(pattern);
                oxc::AssignmentTargetMaybeDefault::from(target)
            }
        }
    }

    fn convert_expression_to_simple_assignment_target(
        &self,
        expr: &Expression,
    ) -> oxc::SimpleAssignmentTarget<'a> {
        match expr {
            Expression::Identifier(id) => {
                oxc::SimpleAssignmentTarget::new_assignment_target_identifier(
                    SPAN,
                    self.str(&id.name),
                    self,
                )
            }
            Expression::MemberExpression(m) => {
                let object = self.convert_expression(&m.object);
                if m.computed {
                    let property = self.convert_expression(&m.property);
                    let mem =
                        oxc::ComputedMemberExpression::new(SPAN, object, property, false, self);
                    oxc::SimpleAssignmentTarget::ComputedMemberExpression(ArenaBox::new_in(
                        mem, self,
                    ))
                } else {
                    let prop_name = self.expression_to_identifier_name(&m.property);
                    let mem =
                        oxc::StaticMemberExpression::new(SPAN, object, prop_name, false, self);
                    oxc::SimpleAssignmentTarget::StaticMemberExpression(ArenaBox::new_in(mem, self))
                }
            }
            Expression::TSAsExpression(e) => {
                self.convert_ts_as_expression_to_simple_assignment_target(e)
            }
            Expression::TSSatisfiesExpression(e) => {
                self.convert_ts_satisfies_expression_to_simple_assignment_target(e)
            }
            Expression::TSNonNullExpression(e) => {
                self.convert_ts_non_null_expression_to_simple_assignment_target(e)
            }
            Expression::TSTypeAssertion(e) => {
                self.convert_ts_type_assertion_to_simple_assignment_target(e)
            }
            _ => oxc::SimpleAssignmentTarget::new_assignment_target_identifier(
                SPAN,
                self.str("__unknown__"),
                self,
            ),
        }
    }

    fn convert_ts_as_expression_to_simple_assignment_target(
        &self,
        expr: &TSAsExpression,
    ) -> oxc::SimpleAssignmentTarget<'a> {
        let expression = self.convert_expression(&expr.expression);
        let parsed_type = expr.type_annotation.parse_value();
        if let Some(type_annotation) = self.convert_ts_type_from_json(&parsed_type).or_else(|| {
            if Self::ts_type_json_contains_type_query(&parsed_type) {
                None
            } else {
                self.extract_source_ts_as_type(&expr.base)
            }
        }) {
            oxc::SimpleAssignmentTarget::new_ts_as_expression(
                SPAN,
                expression,
                type_annotation,
                self,
            )
        } else {
            self.convert_expression_to_simple_assignment_target(&expr.expression)
        }
    }

    fn convert_ts_satisfies_expression_to_simple_assignment_target(
        &self,
        expr: &TSSatisfiesExpression,
    ) -> oxc::SimpleAssignmentTarget<'a> {
        let expression = self.convert_expression(&expr.expression);
        let parsed_type = expr.type_annotation.parse_value();
        if let Some(type_annotation) = self.convert_ts_type_from_json(&parsed_type).or_else(|| {
            if Self::ts_type_json_contains_type_query(&parsed_type) {
                None
            } else {
                self.extract_source_ts_satisfies_type(&expr.base)
            }
        }) {
            oxc::SimpleAssignmentTarget::new_ts_satisfies_expression(
                SPAN,
                expression,
                type_annotation,
                self,
            )
        } else {
            self.convert_expression_to_simple_assignment_target(&expr.expression)
        }
    }

    fn convert_ts_non_null_expression_to_simple_assignment_target(
        &self,
        expr: &TSNonNullExpression,
    ) -> oxc::SimpleAssignmentTarget<'a> {
        let expression = self.convert_expression(&expr.expression);
        oxc::SimpleAssignmentTarget::new_ts_non_null_expression(SPAN, expression, self)
    }

    fn convert_ts_type_assertion_to_simple_assignment_target(
        &self,
        expr: &TSTypeAssertion,
    ) -> oxc::SimpleAssignmentTarget<'a> {
        let expression = self.convert_expression(&expr.expression);
        let parsed_type = expr.type_annotation.parse_value();
        if let Some(type_annotation) = self.convert_ts_type_from_json(&parsed_type).or_else(|| {
            if Self::ts_type_json_contains_type_query(&parsed_type) {
                None
            } else {
                self.extract_source_ts_type_assertion_type(&expr.base)
            }
        }) {
            oxc::SimpleAssignmentTarget::new_ts_type_assertion(
                SPAN,
                type_annotation,
                expression,
                self,
            )
        } else {
            self.convert_expression_to_simple_assignment_target(&expr.expression)
        }
    }

    // ===== JSX =====

    fn convert_jsx_element(&self, el: &JSXElement) -> oxc::JSXElement<'a> {
        let opening = self.convert_jsx_opening_element(&el.opening_element, Some(&el.base));
        let children =
            ArenaVec::from_iter_in(el.children.iter().map(|c| self.convert_jsx_child(c)), self);
        let closing = el.closing_element.as_ref().map(|c| self.convert_jsx_closing_element(c));
        oxc::JSXElement::new(SPAN, opening, children, closing, self)
    }

    fn convert_jsx_opening_element(
        &self,
        el: &JSXOpeningElement,
        element_base: Option<&BaseNode>,
    ) -> oxc::JSXOpeningElement<'a> {
        let name = self.convert_jsx_element_name(&el.name);
        let attrs = ArenaVec::from_iter_in(
            el.attributes.iter().map(|a| self.convert_jsx_attribute_item(a)),
            self,
        );
        let type_arguments = self
            .extract_source_jsx_type_arguments(&el.base)
            .or_else(|| element_base.and_then(|base| self.extract_source_jsx_type_arguments(base)));
        oxc::JSXOpeningElement::new(SPAN, name, type_arguments, attrs, self)
    }

    fn convert_jsx_closing_element(&self, el: &JSXClosingElement) -> oxc::JSXClosingElement<'a> {
        let name = self.convert_jsx_element_name(&el.name);
        oxc::JSXClosingElement::new(SPAN, name, self)
    }

    fn convert_jsx_element_name(&self, name: &JSXElementName) -> oxc::JSXElementName<'a> {
        match name {
            JSXElementName::JSXIdentifier(id) => {
                let first_char = id.name.chars().next().unwrap_or('a');
                if first_char.is_uppercase() || id.name.contains('.') {
                    oxc::JSXElementName::new_identifier_reference(SPAN, self.str(&id.name), self)
                } else {
                    oxc::JSXElementName::new_identifier(SPAN, self.str(&id.name), self)
                }
            }
            JSXElementName::JSXMemberExpression(m) => {
                let member = self.convert_jsx_member_expression(m);
                oxc::JSXElementName::new_member_expression(
                    SPAN,
                    member.object,
                    member.property,
                    self,
                )
            }
            JSXElementName::JSXNamespacedName(ns) => {
                let namespace = oxc::JSXIdentifier::new(SPAN, self.str(&ns.namespace.name), self);
                let name = oxc::JSXIdentifier::new(SPAN, self.str(&ns.name.name), self);
                oxc::JSXElementName::new_namespaced_name(SPAN, namespace, name, self)
            }
        }
    }

    fn convert_jsx_member_expression(
        &self,
        m: &JSXMemberExpression,
    ) -> oxc::JSXMemberExpression<'a> {
        let object = self.convert_jsx_member_expression_object(&m.object);
        let property = oxc::JSXIdentifier::new(SPAN, self.str(&m.property.name), self);
        oxc::JSXMemberExpression::new(SPAN, object, property, self)
    }

    fn convert_jsx_member_expression_object(
        &self,
        obj: &JSXMemberExprObject,
    ) -> oxc::JSXMemberExpressionObject<'a> {
        match obj {
            JSXMemberExprObject::JSXIdentifier(id) => {
                oxc::JSXMemberExpressionObject::new_identifier_reference(
                    SPAN,
                    self.str(&id.name),
                    self,
                )
            }
            JSXMemberExprObject::JSXMemberExpression(m) => {
                let member = self.convert_jsx_member_expression(m);
                oxc::JSXMemberExpressionObject::new_member_expression(
                    SPAN,
                    member.object,
                    member.property,
                    self,
                )
            }
        }
    }

    fn convert_jsx_attribute_item(&self, item: &JSXAttributeItem) -> oxc::JSXAttributeItem<'a> {
        match item {
            JSXAttributeItem::JSXAttribute(attr) => {
                let name = self.convert_jsx_attribute_name(&attr.name);
                let value = attr.value.as_ref().map(|v| self.convert_jsx_attribute_value(v));
                oxc::JSXAttributeItem::new_attribute(SPAN, name, value, self)
            }
            JSXAttributeItem::JSXSpreadAttribute(s) => oxc::JSXAttributeItem::new_spread_attribute(
                SPAN,
                self.convert_expression(&s.argument),
                self,
            ),
        }
    }

    fn convert_jsx_attribute_name(&self, name: &JSXAttributeName) -> oxc::JSXAttributeName<'a> {
        match name {
            JSXAttributeName::JSXIdentifier(id) => {
                oxc::JSXAttributeName::new_identifier(SPAN, self.str(&id.name), self)
            }
            JSXAttributeName::JSXNamespacedName(ns) => {
                let namespace = oxc::JSXIdentifier::new(SPAN, self.str(&ns.namespace.name), self);
                let name = oxc::JSXIdentifier::new(SPAN, self.str(&ns.name.name), self);
                oxc::JSXAttributeName::new_namespaced_name(SPAN, namespace, name, self)
            }
        }
    }

    fn convert_jsx_attribute_value(&self, value: &JSXAttributeValue) -> oxc::JSXAttributeValue<'a> {
        match value {
            JSXAttributeValue::StringLiteral(s) => oxc::JSXAttributeValue::new_string_literal(
                SPAN,
                self.str(&s.value.to_string_lossy()),
                None,
                self,
            ),
            JSXAttributeValue::JSXExpressionContainer(ec) => {
                let expr = self.convert_jsx_expression_container_expr(&ec.expression);
                oxc::JSXAttributeValue::new_expression_container(SPAN, expr, self)
            }
            JSXAttributeValue::JSXElement(el) => {
                let element = self.convert_jsx_element(el);
                let opening = element.opening_element;
                let closing = element.closing_element;
                oxc::JSXAttributeValue::new_element(SPAN, opening, element.children, closing, self)
            }
            JSXAttributeValue::JSXFragment(frag) => {
                let fragment = self.convert_jsx_fragment(frag);
                oxc::JSXAttributeValue::new_fragment(
                    SPAN,
                    fragment.opening_fragment,
                    fragment.children,
                    fragment.closing_fragment,
                    self,
                )
            }
        }
    }

    fn convert_jsx_expression_container_expr(
        &self,
        expr: &JSXExpressionContainerExpr,
    ) -> oxc::JSXExpression<'a> {
        match expr {
            JSXExpressionContainerExpr::JSXEmptyExpression(_) => {
                oxc::JSXExpression::new_empty_expression(SPAN, self)
            }
            JSXExpressionContainerExpr::Expression(e) => {
                oxc::JSXExpression::from(self.convert_expression(e))
            }
        }
    }

    fn convert_jsx_child(&self, child: &JSXChild) -> oxc::JSXChild<'a> {
        match child {
            JSXChild::JSXText(t) => {
                let value = encode_jsx_text(&t.value);
                oxc::JSXChild::new_text(SPAN, self.str(&value), None, self)
            }
            JSXChild::JSXElement(el) => {
                let element = self.convert_jsx_element(el);
                let opening = element.opening_element;
                let closing = element.closing_element;
                oxc::JSXChild::new_element(SPAN, opening, element.children, closing, self)
            }
            JSXChild::JSXFragment(frag) => {
                let fragment = self.convert_jsx_fragment(frag);
                oxc::JSXChild::new_fragment(
                    SPAN,
                    fragment.opening_fragment,
                    fragment.children,
                    fragment.closing_fragment,
                    self,
                )
            }
            JSXChild::JSXExpressionContainer(ec) => {
                let expr = self.convert_jsx_expression_container_expr(&ec.expression);
                oxc::JSXChild::new_expression_container(SPAN, expr, self)
            }
            JSXChild::JSXSpreadChild(s) => {
                oxc::JSXChild::new_spread(SPAN, self.convert_expression(&s.expression), self)
            }
        }
    }

    fn convert_jsx_fragment(&self, frag: &JSXFragment) -> oxc::JSXFragment<'a> {
        let opening = oxc::JSXOpeningFragment::new(SPAN, self);
        let closing = oxc::JSXClosingFragment::new(SPAN, self);
        let children =
            ArenaVec::from_iter_in(frag.children.iter().map(|c| self.convert_jsx_child(c)), self);
        oxc::JSXFragment::new(SPAN, opening, children, closing, self)
    }

    // ===== Import/Export =====

    fn convert_import_declaration(&self, decl: &ImportDeclaration) -> oxc::ImportDeclaration<'a> {
        let specifiers = ArenaVec::from_iter_in(
            decl.specifiers.iter().map(|s| self.convert_import_specifier(s)),
            self,
        );
        let specifiers =
            if specifiers.is_empty() && !self.import_declaration_has_empty_named_specifiers(decl) {
                None
            } else {
                Some(specifiers)
            };
        let source = oxc::StringLiteral::new(
            SPAN,
            self.str(&decl.source.value.to_string_lossy()),
            None,
            self,
        );
        let import_kind = match decl.import_kind.as_ref() {
            Some(ImportKind::Type) => oxc::ImportOrExportKind::Type,
            _ => oxc::ImportOrExportKind::Value,
        };
        let with_clause =
            self.convert_with_clause(decl.attributes.as_deref().or(decl.assertions.as_deref()));
        oxc::ImportDeclaration::new(
            SPAN,
            specifiers,
            source,
            None, // phase
            with_clause,
            import_kind,
            self,
        )
    }

    fn convert_with_clause(
        &self,
        attributes: Option<&[ImportAttribute]>,
    ) -> Option<ArenaBox<'a, oxc::WithClause<'a>>> {
        attributes.map(|attributes| {
            let with_entries = ArenaVec::from_iter_in(
                attributes.iter().map(|attr| self.convert_import_attribute(attr)),
                self,
            );
            oxc::WithClause::boxed(SPAN, oxc::WithClauseKeyword::With, with_entries, self)
        })
    }

    fn convert_import_attribute(&self, attr: &ImportAttribute) -> oxc::ImportAttribute<'a> {
        let key_was_quoted = self
            .source_text_for_base(&attr.key.base)
            .is_some_and(|text| matches!(text.trim_start().as_bytes().first(), Some(b'"' | b'\'')));
        let key = if key_was_quoted || !is_identifier_name(&attr.key.name) {
            oxc::ImportAttributeKey::new_string_literal(SPAN, self.str(&attr.key.name), None, self)
        } else {
            oxc::ImportAttributeKey::new_identifier(SPAN, self.str(&attr.key.name), self)
        };
        let value = oxc::StringLiteral::new(
            SPAN,
            self.str(&attr.value.value.to_string_lossy()),
            None,
            self,
        );
        oxc::ImportAttribute::new(SPAN, key, value, self)
    }

    fn convert_import_specifier(
        &self,
        spec: &ImportSpecifier,
    ) -> oxc::ImportDeclarationSpecifier<'a> {
        match spec {
            ImportSpecifier::ImportSpecifier(s) => {
                let local = oxc::BindingIdentifier::new(SPAN, self.str(&s.local.name), self);
                let imported = self.convert_module_export_name(&s.imported);
                let import_kind = match s.import_kind.as_ref() {
                    Some(ImportKind::Type) => oxc::ImportOrExportKind::Type,
                    _ => oxc::ImportOrExportKind::Value,
                };
                let is = oxc::ImportSpecifier::new(SPAN, imported, local, import_kind, self);
                oxc::ImportDeclarationSpecifier::ImportSpecifier(ArenaBox::new_in(is, self))
            }
            ImportSpecifier::ImportDefaultSpecifier(s) => {
                let local = oxc::BindingIdentifier::new(SPAN, self.str(&s.local.name), self);
                let ids = oxc::ImportDefaultSpecifier::new(SPAN, local, self);
                oxc::ImportDeclarationSpecifier::ImportDefaultSpecifier(ArenaBox::new_in(ids, self))
            }
            ImportSpecifier::ImportNamespaceSpecifier(s) => {
                let local = oxc::BindingIdentifier::new(SPAN, self.str(&s.local.name), self);
                let ins = oxc::ImportNamespaceSpecifier::new(SPAN, local, self);
                oxc::ImportDeclarationSpecifier::ImportNamespaceSpecifier(ArenaBox::new_in(
                    ins, self,
                ))
            }
        }
    }

    fn convert_module_export_name(&self, name: &ModuleExportName) -> oxc::ModuleExportName<'a> {
        match name {
            ModuleExportName::Identifier(id) => {
                oxc::ModuleExportName::new_identifier_name(SPAN, self.str(&id.name), self)
            }
            ModuleExportName::StringLiteral(s) => oxc::ModuleExportName::new_string_literal(
                SPAN,
                self.str(&s.value.to_string_lossy()),
                None,
                self,
            ),
        }
    }

    /// Like [`Self::convert_module_export_name`], but builds an identifier `local`
    /// of a local export specifier as an `IdentifierReference` (not a bare
    /// `IdentifierName`) so semantic analysis links it to the exported binding. A
    /// string-literal local is only valid with a `source`, so it falls back to the
    /// plain name.
    fn convert_module_export_name_local_ref(
        &self,
        name: &ModuleExportName,
    ) -> oxc::ModuleExportName<'a> {
        match name {
            ModuleExportName::Identifier(id) => {
                oxc::ModuleExportName::new_identifier_reference(SPAN, self.str(&id.name), self)
            }
            ModuleExportName::StringLiteral(_) => self.convert_module_export_name(name),
        }
    }

    fn convert_export_named_declaration(
        &self,
        decl: &ExportNamedDeclaration,
    ) -> oxc::ExportNamedDeclaration<'a> {
        let declaration = decl.declaration.as_ref().map(|d| self.convert_declaration(d));
        // For a local export (`export { x }`, no `source`) the specifier `local`
        // refers to a binding in this module, so it must be an `IdentifierReference`
        // for semantic analysis to link it (and thus keep its import alive through
        // TypeScript import elision). Re-exports (`export { x } from`) keep an
        // `IdentifierName`, since `local` names an export of the other module. This
        // mirrors the parser (`parse_export_named_specifiers`).
        let local_is_reference = decl.source.is_none();
        let specifiers = ArenaVec::from_iter_in(
            decl.specifiers.iter().map(|s| self.convert_export_specifier(s, local_is_reference)),
            self,
        );
        let source = decl.source.as_ref().map(|s| {
            oxc::StringLiteral::new(SPAN, self.str(&s.value.to_string_lossy()), None, self)
        });
        let export_kind = match decl.export_kind.as_ref() {
            Some(ExportKind::Type) => oxc::ImportOrExportKind::Type,
            _ => oxc::ImportOrExportKind::Value,
        };
        let with_clause =
            self.convert_with_clause(decl.attributes.as_deref().or(decl.assertions.as_deref()));
        oxc::ExportNamedDeclaration::new(
            SPAN,
            declaration,
            specifiers,
            source,
            export_kind,
            with_clause,
            self,
        )
    }

    fn convert_declaration(&self, decl: &Declaration) -> oxc::Declaration<'a> {
        match decl {
            Declaration::FunctionDeclaration(f) => {
                let func = self.convert_function_decl(f, oxc::FunctionType::FunctionDeclaration);
                oxc::Declaration::FunctionDeclaration(ArenaBox::new_in(func, self))
            }
            Declaration::VariableDeclaration(v) => {
                let d = self.convert_variable_declaration(v);
                oxc::Declaration::VariableDeclaration(ArenaBox::new_in(d, self))
            }
            Declaration::ClassDeclaration(c) => {
                let class = self.convert_class_declaration(c);
                oxc::Declaration::ClassDeclaration(ArenaBox::new_in(class, self))
            }
            _ => {
                let d = oxc::VariableDeclaration::new(
                    SPAN,
                    oxc::VariableDeclarationKind::Const,
                    ArenaVec::new_in(self),
                    true,
                    self,
                );
                oxc::Declaration::VariableDeclaration(ArenaBox::new_in(d, self))
            }
        }
    }

    fn convert_export_specifier(
        &self,
        spec: &ExportSpecifier,
        local_is_reference: bool,
    ) -> oxc::ExportSpecifier<'a> {
        match spec {
            ExportSpecifier::ExportSpecifier(s) => {
                let local = if local_is_reference {
                    self.convert_module_export_name_local_ref(&s.local)
                } else {
                    self.convert_module_export_name(&s.local)
                };
                let exported = self.convert_module_export_name(&s.exported);
                let export_kind = match s.export_kind.as_ref() {
                    Some(ExportKind::Type) => oxc::ImportOrExportKind::Type,
                    _ => oxc::ImportOrExportKind::Value,
                };
                oxc::ExportSpecifier::new(SPAN, local, exported, export_kind, self)
            }
            ExportSpecifier::ExportDefaultSpecifier(s) => {
                let name = oxc::ModuleExportName::new_identifier_name(
                    SPAN,
                    self.str(&s.exported.name),
                    self,
                );
                let default_name =
                    oxc::ModuleExportName::new_identifier_name(SPAN, self.str("default"), self);
                oxc::ExportSpecifier::new(
                    SPAN,
                    name,
                    default_name,
                    oxc::ImportOrExportKind::Value,
                    self,
                )
            }
            ExportSpecifier::ExportNamespaceSpecifier(s) => {
                let exported = self.convert_module_export_name(&s.exported);
                let star = oxc::ModuleExportName::new_identifier_name(SPAN, self.str("*"), self);
                oxc::ExportSpecifier::new(
                    SPAN,
                    star,
                    exported,
                    oxc::ImportOrExportKind::Value,
                    self,
                )
            }
        }
    }

    fn convert_export_default_declaration(
        &self,
        decl: &ExportDefaultDeclaration,
    ) -> oxc::ExportDefaultDeclaration<'a> {
        let declaration = self.convert_export_default_decl(&decl.declaration);
        oxc::ExportDefaultDeclaration::new(SPAN, declaration, self)
    }

    fn convert_export_default_decl(
        &self,
        decl: &ExportDefaultDecl,
    ) -> oxc::ExportDefaultDeclarationKind<'a> {
        match decl {
            ExportDefaultDecl::FunctionDeclaration(f) => {
                let func = self.convert_function_decl(f, oxc::FunctionType::FunctionDeclaration);
                oxc::ExportDefaultDeclarationKind::FunctionDeclaration(ArenaBox::new_in(func, self))
            }
            ExportDefaultDecl::ClassDeclaration(c) => {
                let class = self.convert_class_declaration(c);
                oxc::ExportDefaultDeclarationKind::ClassDeclaration(ArenaBox::new_in(class, self))
            }
            ExportDefaultDecl::EnumDeclaration(_) => {
                // Flow enum declarations cannot be represented in OXC AST;
                // emit a null placeholder to preserve the export shape.
                oxc::ExportDefaultDeclarationKind::new_null_literal(oxc::Span::default(), self)
            }
            ExportDefaultDecl::Expression(e) => {
                oxc::ExportDefaultDeclarationKind::from(self.convert_expression(e))
            }
        }
    }

    fn convert_export_all_declaration(
        &self,
        decl: &ExportAllDeclaration,
    ) -> oxc::ExportAllDeclaration<'a> {
        let source = oxc::StringLiteral::new(
            SPAN,
            self.str(&decl.source.value.to_string_lossy()),
            None,
            self,
        );
        let export_kind = match decl.export_kind.as_ref() {
            Some(ExportKind::Type) => oxc::ImportOrExportKind::Type,
            _ => oxc::ImportOrExportKind::Value,
        };
        let with_clause =
            self.convert_with_clause(decl.attributes.as_deref().or(decl.assertions.as_deref()));
        oxc::ExportAllDeclaration::new(
            SPAN,
            None, // exported
            source,
            with_clause,
            export_kind,
            self,
        )
    }

    // ===== Operators =====

    fn convert_binary_operator(&self, op: &BinaryOperator) -> OxcBinOp {
        match op {
            BinaryOperator::Add => OxcBinOp::Addition,
            BinaryOperator::Sub => OxcBinOp::Subtraction,
            BinaryOperator::Mul => OxcBinOp::Multiplication,
            BinaryOperator::Div => OxcBinOp::Division,
            BinaryOperator::Rem => OxcBinOp::Remainder,
            BinaryOperator::Exp => OxcBinOp::Exponential,
            BinaryOperator::Eq => OxcBinOp::Equality,
            BinaryOperator::StrictEq => OxcBinOp::StrictEquality,
            BinaryOperator::Neq => OxcBinOp::Inequality,
            BinaryOperator::StrictNeq => OxcBinOp::StrictInequality,
            BinaryOperator::Lt => OxcBinOp::LessThan,
            BinaryOperator::Lte => OxcBinOp::LessEqualThan,
            BinaryOperator::Gt => OxcBinOp::GreaterThan,
            BinaryOperator::Gte => OxcBinOp::GreaterEqualThan,
            BinaryOperator::Shl => OxcBinOp::ShiftLeft,
            BinaryOperator::Shr => OxcBinOp::ShiftRight,
            BinaryOperator::UShr => OxcBinOp::ShiftRightZeroFill,
            BinaryOperator::BitOr => OxcBinOp::BitwiseOR,
            BinaryOperator::BitXor => OxcBinOp::BitwiseXOR,
            BinaryOperator::BitAnd => OxcBinOp::BitwiseAnd,
            BinaryOperator::In => OxcBinOp::In,
            BinaryOperator::Instanceof => OxcBinOp::Instanceof,
            BinaryOperator::Pipeline => OxcBinOp::BitwiseOR, // no pipeline in OXC
        }
    }

    fn convert_logical_operator(&self, op: &LogicalOperator) -> OxcLogOp {
        match op {
            LogicalOperator::Or => OxcLogOp::Or,
            LogicalOperator::And => OxcLogOp::And,
            LogicalOperator::NullishCoalescing => OxcLogOp::Coalesce,
        }
    }

    fn convert_unary_operator(&self, op: &UnaryOperator) -> OxcUnOp {
        match op {
            UnaryOperator::Neg => OxcUnOp::UnaryNegation,
            UnaryOperator::Plus => OxcUnOp::UnaryPlus,
            UnaryOperator::Not => OxcUnOp::LogicalNot,
            UnaryOperator::BitNot => OxcUnOp::BitwiseNot,
            UnaryOperator::TypeOf => OxcUnOp::Typeof,
            UnaryOperator::Void => OxcUnOp::Void,
            UnaryOperator::Delete => OxcUnOp::Delete,
            UnaryOperator::Throw => OxcUnOp::Void, // no throw-as-unary in OXC
        }
    }

    fn convert_update_operator(&self, op: &UpdateOperator) -> OxcUpOp {
        match op {
            UpdateOperator::Increment => OxcUpOp::Increment,
            UpdateOperator::Decrement => OxcUpOp::Decrement,
        }
    }

    fn convert_assignment_operator(&self, op: &AssignmentOperator) -> OxcAssOp {
        match op {
            AssignmentOperator::Assign => OxcAssOp::Assign,
            AssignmentOperator::AddAssign => OxcAssOp::Addition,
            AssignmentOperator::SubAssign => OxcAssOp::Subtraction,
            AssignmentOperator::MulAssign => OxcAssOp::Multiplication,
            AssignmentOperator::DivAssign => OxcAssOp::Division,
            AssignmentOperator::RemAssign => OxcAssOp::Remainder,
            AssignmentOperator::ExpAssign => OxcAssOp::Exponential,
            AssignmentOperator::ShlAssign => OxcAssOp::ShiftLeft,
            AssignmentOperator::ShrAssign => OxcAssOp::ShiftRight,
            AssignmentOperator::UShrAssign => OxcAssOp::ShiftRightZeroFill,
            AssignmentOperator::BitOrAssign => OxcAssOp::BitwiseOR,
            AssignmentOperator::BitXorAssign => OxcAssOp::BitwiseXOR,
            AssignmentOperator::BitAndAssign => OxcAssOp::BitwiseAnd,
            AssignmentOperator::OrAssign => OxcAssOp::LogicalOr,
            AssignmentOperator::AndAssign => OxcAssOp::LogicalAnd,
            AssignmentOperator::NullishAssign => OxcAssOp::LogicalNullish,
        }
    }

    fn parse_regexp_flags(&self, flags_str: &str) -> oxc::RegExpFlags {
        let mut flags = oxc::RegExpFlags::empty();
        for ch in flags_str.chars() {
            match ch {
                'd' => flags |= oxc::RegExpFlags::D,
                'g' => flags |= oxc::RegExpFlags::G,
                'i' => flags |= oxc::RegExpFlags::I,
                'm' => flags |= oxc::RegExpFlags::M,
                's' => flags |= oxc::RegExpFlags::S,
                'u' => flags |= oxc::RegExpFlags::U,
                'v' => flags |= oxc::RegExpFlags::V,
                'y' => flags |= oxc::RegExpFlags::Y,
                _ => {}
            }
        }
        flags
    }
}

impl<'a> GetAllocator<'a> for ReverseCtx<'a> {
    fn allocator(&self) -> &'a Allocator {
        self.allocator
    }
}

impl<'a> GetAstBuilder<'a> for ReverseCtx<'a> {
    type Builder = AstBuilder<'a>;

    fn builder(&self) -> &AstBuilder<'a> {
        &self.builder
    }
}
