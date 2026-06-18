// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

//! Reverse AST converter: react_compiler_ast (Babel format) → OXC AST.
//!
//! This is the inverse of `convert_ast.rs`. It takes a `crate::react_compiler_ast::File`
//! (which represents the compiler's Babel-compatible output) and produces OXC AST
//! nodes allocated in an OXC arena, suitable for code generation via `oxc_codegen`.

use crate::react_compiler_ast::common::BaseNode;
use crate::react_compiler_ast::declarations::*;
use crate::react_compiler_ast::expressions::*;
use crate::react_compiler_ast::jsx::*;
use crate::react_compiler_ast::operators::*;
use crate::react_compiler_ast::patterns::*;
use crate::react_compiler_ast::statements::*;
use oxc_allocator::{Allocator, Box as ArenaBox};
use oxc_ast::ast as oxc;
use oxc_ast_visit::VisitMut;
use oxc_span::SPAN;
use oxc_span::Span;
use oxc_syntax::identifier::is_identifier_name;

fn set_statement_span(stmt: &mut oxc::Statement<'_>, span: Span) {
    use oxc_span::GetSpanMut;
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
pub fn convert_program_to_oxc<'a>(
    file: &crate::react_compiler_ast::File,
    allocator: &'a Allocator,
) -> oxc::Program<'a> {
    let ctx = ReverseCtx::new(allocator, None);
    ctx.convert_program(&file.program)
}

/// Convert with source text available for extracting TS declarations.
pub fn convert_program_to_oxc_with_source<'a>(
    file: &crate::react_compiler_ast::File,
    allocator: &'a Allocator,
    source_text: &str,
) -> oxc::Program<'a> {
    let ctx = ReverseCtx::new(allocator, Some(source_text.to_string()));
    ctx.convert_program(&file.program)
}

struct ReverseCtx<'a> {
    allocator: &'a Allocator,
    builder: oxc_ast::AstBuilder<'a>,
    source_text: Option<String>,
}

impl<'a> ReverseCtx<'a> {
    fn new(allocator: &'a Allocator, source_text: Option<String>) -> Self {
        Self { allocator, builder: oxc_ast::AstBuilder::new(allocator), source_text }
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
        let parsed =
            oxc_parser::Parser::new(self.allocator, text_ref, oxc_span::SourceType::tsx()).parse();
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
            oxc_parser::Parser::new(self.allocator, text_ref, oxc_span::SourceType::tsx())
                .parse_expression()
                .ok()?;
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
        oxc_allocator::StringBuilder::from_str_in(text, self.allocator).into_str()
    }

    fn extract_source_class_expression(
        &self,
        class: &crate::react_compiler_ast::expressions::ClassExpression,
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

    /// Re-emit a TS type from its original source span, applying any identifier
    /// renames recorded on the `RawNode`'s metadata (text substitution before
    /// re-parsing). Replaces JSON-tree reconstruction: the oxc frontend always has
    /// the source, so types round-trip by re-parsing the original text.
    fn convert_type_from_raw(
        &self,
        raw: &crate::react_compiler_ast::common::RawNode,
    ) -> Option<oxc::TSType<'a>> {
        let source = self.source_text.as_deref()?;
        let start = raw.type_start? as usize;
        let end = raw.type_end? as usize;
        if start >= source.len() || end > source.len() || start >= end {
            return None;
        }
        let slice = &source[start..end];
        // Apply renames (e.g. `typeof x` -> `typeof x_0`) as text edits, right to
        // left so earlier offsets stay valid, then re-parse the rendered type.
        let mut edits: Vec<(usize, usize, &str)> = raw
            .idents
            .iter()
            .filter_map(|id| {
                let renamed = id.renamed_to.as_deref()?;
                let rel = (id.start as usize).checked_sub(start)?;
                Some((rel, id.name.len(), renamed))
            })
            .collect();
        if edits.is_empty() {
            return self.parse_source_ts_type_text_at(slice, start);
        }
        edits.sort_by_key(|edit| std::cmp::Reverse(edit.0));
        let mut text = slice.to_string();
        for (rel, old_len, renamed) in edits {
            if rel + old_len <= text.len() {
                text.replace_range(rel..rel + old_len, renamed);
            }
        }
        self.parse_source_ts_type_text_at(&text, start)
    }

    fn convert_type_annotation_from_raw(
        &self,
        raw: &crate::react_compiler_ast::common::RawNode,
    ) -> Option<ArenaBox<'a, oxc::TSTypeAnnotation<'a>>> {
        let ty = self.convert_type_from_raw(raw)?;
        Some(self.builder.alloc_ts_type_annotation(SPAN, ty))
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
                Ok(self.builder.expression_ts_as(SPAN, expression, expr.type_annotation))
            }
            oxc::Expression::TSSatisfiesExpression(expr) => {
                let expr = expr.unbox();
                if Self::ts_type_contains_type_query(&expr.type_annotation) {
                    return Err(expression);
                }
                Ok(self.builder.expression_ts_satisfies(SPAN, expression, expr.type_annotation))
            }
            oxc::Expression::TSTypeAssertion(expr) => {
                let expr = expr.unbox();
                if Self::ts_type_contains_type_query(&expr.type_annotation) {
                    return Err(expression);
                }
                Ok(self.builder.expression_ts_type_assertion(
                    SPAN,
                    expr.type_annotation,
                    expression,
                ))
            }
            oxc::Expression::TSInstantiationExpression(expr) => {
                let expr = expr.unbox();
                if Self::ts_type_arguments_contain_type_query(&expr.type_arguments) {
                    return Err(expression);
                }
                Ok(self.builder.expression_ts_instantiation(SPAN, expression, expr.type_arguments))
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
                Ok(self.builder.expression_ts_as(SPAN, expression, expr.type_annotation))
            }
            oxc::Argument::TSSatisfiesExpression(expr) => {
                let expr = expr.unbox();
                if Self::ts_type_contains_type_query(&expr.type_annotation) {
                    return Err(expression);
                }
                Ok(self.builder.expression_ts_satisfies(SPAN, expression, expr.type_annotation))
            }
            oxc::Argument::TSTypeAssertion(expr) => {
                let expr = expr.unbox();
                if Self::ts_type_contains_type_query(&expr.type_annotation) {
                    return Err(expression);
                }
                Ok(self.builder.expression_ts_type_assertion(
                    SPAN,
                    expr.type_annotation,
                    expression,
                ))
            }
            oxc::Argument::TSInstantiationExpression(expr) => {
                let expr = expr.unbox();
                if Self::ts_type_arguments_contain_type_query(&expr.type_arguments) {
                    return Err(expression);
                }
                Ok(self.builder.expression_ts_instantiation(SPAN, expression, expr.type_arguments))
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

    /// Allocate a string in the arena and return a `&str` with lifetime 'a.
    ///
    /// The returned `&'a str` converts into both `Ident` and `Str` (identifier
    /// and string-literal name types), so it feeds every `AstBuilder` method.
    fn atom(&self, s: &str) -> &'a str {
        oxc_allocator::StringBuilder::from_str_in(s, self.allocator).into_str()
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

    fn convert_program(&self, program: &crate::react_compiler_ast::Program) -> oxc::Program<'a> {
        let source_type = match program.source_type {
            crate::react_compiler_ast::SourceType::Module => oxc_span::SourceType::mjs(),
            crate::react_compiler_ast::SourceType::Script => oxc_span::SourceType::cjs(),
        };

        // Use convert_statements_with_spans for the top-level body so that
        // original source positions are preserved. This allows comments from
        // the original source to be correctly attached to statements.
        let body = self.convert_statements_with_spans(&program.body);
        let directives = self.convert_directives(&program.directives);
        let hashbang = program.interpreter.as_ref().map(|interpreter| {
            self.builder
                .hashbang(self.span_from_base(&interpreter.base), self.atom(&interpreter.value))
        });
        let comments = self.builder.vec();

        self.builder.program(SPAN, source_type, "", comments, hashbang, directives, body)
    }

    // ===== Directives =====

    fn convert_directives(
        &self,
        directives: &[Directive],
    ) -> oxc_allocator::Vec<'a, oxc::Directive<'a>> {
        self.builder.vec_from_iter(directives.iter().map(|d| self.convert_directive(d)))
    }

