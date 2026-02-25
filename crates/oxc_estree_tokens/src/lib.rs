use serde::Serialize;

use oxc_allocator::{Allocator, Vec as ArenaVec};
use oxc_ast::ast::*;
use oxc_ast_visit::{Visit, utf8_to_utf16::Utf8ToUtf16, walk};
use oxc_parser::{Kind, Token};
use oxc_span::{GetSpan, Span};

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

#[derive(Serialize)]
pub struct EstreeRegExpToken<'a> {
    pub pattern: &'a str,
    pub flags: &'a str,
}

#[derive(Debug, Clone, Copy)]
pub struct EstreeTokenOptions {
    pub exclude_legacy_keyword_identifiers: bool,
    pub decode_identifier_escapes: bool,
    pub jsx_namespace_jsx_identifiers: bool,
    pub member_expr_in_jsx_expression_jsx_identifiers: bool,
}

impl EstreeTokenOptions {
    pub const fn test262() -> Self {
        Self {
            exclude_legacy_keyword_identifiers: true,
            decode_identifier_escapes: true,
            jsx_namespace_jsx_identifiers: true,
            member_expr_in_jsx_expression_jsx_identifiers: false,
        }
    }

    pub const fn typescript() -> Self {
        Self {
            exclude_legacy_keyword_identifiers: false,
            decode_identifier_escapes: false,
            jsx_namespace_jsx_identifiers: false,
            member_expr_in_jsx_expression_jsx_identifiers: true,
        }
    }

    pub const fn linter() -> Self {
        Self {
            exclude_legacy_keyword_identifiers: true,
            decode_identifier_escapes: false,
            jsx_namespace_jsx_identifiers: true,
            member_expr_in_jsx_expression_jsx_identifiers: false,
        }
    }
}

/// Serialize tokens to JSON.
///
/// `source_text` must be the original source text, prior to BOM removal.
/// i.e. BOM must be present on start of `source_text`, if the file has a BOM.
pub fn to_estree_tokens_json(
    tokens: &[Token],
    program: &Program<'_>,
    source_text: &str,
    span_converter: &Utf8ToUtf16,
    options: EstreeTokenOptions,
    allocator: &Allocator,
) -> String {
    let estree_tokens =
        to_estree_tokens(tokens, program, source_text, span_converter, options, allocator);
    serde_json::to_string(&estree_tokens).unwrap_or_default()
}

/// Serialize tokens to pretty-printed JSON.
///
/// `source_text` must be the original source text, prior to BOM removal.
/// i.e. BOM must be present on start of `source_text`, if the file has a BOM.
pub fn to_estree_tokens_pretty_json(
    tokens: &[Token],
    program: &Program<'_>,
    source_text: &str,
    span_converter: &Utf8ToUtf16,
    options: EstreeTokenOptions,
    allocator: &Allocator,
) -> String {
    let estree_tokens =
        to_estree_tokens(tokens, program, source_text, span_converter, options, allocator);
    serde_json::to_string_pretty(&estree_tokens).unwrap_or_default()
}

/// Convert `Token`s to `EstreeToken`s.
fn to_estree_tokens<'a>(
    tokens: &[Token],
    program: &Program<'a>,
    source_text: &'a str,
    span_converter: &Utf8ToUtf16,
    options: EstreeTokenOptions,
    allocator: &'a Allocator,
) -> ArenaVec<'a, EstreeToken<'a>> {
    // Traverse AST to collect details of tokens requiring correction, depending on provided options
    let mut context = EstreeTokenContext {
        exclude_legacy_keyword_identifiers: options.exclude_legacy_keyword_identifiers,
        jsx_namespace_jsx_identifiers: options.jsx_namespace_jsx_identifiers,
        member_expr_in_jsx_expression_jsx_identifiers: options
            .member_expr_in_jsx_expression_jsx_identifiers,
        ..Default::default()
    };
    context.visit_program(program);

    // Convert tokens to `EstreeToken`s
    let mut span_converter = span_converter.converter();

    let mut estree_tokens = ArenaVec::with_capacity_in(tokens.len(), allocator);

    // If no overrides, use `u32::MAX` as span start.
    // No token can start at `u32::MAX`, since source text is capped at `u32::MAX` bytes length.
    let mut overrides = context.overrides.into_iter();
    let (mut next_override_start, mut next_override_kind) =
        overrides.next().unwrap_or((u32::MAX, TokenKindOverride::Identifier));

    for token in tokens {
        let kind = token.kind();
        let source_value = &source_text[token.start() as usize..token.end() as usize];

        let mut start = token.start();
        let mut end = token.end();
        if let Some(span_converter) = span_converter.as_mut() {
            span_converter.convert_offset(&mut start);
            span_converter.convert_offset(&mut end);
        }

        let token_type = if next_override_start == start {
            let token_type = match next_override_kind {
                TokenKindOverride::Identifier => "Identifier",
                TokenKindOverride::JSXIdentifier => "JSXIdentifier",
                TokenKindOverride::JSXText => "JSXText",
            };

            // Get next override
            (next_override_start, next_override_kind) =
                overrides.next().unwrap_or((u32::MAX, TokenKindOverride::Identifier));

            token_type
        } else {
            get_token_type(kind)
        };

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
            regex_parts(source_value).map(|(pattern, flags)| EstreeRegExpToken { pattern, flags })
        } else {
            None
        };

        estree_tokens.push(EstreeToken { token_type, value, regex, start, end });
    }

    estree_tokens
}

