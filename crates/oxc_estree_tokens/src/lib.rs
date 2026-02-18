use rustc_hash::FxHashSet;
use serde::Serialize;

use oxc_allocator::{Allocator, Vec as ArenaVec};
use oxc_ast::ast::{
    BindingIdentifier, IdentifierName, IdentifierReference, JSXAttribute, JSXAttributeValue,
    JSXElementName, JSXExpressionContainer, JSXIdentifier, JSXMemberExpressionObject,
    JSXNamespacedName, JSXSpreadAttribute, JSXSpreadChild, LabelIdentifier, MemberExpression,
    MetaProperty, ObjectPropertyKind, Program, PropertyKey, TSImportType, TSThisParameter,
    TSTypeName, TSTypeParameterDeclaration, TSTypeQuery, TSTypeQueryExprName, WithClause,
    WithClauseKeyword,
};
use oxc_ast_visit::{Visit, utf8_to_utf16::Utf8ToUtf16, walk};
use oxc_parser::{Kind, Token};
use oxc_span::Span;

#[derive(Serialize)]
pub struct EstreeRegExpToken<'a> {
    pub flags: &'a str,
    pub pattern: &'a str,
}

#[derive(Serialize)]
pub struct EstreeToken<'a> {
    #[serde(rename = "type")]
    pub token_type: &'static str,
    pub value: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regex: Option<EstreeRegExpToken<'a>>,
    pub start: u32,
    pub end: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct EstreeTokenOptions {
    pub exclude_legacy_keyword_identifiers: bool,
    pub decode_identifier_escapes: bool,
}

impl EstreeTokenOptions {
    pub const fn test262() -> Self {
        Self { exclude_legacy_keyword_identifiers: true, decode_identifier_escapes: true }
    }

    pub const fn typescript() -> Self {
        Self { exclude_legacy_keyword_identifiers: false, decode_identifier_escapes: false }
    }

    pub const fn linter() -> Self {
        Self { exclude_legacy_keyword_identifiers: true, decode_identifier_escapes: false }
    }
}

#[derive(Default)]
pub struct EstreeTokenContext {
    ast_identifier_spans: FxHashSet<Span>,
    force_keyword_spans: FxHashSet<Span>,
    jsx_identifier_spans: FxHashSet<Span>,
    non_jsx_identifier_spans: FxHashSet<Span>,
    jsx_text_spans: FxHashSet<Span>,
    ts_type_parameter_starts: FxHashSet<u32>,
    jsx_expression_depth: usize,
    jsx_member_expression_depth: usize,
    jsx_computed_member_depth: usize,
}

impl<'a> Visit<'a> for EstreeTokenContext {
    fn visit_ts_type_query(&mut self, type_query: &TSTypeQuery<'a>) {
        fn collect_type_query_this(spans: &mut FxHashSet<Span>, type_name: &TSTypeName<'_>) {
            match type_name {
                TSTypeName::ThisExpression(this_expression) => {
                    spans.insert(this_expression.span);
                }
                TSTypeName::QualifiedName(qualified_name) => {
                    collect_type_query_this(spans, &qualified_name.left);
                }
                TSTypeName::IdentifierReference(_) => {}
            }
        }

        match &type_query.expr_name {
            TSTypeQueryExprName::ThisExpression(this_expression) => {
                self.ast_identifier_spans.insert(this_expression.span);
            }
            TSTypeQueryExprName::QualifiedName(qualified_name) => {
                collect_type_query_this(&mut self.ast_identifier_spans, &qualified_name.left);
            }
            TSTypeQueryExprName::IdentifierReference(_) | TSTypeQueryExprName::TSImportType(_) => {}
        }

        walk::walk_ts_type_query(self, type_query);
    }

    fn visit_ts_import_type(&mut self, import_type: &TSImportType<'a>) {
        if let Some(options) = &import_type.options {
            for property in &options.properties {
                if let ObjectPropertyKind::ObjectProperty(property) = property
                    && let PropertyKey::StaticIdentifier(identifier) = &property.key
                    && identifier.name == "with"
                {
                    self.force_keyword_spans.insert(identifier.span);
                }
            }
        }
        walk::walk_ts_import_type(self, import_type);
    }

    fn visit_ts_type_parameter_declaration(
        &mut self,
        declaration: &TSTypeParameterDeclaration<'a>,
    ) {
        self.ts_type_parameter_starts.insert(declaration.span.start);
        walk::walk_ts_type_parameter_declaration(self, declaration);
    }

    fn visit_identifier_name(&mut self, identifier: &IdentifierName<'a>) {
        self.ast_identifier_spans.insert(identifier.span);
        if self.jsx_expression_depth > 0
            && self.jsx_member_expression_depth > 0
            && self.jsx_computed_member_depth == 0
        {
            self.jsx_identifier_spans.insert(identifier.span);
        }
    }

    fn visit_identifier_reference(&mut self, identifier: &IdentifierReference<'a>) {
        self.ast_identifier_spans.insert(identifier.span);
        if self.jsx_expression_depth > 0
            && self.jsx_member_expression_depth > 0
            && self.jsx_computed_member_depth == 0
        {
            self.jsx_identifier_spans.insert(identifier.span);
        }
    }

