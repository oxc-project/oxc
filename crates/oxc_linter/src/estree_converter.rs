//! ESTree to oxc AST conversion bridge.
//!
//! This module provides the conversion from ESTree AST (from custom parsers)
//! to oxc AST. It uses utilities from `oxc_estree::deserialize` but implements
//! the actual AST construction here since it has access to `oxc_allocator` and `oxc_ast`.

use oxc_allocator::{Allocator, FromIn, Vec};
use oxc_ast::{ast::Program, AstBuilder};
use oxc_estree::deserialize::{
    convert_identifier, convert_literal, get_boolean_value,
    get_literal_span, get_numeric_value, get_string_value, ConversionContext, ConversionError,
    ConversionResult, EstreeConverter, EstreeIdentifier, EstreeLiteral, IdentifierKind,
    LiteralKind,
};
use oxc_span::{Atom, Span};
use serde_json::Value;

/// Convert ESTree AST (from raw transfer buffer) to oxc AST Program.
///
/// This is the main entry point for converting an ESTree AST from a custom parser
/// to an oxc AST program. The ESTree AST is read from a raw transfer buffer.
///
/// Buffer format (JSON-based for MVP):
/// - [0-4]: Length of JSON string (u32, little-endian)
/// - [4-N]: JSON string (UTF-8 encoded)
/// - [N-N+4]: Offset where JSON starts (for consistency)
///
/// # Arguments
///
/// * `buffer` - Raw transfer buffer containing ESTree AST
/// * `estree_offset` - Offset where ESTree data starts in the buffer
/// * `source_text` - Original source code (needed for span conversion)
/// * `allocator` - Arena allocator for AST node allocation
///
/// # Returns
///
/// Returns a `Program` allocated in the arena, or an error if conversion fails.
pub fn convert_estree_to_oxc_program<'a>(
    buffer: &[u8],
    estree_offset: u32,
    source_text: &'a str,
    allocator: &'a Allocator,
) -> ConversionResult<Program<'a>> {
    // Read JSON length from buffer start (u32, little-endian)
    if buffer.len() < 4 {
        return Err(ConversionError::JsonParseError {
            message: "Buffer too small to read JSON length".to_string(),
        });
    }
    
    let json_length = u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]) as usize;
    
    // Validate offset
    let offset = estree_offset as usize;
    if offset + json_length > buffer.len() {
        return Err(ConversionError::JsonParseError {
            message: format!(
                "JSON extends beyond buffer: offset={}, length={}, buffer_len={}",
                offset,
                json_length,
                buffer.len()
            ),
        });
    }
    
    // Read JSON string from buffer
    let json_bytes = &buffer[offset..offset + json_length];
    let json_string = std::str::from_utf8(json_bytes).map_err(|e| {
        ConversionError::JsonParseError {
            message: format!("Invalid UTF-8 in JSON: {}", e),
        }
    })?;
    
    // Use the JSON converter
    convert_estree_json_to_oxc_program(json_string, source_text, allocator)
}

/// Convert ESTree JSON (fallback) to oxc AST Program.
///
/// This is a fallback for platforms without raw transfer support.
/// Uses JSON deserialization and conversion.
pub fn convert_estree_json_to_oxc_program<'a>(
    estree_json: &str,
    source_text: &'a str,
    allocator: &'a Allocator,
) -> ConversionResult<Program<'a>> {
    // Parse JSON
    let estree: Value = serde_json::from_str(estree_json)
        .map_err(|e| ConversionError::JsonParseError {
            message: format!("Failed to parse ESTree JSON: {}", e),
        })?;

    // Validate and convert
    let converter = EstreeConverter::new(source_text);
    converter.validate_program(&estree)?;

    // Convert Program node
    let mut converter_impl = EstreeConverterImpl::new(source_text, allocator);
    converter_impl.convert_program(&estree)
}

/// Internal converter implementation that handles the actual AST construction.
struct EstreeConverterImpl<'a> {
    source_text: &'a str,
    builder: AstBuilder<'a>,
    context: ConversionContext,
}

impl<'a> EstreeConverterImpl<'a> {
    fn new(source_text: &'a str, allocator: &'a Allocator) -> Self {
        Self {
            source_text,
            builder: AstBuilder::new(allocator),
            context: ConversionContext::new(),
        }
    }

    /// Convert an ESTree Program node to oxc Program.
    fn convert_program(&mut self, estree: &Value) -> ConversionResult<Program<'a>> {
        use oxc_ast::ast::Statement;
        use oxc_estree::deserialize::{EstreeNode, EstreeNodeType};
        use oxc_span::SourceType;

        let node_type = <Value as EstreeNode>::get_type(estree)
            .ok_or_else(|| ConversionError::MissingField {
                field: "type".to_string(),
                node_type: "unknown".to_string(),
                span: (0, 0),
            })?;

        if !matches!(node_type, EstreeNodeType::Program) {
            return Err(ConversionError::UnsupportedNodeType {
                node_type: format!("{:?}", node_type),
                span: (0, 0),
            });
        }

        // Get body array
        let body_value = estree.get("body").ok_or_else(|| ConversionError::MissingField {
            field: "body".to_string(),
            node_type: "Program".to_string(),
            span: (0, 0),
        })?;

        let body_array = body_value.as_array().ok_or_else(|| ConversionError::InvalidFieldType {
            field: "body".to_string(),
            expected: "array".to_string(),
            got: format!("{:?}", body_value),
            span: (0, 0),
        })?;

        // Convert each statement
        let mut statements = Vec::new_in(self.builder.allocator);
        for stmt_value in body_array {
            // Push context for this statement
            self.context = self.context.clone().with_parent("Program", "body");
            let statement = self.convert_statement(stmt_value)?;
            statements.push(statement);
        }

        // Get span
        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        // Build Program
        // Note: Program needs source_type, but we don't have it here.
        // For now, use a default. This should be passed in from the caller.
        let source_type = SourceType::default().with_module(true);
        let comments = Vec::new_in(self.builder.allocator);
        let directives = Vec::new_in(self.builder.allocator);
        let program = self.builder.program(
            span,
            source_type,
            self.source_text,
            comments,
            None, // hashbang
            directives,
            statements,
        );