/// Override for a token's type, determined by AST context.
/// Each span is assigned at most one override during the AST walk.
#[derive(Debug, Clone, Copy)]
enum TokenKindOverride {
    /// Token is an identifier in the AST (overrides keyword `Kind`s like `yield`, `let`, etc).
    Identifier,
    /// Token is a JSX identifier.
    JSXIdentifier,
    /// Token is JSX text (a string literal in a JSX attribute).
    JSXText,
}

#[derive(Default)]
pub struct EstreeTokenContext {
    // Options
    exclude_legacy_keyword_identifiers: bool,
    jsx_namespace_jsx_identifiers: bool,
    member_expr_in_jsx_expression_jsx_identifiers: bool,
    /// Token kind overrides, stored in source order.
    /// Each entry is `(token_start_position, override)`.
    overrides: Vec<(u32, TokenKindOverride)>,
    // State
    jsx_expression_depth: usize,
    jsx_member_expression_depth: usize,
    jsx_computed_member_depth: usize,
}

impl EstreeTokenContext {
    fn set_override(&mut self, span: Span, token_override: TokenKindOverride) {
        debug_assert!(
            self.overrides.last().is_none_or(|&(prev_start, _)| span.start > prev_start),
            "Out of order: {span:?} ({token_override:?}) not after previous start {}",
            self.overrides.last().unwrap().0,
        );

        self.overrides.push((span.start, token_override));
    }

    fn set_identifier_override_unless_excluded(&mut self, span: Span, name: &str) {
        if !self.exclude_legacy_keyword_identifiers || !matches!(name, "yield" | "let" | "static") {
            self.set_override(span, TokenKindOverride::Identifier);
        }
    }
}

impl<'a> Visit<'a> for EstreeTokenContext {
    fn visit_ts_type_query(&mut self, type_query: &TSTypeQuery<'a>) {
        fn collect_type_query_this(ctx: &mut EstreeTokenContext, type_name: &TSTypeName<'_>) {
            match type_name {
                TSTypeName::ThisExpression(this_expression) => {
                    ctx.set_override(this_expression.span, TokenKindOverride::Identifier);
                }
                TSTypeName::QualifiedName(qualified_name) => {
                    collect_type_query_this(ctx, &qualified_name.left);
                }
                TSTypeName::IdentifierReference(_) => {}
            }
        }

        match &type_query.expr_name {
            TSTypeQueryExprName::ThisExpression(this_expression) => {
                self.set_override(this_expression.span, TokenKindOverride::Identifier);
            }
            TSTypeQueryExprName::QualifiedName(qualified_name) => {
                collect_type_query_this(self, &qualified_name.left);
            }
            TSTypeQueryExprName::IdentifierReference(_) | TSTypeQueryExprName::TSImportType(_) => {}
        }

        walk::walk_ts_type_query(self, type_query);
    }

    fn visit_ts_import_type(&mut self, import_type: &TSImportType<'a>) {
        // Manual walk.
        // * `source` is a `StringLiteral` — can't contain identifiers, so we skip visiting it.
        // * `options` is an `ObjectExpression`. Manually walk each property, but don't visit the key if it's `with`,
        //   as it needs to remain "Keyword" token, not get converted to "Identifier".
        // * `qualifier` and `type_arguments` are visited as usual.
        if let Some(options) = &import_type.options {
            for property in &options.properties {
                match property {
                    ObjectPropertyKind::ObjectProperty(property) => {
                        let is_with_key = matches!(
                            &property.key,
                            PropertyKey::StaticIdentifier(id) if id.name == "with"
                        );
                        if !is_with_key {
                            self.visit_property_key(&property.key);
                        }
                        self.visit_expression(&property.value);
                    }
                    ObjectPropertyKind::SpreadProperty(spread) => {
                        self.visit_spread_element(spread);
                    }
                }
            }
        }
        if let Some(qualifier) = &import_type.qualifier {
            self.visit_ts_import_type_qualifier(qualifier);
        }
        if let Some(type_arguments) = &import_type.type_arguments {
            self.visit_ts_type_parameter_instantiation(type_arguments);
        }
    }