    fn visit_binding_identifier(&mut self, identifier: &BindingIdentifier<'a>) {
        self.ast_identifier_spans.insert(identifier.span);
    }

    fn visit_label_identifier(&mut self, identifier: &LabelIdentifier<'a>) {
        self.ast_identifier_spans.insert(identifier.span);
    }

    fn visit_ts_this_parameter(&mut self, parameter: &TSThisParameter<'a>) {
        self.ast_identifier_spans.insert(parameter.this_span);
        walk::walk_ts_this_parameter(self, parameter);
    }

    fn visit_meta_property(&mut self, meta_property: &MetaProperty<'a>) {
        self.force_keyword_spans.insert(meta_property.meta.span);
        walk::walk_meta_property(self, meta_property);
    }

    fn visit_with_clause(&mut self, with_clause: &WithClause<'a>) {
        if matches!(with_clause.keyword, WithClauseKeyword::With) {
            let span = Span::new(with_clause.span.start, with_clause.span.start + 4);
            self.force_keyword_spans.insert(span);
        }
        walk::walk_with_clause(self, with_clause);
    }

    fn visit_jsx_identifier(&mut self, identifier: &JSXIdentifier<'a>) {
        self.jsx_identifier_spans.insert(identifier.span);
    }

    fn visit_jsx_element_name(&mut self, name: &JSXElementName<'a>) {
        if let JSXElementName::IdentifierReference(identifier) = name {
            self.jsx_identifier_spans.insert(identifier.span);
        }
        walk::walk_jsx_element_name(self, name);
    }

    fn visit_jsx_member_expression_object(&mut self, object: &JSXMemberExpressionObject<'a>) {
        if let JSXMemberExpressionObject::IdentifierReference(identifier) = object {
            self.jsx_identifier_spans.insert(identifier.span);
        }
        walk::walk_jsx_member_expression_object(self, object);
    }

    fn visit_jsx_namespaced_name(&mut self, name: &JSXNamespacedName<'a>) {
        self.non_jsx_identifier_spans.insert(name.namespace.span);
        self.non_jsx_identifier_spans.insert(name.name.span);
        walk::walk_jsx_namespaced_name(self, name);
    }

    fn visit_jsx_expression_container(&mut self, container: &JSXExpressionContainer<'a>) {
        self.jsx_expression_depth += 1;
        walk::walk_jsx_expression_container(self, container);
        self.jsx_expression_depth -= 1;
    }

    fn visit_member_expression(&mut self, expression: &MemberExpression<'a>) {
        if self.jsx_expression_depth > 0 {
            self.jsx_member_expression_depth += 1;
            if matches!(expression, MemberExpression::ComputedMemberExpression(_)) {
                self.jsx_computed_member_depth += 1;
            }
            walk::walk_member_expression(self, expression);
            if matches!(expression, MemberExpression::ComputedMemberExpression(_)) {
                self.jsx_computed_member_depth -= 1;
            }
            self.jsx_member_expression_depth -= 1;
        } else {
            walk::walk_member_expression(self, expression);
        }
    }

    fn visit_jsx_spread_attribute(&mut self, attribute: &JSXSpreadAttribute<'a>) {
        self.jsx_expression_depth += 1;
        walk::walk_jsx_spread_attribute(self, attribute);
        self.jsx_expression_depth -= 1;
    }

    fn visit_jsx_spread_child(&mut self, spread_child: &JSXSpreadChild<'a>) {
        self.jsx_expression_depth += 1;
        walk::walk_jsx_spread_child(self, spread_child);
        self.jsx_expression_depth -= 1;
    }

    fn visit_jsx_attribute(&mut self, attribute: &JSXAttribute<'a>) {
        if let Some(JSXAttributeValue::StringLiteral(string_literal)) = &attribute.value {
            self.jsx_text_spans.insert(string_literal.span);
        }
        walk::walk_jsx_attribute(self, attribute);
    }
}

pub fn collect_token_context(program: &Program<'_>) -> EstreeTokenContext {
    let mut context = EstreeTokenContext::default();
    context.visit_program(program);
    context
}

fn regex_parts(raw: &str) -> Option<(&str, &str)> {
    let bytes = raw.as_bytes();
    if bytes.first() != Some(&b'/') {
        return None;
    }

    let mut escaped = false;
    let mut in_character_class = false;
    for index in 1..bytes.len() {
        let byte = bytes[index];
        if escaped {
            escaped = false;
            continue;
        }
        match byte {
            b'\\' => escaped = true,
            b'[' if !in_character_class => in_character_class = true,
            b']' if in_character_class => in_character_class = false,
            b'/' if !in_character_class => return Some((&raw[1..index], &raw[index + 1..])),
            _ => {}
        }
    }

    None
}