        Ok(program)
    }

    /// Convert an ESTree Statement to oxc Statement.
    fn convert_statement(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::Statement<'a>> {
        use oxc_ast::ast::Statement;
        use oxc_estree::deserialize::{EstreeNode, EstreeNodeType};

        let node_type = <Value as EstreeNode>::get_type(estree).ok_or_else(|| ConversionError::MissingField {
            field: "type".to_string(),
            node_type: "unknown".to_string(),
            span: (0, 0),
        })?;

        match node_type {
            EstreeNodeType::ExpressionStatement => {
                self.context = self.context.clone().with_parent("ExpressionStatement", "expression");
                let expr = self.convert_expression(
                    estree.get("expression").ok_or_else(|| ConversionError::MissingField {
                        field: "expression".to_string(),
                        node_type: "ExpressionStatement".to_string(),
                        span: self.get_node_span(estree),
                    })?,
                )?;
                let (start, end) = self.get_node_span(estree);
                let span = Span::new(start, end);
                let stmt = self.builder.alloc_expression_statement(span, expr);
                Ok(Statement::ExpressionStatement(stmt))
            }
            EstreeNodeType::VariableDeclaration => {
                self.convert_variable_declaration(estree)
            }
            EstreeNodeType::ReturnStatement => {
                self.convert_return_statement(estree)
            }
            EstreeNodeType::IfStatement => {
                self.convert_if_statement(estree)
            }
            EstreeNodeType::BlockStatement => {
                self.convert_block_statement(estree)
            }
            _ => Err(ConversionError::UnsupportedNodeType {
                node_type: format!("{:?}", node_type),
                span: self.get_node_span(estree),
            }),
        }
    }

    /// Convert an ESTree BlockStatement to oxc Statement.
    fn convert_block_statement(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::Statement<'a>> {
        use oxc_ast::ast::Statement;

        // Get body array
        let body_value = estree.get("body").ok_or_else(|| ConversionError::MissingField {
            field: "body".to_string(),
            node_type: "BlockStatement".to_string(),
            span: self.get_node_span(estree),
        })?;

        let body_array = body_value.as_array().ok_or_else(|| ConversionError::InvalidFieldType {
            field: "body".to_string(),
            expected: "array".to_string(),
            got: format!("{:?}", body_value),
            span: self.get_node_span(estree),
        })?;

        // Convert each statement
        let mut statements = Vec::new_in(self.builder.allocator);
        for stmt_value in body_array {
            self.context = self.context.clone().with_parent("BlockStatement", "body");
            let statement = self.convert_statement(stmt_value)?;
            statements.push(statement);
        }

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        let block_stmt = self.builder.alloc_block_statement(span, statements);
        Ok(Statement::BlockStatement(block_stmt))
    }

    /// Convert an ESTree IfStatement to oxc Statement.
    fn convert_if_statement(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::Statement<'a>> {
        use oxc_ast::ast::Statement;

        // Get test
        self.context = self.context.clone().with_parent("IfStatement", "test");
        let test_value = estree.get("test").ok_or_else(|| ConversionError::MissingField {
            field: "test".to_string(),
            node_type: "IfStatement".to_string(),
            span: self.get_node_span(estree),
        })?;
        let test = self.convert_expression(test_value)?;

        // Get consequent
        self.context = self.context.clone().with_parent("IfStatement", "consequent");
        let consequent_value = estree.get("consequent").ok_or_else(|| ConversionError::MissingField {
            field: "consequent".to_string(),
            node_type: "IfStatement".to_string(),
            span: self.get_node_span(estree),
        })?;
        let consequent = self.convert_statement(consequent_value)?;

        // Get alternate (optional)
        let alternate = if let Some(alt_value) = estree.get("alternate") {
            if alt_value.is_null() {
                None
            } else {
                self.context = self.context.clone().with_parent("IfStatement", "alternate");
                Some(self.convert_statement(alt_value)?)
            }
        } else {
            None
        };

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        let if_stmt = self.builder.alloc_if_statement(span, test, consequent, alternate);
        Ok(Statement::IfStatement(if_stmt))
    }

    /// Convert an ESTree ReturnStatement to oxc Statement.
    fn convert_return_statement(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::Statement<'a>> {
        use oxc_ast::ast::Statement;

        // Get argument (optional)
        let argument = if let Some(arg_value) = estree.get("argument") {
            if arg_value.is_null() {
                None
            } else {
                self.context = self.context.clone().with_parent("ReturnStatement", "argument");
                Some(self.convert_expression(arg_value)?)
            }
        } else {
            None
        };

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        let return_stmt = self.builder.alloc_return_statement(span, argument);
        Ok(Statement::ReturnStatement(return_stmt))
    }

    /// Convert an ESTree VariableDeclaration to oxc Statement.
    fn convert_variable_declaration(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::Statement<'a>> {
        use oxc_ast::ast::{Statement, VariableDeclarationKind};
        use oxc_estree::deserialize::{EstreeNode, EstreeNodeType};

        // Get kind
        let kind_str = <Value as EstreeNode>::get_string(estree, "kind")
            .ok_or_else(|| ConversionError::MissingField {
                field: "kind".to_string(),
                node_type: "VariableDeclaration".to_string(),
                span: self.get_node_span(estree),
            })?;

        let kind = match kind_str.as_str() {
            "var" => VariableDeclarationKind::Var,
            "let" => VariableDeclarationKind::Let,
            "const" => VariableDeclarationKind::Const,
            "using" => VariableDeclarationKind::Using,
            "await using" => VariableDeclarationKind::AwaitUsing,
            _ => {
                return Err(ConversionError::InvalidFieldType {
                    field: "kind".to_string(),
                    expected: "var|let|const|using|await using".to_string(),
                    got: kind_str,
                    span: self.get_node_span(estree),
                });
            }
        };

        // Get declarations
        let declarations_value = estree.get("declarations").ok_or_else(|| ConversionError::MissingField {
            field: "declarations".to_string(),
            node_type: "VariableDeclaration".to_string(),
            span: self.get_node_span(estree),
        })?;

        let declarations_array = declarations_value.as_array().ok_or_else(|| ConversionError::InvalidFieldType {
            field: "declarations".to_string(),
            expected: "array".to_string(),
            got: format!("{:?}", declarations_value),
            span: self.get_node_span(estree),
        })?;

        let mut declarators = Vec::new_in(self.builder.allocator);
        for decl_value in declarations_array {
            self.context = self.context.clone().with_parent("VariableDeclaration", "declarations");
            let declarator = self.convert_variable_declarator(decl_value, kind)?;
            declarators.push(declarator);
        }

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);
        let declare = estree.get("declare").and_then(|v| v.as_bool()).unwrap_or(false);

        let var_decl = self.builder.alloc_variable_declaration(span, kind, declarators, declare);
        Ok(Statement::VariableDeclaration(var_decl))
    }

    /// Convert an ESTree VariableDeclarator to oxc VariableDeclarator.
    fn convert_variable_declarator(
        &mut self,
        estree: &Value,
        kind: oxc_ast::ast::VariableDeclarationKind,
    ) -> ConversionResult<oxc_ast::ast::VariableDeclarator<'a>> {
        // Get id (pattern)
        self.context = self.context.clone().with_parent("VariableDeclarator", "id");
        let id_value = estree.get("id").ok_or_else(|| ConversionError::MissingField {
            field: "id".to_string(),
            node_type: "VariableDeclarator".to_string(),
            span: self.get_node_span(estree),
        })?;

        let pattern = self.convert_binding_pattern(id_value)?;

        // Get init (optional)
        let init = if let Some(init_value) = estree.get("init") {
            self.context = self.context.clone().with_parent("VariableDeclarator", "init");
            Some(self.convert_expression(init_value)?)
        } else {
            None
        };

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);
        let definite = estree.get("definite").and_then(|v| v.as_bool()).unwrap_or(false);

        Ok(self.builder.variable_declarator(span, kind, pattern, init, definite))
    }

    /// Convert an ESTree Expression to oxc Expression.
    fn convert_expression(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::Expression<'a>> {
        use oxc_ast::ast::Expression;
        use oxc_estree::deserialize::{EstreeNode, EstreeNodeType};

        let node_type = <Value as EstreeNode>::get_type(estree).ok_or_else(|| ConversionError::MissingField {
            field: "type".to_string(),
            node_type: "unknown".to_string(),
            span: self.get_node_span(estree),
        })?;

        match node_type {
            EstreeNodeType::Literal => {
                let literal_expr = self.convert_literal_to_expression(estree)?;
                Ok(literal_expr)
            }
            EstreeNodeType::Identifier => {
                let ident = self.convert_identifier_to_reference(estree)?;
                Ok(Expression::Identifier(oxc_allocator::Box::new_in(ident, self.builder.allocator)))
            }
            EstreeNodeType::CallExpression => {
                self.convert_call_expression(estree)
            }
            EstreeNodeType::BinaryExpression => {
                self.convert_binary_expression(estree)
            }
            EstreeNodeType::MemberExpression => {
                self.convert_member_expression(estree)
            }
            EstreeNodeType::UnaryExpression => {
                self.convert_unary_expression(estree)
            }
            EstreeNodeType::ArrayExpression => {
                self.convert_array_expression(estree)
            }
            EstreeNodeType::ObjectExpression => {
                self.convert_object_expression(estree)
            }
            EstreeNodeType::LogicalExpression => {
                self.convert_logical_expression(estree)
            }
            EstreeNodeType::ConditionalExpression => {
                self.convert_conditional_expression(estree)
            }
            EstreeNodeType::AssignmentExpression => {
                self.convert_assignment_expression(estree)
            }
            EstreeNodeType::UpdateExpression => {
                self.convert_update_expression(estree)
            }
            EstreeNodeType::SequenceExpression => {
                self.convert_sequence_expression(estree)
            }
            _ => Err(ConversionError::UnsupportedNodeType {
                node_type: format!("{:?}", node_type),
                span: self.get_node_span(estree),
            }),
        }
    }

    /// Convert an ESTree Literal to oxc expression.
    fn convert_literal_to_expression(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::Expression<'a>> {
        use oxc_ast::ast::Expression;
        use oxc_span::Atom;

        let estree_literal = EstreeLiteral::from_json(estree)
            .ok_or_else(|| ConversionError::InvalidFieldType {
                field: "Literal".to_string(),
                expected: "valid Literal node".to_string(),
                got: format!("{:?}", estree),
                span: self.get_node_span(estree),
            })?;

        let (start, end) = get_literal_span(&estree_literal);
        let span = convert_span(self.source_text, start as usize, end as usize);

        match convert_literal(&estree_literal)? {
            LiteralKind::Boolean => {
                let value = get_boolean_value(&estree_literal)?;
                Ok(Expression::BooleanLiteral(self.builder.alloc_boolean_literal(span, value)))
            }
            LiteralKind::Numeric => {
                let value = get_numeric_value(&estree_literal)?;
                let raw = estree_literal.raw.as_ref().map(|s| {
                    Atom::from_in(s.as_str(), self.builder.allocator)
                });
                Ok(Expression::NumericLiteral(self.builder.alloc_numeric_literal(span, value, raw, oxc_syntax::number::NumberBase::Decimal)))
            }
            LiteralKind::String => {
                let value_str = get_string_value(&estree_literal)?;
                let atom = Atom::from_in(value_str, self.builder.allocator);
                let raw = estree_literal.raw.as_ref().map(|s| {
                    Atom::from_in(s.as_str(), self.builder.allocator)
                });
                Ok(Expression::StringLiteral(self.builder.alloc_string_literal(span, atom, raw)))
            }
            LiteralKind::Null => {
                Ok(Expression::NullLiteral(self.builder.alloc_null_literal(span)))
            }
            _ => Err(ConversionError::UnsupportedNodeType {
                node_type: format!("Unsupported literal kind"),
                span: self.get_node_span(estree),
            }),
        }
    }

    /// Convert an ESTree Identifier to oxc IdentifierReference.
    fn convert_identifier_to_reference(
        &mut self,
        estree: &Value,
    ) -> ConversionResult<oxc_ast::ast::IdentifierReference<'a>> {
        use oxc_span::Atom;

        let estree_id = EstreeIdentifier::from_json(estree)
            .ok_or_else(|| ConversionError::InvalidFieldType {
                field: "Identifier".to_string(),
                expected: "valid Identifier node".to_string(),
                got: format!("{:?}", estree),
                span: self.get_node_span(estree),
            })?;

        let kind = convert_identifier(&estree_id, &self.context, self.source_text)?;
        
        // For now, only handle Reference case
        // TODO: Handle other kinds when needed
        if kind != IdentifierKind::Reference {
            return Err(ConversionError::InvalidIdentifierContext {
                context: format!("Expected Reference, got {:?}", kind),
                span: self.get_node_span(estree),
            });
        }

        let name = Atom::from_in(estree_id.name.as_str(), self.builder.allocator);
        let range = estree_id.range.unwrap_or([0, 0]);
        let span = convert_span(self.source_text, range[0] as usize, range[1] as usize);

        Ok(self.builder.identifier_reference(span, name))
    }

    /// Convert an ESTree Pattern to oxc BindingPattern.
    fn convert_binding_pattern(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::BindingPattern<'a>> {
        use oxc_ast::ast::BindingPattern;
        use oxc_estree::deserialize::{EstreeNode, EstreeNodeType};

        let node_type = <Value as EstreeNode>::get_type(estree).ok_or_else(|| ConversionError::MissingField {
            field: "type".to_string(),
            node_type: "unknown".to_string(),
            span: self.get_node_span(estree),
        })?;

        match node_type {
            EstreeNodeType::Identifier => {
                // Convert to BindingIdentifier
                let estree_id = EstreeIdentifier::from_json(estree)
                    .ok_or_else(|| ConversionError::InvalidFieldType {
                        field: "Identifier".to_string(),
                        expected: "valid Identifier node".to_string(),
                        got: format!("{:?}", estree),
                        span: self.get_node_span(estree),
                    })?;

                let kind = convert_identifier(&estree_id, &self.context, self.source_text)?;
                if kind != IdentifierKind::Binding {
                    return Err(ConversionError::InvalidIdentifierContext {
                        context: format!("Expected Binding, got {:?}", kind),
                        span: self.get_node_span(estree),
                    });
                }

                use oxc_ast::ast::BindingPatternKind;
                let name = Atom::from_in(estree_id.name.as_str(), self.builder.allocator);
                let range = estree_id.range.unwrap_or([0, 0]);
                let span = convert_span(self.source_text, range[0] as usize, range[1] as usize);
                let binding_id = self.builder.alloc_binding_identifier(span, name);
                let pattern = self.builder.binding_pattern(BindingPatternKind::BindingIdentifier(binding_id), None::<oxc_ast::ast::TSTypeAnnotation>, false);
                Ok(pattern)
            }
            _ => Err(ConversionError::UnsupportedNodeType {
                node_type: format!("{:?}", node_type),
                span: self.get_node_span(estree),
            }),
        }
    }

    /// Convert an ESTree ObjectExpression to oxc ObjectExpression.
    fn convert_object_expression(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::Expression<'a>> {
        use oxc_ast::ast::{Expression, ObjectPropertyKind};
        use oxc_estree::deserialize::{EstreeNode, EstreeNodeType};
        use oxc_span::Atom;

        // Get properties array
        let properties_value = estree.get("properties").ok_or_else(|| ConversionError::MissingField {
            field: "properties".to_string(),
            node_type: "ObjectExpression".to_string(),
            span: self.get_node_span(estree),
        })?;

        let properties_array = properties_value.as_array().ok_or_else(|| ConversionError::InvalidFieldType {
            field: "properties".to_string(),
            expected: "array".to_string(),
            got: format!("{:?}", properties_value),
            span: self.get_node_span(estree),
        })?;

        // Convert each property
        let mut properties = Vec::new_in(self.builder.allocator);
        for prop_value in properties_array {
            self.context = self.context.clone().with_parent("ObjectExpression", "properties");
            let prop = self.convert_object_property(prop_value)?;
            properties.push(prop);
        }

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        let obj_expr = self.builder.alloc_object_expression(span, properties);
        Ok(Expression::ObjectExpression(obj_expr))
    }

    /// Convert an ESTree Property to oxc ObjectPropertyKind.
    fn convert_object_property(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::ObjectPropertyKind<'a>> {
        use oxc_ast::ast::{ObjectProperty, ObjectPropertyKind, PropertyKey};
        use oxc_estree::deserialize::{EstreeNode, EstreeNodeType};
        use oxc_span::Atom;

        let node_type = <Value as EstreeNode>::get_type(estree).ok_or_else(|| ConversionError::MissingField {
            field: "type".to_string(),
            node_type: "unknown".to_string(),
            span: self.get_node_span(estree),
        })?;

        if !matches!(node_type, EstreeNodeType::Property) {
            return Err(ConversionError::UnsupportedNodeType {
                node_type: format!("{:?}", node_type),
                span: self.get_node_span(estree),
            });
        }

        // Get kind (init, get, set)
        let kind_str = <Value as EstreeNode>::get_string(estree, "kind")
            .unwrap_or_else(|| "init".to_string());

        // For now, only handle "init" properties
        if kind_str != "init" {
            return Err(ConversionError::UnsupportedNodeType {
                node_type: format!("Property with kind={}", kind_str),
                span: self.get_node_span(estree),
            });
        }

        // Get key
        self.context = self.context.clone().with_parent("Property", "key");
        let key_value = estree.get("key").ok_or_else(|| ConversionError::MissingField {
            field: "key".to_string(),
            node_type: "Property".to_string(),
            span: self.get_node_span(estree),
        })?;

        let computed = estree.get("computed").and_then(|v| v.as_bool()).unwrap_or(false);
        let key = if computed {
            // Computed property: [expr]
            let key_expr = self.convert_expression(key_value)?;
            PropertyKey::from(key_expr)
        } else {
            // Static property: identifier or literal
            let key_node_type = <Value as EstreeNode>::get_type(key_value).ok_or_else(|| ConversionError::MissingField {
                field: "type".to_string(),
                node_type: "key".to_string(),
                span: self.get_node_span(estree),
            })?;

            match key_node_type {
                EstreeNodeType::Identifier => {
                    let estree_id = oxc_estree::deserialize::EstreeIdentifier::from_json(key_value)
                        .ok_or_else(|| ConversionError::InvalidFieldType {
                            field: "key".to_string(),
                            expected: "valid Identifier node".to_string(),
                            got: format!("{:?}", key_value),
                            span: self.get_node_span(estree),
                        })?;

                    // In Property.key, identifier should be IdentifierName
                    let kind = oxc_estree::deserialize::convert_identifier(&estree_id, &self.context, self.source_text)?;
                    if kind != oxc_estree::deserialize::IdentifierKind::Name {
                        return Err(ConversionError::InvalidIdentifierContext {
                            context: format!("Expected Name in Property.key, got {:?}", kind),
                            span: self.get_node_span(estree),
                        });
                    }

                    let name = Atom::from_in(estree_id.name.as_str(), self.builder.allocator);
                    let range = estree_id.range.unwrap_or([0, 0]);
                    let span = convert_span(self.source_text, range[0] as usize, range[1] as usize);
                    let ident = self.builder.identifier_name(span, name);
                    PropertyKey::StaticIdentifier(oxc_allocator::Box::new_in(ident, self.builder.allocator))
                }
                EstreeNodeType::Literal => {
                    // String literal as key: "key"
                    let estree_literal = oxc_estree::deserialize::EstreeLiteral::from_json(key_value)
                        .ok_or_else(|| ConversionError::InvalidFieldType {
                            field: "key".to_string(),
                            expected: "valid Literal node".to_string(),
                            got: format!("{:?}", key_value),
                            span: self.get_node_span(estree),
                        })?;

                    let value_str = oxc_estree::deserialize::get_string_value(&estree_literal)?;
                    let atom = Atom::from_in(value_str, self.builder.allocator);
                    let (start, end) = oxc_estree::deserialize::get_literal_span(&estree_literal);
                    let span = convert_span(self.source_text, start as usize, end as usize);
                    let str_lit = self.builder.alloc_string_literal(span, atom, None);
                    PropertyKey::StringLiteral(str_lit)
                }
                _ => {
                    return Err(ConversionError::UnsupportedNodeType {
                        node_type: format!("Property key type: {:?}", key_node_type),
                        span: self.get_node_span(estree),
                    });
                }
            }
        };

        // Get value
        self.context = self.context.clone().with_parent("Property", "value");
        let value_value = estree.get("value").ok_or_else(|| ConversionError::MissingField {
            field: "value".to_string(),
            node_type: "Property".to_string(),
            span: self.get_node_span(estree),
        })?;
        let value = self.convert_expression(value_value)?;

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        let method = estree.get("method").and_then(|v| v.as_bool()).unwrap_or(false);
        let shorthand = estree.get("shorthand").and_then(|v| v.as_bool()).unwrap_or(false);

        // TODO: Handle method and shorthand properties
        if method || shorthand {
            return Err(ConversionError::UnsupportedNodeType {
                node_type: format!("Property with method={} or shorthand={}", method, shorthand),
                span: self.get_node_span(estree),
            });
        }

        use oxc_ast::ast::PropertyKind;
        let kind = PropertyKind::Init;
        let obj_prop = self.builder.alloc_object_property(span, kind, key, value, method, shorthand, computed);
        Ok(ObjectPropertyKind::ObjectProperty(obj_prop))
    }

    /// Convert an ESTree SequenceExpression to oxc SequenceExpression.
    fn convert_sequence_expression(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::Expression<'a>> {
        use oxc_ast::ast::Expression;
        use oxc_estree::deserialize::{EstreeNode, EstreeNodeType};

        // Get expressions array
        let expressions_value = estree.get("expressions").ok_or_else(|| ConversionError::MissingField {
            field: "expressions".to_string(),
            node_type: "SequenceExpression".to_string(),
            span: self.get_node_span(estree),
        })?;

        let expressions_array = expressions_value.as_array().ok_or_else(|| ConversionError::InvalidFieldType {
            field: "expressions".to_string(),
            expected: "array".to_string(),
            got: format!("{:?}", expressions_value),
            span: self.get_node_span(estree),
        })?;

        // Convert each expression
        let mut expressions = Vec::new_in(self.builder.allocator);
        for expr_value in expressions_array {
            self.context = self.context.clone().with_parent("SequenceExpression", "expressions");
            let expr = self.convert_expression(expr_value)?;
            expressions.push(expr);
        }

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        let seq_expr = self.builder.alloc_sequence_expression(span, expressions);
        Ok(Expression::SequenceExpression(seq_expr))
    }

    /// Convert an ESTree UpdateExpression to oxc UpdateExpression.
    fn convert_update_expression(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::Expression<'a>> {
        use oxc_ast::ast::{Expression, SimpleAssignmentTarget};
        use oxc_estree::deserialize::{EstreeNode, EstreeNodeType};
        use oxc_syntax::operator::UpdateOperator;

        // Get operator
        let operator_str = <Value as EstreeNode>::get_string(estree, "operator")
            .ok_or_else(|| ConversionError::MissingField {
                field: "operator".to_string(),
                node_type: "UpdateExpression".to_string(),
                span: self.get_node_span(estree),
            })?;

        let operator = match operator_str.as_str() {
            "++" => UpdateOperator::Increment,
            "--" => UpdateOperator::Decrement,
            _ => {
                return Err(ConversionError::InvalidFieldType {
                    field: "operator".to_string(),
                    expected: "valid update operator (++, --)".to_string(),
                    got: operator_str,
                    span: self.get_node_span(estree),
                });
            }
        };

        // Get prefix flag
        let prefix = estree.get("prefix").and_then(|v| v.as_bool()).unwrap_or(false);

        // Get argument (must be a SimpleAssignmentTarget)
        self.context = self.context.clone().with_parent("UpdateExpression", "argument");
        let argument_value = estree.get("argument").ok_or_else(|| ConversionError::MissingField {
            field: "argument".to_string(),
            node_type: "UpdateExpression".to_string(),
            span: self.get_node_span(estree),
        })?;

        // Convert to AssignmentTarget first, then extract SimpleAssignmentTarget
        let assignment_target = self.convert_to_assignment_target(argument_value)?;
        let argument = match assignment_target {
            oxc_ast::ast::AssignmentTarget::AssignmentTargetIdentifier(ident) => {
                SimpleAssignmentTarget::AssignmentTargetIdentifier(ident)
            }
            oxc_ast::ast::AssignmentTarget::ComputedMemberExpression(expr) => {
                SimpleAssignmentTarget::ComputedMemberExpression(expr)
            }
            oxc_ast::ast::AssignmentTarget::StaticMemberExpression(expr) => {
                SimpleAssignmentTarget::StaticMemberExpression(expr)
            }
            _ => {
                return Err(ConversionError::UnsupportedNodeType {
                    node_type: format!("UpdateExpression argument must be SimpleAssignmentTarget, got {:?}", assignment_target),
                    span: self.get_node_span(estree),
                });
            }
        };

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        let update_expr = self.builder.alloc_update_expression(span, operator, prefix, argument);
        Ok(Expression::UpdateExpression(update_expr))
    }

    /// Convert an ESTree AssignmentExpression to oxc AssignmentExpression.
    fn convert_assignment_expression(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::Expression<'a>> {
        use oxc_ast::ast::{AssignmentTarget, Expression};
        use oxc_estree::deserialize::{determine_pattern_kind, EstreeNode, EstreeNodeType, PatternTargetKind};
        use oxc_syntax::operator::AssignmentOperator;
        use oxc_span::Atom;

        // Get operator
        let operator_str = <Value as EstreeNode>::get_string(estree, "operator")
            .ok_or_else(|| ConversionError::MissingField {
                field: "operator".to_string(),
                node_type: "AssignmentExpression".to_string(),
                span: self.get_node_span(estree),
            })?;

        let operator = match operator_str.as_str() {
            "=" => AssignmentOperator::Assign,
            "+=" => AssignmentOperator::Addition,
            "-=" => AssignmentOperator::Subtraction,
            "*=" => AssignmentOperator::Multiplication,
            "/=" => AssignmentOperator::Division,
            "%=" => AssignmentOperator::Remainder,
            "**=" => AssignmentOperator::Exponential,
            "<<=" => AssignmentOperator::ShiftLeft,
            ">>=" => AssignmentOperator::ShiftRight,
            ">>>=" => AssignmentOperator::ShiftRightZeroFill,
            "&=" => AssignmentOperator::BitwiseAnd,
            "|=" => AssignmentOperator::BitwiseOR,
            "^=" => AssignmentOperator::BitwiseXOR,
            "||=" => AssignmentOperator::LogicalOr,
            "&&=" => AssignmentOperator::LogicalAnd,
            "??=" => AssignmentOperator::LogicalNullish,
            _ => {
                return Err(ConversionError::InvalidFieldType {
                    field: "operator".to_string(),
                    expected: "valid assignment operator".to_string(),
                    got: operator_str,
                    span: self.get_node_span(estree),
                });
            }
        };

        // Get left (assignment target)
        self.context = self.context.clone().with_parent("AssignmentExpression", "left");
        let left_value = estree.get("left").ok_or_else(|| ConversionError::MissingField {
            field: "left".to_string(),
            node_type: "AssignmentExpression".to_string(),
            span: self.get_node_span(estree),
        })?;

        // Determine if left is a pattern or assignment target
        let pattern_kind = determine_pattern_kind(left_value, &self.context)?;
        let left = match pattern_kind {
            PatternTargetKind::AssignmentTarget => {
                // Convert to AssignmentTarget
                self.convert_to_assignment_target(left_value)?
            }
            _ => {
                return Err(ConversionError::InvalidFieldType {
                    field: "left".to_string(),
                    expected: "AssignmentTarget".to_string(),
                    got: format!("Pattern kind: {:?}", pattern_kind),
                    span: self.get_node_span(estree),
                });
            }
        };

        // Get right
        self.context = self.context.clone().with_parent("AssignmentExpression", "right");
        let right_value = estree.get("right").ok_or_else(|| ConversionError::MissingField {
            field: "right".to_string(),
            node_type: "AssignmentExpression".to_string(),
            span: self.get_node_span(estree),
        })?;
        let right = self.convert_expression(right_value)?;

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        let assign_expr = self.builder.alloc_assignment_expression(span, operator, left, right);
        Ok(Expression::AssignmentExpression(assign_expr))
    }

    /// Convert an ESTree node to oxc AssignmentTarget.
    fn convert_to_assignment_target(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::AssignmentTarget<'a>> {
        use oxc_ast::ast::{AssignmentTarget, IdentifierReference};
        use oxc_estree::deserialize::{convert_identifier, EstreeIdentifier, EstreeNode, EstreeNodeType, IdentifierKind};
        use oxc_span::Atom;

        let node_type = <Value as EstreeNode>::get_type(estree).ok_or_else(|| ConversionError::MissingField {
            field: "type".to_string(),
            node_type: "unknown".to_string(),
            span: self.get_node_span(estree),
        })?;

        match node_type {
            EstreeNodeType::Identifier => {
                // Convert to AssignmentTargetIdentifier
                let estree_id = EstreeIdentifier::from_json(estree)
                    .ok_or_else(|| ConversionError::InvalidFieldType {
                        field: "Identifier".to_string(),
                        expected: "valid Identifier node".to_string(),
                        got: format!("{:?}", estree),
                        span: self.get_node_span(estree),
                    })?;

                let kind = convert_identifier(&estree_id, &self.context, self.source_text)?;
                if kind != IdentifierKind::Reference {
                    return Err(ConversionError::InvalidIdentifierContext {
                        context: format!("Expected Reference in AssignmentExpression.left, got {:?}", kind),
                        span: self.get_node_span(estree),
                    });
                }

                let name = Atom::from_in(estree_id.name.as_str(), self.builder.allocator);
                let range = estree_id.range.unwrap_or([0, 0]);
                let span = convert_span(self.source_text, range[0] as usize, range[1] as usize);
                let ident = self.builder.identifier_reference(span, name);
                Ok(AssignmentTarget::AssignmentTargetIdentifier(oxc_allocator::Box::new_in(ident, self.builder.allocator)))
            }
            _ => Err(ConversionError::UnsupportedNodeType {
                node_type: format!("AssignmentTarget from {:?}", node_type),
                span: self.get_node_span(estree),
            }),
        }
    }

    /// Convert an ESTree ConditionalExpression to oxc ConditionalExpression.
    fn convert_conditional_expression(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::Expression<'a>> {
        use oxc_ast::ast::Expression;
        use oxc_estree::deserialize::{EstreeNode, EstreeNodeType};

        // Get test
        self.context = self.context.clone().with_parent("ConditionalExpression", "test");
        let test_value = estree.get("test").ok_or_else(|| ConversionError::MissingField {
            field: "test".to_string(),
            node_type: "ConditionalExpression".to_string(),
            span: self.get_node_span(estree),
        })?;
        let test = self.convert_expression(test_value)?;

        // Get consequent
        self.context = self.context.clone().with_parent("ConditionalExpression", "consequent");
        let consequent_value = estree.get("consequent").ok_or_else(|| ConversionError::MissingField {
            field: "consequent".to_string(),
            node_type: "ConditionalExpression".to_string(),
            span: self.get_node_span(estree),
        })?;
        let consequent = self.convert_expression(consequent_value)?;

        // Get alternate
        self.context = self.context.clone().with_parent("ConditionalExpression", "alternate");
        let alternate_value = estree.get("alternate").ok_or_else(|| ConversionError::MissingField {
            field: "alternate".to_string(),
            node_type: "ConditionalExpression".to_string(),
            span: self.get_node_span(estree),
        })?;
        let alternate = self.convert_expression(alternate_value)?;

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        let cond_expr = self.builder.alloc_conditional_expression(span, test, consequent, alternate);
        Ok(Expression::ConditionalExpression(cond_expr))
    }

    /// Convert an ESTree LogicalExpression to oxc LogicalExpression.
    fn convert_logical_expression(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::Expression<'a>> {
        use oxc_ast::ast::Expression;
        use oxc_estree::deserialize::{EstreeNode, EstreeNodeType};
        use oxc_syntax::operator::LogicalOperator;

        // Get operator
        let operator_str = <Value as EstreeNode>::get_string(estree, "operator")
            .ok_or_else(|| ConversionError::MissingField {
                field: "operator".to_string(),
                node_type: "LogicalExpression".to_string(),
                span: self.get_node_span(estree),
            })?;

        let operator = match operator_str.as_str() {
            "&&" => LogicalOperator::And,
            "||" => LogicalOperator::Or,
            "??" => LogicalOperator::Coalesce,
            _ => {
                return Err(ConversionError::InvalidFieldType {
                    field: "operator".to_string(),
                    expected: "valid logical operator (&&, ||, ??)".to_string(),
                    got: operator_str,
                    span: self.get_node_span(estree),
                });
            }
        };

        // Get left operand
        self.context = self.context.clone().with_parent("LogicalExpression", "left");
        let left_value = estree.get("left").ok_or_else(|| ConversionError::MissingField {
            field: "left".to_string(),
            node_type: "LogicalExpression".to_string(),
            span: self.get_node_span(estree),
        })?;
        let left = self.convert_expression(left_value)?;

        // Get right operand
        self.context = self.context.clone().with_parent("LogicalExpression", "right");
        let right_value = estree.get("right").ok_or_else(|| ConversionError::MissingField {
            field: "right".to_string(),
            node_type: "LogicalExpression".to_string(),
            span: self.get_node_span(estree),
        })?;
        let right = self.convert_expression(right_value)?;

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        let logical_expr = self.builder.alloc_logical_expression(span, left, operator, right);
        Ok(Expression::LogicalExpression(logical_expr))
    }

    /// Convert an ESTree ArrayExpression to oxc ArrayExpression.
    fn convert_array_expression(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::Expression<'a>> {
        use oxc_ast::ast::{ArrayExpressionElement, Expression};
        use oxc_estree::deserialize::{EstreeNode, EstreeNodeType};

        // Get elements array
        let elements_value = estree.get("elements").ok_or_else(|| ConversionError::MissingField {
            field: "elements".to_string(),
            node_type: "ArrayExpression".to_string(),
            span: self.get_node_span(estree),
        })?;

        let elements_array = elements_value.as_array().ok_or_else(|| ConversionError::InvalidFieldType {
            field: "elements".to_string(),
            expected: "array".to_string(),
            got: format!("{:?}", elements_value),
            span: self.get_node_span(estree),
        })?;

        // Convert each element
        let mut elements = Vec::new_in(self.builder.allocator);
        for elem_value in elements_array {
            // ESTree allows null for sparse arrays (elisions)
            if elem_value.is_null() {
                // For now, skip null elements (sparse arrays)
                // TODO: Handle Elision properly
                continue;
            }

            self.context = self.context.clone().with_parent("ArrayExpression", "elements");
            let expr = self.convert_expression(elem_value)?;
            elements.push(ArrayExpressionElement::from(expr));
        }

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        let array_expr = self.builder.alloc_array_expression(span, elements);
        Ok(Expression::ArrayExpression(array_expr))
    }

    /// Convert an ESTree UnaryExpression to oxc UnaryExpression.
    fn convert_unary_expression(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::Expression<'a>> {
        use oxc_ast::ast::Expression;
        use oxc_estree::deserialize::{EstreeNode, EstreeNodeType};
        use oxc_syntax::operator::UnaryOperator;

        // Get operator
        let operator_str = <Value as EstreeNode>::get_string(estree, "operator")
            .ok_or_else(|| ConversionError::MissingField {
                field: "operator".to_string(),
                node_type: "UnaryExpression".to_string(),
                span: self.get_node_span(estree),
            })?;

        let operator = match operator_str.as_str() {
            "!" => UnaryOperator::LogicalNot,
            "-" => UnaryOperator::UnaryNegation,
            "+" => UnaryOperator::UnaryPlus,
            "~" => UnaryOperator::BitwiseNot,
            "typeof" => UnaryOperator::Typeof,
            "void" => UnaryOperator::Void,
            "delete" => UnaryOperator::Delete,
            _ => {
                return Err(ConversionError::InvalidFieldType {
                    field: "operator".to_string(),
                    expected: "valid unary operator".to_string(),
                    got: operator_str,
                    span: self.get_node_span(estree),
                });
            }
        };

        // Get argument
        self.context = self.context.clone().with_parent("UnaryExpression", "argument");
        let argument_value = estree.get("argument").ok_or_else(|| ConversionError::MissingField {
            field: "argument".to_string(),
            node_type: "UnaryExpression".to_string(),
            span: self.get_node_span(estree),
        })?;
        let argument = self.convert_expression(argument_value)?;

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        let unary_expr = self.builder.alloc_unary_expression(span, operator, argument);
        Ok(Expression::UnaryExpression(unary_expr))
    }

    /// Convert an ESTree MemberExpression to oxc MemberExpression.
    fn convert_member_expression(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::Expression<'a>> {
        use oxc_ast::ast::Expression;
        use oxc_estree::deserialize::{EstreeNode, EstreeNodeType};

        // Get object
        self.context = self.context.clone().with_parent("MemberExpression", "object");
        let object_value = estree.get("object").ok_or_else(|| ConversionError::MissingField {
            field: "object".to_string(),
            node_type: "MemberExpression".to_string(),
            span: self.get_node_span(estree),
        })?;
        let object = self.convert_expression(object_value)?;

        // Get property
        self.context = self.context.clone().with_parent("MemberExpression", "property");
        let property_value = estree.get("property").ok_or_else(|| ConversionError::MissingField {
            field: "property".to_string(),
            node_type: "MemberExpression".to_string(),
            span: self.get_node_span(estree),
        })?;

        let computed = estree.get("computed").and_then(|v| v.as_bool()).unwrap_or(false);
        let optional = estree.get("optional").and_then(|v| v.as_bool()).unwrap_or(false);

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        if computed {
            // ComputedMemberExpression: obj[prop]
            let property_expr = self.convert_expression(property_value)?;
            let computed_expr = self.builder.alloc_computed_member_expression(span, object, property_expr, optional);
            Ok(Expression::ComputedMemberExpression(computed_expr))
        } else {
            // StaticMemberExpression: obj.prop
            // Property should be an IdentifierName
            let property_node_type = <Value as EstreeNode>::get_type(property_value)
                .ok_or_else(|| ConversionError::MissingField {
                    field: "type".to_string(),
                    node_type: "property".to_string(),
                    span: self.get_node_span(estree),
                })?;

            if !matches!(property_node_type, EstreeNodeType::Identifier) {
                return Err(ConversionError::InvalidFieldType {
                    field: "property".to_string(),
                    expected: "Identifier".to_string(),
                    got: format!("{:?}", property_node_type),
                    span: self.get_node_span(estree),
                });
            }

            let estree_id = EstreeIdentifier::from_json(property_value)
                .ok_or_else(|| ConversionError::InvalidFieldType {
                    field: "property".to_string(),
                    expected: "valid Identifier node".to_string(),
                    got: format!("{:?}", property_value),
                    span: self.get_node_span(estree),
                })?;

            // In MemberExpression.property, identifier should be IdentifierName
            let kind = convert_identifier(&estree_id, &self.context, self.source_text)?;
            if kind != IdentifierKind::Name {
                return Err(ConversionError::InvalidIdentifierContext {
                    context: format!("Expected Name in MemberExpression.property, got {:?}", kind),
                    span: self.get_node_span(estree),
                });
            }

            let name = Atom::from_in(estree_id.name.as_str(), self.builder.allocator);
            let property_span = convert_span(self.source_text, estree_id.range.unwrap_or([0, 0])[0] as usize, estree_id.range.unwrap_or([0, 0])[1] as usize);
            let property = self.builder.identifier_name(property_span, name);

            let static_expr = self.builder.alloc_static_member_expression(span, object, property, optional);
            Ok(Expression::StaticMemberExpression(static_expr))
        }
    }

    /// Convert an ESTree BinaryExpression to oxc BinaryExpression.
    fn convert_binary_expression(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::Expression<'a>> {
        use oxc_ast::ast::Expression;
        use oxc_estree::deserialize::{EstreeNode, EstreeNodeType};
        use oxc_syntax::operator::BinaryOperator;

        // Get operator
        let operator_str = <Value as EstreeNode>::get_string(estree, "operator")
            .ok_or_else(|| ConversionError::MissingField {
                field: "operator".to_string(),
                node_type: "BinaryExpression".to_string(),
                span: self.get_node_span(estree),
            })?;

        let operator = match operator_str.as_str() {
            "+" => BinaryOperator::Addition,
            "-" => BinaryOperator::Subtraction,
            "*" => BinaryOperator::Multiplication,
            "/" => BinaryOperator::Division,
            "%" => BinaryOperator::Remainder,
            "**" => BinaryOperator::Exponential,
            "==" => BinaryOperator::Equality,
            "!=" => BinaryOperator::Inequality,
            "===" => BinaryOperator::StrictEquality,
            "!==" => BinaryOperator::StrictInequality,
            "<" => BinaryOperator::LessThan,
            "<=" => BinaryOperator::LessEqualThan,
            ">" => BinaryOperator::GreaterThan,
            ">=" => BinaryOperator::GreaterEqualThan,
            "<<" => BinaryOperator::ShiftLeft,
            ">>" => BinaryOperator::ShiftRight,
            ">>>" => BinaryOperator::ShiftRightZeroFill,
            "&" => BinaryOperator::BitwiseAnd,
            "|" => BinaryOperator::BitwiseOR,
            "^" => BinaryOperator::BitwiseXOR,
            "in" => BinaryOperator::In,
            "instanceof" => BinaryOperator::Instanceof,
            _ => {
                return Err(ConversionError::InvalidFieldType {
                    field: "operator".to_string(),
                    expected: "valid binary operator".to_string(),
                    got: operator_str,
                    span: self.get_node_span(estree),
                });
            }
        };

        // Get left operand
        self.context = self.context.clone().with_parent("BinaryExpression", "left");
        let left_value = estree.get("left").ok_or_else(|| ConversionError::MissingField {
            field: "left".to_string(),
            node_type: "BinaryExpression".to_string(),
            span: self.get_node_span(estree),
        })?;
        let left = self.convert_expression(left_value)?;

        // Get right operand
        self.context = self.context.clone().with_parent("BinaryExpression", "right");
        let right_value = estree.get("right").ok_or_else(|| ConversionError::MissingField {
            field: "right".to_string(),
            node_type: "BinaryExpression".to_string(),
            span: self.get_node_span(estree),
        })?;
        let right = self.convert_expression(right_value)?;

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        let bin_expr = self.builder.alloc_binary_expression(span, left, operator, right);
        Ok(Expression::BinaryExpression(bin_expr))
    }

    /// Convert an ESTree CallExpression to oxc CallExpression.
    fn convert_call_expression(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::Expression<'a>> {
        use oxc_ast::ast::{Argument, Expression};
        use oxc_estree::deserialize::{EstreeNode, EstreeNodeType};

        // Get callee
        self.context = self.context.clone().with_parent("CallExpression", "callee");
        let callee_value = estree.get("callee").ok_or_else(|| ConversionError::MissingField {
            field: "callee".to_string(),
            node_type: "CallExpression".to_string(),
            span: self.get_node_span(estree),
        })?;
        let callee = self.convert_expression(callee_value)?;

        // Get arguments
        let arguments_value = estree.get("arguments").ok_or_else(|| ConversionError::MissingField {
            field: "arguments".to_string(),
            node_type: "CallExpression".to_string(),
            span: self.get_node_span(estree),
        })?;

        let arguments_array = arguments_value.as_array().ok_or_else(|| ConversionError::InvalidFieldType {
            field: "arguments".to_string(),
            expected: "array".to_string(),
            got: format!("{:?}", arguments_value),
            span: self.get_node_span(estree),
        })?;

        let mut args = Vec::new_in(self.builder.allocator);
        for arg_value in arguments_array {
            self.context = self.context.clone().with_parent("CallExpression", "arguments");
            // For now, treat all arguments as expressions
            // TODO: Handle SpreadElement separately
            let arg_expr = self.convert_expression(arg_value)?;
            args.push(Argument::from(arg_expr));
        }

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);
        let optional = estree.get("optional").and_then(|v| v.as_bool()).unwrap_or(false);

        let call_expr = self.builder.alloc_call_expression(span, callee, oxc_ast::NONE, args, optional);
        Ok(Expression::CallExpression(call_expr))
    }

    /// Get the span from an ESTree node.
    fn get_node_span(&self, estree: &Value) -> (u32, u32) {
        if let Some(range) = estree.get("range").and_then(|v| v.as_array()) {
            if range.len() >= 2 {
                let start = range[0].as_u64().unwrap_or(0) as usize;
                let end = range[1].as_u64().unwrap_or(0) as usize;
                return (start as u32, end as u32);
            }
        }
        (0, 0)
    }
}

/// Convert ESTree span (character offsets) to oxc span (byte offsets).
///
/// ESTree uses character offsets (for UTF-16 compatibility),
/// while oxc uses byte offsets.
pub fn convert_span(source_text: &str, start: usize, end: usize) -> Span {
    // Fast path for ASCII-only files
    if source_text.is_ascii() {
        return Span::new(
            start.min(source_text.len()) as u32,
            end.min(source_text.len()) as u32,
        );
    }

    // Slow path for UTF-8 files
    let start_byte = source_text
        .char_indices()
        .nth(start)
        .map(|(byte_offset, _)| byte_offset)
        .unwrap_or(source_text.len());
    let end_byte = source_text
        .char_indices()
        .nth(end)
        .map(|(byte_offset, _)| byte_offset)
        .unwrap_or(source_text.len());

    Span::new(start_byte as u32, end_byte as u32)
}