    fn visit_identifier_name(&mut self, identifier: &IdentifierName<'a>) {
        // `JSXIdentifier` takes priority over `Identifier` in `token_type`,
        // so `TokenKindOverride::Identifier` is redundant when `TokenKindOverride::JSXIdentifier` is set.
        if self.member_expr_in_jsx_expression_jsx_identifiers
            && self.jsx_expression_depth > 0
            && self.jsx_member_expression_depth > 0
            && self.jsx_computed_member_depth == 0
        {
            self.set_override(identifier.span, TokenKindOverride::JSXIdentifier);
        } else {
            self.set_identifier_override_unless_excluded(identifier.span, &identifier.name);
        }
    }

    fn visit_identifier_reference(&mut self, identifier: &IdentifierReference<'a>) {
        if self.member_expr_in_jsx_expression_jsx_identifiers
            && self.jsx_expression_depth > 0
            && self.jsx_member_expression_depth > 0
            && self.jsx_computed_member_depth == 0
        {
            self.set_override(identifier.span, TokenKindOverride::JSXIdentifier);
        } else {
            self.set_identifier_override_unless_excluded(identifier.span, &identifier.name);
        }
    }

    fn visit_binding_identifier(&mut self, identifier: &BindingIdentifier<'a>) {
        self.set_identifier_override_unless_excluded(identifier.span, &identifier.name);
    }

    fn visit_label_identifier(&mut self, identifier: &LabelIdentifier<'a>) {
        self.set_identifier_override_unless_excluded(identifier.span, &identifier.name);
    }

    fn visit_ts_this_parameter(&mut self, parameter: &TSThisParameter<'a>) {
        self.set_override(parameter.this_span, TokenKindOverride::Identifier);
        walk::walk_ts_this_parameter(self, parameter);
    }

    fn visit_meta_property(&mut self, _meta_property: &MetaProperty<'a>) {
        // Don't walk.
        // * `meta` (either `import` or `new`) has a "Keyword" token already, which is correct.
        // * `property` (either `meta` or `target`) has an "Identifier" token, which is correct.
    }

    fn visit_object_property(&mut self, property: &ObjectProperty<'a>) {
        // For shorthand `{ x }`, key and value share the same span.
        // Skip the key to avoid double-flagging.
        if !property.shorthand {
            self.visit_property_key(&property.key);
        }
        self.visit_expression(&property.value);
    }

    fn visit_binding_property(&mut self, property: &BindingProperty<'a>) {
        if !property.shorthand {
            self.visit_property_key(&property.key);
        }
        self.visit_binding_pattern(&property.value);
    }

    fn visit_import_specifier(&mut self, specifier: &ImportSpecifier<'a>) {
        // For `import { x }`, `imported` and `local` share the same span.
        // Only visit `imported` when it differs from `local`.
        if specifier.imported.span() != specifier.local.span {
            self.visit_module_export_name(&specifier.imported);
        }
        self.visit_binding_identifier(&specifier.local);
    }

    fn visit_export_specifier(&mut self, specifier: &ExportSpecifier<'a>) {
        // For `export { x }`, `local` and `exported` share the same span.
        // Only visit `exported` when it differs from `local`.
        self.visit_module_export_name(&specifier.local);
        if specifier.exported.span() != specifier.local.span() {
            self.visit_module_export_name(&specifier.exported);
        }
    }

    fn visit_jsx_identifier(&mut self, identifier: &JSXIdentifier<'a>) {
        self.set_override(identifier.span, TokenKindOverride::JSXIdentifier);
    }

    fn visit_jsx_element_name(&mut self, name: &JSXElementName<'a>) {
        if let JSXElementName::IdentifierReference(identifier) = name {
            self.set_override(identifier.span, TokenKindOverride::JSXIdentifier);
        } else {
            walk::walk_jsx_element_name(self, name);
        }
    }

    fn visit_jsx_member_expression_object(&mut self, object: &JSXMemberExpressionObject<'a>) {
        if let JSXMemberExpressionObject::IdentifierReference(identifier) = object {
            self.set_override(identifier.span, TokenKindOverride::JSXIdentifier);
        } else {
            walk::walk_jsx_member_expression_object(self, object);
        }
    }

    fn visit_jsx_namespaced_name(&mut self, name: &JSXNamespacedName<'a>) {
        if self.jsx_namespace_jsx_identifiers {
            self.set_override(name.namespace.span, TokenKindOverride::JSXIdentifier);
            self.set_override(name.name.span, TokenKindOverride::JSXIdentifier);
        }
        // When `!jsx_namespace_jsx_identifiers`, these spans are not marked,
        // so they retain their default token kind
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
        // Manual walk.
        // * `name`: Visit normally.
        // * `value`: Set `JSXText` override if is a `StringLiteral`.
        self.visit_jsx_attribute_name(&attribute.name);
        match &attribute.value {
            Some(JSXAttributeValue::StringLiteral(string_literal)) => {
                self.set_override(string_literal.span, TokenKindOverride::JSXText);
            }
            Some(value) => self.visit_jsx_attribute_value(value),
            None => {}
        }
    }
}

fn get_token_type(kind: Kind) -> &'static str {
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