fn decode_js_unicode_escapes<'a>(allocator: &'a Allocator, raw: &'a str) -> &'a str {
    if !raw.contains("\\u") {
        return raw;
    }

    let bytes = raw.as_bytes();
    let mut output = String::with_capacity(raw.len());
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'\\' && i + 1 < bytes.len() && bytes[i + 1] == b'u' {
            // `\u{...}`
            if i + 2 < bytes.len() && bytes[i + 2] == b'{' {
                let mut j = i + 3;
                while j < bytes.len() && bytes[j] != b'}' {
                    j += 1;
                }
                if j < bytes.len()
                    && j > i + 3
                    && let Ok(codepoint) = u32::from_str_radix(&raw[i + 3..j], 16)
                    && let Some(ch) = char::from_u32(codepoint)
                {
                    output.push(ch);
                    i = j + 1;
                    continue;
                }
            // `\uXXXX`
            } else if i + 6 <= bytes.len()
                && let Ok(codepoint) = u32::from_str_radix(&raw[i + 2..i + 6], 16)
                && let Some(ch) = char::from_u32(codepoint)
            {
                output.push(ch);
                i += 6;
                continue;
            }
        }

        let ch = raw[i..].chars().next().unwrap();
        output.push(ch);
        i += ch.len_utf8();
    }

    allocator.alloc_str(&output)
}

fn token_type(
    kind: Kind,
    is_ast_identifier: bool,
    is_forced_keyword: bool,
    is_jsx_identifier: bool,
    is_jsx_text: bool,
) -> &'static str {
    if is_jsx_identifier {
        return "JSXIdentifier";
    }
    if is_jsx_text {
        return "JSXText";
    }
    if is_forced_keyword {
        return "Keyword";
    }
    if is_ast_identifier {
        return "Identifier";
    }

    match kind {
        Kind::Ident | Kind::Await => "Identifier",
        Kind::PrivateIdentifier => "PrivateIdentifier",
        Kind::JSXText => "JSXText",
        Kind::Str => "String",
        Kind::RegExp => "RegularExpression",
        Kind::NoSubstitutionTemplate
        | Kind::TemplateHead
        | Kind::TemplateMiddle
        | Kind::TemplateTail => "Template",
        Kind::True | Kind::False => "Boolean",
        Kind::Null => "Null",
        _ if kind.is_number() => "Numeric",
        _ if kind.is_contextual_keyword() => "Identifier",
        _ if kind.is_any_keyword() => "Keyword",
        _ => "Punctuator",
    }
}

pub fn to_estree_tokens<'a>(
    allocator: &'a Allocator,
    source_text: &'a str,
    tokens: &[Token],
    context: &EstreeTokenContext,
    options: EstreeTokenOptions,
) -> ArenaVec<'a, EstreeToken<'a>> {
    let utf8_to_utf16 = Utf8ToUtf16::new(source_text);
    let mut converter = utf8_to_utf16.converter();

    let mut estree_tokens = ArenaVec::with_capacity_in(tokens.len(), allocator);
    for token in tokens {
        let kind = token.kind();
        let source_value = &source_text[token.start() as usize..token.end() as usize];

        let mut start = token.start();
        let mut end = token.end();
        if let Some(converter) = converter.as_mut() {
            converter.convert_offset(&mut start);
            converter.convert_offset(&mut end);
        }
        let span_utf16 = Span::new(start, end);

        // TS estree token streams may already contain the second `<` as a
        // standalone token here; skip the overlapping `<<` token.
        if kind == Kind::ShiftLeft
            && context.ts_type_parameter_starts.contains(&(span_utf16.start + 1))
        {
            continue;
        }

        let is_ast_identifier = context.ast_identifier_spans.contains(&span_utf16);
        let is_ast_identifier = if options.exclude_legacy_keyword_identifiers {
            is_ast_identifier && !matches!(kind, Kind::Yield | Kind::Let | Kind::Static)
        } else {
            is_ast_identifier
        };
        let token_type = token_type(
            kind,
            is_ast_identifier,
            context.force_keyword_spans.contains(&span_utf16),
            context.jsx_identifier_spans.contains(&span_utf16)
                && !context.non_jsx_identifier_spans.contains(&span_utf16),
            context.jsx_text_spans.contains(&span_utf16),
        );

        let source_value =
            if kind == Kind::PrivateIdentifier { &source_value[1..] } else { source_value };
        let value = if options.decode_identifier_escapes
            && token.escaped()
            && (kind.is_identifier_name() || kind == Kind::PrivateIdentifier)
        {
            decode_js_unicode_escapes(allocator, source_value)
        } else {
            source_value
        };
        let regex = if kind == Kind::RegExp {
            regex_parts(source_value).map(|(pattern, flags)| EstreeRegExpToken { flags, pattern })
        } else {
            None
        };

        estree_tokens.push(EstreeToken { token_type, value, regex, start, end });
    }

    estree_tokens
}

pub fn to_estree_tokens_json(
    allocator: &Allocator,
    source_text: &str,
    tokens: &[Token],
    context: &EstreeTokenContext,
    options: EstreeTokenOptions,
) -> String {
    let estree_tokens = to_estree_tokens(allocator, source_text, tokens, context, options);
    serde_json::to_string_pretty(&estree_tokens).unwrap_or_default()
}