    fn convert_directive(&self, d: &Directive) -> oxc::Directive<'a> {
        let expression = self.builder.string_literal(SPAN, self.atom(&d.value.value), None);
        self.builder.directive(SPAN, expression, self.atom(&d.value.value))
    }

    // ===== Statements =====

    /// Convert statements preserving span info from the Babel AST.
    /// This is used for top-level program body where span positions
    /// are needed for comment attachment.
    fn convert_statements_with_spans(
        &self,
        stmts: &[Statement],
    ) -> oxc_allocator::Vec<'a, oxc::Statement<'a>> {
        self.builder.vec_from_iter(stmts.iter().map(|s| {
            let span = self.get_statement_span(s);
            let mut oxc_stmt = self.convert_statement(s);
            if span != SPAN {
                set_statement_span(&mut oxc_stmt, span);
            }
            oxc_stmt
        }))
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
                self.builder.statement_block(SPAN, self.convert_statement_vec(&s.body))
            }
            Statement::ReturnStatement(s) => self
                .builder
                .statement_return(SPAN, s.argument.as_ref().map(|a| self.convert_expression(a))),
            Statement::ExpressionStatement(s) => {
                self.builder.statement_expression(SPAN, self.convert_expression(&s.expression))
            }
            Statement::IfStatement(s) => self.builder.statement_if(
                SPAN,
                self.convert_expression(&s.test),
                self.convert_statement(&s.consequent),
                s.alternate.as_ref().map(|a| self.convert_statement(a)),
            ),
            Statement::ForStatement(s) => {
                let init = s.init.as_ref().map(|i| self.convert_for_init(i));
                let test = s.test.as_ref().map(|t| self.convert_expression(t));
                let update = s.update.as_ref().map(|u| self.convert_expression(u));
                let body = self.convert_statement(&s.body);
                self.builder.statement_for(SPAN, init, test, update, body)
            }
            Statement::WhileStatement(s) => self.builder.statement_while(
                SPAN,
                self.convert_expression(&s.test),
                self.convert_statement(&s.body),
            ),
            Statement::DoWhileStatement(s) => self.builder.statement_do_while(
                SPAN,
                self.convert_statement(&s.body),
                self.convert_expression(&s.test),
            ),
            Statement::ForInStatement(s) => self.builder.statement_for_in(
                SPAN,
                self.convert_for_in_of_left(&s.left),
                self.convert_expression(&s.right),
                self.convert_statement(&s.body),
            ),
            Statement::ForOfStatement(s) => self.builder.statement_for_of(
                SPAN,
                s.is_await,
                self.convert_for_in_of_left(&s.left),
                self.convert_expression(&s.right),
                self.convert_statement(&s.body),
            ),
            Statement::SwitchStatement(s) => {
                let cases = self.builder.vec_from_iter(s.cases.iter().map(|c| {
                    self.builder.switch_case(
                        SPAN,
                        c.test.as_ref().map(|t| self.convert_expression(t)),
                        self.convert_statement_vec(&c.consequent),
                    )
                }));
                self.builder.statement_switch(SPAN, self.convert_expression(&s.discriminant), cases)
            }
            Statement::ThrowStatement(s) => {
                self.builder.statement_throw(SPAN, self.convert_expression(&s.argument))
            }
            Statement::TryStatement(s) => {
                let block = self.convert_block_statement(&s.block);
                let handler = s.handler.as_ref().map(|h| self.convert_catch_clause(h));
                let finalizer = s.finalizer.as_ref().map(|f| self.convert_block_statement(f));
                self.builder.statement_try(SPAN, block, handler, finalizer)
            }
            Statement::BreakStatement(s) => {
                let label = s
                    .label
                    .as_ref()
                    .map(|l| self.builder.label_identifier(SPAN, self.atom(&l.name)));
                self.builder.statement_break(SPAN, label)
            }
            Statement::ContinueStatement(s) => {
                let label = s
                    .label
                    .as_ref()
                    .map(|l| self.builder.label_identifier(SPAN, self.atom(&l.name)));
                self.builder.statement_continue(SPAN, label)
            }
            Statement::LabeledStatement(s) => {
                let label = self.builder.label_identifier(SPAN, self.atom(&s.label.name));
                self.builder.statement_labeled(SPAN, label, self.convert_statement(&s.body))
            }
            Statement::EmptyStatement(_) => self.builder.statement_empty(SPAN),
            Statement::DebuggerStatement(_) => self.builder.statement_debugger(SPAN),
            Statement::WithStatement(s) => self.builder.statement_with(
                SPAN,
                self.convert_expression(&s.object),
                self.convert_statement(&s.body),
            ),
            Statement::VariableDeclaration(d) => {
                let decl = self.convert_variable_declaration(d);
                oxc::Statement::VariableDeclaration(self.builder.alloc(decl))
            }
            Statement::FunctionDeclaration(f) => {
                let func = self.convert_function_decl(f, oxc::FunctionType::FunctionDeclaration);
                oxc::Statement::FunctionDeclaration(self.builder.alloc(func))
            }
            // The compiler never compiles classes, so the converter stubs out the
            // class body (members -> null forward, empty reverse). Re-parse the
            // original class from source to preserve its members; fall back to the
            // structural (member-less) conversion only if the slice is unavailable.
            Statement::ClassDeclaration(c) => {
                self.extract_source_stmt(&c.base).unwrap_or_else(|| {
                    oxc::Statement::ClassDeclaration(
                        self.builder.alloc(self.convert_class_declaration(c)),
                    )
                })
            }
            Statement::ImportDeclaration(d) => {
                if d.specifiers.is_empty()
                    && let Some(stmt) = self.extract_source_stmt(&d.base)
                {
                    return stmt;
                }
                let decl = self.convert_import_declaration(d);
                oxc::Statement::ImportDeclaration(self.builder.alloc(decl))
            }
            Statement::ExportNamedDeclaration(d) => {
                if self.export_named_needs_source_stmt(d) {
                    if let Some(stmt) = self.extract_source_stmt(&d.base) {
                        return stmt;
                    }
                }
                let decl = self.convert_export_named_declaration(d);
                oxc::Statement::ExportNamedDeclaration(self.builder.alloc(decl))
            }
            Statement::ExportDefaultDeclaration(d) => {
                if self.export_default_needs_source_stmt(d) {
                    return self.extract_source_stmt(&d.base).unwrap_or_else(|| {
                        if matches!(d.declaration.as_ref(), ExportDefaultDecl::Expression(_)) {
                            self.builder.statement_empty(SPAN)
                        } else {
                            let decl = self.convert_export_default_declaration(d);
                            oxc::Statement::ExportDefaultDeclaration(self.builder.alloc(decl))
                        }
                    });
                }
                let decl = self.convert_export_default_declaration(d);
                oxc::Statement::ExportDefaultDeclaration(self.builder.alloc(decl))
            }
            Statement::ExportAllDeclaration(d) => {
                if let Some(stmt) = self.extract_source_stmt(&d.base) {
                    return stmt;
                }
                let decl = self.convert_export_all_declaration(d);
                oxc::Statement::ExportAllDeclaration(self.builder.alloc(decl))
            }
            // TS/Flow declarations - try to extract from source text, fall back to empty
            Statement::TSTypeAliasDeclaration(d) => self
                .extract_source_stmt(&d.base)
                .unwrap_or_else(|| self.builder.statement_empty(SPAN)),
            Statement::TSInterfaceDeclaration(d) => self
                .extract_source_stmt(&d.base)
                .unwrap_or_else(|| self.builder.statement_empty(SPAN)),
            Statement::TSEnumDeclaration(d) => self
                .extract_source_stmt(&d.base)
                .unwrap_or_else(|| self.builder.statement_empty(SPAN)),
            Statement::TSModuleDeclaration(d) => self
                .extract_source_stmt(&d.base)
                .unwrap_or_else(|| self.builder.statement_empty(SPAN)),
            Statement::TSDeclareFunction(d) => self
                .extract_source_stmt(&d.base)
                .unwrap_or_else(|| self.builder.statement_empty(SPAN)),
            Statement::TypeAlias(d) => self
                .extract_source_stmt(&d.base)
                .unwrap_or_else(|| self.builder.statement_empty(SPAN)),
            Statement::OpaqueType(d) => self
                .extract_source_stmt(&d.base)
                .unwrap_or_else(|| self.builder.statement_empty(SPAN)),
            Statement::InterfaceDeclaration(d) => self
                .extract_source_stmt(&d.base)
                .unwrap_or_else(|| self.builder.statement_empty(SPAN)),
            Statement::EnumDeclaration(d) => self
                .extract_source_stmt(&d.base)
                .unwrap_or_else(|| self.builder.statement_empty(SPAN)),
            Statement::DeclareVariable(_)
            | Statement::DeclareFunction(_)
            | Statement::DeclareClass(_)
            | Statement::DeclareModule(_)
            | Statement::DeclareModuleExports(_)
            | Statement::DeclareExportDeclaration(_)
            | Statement::DeclareExportAllDeclaration(_)
            | Statement::DeclareInterface(_)
            | Statement::DeclareTypeAlias(_)
            | Statement::DeclareOpaqueType(_) => self.builder.statement_empty(SPAN),
            Statement::Unknown(s) => self
                .extract_source_stmt(s.base())
                .unwrap_or_else(|| self.builder.statement_empty(SPAN)),
        }
    }

    fn convert_statement_vec(
        &self,
        stmts: &[Statement],
    ) -> oxc_allocator::Vec<'a, oxc::Statement<'a>> {
        self.builder.vec_from_iter(stmts.iter().map(|s| self.convert_statement(s)))
    }

    fn convert_block_statement(&self, block: &BlockStatement) -> oxc::BlockStatement<'a> {
        self.builder.block_statement(SPAN, self.convert_statement_vec(&block.body))
    }

    fn convert_catch_clause(&self, clause: &CatchClause) -> oxc::CatchClause<'a> {
        let param = clause.param.as_ref().map(|p| {
            let pattern = self.convert_pattern_to_binding_pattern(p);
            let type_annotation = self.pattern_type_annotation(p);
            self.builder.catch_parameter(SPAN, pattern, type_annotation)
        });
        self.builder.catch_clause(SPAN, param, self.convert_block_statement(&clause.body))
    }

    fn convert_for_init(&self, init: &ForInit) -> oxc::ForStatementInit<'a> {
        match init {
            ForInit::VariableDeclaration(v) => {
                let decl = self.convert_variable_declaration(v);
                oxc::ForStatementInit::VariableDeclaration(self.builder.alloc(decl))
            }
            ForInit::Expression(e) => oxc::ForStatementInit::from(self.convert_expression(e)),
        }
    }

    fn convert_for_in_of_left(&self, left: &ForInOfLeft) -> oxc::ForStatementLeft<'a> {
        match left {
            ForInOfLeft::VariableDeclaration(v) => {
                let decl = self.convert_variable_declaration(v);
                oxc::ForStatementLeft::VariableDeclaration(self.builder.alloc(decl))
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
        let declarators = self.builder.vec_from_iter(
            decl.declarations.iter().map(|d| self.convert_variable_declarator(d, kind)),
        );
        let declare = decl.declare.unwrap_or(false);
        self.builder.variable_declaration(SPAN, kind, declarators, declare)
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
        self.builder.variable_declarator(SPAN, kind, id, type_annotation, init, definite)
    }

    // ===== Expressions =====

    fn convert_expression(&self, expr: &Expression) -> oxc::Expression<'a> {
        let converted = match expr {
            Expression::Identifier(id) => {
                self.builder.expression_identifier(SPAN, self.atom(&id.name))
            }
            Expression::StringLiteral(lit) => self.builder.expression_string_literal(
                SPAN,
                self.atom(&lit.value.to_string_lossy()),
                None,
            ),
            Expression::NumericLiteral(lit) => self.builder.expression_numeric_literal(
                SPAN,
                lit.value,
                None,
                oxc::NumberBase::Decimal,
            ),
            Expression::BooleanLiteral(lit) => {
                self.builder.expression_boolean_literal(SPAN, lit.value)
            }
            Expression::NullLiteral(_) => self.builder.expression_null_literal(SPAN),
            Expression::BigIntLiteral(lit) => self.builder.expression_big_int_literal(
                SPAN,
                self.atom(lit.value.strip_suffix('n').unwrap_or(&lit.value)),
                None,
                oxc::BigintBase::Decimal,
            ),
            Expression::RegExpLiteral(lit) => {
                let flags = self.parse_regexp_flags(&lit.flags);
                let pattern =
                    oxc::RegExpPattern { text: self.atom(&lit.pattern).into(), pattern: None };
                let regex = oxc::RegExp { pattern, flags };
                self.builder.expression_reg_exp_literal(SPAN, regex, None)
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
                self.builder.expression_call(SPAN, callee, type_arguments, args, false)
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
                let chain_call = self.builder.chain_element_call_expression(
                    SPAN,
                    callee,
                    type_arguments,
                    args,
                    call.optional,
                );
                self.builder.expression_chain(SPAN, chain_call)
            }
            Expression::OptionalMemberExpression(m) => {
                let chain_elem = self.convert_optional_member_to_chain_element(m);
                self.builder.expression_chain(SPAN, chain_elem)
            }
            Expression::BinaryExpression(bin) => {
                let op = self.convert_binary_operator(&bin.operator);
                self.builder.expression_binary(
                    SPAN,
                    self.convert_expression(&bin.left),
                    op,
                    self.convert_expression(&bin.right),
                )
            }
            Expression::LogicalExpression(log) => {
                let op = self.convert_logical_operator(&log.operator);
                self.builder.expression_logical(
                    SPAN,
                    self.convert_expression(&log.left),
                    op,
                    self.convert_expression(&log.right),
                )
            }
            Expression::UnaryExpression(un) => {
                let op = self.convert_unary_operator(&un.operator);
                self.builder.expression_unary(SPAN, op, self.convert_expression(&un.argument))
            }
            Expression::UpdateExpression(up) => {
                let op = self.convert_update_operator(&up.operator);
                let arg = self.convert_expression_to_simple_assignment_target(&up.argument);
                self.builder.expression_update(SPAN, op, up.prefix, arg)
            }
            Expression::ConditionalExpression(cond) => self.builder.expression_conditional(
                SPAN,
                self.convert_expression(&cond.test),
                self.convert_expression(&cond.consequent),
                self.convert_expression(&cond.alternate),
            ),
            Expression::AssignmentExpression(assign) => {
                let op = self.convert_assignment_operator(&assign.operator);
                let left = self.convert_pattern_to_assignment_target(&assign.left);
                self.builder.expression_assignment(
                    SPAN,
                    op,
                    left,
                    self.convert_expression(&assign.right),
                )
            }
            Expression::SequenceExpression(seq) => {
                let exprs = self
                    .builder
                    .vec_from_iter(seq.expressions.iter().map(|e| self.convert_expression(e)));
                self.builder.expression_sequence(SPAN, exprs)
            }
            Expression::ArrowFunctionExpression(arrow) => self.convert_arrow_function(arrow),
            Expression::FunctionExpression(func) => {
                let f = self.convert_function_expr(func);
                oxc::Expression::FunctionExpression(self.builder.alloc(f))
            }
            Expression::ObjectExpression(obj) => {
                let properties = self.builder.vec_from_iter(
                    obj.properties.iter().map(|p| self.convert_object_expression_property(p)),
                );
                self.builder.expression_object(SPAN, properties)
            }
            Expression::ArrayExpression(arr) => {
                let elements = self
                    .builder
                    .vec_from_iter(arr.elements.iter().map(|e| self.convert_array_element(e)));
                self.builder.expression_array(SPAN, elements)
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
                self.builder.expression_new(SPAN, callee, type_arguments, args)
            }
            Expression::TemplateLiteral(tl) => {
                let template = self.convert_template_literal(tl);
                oxc::Expression::TemplateLiteral(self.builder.alloc(template))
            }
            Expression::TaggedTemplateExpression(tag) => {
                let t = self.convert_expression(&tag.tag);
                let quasi = self.convert_template_literal(&tag.quasi);
                let type_arguments = if tag.type_parameters.is_some() {
                    self.extract_source_tagged_template_type_arguments(&tag.base)
                } else {
                    None
                };
                self.builder.expression_tagged_template(SPAN, t, type_arguments, quasi)
            }
            Expression::AwaitExpression(a) => {
                self.builder.expression_await(SPAN, self.convert_expression(&a.argument))
            }
            Expression::YieldExpression(y) => self.builder.expression_yield(
                SPAN,
                y.delegate,
                y.argument.as_ref().map(|a| self.convert_expression(a)),
            ),
            Expression::SpreadElement(s) => {
                // SpreadElement can't be a standalone expression in OXC.
                // Return the argument directly as a fallback.
                self.convert_expression(&s.argument)
            }
            Expression::MetaProperty(mp) => {
                let meta = self.builder.identifier_name(SPAN, self.atom(&mp.meta.name));
                let property = self.builder.identifier_name(SPAN, self.atom(&mp.property.name));
                self.builder.expression_meta_property(SPAN, meta, property)
            }
            Expression::ClassExpression(c) => {
                if let Some(expr) = self.extract_source_class_expression(c) {
                    return expr;
                }
                let class = self.convert_class_to_oxc(c, oxc::ClassType::ClassExpression);
                oxc::Expression::ClassExpression(self.builder.alloc(class))
            }
            Expression::PrivateName(_) => {
                self.builder.expression_identifier(SPAN, self.atom("__private__"))
            }
            Expression::Super(_) => self.builder.expression_super(SPAN),
            Expression::Import(_) => {
                self.builder.expression_identifier(SPAN, self.atom("__import__"))
            }
            Expression::ThisExpression(_) => self.builder.expression_this(SPAN),
            Expression::ParenthesizedExpression(p) => {
                self.builder.expression_parenthesized(SPAN, self.convert_expression(&p.expression))
            }
            Expression::JSXElement(el) => {
                let element = self.convert_jsx_element(el);
                oxc::Expression::JSXElement(self.builder.alloc(element))
            }
            Expression::JSXFragment(frag) => {
                let fragment = self.convert_jsx_fragment(frag);
                oxc::Expression::JSXFragment(self.builder.alloc(fragment))
            }
            // TS expressions carry their actual type AST as `null` through the
            // React AST bridge. Rebuild the wrapper with the converted child
            // expression and recover only the type from the original source.
            Expression::TSAsExpression(e) => {
                let expression = self.convert_expression(&e.expression);
                if let Some(type_annotation) = self.convert_type_from_raw(&e.type_annotation) {
                    self.builder.expression_ts_as(SPAN, expression, type_annotation)
                } else {
                    expression
                }
            }
            Expression::TSSatisfiesExpression(e) => {
                let expression = self.convert_expression(&e.expression);
                if let Some(type_annotation) = self.convert_type_from_raw(&e.type_annotation) {
                    self.builder.expression_ts_satisfies(SPAN, expression, type_annotation)
                } else {
                    expression
                }
            }
            Expression::TSNonNullExpression(e) => {
                self.builder.expression_ts_non_null(SPAN, self.convert_expression(&e.expression))
            }
            Expression::TSTypeAssertion(e) => {
                let expression = self.convert_expression(&e.expression);
                if let Some(type_annotation) = self.convert_type_from_raw(&e.type_annotation) {
                    self.builder.expression_ts_type_assertion(SPAN, type_annotation, expression)
                } else {
                    expression
                }
            }
            Expression::TSInstantiationExpression(e) => {
                let expression = self.convert_expression(&e.expression);
                if let Some(type_arguments) =
                    self.extract_source_ts_instantiation_type_arguments(&e.base)
                {
                    self.builder.expression_ts_instantiation(SPAN, expression, type_arguments)
                } else {
                    expression
                }
            }
            Expression::TypeCastExpression(e) => self.convert_expression(&e.expression),
            Expression::AssignmentPattern(p) => {
                let left = self.convert_pattern_to_assignment_target(&p.left);
                self.builder.expression_assignment(
                    SPAN,
                    oxc_syntax::operator::AssignmentOperator::Assign,
                    left,
                    self.convert_expression(&p.right),
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
        Some(self.builder.expression_import(SPAN, source, options, None))
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
                let call_expr =
                    self.builder.call_expression(SPAN, callee, type_arguments, args, call.optional);
                oxc::Expression::CallExpression(self.builder.alloc(call_expr))
            }
            _ => self.convert_expression(expr),
        }
    }

    fn convert_member_expression(&self, m: &MemberExpression) -> oxc::Expression<'a> {
        let object = self.convert_expression(&m.object);
        if m.computed {
            let property = self.convert_expression(&m.property);
            oxc::Expression::ComputedMemberExpression(
                self.builder
                    .alloc(self.builder.computed_member_expression(SPAN, object, property, false)),
            )
        } else {
            let prop_name = self.expression_to_identifier_name(&m.property);
            oxc::Expression::StaticMemberExpression(
                self.builder
                    .alloc(self.builder.static_member_expression(SPAN, object, prop_name, false)),
            )
        }
    }

    fn convert_optional_member_to_chain_element(
        &self,
        m: &OptionalMemberExpression,
    ) -> oxc::ChainElement<'a> {
        let object = self.convert_expression_for_chain(&m.object);
        if m.computed {
            let property = self.convert_expression(&m.property);
            oxc::ChainElement::ComputedMemberExpression(
                self.builder.alloc(
                    self.builder.computed_member_expression(SPAN, object, property, m.optional),
                ),
            )
        } else {
            let prop_name = self.expression_to_identifier_name(&m.property);
            oxc::ChainElement::StaticMemberExpression(
                self.builder.alloc(
                    self.builder.static_member_expression(SPAN, object, prop_name, m.optional),
                ),
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
            oxc::Expression::ComputedMemberExpression(
                self.builder.alloc(
                    self.builder.computed_member_expression(SPAN, object, property, m.optional),
                ),
            )
        } else {
            let prop_name = self.expression_to_identifier_name(&m.property);
            oxc::Expression::StaticMemberExpression(
                self.builder.alloc(
                    self.builder.static_member_expression(SPAN, object, prop_name, m.optional),
                ),
            )
        }
    }

    fn expression_to_identifier_name(&self, expr: &Expression) -> oxc::IdentifierName<'a> {
        match expr {
            Expression::Identifier(id) => self.builder.identifier_name(SPAN, self.atom(&id.name)),
            _ => self.builder.identifier_name(SPAN, self.atom("__unknown__")),
        }
    }

    fn convert_arguments_with_source(
        &self,
        args: &[Expression],
        source_args: Option<Vec<oxc::Argument<'a>>>,
    ) -> oxc_allocator::Vec<'a, oxc::Argument<'a>> {
        let mut source_args = source_args.map(Vec::into_iter);
        self.builder.vec_from_iter(args.iter().map(|arg| {
            let source_arg = source_args.as_mut().and_then(Iterator::next);
            self.convert_argument_with_source(arg, source_arg)
        }))
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
                self.builder.argument_spread_element(SPAN, converted)
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
            None => self.builder.array_expression_element_elision(SPAN),
            Some(Expression::SpreadElement(s)) => {
                self.builder.array_expression_element_spread_element(
                    SPAN,
                    self.convert_expression(&s.argument),
                )
            }
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
                let obj_prop = self.builder.object_property(
                    SPAN,
                    oxc::PropertyKind::Init,
                    key,
                    value,
                    method,
                    p.shorthand,
                    p.computed,
                );
                oxc::ObjectPropertyKind::ObjectProperty(self.builder.alloc(obj_prop))
            }
            ObjectExpressionProperty::ObjectMethod(m) => {
                let kind = match m.kind {
                    ObjectMethodKind::Method => oxc::PropertyKind::Init,
                    ObjectMethodKind::Get => oxc::PropertyKind::Get,
                    ObjectMethodKind::Set => oxc::PropertyKind::Set,
                };
                let key = self.convert_expression_to_property_key(&m.key, m.computed);
                let func = self.convert_object_method_to_function(m);
                let func_expr = oxc::Expression::FunctionExpression(self.builder.alloc(func));
                let obj_prop = self.builder.object_property(
                    SPAN, kind, key, func_expr, m.method, false, // shorthand
                    m.computed,
                );
                oxc::ObjectPropertyKind::ObjectProperty(self.builder.alloc(obj_prop))
            }
            ObjectExpressionProperty::SpreadElement(s) => {
                let spread =
                    self.builder.spread_element(SPAN, self.convert_expression(&s.argument));
                oxc::ObjectPropertyKind::SpreadProperty(self.builder.alloc(spread))
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
                self.builder.property_key_static_identifier(SPAN, self.atom(&id.name))
            }
            Expression::StringLiteral(s) => {
                let lit =
                    self.builder.string_literal(SPAN, self.atom(&s.value.to_string_lossy()), None);
                oxc::PropertyKey::StringLiteral(self.builder.alloc(lit))
            }
            Expression::NumericLiteral(n) => {
                let lit =
                    self.builder.numeric_literal(SPAN, n.value, None, oxc::NumberBase::Decimal);
                oxc::PropertyKey::NumericLiteral(self.builder.alloc(lit))
            }
            Expression::PrivateName(p) => {
                self.builder.property_key_private_identifier(SPAN, self.atom(&p.id.name))
            }
            _ => oxc::PropertyKey::from(self.convert_expression(expr)),
        }
    }

    fn convert_template_literal(
        &self,
        tl: &crate::react_compiler_ast::expressions::TemplateLiteral,
    ) -> oxc::TemplateLiteral<'a> {
        let quasis = self.builder.vec_from_iter(tl.quasis.iter().map(|q| {
            let raw = self.atom(&q.value.raw).into();
            let cooked = q.value.cooked.as_ref().map(|c| self.atom(c).into());
            let value = oxc::TemplateElementValue { raw, cooked };
            self.builder.template_element(SPAN, value, q.tail)
        }));
        let expressions =
            self.builder.vec_from_iter(tl.expressions.iter().map(|e| self.convert_expression(e)));
        self.builder.template_literal(SPAN, quasis, expressions)
    }

    // ===== Functions =====

    fn convert_function_decl(
        &self,
        f: &FunctionDeclaration,
        fn_type: oxc::FunctionType,
    ) -> oxc::Function<'a> {
        let id = f.id.as_ref().map(|id| self.builder.binding_identifier(SPAN, self.atom(&id.name)));
        let params = self.convert_params_to_formal_parameters(&f.params);
        let body = self.convert_block_to_function_body(&f.body);
        let mut func = self.builder.function(
            SPAN,
            fn_type,
            id,
            f.generator,
            f.is_async,
            f.declare.unwrap_or(false),
            None::<oxc_allocator::Box<'a, oxc::TSTypeParameterDeclaration<'a>>>,
            None::<oxc_allocator::Box<'a, oxc::TSThisParameter<'a>>>,
            params,
            None::<oxc_allocator::Box<'a, oxc::TSTypeAnnotation<'a>>>,
            Some(body),
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
        let id = c.id.as_ref().map(|id| self.builder.binding_identifier(SPAN, self.atom(&id.name)));
        let super_class = c.super_class.as_ref().map(|s| self.convert_expression(s));
        let body = self.builder.class_body(SPAN, self.builder.vec());
        self.builder.class(
            SPAN,
            oxc::ClassType::ClassDeclaration,
            self.builder.vec(), // decorators
            id,
            None::<oxc_allocator::Box<'a, oxc::TSTypeParameterDeclaration<'a>>>,
            super_class,
            None::<oxc_allocator::Box<'a, oxc::TSTypeParameterInstantiation<'a>>>,
            self.builder.vec(), // implements
            body,
            c.is_abstract.unwrap_or(false),
            c.declare.unwrap_or(false),
        )
    }

    fn convert_class_to_oxc(
        &self,
        c: &crate::react_compiler_ast::expressions::ClassExpression,
        class_type: oxc::ClassType,
    ) -> oxc::Class<'a> {
        let id = c.id.as_ref().map(|id| self.builder.binding_identifier(SPAN, self.atom(&id.name)));
        let super_class = c.super_class.as_ref().map(|s| self.convert_expression(s));
        let body = self.builder.class_body(SPAN, self.builder.vec());
        self.builder.class(
            SPAN,
            class_type,
            self.builder.vec(), // decorators
            id,
            None::<oxc_allocator::Box<'a, oxc::TSTypeParameterDeclaration<'a>>>,
            super_class,
            None::<oxc_allocator::Box<'a, oxc::TSTypeParameterInstantiation<'a>>>,
            self.builder.vec(), // implements
            body,
            false, // is_abstract
            false, // declare
        )
    }

    fn convert_function_expr(&self, f: &FunctionExpression) -> oxc::Function<'a> {
        let id = f.id.as_ref().map(|id| self.builder.binding_identifier(SPAN, self.atom(&id.name)));
        let params = self.convert_params_to_formal_parameters(&f.params);
        let body = self.convert_block_to_function_body(&f.body);
        let initializes_react_cache = self.block_initializes_react_cache(&f.body);
        let return_type = if initializes_react_cache {
            None
        } else {
            f.return_type.as_ref().and_then(|value| self.convert_type_annotation_from_raw(value))
        };
        let mut func = self.builder.function(
            SPAN,
            oxc::FunctionType::FunctionExpression,
            id,
            f.generator,
            f.is_async,
            false,
            None::<oxc_allocator::Box<'a, oxc::TSTypeParameterDeclaration<'a>>>,
            None::<oxc_allocator::Box<'a, oxc::TSThisParameter<'a>>>,
            params,
            return_type,
            Some(body),
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
            m.return_type.as_ref().and_then(|value| self.convert_type_annotation_from_raw(value))
        };
        let mut func = self.builder.function(
            SPAN,
            oxc::FunctionType::FunctionExpression,
            None,
            m.generator,
            m.is_async,
            false,
            None::<oxc_allocator::Box<'a, oxc::TSTypeParameterDeclaration<'a>>>,
            None::<oxc_allocator::Box<'a, oxc::TSThisParameter<'a>>>,
            params,
            return_type,
            Some(body),
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
                let stmt = self.builder.statement_expression(SPAN, oxc_expr);
                let stmts = self.builder.vec_from_iter(std::iter::once(stmt));
                self.builder.function_body(SPAN, self.builder.vec(), stmts)
            }
        };

        let mut arrow_expr = self.builder.arrow_function_expression(
            SPAN,
            is_expression,
            arrow.is_async,
            None::<oxc_allocator::Box<'a, oxc::TSTypeParameterDeclaration<'a>>>,
            params,
            if arrow_initializes_react_cache {
                None
            } else {
                arrow
                    .return_type
                    .as_ref()
                    .and_then(|value| self.convert_type_annotation_from_raw(value))
            },
            body,
        );
        if !arrow_initializes_react_cache
            && let Some(source_arrow) = self.extract_source_arrow_function(&arrow.base)
        {
            self.apply_arrow_signature_from_source(&mut arrow_expr, source_arrow);
        }
        oxc::Expression::ArrowFunctionExpression(self.builder.alloc(arrow_expr))
    }

    fn convert_block_to_function_body(&self, block: &BlockStatement) -> oxc::FunctionBody<'a> {
        let stmts = self.convert_statement_vec(&block.body);
        let directives = self.convert_directives(&block.directives);
        self.builder.function_body(SPAN, directives, stmts)
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
                    let rest_elem = self.builder.binding_rest_element(SPAN, arg);
                    let type_annotation = r
                        .type_annotation
                        .as_ref()
                        .and_then(|value| self.convert_type_annotation_from_raw(value));
                    rest = Some(self.builder.formal_parameter_rest(
                        SPAN,
                        self.builder.vec(),
                        rest_elem,
                        type_annotation,
                    ));
                }
                PatternLike::AssignmentPattern(ap) => {
                    // OXC stores default parameter values in FormalParameter.initializer
                    // rather than using BindingPattern::AssignmentPattern (which OXC considers
                    // invalid in FormalParameter position).
                    let left = self.convert_pattern_to_binding_pattern(&ap.left);
                    let right = self.convert_expression(&ap.right);
                    let initializer = Some(oxc_allocator::Box::new_in(right, self.allocator));
                    let type_annotation = self
                        .pattern_type_annotation(param)
                        .or_else(|| self.pattern_type_annotation(&ap.left));
                    let optional = self.pattern_optional(param) || self.pattern_optional(&ap.left);
                    let fp = self.builder.formal_parameter(
                        SPAN,
                        self.builder.vec(), // decorators
                        left,
                        type_annotation,
                        initializer,
                        optional,
                        None,  // accessibility
                        false, // readonly
                        false, // override
                    );
                    items.push(fp);
                }
                _ => {
                    let pattern = self.convert_pattern_to_binding_pattern(param);
                    let type_annotation = self.pattern_type_annotation(param);
                    let optional = self.pattern_optional(param);
                    let fp = self.builder.formal_parameter(
                        SPAN,
                        self.builder.vec(), // decorators
                        pattern,
                        type_annotation,
                        None::<oxc_allocator::Box<'a, oxc::Expression<'a>>>,
                        optional,
                        None,  // accessibility
                        false, // readonly
                        false, // override
                    );
                    items.push(fp);
                }
            }
        }

        let items_vec = self.builder.vec_from_iter(items);
        self.builder.formal_parameters(
            SPAN,
            oxc::FormalParameterKind::FormalParameter,
            items_vec,
            rest,
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
        self.convert_type_annotation_from_raw(value)
    }

    // ===== Patterns → BindingPattern =====

    fn convert_pattern_to_binding_pattern(&self, pattern: &PatternLike) -> oxc::BindingPattern<'a> {
        match pattern {
            PatternLike::Identifier(id) => {
                self.builder.binding_pattern_binding_identifier(SPAN, self.atom(&id.name))
            }
            PatternLike::ObjectPattern(obj) => {
                let mut properties: Vec<oxc::BindingProperty<'a>> = Vec::new();
                let mut rest: Option<oxc::BindingRestElement<'a>> = None;

                for prop in &obj.properties {
                    match prop {
                        ObjectPatternProperty::ObjectProperty(p) => {
                            let key = self.convert_expression_to_property_key(&p.key, p.computed);
                            let value = self.convert_pattern_to_binding_pattern(&p.value);
                            let bp = self.builder.binding_property(
                                SPAN,
                                key,
                                value,
                                p.shorthand,
                                p.computed,
                            );
                            properties.push(bp);
                        }
                        ObjectPatternProperty::RestElement(r) => {
                            let arg = self.convert_pattern_to_binding_pattern(&r.argument);
                            rest = Some(self.builder.binding_rest_element(SPAN, arg));
                        }
                    }
                }

                let props_vec = self.builder.vec_from_iter(properties);
                self.builder.binding_pattern_object_pattern(SPAN, props_vec, rest)
            }
            PatternLike::ArrayPattern(arr) => {
                let mut elements: Vec<Option<oxc::BindingPattern<'a>>> = Vec::new();
                let mut rest: Option<oxc::BindingRestElement<'a>> = None;

                for elem in &arr.elements {
                    match elem {
                        None => elements.push(None),
                        Some(PatternLike::RestElement(r)) => {
                            let arg = self.convert_pattern_to_binding_pattern(&r.argument);
                            rest = Some(self.builder.binding_rest_element(SPAN, arg));
                        }
                        Some(p) => {
                            elements.push(Some(self.convert_pattern_to_binding_pattern(p)));
                        }
                    }
                }

                let elems_vec = self.builder.vec_from_iter(elements);
                self.builder.binding_pattern_array_pattern(SPAN, elems_vec, rest)
            }
            PatternLike::AssignmentPattern(ap) => {
                let left = self.convert_pattern_to_binding_pattern(&ap.left);
                let right = self.convert_expression(&ap.right);
                self.builder.binding_pattern_assignment_pattern(SPAN, left, right)
            }
            PatternLike::RestElement(r) => self.convert_pattern_to_binding_pattern(&r.argument),
            PatternLike::MemberExpression(_)
            | PatternLike::TSAsExpression(_)
            | PatternLike::TSSatisfiesExpression(_)
            | PatternLike::TSNonNullExpression(_)
            | PatternLike::TSTypeAssertion(_)
            | PatternLike::TypeCastExpression(_) => self
                .builder
                .binding_pattern_binding_identifier(SPAN, self.atom("__member_pattern__")),
        }
    }

    // ===== Patterns → AssignmentTarget =====

    fn convert_pattern_to_assignment_target(
        &self,
        pattern: &PatternLike,
    ) -> oxc::AssignmentTarget<'a> {
        match pattern {
            PatternLike::Identifier(id) => self
                .builder
                .simple_assignment_target_assignment_target_identifier(SPAN, self.atom(&id.name))
                .into(),
            PatternLike::MemberExpression(m) => {
                let object = self.convert_expression(&m.object);
                if m.computed {
                    let property = self.convert_expression(&m.property);
                    let mem =
                        self.builder.computed_member_expression(SPAN, object, property, false);
                    oxc::AssignmentTarget::ComputedMemberExpression(self.builder.alloc(mem))
                } else {
                    let prop_name = self.expression_to_identifier_name(&m.property);
                    let mem = self.builder.static_member_expression(SPAN, object, prop_name, false);
                    oxc::AssignmentTarget::StaticMemberExpression(self.builder.alloc(mem))
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
                                    let binding = self
                                        .builder
                                        .identifier_reference(SPAN, self.atom(&id.name));
                                    let init = match &*p.value {
                                        PatternLike::AssignmentPattern(ap) => {
                                            Some(self.convert_expression(&ap.right))
                                        }
                                        _ => None,
                                    };
                                    let atp = self
                                        .builder
                                        .assignment_target_property_assignment_target_property_identifier(
                                            SPAN, binding, init,
                                        );
                                    properties.push(atp);
                                } else {
                                    // Fallback to non-shorthand
                                    let key =
                                        self.convert_expression_to_property_key(&p.key, p.computed);
                                    let binding = self
                                        .convert_pattern_to_assignment_target_maybe_default(
                                            &p.value,
                                        );
                                    let atp = self
                                        .builder
                                        .assignment_target_property_assignment_target_property_property(
                                            SPAN, key, binding, p.computed,
                                        );
                                    properties.push(atp);
                                }
                            } else {
                                let key =
                                    self.convert_expression_to_property_key(&p.key, p.computed);
                                let binding = self
                                    .convert_pattern_to_assignment_target_maybe_default(&p.value);
                                let atp = self
                                    .builder
                                    .assignment_target_property_assignment_target_property_property(
                                        SPAN, key, binding, p.computed,
                                    );
                                properties.push(atp);
                            }
                        }
                        ObjectPatternProperty::RestElement(r) => {
                            let target = self.convert_pattern_to_assignment_target(&r.argument);
                            rest = Some(self.builder.assignment_target_rest(SPAN, target));
                        }
                    }
                }

                let props_vec = self.builder.vec_from_iter(properties);
                self.builder
                    .assignment_target_pattern_object_assignment_target(SPAN, props_vec, rest)
                    .into()
            }
            PatternLike::ArrayPattern(arr) => {
                let mut elements: Vec<Option<oxc::AssignmentTargetMaybeDefault<'a>>> = Vec::new();
                let mut rest: Option<oxc::AssignmentTargetRest<'a>> = None;

                for elem in &arr.elements {
                    match elem {
                        None => elements.push(None),
                        Some(PatternLike::RestElement(r)) => {
                            let target = self.convert_pattern_to_assignment_target(&r.argument);
                            rest = Some(self.builder.assignment_target_rest(SPAN, target));
                        }
                        Some(p) => {
                            elements.push(Some(
                                self.convert_pattern_to_assignment_target_maybe_default(p),
                            ));
                        }
                    }
                }

                let elems_vec = self.builder.vec_from_iter(elements);
                self.builder
                    .assignment_target_pattern_array_assignment_target(SPAN, elems_vec, rest)
                    .into()
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
            PatternLike::TypeCastExpression(_) => self
                .builder
                .simple_assignment_target_assignment_target_identifier(
                    SPAN,
                    self.atom("__unknown__"),
                )
                .into(),
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
                self.builder.assignment_target_maybe_default_assignment_target_with_default(
                    SPAN, binding, init,
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
            Expression::Identifier(id) => self
                .builder
                .simple_assignment_target_assignment_target_identifier(SPAN, self.atom(&id.name)),
            Expression::MemberExpression(m) => {
                let object = self.convert_expression(&m.object);
                if m.computed {
                    let property = self.convert_expression(&m.property);
                    let mem =
                        self.builder.computed_member_expression(SPAN, object, property, false);
                    oxc::SimpleAssignmentTarget::ComputedMemberExpression(self.builder.alloc(mem))
                } else {
                    let prop_name = self.expression_to_identifier_name(&m.property);
                    let mem = self.builder.static_member_expression(SPAN, object, prop_name, false);
                    oxc::SimpleAssignmentTarget::StaticMemberExpression(self.builder.alloc(mem))
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
            _ => self.builder.simple_assignment_target_assignment_target_identifier(
                SPAN,
                self.atom("__unknown__"),
            ),
        }
    }

    fn convert_ts_as_expression_to_simple_assignment_target(
        &self,
        expr: &TSAsExpression,
    ) -> oxc::SimpleAssignmentTarget<'a> {
        let expression = self.convert_expression(&expr.expression);
        if let Some(type_annotation) = self.convert_type_from_raw(&expr.type_annotation) {
            self.builder.simple_assignment_target_ts_as_expression(
                SPAN,
                expression,
                type_annotation,
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
        if let Some(type_annotation) = self.convert_type_from_raw(&expr.type_annotation) {
            self.builder.simple_assignment_target_ts_satisfies_expression(
                SPAN,
                expression,
                type_annotation,
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
        self.builder.simple_assignment_target_ts_non_null_expression(SPAN, expression)
    }

    fn convert_ts_type_assertion_to_simple_assignment_target(
        &self,
        expr: &TSTypeAssertion,
    ) -> oxc::SimpleAssignmentTarget<'a> {
        let expression = self.convert_expression(&expr.expression);
        if let Some(type_annotation) = self.convert_type_from_raw(&expr.type_annotation) {
            self.builder.simple_assignment_target_ts_type_assertion(
                SPAN,
                type_annotation,
                expression,
            )
        } else {
            self.convert_expression_to_simple_assignment_target(&expr.expression)
        }
    }

    // ===== JSX =====

    fn convert_jsx_element(&self, el: &JSXElement) -> oxc::JSXElement<'a> {
        let opening = self.convert_jsx_opening_element(&el.opening_element, Some(&el.base));
        let children =
            self.builder.vec_from_iter(el.children.iter().map(|c| self.convert_jsx_child(c)));
        let closing = el.closing_element.as_ref().map(|c| self.convert_jsx_closing_element(c));
        self.builder.jsx_element(SPAN, opening, children, closing)
    }

    fn convert_jsx_opening_element(
        &self,
        el: &JSXOpeningElement,
        element_base: Option<&BaseNode>,
    ) -> oxc::JSXOpeningElement<'a> {
        let name = self.convert_jsx_element_name(&el.name);
        let attrs = self
            .builder
            .vec_from_iter(el.attributes.iter().map(|a| self.convert_jsx_attribute_item(a)));
        let type_arguments = self
            .extract_source_jsx_type_arguments(&el.base)
            .or_else(|| element_base.and_then(|base| self.extract_source_jsx_type_arguments(base)));
        self.builder.jsx_opening_element(SPAN, name, type_arguments, attrs)
    }

    fn convert_jsx_closing_element(&self, el: &JSXClosingElement) -> oxc::JSXClosingElement<'a> {
        let name = self.convert_jsx_element_name(&el.name);
        self.builder.jsx_closing_element(SPAN, name)
    }

    fn convert_jsx_element_name(&self, name: &JSXElementName) -> oxc::JSXElementName<'a> {
        match name {
            JSXElementName::JSXIdentifier(id) => {
                let first_char = id.name.chars().next().unwrap_or('a');
                if first_char.is_uppercase() || id.name.contains('.') {
                    self.builder.jsx_element_name_identifier_reference(SPAN, self.atom(&id.name))
                } else {
                    self.builder.jsx_element_name_identifier(SPAN, self.atom(&id.name))
                }
            }
            JSXElementName::JSXMemberExpression(m) => {
                let member = self.convert_jsx_member_expression(m);
                self.builder.jsx_element_name_member_expression(
                    SPAN,
                    member.object,
                    member.property,
                )
            }
            JSXElementName::JSXNamespacedName(ns) => {
                let namespace = self.builder.jsx_identifier(SPAN, self.atom(&ns.namespace.name));
                let name = self.builder.jsx_identifier(SPAN, self.atom(&ns.name.name));
                self.builder.jsx_element_name_namespaced_name(SPAN, namespace, name)
            }
        }
    }

    fn convert_jsx_member_expression(
        &self,
        m: &JSXMemberExpression,
    ) -> oxc::JSXMemberExpression<'a> {
        let object = self.convert_jsx_member_expression_object(&m.object);
        let property = self.builder.jsx_identifier(SPAN, self.atom(&m.property.name));
        self.builder.jsx_member_expression(SPAN, object, property)
    }

    fn convert_jsx_member_expression_object(
        &self,
        obj: &JSXMemberExprObject,
    ) -> oxc::JSXMemberExpressionObject<'a> {
        match obj {
            JSXMemberExprObject::JSXIdentifier(id) => self
                .builder
                .jsx_member_expression_object_identifier_reference(SPAN, self.atom(&id.name)),
            JSXMemberExprObject::JSXMemberExpression(m) => {
                let member = self.convert_jsx_member_expression(m);
                self.builder.jsx_member_expression_object_member_expression(
                    SPAN,
                    member.object,
                    member.property,
                )
            }
        }
    }

    fn convert_jsx_attribute_item(&self, item: &JSXAttributeItem) -> oxc::JSXAttributeItem<'a> {
        match item {
            JSXAttributeItem::JSXAttribute(attr) => {
                let name = self.convert_jsx_attribute_name(&attr.name);
                let value = attr.value.as_ref().map(|v| self.convert_jsx_attribute_value(v));
                self.builder.jsx_attribute_item_attribute(SPAN, name, value)
            }
            JSXAttributeItem::JSXSpreadAttribute(s) => self
                .builder
                .jsx_attribute_item_spread_attribute(SPAN, self.convert_expression(&s.argument)),
        }
    }

    fn convert_jsx_attribute_name(&self, name: &JSXAttributeName) -> oxc::JSXAttributeName<'a> {
        match name {
            JSXAttributeName::JSXIdentifier(id) => {
                self.builder.jsx_attribute_name_identifier(SPAN, self.atom(&id.name))
            }
            JSXAttributeName::JSXNamespacedName(ns) => {
                let namespace = self.builder.jsx_identifier(SPAN, self.atom(&ns.namespace.name));
                let name = self.builder.jsx_identifier(SPAN, self.atom(&ns.name.name));
                self.builder.jsx_attribute_name_namespaced_name(SPAN, namespace, name)
            }
        }
    }

    fn convert_jsx_attribute_value(&self, value: &JSXAttributeValue) -> oxc::JSXAttributeValue<'a> {
        match value {
            JSXAttributeValue::StringLiteral(s) => self.builder.jsx_attribute_value_string_literal(
                SPAN,
                self.atom(&s.value.to_string_lossy()),
                None,
            ),
            JSXAttributeValue::JSXExpressionContainer(ec) => {
                let expr = self.convert_jsx_expression_container_expr(&ec.expression);
                self.builder.jsx_attribute_value_expression_container(SPAN, expr)
            }
            JSXAttributeValue::JSXElement(el) => {
                let element = self.convert_jsx_element(el);
                let opening = element.opening_element;
                let closing = element.closing_element;
                self.builder.jsx_attribute_value_element(SPAN, opening, element.children, closing)
            }
            JSXAttributeValue::JSXFragment(frag) => {
                let fragment = self.convert_jsx_fragment(frag);
                self.builder.jsx_attribute_value_fragment(
                    SPAN,
                    fragment.opening_fragment,
                    fragment.children,
                    fragment.closing_fragment,
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
                self.builder.jsx_expression_empty_expression(SPAN)
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
                self.builder.jsx_child_text(SPAN, self.atom(&value), None)
            }
            JSXChild::JSXElement(el) => {
                let element = self.convert_jsx_element(el);
                let opening = element.opening_element;
                let closing = element.closing_element;
                self.builder.jsx_child_element(SPAN, opening, element.children, closing)
            }
            JSXChild::JSXFragment(frag) => {
                let fragment = self.convert_jsx_fragment(frag);
                self.builder.jsx_child_fragment(
                    SPAN,
                    fragment.opening_fragment,
                    fragment.children,
                    fragment.closing_fragment,
                )
            }
            JSXChild::JSXExpressionContainer(ec) => {
                let expr = self.convert_jsx_expression_container_expr(&ec.expression);
                self.builder.jsx_child_expression_container(SPAN, expr)
            }
            JSXChild::JSXSpreadChild(s) => {
                self.builder.jsx_child_spread(SPAN, self.convert_expression(&s.expression))
            }
        }
    }

    fn convert_jsx_fragment(&self, frag: &JSXFragment) -> oxc::JSXFragment<'a> {
        let opening = self.builder.jsx_opening_fragment(SPAN);
        let closing = self.builder.jsx_closing_fragment(SPAN);
        let children =
            self.builder.vec_from_iter(frag.children.iter().map(|c| self.convert_jsx_child(c)));
        self.builder.jsx_fragment(SPAN, opening, children, closing)
    }

    // ===== Import/Export =====

    fn convert_import_declaration(&self, decl: &ImportDeclaration) -> oxc::ImportDeclaration<'a> {
        let specifiers = self
            .builder
            .vec_from_iter(decl.specifiers.iter().map(|s| self.convert_import_specifier(s)));
        let specifiers =
            if specifiers.is_empty() && !self.import_declaration_has_empty_named_specifiers(decl) {
                None
            } else {
                Some(specifiers)
            };
        let source = self.builder.string_literal(
            SPAN,
            self.atom(&decl.source.value.to_string_lossy()),
            None,
        );
        let import_kind = match decl.import_kind.as_ref() {
            Some(ImportKind::Type) => oxc::ImportOrExportKind::Type,
            _ => oxc::ImportOrExportKind::Value,
        };
        let with_clause =
            self.convert_with_clause(decl.attributes.as_deref().or(decl.assertions.as_deref()));
        self.builder.import_declaration(
            SPAN,
            specifiers,
            source,
            None, // phase
            with_clause,
            import_kind,
        )
    }

    fn convert_with_clause(
        &self,
        attributes: Option<&[ImportAttribute]>,
    ) -> Option<oxc_allocator::Box<'a, oxc::WithClause<'a>>> {
        attributes.map(|attributes| {
            let with_entries = self
                .builder
                .vec_from_iter(attributes.iter().map(|attr| self.convert_import_attribute(attr)));
            self.builder.alloc_with_clause(SPAN, oxc::WithClauseKeyword::With, with_entries)
        })
    }

    fn convert_import_attribute(&self, attr: &ImportAttribute) -> oxc::ImportAttribute<'a> {
        let key_was_quoted = self
            .source_text_for_base(&attr.key.base)
            .is_some_and(|text| matches!(text.trim_start().as_bytes().first(), Some(b'"' | b'\'')));
        let key = if key_was_quoted || !is_identifier_name(&attr.key.name) {
            self.builder.import_attribute_key_string_literal(SPAN, self.atom(&attr.key.name), None)
        } else {
            self.builder.import_attribute_key_identifier(SPAN, self.atom(&attr.key.name))
        };
        let value =
            self.builder.string_literal(SPAN, self.atom(&attr.value.value.to_string_lossy()), None);
        self.builder.import_attribute(SPAN, key, value)
    }

    fn convert_import_specifier(
        &self,
        spec: &crate::react_compiler_ast::declarations::ImportSpecifier,
    ) -> oxc::ImportDeclarationSpecifier<'a> {
        match spec {
            crate::react_compiler_ast::declarations::ImportSpecifier::ImportSpecifier(s) => {
                let local = self.builder.binding_identifier(SPAN, self.atom(&s.local.name));
                let imported = self.convert_module_export_name(&s.imported);
                let import_kind = match s.import_kind.as_ref() {
                    Some(ImportKind::Type) => oxc::ImportOrExportKind::Type,
                    _ => oxc::ImportOrExportKind::Value,
                };
                let is = self.builder.import_specifier(SPAN, imported, local, import_kind);
                oxc::ImportDeclarationSpecifier::ImportSpecifier(self.builder.alloc(is))
            }
            crate::react_compiler_ast::declarations::ImportSpecifier::ImportDefaultSpecifier(s) => {
                let local = self.builder.binding_identifier(SPAN, self.atom(&s.local.name));
                let ids = self.builder.import_default_specifier(SPAN, local);
                oxc::ImportDeclarationSpecifier::ImportDefaultSpecifier(self.builder.alloc(ids))
            }
            crate::react_compiler_ast::declarations::ImportSpecifier::ImportNamespaceSpecifier(
                s,
            ) => {
                let local = self.builder.binding_identifier(SPAN, self.atom(&s.local.name));
                let ins = self.builder.import_namespace_specifier(SPAN, local);
                oxc::ImportDeclarationSpecifier::ImportNamespaceSpecifier(self.builder.alloc(ins))
            }
        }
    }

    fn convert_module_export_name(
        &self,
        name: &crate::react_compiler_ast::declarations::ModuleExportName,
    ) -> oxc::ModuleExportName<'a> {
        match name {
            crate::react_compiler_ast::declarations::ModuleExportName::Identifier(id) => {
                oxc::ModuleExportName::IdentifierName(
                    self.builder.identifier_name(SPAN, self.atom(&id.name)),
                )
            }
            crate::react_compiler_ast::declarations::ModuleExportName::StringLiteral(s) => {
                oxc::ModuleExportName::StringLiteral(self.builder.string_literal(
                    SPAN,
                    self.atom(&s.value.to_string_lossy()),
                    None,
                ))
            }
        }
    }

    /// Like [`Self::convert_module_export_name`], but builds an identifier `local`
    /// of a local export specifier as an `IdentifierReference` (not a bare
    /// `IdentifierName`) so semantic analysis links it to the exported binding. A
    /// string-literal local is only valid with a `source`, so it falls back to the
    /// plain name.
    fn convert_module_export_name_local_ref(
        &self,
        name: &crate::react_compiler_ast::declarations::ModuleExportName,
    ) -> oxc::ModuleExportName<'a> {
        match name {
            crate::react_compiler_ast::declarations::ModuleExportName::Identifier(id) => {
                oxc::ModuleExportName::IdentifierReference(
                    self.builder.identifier_reference(SPAN, self.atom(&id.name)),
                )
            }
            crate::react_compiler_ast::declarations::ModuleExportName::StringLiteral(_) => {
                self.convert_module_export_name(name)
            }
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
        let specifiers = self.builder.vec_from_iter(
            decl.specifiers.iter().map(|s| self.convert_export_specifier(s, local_is_reference)),
        );
        let source = decl.source.as_ref().map(|s| {
            self.builder.string_literal(SPAN, self.atom(&s.value.to_string_lossy()), None)
        });
        let export_kind = match decl.export_kind.as_ref() {
            Some(ExportKind::Type) => oxc::ImportOrExportKind::Type,
            _ => oxc::ImportOrExportKind::Value,
        };
        let with_clause =
            self.convert_with_clause(decl.attributes.as_deref().or(decl.assertions.as_deref()));
        self.builder.export_named_declaration(
            SPAN,
            declaration,
            specifiers,
            source,
            export_kind,
            with_clause,
        )
    }

    fn convert_declaration(&self, decl: &Declaration) -> oxc::Declaration<'a> {
        match decl {
            Declaration::FunctionDeclaration(f) => {
                let func = self.convert_function_decl(f, oxc::FunctionType::FunctionDeclaration);
                oxc::Declaration::FunctionDeclaration(self.builder.alloc(func))
            }
            Declaration::VariableDeclaration(v) => {
                let d = self.convert_variable_declaration(v);
                oxc::Declaration::VariableDeclaration(self.builder.alloc(d))
            }
            Declaration::ClassDeclaration(c) => {
                let class = self.convert_class_declaration(c);
                oxc::Declaration::ClassDeclaration(self.builder.alloc(class))
            }
            _ => {
                let d = self.builder.variable_declaration(
                    SPAN,
                    oxc::VariableDeclarationKind::Const,
                    self.builder.vec(),
                    true,
                );
                oxc::Declaration::VariableDeclaration(self.builder.alloc(d))
            }
        }
    }

    fn convert_export_specifier(
        &self,
        spec: &crate::react_compiler_ast::declarations::ExportSpecifier,
        local_is_reference: bool,
    ) -> oxc::ExportSpecifier<'a> {
        match spec {
            crate::react_compiler_ast::declarations::ExportSpecifier::ExportSpecifier(s) => {
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
                self.builder.export_specifier(SPAN, local, exported, export_kind)
            }
            crate::react_compiler_ast::declarations::ExportSpecifier::ExportDefaultSpecifier(s) => {
                let name = oxc::ModuleExportName::IdentifierName(
                    self.builder.identifier_name(SPAN, self.atom(&s.exported.name)),
                );
                let default_name = oxc::ModuleExportName::IdentifierName(
                    self.builder.identifier_name(SPAN, self.atom("default")),
                );
                self.builder.export_specifier(
                    SPAN,
                    name,
                    default_name,
                    oxc::ImportOrExportKind::Value,
                )
            }
            crate::react_compiler_ast::declarations::ExportSpecifier::ExportNamespaceSpecifier(
                s,
            ) => {
                let exported = self.convert_module_export_name(&s.exported);
                let star = oxc::ModuleExportName::IdentifierName(
                    self.builder.identifier_name(SPAN, self.atom("*")),
                );
                self.builder.export_specifier(SPAN, star, exported, oxc::ImportOrExportKind::Value)
            }
        }
    }

    fn convert_export_default_declaration(
        &self,
        decl: &ExportDefaultDeclaration,
    ) -> oxc::ExportDefaultDeclaration<'a> {
        let declaration = self.convert_export_default_decl(&decl.declaration);
        self.builder.export_default_declaration(SPAN, declaration)
    }

    fn convert_export_default_decl(
        &self,
        decl: &ExportDefaultDecl,
    ) -> oxc::ExportDefaultDeclarationKind<'a> {
        match decl {
            ExportDefaultDecl::FunctionDeclaration(f) => {
                let func = self.convert_function_decl(f, oxc::FunctionType::FunctionDeclaration);
                oxc::ExportDefaultDeclarationKind::FunctionDeclaration(self.builder.alloc(func))
            }
            ExportDefaultDecl::ClassDeclaration(c) => {
                let class = self.convert_class_declaration(c);
                oxc::ExportDefaultDeclarationKind::ClassDeclaration(self.builder.alloc(class))
            }
            ExportDefaultDecl::EnumDeclaration(_) => {
                // Flow enum declarations cannot be represented in OXC AST;
                // emit a null placeholder to preserve the export shape.
                oxc::ExportDefaultDeclarationKind::from(
                    self.builder.expression_null_literal(oxc::Span::default()),
                )
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
        let source = self.builder.string_literal(
            SPAN,
            self.atom(&decl.source.value.to_string_lossy()),
            None,
        );
        let export_kind = match decl.export_kind.as_ref() {
            Some(ExportKind::Type) => oxc::ImportOrExportKind::Type,
            _ => oxc::ImportOrExportKind::Value,
        };
        let with_clause =
            self.convert_with_clause(decl.attributes.as_deref().or(decl.assertions.as_deref()));
        self.builder.export_all_declaration(
            SPAN,
            None, // exported
            source,
            with_clause,
            export_kind,
        )
    }

    // ===== Operators =====

    fn convert_binary_operator(&self, op: &BinaryOperator) -> oxc_syntax::operator::BinaryOperator {
        use oxc_syntax::operator::BinaryOperator as OxcBinOp;
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

    fn convert_logical_operator(
        &self,
        op: &LogicalOperator,
    ) -> oxc_syntax::operator::LogicalOperator {
        use oxc_syntax::operator::LogicalOperator as OxcLogOp;
        match op {
            LogicalOperator::Or => OxcLogOp::Or,
            LogicalOperator::And => OxcLogOp::And,
            LogicalOperator::NullishCoalescing => OxcLogOp::Coalesce,
        }
    }

    fn convert_unary_operator(&self, op: &UnaryOperator) -> oxc_syntax::operator::UnaryOperator {
        use oxc_syntax::operator::UnaryOperator as OxcUnOp;
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

    fn convert_update_operator(&self, op: &UpdateOperator) -> oxc_syntax::operator::UpdateOperator {
        use oxc_syntax::operator::UpdateOperator as OxcUpOp;
        match op {
            UpdateOperator::Increment => OxcUpOp::Increment,
            UpdateOperator::Decrement => OxcUpOp::Decrement,
        }
    }

    fn convert_assignment_operator(
        &self,
        op: &AssignmentOperator,
    ) -> oxc_syntax::operator::AssignmentOperator {
        use oxc_syntax::operator::AssignmentOperator as OxcAssOp;
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
