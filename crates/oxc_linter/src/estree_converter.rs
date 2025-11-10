//! ESTree to oxc AST conversion bridge.
//!
//! This module provides the conversion from ESTree AST (from custom parsers)
//! to oxc AST. It uses utilities from `oxc_estree::deserialize` but implements
//! the actual AST construction here since it has access to `oxc_allocator` and `oxc_ast`.

use oxc_allocator::{Allocator, CloneIn, FromIn, Vec};
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
                node_type: "convert_program".to_string(),
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
            // Skip null values (sparse arrays)
            if stmt_value.is_null() {
                continue;
            }
            // Push context for this statement
            self.context = self.context.clone().with_parent("Program", "body");
            // Debug: check if stmt_value is actually a JSON object
            if !stmt_value.is_object() {
                return Err(ConversionError::InvalidFieldType {
                    field: "body[statement]".to_string(),
                    expected: "object".to_string(),
                    got: format!("{:?}", stmt_value),
                    span: (0, 0),
                });
            }
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

        // Debug: check if estree is actually a JSON object
        if !estree.is_object() {
            return Err(ConversionError::InvalidFieldType {
                field: "statement".to_string(),
                expected: "object".to_string(),
                got: format!("{:?}", estree),
                span: (0, 0),
            });
        }
        let node_type = <Value as EstreeNode>::get_type(estree).ok_or_else(|| {
            // Debug: check what we actually have
            let has_type = estree.get("type").is_some();
            let type_str = estree.get("type").and_then(|v| v.as_str()).unwrap_or("none");
            ConversionError::MissingField {
                field: "type".to_string(),
                node_type: format!("convert_statement (is_object: {}, has_type: {}, type: {})", estree.is_object(), has_type, type_str),
                span: self.get_node_span(estree),
            }
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
            EstreeNodeType::WhileStatement => {
                self.convert_while_statement(estree)
            }
            EstreeNodeType::ForStatement => {
                self.convert_for_statement(estree)
            }
            EstreeNodeType::BreakStatement => {
                self.convert_break_statement(estree)
            }
            EstreeNodeType::ContinueStatement => {
                self.convert_continue_statement(estree)
            }
            EstreeNodeType::ThrowStatement => {
                self.convert_throw_statement(estree)
            }
            EstreeNodeType::DoWhileStatement => {
                self.convert_do_while_statement(estree)
            }
            EstreeNodeType::ForInStatement => {
                self.convert_for_in_statement(estree)
            }
            EstreeNodeType::ForOfStatement => {
                self.convert_for_of_statement(estree)
            }
            EstreeNodeType::EmptyStatement => {
                self.convert_empty_statement(estree)
            }
            EstreeNodeType::LabeledStatement => {
                self.convert_labeled_statement(estree)
            }
            EstreeNodeType::SwitchStatement => {
                self.convert_switch_statement(estree)
            }
            EstreeNodeType::TryStatement => {
                self.convert_try_statement(estree)
            }
            EstreeNodeType::FunctionDeclaration => {
                self.convert_function_declaration(estree)
            }
            EstreeNodeType::ClassDeclaration => {
                self.convert_class_declaration(estree)
            }
            EstreeNodeType::ImportDeclaration => {
                self.convert_import_declaration(estree)
            }
            EstreeNodeType::ExportNamedDeclaration => {
                self.convert_export_named_declaration(estree)
            }
            EstreeNodeType::ExportDefaultDeclaration => {
                self.convert_export_default_declaration(estree)
            }
            EstreeNodeType::ExportAllDeclaration => {
                self.convert_export_all_declaration(estree)
            }
            EstreeNodeType::TSInterfaceDeclaration => {
                self.convert_ts_interface_declaration(estree)
            }
            EstreeNodeType::TSEnumDeclaration => {
                self.convert_ts_enum_declaration(estree)
            }
            EstreeNodeType::TSTypeAliasDeclaration => {
                self.convert_ts_type_alias_declaration(estree)
            }
            EstreeNodeType::TSModuleDeclaration => {
                self.convert_ts_module_declaration(estree)
            }
            EstreeNodeType::TSImportEqualsDeclaration => {
                self.convert_ts_import_equals_declaration(estree)
            }
            EstreeNodeType::TSExportAssignment => {
                self.convert_ts_export_assignment(estree)
            }
            EstreeNodeType::TSNamespaceExportDeclaration => {
                self.convert_ts_namespace_export_declaration(estree)
            }
            EstreeNodeType::DebuggerStatement => {
                self.convert_debugger_statement(estree)
            }
            EstreeNodeType::WithStatement => {
                self.convert_with_statement(estree)
            }
            _ => Err(ConversionError::UnsupportedNodeType {
                node_type: format!("{:?}", node_type),
                span: self.get_node_span(estree),
            }),
        }
    }

    /// Convert an ESTree WhileStatement to oxc Statement.
    fn convert_while_statement(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::Statement<'a>> {
        use oxc_ast::ast::Statement;

        // Get test
        self.context = self.context.clone().with_parent("WhileStatement", "test");
        let test_value = estree.get("test").ok_or_else(|| ConversionError::MissingField {
            field: "test".to_string(),
            node_type: "WhileStatement".to_string(),
            span: self.get_node_span(estree),
        })?;
        let test = self.convert_expression(test_value)?;

        // Get body
        self.context = self.context.clone().with_parent("WhileStatement", "body");
        let body_value = estree.get("body").ok_or_else(|| ConversionError::MissingField {
            field: "body".to_string(),
            node_type: "WhileStatement".to_string(),
            span: self.get_node_span(estree),
        })?;
        let body = self.convert_statement(body_value)?;

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        let while_stmt = self.builder.alloc_while_statement(span, test, body);
        Ok(Statement::WhileStatement(while_stmt))
    }

    /// Convert an ESTree ForStatement to oxc Statement.
    fn convert_for_statement(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::Statement<'a>> {
        use oxc_ast::ast::{Expression, ForStatementInit, Statement};
        use oxc_estree::deserialize::{EstreeNode, EstreeNodeType};

        // Get init (optional)
        let init = if let Some(init_value) = estree.get("init") {
            if init_value.is_null() {
                None
            } else {
                self.context = self.context.clone().with_parent("ForStatement", "init");
                let init_node_type = <Value as EstreeNode>::get_type(init_value).ok_or_else(|| ConversionError::MissingField {
                    field: "type".to_string(),
                    node_type: "init".to_string(),
                    span: self.get_node_span(estree),
                })?;

                Some(match init_node_type {
                    EstreeNodeType::VariableDeclaration => {
                        let var_decl_stmt = self.convert_variable_declaration(init_value)?;
                        match var_decl_stmt {
                            Statement::VariableDeclaration(vd) => {
                                ForStatementInit::VariableDeclaration(vd)
                            }
                            _ => unreachable!(),
                        }
                    }
                    _ => {
                        // Try as expression
                        let expr = self.convert_expression(init_value)?;
                        ForStatementInit::from(expr)
                    }
                })
            }
        } else {
            None
        };

        // Get test (optional)
        let test = if let Some(test_value) = estree.get("test") {
            if test_value.is_null() {
                None
            } else {
                self.context = self.context.clone().with_parent("ForStatement", "test");
                Some(self.convert_expression(test_value)?)
            }
        } else {
            None
        };

        // Get update (optional)
        let update = if let Some(update_value) = estree.get("update") {
            if update_value.is_null() {
                None
            } else {
                self.context = self.context.clone().with_parent("ForStatement", "update");
                Some(self.convert_expression(update_value)?)
            }
        } else {
            None
        };

        // Get body
        self.context = self.context.clone().with_parent("ForStatement", "body");
        let body_value = estree.get("body").ok_or_else(|| ConversionError::MissingField {
            field: "body".to_string(),
            node_type: "ForStatement".to_string(),
            span: self.get_node_span(estree),
        })?;
        let body = self.convert_statement(body_value)?;

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        let for_stmt = self.builder.alloc_for_statement(span, init, test, update, body);
        Ok(Statement::ForStatement(for_stmt))
    }

    /// Convert an ESTree BreakStatement to oxc Statement.
    fn convert_break_statement(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::Statement<'a>> {
        use oxc_ast::ast::Statement;
        use oxc_span::Atom;

        // Get label (optional)
        let label = if let Some(label_value) = estree.get("label") {
            if label_value.is_null() {
                None
            } else {
                self.context = self.context.clone().with_parent("BreakStatement", "label");
                let estree_id = oxc_estree::deserialize::EstreeIdentifier::from_json(label_value)
                    .ok_or_else(|| ConversionError::InvalidFieldType {
                        field: "label".to_string(),
                        expected: "valid Identifier node".to_string(),
                        got: format!("{:?}", label_value),
                        span: self.get_node_span(estree),
                    })?;

                let kind = oxc_estree::deserialize::convert_identifier(&estree_id, &self.context, self.source_text)?;
                if kind != oxc_estree::deserialize::IdentifierKind::Label {
                    return Err(ConversionError::InvalidIdentifierContext {
                        context: format!("Expected Label in BreakStatement.label, got {:?}", kind),
                        span: self.get_node_span(estree),
                    });
                }

                let name = Atom::from_in(estree_id.name.as_str(), self.builder.allocator);
                let range = estree_id.range.unwrap_or([0, 0]);
                let span = convert_span(self.source_text, range[0] as usize, range[1] as usize);
                Some(self.builder.label_identifier(span, name))
            }
        } else {
            None
        };

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        let break_stmt = self.builder.alloc_break_statement(span, label);
        Ok(Statement::BreakStatement(break_stmt))
    }

    /// Convert an ESTree ContinueStatement to oxc Statement.
    fn convert_continue_statement(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::Statement<'a>> {
        use oxc_ast::ast::Statement;
        use oxc_span::Atom;

        // Get label (optional)
        let label = if let Some(label_value) = estree.get("label") {
            if label_value.is_null() {
                None
            } else {
                self.context = self.context.clone().with_parent("ContinueStatement", "label");
                let estree_id = oxc_estree::deserialize::EstreeIdentifier::from_json(label_value)
                    .ok_or_else(|| ConversionError::InvalidFieldType {
                        field: "label".to_string(),
                        expected: "valid Identifier node".to_string(),
                        got: format!("{:?}", label_value),
                        span: self.get_node_span(estree),
                    })?;

                let kind = oxc_estree::deserialize::convert_identifier(&estree_id, &self.context, self.source_text)?;
                if kind != oxc_estree::deserialize::IdentifierKind::Label {
                    return Err(ConversionError::InvalidIdentifierContext {
                        context: format!("Expected Label in ContinueStatement.label, got {:?}", kind),
                        span: self.get_node_span(estree),
                    });
                }

                let name = Atom::from_in(estree_id.name.as_str(), self.builder.allocator);
                let range = estree_id.range.unwrap_or([0, 0]);
                let span = convert_span(self.source_text, range[0] as usize, range[1] as usize);
                Some(self.builder.label_identifier(span, name))
            }
        } else {
            None
        };

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        let continue_stmt = self.builder.alloc_continue_statement(span, label);
        Ok(Statement::ContinueStatement(continue_stmt))
    }

    /// Convert an ESTree DoWhileStatement to oxc Statement.
    fn convert_do_while_statement(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::Statement<'a>> {
        use oxc_ast::ast::Statement;

        // Get body
        self.context = self.context.clone().with_parent("DoWhileStatement", "body");
        let body_value = estree.get("body").ok_or_else(|| ConversionError::MissingField {
            field: "body".to_string(),
            node_type: "DoWhileStatement".to_string(),
            span: self.get_node_span(estree),
        })?;
        let body = self.convert_statement(body_value)?;

        // Get test
        self.context = self.context.clone().with_parent("DoWhileStatement", "test");
        let test_value = estree.get("test").ok_or_else(|| ConversionError::MissingField {
            field: "test".to_string(),
            node_type: "DoWhileStatement".to_string(),
            span: self.get_node_span(estree),
        })?;
        let test = self.convert_expression(test_value)?;

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        let do_while_stmt = self.builder.alloc_do_while_statement(span, body, test);
        Ok(Statement::DoWhileStatement(do_while_stmt))
    }

    /// Convert an ESTree ForInStatement to oxc Statement.
    fn convert_for_in_statement(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::Statement<'a>> {
        use oxc_ast::ast::{Expression, ForStatementLeft, Statement};
        use oxc_estree::deserialize::{EstreeNode, EstreeNodeType};

        // Get left (can be VariableDeclaration or AssignmentTarget)
        let left_value = estree.get("left").ok_or_else(|| ConversionError::MissingField {
            field: "left".to_string(),
            node_type: "ForInStatement".to_string(),
            span: self.get_node_span(estree),
        })?;

        // Set context before processing left
        self.context = self.context.clone().with_parent("ForInStatement", "left");
        // get_type doesn't need context, but we need to check the type to determine how to convert
        // Debug: check if left_value is actually a JSON object
        if !left_value.is_object() {
            return Err(ConversionError::InvalidFieldType {
                field: "left".to_string(),
                expected: "object".to_string(),
                got: format!("{:?}", left_value),
                span: self.get_node_span(estree),
            });
        }
        // Debug: manually check the type field and use get_type for consistency
        // First, manually extract the type string to see what we have
        let type_str_manual = left_value.get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("none");
        
        // Now try get_type
        let left_node_type = <Value as EstreeNode>::get_type(left_value);
        
        // If get_type returns None, that's the problem
        let left_node_type = left_node_type.ok_or_else(|| {
            ConversionError::MissingField {
                field: "type".to_string(),
                node_type: format!("ForInStatement.left (get_type returned None, but manual type_str: {})", type_str_manual),
                span: self.get_node_span(left_value),
            }
        })?;

        let left = match left_node_type {
            EstreeNodeType::VariableDeclaration => {
                // Context already set above
                let var_decl_stmt = self.convert_variable_declaration(left_value)?;
                match var_decl_stmt {
                    Statement::VariableDeclaration(vd) => {
                        ForStatementLeft::VariableDeclaration(vd)
                    }
                    _ => unreachable!(),
                }
            }
            other_type => {
                // This should not happen for our test case
                return Err(ConversionError::UnsupportedNodeType {
                    node_type: format!("ForInStatement.left expected VariableDeclaration, got {:?} (manual type_str: {})", other_type, type_str_manual),
                    span: self.get_node_span(left_value),
                });
            }
        };

        // Get right
        self.context = self.context.clone().with_parent("ForInStatement", "right");
        let right_value = estree.get("right").ok_or_else(|| ConversionError::MissingField {
            field: "right".to_string(),
            node_type: "ForInStatement".to_string(),
            span: self.get_node_span(estree),
        })?;
        let right = self.convert_expression(right_value)?;

        // Get body
        self.context = self.context.clone().with_parent("ForInStatement", "body");
        let body_value = estree.get("body").ok_or_else(|| ConversionError::MissingField {
            field: "body".to_string(),
            node_type: "ForInStatement".to_string(),
            span: self.get_node_span(estree),
        })?;
        // Debug: check if body_value is actually a JSON object
        if !body_value.is_object() {
            return Err(ConversionError::InvalidFieldType {
                field: "body".to_string(),
                expected: "object".to_string(),
                got: format!("{:?}", body_value),
                span: self.get_node_span(estree),
            });
        }
        let body = self.convert_statement(body_value)?;

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        let for_in_stmt = self.builder.alloc_for_in_statement(span, left, right, body);
        Ok(Statement::ForInStatement(for_in_stmt))
    }

    /// Convert an ESTree ForOfStatement to oxc Statement.
    fn convert_for_of_statement(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::Statement<'a>> {
        use oxc_ast::ast::{Expression, ForStatementLeft, Statement};
        use oxc_estree::deserialize::{EstreeNode, EstreeNodeType};

        // Get left (can be VariableDeclaration or AssignmentTarget)
        let left_value = estree.get("left").ok_or_else(|| ConversionError::MissingField {
            field: "left".to_string(),
            node_type: "ForOfStatement".to_string(),
            span: self.get_node_span(estree),
        })?;

        let left_node_type = <Value as EstreeNode>::get_type(left_value).ok_or_else(|| ConversionError::MissingField {
            field: "type".to_string(),
            node_type: "left".to_string(),
            span: self.get_node_span(estree),
        })?;

        let left = match left_node_type {
            EstreeNodeType::VariableDeclaration => {
                self.context = self.context.clone().with_parent("ForOfStatement", "left");
                let var_decl_stmt = self.convert_variable_declaration(left_value)?;
                match var_decl_stmt {
                    Statement::VariableDeclaration(vd) => {
                        ForStatementLeft::VariableDeclaration(vd)
                    }
                    _ => unreachable!(),
                }
            }
            _ => {
                // Try as AssignmentTarget
                self.context = self.context.clone().with_parent("ForOfStatement", "left");
                let assignment_target = self.convert_to_assignment_target(left_value)?;
                ForStatementLeft::from(assignment_target)
            }
        };

        // Get right
        self.context = self.context.clone().with_parent("ForOfStatement", "right");
        let right_value = estree.get("right").ok_or_else(|| ConversionError::MissingField {
            field: "right".to_string(),
            node_type: "ForOfStatement".to_string(),
            span: self.get_node_span(estree),
        })?;
        let right = self.convert_expression(right_value)?;

        // Get body
        self.context = self.context.clone().with_parent("ForOfStatement", "body");
        let body_value = estree.get("body").ok_or_else(|| ConversionError::MissingField {
            field: "body".to_string(),
            node_type: "ForOfStatement".to_string(),
            span: self.get_node_span(estree),
        })?;
        let body = self.convert_statement(body_value)?;

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        let await_token = estree.get("await").and_then(|v| v.as_bool()).unwrap_or(false);
        let for_of_stmt = self.builder.alloc_for_of_statement(span, await_token, left, right, body);
        Ok(Statement::ForOfStatement(for_of_stmt))
    }

    /// Convert an ESTree EmptyStatement to oxc Statement.
    fn convert_empty_statement(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::Statement<'a>> {
        use oxc_ast::ast::Statement;

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        let empty_stmt = self.builder.alloc_empty_statement(span);
        Ok(Statement::EmptyStatement(empty_stmt))
    }

    /// Convert an ESTree SwitchStatement to oxc Statement.
    fn convert_switch_statement(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::Statement<'a>> {
        use oxc_ast::ast::{Expression, Statement};

        // Get discriminant
        self.context = self.context.clone().with_parent("SwitchStatement", "discriminant");
        let discriminant_value = estree.get("discriminant").ok_or_else(|| ConversionError::MissingField {
            field: "discriminant".to_string(),
            node_type: "SwitchStatement".to_string(),
            span: self.get_node_span(estree),
        })?;
        let discriminant = self.convert_expression(discriminant_value)?;

        // Get cases
        let cases_value = estree.get("cases").ok_or_else(|| ConversionError::MissingField {
            field: "cases".to_string(),
            node_type: "SwitchStatement".to_string(),
            span: self.get_node_span(estree),
        })?;

        let cases_array = cases_value.as_array().ok_or_else(|| ConversionError::InvalidFieldType {
            field: "cases".to_string(),
            expected: "array".to_string(),
            got: format!("{:?}", cases_value),
            span: self.get_node_span(estree),
        })?;

        let mut switch_cases = Vec::new_in(self.builder.allocator);
        for case_value in cases_array {
            self.context = self.context.clone().with_parent("SwitchStatement", "cases");
            let switch_case = self.convert_switch_case(case_value)?;
            switch_cases.push(switch_case);
        }

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        let switch_stmt = self.builder.alloc_switch_statement(span, discriminant, switch_cases);
        Ok(Statement::SwitchStatement(switch_stmt))
    }

    /// Convert an ESTree SwitchCase to oxc SwitchCase.
    fn convert_switch_case(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::SwitchCase<'a>> {
        use oxc_ast::ast::{Expression, Statement};
        use oxc_estree::deserialize::{EstreeNode, EstreeNodeType};

        // Get test (optional - null for default case)
        let test = if let Some(test_value) = estree.get("test") {
            if test_value.is_null() {
                None
            } else {
                self.context = self.context.clone().with_parent("SwitchCase", "test");
                Some(self.convert_expression(test_value)?)
            }
        } else {
            None
        };

        // Get consequent
        let consequent_value = estree.get("consequent").ok_or_else(|| ConversionError::MissingField {
            field: "consequent".to_string(),
            node_type: "SwitchCase".to_string(),
            span: self.get_node_span(estree),
        })?;

        let consequent_array = consequent_value.as_array().ok_or_else(|| ConversionError::InvalidFieldType {
            field: "consequent".to_string(),
            expected: "array".to_string(),
            got: format!("{:?}", consequent_value),
            span: self.get_node_span(estree),
        })?;

        let mut statements = Vec::new_in(self.builder.allocator);
        for stmt_value in consequent_array {
            self.context = self.context.clone().with_parent("SwitchCase", "consequent");
            let statement = self.convert_statement(stmt_value)?;
            statements.push(statement);
        }

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        let switch_case = self.builder.switch_case(span, test, statements);
        Ok(switch_case)
    }

    /// Convert an ESTree TryStatement to oxc Statement.
    fn convert_try_statement(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::Statement<'a>> {
        use oxc_ast::ast::Statement;

        // Get block
        self.context = self.context.clone().with_parent("TryStatement", "block");
        let block_value = estree.get("block").ok_or_else(|| ConversionError::MissingField {
            field: "block".to_string(),
            node_type: "TryStatement".to_string(),
            span: self.get_node_span(estree),
        })?;
        let block_stmt = self.convert_block_statement(block_value)?;
        let block = match block_stmt {
            Statement::BlockStatement(bs) => bs,
            _ => return Err(ConversionError::InvalidFieldType {
                field: "block".to_string(),
                expected: "BlockStatement".to_string(),
                got: format!("{:?}", block_stmt),
                span: self.get_node_span(estree),
            }),
        };

        // Get handler (optional)
        let handler = if let Some(handler_value) = estree.get("handler") {
            if handler_value.is_null() {
                None
            } else {
                self.context = self.context.clone().with_parent("TryStatement", "handler");
                Some(self.convert_catch_clause(handler_value)?)
            }
        } else {
            None
        };

        // Get finalizer (optional)
        let finalizer = if let Some(finalizer_value) = estree.get("finalizer") {
            if finalizer_value.is_null() {
                None
            } else {
                self.context = self.context.clone().with_parent("TryStatement", "finalizer");
                let finalizer_stmt = self.convert_block_statement(finalizer_value)?;
                let finalizer_block = match finalizer_stmt {
                    Statement::BlockStatement(bs) => bs,
                    _ => return Err(ConversionError::InvalidFieldType {
                        field: "finalizer".to_string(),
                        expected: "BlockStatement".to_string(),
                        got: format!("{:?}", finalizer_stmt),
                        span: self.get_node_span(estree),
                    }),
                };
                Some(finalizer_block)
            }
        } else {
            None
        };

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        let try_stmt = self.builder.alloc_try_statement(span, block, handler, finalizer);
        Ok(Statement::TryStatement(try_stmt))
    }

    /// Convert an ESTree CatchClause to oxc CatchClause.
    fn convert_catch_clause(&mut self, estree: &Value) -> ConversionResult<oxc_allocator::Box<'a, oxc_ast::ast::CatchClause<'a>>> {
        use oxc_ast::ast::{BindingPattern, CatchParameter, Statement};

        // Get param (optional)
        let param = if let Some(param_value) = estree.get("param") {
            if param_value.is_null() {
                None
            } else {
                self.context = self.context.clone().with_parent("CatchClause", "param");
                let binding_pattern = self.convert_binding_pattern(param_value)?;
                let (start, end) = self.get_node_span(param_value);
                let span = Span::new(start, end);
                Some(self.builder.catch_parameter(span, binding_pattern))
            }
        } else {
            None
        };

        // Get body
        self.context = self.context.clone().with_parent("CatchClause", "body");
        let body_value = estree.get("body").ok_or_else(|| ConversionError::MissingField {
            field: "body".to_string(),
            node_type: "CatchClause".to_string(),
            span: self.get_node_span(estree),
        })?;
        let body_stmt = self.convert_block_statement(body_value)?;
        let body = match body_stmt {
            Statement::BlockStatement(bs) => bs,
            _ => return Err(ConversionError::InvalidFieldType {
                field: "body".to_string(),
                expected: "BlockStatement".to_string(),
                got: format!("{:?}", body_stmt),
                span: self.get_node_span(estree),
            }),
        };

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        let catch_clause = self.builder.alloc_catch_clause(span, param, body);
        Ok(catch_clause)
    }

    /// Convert an ESTree SpreadElement to oxc SpreadElement.
    fn convert_spread_element(&mut self, estree: &Value) -> ConversionResult<oxc_allocator::Box<'a, oxc_ast::ast::SpreadElement<'a>>> {
        use oxc_ast::ast::Expression;

        // Get argument
        self.context = self.context.clone().with_parent("SpreadElement", "argument");
        let argument_value = estree.get("argument").ok_or_else(|| ConversionError::MissingField {
            field: "argument".to_string(),
            node_type: "SpreadElement".to_string(),
            span: self.get_node_span(estree),
        })?;
        let argument = self.convert_expression(argument_value)?;

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        let spread = self.builder.alloc_spread_element(span, argument);
        Ok(spread)
    }

    /// Convert an ESTree LabeledStatement to oxc Statement.
    fn convert_labeled_statement(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::Statement<'a>> {
        use oxc_ast::ast::Statement;
        use oxc_span::Atom;

        // Get label
        self.context = self.context.clone().with_parent("LabeledStatement", "label");
        let label_value = estree.get("label").ok_or_else(|| ConversionError::MissingField {
            field: "label".to_string(),
            node_type: "LabeledStatement".to_string(),
            span: self.get_node_span(estree),
        })?;

        let estree_id = oxc_estree::deserialize::EstreeIdentifier::from_json(label_value)
            .ok_or_else(|| ConversionError::InvalidFieldType {
                field: "label".to_string(),
                expected: "valid Identifier node".to_string(),
                got: format!("{:?}", label_value),
                span: self.get_node_span(estree),
            })?;

        let kind = oxc_estree::deserialize::convert_identifier(&estree_id, &self.context, self.source_text)?;
        if kind != oxc_estree::deserialize::IdentifierKind::Label {
            return Err(ConversionError::InvalidIdentifierContext {
                context: format!("Expected Label in LabeledStatement.label, got {:?}", kind),
                span: self.get_node_span(estree),
            });
        }

        let name = Atom::from_in(estree_id.name.as_str(), self.builder.allocator);
        let range = estree_id.range.unwrap_or([0, 0]);
        let label_span = convert_span(self.source_text, range[0] as usize, range[1] as usize);
        let label = self.builder.label_identifier(label_span, name);

        // Get body
        self.context = self.context.clone().with_parent("LabeledStatement", "body");
        let body_value = estree.get("body").ok_or_else(|| ConversionError::MissingField {
            field: "body".to_string(),
            node_type: "LabeledStatement".to_string(),
            span: self.get_node_span(estree),
        })?;
        let body = self.convert_statement(body_value)?;

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        let labeled_stmt = self.builder.alloc_labeled_statement(span, label, body);
        Ok(Statement::LabeledStatement(labeled_stmt))
    }

    /// Convert an ESTree ThrowStatement to oxc Statement.
    fn convert_throw_statement(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::Statement<'a>> {
        use oxc_ast::ast::Statement;

        // Get argument
        self.context = self.context.clone().with_parent("ThrowStatement", "argument");
        let argument_value = estree.get("argument").ok_or_else(|| ConversionError::MissingField {
            field: "argument".to_string(),
            node_type: "ThrowStatement".to_string(),
            span: self.get_node_span(estree),
        })?;
        let argument = self.convert_expression(argument_value)?;

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        let throw_stmt = self.builder.alloc_throw_statement(span, argument);
        Ok(Statement::ThrowStatement(throw_stmt))
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

        // Verify we have a VariableDeclaration node
        let node_type = <Value as EstreeNode>::get_type(estree).ok_or_else(|| ConversionError::MissingField {
            field: "type".to_string(),
            node_type: "VariableDeclaration".to_string(),
            span: self.get_node_span(estree),
        })?;
        if node_type != EstreeNodeType::VariableDeclaration {
            return Err(ConversionError::InvalidFieldType {
                field: "type".to_string(),
                expected: "VariableDeclaration".to_string(),
                got: format!("{:?}", node_type),
                span: self.get_node_span(estree),
            });
        }

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

        // Get init (optional) - reset binding context as init is an expression, not a binding
        let init = if let Some(init_value) = estree.get("init") {
            // Skip null values
            if init_value.is_null() {
                None
            } else {
                let mut init_context = self.context.clone().with_parent("VariableDeclarator", "init");
                init_context.is_binding_context = false; // init is an expression, not a binding
                self.context = init_context;
                Some(self.convert_expression(init_value)?)
            }
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

        // Debug: check if estree is actually a JSON object
        if !estree.is_object() {
            return Err(ConversionError::InvalidFieldType {
                field: "expression".to_string(),
                expected: "object".to_string(),
                got: format!("{:?}", estree),
                span: self.get_node_span(estree),
            });
        }
        let node_type = <Value as EstreeNode>::get_type(estree).ok_or_else(|| {
            // Debug: check what we actually have
            let has_type = estree.get("type").is_some();
            let type_str = estree.get("type").and_then(|v| v.as_str()).unwrap_or("none");
            ConversionError::MissingField {
                field: "type".to_string(),
                node_type: format!("convert_expression (is_object: {}, has_type: {}, type: {})", estree.is_object(), has_type, type_str),
                span: self.get_node_span(estree),
            }
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
            EstreeNodeType::ThisExpression => {
                self.convert_this_expression(estree)
            }
            EstreeNodeType::NewExpression => {
                self.convert_new_expression(estree)
            }
            EstreeNodeType::AwaitExpression => {
                self.convert_await_expression(estree)
            }
            EstreeNodeType::YieldExpression => {
                self.convert_yield_expression(estree)
            }
            EstreeNodeType::Super => {
                self.convert_super_expression(estree)
            }
            EstreeNodeType::TemplateLiteral => {
                self.convert_template_literal(estree)
            }
            EstreeNodeType::TaggedTemplateExpression => {
                self.convert_tagged_template_expression(estree)
            }
            EstreeNodeType::ArrowFunctionExpression => {
                self.convert_arrow_function_expression(estree)
            }
            EstreeNodeType::FunctionExpression => {
                self.convert_function_expression(estree)
            }
            EstreeNodeType::ClassExpression => {
                self.convert_class_expression(estree)
            }
            EstreeNodeType::ImportExpression => {
                self.convert_import_expression(estree)
            }
            EstreeNodeType::MetaProperty => {
                self.convert_meta_property(estree)
            }
            EstreeNodeType::TSAsExpression => {
                self.convert_ts_as_expression(estree)
            }
            EstreeNodeType::TSSatisfiesExpression => {
                self.convert_ts_satisfies_expression(estree)
            }
            EstreeNodeType::TSNonNullExpression => {
                self.convert_ts_non_null_expression(estree)
            }
            EstreeNodeType::TSInstantiationExpression => {
                self.convert_ts_instantiation_expression(estree)
            }
            EstreeNodeType::TSTypeAssertion => {
                self.convert_ts_type_assertion(estree)
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

        // Check for RegExp first (regex property is at top level of Literal node, not in value)
        if estree.get("regex").is_some() {
            // Handle RegExp literal
            let regex_value = estree.get("regex")
                .ok_or_else(|| ConversionError::MissingField {
                    field: "regex".to_string(),
                    node_type: "RegExpLiteral".to_string(),
                    span: self.get_node_span(estree),
                })?;
            
            let pattern_str = regex_value.get("pattern")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ConversionError::MissingField {
                    field: "regex.pattern".to_string(),
                    node_type: "RegExpLiteral".to_string(),
                    span: self.get_node_span(estree),
                })?;
            
            let flags_str = regex_value.get("flags")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            
            // Parse flags string into RegExpFlags
            use oxc_ast::ast::RegExpFlags;
            let mut flags = RegExpFlags::empty();
            for ch in flags_str.chars() {
                if let Ok(flag) = RegExpFlags::try_from(ch) {
                    flags |= flag;
                }
                // Ignore invalid flags (non-fatal)
            }
            
            // Create RegExpPattern
            let pattern_atom = Atom::from_in(pattern_str, self.builder.allocator);
            let pattern = oxc_ast::ast::RegExpPattern {
                text: pattern_atom,
                pattern: None, // Don't parse the pattern here (can be done later if needed)
            };
            
            // Create RegExp
            let regex = oxc_ast::ast::RegExp {
                pattern,
                flags,
            };
            
            // Get raw value (the literal as it appears in source, e.g., "/pattern/flags")
            let raw = estree_literal.raw.as_ref().map(|s| {
                Atom::from_in(s.as_str(), self.builder.allocator)
            });
            
            return Ok(Expression::RegExpLiteral(self.builder.alloc_reg_exp_literal(span, regex, raw)));
        }

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
            LiteralKind::BigInt => {
                // BigIntLiteral: 123n
                // ESTree represents BigInt as a string value ending with 'n'
                let value_str = get_string_value(&estree_literal)?;
                // Remove the trailing 'n' to get the numeric part
                let numeric_str = value_str.strip_suffix('n')
                    .ok_or_else(|| ConversionError::InvalidFieldType {
                        field: "value".to_string(),
                        expected: "string ending with 'n'".to_string(),
                        got: value_str.to_string(),
                        span: self.get_node_span(estree),
                    })?;
                
                let value_atom = Atom::from_in(numeric_str, self.builder.allocator);
                let raw = estree_literal.raw.as_ref().map(|s| {
                    Atom::from_in(s.as_str(), self.builder.allocator)
                });
                
                // Determine base from raw value (default to Decimal)
                use oxc_syntax::number::BigintBase;
                let base = if let Some(raw_str) = estree_literal.raw.as_ref() {
                    if raw_str.starts_with("0x") || raw_str.starts_with("0X") {
                        BigintBase::Hex
                    } else if raw_str.starts_with("0o") || raw_str.starts_with("0O") {
                        BigintBase::Octal
                    } else if raw_str.starts_with("0b") || raw_str.starts_with("0B") {
                        BigintBase::Binary
                    } else {
                        BigintBase::Decimal
                    }
                } else {
                    BigintBase::Decimal
                };
                
                Ok(Expression::BigIntLiteral(self.builder.alloc_big_int_literal(span, value_atom, raw, base)))
            }
            _ => Err(ConversionError::UnsupportedNodeType {
                node_type: format!("Unsupported literal kind"),
                span: self.get_node_span(estree),
            }),
        }
    }

    /// Convert an ESTree Directive to oxc Directive.
    /// A directive in ESTree is typically an ExpressionStatement with a StringLiteral expression.
    fn convert_directive(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::Directive<'a>> {
        use oxc_ast::ast::StringLiteral;
        use oxc_estree::deserialize::{EstreeNode, EstreeNodeType};
        use oxc_span::Atom;

        let node_type = <Value as EstreeNode>::get_type(estree)
            .ok_or_else(|| ConversionError::MissingField {
                field: "type".to_string(),
                node_type: "Directive".to_string(),
                span: self.get_node_span(estree),
            })?;

        // Directives are typically ExpressionStatements with StringLiteral expressions
        if node_type == EstreeNodeType::ExpressionStatement {
            let expression_value = estree.get("expression").ok_or_else(|| ConversionError::MissingField {
                field: "expression".to_string(),
                node_type: "Directive (ExpressionStatement)".to_string(),
                span: self.get_node_span(estree),
            })?;

            let expr_type = <Value as EstreeNode>::get_type(expression_value)
                .ok_or_else(|| ConversionError::MissingField {
                    field: "expression.type".to_string(),
                    node_type: "Directive".to_string(),
                    span: self.get_node_span(estree),
                })?;

            if expr_type == EstreeNodeType::Literal {
                // Convert the literal to get the string value
                let literal_expr = self.convert_literal_to_expression(expression_value)?;
                if let oxc_ast::ast::Expression::StringLiteral(string_lit_box) = literal_expr {
                    let (start, end) = self.get_node_span(estree);
                    let span = convert_span(self.source_text, start as usize, end as usize);
                    
                    // Clone the StringLiteral from the Box (we can't move out of Box)
                    let string_lit = (*string_lit_box).clone();
                    
                    // Get the raw directive value (as it appears in source)
                    let directive_value = string_lit.value.as_str();
                    let directive_atom = Atom::from_in(directive_value, self.builder.allocator);
                    
                    // Create the directive
                    let directive = self.builder.directive(span, string_lit, directive_atom);
                    Ok(directive)
                } else {
                    Err(ConversionError::InvalidFieldType {
                        field: "expression".to_string(),
                        expected: "StringLiteral".to_string(),
                        got: format!("{:?}", literal_expr),
                        span: self.get_node_span(estree),
                    })
                }
            } else {
                Err(ConversionError::InvalidFieldType {
                    field: "expression".to_string(),
                    expected: "Literal (StringLiteral)".to_string(),
                    got: format!("{:?}", expr_type),
                    span: self.get_node_span(estree),
                })
            }
        } else {
            Err(ConversionError::UnsupportedNodeType {
                node_type: format!("Directive must be ExpressionStatement, got {:?}", node_type),
                span: self.get_node_span(estree),
            })
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

    /// Convert an ESTree node to oxc Argument.
    /// Handles both regular expressions and SpreadElement.
    fn convert_to_argument(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::Argument<'a>> {
        use oxc_ast::ast::Argument;
        use oxc_estree::deserialize::{EstreeNode, EstreeNodeType};

        let node_type = <Value as EstreeNode>::get_type(estree)
            .ok_or_else(|| ConversionError::MissingField {
                field: "type".to_string(),
                node_type: "convert_to_argument".to_string(),
                span: self.get_node_span(estree),
            })?;

        match node_type {
            EstreeNodeType::SpreadElement => {
                // Get argument (the expression being spread)
                self.context = self.context.clone().with_parent("SpreadElement", "argument");
                let argument_value = estree.get("argument").ok_or_else(|| ConversionError::MissingField {
                    field: "argument".to_string(),
                    node_type: "SpreadElement".to_string(),
                    span: self.get_node_span(estree),
                })?;
                let argument_expr = self.convert_expression(argument_value)?;
                
                let (start, end) = self.get_node_span(estree);
                let span = Span::new(start, end);
                
                Ok(Argument::SpreadElement(self.builder.alloc_spread_element(span, argument_expr)))
            }
            _ => {
                // Regular expression argument
                let expr = self.convert_expression(estree)?;
                Ok(Argument::from(expr))
            }
        }
    }

    /// Convert an ESTree parameter node to oxc FormalParameter.
    /// ESTree parameters can be Identifier, Pattern, or RestElement.
    fn convert_to_formal_parameter(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::FormalParameter<'a>> {
        use oxc_ast::ast::FormalParameter;
        
        // Convert the parameter as a BindingPattern
        // (FormalParameter is essentially a BindingPattern with optional decorators/modifiers)
        let pattern = self.convert_binding_pattern(estree)?;
        
        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);
        
        // Get decorators (optional array of Decorator)
        let decorators = if let Some(decorators_value) = estree.get("decorators") {
            if decorators_value.is_null() {
                Vec::new_in(self.builder.allocator)
            } else {
                let decorators_array = decorators_value.as_array().ok_or_else(|| ConversionError::InvalidFieldType {
                    field: "decorators".to_string(),
                    expected: "array".to_string(),
                    got: format!("{:?}", decorators_value),
                    span: self.get_node_span(estree),
                })?;
                
                let mut decorators_vec = Vec::new_in(self.builder.allocator);
                for decorator_value in decorators_array {
                    self.context = self.context.clone().with_parent("FormalParameter", "decorators");
                    let decorator = self.convert_decorator(decorator_value)?;
                    decorators_vec.push(decorator);
                }
                decorators_vec
            }
        } else {
            Vec::new_in(self.builder.allocator)
        };
        
        // Get accessibility (TypeScript)
        let accessibility: Option<oxc_ast::ast::TSAccessibility> = if let Some(accessibility_str) = estree.get("accessibility").and_then(|v| v.as_str()) {
            match accessibility_str {
                "public" => Some(oxc_ast::ast::TSAccessibility::Public),
                "private" => Some(oxc_ast::ast::TSAccessibility::Private),
                "protected" => Some(oxc_ast::ast::TSAccessibility::Protected),
                _ => None,
            }
        } else {
            None
        };
        let readonly = estree.get("readonly").and_then(|v| v.as_bool()).unwrap_or(false);
        let r#override = estree.get("override").and_then(|v| v.as_bool()).unwrap_or(false);
        
        Ok(self.builder.formal_parameter(span, decorators, pattern, accessibility, readonly, r#override))
    }

    /// Convert an ESTree RestElement to oxc BindingRestElement.
    fn convert_rest_element_to_binding_rest(&mut self, estree: &Value) -> ConversionResult<oxc_allocator::Box<'a, oxc_ast::ast::BindingRestElement<'a>>> {
        use oxc_ast::ast::BindingRestElement;
        
        // Get argument (the pattern being rest) - rest element arguments are always bindings
        let mut rest_context = self.context.clone().with_parent("RestElement", "argument");
        rest_context.is_binding_context = true;
        self.context = rest_context;
        let argument_value = estree.get("argument").ok_or_else(|| ConversionError::MissingField {
            field: "argument".to_string(),
            node_type: "RestElement".to_string(),
            span: self.get_node_span(estree),
        })?;
        let argument = self.convert_binding_pattern(argument_value)?;
        
        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);
        
        // BindingRestElement doesn't have type annotation in oxc AST
        // (type annotations are on the BindingPattern itself)
        Ok(self.builder.alloc_binding_rest_element(span, argument))
    }

    /// Convert an ESTree Pattern to oxc BindingPattern.
    fn convert_binding_pattern(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::BindingPattern<'a>> {
        use oxc_ast::ast::BindingPattern;
        use oxc_estree::deserialize::{EstreeNode, EstreeNodeType};

        // Debug: check if estree is actually a JSON object
        if !estree.is_object() {
            return Err(ConversionError::InvalidFieldType {
                field: "binding_pattern".to_string(),
                expected: "object".to_string(),
                got: format!("{:?}", estree),
                span: self.get_node_span(estree),
            });
        }
        let node_type = <Value as EstreeNode>::get_type(estree).ok_or_else(|| {
            // Debug: check what we actually have
            let has_type = estree.get("type").is_some();
            let type_str = estree.get("type").and_then(|v| v.as_str()).unwrap_or("none");
            ConversionError::MissingField {
                field: "type".to_string(),
                node_type: format!("convert_binding_pattern (is_object: {}, has_type: {}, type: {})", estree.is_object(), has_type, type_str),
                span: self.get_node_span(estree),
            }
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
            EstreeNodeType::ObjectPattern => {
                self.convert_object_pattern_to_binding_pattern(estree)
            }
            EstreeNodeType::ArrayPattern => {
                self.convert_array_pattern_to_binding_pattern(estree)
            }
            EstreeNodeType::RestElement => {
                // RestElement in binding context should be converted to BindingRestElement
                // But RestElement itself is not a BindingPattern, it contains one
                // This case should not be reached in convert_binding_pattern
                Err(ConversionError::InvalidFieldType {
                    field: "pattern".to_string(),
                    expected: "Identifier|ObjectPattern|ArrayPattern|AssignmentPattern".to_string(),
                    got: "RestElement".to_string(),
                    span: self.get_node_span(estree),
                })
            }
            EstreeNodeType::AssignmentPattern => {
                self.convert_assignment_pattern_to_binding_pattern(estree)
            }
            _ => Err(ConversionError::UnsupportedNodeType {
                node_type: format!("{:?}", node_type),
                span: self.get_node_span(estree),
            }),
        }
    }

    /// Convert an ESTree ObjectPattern to oxc BindingPattern.
    fn convert_object_pattern_to_binding_pattern(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::BindingPattern<'a>> {
        use oxc_ast::ast::{BindingPattern, BindingPatternKind};

        // Get properties
        let properties_value = estree.get("properties").ok_or_else(|| ConversionError::MissingField {
            field: "properties".to_string(),
            node_type: "ObjectPattern".to_string(),
            span: self.get_node_span(estree),
        })?;

        let properties_array = properties_value.as_array().ok_or_else(|| ConversionError::InvalidFieldType {
            field: "properties".to_string(),
            expected: "array".to_string(),
            got: format!("{:?}", properties_value),
            span: self.get_node_span(estree),
        })?;

        let mut binding_properties = Vec::new_in(self.builder.allocator);
        let mut rest: Option<oxc_allocator::Box<'a, oxc_ast::ast::BindingRestElement<'a>>> = None;
        
        // In ESTree, RestElement appears as the last element in the properties array
        for (idx, prop_value) in properties_array.iter().enumerate() {
            let mut prop_context = self.context.clone().with_parent("ObjectPattern", "properties");
            prop_context.is_binding_context = true; // Properties in ObjectPattern are always bindings
            self.context = prop_context;
            
            // Check if this is a RestElement (last element)
            use oxc_estree::deserialize::{EstreeNode, EstreeNodeType};
            let is_rest = idx == properties_array.len() - 1 
                && <Value as EstreeNode>::get_type(prop_value) == Some(EstreeNodeType::RestElement);
            
            if is_rest {
                let mut rest_context = self.context.clone().with_parent("ObjectPattern", "rest");
                rest_context.is_binding_context = true;
                self.context = rest_context;
                rest = Some(self.convert_rest_element(prop_value)?);
            } else {
                let binding_prop = self.convert_binding_property(prop_value)?;
                binding_properties.push(binding_prop);
            }
        }

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        let object_pattern = self.builder.object_pattern(span, binding_properties, rest);
        let object_pattern_box = oxc_allocator::Box::new_in(object_pattern, self.builder.allocator);
        let pattern = self.builder.binding_pattern(BindingPatternKind::ObjectPattern(object_pattern_box), None::<oxc_ast::ast::TSTypeAnnotation>, false);
        Ok(pattern)
    }

    /// Convert an ESTree ArrayPattern to oxc BindingPattern.
    fn convert_array_pattern_to_binding_pattern(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::BindingPattern<'a>> {
        use oxc_ast::ast::{BindingPattern, BindingPatternKind};

        // Get elements
        let elements_value = estree.get("elements").ok_or_else(|| ConversionError::MissingField {
            field: "elements".to_string(),
            node_type: "ArrayPattern".to_string(),
            span: self.get_node_span(estree),
        })?;

        let elements_array = elements_value.as_array().ok_or_else(|| ConversionError::InvalidFieldType {
            field: "elements".to_string(),
            expected: "array".to_string(),
            got: format!("{:?}", elements_value),
            span: self.get_node_span(estree),
        })?;

        let mut binding_elements = Vec::new_in(self.builder.allocator);
        let mut rest: Option<oxc_allocator::Box<'a, oxc_ast::ast::BindingRestElement<'a>>> = None;
        
        // In ESTree, RestElement appears as the last element in the elements array
        for (idx, elem_value) in elements_array.iter().enumerate() {
            self.context = self.context.clone().with_parent("ArrayPattern", "elements");
            
            if elem_value.is_null() {
                // Sparse array - None
                binding_elements.push(None);
            } else {
                // Check if this is a RestElement (last non-null element)
                use oxc_estree::deserialize::{EstreeNode, EstreeNodeType};
                let is_rest = <Value as EstreeNode>::get_type(elem_value) == Some(EstreeNodeType::RestElement)
                    && elements_array.iter().skip(idx + 1).all(|v| v.is_null());
                
                if is_rest {
                    let mut rest_context = self.context.clone().with_parent("ArrayPattern", "rest");
                    rest_context.is_binding_context = true;
                    self.context = rest_context;
                    rest = Some(self.convert_rest_element(elem_value)?);
                } else {
                    let binding_pattern = self.convert_binding_pattern(elem_value)?;
                    binding_elements.push(Some(binding_pattern));
                }
            }
        }

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        let array_pattern = self.builder.array_pattern(span, binding_elements, rest);
        let array_pattern_box = oxc_allocator::Box::new_in(array_pattern, self.builder.allocator);
        let pattern = self.builder.binding_pattern(BindingPatternKind::ArrayPattern(array_pattern_box), None::<oxc_ast::ast::TSTypeAnnotation>, false);
        Ok(pattern)
    }

    /// Convert an ESTree RestElement to oxc BindingRestElement.
    fn convert_rest_element(&mut self, estree: &Value) -> ConversionResult<oxc_allocator::Box<'a, oxc_ast::ast::BindingRestElement<'a>>> {
        use oxc_estree::deserialize::EstreeNode;

        // Get argument (must be a BindingPattern)
        self.context = self.context.clone().with_parent("RestElement", "argument");
        let argument_value = estree.get("argument").ok_or_else(|| ConversionError::MissingField {
            field: "argument".to_string(),
            node_type: "RestElement".to_string(),
            span: self.get_node_span(estree),
        })?;

        let argument_pattern = self.convert_binding_pattern(argument_value)?;

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        let rest_element = self.builder.alloc_binding_rest_element(span, argument_pattern);
        Ok(rest_element)
    }

    /// Convert an ESTree AssignmentPattern to oxc BindingPattern.
    fn convert_assignment_pattern_to_binding_pattern(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::BindingPattern<'a>> {
        use oxc_ast::ast::BindingPatternKind;

        // Get left (must be a BindingPattern)
        self.context = self.context.clone().with_parent("AssignmentPattern", "left");
        let left_value = estree.get("left").ok_or_else(|| ConversionError::MissingField {
            field: "left".to_string(),
            node_type: "AssignmentPattern".to_string(),
            span: self.get_node_span(estree),
        })?;

        let left_pattern = self.convert_binding_pattern(left_value)?;

        // Get right (must be an Expression)
        self.context = self.context.clone().with_parent("AssignmentPattern", "right");
        let right_value = estree.get("right").ok_or_else(|| ConversionError::MissingField {
            field: "right".to_string(),
            node_type: "AssignmentPattern".to_string(),
            span: self.get_node_span(estree),
        })?;

        let right_expr = self.convert_expression(right_value)?;

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        let assignment_pattern = self.builder.alloc_assignment_pattern(span, left_pattern, right_expr);
        let pattern = self.builder.binding_pattern(BindingPatternKind::AssignmentPattern(assignment_pattern), None::<oxc_ast::ast::TSTypeAnnotation>, false);
        Ok(pattern)
    }

    /// Convert an ESTree Property (in ObjectPattern context) to oxc BindingProperty.
    fn convert_binding_property(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::BindingProperty<'a>> {
        use oxc_ast::ast::{BindingProperty, PropertyKey};
        use oxc_span::Atom;

        // Get key
        self.context = self.context.clone().with_parent("Property", "key");
        let key_value = estree.get("key").ok_or_else(|| ConversionError::MissingField {
            field: "key".to_string(),
            node_type: "Property".to_string(),
            span: self.get_node_span(estree),
        })?;

        let key = self.convert_property_key(key_value)?;

        // Get value - in ObjectPattern context, this is always a binding
        // Preserve the binding context from parent (ObjectPattern)
        let mut value_context = self.context.clone().with_parent("Property", "value");
        if self.context.is_binding_context {
            value_context.is_binding_context = true;
        }
        self.context = value_context;
        let value_value = estree.get("value").ok_or_else(|| ConversionError::MissingField {
            field: "value".to_string(),
            node_type: "Property".to_string(),
            span: self.get_node_span(estree),
        })?;

        let value_pattern = self.convert_binding_pattern(value_value)?;

        // Get shorthand and computed flags
        let shorthand = estree.get("shorthand").and_then(|v| v.as_bool()).unwrap_or(false);
        let computed = estree.get("computed").and_then(|v| v.as_bool()).unwrap_or(false);

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        let binding_prop = self.builder.binding_property(span, key, value_pattern, shorthand, computed);
        Ok(binding_prop)
    }

    /// Convert an ESTree node to oxc PropertyKey (helper for binding properties).
    fn convert_property_key(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::PropertyKey<'a>> {
        use oxc_ast::ast::{Expression, PropertyKey};
        use oxc_estree::deserialize::{EstreeNode, EstreeNodeType};
        use oxc_span::Atom;

        let node_type = <Value as EstreeNode>::get_type(estree).ok_or_else(|| ConversionError::MissingField {
            field: "type".to_string(),
            node_type: "convert_property_key".to_string(),
            span: self.get_node_span(estree),
        })?;

        match node_type {
            EstreeNodeType::Identifier => {
                let estree_id = oxc_estree::deserialize::EstreeIdentifier::from_json(estree)
                    .ok_or_else(|| ConversionError::InvalidFieldType {
                        field: "Identifier".to_string(),
                        expected: "valid Identifier node".to_string(),
                        got: format!("{:?}", estree),
                        span: self.get_node_span(estree),
                    })?;
                let name = Atom::from_in(estree_id.name.as_str(), self.builder.allocator);
                let range = estree_id.range.unwrap_or([0, 0]);
                let span = convert_span(self.source_text, range[0] as usize, range[1] as usize);
                let ident_name = self.builder.identifier_name(span, name);
                Ok(PropertyKey::StaticIdentifier(oxc_allocator::Box::new_in(ident_name, self.builder.allocator)))
            }
            EstreeNodeType::Literal => {
                // For string literals used as keys
                let expr = self.convert_literal_to_expression(estree)?;
                Ok(PropertyKey::from(expr))
            }
            _ => {
                // For computed properties, convert to expression
                let expr = self.convert_expression(estree)?;
                Ok(PropertyKey::from(expr))
            }
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

        // Convert kind string to PropertyKind enum
        use oxc_ast::ast::PropertyKind;
        let kind = match kind_str.as_str() {
            "init" => PropertyKind::Init,
            "get" => PropertyKind::Get,
            "set" => PropertyKind::Set,
            _ => {
                return Err(ConversionError::UnsupportedNodeType {
                    node_type: format!("Property with kind={}", kind_str),
                    span: self.get_node_span(estree),
                });
            }
        };

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

        // Handle shorthand properties: { a } is equivalent to { a: a }
        use oxc_ast::ast::Expression;
        let final_value = if shorthand {
            // For shorthand, value should be the same identifier as key
            // The ESTree spec says shorthand properties don't have a separate value field
            // We need to create an IdentifierReference from the key
            match &key {
                PropertyKey::StaticIdentifier(ident_name) => {
                    // Create an IdentifierReference from the IdentifierName
                    let ident_ref = self.builder.identifier_reference(ident_name.span, ident_name.name.clone());
                    Expression::Identifier(oxc_allocator::Box::new_in(ident_ref, self.builder.allocator))
                }
                _ => {
                    // If key is not a StaticIdentifier, we can't create a shorthand
                    // This shouldn't happen per ESTree spec, but handle gracefully
                    return Err(ConversionError::InvalidFieldType {
                        field: "key".to_string(),
                        expected: "StaticIdentifier for shorthand property".to_string(),
                        got: format!("{:?}", key),
                        span: self.get_node_span(estree),
                    });
                }
            }
        } else {
            value
        };

        // Handle method properties: the value should be a FunctionExpression
        // ESTree already provides this, so we just pass it through
        // The `method` flag is already extracted above
        // For getter/setter properties, the value must be a FunctionExpression
        // Shorthand properties are only valid for "init" kind
        if shorthand && kind != PropertyKind::Init {
            return Err(ConversionError::InvalidFieldType {
                field: "shorthand".to_string(),
                expected: "false for get/set properties".to_string(),
                got: "true".to_string(),
                span: self.get_node_span(estree),
            });
        }

        let obj_prop = self.builder.alloc_object_property(span, kind, key, final_value, method, shorthand, computed);
        Ok(ObjectPropertyKind::ObjectProperty(obj_prop))
    }

    /// Convert an ESTree declaration node to oxc Declaration.
    /// Declaration can be VariableDeclaration, FunctionDeclaration, ClassDeclaration, or TypeScript declarations.
    fn convert_to_declaration(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::Declaration<'a>> {
        use oxc_ast::ast::Declaration;
        use oxc_estree::deserialize::{EstreeNode, EstreeNodeType};

        let node_type = <Value as EstreeNode>::get_type(estree).ok_or_else(|| ConversionError::MissingField {
            field: "type".to_string(),
            node_type: "declaration".to_string(),
            span: self.get_node_span(estree),
        })?;

        match node_type {
            EstreeNodeType::VariableDeclaration => {
                let stmt = self.convert_variable_declaration(estree)?;
                match stmt {
                    oxc_ast::ast::Statement::VariableDeclaration(var_decl) => {
                        Ok(Declaration::VariableDeclaration(var_decl))
                    }
                    _ => Err(ConversionError::InvalidFieldType {
                        field: "declaration".to_string(),
                        expected: "VariableDeclaration statement".to_string(),
                        got: format!("{:?}", stmt),
                        span: self.get_node_span(estree),
                    }),
                }
            }
            EstreeNodeType::FunctionDeclaration => {
                let stmt = self.convert_function_declaration(estree)?;
                match stmt {
                    oxc_ast::ast::Statement::FunctionDeclaration(func_decl) => {
                        Ok(Declaration::FunctionDeclaration(func_decl))
                    }
                    _ => Err(ConversionError::InvalidFieldType {
                        field: "declaration".to_string(),
                        expected: "FunctionDeclaration statement".to_string(),
                        got: format!("{:?}", stmt),
                        span: self.get_node_span(estree),
                    }),
                }
            }
            EstreeNodeType::ClassDeclaration => {
                let stmt = self.convert_class_declaration(estree)?;
                match stmt {
                    oxc_ast::ast::Statement::ClassDeclaration(class_decl) => {
                        Ok(Declaration::ClassDeclaration(class_decl))
                    }
                    _ => Err(ConversionError::InvalidFieldType {
                        field: "declaration".to_string(),
                        expected: "ClassDeclaration statement".to_string(),
                        got: format!("{:?}", stmt),
                        span: self.get_node_span(estree),
                    }),
                }
            }
            EstreeNodeType::TSTypeAliasDeclaration => {
                let stmt = self.convert_ts_type_alias_declaration(estree)?;
                match stmt {
                    oxc_ast::ast::Statement::TSTypeAliasDeclaration(type_decl) => {
                        Ok(Declaration::TSTypeAliasDeclaration(type_decl))
                    }
                    _ => Err(ConversionError::InvalidFieldType {
                        field: "declaration".to_string(),
                        expected: "TSTypeAliasDeclaration statement".to_string(),
                        got: format!("{:?}", stmt),
                        span: self.get_node_span(estree),
                    }),
                }
            }
            EstreeNodeType::TSInterfaceDeclaration => {
                let stmt = self.convert_ts_interface_declaration(estree)?;
                match stmt {
                    oxc_ast::ast::Statement::TSInterfaceDeclaration(interface_decl) => {
                        Ok(Declaration::TSInterfaceDeclaration(interface_decl))
                    }
                    _ => Err(ConversionError::InvalidFieldType {
                        field: "declaration".to_string(),
                        expected: "TSInterfaceDeclaration statement".to_string(),
                        got: format!("{:?}", stmt),
                        span: self.get_node_span(estree),
                    }),
                }
            }
            EstreeNodeType::TSEnumDeclaration => {
                let stmt = self.convert_ts_enum_declaration(estree)?;
                match stmt {
                    oxc_ast::ast::Statement::TSEnumDeclaration(enum_decl) => {
                        Ok(Declaration::TSEnumDeclaration(enum_decl))
                    }
                    _ => Err(ConversionError::InvalidFieldType {
                        field: "declaration".to_string(),
                        expected: "TSEnumDeclaration statement".to_string(),
                        got: format!("{:?}", stmt),
                        span: self.get_node_span(estree),
                    }),
                }
            }
            EstreeNodeType::TSModuleDeclaration => {
                let stmt = self.convert_ts_module_declaration(estree)?;
                match stmt {
                    oxc_ast::ast::Statement::TSModuleDeclaration(module_decl) => {
                        Ok(Declaration::TSModuleDeclaration(module_decl))
                    }
                    _ => Err(ConversionError::InvalidFieldType {
                        field: "declaration".to_string(),
                        expected: "TSModuleDeclaration statement".to_string(),
                        got: format!("{:?}", stmt),
                        span: self.get_node_span(estree),
                    }),
                }
            }
            EstreeNodeType::TSImportEqualsDeclaration => {
                let stmt = self.convert_ts_import_equals_declaration(estree)?;
                match stmt {
                    oxc_ast::ast::Statement::TSImportEqualsDeclaration(import_decl) => {
                        Ok(Declaration::TSImportEqualsDeclaration(import_decl))
                    }
                    _ => Err(ConversionError::InvalidFieldType {
                        field: "declaration".to_string(),
                        expected: "TSImportEqualsDeclaration statement".to_string(),
                        got: format!("{:?}", stmt),
                        span: self.get_node_span(estree),
                    }),
                }
            }
            _ => Err(ConversionError::UnsupportedNodeType {
                node_type: format!("Declaration type: {:?}", node_type),
                span: self.get_node_span(estree),
            }),
        }
    }

    /// Convert an ESTree ThisExpression to oxc ThisExpression.
    fn convert_this_expression(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::Expression<'a>> {
        use oxc_ast::ast::Expression;

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        let this_expr = self.builder.alloc_this_expression(span);
        Ok(Expression::ThisExpression(this_expr))
    }

    /// Convert an ESTree AwaitExpression to oxc AwaitExpression.
    fn convert_await_expression(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::Expression<'a>> {
        use oxc_ast::ast::Expression;

        // Get argument
        self.context = self.context.clone().with_parent("AwaitExpression", "argument");
        let argument_value = estree.get("argument").ok_or_else(|| ConversionError::MissingField {
            field: "argument".to_string(),
            node_type: "AwaitExpression".to_string(),
            span: self.get_node_span(estree),
        })?;
        let argument = self.convert_expression(argument_value)?;

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        let await_expr = self.builder.alloc_await_expression(span, argument);
        Ok(Expression::AwaitExpression(await_expr))
    }

    /// Convert an ESTree TemplateLiteral to oxc Expression.
    fn convert_template_literal(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::Expression<'a>> {
        use oxc_ast::ast::Expression;
        use oxc_span::Atom;

        // Get quasis
        let quasis_value = estree.get("quasis").ok_or_else(|| ConversionError::MissingField {
            field: "quasis".to_string(),
            node_type: "TemplateLiteral".to_string(),
            span: self.get_node_span(estree),
        })?;

        let quasis_array = quasis_value.as_array().ok_or_else(|| ConversionError::InvalidFieldType {
            field: "quasis".to_string(),
            expected: "array".to_string(),
            got: format!("{:?}", quasis_value),
            span: self.get_node_span(estree),
        })?;

        let mut quasis = Vec::new_in(self.builder.allocator);
        for (idx, quasi_value) in quasis_array.iter().enumerate() {
            self.context = self.context.clone().with_parent("TemplateLiteral", "quasis");
            let is_tail = idx == quasis_array.len() - 1;
            let template_element = self.convert_template_element(quasi_value, is_tail)?;
            quasis.push(template_element);
        }

        // Get expressions
        let expressions_value = estree.get("expressions").ok_or_else(|| ConversionError::MissingField {
            field: "expressions".to_string(),
            node_type: "TemplateLiteral".to_string(),
            span: self.get_node_span(estree),
        })?;

        let expressions_array = expressions_value.as_array().ok_or_else(|| ConversionError::InvalidFieldType {
            field: "expressions".to_string(),
            expected: "array".to_string(),
            got: format!("{:?}", expressions_value),
            span: self.get_node_span(estree),
        })?;

        let mut expressions = Vec::new_in(self.builder.allocator);
        for expr_value in expressions_array {
            self.context = self.context.clone().with_parent("TemplateLiteral", "expressions");
            let expr = self.convert_expression(expr_value)?;
            expressions.push(expr);
        }

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        let template_lit = self.builder.alloc_template_literal(span, quasis, expressions);
        Ok(Expression::TemplateLiteral(template_lit))
    }

    /// Convert an ESTree TaggedTemplateExpression to oxc Expression.
    fn convert_tagged_template_expression(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::Expression<'a>> {
        use oxc_ast::ast::Expression;

        // Get tag
        self.context = self.context.clone().with_parent("TaggedTemplateExpression", "tag");
        let tag_value = estree.get("tag").ok_or_else(|| ConversionError::MissingField {
            field: "tag".to_string(),
            node_type: "TaggedTemplateExpression".to_string(),
            span: self.get_node_span(estree),
        })?;
        let tag = self.convert_expression(tag_value)?;

        // Get quasi (template literal) - convert directly to TemplateLiteral (not Expression)
        self.context = self.context.clone().with_parent("TaggedTemplateExpression", "quasi");
        let quasi_value = estree.get("quasi").ok_or_else(|| ConversionError::MissingField {
            field: "quasi".to_string(),
            node_type: "TaggedTemplateExpression".to_string(),
            span: self.get_node_span(estree),
        })?;
        
        // Convert quasi to TemplateLiteral directly using template_literal (not alloc_template_literal)
        let quasis_value = quasi_value.get("quasis").ok_or_else(|| ConversionError::MissingField {
            field: "quasis".to_string(),
            node_type: "quasi".to_string(),
            span: self.get_node_span(estree),
        })?;
        let quasis_array = quasis_value.as_array().ok_or_else(|| ConversionError::InvalidFieldType {
            field: "quasis".to_string(),
            expected: "array".to_string(),
            got: format!("{:?}", quasis_value),
            span: self.get_node_span(estree),
        })?;

        let mut quasis = Vec::new_in(self.builder.allocator);
        for (idx, quasi_elem_value) in quasis_array.iter().enumerate() {
            self.context = self.context.clone().with_parent("TaggedTemplateExpression", "quasi.quasis");
            let is_tail = idx == quasis_array.len() - 1;
            let template_element = self.convert_template_element(quasi_elem_value, is_tail)?;
            quasis.push(template_element);
        }

        let expressions_value = quasi_value.get("expressions").ok_or_else(|| ConversionError::MissingField {
            field: "expressions".to_string(),
            node_type: "quasi".to_string(),
            span: self.get_node_span(estree),
        })?;
        let expressions_array = expressions_value.as_array().ok_or_else(|| ConversionError::InvalidFieldType {
            field: "expressions".to_string(),
            expected: "array".to_string(),
            got: format!("{:?}", expressions_value),
            span: self.get_node_span(estree),
        })?;

        let mut expressions = Vec::new_in(self.builder.allocator);
        for expr_value in expressions_array {
            self.context = self.context.clone().with_parent("TaggedTemplateExpression", "quasi.expressions");
            let expr = self.convert_expression(expr_value)?;
            expressions.push(expr);
        }

        let (quasi_start, quasi_end) = self.get_node_span(quasi_value);
        let quasi_span = Span::new(quasi_start, quasi_end);
        let quasi = self.builder.template_literal(quasi_span, quasis, expressions);

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        // Get typeArguments (optional TSTypeParameterInstantiation)
        let type_args: Option<oxc_allocator::Box<'a, oxc_ast::ast::TSTypeParameterInstantiation<'a>>> = if let Some(type_args_value) = estree.get("typeArguments") {
            if type_args_value.is_null() {
                None
            } else {
                self.context = self.context.clone().with_parent("TaggedTemplateExpression", "typeArguments");
                Some(self.convert_ts_type_parameter_instantiation(type_args_value)?)
            }
        } else {
            None
        };
        let tagged = self.builder.alloc_tagged_template_expression(span, tag, type_args, quasi);
        Ok(Expression::TaggedTemplateExpression(tagged))
    }

    /// Convert an ESTree TemplateElement to oxc TemplateElement.
    fn convert_template_element(&mut self, estree: &Value, is_tail: bool) -> ConversionResult<oxc_ast::ast::TemplateElement<'a>> {
        use oxc_span::Atom;

        // Get value (object with raw and cooked)
        let value_obj = estree.get("value").ok_or_else(|| ConversionError::MissingField {
            field: "value".to_string(),
            node_type: "TemplateElement".to_string(),
            span: self.get_node_span(estree),
        })?;

        let raw_str = value_obj.get("raw").and_then(|v| v.as_str())
            .ok_or_else(|| ConversionError::MissingField {
                field: "value.raw".to_string(),
                node_type: "TemplateElement".to_string(),
                span: self.get_node_span(estree),
            })?;

        let cooked_str = value_obj.get("cooked").and_then(|v| v.as_str());

        // Get tail (use parameter if provided, otherwise from ESTree)
        let tail = if estree.get("tail").is_some() {
            estree.get("tail").and_then(|v| v.as_bool()).unwrap_or(false)
        } else {
            is_tail
        };

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        let raw = Atom::from_in(raw_str, self.builder.allocator);
        let cooked = cooked_str.map(|s| Atom::from_in(s, self.builder.allocator));
        use oxc_ast::ast::TemplateElementValue;
        let value = TemplateElementValue { raw, cooked };
        let template_element = self.builder.template_element(span, value, tail);
        Ok(template_element)
    }

    /// Convert an ESTree Super to oxc Super.
    fn convert_super_expression(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::Expression<'a>> {
        use oxc_ast::ast::Expression;

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        let super_expr = self.builder.alloc_super(span);
        Ok(Expression::Super(super_expr))
    }

    /// Convert an ESTree FunctionDeclaration to oxc Statement.
    fn convert_function_declaration(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::Statement<'a>> {
        use oxc_ast::ast::{FunctionType, Statement};
        
        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);
        
        let function_box = self.convert_function(estree, FunctionType::FunctionDeclaration)?;
        
        // FunctionDeclaration is a Statement that wraps a Function
        Ok(Statement::FunctionDeclaration(function_box))
    }

    /// Convert an ESTree FunctionExpression to oxc Expression.
    fn convert_function_expression(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::Expression<'a>> {
        use oxc_ast::ast::{Expression, FunctionType};
        
        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);
        
        let function_box = self.convert_function(estree, FunctionType::FunctionExpression)?;
        
        // FunctionExpression is an Expression that wraps a Function
        Ok(Expression::FunctionExpression(function_box))
    }

    /// Convert an ESTree ArrowFunctionExpression to oxc Expression.
    fn convert_arrow_function_expression(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::Expression<'a>> {
        use oxc_ast::ast::{Expression, FormalParameterKind, FormalParameters, Statement};
        
        // Get params
        self.context = self.context.clone().with_parent("ArrowFunctionExpression", "params");
        let params_value = estree.get("params").ok_or_else(|| ConversionError::MissingField {
            field: "params".to_string(),
            node_type: "ArrowFunctionExpression".to_string(),
            span: self.get_node_span(estree),
        })?;
        let params_array = params_value.as_array().ok_or_else(|| ConversionError::InvalidFieldType {
            field: "params".to_string(),
            expected: "array".to_string(),
            got: format!("{:?}", params_value),
            span: self.get_node_span(estree),
        })?;

        let mut param_items = Vec::new_in(self.builder.allocator);
        let mut rest_param: Option<oxc_allocator::Box<'a, oxc_ast::ast::BindingRestElement<'a>>> = None;
        
        for (index, param_value) in params_array.iter().enumerate() {
            // Set context for parameters - they are always bindings
            let mut param_context = self.context.clone().with_parent("ArrowFunctionExpression", "params");
            param_context.is_binding_context = true;
            self.context = param_context;
            
            // Check if it's a RestElement (must be last)
            use oxc_estree::deserialize::{EstreeNode, EstreeNodeType};
            let param_type = <Value as EstreeNode>::get_type(param_value);
            if let Some(EstreeNodeType::RestElement) = param_type {
                if index != params_array.len() - 1 {
                    return Err(ConversionError::InvalidFieldType {
                        field: "params".to_string(),
                        expected: "RestElement must be last parameter".to_string(),
                        got: format!("RestElement at index {}", index),
                        span: self.get_node_span(param_value),
                    });
                }
                // Convert RestElement to BindingRestElement
                let rest = self.convert_rest_element_to_binding_rest(param_value)?;
                rest_param = Some(rest);
                break;
            }
            
            // Convert to FormalParameter
            let formal_param = self.convert_to_formal_parameter(param_value)?;
            param_items.push(formal_param);
        }

        let (params_start, params_end) = self.get_node_span(params_value);
        let params_span = Span::new(params_start, params_end);
        let params = self.builder.formal_parameters(params_span, FormalParameterKind::ArrowFormalParameters, param_items, rest_param);
        let params_box = oxc_allocator::Box::new_in(params, self.builder.allocator);

        // Get body - can be Expression or BlockStatement
        // Reset context (body is not a binding context)
        let mut body_context = self.context.clone().with_parent("ArrowFunctionExpression", "body");
        body_context.is_binding_context = false;
        self.context = body_context;
        let body_value = estree.get("body").ok_or_else(|| ConversionError::MissingField {
            field: "body".to_string(),
            node_type: "ArrowFunctionExpression".to_string(),
            span: self.get_node_span(estree),
        })?;
        
        // Check if body is an expression or block statement
        use oxc_estree::deserialize::{EstreeNode, EstreeNodeType};
        let body_type = <Value as EstreeNode>::get_type(body_value);
        let (body, is_expression) = if let Some(EstreeNodeType::BlockStatement) = body_type {
            // Block statement body
            let body_stmt = self.convert_block_statement(body_value)?;
            match body_stmt {
                Statement::BlockStatement(bs) => {
                    let (body_start, body_end) = self.get_node_span(body_value);
                    let body_span = Span::new(body_start, body_end);
                    let directives: Vec<'a, oxc_ast::ast::Directive<'a>> = Vec::new_in(self.builder.allocator);
                    // Clone statements from BlockStatement
                    let mut statements = Vec::new_in(self.builder.allocator);
                    for stmt in bs.body.iter() {
                        statements.push(stmt.clone_in(self.builder.allocator));
                    }
                    let function_body = self.builder.function_body(body_span, directives, statements);
                    let body_box = oxc_allocator::Box::new_in(function_body, self.builder.allocator);
                    (body_box, false)
                }
                _ => return Err(ConversionError::InvalidFieldType {
                    field: "body".to_string(),
                    expected: "BlockStatement".to_string(),
                    got: format!("{:?}", body_stmt),
                    span: self.get_node_span(estree),
                }),
            }
        } else {
            // Expression body
            let expr = self.convert_expression(body_value)?;
            // Create a FunctionBody with a single ReturnStatement containing the expression
            let (body_start, body_end) = self.get_node_span(body_value);
            let body_span = Span::new(body_start, body_end);
            let mut statements = Vec::new_in(self.builder.allocator);
            let return_span = body_span;
            let return_stmt = self.builder.alloc_return_statement(return_span, Some(expr));
            statements.push(Statement::ReturnStatement(return_stmt));
            let directives: Vec<'a, oxc_ast::ast::Directive<'a>> = Vec::new_in(self.builder.allocator);
            let function_body = self.builder.function_body(body_span, directives, statements);
            let body_box = oxc_allocator::Box::new_in(function_body, self.builder.allocator);
            (body_box, true)
        };

        // Get async flag
        let async_flag = estree.get("async").and_then(|v| v.as_bool()).unwrap_or(false);

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        // Get typeParameters (optional)
        let type_params: Option<oxc_allocator::Box<'a, oxc_ast::ast::TSTypeParameterDeclaration<'a>>> = if let Some(type_params_value) = estree.get("typeParameters") {
            if type_params_value.is_null() {
                None
            } else {
                self.context = self.context.clone().with_parent("ArrowFunctionExpression", "typeParameters");
                Some(self.convert_ts_type_parameter_declaration(type_params_value)?)
            }
        } else {
            None
        };
        
        // Get returnType (optional)
        let return_type: Option<oxc_allocator::Box<'a, oxc_ast::ast::TSTypeAnnotation<'a>>> = if let Some(return_type_value) = estree.get("returnType") {
            if return_type_value.is_null() {
                None
            } else {
                self.context = self.context.clone().with_parent("ArrowFunctionExpression", "returnType");
                Some(self.convert_ts_type_annotation(return_type_value)?)
            }
        } else {
            None
        };
        let arrow = self.builder.alloc_arrow_function_expression(span, is_expression, async_flag, type_params, params_box, return_type, body);
        Ok(Expression::ArrowFunctionExpression(arrow))
    }

    /// Convert an ESTree Function to oxc Function (helper for FunctionDeclaration and FunctionExpression).
    fn convert_function(&mut self, estree: &Value, function_type: oxc_ast::ast::FunctionType) -> ConversionResult<oxc_allocator::Box<'a, oxc_ast::ast::Function<'a>>> {
        use oxc_ast::ast::{BindingIdentifier, FormalParameterKind, FormalParameters, FunctionBody, Statement};
        use oxc_span::Atom;

        // Get id (optional)
        let id = if let Some(id_value) = estree.get("id") {
            if id_value.is_null() {
                None
            } else {
                self.context = self.context.clone().with_parent("Function", "id");
                let estree_id = oxc_estree::deserialize::EstreeIdentifier::from_json(id_value)
                    .ok_or_else(|| ConversionError::InvalidFieldType {
                        field: "id".to_string(),
                        expected: "valid Identifier node".to_string(),
                        got: format!("{:?}", id_value),
                        span: self.get_node_span(estree),
                    })?;
                let name = Atom::from_in(estree_id.name.as_str(), self.builder.allocator);
                let range = estree_id.range.unwrap_or([0, 0]);
                let span = convert_span(self.source_text, range[0] as usize, range[1] as usize);
                Some(self.builder.binding_identifier(span, name))
            }
        } else {
            None
        };

        // Get params
        self.context = self.context.clone().with_parent("Function", "params");
        let params_value = estree.get("params").ok_or_else(|| ConversionError::MissingField {
            field: "params".to_string(),
            node_type: "Function".to_string(),
            span: self.get_node_span(estree),
        })?;
        let params_array = params_value.as_array().ok_or_else(|| ConversionError::InvalidFieldType {
            field: "params".to_string(),
            expected: "array".to_string(),
            got: format!("{:?}", params_value),
            span: self.get_node_span(estree),
        })?;

        let mut param_items = Vec::new_in(self.builder.allocator);
        let mut rest_param: Option<oxc_allocator::Box<'a, oxc_ast::ast::BindingRestElement<'a>>> = None;
        
        use oxc_estree::deserialize::{EstreeNode, EstreeNodeType};
        
        for (index, param_value) in params_array.iter().enumerate() {
            // Set context for parameters - they are always bindings
            let mut param_context = self.context.clone().with_parent("Function", "params");
            param_context.is_binding_context = true;
            self.context = param_context;
            
            // Check if it's a RestElement (must be last)
            let param_type = <Value as EstreeNode>::get_type(param_value);
            if let Some(EstreeNodeType::RestElement) = param_type {
                if index != params_array.len() - 1 {
                    return Err(ConversionError::InvalidFieldType {
                        field: "params".to_string(),
                        expected: "RestElement must be last parameter".to_string(),
                        got: format!("RestElement at index {}", index),
                        span: self.get_node_span(param_value),
                    });
                }
                // Convert RestElement to BindingRestElement
                let rest = self.convert_rest_element_to_binding_rest(param_value)?;
                rest_param = Some(rest);
                break;
            }
            
            // Convert to FormalParameter
            let formal_param = self.convert_to_formal_parameter(param_value)?;
            param_items.push(formal_param);
        }

        let (params_start, params_end) = self.get_node_span(params_value);
        let params_span = Span::new(params_start, params_end);
        let params = self.builder.formal_parameters(params_span, FormalParameterKind::FormalParameter, param_items, rest_param);
        let params_box = oxc_allocator::Box::new_in(params, self.builder.allocator);

        // Get body - reset context (body is not a binding context)
        let mut body_context = self.context.clone().with_parent("Function", "body");
        body_context.is_binding_context = false;
        self.context = body_context;
        let body_value = estree.get("body").ok_or_else(|| ConversionError::MissingField {
            field: "body".to_string(),
            node_type: "Function".to_string(),
            span: self.get_node_span(estree),
        })?;
        let body_stmt = self.convert_block_statement(body_value)?;
        let body = match body_stmt {
            Statement::BlockStatement(bs) => {
                let (body_start, body_end) = self.get_node_span(body_value);
                let body_span = Span::new(body_start, body_end);
                let directives: Vec<'a, oxc_ast::ast::Directive<'a>> = Vec::new_in(self.builder.allocator);
                // Clone statements from BlockStatement
                let mut statements = Vec::new_in(self.builder.allocator);
                for stmt in bs.body.iter() {
                    statements.push(stmt.clone_in(self.builder.allocator));
                }
                let function_body = self.builder.function_body(body_span, directives, statements);
                function_body
            }
            _ => return Err(ConversionError::InvalidFieldType {
                field: "body".to_string(),
                expected: "BlockStatement".to_string(),
                got: format!("{:?}", body_stmt),
                span: self.get_node_span(estree),
            }),
        };

        // Get generator and async flags
        let generator = estree.get("generator").and_then(|v| v.as_bool()).unwrap_or(false);
        let async_flag = estree.get("async").and_then(|v| v.as_bool()).unwrap_or(false);

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        // body needs to be Option<Box<FunctionBody>>
        let body_box = Some(oxc_allocator::Box::new_in(body, self.builder.allocator));
        // alloc_function signature: span, type, id, generator, async, declare, type_parameters, this_param, params, return_type, body
        
        // Get typeParameters (optional)
        let type_params: Option<oxc_allocator::Box<'a, oxc_ast::ast::TSTypeParameterDeclaration<'a>>> = if let Some(type_params_value) = estree.get("typeParameters") {
            if type_params_value.is_null() {
                None
            } else {
                self.context = self.context.clone().with_parent("Function", "typeParameters");
                Some(self.convert_ts_type_parameter_declaration(type_params_value)?)
            }
        } else {
            None
        };
        
        // Get thisParam (optional TSThisParameter)
        // In ESTree, this is represented as an Identifier with name "this" and optional typeAnnotation
        let this_param: Option<oxc_allocator::Box<'a, oxc_ast::ast::TSThisParameter<'a>>> = if let Some(this_param_value) = estree.get("thisParam") {
            if this_param_value.is_null() {
                None
            } else {
                self.context = self.context.clone().with_parent("Function", "thisParam");
                // Check if it's an Identifier with name "this"
                let name_value = this_param_value.get("name").and_then(|v| v.as_str());
                if name_value == Some("this") {
                    let (start, end) = self.get_node_span(this_param_value);
                    let span = Span::new(start, end);
                    // Get the span for just "this" - use the start of the identifier
                    // For now, use the same span (could be improved with more precise location info)
                    let this_span = span;
                    
                    // Get typeAnnotation (optional)
                    let type_annotation: Option<oxc_allocator::Box<'a, oxc_ast::ast::TSTypeAnnotation<'a>>> = if let Some(type_ann_value) = this_param_value.get("typeAnnotation") {
                        if type_ann_value.is_null() {
                            None
                        } else {
                            self.context = self.context.clone().with_parent("TSThisParameter", "typeAnnotation");
                            Some(self.convert_ts_type_annotation(type_ann_value)?)
                        }
                    } else {
                        None
                    };
                    
                    let this_param = self.builder.alloc_ts_this_parameter(span, this_span, type_annotation);
                    Some(this_param)
                } else {
                    // Not a this parameter, skip it
                    None
                }
            }
        } else {
            None
        };
        
        // Get returnType (optional)
        let return_type: Option<oxc_allocator::Box<'a, oxc_ast::ast::TSTypeAnnotation<'a>>> = if let Some(return_type_value) = estree.get("returnType") {
            if return_type_value.is_null() {
                None
            } else {
                self.context = self.context.clone().with_parent("Function", "returnType");
                Some(self.convert_ts_type_annotation(return_type_value)?)
            }
        } else {
            None
        };
        
        let function = self.builder.alloc_function(span, function_type, id, generator, async_flag, false, type_params, this_param, params_box, return_type, body_box);
        Ok(function)
    }

    /// Convert an ESTree YieldExpression to oxc YieldExpression.
    fn convert_yield_expression(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::Expression<'a>> {
        use oxc_ast::ast::Expression;

        // Get argument (optional)
        let argument = if let Some(arg_value) = estree.get("argument") {
            if arg_value.is_null() {
                None
            } else {
                self.context = self.context.clone().with_parent("YieldExpression", "argument");
                Some(self.convert_expression(arg_value)?)
            }
        } else {
            None
        };

        // Get delegate flag
        let delegate = estree.get("delegate").and_then(|v| v.as_bool()).unwrap_or(false);

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        let yield_expr = self.builder.alloc_yield_expression(span, delegate, argument);
        Ok(Expression::YieldExpression(yield_expr))
    }

    /// Convert an ESTree NewExpression to oxc NewExpression.
    fn convert_new_expression(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::Expression<'a>> {
        use oxc_ast::ast::{Argument, Expression};
        use oxc_estree::deserialize::{EstreeNode, EstreeNodeType};

        // Get callee
        self.context = self.context.clone().with_parent("NewExpression", "callee");
        let callee_value = estree.get("callee").ok_or_else(|| ConversionError::MissingField {
            field: "callee".to_string(),
            node_type: "NewExpression".to_string(),
            span: self.get_node_span(estree),
        })?;
        let callee = self.convert_expression(callee_value)?;

        // Get arguments
        let arguments_value = estree.get("arguments").ok_or_else(|| ConversionError::MissingField {
            field: "arguments".to_string(),
            node_type: "NewExpression".to_string(),
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
            self.context = self.context.clone().with_parent("NewExpression", "arguments");
            let arg = self.convert_to_argument(arg_value)?;
            args.push(arg);
        }

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        // Get typeArguments (optional TSTypeParameterInstantiation)
        let type_args: Option<oxc_allocator::Box<'a, oxc_ast::ast::TSTypeParameterInstantiation<'a>>> = if let Some(type_args_value) = estree.get("typeArguments") {
            if type_args_value.is_null() {
                None
            } else {
                self.context = self.context.clone().with_parent("NewExpression", "typeArguments");
                Some(self.convert_ts_type_parameter_instantiation(type_args_value)?)
            }
        } else {
            None
        };
        let new_expr = self.builder.alloc_new_expression(span, callee, type_args, args);
        Ok(Expression::NewExpression(new_expr))
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

    /// Convert an ESTree ClassDeclaration to oxc Statement.
    fn convert_class_declaration(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::Statement<'a>> {
        use oxc_ast::ast::{ClassType, Statement};
        
        let class_box = self.convert_class(estree, ClassType::ClassDeclaration)?;
        Ok(Statement::ClassDeclaration(class_box))
    }

    /// Convert an ESTree ClassExpression to oxc Expression.
    fn convert_class_expression(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::Expression<'a>> {
        use oxc_ast::ast::{ClassType, Expression};
        
        let class_box = self.convert_class(estree, ClassType::ClassExpression)?;
        // Use the boxed class directly - expression_class will handle it
        let expr = Expression::ClassExpression(class_box);
        Ok(expr)
    }

    /// Convert an ESTree Class to oxc Class (shared helper for ClassDeclaration and ClassExpression).
    fn convert_class(&mut self, estree: &Value, class_type: oxc_ast::ast::ClassType) -> ConversionResult<oxc_allocator::Box<'a, oxc_ast::ast::Class<'a>>> {
        use oxc_ast::ast::{BindingIdentifier, Class, ClassBody, ClassElement, Expression};
        use oxc_span::Atom;

        // Get id (optional)
        let id = if let Some(id_value) = estree.get("id") {
            if id_value.is_null() {
                None
            } else {
                self.context = self.context.clone().with_parent("Class", "id");
                let estree_id = oxc_estree::deserialize::EstreeIdentifier::from_json(id_value)
                    .ok_or_else(|| ConversionError::InvalidFieldType {
                        field: "id".to_string(),
                        expected: "valid Identifier node".to_string(),
                        got: format!("{:?}", id_value),
                        span: self.get_node_span(estree),
                    })?;
                let name = Atom::from_in(estree_id.name.as_str(), self.builder.allocator);
                let range = estree_id.range.unwrap_or([0, 0]);
                let span = convert_span(self.source_text, range[0] as usize, range[1] as usize);
                Some(self.builder.binding_identifier(span, name))
            }
        } else {
            None
        };

        // Get superClass (optional)
        let super_class = if let Some(super_value) = estree.get("superClass") {
            if super_value.is_null() {
                None
            } else {
                self.context = self.context.clone().with_parent("Class", "superClass");
                Some(self.convert_expression(super_value)?)
            }
        } else {
            None
        };

        // Get body
        self.context = self.context.clone().with_parent("Class", "body");
        let body_value = estree.get("body").ok_or_else(|| ConversionError::MissingField {
            field: "body".to_string(),
            node_type: "Class".to_string(),
            span: self.get_node_span(estree),
        })?;

        let body = self.convert_class_body(body_value)?;

        // Get decorators (optional array of Decorator)
        let decorators = if let Some(decorators_value) = estree.get("decorators") {
            if decorators_value.is_null() {
                Vec::new_in(self.builder.allocator)
            } else {
                let decorators_array = decorators_value.as_array().ok_or_else(|| ConversionError::InvalidFieldType {
                    field: "decorators".to_string(),
                    expected: "array".to_string(),
                    got: format!("{:?}", decorators_value),
                    span: self.get_node_span(estree),
                })?;
                
                let mut decorators_vec = Vec::new_in(self.builder.allocator);
                for decorator_value in decorators_array {
                    self.context = self.context.clone().with_parent("Class", "decorators");
                    let decorator = self.convert_decorator(decorator_value)?;
                    decorators_vec.push(decorator);
                }
                decorators_vec
            }
        } else {
            Vec::new_in(self.builder.allocator)
        };

        // Get implements (optional array of TSClassImplements - TypeScript only)
        let implements = if let Some(implements_value) = estree.get("implements") {
            if implements_value.is_null() {
                Vec::new_in(self.builder.allocator)
            } else {
                let implements_array = implements_value.as_array().ok_or_else(|| ConversionError::InvalidFieldType {
                    field: "implements".to_string(),
                    expected: "array".to_string(),
                    got: format!("{:?}", implements_value),
                    span: self.get_node_span(estree),
                })?;
                
                let mut implements_vec = Vec::new_in(self.builder.allocator);
                for implement_value in implements_array {
                    self.context = self.context.clone().with_parent("Class", "implements");
                    let class_implements = self.convert_ts_class_implements(implement_value)?;
                    implements_vec.push(class_implements);
                }
                implements_vec
            }
        } else {
            Vec::new_in(self.builder.allocator)
        };

        // Get abstract and declare (optional, false for now)
        let r#abstract = estree.get("abstract").and_then(|v| v.as_bool()).unwrap_or(false);
        let declare = estree.get("declare").and_then(|v| v.as_bool()).unwrap_or(false);

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        // Get typeParameters (optional)
        let type_params: Option<oxc_allocator::Box<'a, oxc_ast::ast::TSTypeParameterDeclaration<'a>>> = if let Some(type_params_value) = estree.get("typeParameters") {
            if type_params_value.is_null() {
                None
            } else {
                self.context = self.context.clone().with_parent("Class", "typeParameters");
                Some(self.convert_ts_type_parameter_declaration(type_params_value)?)
            }
        } else {
            None
        };
        
        // Get superTypeArguments (optional TSTypeParameterInstantiation)
        let super_type_args: Option<oxc_allocator::Box<'a, oxc_ast::ast::TSTypeParameterInstantiation<'a>>> = if let Some(super_type_args_value) = estree.get("superTypeArguments") {
            if super_type_args_value.is_null() {
                None
            } else {
                self.context = self.context.clone().with_parent("Class", "superTypeArguments");
                Some(self.convert_ts_type_parameter_instantiation(super_type_args_value)?)
            }
        } else {
            None
        };
        
        let class = self.builder.alloc_class(
            span,
            class_type,
            decorators,
            id,
            type_params,
            super_class,
            super_type_args,
            implements,
            body,
            r#abstract,
            declare,
        );
        Ok(class)
    }

    /// Convert an ESTree ClassBody to oxc ClassBody.
    fn convert_class_body(&mut self, estree: &Value) -> ConversionResult<oxc_allocator::Box<'a, oxc_ast::ast::ClassBody<'a>>> {
        use oxc_ast::ast::{ClassBody, ClassElement};

        // Get body array
        let body_value = estree.get("body").ok_or_else(|| ConversionError::MissingField {
            field: "body".to_string(),
            node_type: "ClassBody".to_string(),
            span: self.get_node_span(estree),
        })?;

        let body_array = body_value.as_array().ok_or_else(|| ConversionError::InvalidFieldType {
            field: "body".to_string(),
            expected: "array".to_string(),
            got: format!("{:?}", body_value),
            span: self.get_node_span(estree),
        })?;

        let mut class_elements = Vec::new_in(self.builder.allocator);
        for elem_value in body_array {
            // Skip null values
            if elem_value.is_null() {
                continue;
            }
            
            self.context = self.context.clone().with_parent("ClassBody", "body");
            let class_elem = self.convert_class_element(elem_value)?;
            class_elements.push(class_elem);
        }

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        let class_body = self.builder.class_body(span, class_elements);
        Ok(oxc_allocator::Box::new_in(class_body, self.builder.allocator))
    }

    /// Convert an ESTree class element to oxc ClassElement.
    fn convert_class_element(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::ClassElement<'a>> {
        use oxc_ast::ast::{ClassElement, MethodDefinitionKind, MethodDefinitionType, PropertyDefinitionType};
        use oxc_estree::deserialize::{EstreeNode, EstreeNodeType};

        let node_type = <Value as EstreeNode>::get_type(estree).ok_or_else(|| ConversionError::MissingField {
            field: "type".to_string(),
            node_type: "ClassElement".to_string(),
            span: self.get_node_span(estree),
        })?;

        match node_type {
            EstreeNodeType::MethodDefinition => {
                self.convert_method_definition(estree)
            }
            EstreeNodeType::PropertyDefinition => {
                self.convert_property_definition(estree)
            }
            EstreeNodeType::AccessorProperty => {
                self.convert_accessor_property(estree)
            }
            EstreeNodeType::StaticBlock => {
                self.convert_static_block(estree)
            }
            _ => Err(ConversionError::UnsupportedNodeType {
                node_type: format!("{:?}", node_type),
                span: self.get_node_span(estree),
            }),
        }
    }

    /// Convert an ESTree MethodDefinition to oxc ClassElement.
    fn convert_method_definition(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::ClassElement<'a>> {
        use oxc_ast::ast::{ClassElement, FunctionType, MethodDefinitionKind, MethodDefinitionType, PropertyKey};

        // Get key
        self.context = self.context.clone().with_parent("MethodDefinition", "key");
        let key_value = estree.get("key").ok_or_else(|| ConversionError::MissingField {
            field: "key".to_string(),
            node_type: "MethodDefinition".to_string(),
            span: self.get_node_span(estree),
        })?;
        let key = self.convert_property_key(key_value)?;

        // Get value (FunctionExpression)
        self.context = self.context.clone().with_parent("MethodDefinition", "value");
        let value_value = estree.get("value").ok_or_else(|| ConversionError::MissingField {
            field: "value".to_string(),
            node_type: "MethodDefinition".to_string(),
            span: self.get_node_span(estree),
        })?;
        let function = self.convert_function(value_value, FunctionType::FunctionExpression)?;

        // Get kind (constructor, method, get, set)
        let kind_str = estree.get("kind").and_then(|v| v.as_str()).unwrap_or("method");
        let kind = match kind_str {
            "constructor" => MethodDefinitionKind::Constructor,
            "get" => MethodDefinitionKind::Get,
            "set" => MethodDefinitionKind::Set,
            "method" | _ => MethodDefinitionKind::Method,
        };

        // Get computed
        let computed = estree.get("computed").and_then(|v| v.as_bool()).unwrap_or(false);

        // Get static
        let r#static = estree.get("static").and_then(|v| v.as_bool()).unwrap_or(false);

        // Get optional (TypeScript)
        let optional = estree.get("optional").and_then(|v| v.as_bool()).unwrap_or(false);

        // Get override (TypeScript)
        let r#override = estree.get("override").and_then(|v| v.as_bool()).unwrap_or(false);

        // Get accessibility (TypeScript)
        let accessibility: Option<oxc_ast::ast::TSAccessibility> = if let Some(accessibility_str) = estree.get("accessibility").and_then(|v| v.as_str()) {
            match accessibility_str {
                "public" => Some(oxc_ast::ast::TSAccessibility::Public),
                "private" => Some(oxc_ast::ast::TSAccessibility::Private),
                "protected" => Some(oxc_ast::ast::TSAccessibility::Protected),
                _ => None,
            }
        } else {
            None
        };

        // Get decorators (optional array of Decorator)
        let decorators = if let Some(decorators_value) = estree.get("decorators") {
            if decorators_value.is_null() {
                Vec::new_in(self.builder.allocator)
            } else {
                let decorators_array = decorators_value.as_array().ok_or_else(|| ConversionError::InvalidFieldType {
                    field: "decorators".to_string(),
                    expected: "array".to_string(),
                    got: format!("{:?}", decorators_value),
                    span: self.get_node_span(estree),
                })?;
                
                let mut decorators_vec = Vec::new_in(self.builder.allocator);
                for decorator_value in decorators_array {
                    self.context = self.context.clone().with_parent("MethodDefinition", "decorators");
                    let decorator = self.convert_decorator(decorator_value)?;
                    decorators_vec.push(decorator);
                }
                decorators_vec
            }
        } else {
            Vec::new_in(self.builder.allocator)
        };

        // Get type (MethodDefinitionType)
        let r#type = MethodDefinitionType::MethodDefinition;

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        let method_def = self.builder.class_element_method_definition(
            span,
            r#type,
            decorators,
            key,
            function,
            kind,
            computed,
            r#static,
            r#override,
            optional,
            accessibility,
        );
        Ok(method_def)
    }

    /// Convert an ESTree PropertyDefinition to oxc ClassElement.
    fn convert_property_definition(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::ClassElement<'a>> {
        use oxc_ast::ast::{ClassElement, PropertyDefinitionType, PropertyKey};

        // Get key
        self.context = self.context.clone().with_parent("PropertyDefinition", "key");
        let key_value = estree.get("key").ok_or_else(|| ConversionError::MissingField {
            field: "key".to_string(),
            node_type: "PropertyDefinition".to_string(),
            span: self.get_node_span(estree),
        })?;
        let key = self.convert_property_key(key_value)?;

        // Get value (optional)
        let value = if let Some(value_value) = estree.get("value") {
            if value_value.is_null() {
                None
            } else {
                self.context = self.context.clone().with_parent("PropertyDefinition", "value");
                Some(self.convert_expression(value_value)?)
            }
        } else {
            None
        };

        // Get computed
        let computed = estree.get("computed").and_then(|v| v.as_bool()).unwrap_or(false);

        // Get static
        let r#static = estree.get("static").and_then(|v| v.as_bool()).unwrap_or(false);

        // Get optional (TypeScript)
        let optional = estree.get("optional").and_then(|v| v.as_bool()).unwrap_or(false);

        // Get override (TypeScript)
        let r#override = estree.get("override").and_then(|v| v.as_bool()).unwrap_or(false);

        // Get declare (TypeScript)
        let declare = estree.get("declare").and_then(|v| v.as_bool()).unwrap_or(false);

        // Get definite (TypeScript)
        let definite = estree.get("definite").and_then(|v| v.as_bool()).unwrap_or(false);

        // Get readonly (TypeScript)
        let readonly = estree.get("readonly").and_then(|v| v.as_bool()).unwrap_or(false);

        // Get type_annotation (TypeScript)
        let type_annotation: Option<oxc_allocator::Box<'a, oxc_ast::ast::TSTypeAnnotation<'a>>> = if let Some(type_ann_value) = estree.get("typeAnnotation") {
            if type_ann_value.is_null() {
                None
            } else {
                self.context = self.context.clone().with_parent("PropertyDefinition", "typeAnnotation");
                Some(self.convert_ts_type_annotation(type_ann_value)?)
            }
        } else {
            None
        };

        // Get accessibility (TypeScript)
        let accessibility: Option<oxc_ast::ast::TSAccessibility> = if let Some(accessibility_str) = estree.get("accessibility").and_then(|v| v.as_str()) {
            match accessibility_str {
                "public" => Some(oxc_ast::ast::TSAccessibility::Public),
                "private" => Some(oxc_ast::ast::TSAccessibility::Private),
                "protected" => Some(oxc_ast::ast::TSAccessibility::Protected),
                _ => None,
            }
        } else {
            None
        };

        // Get decorators (optional array of Decorator)
        let decorators = if let Some(decorators_value) = estree.get("decorators") {
            if decorators_value.is_null() {
                Vec::new_in(self.builder.allocator)
            } else {
                let decorators_array = decorators_value.as_array().ok_or_else(|| ConversionError::InvalidFieldType {
                    field: "decorators".to_string(),
                    expected: "array".to_string(),
                    got: format!("{:?}", decorators_value),
                    span: self.get_node_span(estree),
                })?;
                
                let mut decorators_vec = Vec::new_in(self.builder.allocator);
                for decorator_value in decorators_array {
                    self.context = self.context.clone().with_parent("PropertyDefinition", "decorators");
                    let decorator = self.convert_decorator(decorator_value)?;
                    decorators_vec.push(decorator);
                }
                decorators_vec
            }
        } else {
            Vec::new_in(self.builder.allocator)
        };

        // Get type (PropertyDefinitionType)
        let r#type = PropertyDefinitionType::PropertyDefinition;

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        let prop_def = self.builder.class_element_property_definition(
            span,
            r#type,
            decorators,
            key,
            type_annotation,
            value,
            computed,
            r#static,
            declare,
            r#override,
            optional,
            definite,
            readonly,
            accessibility,
        );
        Ok(prop_def)
    }

    /// Convert an ESTree AccessorProperty to oxc ClassElement.
    fn convert_accessor_property(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::ClassElement<'a>> {
        use oxc_ast::ast::{AccessorPropertyType, ClassElement, PropertyKey};

        // Get key
        self.context = self.context.clone().with_parent("AccessorProperty", "key");
        let key_value = estree.get("key").ok_or_else(|| ConversionError::MissingField {
            field: "key".to_string(),
            node_type: "AccessorProperty".to_string(),
            span: self.get_node_span(estree),
        })?;
        let key = self.convert_property_key(key_value)?;

        // Get value (optional)
        let value = if let Some(value_value) = estree.get("value") {
            if value_value.is_null() {
                None
            } else {
                self.context = self.context.clone().with_parent("AccessorProperty", "value");
                Some(self.convert_expression(value_value)?)
            }
        } else {
            None
        };

        // Get computed
        let computed = estree.get("computed").and_then(|v| v.as_bool()).unwrap_or(false);

        // Get static
        let r#static = estree.get("static").and_then(|v| v.as_bool()).unwrap_or(false);

        // Get override (TypeScript)
        let r#override = estree.get("override").and_then(|v| v.as_bool()).unwrap_or(false);

        // Get definite (TypeScript)
        let definite = estree.get("definite").and_then(|v| v.as_bool()).unwrap_or(false);

        // Get accessibility (TypeScript)
        let accessibility: Option<oxc_ast::ast::TSAccessibility> = if let Some(accessibility_str) = estree.get("accessibility").and_then(|v| v.as_str()) {
            match accessibility_str {
                "public" => Some(oxc_ast::ast::TSAccessibility::Public),
                "private" => Some(oxc_ast::ast::TSAccessibility::Private),
                "protected" => Some(oxc_ast::ast::TSAccessibility::Protected),
                _ => None,
            }
        } else {
            None
        };

        // Get decorators (optional array of Decorator)
        let decorators = if let Some(decorators_value) = estree.get("decorators") {
            if decorators_value.is_null() {
                Vec::new_in(self.builder.allocator)
            } else {
                let decorators_array = decorators_value.as_array().ok_or_else(|| ConversionError::InvalidFieldType {
                    field: "decorators".to_string(),
                    expected: "array".to_string(),
                    got: format!("{:?}", decorators_value),
                    span: self.get_node_span(estree),
                })?;
                
                let mut decorators_vec = Vec::new_in(self.builder.allocator);
                for decorator_value in decorators_array {
                    self.context = self.context.clone().with_parent("PropertyDefinition", "decorators");
                    let decorator = self.convert_decorator(decorator_value)?;
                    decorators_vec.push(decorator);
                }
                decorators_vec
            }
        } else {
            Vec::new_in(self.builder.allocator)
        };

        // Get type_annotation (TypeScript)
        let type_annotation: Option<oxc_allocator::Box<'a, oxc_ast::ast::TSTypeAnnotation<'a>>> = if let Some(type_ann_value) = estree.get("typeAnnotation") {
            if type_ann_value.is_null() {
                None
            } else {
                self.context = self.context.clone().with_parent("AccessorProperty", "typeAnnotation");
                Some(self.convert_ts_type_annotation(type_ann_value)?)
            }
        } else {
            None
        };

        // Get type (AccessorPropertyType)
        let r#type = AccessorPropertyType::AccessorProperty;

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        let accessor_prop = self.builder.class_element_accessor_property(
            span,
            r#type,
            decorators,
            key,
            type_annotation,
            value,
            computed,
            r#static,
            r#override,
            definite,
            accessibility,
        );
        Ok(accessor_prop)
    }

    /// Convert an ESTree StaticBlock to oxc ClassElement.
    fn convert_static_block(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::ClassElement<'a>> {
        use oxc_ast::ast::ClassElement;

        // Get body
        self.context = self.context.clone().with_parent("StaticBlock", "body");
        let body_value = estree.get("body").ok_or_else(|| ConversionError::MissingField {
            field: "body".to_string(),
            node_type: "StaticBlock".to_string(),
            span: self.get_node_span(estree),
        })?;
        let body_array = body_value.as_array().ok_or_else(|| ConversionError::InvalidFieldType {
            field: "body".to_string(),
            expected: "array".to_string(),
            got: format!("{:?}", body_value),
            span: self.get_node_span(estree),
        })?;

        let mut statements = Vec::new_in(self.builder.allocator);
        for stmt_value in body_array {
            if stmt_value.is_null() {
                continue;
            }
            self.context = self.context.clone().with_parent("StaticBlock", "body");
            let stmt = self.convert_statement(stmt_value)?;
            statements.push(stmt);
        }

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        let static_block = self.builder.class_element_static_block(span, statements);
        Ok(static_block)
    }

    /// Convert an ESTree ImportDeclaration to oxc Statement.
    fn convert_import_declaration(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::Statement<'a>> {
        use oxc_ast::ast::{ImportDeclarationSpecifier, ImportOrExportKind, Statement};
        use oxc_span::Atom;

        // Get specifiers (optional)
        let specifiers = if let Some(specifiers_value) = estree.get("specifiers") {
            if specifiers_value.is_null() {
                None
            } else {
                let specifiers_array = specifiers_value.as_array().ok_or_else(|| ConversionError::InvalidFieldType {
                    field: "specifiers".to_string(),
                    expected: "array".to_string(),
                    got: format!("{:?}", specifiers_value),
                    span: self.get_node_span(estree),
                })?;

                let mut specifier_items = Vec::new_in(self.builder.allocator);
                for spec_value in specifiers_array {
                    if spec_value.is_null() {
                        continue;
                    }
                    self.context = self.context.clone().with_parent("ImportDeclaration", "specifiers");
                    let specifier = self.convert_import_specifier(spec_value)?;
                    specifier_items.push(specifier);
                }
                Some(specifier_items)
            }
        } else {
            None
        };

        // Get source
        self.context = self.context.clone().with_parent("ImportDeclaration", "source");
        let source_value = estree.get("source").ok_or_else(|| ConversionError::MissingField {
            field: "source".to_string(),
            node_type: "ImportDeclaration".to_string(),
            span: self.get_node_span(estree),
        })?;
        let source_literal = self.convert_string_literal(source_value)?;

        // Get importKind (optional, defaults to Value)
        let import_kind = estree.get("importKind")
            .and_then(|v| v.as_str())
            .map(|s| if s == "type" { ImportOrExportKind::Type } else { ImportOrExportKind::Value })
            .unwrap_or(ImportOrExportKind::Value);

        // Get phase (optional)
        let phase = estree.get("phase")
            .and_then(|v| v.as_str())
            .map(|s| if s == "defer" { oxc_ast::ast::ImportPhase::Defer } else { oxc_ast::ast::ImportPhase::Source });

        // Get attributes/with_clause (optional)
        // ESTree uses "attributes" field which can be an array directly or an ImportAttributes object
        let with_clause: Option<oxc_allocator::Box<'a, oxc_ast::ast::WithClause<'a>>> = if let Some(attributes_value) = estree.get("attributes") {
            if attributes_value.is_null() {
                None
            } else {
                self.context = self.context.clone().with_parent("ImportDeclaration", "attributes");
                
                // ESTree can have attributes as an array directly, or as an object with "attributes" field
                let attributes_array = if attributes_value.is_array() {
                    attributes_value.as_array().ok_or_else(|| ConversionError::InvalidFieldType {
                        field: "attributes".to_string(),
                        expected: "array".to_string(),
                        got: format!("{:?}", attributes_value),
                        span: self.get_node_span(estree),
                    })?
                } else if let Some(attrs_inner) = attributes_value.get("attributes") {
                    attrs_inner.as_array().ok_or_else(|| ConversionError::InvalidFieldType {
                        field: "attributes.attributes".to_string(),
                        expected: "array".to_string(),
                        got: format!("{:?}", attrs_inner),
                        span: self.get_node_span(estree),
                    })?
                } else {
                    return Err(ConversionError::InvalidFieldType {
                        field: "attributes".to_string(),
                        expected: "array or object with attributes field".to_string(),
                        got: format!("{:?}", attributes_value),
                        span: self.get_node_span(estree),
                    });
                };
                
                // Determine keyword (with or assert)
                // ESTree may have a "keyword" field, default to "with"
                let keyword_str = attributes_value.get("keyword")
                    .and_then(|v| v.as_str())
                    .unwrap_or("with");
                let keyword = match keyword_str {
                    "assert" => oxc_ast::ast::WithClauseKeyword::Assert,
                    "with" | _ => oxc_ast::ast::WithClauseKeyword::With,
                };
                
                let mut with_entries = Vec::new_in(self.builder.allocator);
                for attr_value in attributes_array {
                    self.context = self.context.clone().with_parent("WithClause", "with_entries");
                    let import_attr = self.convert_import_attribute(attr_value)?;
                    with_entries.push(import_attr);
                }
                
                let (start, end) = self.get_node_span(attributes_value);
                let span = Span::new(start, end);
                let with_clause_box = self.builder.alloc_with_clause(span, keyword, with_entries);
                Some(with_clause_box)
            }
        } else {
            None
        };

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        let import_decl = self.builder.module_declaration_import_declaration(
            span,
            specifiers,
            source_literal,
            phase,
            with_clause,
            import_kind,
        );
        // ModuleDeclaration variants are inherited by Statement
        match import_decl {
            oxc_ast::ast::ModuleDeclaration::ImportDeclaration(boxed) => {
                Ok(Statement::ImportDeclaration(boxed))
            }
            _ => unreachable!(),
        }
    }

    /// Convert an ESTree WithClause (attributes) to oxc WithClause.
    fn convert_with_clause(&mut self, estree: &Value) -> ConversionResult<oxc_allocator::Box<'a, oxc_ast::ast::WithClause<'a>>> {
        use oxc_ast::ast::{ImportAttribute, ImportAttributeKey, WithClause, WithClauseKeyword};
        use oxc_span::Atom;

        // Get keyword (optional, default to With)
        let keyword = estree.get("keyword").and_then(|v| v.as_str())
            .map(|s| match s {
                "assert" => WithClauseKeyword::Assert,
                "with" | _ => WithClauseKeyword::With,
            })
            .unwrap_or(WithClauseKeyword::With);

        // Get attributes array
        let attributes_value = estree.get("attributes").or_else(|| estree.get("withEntries"))
            .ok_or_else(|| ConversionError::MissingField {
                field: "attributes".to_string(),
                node_type: "WithClause".to_string(),
                span: self.get_node_span(estree),
            })?;
        let attributes_array = attributes_value.as_array().ok_or_else(|| ConversionError::InvalidFieldType {
            field: "attributes".to_string(),
            expected: "array".to_string(),
            got: format!("{:?}", attributes_value),
            span: self.get_node_span(estree),
        })?;

        let mut with_entries = Vec::new_in(self.builder.allocator);
        for attr_value in attributes_array {
            if attr_value.is_null() {
                continue;
            }
            self.context = self.context.clone().with_parent("WithClause", "attributes");
            let attr = self.convert_import_attribute(attr_value)?;
            with_entries.push(attr);
        }

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        let with_clause = self.builder.with_clause(span, keyword, with_entries);
        Ok(oxc_allocator::Box::new_in(with_clause, self.builder.allocator))
    }

    /// Convert an ESTree ImportAttribute to oxc ImportAttribute.
    fn convert_import_attribute(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::ImportAttribute<'a>> {
        use oxc_ast::ast::{ImportAttribute, ImportAttributeKey};
        use oxc_span::Atom;

        // Get key (Identifier or StringLiteral)
        self.context = self.context.clone().with_parent("ImportAttribute", "key");
        let key_value = estree.get("key").ok_or_else(|| ConversionError::MissingField {
            field: "key".to_string(),
            node_type: "ImportAttribute".to_string(),
            span: self.get_node_span(estree),
        })?;
        
        let key = if let Some(name_str) = key_value.get("name").and_then(|v| v.as_str()) {
            // Identifier
            let name = Atom::from_in(name_str, self.builder.allocator);
            let range = key_value.get("range").and_then(|v| v.as_array())
                .and_then(|arr| {
                    if arr.len() >= 2 {
                        Some([arr[0].as_u64()? as usize, arr[1].as_u64()? as usize])
                    } else {
                        None
                    }
                });
            let span = convert_span(self.source_text, range.unwrap_or([0, 0])[0], range.unwrap_or([0, 0])[1]);
            let ident = self.builder.identifier_name(span, name);
            ImportAttributeKey::Identifier(ident)
        } else if let Some(value_str) = key_value.get("value").and_then(|v| v.as_str()) {
            // StringLiteral
            let value = Atom::from_in(value_str, self.builder.allocator);
            let raw = key_value.get("raw").and_then(|v| v.as_str())
                .map(|s| Atom::from_in(s, self.builder.allocator));
            let range = key_value.get("range").and_then(|v| v.as_array())
                .and_then(|arr| {
                    if arr.len() >= 2 {
                        Some([arr[0].as_u64()? as usize, arr[1].as_u64()? as usize])
                    } else {
                        None
                    }
                });
            let span = convert_span(self.source_text, range.unwrap_or([0, 0])[0], range.unwrap_or([0, 0])[1]);
            let string_lit = self.builder.string_literal(span, value, raw);
            ImportAttributeKey::StringLiteral(string_lit)
        } else {
            return Err(ConversionError::InvalidFieldType {
                field: "key".to_string(),
                expected: "Identifier or StringLiteral".to_string(),
                got: format!("{:?}", key_value),
                span: self.get_node_span(estree),
            });
        };

        // Get value (StringLiteral)
        self.context = self.context.clone().with_parent("ImportAttribute", "value");
        let value_value = estree.get("value").ok_or_else(|| ConversionError::MissingField {
            field: "value".to_string(),
            node_type: "ImportAttribute".to_string(),
            span: self.get_node_span(estree),
        })?;
        let value = self.convert_string_literal(value_value)?;

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        let import_attr = self.builder.import_attribute(span, key, value);
        Ok(import_attr)
    }

    /// Convert an ESTree import specifier to oxc ImportDeclarationSpecifier.
    fn convert_import_specifier(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::ImportDeclarationSpecifier<'a>> {
        use oxc_ast::ast::{BindingIdentifier, ImportDeclarationSpecifier, ImportOrExportKind, ModuleExportName};
        use oxc_estree::deserialize::{EstreeNode, EstreeNodeType};
        use oxc_span::Atom;

        let node_type = <Value as EstreeNode>::get_type(estree).ok_or_else(|| ConversionError::MissingField {
            field: "type".to_string(),
            node_type: "ImportSpecifier".to_string(),
            span: self.get_node_span(estree),
        })?;

        match node_type {
            EstreeNodeType::ImportSpecifier => {
                // Get imported (can be Identifier or StringLiteral)
                self.context = self.context.clone().with_parent("ImportSpecifier", "imported");
                let imported_value = estree.get("imported").ok_or_else(|| ConversionError::MissingField {
                    field: "imported".to_string(),
                    node_type: "ImportSpecifier".to_string(),
                    span: self.get_node_span(estree),
                })?;
                let imported = self.convert_module_export_name(imported_value)?;

                // Get local
                self.context = self.context.clone().with_parent("ImportSpecifier", "local");
                let local_value = estree.get("local").ok_or_else(|| ConversionError::MissingField {
                    field: "local".to_string(),
                    node_type: "ImportSpecifier".to_string(),
                    span: self.get_node_span(estree),
                })?;
                let local = self.convert_binding_identifier(local_value)?;

                // Get importKind (optional)
                let import_kind = estree.get("importKind")
                    .and_then(|v| v.as_str())
                    .map(|s| if s == "type" { ImportOrExportKind::Type } else { ImportOrExportKind::Value })
                    .unwrap_or(ImportOrExportKind::Value);

                let (start, end) = self.get_node_span(estree);
                let span = Span::new(start, end);

                let import_spec = self.builder.alloc_import_specifier(span, imported, local, import_kind);
                Ok(ImportDeclarationSpecifier::ImportSpecifier(import_spec))
            }
            EstreeNodeType::ImportDefaultSpecifier => {
                // Get local
                self.context = self.context.clone().with_parent("ImportDefaultSpecifier", "local");
                let local_value = estree.get("local").ok_or_else(|| ConversionError::MissingField {
                    field: "local".to_string(),
                    node_type: "ImportDefaultSpecifier".to_string(),
                    span: self.get_node_span(estree),
                })?;
                let local = self.convert_binding_identifier(local_value)?;

                let (start, end) = self.get_node_span(estree);
                let span = Span::new(start, end);

                let default_spec = self.builder.alloc_import_default_specifier(span, local);
                Ok(ImportDeclarationSpecifier::ImportDefaultSpecifier(default_spec))
            }
            EstreeNodeType::ImportNamespaceSpecifier => {
                // Get local
                self.context = self.context.clone().with_parent("ImportNamespaceSpecifier", "local");
                let local_value = estree.get("local").ok_or_else(|| ConversionError::MissingField {
                    field: "local".to_string(),
                    node_type: "ImportNamespaceSpecifier".to_string(),
                    span: self.get_node_span(estree),
                })?;
                let local = self.convert_binding_identifier(local_value)?;

                let (start, end) = self.get_node_span(estree);
                let span = Span::new(start, end);

                let namespace_spec = self.builder.alloc_import_namespace_specifier(span, local);
                Ok(ImportDeclarationSpecifier::ImportNamespaceSpecifier(namespace_spec))
            }
            _ => Err(ConversionError::UnsupportedNodeType {
                node_type: format!("{:?}", node_type),
                span: self.get_node_span(estree),
            }),
        }
    }

    /// Convert an ESTree node to oxc ModuleExportName.
    fn convert_module_export_name(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::ModuleExportName<'a>> {
        use oxc_ast::ast::{ModuleExportName, StringLiteral};
        use oxc_estree::deserialize::{EstreeNode, EstreeNodeType};

        let node_type = <Value as EstreeNode>::get_type(estree).ok_or_else(|| ConversionError::MissingField {
            field: "type".to_string(),
            node_type: "ModuleExportName".to_string(),
            span: self.get_node_span(estree),
        })?;

        match node_type {
            EstreeNodeType::Identifier => {
                let ident = self.convert_identifier_to_name(estree)?;
                Ok(ModuleExportName::IdentifierName(ident))
            }
            EstreeNodeType::Literal => {
                let literal = self.convert_string_literal(estree)?;
                Ok(ModuleExportName::StringLiteral(literal))
            }
            _ => Err(ConversionError::UnsupportedNodeType {
                node_type: format!("{:?}", node_type),
                span: self.get_node_span(estree),
            }),
        }
    }

    /// Convert an ESTree ExportNamedDeclaration to oxc Statement.
    fn convert_export_named_declaration(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::Statement<'a>> {
        use oxc_ast::ast::{Declaration, ExportSpecifier, ImportOrExportKind, Statement};

        // Get declaration (optional)
        let declaration = if let Some(decl_value) = estree.get("declaration") {
            if decl_value.is_null() {
                None
            } else {
                self.context = self.context.clone().with_parent("ExportNamedDeclaration", "declaration");
                Some(self.convert_to_declaration(decl_value)?)
            }
        } else {
            None
        };

        // Get specifiers
        let empty_array = serde_json::Value::Array(vec![]);
        let specifiers_value = estree.get("specifiers").unwrap_or(&empty_array);
        let specifiers_array = specifiers_value.as_array().ok_or_else(|| ConversionError::InvalidFieldType {
            field: "specifiers".to_string(),
            expected: "array".to_string(),
            got: format!("{:?}", specifiers_value),
            span: self.get_node_span(estree),
        })?;

        let mut specifier_items = Vec::new_in(self.builder.allocator);
        for spec_value in specifiers_array {
            if spec_value.is_null() {
                continue;
            }
            self.context = self.context.clone().with_parent("ExportNamedDeclaration", "specifiers");
            let specifier = self.convert_export_specifier(spec_value)?;
            specifier_items.push(specifier);
        }

        // Get source (optional)
        let source = if let Some(source_value) = estree.get("source") {
            if source_value.is_null() {
                None
            } else {
                self.context = self.context.clone().with_parent("ExportNamedDeclaration", "source");
                Some(self.convert_string_literal(source_value)?)
            }
        } else {
            None
        };

        // Get exportKind (optional, defaults to Value)
        let export_kind = estree.get("exportKind")
            .and_then(|v| v.as_str())
            .map(|s| if s == "type" { ImportOrExportKind::Type } else { ImportOrExportKind::Value })
            .unwrap_or(ImportOrExportKind::Value);

        // Get attributes/with_clause (optional)
        let with_clause = if let Some(attributes_value) = estree.get("attributes") {
            if attributes_value.is_null() {
                None
            } else {
                self.context = self.context.clone().with_parent("ExportNamedDeclaration", "attributes");
                // Check if it's an array (direct format) or an object (ImportAttributes wrapper)
                if attributes_value.is_array() {
                    // Direct array format - wrap in a WithClause object
                    let mut with_entries = Vec::new_in(self.builder.allocator);
                    for attr_value in attributes_value.as_array().unwrap() {
                        if attr_value.is_null() {
                            continue;
                        }
                        self.context = self.context.clone().with_parent("ExportNamedDeclaration", "attributes");
                        let attr = self.convert_import_attribute(attr_value)?;
                        with_entries.push(attr);
                    }
                    let (start, end) = self.get_node_span(estree);
                    let span = Span::new(start, end);
                    let with_clause = self.builder.with_clause(span, oxc_ast::ast::WithClauseKeyword::With, with_entries);
                    Some(oxc_allocator::Box::new_in(with_clause, self.builder.allocator))
                } else {
                    // ImportAttributes object format
                    Some(self.convert_with_clause(attributes_value)?)
                }
            }
        } else {
            None
        };

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        let export_decl = self.builder.module_declaration_export_named_declaration(
            span,
            declaration,
            specifier_items,
            source,
            export_kind,
            with_clause,
        );
        match export_decl {
            oxc_ast::ast::ModuleDeclaration::ExportNamedDeclaration(boxed) => {
                Ok(Statement::ExportNamedDeclaration(boxed))
            }
            _ => unreachable!(),
        }
    }

    /// Convert an ESTree ExportSpecifier to oxc ExportSpecifier.
    fn convert_export_specifier(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::ExportSpecifier<'a>> {
        use oxc_ast::ast::{ExportSpecifier, ImportOrExportKind, ModuleExportName};

        // Get exported (can be Identifier or StringLiteral)
        self.context = self.context.clone().with_parent("ExportSpecifier", "exported");
        let exported_value = estree.get("exported").ok_or_else(|| ConversionError::MissingField {
            field: "exported".to_string(),
            node_type: "ExportSpecifier".to_string(),
            span: self.get_node_span(estree),
        })?;
        let exported = self.convert_module_export_name(exported_value)?;

        // Get local (optional, defaults to exported)
        // In ESTree, if local is missing, it's the same as exported
        // For oxc, local should be IdentifierReference if it's an identifier
        let local = if let Some(local_value) = estree.get("local") {
            if local_value.is_null() {
                // Use exported as local, but convert IdentifierName to IdentifierReference
                match &exported {
                    ModuleExportName::IdentifierName(ident) => {
                        ModuleExportName::IdentifierReference(
                            self.builder.identifier_reference(ident.span, ident.name)
                        )
                    }
                    _ => exported.clone_in(self.builder.allocator),
                }
            } else {
                self.context = self.context.clone().with_parent("ExportSpecifier", "local");
                let local_name = self.convert_module_export_name(local_value)?;
                // For local, use IdentifierReference if it's an identifier
                match local_name {
                    ModuleExportName::IdentifierName(ident) => {
                        ModuleExportName::IdentifierReference(
                            self.builder.identifier_reference(ident.span, ident.name)
                        )
                    }
                    other => other,
                }
            }
        } else {
            // If no local, use exported as local (for `export { foo }`)
            // Convert IdentifierName to IdentifierReference for local
            match &exported {
                ModuleExportName::IdentifierName(ident) => {
                    ModuleExportName::IdentifierReference(
                        self.builder.identifier_reference(ident.span, ident.name)
                    )
                }
                _ => exported.clone_in(self.builder.allocator),
            }
        };

        // Get exportKind (optional)
        let export_kind = estree.get("exportKind")
            .and_then(|v| v.as_str())
            .map(|s| if s == "type" { ImportOrExportKind::Type } else { ImportOrExportKind::Value })
            .unwrap_or(ImportOrExportKind::Value);

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        let export_spec = self.builder.export_specifier(span, local, exported, export_kind);
        Ok(export_spec)
    }

    /// Convert an ESTree ExportDefaultDeclaration to oxc Statement.
    fn convert_export_default_declaration(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::Statement<'a>> {
        use oxc_ast::ast::{ExportDefaultDeclarationKind, Statement};
        use oxc_estree::deserialize::{EstreeNode, EstreeNodeType};

        // Get declaration
        self.context = self.context.clone().with_parent("ExportDefaultDeclaration", "declaration");
        let decl_value = estree.get("declaration").ok_or_else(|| ConversionError::MissingField {
            field: "declaration".to_string(),
            node_type: "ExportDefaultDeclaration".to_string(),
            span: self.get_node_span(estree),
        })?;

        // Convert declaration to ExportDefaultDeclarationKind
        // ExportDefaultDeclarationKind can be FunctionDeclaration, ClassDeclaration, TSInterfaceDeclaration, or any Expression
        let node_type = <Value as EstreeNode>::get_type(decl_value).ok_or_else(|| ConversionError::MissingField {
            field: "type".to_string(),
            node_type: "ExportDefaultDeclaration.declaration".to_string(),
            span: self.get_node_span(decl_value),
        })?;

        let decl_kind = match node_type {
            EstreeNodeType::FunctionDeclaration => {
                let function = self.convert_function(decl_value, oxc_ast::ast::FunctionType::FunctionDeclaration)?;
                ExportDefaultDeclarationKind::FunctionDeclaration(function)
            }
            EstreeNodeType::ClassDeclaration => {
                let class = self.convert_class(decl_value, oxc_ast::ast::ClassType::ClassDeclaration)?;
                ExportDefaultDeclarationKind::ClassDeclaration(class)
            }
            // For expressions, use convert_expression which returns Expression
            // ExportDefaultDeclarationKind inherits from Expression, so we can match on Expression variants
            _ => {
                // Try converting as expression
                let expr = self.convert_expression(decl_value)?;
                // Convert Expression to ExportDefaultDeclarationKind
                // Since ExportDefaultDeclarationKind inherits from Expression, we need to match and convert
                match expr {
                    oxc_ast::ast::Expression::Identifier(ident) => {
                        ExportDefaultDeclarationKind::Identifier(ident)
                    }
                    oxc_ast::ast::Expression::BooleanLiteral(lit) => {
                        ExportDefaultDeclarationKind::BooleanLiteral(lit)
                    }
                    oxc_ast::ast::Expression::NullLiteral(lit) => {
                        ExportDefaultDeclarationKind::NullLiteral(lit)
                    }
                    oxc_ast::ast::Expression::NumericLiteral(lit) => {
                        ExportDefaultDeclarationKind::NumericLiteral(lit)
                    }
                    oxc_ast::ast::Expression::StringLiteral(lit) => {
                        ExportDefaultDeclarationKind::StringLiteral(lit)
                    }
                    oxc_ast::ast::Expression::ArrayExpression(expr) => {
                        ExportDefaultDeclarationKind::ArrayExpression(expr)
                    }
                    oxc_ast::ast::Expression::ObjectExpression(expr) => {
                        ExportDefaultDeclarationKind::ObjectExpression(expr)
                    }
                    oxc_ast::ast::Expression::CallExpression(expr) => {
                        ExportDefaultDeclarationKind::CallExpression(expr)
                    }
                    oxc_ast::ast::Expression::NewExpression(expr) => {
                        ExportDefaultDeclarationKind::NewExpression(expr)
                    }
                    oxc_ast::ast::Expression::StaticMemberExpression(expr) => {
                        ExportDefaultDeclarationKind::StaticMemberExpression(expr)
                    }
                    oxc_ast::ast::Expression::ComputedMemberExpression(expr) => {
                        ExportDefaultDeclarationKind::ComputedMemberExpression(expr)
                    }
                    oxc_ast::ast::Expression::PrivateFieldExpression(expr) => {
                        ExportDefaultDeclarationKind::PrivateFieldExpression(expr)
                    }
                    oxc_ast::ast::Expression::BinaryExpression(expr) => {
                        ExportDefaultDeclarationKind::BinaryExpression(expr)
                    }
                    oxc_ast::ast::Expression::UnaryExpression(expr) => {
                        ExportDefaultDeclarationKind::UnaryExpression(expr)
                    }
                    oxc_ast::ast::Expression::UpdateExpression(expr) => {
                        ExportDefaultDeclarationKind::UpdateExpression(expr)
                    }
                    oxc_ast::ast::Expression::LogicalExpression(expr) => {
                        ExportDefaultDeclarationKind::LogicalExpression(expr)
                    }
                    oxc_ast::ast::Expression::ConditionalExpression(expr) => {
                        ExportDefaultDeclarationKind::ConditionalExpression(expr)
                    }
                    oxc_ast::ast::Expression::AssignmentExpression(expr) => {
                        ExportDefaultDeclarationKind::AssignmentExpression(expr)
                    }
                    oxc_ast::ast::Expression::SequenceExpression(expr) => {
                        ExportDefaultDeclarationKind::SequenceExpression(expr)
                    }
                    oxc_ast::ast::Expression::ThisExpression(expr) => {
                        ExportDefaultDeclarationKind::ThisExpression(expr)
                    }
                    oxc_ast::ast::Expression::Super(expr) => {
                        ExportDefaultDeclarationKind::Super(expr)
                    }
                    oxc_ast::ast::Expression::YieldExpression(expr) => {
                        ExportDefaultDeclarationKind::YieldExpression(expr)
                    }
                    oxc_ast::ast::Expression::AwaitExpression(expr) => {
                        ExportDefaultDeclarationKind::AwaitExpression(expr)
                    }
                    oxc_ast::ast::Expression::TemplateLiteral(expr) => {
                        ExportDefaultDeclarationKind::TemplateLiteral(expr)
                    }
                    oxc_ast::ast::Expression::TaggedTemplateExpression(expr) => {
                        ExportDefaultDeclarationKind::TaggedTemplateExpression(expr)
                    }
                    oxc_ast::ast::Expression::ArrowFunctionExpression(expr) => {
                        ExportDefaultDeclarationKind::ArrowFunctionExpression(expr)
                    }
                    oxc_ast::ast::Expression::FunctionExpression(expr) => {
                        ExportDefaultDeclarationKind::FunctionExpression(expr)
                    }
                    oxc_ast::ast::Expression::ClassExpression(expr) => {
                        ExportDefaultDeclarationKind::ClassExpression(expr)
                    }
                    _ => {
                        return Err(ConversionError::UnsupportedNodeType {
                            node_type: format!("ExportDefaultDeclaration.declaration: {:?}", node_type),
                            span: self.get_node_span(decl_value),
                        });
                    }
                }
            }
        };

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        let export_default_decl = self.builder.module_declaration_export_default_declaration(span, decl_kind);
        match export_default_decl {
            oxc_ast::ast::ModuleDeclaration::ExportDefaultDeclaration(boxed) => {
                Ok(Statement::ExportDefaultDeclaration(boxed))
            }
            _ => unreachable!(),
        }
    }

    /// Convert an ESTree ExportAllDeclaration to oxc Statement.
    fn convert_export_all_declaration(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::Statement<'a>> {
        use oxc_ast::ast::{ImportOrExportKind, ModuleExportName, Statement};

        // Get exported (optional)
        let exported = if let Some(exported_value) = estree.get("exported") {
            if exported_value.is_null() {
                None
            } else {
                self.context = self.context.clone().with_parent("ExportAllDeclaration", "exported");
                Some(self.convert_module_export_name(exported_value)?)
            }
        } else {
            None
        };

        // Get source
        self.context = self.context.clone().with_parent("ExportAllDeclaration", "source");
        let source_value = estree.get("source").ok_or_else(|| ConversionError::MissingField {
            field: "source".to_string(),
            node_type: "ExportAllDeclaration".to_string(),
            span: self.get_node_span(estree),
        })?;
        let source = self.convert_string_literal(source_value)?;

        // Get exportKind (optional, defaults to Value)
        let export_kind = estree.get("exportKind")
            .and_then(|v| v.as_str())
            .map(|s| if s == "type" { ImportOrExportKind::Type } else { ImportOrExportKind::Value })
            .unwrap_or(ImportOrExportKind::Value);

        // Get attributes/with_clause (optional, None for now)
        let with_clause: Option<oxc_allocator::Box<'a, oxc_ast::ast::WithClause<'a>>> = None;

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        let export_all_decl = self.builder.module_declaration_export_all_declaration(
            span,
            exported,
            source,
            with_clause,
            export_kind,
        );
        match export_all_decl {
            oxc_ast::ast::ModuleDeclaration::ExportAllDeclaration(boxed) => {
                Ok(Statement::ExportAllDeclaration(boxed))
            }
            _ => unreachable!(),
        }
    }

    /// Convert an ESTree Identifier to oxc IdentifierName.
    fn convert_identifier_to_name(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::IdentifierName<'a>> {
        use oxc_ast::ast::IdentifierName;
        use oxc_span::Atom;

        let estree_id = oxc_estree::deserialize::EstreeIdentifier::from_json(estree)
            .ok_or_else(|| ConversionError::InvalidFieldType {
                field: "Identifier".to_string(),
                expected: "valid Identifier node".to_string(),
                got: format!("{:?}", estree),
                span: self.get_node_span(estree),
            })?;

        let name = Atom::from_in(estree_id.name.as_str(), self.builder.allocator);
        let range = estree_id.range.unwrap_or([0, 0]);
        let span = convert_span(self.source_text, range[0] as usize, range[1] as usize);
        Ok(self.builder.identifier_name(span, name))
    }

    /// Convert an ESTree Identifier to oxc BindingIdentifier.
    fn convert_binding_identifier(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::BindingIdentifier<'a>> {
        use oxc_ast::ast::BindingIdentifier;
        use oxc_span::Atom;

        let estree_id = oxc_estree::deserialize::EstreeIdentifier::from_json(estree)
            .ok_or_else(|| ConversionError::InvalidFieldType {
                field: "Identifier".to_string(),
                expected: "valid Identifier node".to_string(),
                got: format!("{:?}", estree),
                span: self.get_node_span(estree),
            })?;

        let name = Atom::from_in(estree_id.name.as_str(), self.builder.allocator);
        let range = estree_id.range.unwrap_or([0, 0]);
        let span = convert_span(self.source_text, range[0] as usize, range[1] as usize);
        Ok(self.builder.binding_identifier(span, name))
    }

    /// Convert an ESTree Literal to oxc StringLiteral.
    fn convert_string_literal(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::StringLiteral<'a>> {
        use oxc_ast::ast::StringLiteral;
        use oxc_span::Atom;

        let estree_literal = oxc_estree::deserialize::EstreeLiteral::from_json(estree)
            .ok_or_else(|| ConversionError::InvalidFieldType {
                field: "Literal".to_string(),
                expected: "valid Literal node".to_string(),
                got: format!("{:?}", estree),
                span: self.get_node_span(estree),
            })?;

        let value_str = oxc_estree::deserialize::get_string_value(&estree_literal)?;

        let value = Atom::from_in(value_str, self.builder.allocator);
        let raw = estree_literal.raw.as_deref().map(|s| Atom::from_in(s, self.builder.allocator));
        let range = estree_literal.range.unwrap_or([0, 0]);
        let span = convert_span(self.source_text, range[0] as usize, range[1] as usize);
        Ok(self.builder.string_literal(span, value, raw))
    }

    /// Convert an ESTree TSInterfaceDeclaration to oxc Statement.
    fn convert_ts_interface_declaration(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::Statement<'a>> {
        use oxc_ast::ast::{Statement, TSInterfaceBody, TSInterfaceHeritage};

        // Get id
        self.context = self.context.clone().with_parent("TSInterfaceDeclaration", "id");
        let id_value = estree.get("id").ok_or_else(|| ConversionError::MissingField {
            field: "id".to_string(),
            node_type: "TSInterfaceDeclaration".to_string(),
            span: self.get_node_span(estree),
        })?;
        let id = self.convert_binding_identifier(id_value)?;

        // Get typeParameters (optional)
        let type_parameters: Option<oxc_allocator::Box<'a, oxc_ast::ast::TSTypeParameterDeclaration<'a>>> = if let Some(type_params_value) = estree.get("typeParameters") {
            if type_params_value.is_null() {
                None
            } else {
                self.context = self.context.clone().with_parent("TSInterfaceDeclaration", "typeParameters");
                Some(self.convert_ts_type_parameter_declaration(type_params_value)?)
            }
        } else {
            None
        };

        // Get extends (optional array of TSInterfaceHeritage)
        let extends = if let Some(extends_value) = estree.get("extends") {
            if extends_value.is_null() {
                Vec::new_in(self.builder.allocator)
            } else {
                let extends_array = extends_value.as_array().ok_or_else(|| ConversionError::InvalidFieldType {
                    field: "extends".to_string(),
                    expected: "array".to_string(),
                    got: format!("{:?}", extends_value),
                    span: self.get_node_span(estree),
                })?;
                
                let mut extends_vec = Vec::new_in(self.builder.allocator);
                for extend_value in extends_array {
                    self.context = self.context.clone().with_parent("TSInterfaceDeclaration", "extends");
                    let heritage = self.convert_ts_interface_heritage(extend_value)?;
                    extends_vec.push(heritage);
                }
                extends_vec
            }
        } else {
            Vec::new_in(self.builder.allocator)
        };

        // Get body
        self.context = self.context.clone().with_parent("TSInterfaceDeclaration", "body");
        let body_value = estree.get("body").ok_or_else(|| ConversionError::MissingField {
            field: "body".to_string(),
            node_type: "TSInterfaceDeclaration".to_string(),
            span: self.get_node_span(estree),
        })?;
        let body = self.convert_ts_interface_body(body_value)?;

        // Get declare (optional)
        let declare = estree.get("declare").and_then(|v| v.as_bool()).unwrap_or(false);

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        let interface_decl_box = self.builder.alloc_ts_interface_declaration(
            span,
            id,
            type_parameters,
            extends,
            body,
            declare,
        );
        let interface_decl = oxc_ast::ast::Declaration::TSInterfaceDeclaration(interface_decl_box);
        match interface_decl {
            oxc_ast::ast::Declaration::TSInterfaceDeclaration(boxed) => {
                Ok(Statement::TSInterfaceDeclaration(boxed))
            }
            _ => unreachable!(),
        }
    }

    /// Convert an ESTree TSInterfaceBody to oxc TSInterfaceBody.
    fn convert_ts_interface_body(&mut self, estree: &Value) -> ConversionResult<oxc_allocator::Box<'a, oxc_ast::ast::TSInterfaceBody<'a>>> {
        use oxc_ast::ast::{TSSignature, TSInterfaceBody};

        // Get body array
        let body_value = estree.get("body").ok_or_else(|| ConversionError::MissingField {
            field: "body".to_string(),
            node_type: "TSInterfaceBody".to_string(),
            span: self.get_node_span(estree),
        })?;
        let _body_array = body_value.as_array().ok_or_else(|| ConversionError::InvalidFieldType {
            field: "body".to_string(),
            expected: "array".to_string(),
            got: format!("{:?}", body_value),
            span: self.get_node_span(estree),
        })?;

        // For now, skip interface body members (TSSignature conversion is complex)
        let signatures = Vec::new_in(self.builder.allocator);

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        let interface_body = self.builder.ts_interface_body(span, signatures);
        Ok(oxc_allocator::Box::new_in(interface_body, self.builder.allocator))
    }

    /// Convert an ESTree TSEnumDeclaration to oxc Statement.
    fn convert_ts_enum_declaration(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::Statement<'a>> {
        use oxc_ast::ast::{Statement, TSEnumBody};

        // Get id
        self.context = self.context.clone().with_parent("TSEnumDeclaration", "id");
        let id_value = estree.get("id").ok_or_else(|| ConversionError::MissingField {
            field: "id".to_string(),
            node_type: "TSEnumDeclaration".to_string(),
            span: self.get_node_span(estree),
        })?;
        let id = self.convert_binding_identifier(id_value)?;

        // Get body
        self.context = self.context.clone().with_parent("TSEnumDeclaration", "body");
        let body_value = estree.get("body").ok_or_else(|| ConversionError::MissingField {
            field: "body".to_string(),
            node_type: "TSEnumDeclaration".to_string(),
            span: self.get_node_span(estree),
        })?;
        let body = self.convert_ts_enum_body(body_value)?;

        // Get const (optional)
        let r#const = estree.get("const").and_then(|v| v.as_bool()).unwrap_or(false);

        // Get declare (optional)
        let declare = estree.get("declare").and_then(|v| v.as_bool()).unwrap_or(false);

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        let enum_decl_box = self.builder.alloc_ts_enum_declaration(span, id, body, r#const, declare);
        let enum_decl = oxc_ast::ast::Declaration::TSEnumDeclaration(enum_decl_box);
        match enum_decl {
            oxc_ast::ast::Declaration::TSEnumDeclaration(boxed) => {
                Ok(Statement::TSEnumDeclaration(boxed))
            }
            _ => unreachable!(),
        }
    }

    /// Convert an ESTree TSEnumBody to oxc TSEnumBody.
    fn convert_ts_enum_body(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::TSEnumBody<'a>> {
        use oxc_ast::ast::{TSEnumBody, TSEnumMember};

        // Get members array
        let members_value = estree.get("members").ok_or_else(|| ConversionError::MissingField {
            field: "members".to_string(),
            node_type: "TSEnumBody".to_string(),
            span: self.get_node_span(estree),
        })?;
        let _members_array = members_value.as_array().ok_or_else(|| ConversionError::InvalidFieldType {
            field: "members".to_string(),
            expected: "array".to_string(),
            got: format!("{:?}", members_value),
            span: self.get_node_span(estree),
        })?;

        // For now, skip enum members (TSEnumMember conversion is complex)
        let members = Vec::new_in(self.builder.allocator);

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        let enum_body = self.builder.ts_enum_body(span, members);
        Ok(enum_body)
    }

    /// Convert an ESTree TSTypeAliasDeclaration to oxc Statement.
    fn convert_ts_type_alias_declaration(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::Statement<'a>> {
        use oxc_ast::ast::{Statement, TSType};

        // Get id
        self.context = self.context.clone().with_parent("TSTypeAliasDeclaration", "id");
        let id_value = estree.get("id").ok_or_else(|| ConversionError::MissingField {
            field: "id".to_string(),
            node_type: "TSTypeAliasDeclaration".to_string(),
            span: self.get_node_span(estree),
        })?;
        let id = self.convert_binding_identifier(id_value)?;

        // Get typeParameters (optional)
        let type_parameters: Option<oxc_allocator::Box<'a, oxc_ast::ast::TSTypeParameterDeclaration<'a>>> = if let Some(type_params_value) = estree.get("typeParameters") {
            if type_params_value.is_null() {
                None
            } else {
                self.context = self.context.clone().with_parent("TSTypeAliasDeclaration", "typeParameters");
                Some(self.convert_ts_type_parameter_declaration(type_params_value)?)
            }
        } else {
            None
        };

        // Get typeAnnotation (required)
        self.context = self.context.clone().with_parent("TSTypeAliasDeclaration", "typeAnnotation");
        let type_annotation_value = estree.get("typeAnnotation").ok_or_else(|| ConversionError::MissingField {
            field: "typeAnnotation".to_string(),
            node_type: "TSTypeAliasDeclaration".to_string(),
            span: self.get_node_span(estree),
        })?;
        let type_annotation = self.convert_ts_type(type_annotation_value)?;

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        // Get declare flag (optional, defaults to false)
        let declare = estree.get("declare")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let type_alias = self.builder.ts_type_alias_declaration(span, id, type_parameters, type_annotation, declare);
        let type_alias_box = oxc_allocator::Box::new_in(type_alias, self.builder.allocator);
        Ok(Statement::TSTypeAliasDeclaration(type_alias_box))
    }

    /// Convert an ESTree TSType to oxc TSType.
    /// This handles all TypeScript type nodes (keywords, compound types, etc.).
    fn convert_ts_type(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::TSType<'a>> {
        use oxc_ast::ast::TSType;
        use oxc_estree::deserialize::{EstreeNode, EstreeNodeType};

        let node_type_str = estree.get("type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ConversionError::MissingField {
                field: "type".to_string(),
                node_type: "TSType".to_string(),
                span: self.get_node_span(estree),
            })?;

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        match node_type_str {
            // Keyword types
            "TSAnyKeyword" => {
                let keyword = self.builder.alloc_ts_any_keyword(span);
                Ok(TSType::TSAnyKeyword(keyword))
            }
            "TSBigIntKeyword" => {
                let keyword = self.builder.alloc_ts_big_int_keyword(span);
                Ok(TSType::TSBigIntKeyword(keyword))
            }
            "TSBooleanKeyword" => {
                let keyword = self.builder.alloc_ts_boolean_keyword(span);
                Ok(TSType::TSBooleanKeyword(keyword))
            }
            "TSIntrinsicKeyword" => {
                let keyword = self.builder.alloc_ts_intrinsic_keyword(span);
                Ok(TSType::TSIntrinsicKeyword(keyword))
            }
            "TSNeverKeyword" => {
                let keyword = self.builder.alloc_ts_never_keyword(span);
                Ok(TSType::TSNeverKeyword(keyword))
            }
            "TSNullKeyword" => {
                let keyword = self.builder.alloc_ts_null_keyword(span);
                Ok(TSType::TSNullKeyword(keyword))
            }
            "TSNumberKeyword" => {
                let keyword = self.builder.alloc_ts_number_keyword(span);
                Ok(TSType::TSNumberKeyword(keyword))
            }
            "TSObjectKeyword" => {
                let keyword = self.builder.alloc_ts_object_keyword(span);
                Ok(TSType::TSObjectKeyword(keyword))
            }
            "TSStringKeyword" => {
                let keyword = self.builder.alloc_ts_string_keyword(span);
                Ok(TSType::TSStringKeyword(keyword))
            }
            "TSSymbolKeyword" => {
                let keyword = self.builder.alloc_ts_symbol_keyword(span);
                Ok(TSType::TSSymbolKeyword(keyword))
            }
            "TSUndefinedKeyword" => {
                let keyword = self.builder.alloc_ts_undefined_keyword(span);
                Ok(TSType::TSUndefinedKeyword(keyword))
            }
            "TSUnknownKeyword" => {
                let keyword = self.builder.alloc_ts_unknown_keyword(span);
                Ok(TSType::TSUnknownKeyword(keyword))
            }
            "TSVoidKeyword" => {
                let keyword = self.builder.alloc_ts_void_keyword(span);
                Ok(TSType::TSVoidKeyword(keyword))
            }
            "TSThisType" => {
                let this_type = self.builder.alloc_ts_this_type(span);
                Ok(TSType::TSThisType(this_type))
            }
            // Compound types
            "TSArrayType" => {
                self.context = self.context.clone().with_parent("TSArrayType", "elementType");
                let element_type_value = estree.get("elementType").ok_or_else(|| ConversionError::MissingField {
                    field: "elementType".to_string(),
                    node_type: "TSArrayType".to_string(),
                    span: self.get_node_span(estree),
                })?;
                let element_type = self.convert_ts_type(element_type_value)?;
                let array_type = self.builder.alloc_ts_array_type(span, element_type);
                Ok(TSType::TSArrayType(array_type))
            }
            "TSUnionType" => {
                let types_value = estree.get("types").ok_or_else(|| ConversionError::MissingField {
                    field: "types".to_string(),
                    node_type: "TSUnionType".to_string(),
                    span: self.get_node_span(estree),
                })?;
                let types_array = types_value.as_array().ok_or_else(|| ConversionError::InvalidFieldType {
                    field: "types".to_string(),
                    expected: "array".to_string(),
                    got: format!("{:?}", types_value),
                    span: self.get_node_span(estree),
                })?;
                let mut types = Vec::new_in(self.builder.allocator);
                for type_value in types_array {
                    self.context = self.context.clone().with_parent("TSUnionType", "types");
                    let ts_type = self.convert_ts_type(type_value)?;
                    types.push(ts_type);
                }
                let union_type = self.builder.alloc_ts_union_type(span, types);
                Ok(TSType::TSUnionType(union_type))
            }
            "TSIntersectionType" => {
                let types_value = estree.get("types").ok_or_else(|| ConversionError::MissingField {
                    field: "types".to_string(),
                    node_type: "TSIntersectionType".to_string(),
                    span: self.get_node_span(estree),
                })?;
                let types_array = types_value.as_array().ok_or_else(|| ConversionError::InvalidFieldType {
                    field: "types".to_string(),
                    expected: "array".to_string(),
                    got: format!("{:?}", types_value),
                    span: self.get_node_span(estree),
                })?;
                let mut types = Vec::new_in(self.builder.allocator);
                for type_value in types_array {
                    self.context = self.context.clone().with_parent("TSIntersectionType", "types");
                    let ts_type = self.convert_ts_type(type_value)?;
                    types.push(ts_type);
                }
                let intersection_type = self.builder.alloc_ts_intersection_type(span, types);
                Ok(TSType::TSIntersectionType(intersection_type))
            }
            "TSTypeReference" => {
                // Get typeName (IdentifierReference or TSQualifiedName)
                self.context = self.context.clone().with_parent("TSTypeReference", "typeName");
                let type_name_value = estree.get("typeName").ok_or_else(|| ConversionError::MissingField {
                    field: "typeName".to_string(),
                    node_type: "TSTypeReference".to_string(),
                    span: self.get_node_span(estree),
                })?;
                let type_name = self.convert_ts_type_name(type_name_value)?;
                
                // Get typeArguments (optional)
                let type_arguments = if let Some(type_args_value) = estree.get("typeArguments") {
                    self.context = self.context.clone().with_parent("TSTypeReference", "typeArguments");
                    Some(self.convert_ts_type_parameter_instantiation(type_args_value)?)
                } else {
                    None
                };
                
                let type_ref = self.builder.alloc_ts_type_reference(span, type_name, type_arguments);
                Ok(TSType::TSTypeReference(type_ref))
            }
            "TSParenthesizedType" => {
                self.context = self.context.clone().with_parent("TSParenthesizedType", "typeAnnotation");
                let type_annotation_value = estree.get("typeAnnotation").ok_or_else(|| ConversionError::MissingField {
                    field: "typeAnnotation".to_string(),
                    node_type: "TSParenthesizedType".to_string(),
                    span: self.get_node_span(estree),
                })?;
                let type_annotation = self.convert_ts_type(type_annotation_value)?;
                let parenthesized_type = self.builder.alloc_ts_parenthesized_type(span, type_annotation);
                Ok(TSType::TSParenthesizedType(parenthesized_type))
            }
            "TSLiteralType" => {
                self.context = self.context.clone().with_parent("TSLiteralType", "literal");
                let literal_value = estree.get("literal").ok_or_else(|| ConversionError::MissingField {
                    field: "literal".to_string(),
                    node_type: "TSLiteralType".to_string(),
                    span: self.get_node_span(estree),
                })?;
                let literal = self.convert_ts_literal(literal_value)?;
                let literal_type = self.builder.alloc_ts_literal_type(span, literal);
                Ok(TSType::TSLiteralType(literal_type))
            }
            "TSTypeLiteral" => {
                // TSTypeLiteral has members array (TSSignature[])
                let members_value = estree.get("members").ok_or_else(|| ConversionError::MissingField {
                    field: "members".to_string(),
                    node_type: "TSTypeLiteral".to_string(),
                    span: self.get_node_span(estree),
                })?;
                let members_array = members_value.as_array().ok_or_else(|| ConversionError::InvalidFieldType {
                    field: "members".to_string(),
                    expected: "array".to_string(),
                    got: format!("{:?}", members_value),
                    span: self.get_node_span(estree),
                })?;
                // Convert each member (TSSignature)
                let mut members = Vec::new_in(self.builder.allocator);
                for member_value in members_array {
                    self.context = self.context.clone().with_parent("TSTypeLiteral", "members");
                    let signature = self.convert_ts_signature(member_value)?;
                    members.push(signature);
                }
                let type_literal = self.builder.alloc_ts_type_literal(span, members);
                Ok(TSType::TSTypeLiteral(type_literal))
            }
            "TSTupleType" => {
                // TSTupleType has elementTypes array (TSTupleElement[])
                let element_types_value = estree.get("elementTypes").ok_or_else(|| ConversionError::MissingField {
                    field: "elementTypes".to_string(),
                    node_type: "TSTupleType".to_string(),
                    span: self.get_node_span(estree),
                })?;
                let element_types_array = element_types_value.as_array().ok_or_else(|| ConversionError::InvalidFieldType {
                    field: "elementTypes".to_string(),
                    expected: "array".to_string(),
                    got: format!("{:?}", element_types_value),
                    span: self.get_node_span(estree),
                })?;
                // Convert each element type (TSTupleElement)
                let mut element_types = Vec::new_in(self.builder.allocator);
                for elem_value in element_types_array {
                    self.context = self.context.clone().with_parent("TSTupleType", "elementTypes");
                    let tuple_element = self.convert_ts_tuple_element(elem_value)?;
                    element_types.push(tuple_element);
                }
                let tuple_type = self.builder.alloc_ts_tuple_type(span, element_types);
                Ok(TSType::TSTupleType(tuple_type))
            }
            "TSConditionalType" => {
                // TSConditionalType: T extends U ? X : Y
                let (start, end) = self.get_node_span(estree);
                let span = Span::new(start, end);
                let error_span = (start, end);
                
                // Get checkType
                self.context = self.context.clone().with_parent("TSConditionalType", "checkType");
                let check_type_value = estree.get("checkType").ok_or_else(|| ConversionError::MissingField {
                    field: "checkType".to_string(),
                    node_type: "TSConditionalType".to_string(),
                    span: error_span,
                })?;
                let check_type = self.convert_ts_type(check_type_value)?;
                
                // Get extendsType
                self.context = self.context.clone().with_parent("TSConditionalType", "extendsType");
                let extends_type_value = estree.get("extendsType").ok_or_else(|| ConversionError::MissingField {
                    field: "extendsType".to_string(),
                    node_type: "TSConditionalType".to_string(),
                    span: error_span,
                })?;
                let extends_type = self.convert_ts_type(extends_type_value)?;
                
                // Get trueType
                self.context = self.context.clone().with_parent("TSConditionalType", "trueType");
                let true_type_value = estree.get("trueType").ok_or_else(|| ConversionError::MissingField {
                    field: "trueType".to_string(),
                    node_type: "TSConditionalType".to_string(),
                    span: error_span,
                })?;
                let true_type = self.convert_ts_type(true_type_value)?;
                
                // Get falseType
                self.context = self.context.clone().with_parent("TSConditionalType", "falseType");
                let false_type_value = estree.get("falseType").ok_or_else(|| ConversionError::MissingField {
                    field: "falseType".to_string(),
                    node_type: "TSConditionalType".to_string(),
                    span: error_span,
                })?;
                let false_type = self.convert_ts_type(false_type_value)?;
                
                let conditional_type = self.builder.alloc_ts_conditional_type(span, check_type, extends_type, true_type, false_type);
                Ok(TSType::TSConditionalType(conditional_type))
            }
            "TSFunctionType" => {
                // TSFunctionType: (x: number) => string
                let (start, end) = self.get_node_span(estree);
                let span = Span::new(start, end);
                let error_span = (start, end);
                
                // Get typeParameters (optional)
                let type_parameters: Option<oxc_allocator::Box<'a, oxc_ast::ast::TSTypeParameterDeclaration<'a>>> = if let Some(type_params_value) = estree.get("typeParameters") {
                    if type_params_value.is_null() {
                        None
                    } else {
                        self.context = self.context.clone().with_parent("TSFunctionType", "typeParameters");
                        Some(self.convert_ts_type_parameter_declaration(type_params_value)?)
                    }
                } else {
                    None
                };
                
                // Get thisParam (optional, skipped in ESTree)
                let this_param: Option<oxc_allocator::Box<'a, oxc_ast::ast::TSThisParameter<'a>>> = None;
                
                // Get params (FormalParameters)
                self.context = self.context.clone().with_parent("TSFunctionType", "params");
                let params_value = estree.get("params").ok_or_else(|| ConversionError::MissingField {
                    field: "params".to_string(),
                    node_type: "TSFunctionType".to_string(),
                    span: error_span,
                })?;
                let params_array = params_value.as_array().ok_or_else(|| ConversionError::InvalidFieldType {
                    field: "params".to_string(),
                    expected: "array".to_string(),
                    got: format!("{:?}", params_value),
                    span: error_span,
                })?;
                
                // Convert parameters
                let mut params_vec = Vec::new_in(self.builder.allocator);
                let mut rest_param: Option<oxc_allocator::Box<'a, oxc_ast::ast::BindingRestElement<'a>>> = None;
                
                for (idx, param_value) in params_array.iter().enumerate() {
                    let param_context = self.context.clone().with_parent("TSFunctionType", "params");
                    self.context = param_context;
                    self.context.is_binding_context = true;
                    
                    use oxc_estree::deserialize::{EstreeNode, EstreeNodeType};
                    
                    let node_type = <Value as EstreeNode>::get_type(param_value).ok_or_else(|| ConversionError::MissingField {
                        field: "type".to_string(),
                        node_type: "TSFunctionType.params".to_string(),
                        span: self.get_node_span(param_value),
                    })?;
                    
                    if node_type == EstreeNodeType::RestElement {
                        // Rest parameter must be last
                        if idx != params_array.len() - 1 {
                            return Err(ConversionError::InvalidFieldType {
                                field: "params".to_string(),
                                expected: "RestElement must be last parameter".to_string(),
                                got: format!("RestElement at index {}", idx),
                                span: self.get_node_span(param_value),
                            });
                        }
                        let rest = self.convert_rest_element_to_binding_rest(param_value)?;
                        rest_param = Some(rest);
                    } else {
                        let formal_param = self.convert_to_formal_parameter(param_value)?;
                        params_vec.push(formal_param);
                    }
                }
                
                let params_span = if let Some(first_param) = params_array.first() {
                    let (start, _) = self.get_node_span(first_param);
                    let (_, end) = params_array.last().map(|p| self.get_node_span(p)).unwrap_or((start, start));
                    Span::new(start, end)
                } else {
                    Span::new(start, start)
                };
                
                let formal_params = self.builder.alloc_formal_parameters(params_span, oxc_ast::ast::FormalParameterKind::Signature, params_vec, rest_param);
                
                // Get returnType (required)
                self.context = self.context.clone().with_parent("TSFunctionType", "returnType");
                let return_type_value = estree.get("returnType").ok_or_else(|| ConversionError::MissingField {
                    field: "returnType".to_string(),
                    node_type: "TSFunctionType".to_string(),
                    span: error_span,
                })?;
                let ts_type = self.convert_ts_type(return_type_value)?;
                let return_type_span = self.get_node_span(return_type_value);
                let return_type = oxc_allocator::Box::new_in(
                    oxc_ast::ast::TSTypeAnnotation {
                        span: Span::new(return_type_span.0, return_type_span.1),
                        type_annotation: ts_type,
                    },
                    self.builder.allocator,
                );
                
                let function_type = self.builder.alloc_ts_function_type(span, type_parameters, this_param, formal_params, return_type);
                Ok(TSType::TSFunctionType(function_type))
            }
            "TSIndexedAccessType" => {
                // TSIndexedAccessType: T[K]
                let (start, end) = self.get_node_span(estree);
                let span = Span::new(start, end);
                let error_span = (start, end);
                
                // Get objectType
                self.context = self.context.clone().with_parent("TSIndexedAccessType", "objectType");
                let object_type_value = estree.get("objectType").ok_or_else(|| ConversionError::MissingField {
                    field: "objectType".to_string(),
                    node_type: "TSIndexedAccessType".to_string(),
                    span: error_span,
                })?;
                let object_type = self.convert_ts_type(object_type_value)?;
                
                // Get indexType
                self.context = self.context.clone().with_parent("TSIndexedAccessType", "indexType");
                let index_type_value = estree.get("indexType").ok_or_else(|| ConversionError::MissingField {
                    field: "indexType".to_string(),
                    node_type: "TSIndexedAccessType".to_string(),
                    span: error_span,
                })?;
                let index_type = self.convert_ts_type(index_type_value)?;
                
                let indexed_access_type = self.builder.alloc_ts_indexed_access_type(span, object_type, index_type);
                Ok(TSType::TSIndexedAccessType(indexed_access_type))
            }
            "TSTypeQuery" => {
                // TSTypeQuery: typeof T
                let (start, end) = self.get_node_span(estree);
                let span = Span::new(start, end);
                let error_span = (start, end);
                
                // Get exprName (TSTypeQueryExprName - can be IdentifierReference, TSQualifiedName, ThisExpression, or TSImportType)
                self.context = self.context.clone().with_parent("TSTypeQuery", "exprName");
                let expr_name_value = estree.get("exprName").ok_or_else(|| ConversionError::MissingField {
                    field: "exprName".to_string(),
                    node_type: "TSTypeQuery".to_string(),
                    span: error_span,
                })?;
                let expr_name = self.convert_ts_type_query_expr_name(expr_name_value)?;
                
                // Get typeArguments (optional)
                let type_arguments = if let Some(type_args_value) = estree.get("typeArguments") {
                    self.context = self.context.clone().with_parent("TSTypeQuery", "typeArguments");
                    Some(self.convert_ts_type_parameter_instantiation(type_args_value)?)
                } else {
                    None
                };
                
                let type_query = self.builder.alloc_ts_type_query(span, expr_name, type_arguments);
                Ok(TSType::TSTypeQuery(type_query))
            }
            "TSInferType" => {
                // TSInferType: infer U
                let (start, end) = self.get_node_span(estree);
                let span = Span::new(start, end);
                let error_span = (start, end);
                
                // Get typeParameter (TSTypeParameter)
                self.context = self.context.clone().with_parent("TSInferType", "typeParameter");
                let type_param_value = estree.get("typeParameter").ok_or_else(|| ConversionError::MissingField {
                    field: "typeParameter".to_string(),
                    node_type: "TSInferType".to_string(),
                    span: error_span,
                })?;
                let type_parameter = self.convert_ts_type_parameter(type_param_value)?;
                let type_parameter_box = oxc_allocator::Box::new_in(type_parameter, self.builder.allocator);
                
                let infer_type = self.builder.alloc_ts_infer_type(span, type_parameter_box);
                Ok(TSType::TSInferType(infer_type))
            }
            "TSImportType" => {
                // TSImportType: import('foo')
                let (start, end) = self.get_node_span(estree);
                let span = Span::new(start, end);
                let error_span = (start, end);
                
                // Get argument (TSType - typically StringLiteral)
                self.context = self.context.clone().with_parent("TSImportType", "argument");
                let argument_value = estree.get("argument").ok_or_else(|| ConversionError::MissingField {
                    field: "argument".to_string(),
                    node_type: "TSImportType".to_string(),
                    span: error_span,
                })?;
                let argument = self.convert_ts_type(argument_value)?;
                
                // Get options (optional ObjectExpression)
                let options = if let Some(options_value) = estree.get("options") {
                    self.context = self.context.clone().with_parent("TSImportType", "options");
                    // Convert ObjectExpression directly
                    use oxc_ast::ast::{ObjectPropertyKind};
                    use oxc_estree::deserialize::{EstreeNode, EstreeNodeType};
                    
                    let node_type = <Value as EstreeNode>::get_type(options_value).ok_or_else(|| ConversionError::MissingField {
                        field: "type".to_string(),
                        node_type: "TSImportType.options".to_string(),
                        span: self.get_node_span(options_value),
                    })?;
                    
                    if node_type != EstreeNodeType::ObjectExpression {
                        return Err(ConversionError::InvalidFieldType {
                            field: "options".to_string(),
                            expected: "ObjectExpression".to_string(),
                            got: format!("{:?}", node_type),
                            span: self.get_node_span(options_value),
                        });
                    }
                    
                    let properties_value = options_value.get("properties").ok_or_else(|| ConversionError::MissingField {
                        field: "properties".to_string(),
                        node_type: "ObjectExpression".to_string(),
                        span: self.get_node_span(options_value),
                    })?;
                    let properties_array = properties_value.as_array().ok_or_else(|| ConversionError::InvalidFieldType {
                        field: "properties".to_string(),
                        expected: "array".to_string(),
                        got: format!("{:?}", properties_value),
                        span: self.get_node_span(options_value),
                    })?;
                    
                    let mut properties = Vec::new_in(self.builder.allocator);
                    for prop_value in properties_array {
                        self.context = self.context.clone().with_parent("ObjectExpression", "properties");
                        let prop = self.convert_object_property(prop_value)?;
                        properties.push(prop);
                    }
                    
                    let (start, end) = self.get_node_span(options_value);
                    let span = Span::new(start, end);
                    let obj_expr = self.builder.alloc_object_expression(span, properties);
                    Some(obj_expr)
                } else {
                    None
                };
                
                // Get qualifier (optional TSImportTypeQualifier)
                let qualifier = if let Some(qualifier_value) = estree.get("qualifier") {
                    self.context = self.context.clone().with_parent("TSImportType", "qualifier");
                    Some(self.convert_ts_import_type_qualifier(qualifier_value)?)
                } else {
                    None
                };
                
                // Get typeArguments (optional)
                let type_arguments = if let Some(type_args_value) = estree.get("typeArguments") {
                    self.context = self.context.clone().with_parent("TSImportType", "typeArguments");
                    Some(self.convert_ts_type_parameter_instantiation(type_args_value)?)
                } else {
                    None
                };
                
                let import_type = self.builder.alloc_ts_import_type(span, argument, options, qualifier, type_arguments);
                Ok(TSType::TSImportType(import_type))
            }
            "TSConstructorType" => {
                // TSConstructorType: new (x: number) => string
                let (start, end) = self.get_node_span(estree);
                let span = Span::new(start, end);
                let error_span = (start, end);
                
                // Get abstract flag
                let r#abstract = estree.get("abstract")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                
                // Get typeParameters (optional)
                let type_parameters: Option<oxc_allocator::Box<'a, oxc_ast::ast::TSTypeParameterDeclaration<'a>>> = if let Some(type_params_value) = estree.get("typeParameters") {
                    if type_params_value.is_null() {
                        None
                    } else {
                        self.context = self.context.clone().with_parent("TSConstructorType", "typeParameters");
                        Some(self.convert_ts_type_parameter_declaration(type_params_value)?)
                    }
                } else {
                    None
                };
                
                // Get params (FormalParameters) - similar to TSFunctionType
                self.context = self.context.clone().with_parent("TSConstructorType", "params");
                let params_value = estree.get("params").ok_or_else(|| ConversionError::MissingField {
                    field: "params".to_string(),
                    node_type: "TSConstructorType".to_string(),
                    span: error_span,
                })?;
                let params_array = params_value.as_array().ok_or_else(|| ConversionError::InvalidFieldType {
                    field: "params".to_string(),
                    expected: "array".to_string(),
                    got: format!("{:?}", params_value),
                    span: error_span,
                })?;
                
                // Convert parameters (same logic as TSFunctionType)
                let mut params_vec = Vec::new_in(self.builder.allocator);
                let mut rest_param: Option<oxc_allocator::Box<'a, oxc_ast::ast::BindingRestElement<'a>>> = None;
                
                for (idx, param_value) in params_array.iter().enumerate() {
                    let param_context = self.context.clone().with_parent("TSConstructorType", "params");
                    self.context = param_context;
                    self.context.is_binding_context = true;
                    
                    use oxc_estree::deserialize::{EstreeNode, EstreeNodeType};
                    
                    let node_type = <Value as EstreeNode>::get_type(param_value).ok_or_else(|| ConversionError::MissingField {
                        field: "type".to_string(),
                        node_type: "TSConstructorType.params".to_string(),
                        span: self.get_node_span(param_value),
                    })?;
                    
                    if node_type == EstreeNodeType::RestElement {
                        if idx != params_array.len() - 1 {
                            return Err(ConversionError::InvalidFieldType {
                                field: "params".to_string(),
                                expected: "RestElement must be last parameter".to_string(),
                                got: format!("RestElement at index {}", idx),
                                span: self.get_node_span(param_value),
                            });
                        }
                        let rest = self.convert_rest_element_to_binding_rest(param_value)?;
                        rest_param = Some(rest);
                    } else {
                        let formal_param = self.convert_to_formal_parameter(param_value)?;
                        params_vec.push(formal_param);
                    }
                }
                
                let params_span = if let Some(first_param) = params_array.first() {
                    let (start, _) = self.get_node_span(first_param);
                    let (_, end) = params_array.last().map(|p| self.get_node_span(p)).unwrap_or((start, start));
                    Span::new(start, end)
                } else {
                    Span::new(start, start)
                };
                
                let formal_params = self.builder.alloc_formal_parameters(params_span, oxc_ast::ast::FormalParameterKind::Signature, params_vec, rest_param);
                
                // Get returnType (required)
                self.context = self.context.clone().with_parent("TSConstructorType", "returnType");
                let return_type_value = estree.get("returnType").ok_or_else(|| ConversionError::MissingField {
                    field: "returnType".to_string(),
                    node_type: "TSConstructorType".to_string(),
                    span: error_span,
                })?;
                let ts_type = self.convert_ts_type(return_type_value)?;
                let return_type_span = self.get_node_span(return_type_value);
                let return_type = oxc_allocator::Box::new_in(
                    oxc_ast::ast::TSTypeAnnotation {
                        span: Span::new(return_type_span.0, return_type_span.1),
                        type_annotation: ts_type,
                    },
                    self.builder.allocator,
                );
                
                let constructor_type = self.builder.alloc_ts_constructor_type(span, r#abstract, type_parameters, formal_params, return_type);
                Ok(TSType::TSConstructorType(constructor_type))
            }
            "TSTypeOperatorType" => {
                // TSTypeOperatorType: keyof T, readonly T, unique T
                let (start, end) = self.get_node_span(estree);
                let span = Span::new(start, end);
                let error_span = (start, end);
                
                // Get operator
                let operator_str = estree.get("operator")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ConversionError::MissingField {
                        field: "operator".to_string(),
                        node_type: "TSTypeOperatorType".to_string(),
                        span: error_span,
                    })?;
                
                let operator = match operator_str {
                    "keyof" => oxc_ast::ast::TSTypeOperatorOperator::Keyof,
                    "readonly" => oxc_ast::ast::TSTypeOperatorOperator::Readonly,
                    "unique" => oxc_ast::ast::TSTypeOperatorOperator::Unique,
                    _ => {
                        return Err(ConversionError::InvalidFieldType {
                            field: "operator".to_string(),
                            expected: "keyof, readonly, or unique".to_string(),
                            got: operator_str.to_string(),
                            span: error_span,
                        });
                    }
                };
                
                // Get typeAnnotation
                self.context = self.context.clone().with_parent("TSTypeOperatorType", "typeAnnotation");
                let type_annotation_value = estree.get("typeAnnotation").ok_or_else(|| ConversionError::MissingField {
                    field: "typeAnnotation".to_string(),
                    node_type: "TSTypeOperatorType".to_string(),
                    span: error_span,
                })?;
                let type_annotation = self.convert_ts_type(type_annotation_value)?;
                
                let type_operator = self.builder.ts_type_type_operator_type(span, operator, type_annotation);
                Ok(type_operator)
            }
            "TSTypePredicate" => {
                // TSTypePredicate: x is string, asserts x is string
                let (start, end) = self.get_node_span(estree);
                let span = Span::new(start, end);
                let error_span = (start, end);
                
                // Get parameterName (TSTypePredicateName - can be Identifier or This)
                self.context = self.context.clone().with_parent("TSTypePredicate", "parameterName");
                let param_name_value = estree.get("parameterName").ok_or_else(|| ConversionError::MissingField {
                    field: "parameterName".to_string(),
                    node_type: "TSTypePredicate".to_string(),
                    span: error_span,
                })?;
                
                let param_name = if param_name_value.get("type").and_then(|v| v.as_str()) == Some("ThisExpression") {
                    // ThisExpression -> TSThisType
                    let (start, end) = self.get_node_span(param_name_value);
                    let this_span = Span::new(start, end);
                    // Use ts_this_type which returns TSThisType directly, not Box
                    let this_type = self.builder.ts_this_type(this_span);
                    oxc_ast::ast::TSTypePredicateName::This(this_type)
                } else {
                    // Identifier -> IdentifierName
                    let ident_name = self.convert_identifier_to_name(param_name_value)?;
                    oxc_ast::ast::TSTypePredicateName::Identifier(oxc_allocator::Box::new_in(ident_name, self.builder.allocator))
                };
                
                // Get asserts flag
                let asserts = estree.get("asserts")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                
                // Get typeAnnotation (optional)
                let type_annotation = if let Some(type_ann_value) = estree.get("typeAnnotation") {
                    self.context = self.context.clone().with_parent("TSTypePredicate", "typeAnnotation");
                    let ts_type = self.convert_ts_type(type_ann_value)?;
                    let type_ann_span = self.get_node_span(type_ann_value);
                    Some(oxc_allocator::Box::new_in(
                        oxc_ast::ast::TSTypeAnnotation {
                            span: Span::new(type_ann_span.0, type_ann_span.1),
                            type_annotation: ts_type,
                        },
                        self.builder.allocator,
                    ))
                } else {
                    None
                };
                
                let type_predicate = self.builder.alloc_ts_type_predicate(span, param_name, asserts, type_annotation);
                Ok(TSType::TSTypePredicate(type_predicate))
            }
            "TSMappedType" => {
                // TSMappedType: { [P in keyof T]: T[P] }
                let (start, end) = self.get_node_span(estree);
                let span = Span::new(start, end);
                let error_span = (start, end);
                
                // Get typeParameter (TSTypeParameter) - required
                self.context = self.context.clone().with_parent("TSMappedType", "typeParameter");
                let type_param_value = estree.get("typeParameter").ok_or_else(|| ConversionError::MissingField {
                    field: "typeParameter".to_string(),
                    node_type: "TSMappedType".to_string(),
                    span: error_span,
                })?;
                let type_parameter = self.convert_ts_type_parameter(type_param_value)?;
                let type_parameter_box = oxc_allocator::Box::new_in(type_parameter, self.builder.allocator);
                
                // Get nameType (optional TSType) - ESTree uses "nameType"
                let name_type = if let Some(name_type_value) = estree.get("nameType") {
                    self.context = self.context.clone().with_parent("TSMappedType", "nameType");
                    Some(self.convert_ts_type(name_type_value)?)
                } else {
                    None
                };
                
                // Get typeAnnotation (optional TSType)
                let type_annotation = if let Some(type_ann_value) = estree.get("typeAnnotation") {
                    self.context = self.context.clone().with_parent("TSMappedType", "typeAnnotation");
                    Some(self.convert_ts_type(type_ann_value)?)
                } else {
                    None
                };
                
                // Get optional modifier (optional TSMappedTypeModifierOperator)
                let optional = if let Some(optional_value) = estree.get("optional") {
                    // ESTree uses boolean or string ("+" or "-")
                    if let Some(b) = optional_value.as_bool() {
                        if b {
                            Some(oxc_ast::ast::TSMappedTypeModifierOperator::True)
                        } else {
                            None
                        }
                    } else if let Some(s) = optional_value.as_str() {
                        match s {
                            "+" => Some(oxc_ast::ast::TSMappedTypeModifierOperator::Plus),
                            "-" => Some(oxc_ast::ast::TSMappedTypeModifierOperator::Minus),
                            _ => None,
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };
                
                // Get readonly modifier (optional TSMappedTypeModifierOperator)
                let readonly = if let Some(readonly_value) = estree.get("readonly") {
                    // ESTree uses boolean or string ("+" or "-")
                    if let Some(b) = readonly_value.as_bool() {
                        if b {
                            Some(oxc_ast::ast::TSMappedTypeModifierOperator::True)
                        } else {
                            None
                        }
                    } else if let Some(s) = readonly_value.as_str() {
                        match s {
                            "+" => Some(oxc_ast::ast::TSMappedTypeModifierOperator::Plus),
                            "-" => Some(oxc_ast::ast::TSMappedTypeModifierOperator::Minus),
                            _ => None,
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };
                
                let mapped_type = self.builder.alloc_ts_mapped_type(span, type_parameter_box, name_type, type_annotation, optional, readonly);
                Ok(TSType::TSMappedType(mapped_type))
            }
            "TSTemplateLiteralType" => {
                // TSTemplateLiteralType: `${T}.${U}`
                let (start, end) = self.get_node_span(estree);
                let span = Span::new(start, end);
                let error_span = (start, end);
                
                // Get quasis (array of TemplateElement)
                self.context = self.context.clone().with_parent("TSTemplateLiteralType", "quasis");
                let quasis_value = estree.get("quasis").ok_or_else(|| ConversionError::MissingField {
                    field: "quasis".to_string(),
                    node_type: "TSTemplateLiteralType".to_string(),
                    span: error_span,
                })?;
                let quasis_array = quasis_value.as_array().ok_or_else(|| ConversionError::InvalidFieldType {
                    field: "quasis".to_string(),
                    expected: "array".to_string(),
                    got: format!("{:?}", quasis_value),
                    span: error_span,
                })?;
                
                let mut quasis: Vec<'a, oxc_ast::ast::TemplateElement<'a>> = Vec::new_in(self.builder.allocator);
                for (idx, quasi_value) in quasis_array.iter().enumerate() {
                    self.context = self.context.clone().with_parent("TSTemplateLiteralType", "quasis");
                    let is_tail = idx == quasis_array.len() - 1;
                    let template_element = self.convert_template_element(quasi_value, is_tail)?;
                    quasis.push(template_element);
                }
                
                // Get types (array of TSType)
                self.context = self.context.clone().with_parent("TSTemplateLiteralType", "types");
                let types_value = estree.get("types").ok_or_else(|| ConversionError::MissingField {
                    field: "types".to_string(),
                    node_type: "TSTemplateLiteralType".to_string(),
                    span: error_span,
                })?;
                let types_array = types_value.as_array().ok_or_else(|| ConversionError::InvalidFieldType {
                    field: "types".to_string(),
                    expected: "array".to_string(),
                    got: format!("{:?}", types_value),
                    span: error_span,
                })?;
                
                let mut types: Vec<'a, oxc_ast::ast::TSType<'a>> = Vec::new_in(self.builder.allocator);
                for type_value in types_array {
                    self.context = self.context.clone().with_parent("TSTemplateLiteralType", "types");
                    let ts_type = self.convert_ts_type(type_value)?;
                    types.push(ts_type);
                }
                
                let template_literal_type = self.builder.alloc_ts_template_literal_type(span, quasis, types);
                Ok(TSType::TSTemplateLiteralType(template_literal_type))
            }
            // Remaining compound types - return error for now
            _ => Err(ConversionError::UnsupportedNodeType {
                node_type: format!("TSType variant: {}", node_type_str),
                span: self.get_node_span(estree),
            }),
        }
    }

    /// Convert an ESTree TSTypeName to oxc TSTypeName.
    /// TSTypeName can be an IdentifierReference, TSQualifiedName, or ThisExpression.
    fn convert_ts_type_name(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::TSTypeName<'a>> {
        use oxc_ast::ast::TSTypeName;
        use oxc_estree::deserialize::{EstreeNode, EstreeNodeType};

        let node_type_str = estree.get("type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ConversionError::MissingField {
                field: "type".to_string(),
                node_type: "TSTypeName".to_string(),
                span: self.get_node_span(estree),
            })?;

        match node_type_str {
            "Identifier" => {
                // IdentifierReference
                let id_ref = self.convert_identifier_to_reference(estree)?;
                Ok(TSTypeName::IdentifierReference(oxc_allocator::Box::new_in(id_ref, self.builder.allocator)))
            }
            "TSQualifiedName" => {
                // TSQualifiedName for TSTypeName
                let qualified_name = self.convert_ts_qualified_name(estree)?;
                Ok(TSTypeName::QualifiedName(qualified_name))
            }
            "ThisExpression" => {
                // ThisExpression for TSTypeName
                let (start, end) = self.get_node_span(estree);
                let span = Span::new(start, end);
                let this_expr = self.builder.alloc_this_expression(span);
                Ok(TSTypeName::ThisExpression(this_expr))
            }
            _ => Err(ConversionError::UnsupportedNodeType {
                node_type: format!("TSTypeName variant: {}", node_type_str),
                span: self.get_node_span(estree),
            }),
        }
    }

    /// Convert an ESTree literal to oxc TSLiteral.
    /// TSLiteral can be BooleanLiteral, NumericLiteral, BigIntLiteral, StringLiteral, TemplateLiteral, or UnaryExpression.
    fn convert_ts_literal(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::TSLiteral<'a>> {
        use oxc_ast::ast::TSLiteral;
        use oxc_estree::deserialize::{EstreeLiteral, LiteralKind, convert_literal, get_boolean_value, get_numeric_value, get_string_value};
        use oxc_span::Atom;

        let estree_literal = EstreeLiteral::from_json(estree)
            .ok_or_else(|| ConversionError::InvalidFieldType {
                field: "literal".to_string(),
                expected: "valid literal node".to_string(),
                got: format!("{:?}", estree),
                span: self.get_node_span(estree),
            })?;

        let (start, end) = get_literal_span(&estree_literal);
        let span = convert_span(self.source_text, start as usize, end as usize);

        match convert_literal(&estree_literal)? {
            LiteralKind::Boolean => {
                let value = get_boolean_value(&estree_literal)?;
                let bool_lit = self.builder.alloc_boolean_literal(span, value);
                Ok(TSLiteral::BooleanLiteral(bool_lit))
            }
            LiteralKind::Numeric => {
                let value = get_numeric_value(&estree_literal)?;
                let raw = estree_literal.raw.as_ref().map(|s| {
                    Atom::from_in(s.as_str(), self.builder.allocator)
                });
                let num_lit = self.builder.alloc_numeric_literal(span, value, raw, oxc_syntax::number::NumberBase::Decimal);
                Ok(TSLiteral::NumericLiteral(num_lit))
            }
            LiteralKind::BigInt => {
                // BigIntLiteral: 123n
                // ESTree represents BigInt as a string value ending with 'n'
                let value_str = get_string_value(&estree_literal)?;
                // Remove the trailing 'n' to get the numeric part
                let numeric_str = value_str.strip_suffix('n')
                    .ok_or_else(|| ConversionError::InvalidFieldType {
                        field: "value".to_string(),
                        expected: "string ending with 'n'".to_string(),
                        got: value_str.to_string(),
                        span: self.get_node_span(estree),
                    })?;
                
                let value_atom = Atom::from_in(numeric_str, self.builder.allocator);
                let raw = estree_literal.raw.as_ref().map(|s| {
                    Atom::from_in(s.as_str(), self.builder.allocator)
                });
                
                // Determine base from raw value (default to Decimal)
                use oxc_syntax::number::BigintBase;
                let base = if let Some(raw_str) = estree_literal.raw.as_ref() {
                    if raw_str.starts_with("0x") || raw_str.starts_with("0X") {
                        BigintBase::Hex
                    } else if raw_str.starts_with("0o") || raw_str.starts_with("0O") {
                        BigintBase::Octal
                    } else if raw_str.starts_with("0b") || raw_str.starts_with("0B") {
                        BigintBase::Binary
                    } else {
                        BigintBase::Decimal
                    }
                } else {
                    BigintBase::Decimal
                };
                
                let big_int_lit = self.builder.ts_literal_big_int_literal(span, value_atom, raw, base);
                Ok(big_int_lit)
            }
            LiteralKind::String => {
                let value_str = get_string_value(&estree_literal)?;
                let atom = Atom::from_in(value_str, self.builder.allocator);
                let raw = estree_literal.raw.as_ref().map(|s| {
                    Atom::from_in(s.as_str(), self.builder.allocator)
                });
                let str_lit = self.builder.alloc_string_literal(span, atom, raw);
                Ok(TSLiteral::StringLiteral(str_lit))
            }
            _ => Err(ConversionError::UnsupportedNodeType {
                node_type: format!("TSLiteral variant: {:?}", convert_literal(&estree_literal)),
                span: self.get_node_span(estree),
            }),
        }
    }

    /// Convert an ESTree TSModuleDeclaration to oxc Statement.
    fn convert_ts_module_declaration(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::Statement<'a>> {
        use oxc_ast::ast::{Statement, TSModuleDeclarationBody, TSModuleDeclarationKind, TSModuleDeclarationName};

        // Get id (name)
        self.context = self.context.clone().with_parent("TSModuleDeclaration", "id");
        let id_value = estree.get("id").ok_or_else(|| ConversionError::MissingField {
            field: "id".to_string(),
            node_type: "TSModuleDeclaration".to_string(),
            span: self.get_node_span(estree),
        })?;
        
        // TSModuleDeclarationName can be Identifier or StringLiteral
        let id = if let Some(name_str) = id_value.get("name").and_then(|v| v.as_str()) {
            // Identifier
            let name = Atom::from_in(name_str, self.builder.allocator);
            let range = id_value.get("range").and_then(|v| v.as_array())
                .and_then(|arr| Some([arr[0].as_u64()? as usize, arr[1].as_u64()? as usize]));
            let span = convert_span(self.source_text, range.unwrap_or([0, 0])[0], range.unwrap_or([0, 0])[1]);
            let binding_id = self.builder.binding_identifier(span, name);
            TSModuleDeclarationName::Identifier(binding_id)
        } else if let Some(value_str) = id_value.get("value").and_then(|v| v.as_str()) {
            // StringLiteral
            let value = Atom::from_in(value_str, self.builder.allocator);
            let raw = id_value.get("raw").and_then(|v| v.as_str())
                .map(|s| Atom::from_in(s, self.builder.allocator));
            let range = id_value.get("range").and_then(|v| v.as_array())
                .and_then(|arr| {
                    if arr.len() >= 2 {
                        Some([arr[0].as_u64()? as usize, arr[1].as_u64()? as usize])
                    } else {
                        None
                    }
                });
            let span = convert_span(self.source_text, range.unwrap_or([0, 0])[0], range.unwrap_or([0, 0])[1]);
            let string_lit = self.builder.string_literal(span, value, raw);
            TSModuleDeclarationName::StringLiteral(string_lit)
        } else {
            return Err(ConversionError::InvalidFieldType {
                field: "id".to_string(),
                expected: "Identifier or StringLiteral".to_string(),
                got: format!("{:?}", id_value),
                span: self.get_node_span(estree),
            });
        };

        // Get body (optional TSModuleDeclarationBody)
        let body: Option<TSModuleDeclarationBody> = if let Some(body_value) = estree.get("body") {
            if body_value.is_null() {
                None
            } else {
                self.context = self.context.clone().with_parent("TSModuleDeclaration", "body");
                // ESTree body can be TSModuleBlock (array of statements) or TSModuleDeclaration (nested)
                let body_type = body_value.get("type").and_then(|v| v.as_str());
                if body_type == Some("TSModuleBlock") {
                    // Get body array (statements)
                    let mut statements = Vec::new_in(self.builder.allocator);
                    if let Some(body_array) = body_value.get("body").and_then(|v| v.as_array()) {
                        for stmt_value in body_array {
                            if stmt_value.is_null() {
                                continue;
                            }
                            self.context = self.context.clone().with_parent("TSModuleBlock", "body");
                            let stmt = self.convert_statement(stmt_value)?;
                            statements.push(stmt);
                        }
                    }
                    
                    // Get directives (optional)
                    // In ESTree, TSModuleBlock may have a 'directives' field containing an array of directive nodes
                    // Each directive is typically an ExpressionStatement with a StringLiteral expression
                    let mut directives = Vec::new_in(self.builder.allocator);
                    if let Some(directives_value) = body_value.get("directives") {
                        if !directives_value.is_null() {
                            if let Some(directives_array) = directives_value.as_array() {
                                for directive_value in directives_array {
                                    if directive_value.is_null() {
                                        continue;
                                    }
                                    self.context = self.context.clone().with_parent("TSModuleBlock", "directives");
                                    if let Ok(directive) = self.convert_directive(directive_value) {
                                        directives.push(directive);
                                    }
                                    // If conversion fails, skip this directive (non-fatal)
                                }
                            }
                        }
                    }
                    
                    let (body_start, body_end) = self.get_node_span(body_value);
                    let body_span = Span::new(body_start, body_end);
                    let module_block = self.builder.ts_module_block(body_span, directives, statements);
                    Some(TSModuleDeclarationBody::TSModuleBlock(oxc_allocator::Box::new_in(module_block, self.builder.allocator)))
                } else {
                    // Nested TSModuleDeclaration
                    let nested_decl = self.convert_ts_module_declaration(body_value)?;
                    match nested_decl {
                        oxc_ast::ast::Statement::TSModuleDeclaration(boxed) => {
                            Some(TSModuleDeclarationBody::TSModuleDeclaration(boxed))
                        }
                        _ => return Err(ConversionError::InvalidFieldType {
                            field: "body".to_string(),
                            expected: "TSModuleDeclaration".to_string(),
                            got: format!("{:?}", body_type),
                            span: self.get_node_span(body_value),
                        }),
                    }
                }
            }
        } else {
            None
        };

        // Get kind (optional, default to Namespace)
        let kind = estree.get("kind").and_then(|v| v.as_str())
            .map(|s| match s {
                "global" => TSModuleDeclarationKind::Global,
                "module" => TSModuleDeclarationKind::Module,
                "namespace" => TSModuleDeclarationKind::Namespace,
                _ => TSModuleDeclarationKind::Namespace,
            })
            .unwrap_or(TSModuleDeclarationKind::Namespace);

        // Get declare (optional)
        let declare = estree.get("declare").and_then(|v| v.as_bool()).unwrap_or(false);

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        let module_decl_box = self.builder.alloc_ts_module_declaration(span, id, body, kind, declare);
        let module_decl = oxc_ast::ast::Declaration::TSModuleDeclaration(module_decl_box);
        match module_decl {
            oxc_ast::ast::Declaration::TSModuleDeclaration(boxed) => {
                Ok(Statement::TSModuleDeclaration(boxed))
            }
            _ => unreachable!(),
        }
    }

    /// Convert an ESTree TSImportEqualsDeclaration to oxc Statement.
    fn convert_ts_import_equals_declaration(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::Statement<'a>> {
        use oxc_ast::ast::{Statement, TSModuleReference, TSExternalModuleReference};
        use oxc_span::Atom;

        // Get id
        self.context = self.context.clone().with_parent("TSImportEqualsDeclaration", "id");
        let id_value = estree.get("id").ok_or_else(|| ConversionError::MissingField {
            field: "id".to_string(),
            node_type: "TSImportEqualsDeclaration".to_string(),
            span: self.get_node_span(estree),
        })?;
        let estree_id = oxc_estree::deserialize::EstreeIdentifier::from_json(id_value)
            .ok_or_else(|| ConversionError::InvalidFieldType {
                field: "id".to_string(),
                expected: "valid Identifier node".to_string(),
                got: format!("{:?}", id_value),
                span: self.get_node_span(estree),
            })?;
        let name = Atom::from_in(estree_id.name.as_str(), self.builder.allocator);
        let range = estree_id.range.unwrap_or([0, 0]);
        let span = convert_span(self.source_text, range[0] as usize, range[1] as usize);
        let id = self.builder.binding_identifier(span, name);

        // Get moduleReference (TSModuleReference - complex, simplified for now)
        self.context = self.context.clone().with_parent("TSImportEqualsDeclaration", "moduleReference");
        let module_ref_value = estree.get("moduleReference").ok_or_else(|| ConversionError::MissingField {
            field: "moduleReference".to_string(),
            node_type: "TSImportEqualsDeclaration".to_string(),
            span: self.get_node_span(estree),
        })?;
        
        // TSModuleReference can be ExternalModuleReference (string literal) or TSTypeName variants
        let module_reference = if let Some(expr_value) = module_ref_value.get("expression") {
            // ExternalModuleReference
            let value_str = expr_value.get("value").and_then(|v| v.as_str())
                .ok_or_else(|| ConversionError::InvalidFieldType {
                    field: "moduleReference.expression.value".to_string(),
                    expected: "string".to_string(),
                    got: format!("{:?}", expr_value),
                    span: self.get_node_span(estree),
                })?;
            let value = Atom::from_in(value_str, self.builder.allocator);
            let raw = expr_value.get("raw").and_then(|v| v.as_str())
                .map(|s| Atom::from_in(s, self.builder.allocator));
            let range = expr_value.get("range").and_then(|v| v.as_array())
                .and_then(|arr| {
                    if arr.len() >= 2 {
                        Some([arr[0].as_u64()? as usize, arr[1].as_u64()? as usize])
                    } else {
                        None
                    }
                });
            let expr_span = convert_span(self.source_text, range.unwrap_or([0, 0])[0], range.unwrap_or([0, 0])[1]);
            let string_lit = self.builder.string_literal(expr_span, value, raw);
            let ext_module_ref = self.builder.ts_external_module_reference(expr_span, string_lit);
            TSModuleReference::ExternalModuleReference(oxc_allocator::Box::new_in(ext_module_ref, self.builder.allocator))
        } else {
            // TSTypeName variants (IdentifierReference, QualifiedName, ThisExpression)
            let type_name = self.convert_ts_type_name(module_ref_value)?;
            match type_name {
                oxc_ast::ast::TSTypeName::IdentifierReference(ident) => {
                    TSModuleReference::IdentifierReference(ident)
                }
                oxc_ast::ast::TSTypeName::QualifiedName(qualified) => {
                    TSModuleReference::QualifiedName(qualified)
                }
                oxc_ast::ast::TSTypeName::ThisExpression(this_expr) => {
                    TSModuleReference::ThisExpression(this_expr)
                }
            }
        };

        // Get importKind (optional, default to Value)
        let import_kind = estree.get("importKind").and_then(|v| v.as_str())
            .map(|s| match s {
                "type" => oxc_ast::ast::ImportOrExportKind::Type,
                "value" | _ => oxc_ast::ast::ImportOrExportKind::Value,
            })
            .unwrap_or(oxc_ast::ast::ImportOrExportKind::Value);

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        let import_equals_decl = self.builder.declaration_ts_import_equals(span, id, module_reference, import_kind);
        match import_equals_decl {
            oxc_ast::ast::Declaration::TSImportEqualsDeclaration(boxed) => {
                Ok(Statement::TSImportEqualsDeclaration(boxed))
            }
            _ => unreachable!(),
        }
    }

    /// Convert an ESTree TSExportAssignment to oxc Statement.
    fn convert_ts_export_assignment(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::Statement<'a>> {
        use oxc_ast::ast::Statement;

        // Get expression
        self.context = self.context.clone().with_parent("TSExportAssignment", "expression");
        let expr_value = estree.get("expression").ok_or_else(|| ConversionError::MissingField {
            field: "expression".to_string(),
            node_type: "TSExportAssignment".to_string(),
            span: self.get_node_span(estree),
        })?;
        let expression = self.convert_expression(expr_value)?;

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        let export_assignment = self.builder.module_declaration_ts_export_assignment(span, expression);
        match export_assignment {
            oxc_ast::ast::ModuleDeclaration::TSExportAssignment(boxed) => {
                Ok(Statement::TSExportAssignment(boxed))
            }
            _ => unreachable!(),
        }
    }

    /// Convert an ESTree TSNamespaceExportDeclaration to oxc Statement.
    fn convert_ts_namespace_export_declaration(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::Statement<'a>> {
        use oxc_ast::ast::{Statement, IdentifierName};
        use oxc_span::Atom;

        // Get id
        self.context = self.context.clone().with_parent("TSNamespaceExportDeclaration", "id");
        let id_value = estree.get("id").ok_or_else(|| ConversionError::MissingField {
            field: "id".to_string(),
            node_type: "TSNamespaceExportDeclaration".to_string(),
            span: self.get_node_span(estree),
        })?;
        let estree_id = oxc_estree::deserialize::EstreeIdentifier::from_json(id_value)
            .ok_or_else(|| ConversionError::InvalidFieldType {
                field: "id".to_string(),
                expected: "valid Identifier node".to_string(),
                got: format!("{:?}", id_value),
                span: self.get_node_span(estree),
            })?;
        let name = Atom::from_in(estree_id.name.as_str(), self.builder.allocator);
        let range = estree_id.range.unwrap_or([0, 0]);
        let span = convert_span(self.source_text, range[0] as usize, range[1] as usize);
        let id = self.builder.identifier_name(span, name);

        let (start, end) = self.get_node_span(estree);
        let full_span = Span::new(start, end);

        let namespace_export_decl = self.builder.module_declaration_ts_namespace_export_declaration(full_span, id);
        match namespace_export_decl {
            oxc_ast::ast::ModuleDeclaration::TSNamespaceExportDeclaration(boxed) => {
                Ok(Statement::TSNamespaceExportDeclaration(boxed))
            }
            _ => unreachable!(),
        }
    }

    /// Convert an ESTree DebuggerStatement to oxc Statement.
    fn convert_debugger_statement(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::Statement<'a>> {
        use oxc_ast::ast::Statement;

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        let debugger_stmt = self.builder.alloc_debugger_statement(span);
        Ok(Statement::DebuggerStatement(debugger_stmt))
    }

    /// Convert an ESTree WithStatement to oxc Statement.
    fn convert_with_statement(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::Statement<'a>> {
        use oxc_ast::ast::Statement;

        // Get object
        self.context = self.context.clone().with_parent("WithStatement", "object");
        let object_value = estree.get("object").ok_or_else(|| ConversionError::MissingField {
            field: "object".to_string(),
            node_type: "WithStatement".to_string(),
            span: self.get_node_span(estree),
        })?;
        let object = self.convert_expression(object_value)?;

        // Get body
        self.context = self.context.clone().with_parent("WithStatement", "body");
        let body_value = estree.get("body").ok_or_else(|| ConversionError::MissingField {
            field: "body".to_string(),
            node_type: "WithStatement".to_string(),
            span: self.get_node_span(estree),
        })?;
        let body = self.convert_statement(body_value)?;

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        let with_stmt = self.builder.alloc_with_statement(span, object, body);
        Ok(Statement::WithStatement(with_stmt))
    }

    /// Convert an ESTree ImportExpression to oxc Expression.
    fn convert_import_expression(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::Expression<'a>> {
        use oxc_ast::ast::{Expression, ImportPhase};

        // Get source
        self.context = self.context.clone().with_parent("ImportExpression", "source");
        let source_value = estree.get("source").ok_or_else(|| ConversionError::MissingField {
            field: "source".to_string(),
            node_type: "ImportExpression".to_string(),
            span: self.get_node_span(estree),
        })?;
        let source = self.convert_expression(source_value)?;

        // Get options (optional)
        let options = estree.get("options").and_then(|opt_value| {
            self.context = self.context.clone().with_parent("ImportExpression", "options");
            Some(self.convert_expression(opt_value).ok()?)
        });

        // Get phase (optional, default to Defer)
        let phase = estree.get("phase").and_then(|v| v.as_str())
            .map(|s| match s {
                "source" => ImportPhase::Source,
                "defer" => ImportPhase::Defer,
                _ => ImportPhase::Defer,
            })
            .unwrap_or(ImportPhase::Defer);

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        let import_expr = self.builder.alloc_import_expression(span, source, options, Some(phase));
        Ok(Expression::ImportExpression(import_expr))
    }

    /// Convert an ESTree MetaProperty to oxc Expression.
    fn convert_meta_property(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::Expression<'a>> {
        use oxc_ast::ast::{Expression, IdentifierName, IdentifierReference};
        use oxc_span::Atom;

        // Get meta
        self.context = self.context.clone().with_parent("MetaProperty", "meta");
        let meta_value = estree.get("meta").ok_or_else(|| ConversionError::MissingField {
            field: "meta".to_string(),
            node_type: "MetaProperty".to_string(),
            span: self.get_node_span(estree),
        })?;
        let meta_name = meta_value.get("name").and_then(|v| v.as_str())
            .ok_or_else(|| ConversionError::InvalidFieldType {
                field: "meta.name".to_string(),
                expected: "string".to_string(),
                got: format!("{:?}", meta_value),
                span: self.get_node_span(estree),
            })?;
        let meta_range = meta_value.get("range").and_then(|v| v.as_array())
            .and_then(|arr| {
                if arr.len() >= 2 {
                    Some([arr[0].as_u64()? as usize, arr[1].as_u64()? as usize])
                } else {
                    None
                }
            });
        let meta_span = convert_span(self.source_text, meta_range.unwrap_or([0, 0])[0], meta_range.unwrap_or([0, 0])[1]);
        let meta_atom = Atom::from_in(meta_name, self.builder.allocator);
        let meta = self.builder.identifier_name(meta_span, meta_atom);

        // Get property
        self.context = self.context.clone().with_parent("MetaProperty", "property");
        let property_value = estree.get("property").ok_or_else(|| ConversionError::MissingField {
            field: "property".to_string(),
            node_type: "MetaProperty".to_string(),
            span: self.get_node_span(estree),
        })?;
        let property_name = property_value.get("name").and_then(|v| v.as_str())
            .ok_or_else(|| ConversionError::InvalidFieldType {
                field: "property.name".to_string(),
                expected: "string".to_string(),
                got: format!("{:?}", property_value),
                span: self.get_node_span(estree),
            })?;
        let property_range = property_value.get("range").and_then(|v| v.as_array())
            .and_then(|arr| {
                if arr.len() >= 2 {
                    Some([arr[0].as_u64()? as usize, arr[1].as_u64()? as usize])
                } else {
                    None
                }
            });
        let property_span = convert_span(self.source_text, property_range.unwrap_or([0, 0])[0], property_range.unwrap_or([0, 0])[1]);
        let property_atom = Atom::from_in(property_name, self.builder.allocator);
        let property = self.builder.identifier_name(property_span, property_atom);

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        let meta_prop = self.builder.alloc_meta_property(span, meta, property);
        Ok(Expression::MetaProperty(meta_prop))
    }

    /// Convert an ESTree TSAsExpression to oxc Expression.
    fn convert_ts_as_expression(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::Expression<'a>> {
        use oxc_ast::ast::Expression;

        // Get expression
        self.context = self.context.clone().with_parent("TSAsExpression", "expression");
        let expr_value = estree.get("expression").ok_or_else(|| ConversionError::MissingField {
            field: "expression".to_string(),
            node_type: "TSAsExpression".to_string(),
            span: self.get_node_span(estree),
        })?;
        let expression = self.convert_expression(expr_value)?;

        // Get typeAnnotation (TSType)
        self.context = self.context.clone().with_parent("TSAsExpression", "typeAnnotation");
        let type_annotation_value = estree.get("typeAnnotation").ok_or_else(|| ConversionError::MissingField {
            field: "typeAnnotation".to_string(),
            node_type: "TSAsExpression".to_string(),
            span: self.get_node_span(estree),
        })?;
        let type_annotation = self.convert_ts_type(type_annotation_value)?;

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        let as_expr = self.builder.alloc_ts_as_expression(span, expression, type_annotation);
        Ok(Expression::TSAsExpression(as_expr))
    }

    /// Convert an ESTree TSSatisfiesExpression to oxc Expression.
    fn convert_ts_satisfies_expression(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::Expression<'a>> {
        use oxc_ast::ast::Expression;

        // Get expression
        self.context = self.context.clone().with_parent("TSSatisfiesExpression", "expression");
        let expr_value = estree.get("expression").ok_or_else(|| ConversionError::MissingField {
            field: "expression".to_string(),
            node_type: "TSSatisfiesExpression".to_string(),
            span: self.get_node_span(estree),
        })?;
        let expression = self.convert_expression(expr_value)?;

        // Get typeAnnotation (TSType)
        self.context = self.context.clone().with_parent("TSSatisfiesExpression", "typeAnnotation");
        let type_annotation_value = estree.get("typeAnnotation").ok_or_else(|| ConversionError::MissingField {
            field: "typeAnnotation".to_string(),
            node_type: "TSSatisfiesExpression".to_string(),
            span: self.get_node_span(estree),
        })?;
        let type_annotation = self.convert_ts_type(type_annotation_value)?;

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        let satisfies_expr = self.builder.alloc_ts_satisfies_expression(span, expression, type_annotation);
        Ok(Expression::TSSatisfiesExpression(satisfies_expr))
    }

    /// Convert an ESTree TSNonNullExpression to oxc Expression.
    fn convert_ts_non_null_expression(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::Expression<'a>> {
        use oxc_ast::ast::Expression;

        // Get expression
        self.context = self.context.clone().with_parent("TSNonNullExpression", "expression");
        let expr_value = estree.get("expression").ok_or_else(|| ConversionError::MissingField {
            field: "expression".to_string(),
            node_type: "TSNonNullExpression".to_string(),
            span: self.get_node_span(estree),
        })?;
        let expression = self.convert_expression(expr_value)?;

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        let non_null_expr = self.builder.alloc_ts_non_null_expression(span, expression);
        Ok(Expression::TSNonNullExpression(non_null_expr))
    }

    /// Convert an ESTree TSInstantiationExpression to oxc Expression.
    fn convert_ts_instantiation_expression(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::Expression<'a>> {
        use oxc_ast::ast::Expression;

        // Get expression
        self.context = self.context.clone().with_parent("TSInstantiationExpression", "expression");
        let expr_value = estree.get("expression").ok_or_else(|| ConversionError::MissingField {
            field: "expression".to_string(),
            node_type: "TSInstantiationExpression".to_string(),
            span: self.get_node_span(estree),
        })?;
        let expression = self.convert_expression(expr_value)?;

        // Get typeArguments (TSTypeParameterInstantiation - required)
        self.context = self.context.clone().with_parent("TSInstantiationExpression", "typeArguments");
        let type_args_value = estree.get("typeArguments").ok_or_else(|| ConversionError::MissingField {
            field: "typeArguments".to_string(),
            node_type: "TSInstantiationExpression".to_string(),
            span: self.get_node_span(estree),
        })?;
        let type_arguments = self.convert_ts_type_parameter_instantiation(type_args_value)?;

        // Build TSInstantiationExpression
        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);
        let ts_instantiation = self.builder.alloc_ts_instantiation_expression(span, expression, type_arguments);
        Ok(Expression::TSInstantiationExpression(ts_instantiation))
    }

    /// Convert an ESTree TSTypeParameterInstantiation to oxc TSTypeParameterInstantiation.
    fn convert_ts_type_parameter_instantiation(&mut self, estree: &Value) -> ConversionResult<oxc_allocator::Box<'a, oxc_ast::ast::TSTypeParameterInstantiation<'a>>> {
        use oxc_ast::ast::TSTypeParameterInstantiation;

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);
        let error_span = (start, end);

        // Get params array
        let params_value = estree.get("params").ok_or_else(|| ConversionError::MissingField {
            field: "params".to_string(),
            node_type: "TSTypeParameterInstantiation".to_string(),
            span: error_span,
        })?;
        let params_array = params_value.as_array().ok_or_else(|| ConversionError::InvalidFieldType {
            field: "params".to_string(),
            expected: "array".to_string(),
            got: format!("{:?}", params_value),
            span: error_span,
        })?;

        // Convert each param (TSType)
        let mut params = Vec::new_in(self.builder.allocator);
        for param_value in params_array {
            self.context = self.context.clone().with_parent("TSTypeParameterInstantiation", "params");
            let ts_type = self.convert_ts_type(param_value)?;
            params.push(ts_type);
        }

        let type_param_instantiation = self.builder.alloc_ts_type_parameter_instantiation(span, params);
        Ok(type_param_instantiation)
    }

    /// Convert an ESTree TSQualifiedName to oxc TSQualifiedName.
    fn convert_ts_qualified_name(&mut self, estree: &Value) -> ConversionResult<oxc_allocator::Box<'a, oxc_ast::ast::TSQualifiedName<'a>>> {
        use oxc_ast::ast::TSQualifiedName;
        use oxc_span::Atom;

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        // Get left (TSTypeName)
        self.context = self.context.clone().with_parent("TSQualifiedName", "left");
        let left_value = estree.get("left").ok_or_else(|| ConversionError::MissingField {
            field: "left".to_string(),
            node_type: "TSQualifiedName".to_string(),
            span: (start, end),
        })?;
        let left = self.convert_ts_type_name(left_value)?;

        // Get right (IdentifierName - can be Identifier or StringLiteral in ESTree)
        self.context = self.context.clone().with_parent("TSQualifiedName", "right");
        let right_value = estree.get("right").ok_or_else(|| ConversionError::MissingField {
            field: "right".to_string(),
            node_type: "TSQualifiedName".to_string(),
            span: (start, end),
        })?;
        // IdentifierName can be Identifier or StringLiteral
        let right = if let Some(ident) = right_value.get("type").and_then(|t| t.as_str()) {
            let (start, end) = self.get_node_span(right_value);
            let span = Span::new(start, end);
            let name_str = if ident == "Identifier" {
                // Convert Identifier to IdentifierName
                right_value.get("name")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ConversionError::MissingField {
                        field: "name".to_string(),
                        node_type: "Identifier".to_string(),
                        span: self.get_node_span(right_value),
                    })?
            } else if ident == "StringLiteral" || ident == "Literal" {
                // Convert StringLiteral to IdentifierName
                right_value.get("value")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ConversionError::MissingField {
                        field: "value".to_string(),
                        node_type: "StringLiteral".to_string(),
                        span: self.get_node_span(right_value),
                    })?
            } else {
                return Err(ConversionError::InvalidFieldType {
                    field: "right".to_string(),
                    expected: "Identifier or StringLiteral".to_string(),
                    got: ident.to_string(),
                    span: self.get_node_span(right_value),
                });
            };
            let atom = Atom::from_in(name_str, self.builder.allocator);
            oxc_ast::ast::IdentifierName { span, name: atom }
        } else {
            return Err(ConversionError::MissingField {
                field: "type".to_string(),
                node_type: "TSQualifiedName.right".to_string(),
                span: self.get_node_span(right_value),
            });
        };

        let qualified_name = self.builder.alloc_ts_qualified_name(span, left, right);
        Ok(qualified_name)
    }

    /// Convert an ESTree TSTypeQueryExprName to oxc TSTypeQueryExprName.
    /// TSTypeQueryExprName can be an IdentifierReference, TSQualifiedName, ThisExpression, or TSImportType.
    fn convert_ts_type_query_expr_name(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::TSTypeQueryExprName<'a>> {
        use oxc_ast::ast::TSTypeQueryExprName;

        let node_type_str = estree.get("type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ConversionError::MissingField {
                field: "type".to_string(),
                node_type: "TSTypeQueryExprName".to_string(),
                span: self.get_node_span(estree),
            })?;

        match node_type_str {
            "Identifier" => {
                // IdentifierReference
                let id_ref = self.convert_identifier_to_reference(estree)?;
                Ok(TSTypeQueryExprName::IdentifierReference(oxc_allocator::Box::new_in(id_ref, self.builder.allocator)))
            }
            "TSQualifiedName" => {
                // TSQualifiedName
                let qualified_name = self.convert_ts_qualified_name(estree)?;
                Ok(TSTypeQueryExprName::QualifiedName(qualified_name))
            }
            "ThisExpression" => {
                // ThisExpression
                let (start, end) = self.get_node_span(estree);
                let span = Span::new(start, end);
                let this_expr = self.builder.alloc_this_expression(span);
                Ok(TSTypeQueryExprName::ThisExpression(this_expr))
            }
            "TSImportType" => {
                // TSImportType in TSTypeQueryExprName context
                // Convert the TSImportType and extract it from TSType
                let ts_type = self.convert_ts_type(estree)?;
                match ts_type {
                    oxc_ast::ast::TSType::TSImportType(import_type_box) => {
                        Ok(TSTypeQueryExprName::TSImportType(import_type_box))
                    }
                    _ => {
                        Err(ConversionError::InvalidFieldType {
                            field: "TSImportType".to_string(),
                            expected: "TSType::TSImportType".to_string(),
                            got: format!("{:?}", ts_type),
                            span: self.get_node_span(estree),
                        })
                    }
                }
            }
            _ => Err(ConversionError::UnsupportedNodeType {
                node_type: format!("TSTypeQueryExprName variant: {}", node_type_str),
                span: self.get_node_span(estree),
            }),
        }
    }

    /// Convert an ESTree TSTypeParameter to oxc TSTypeParameter.
    fn convert_ts_type_parameter(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::TSTypeParameter<'a>> {
        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);
        let error_span = (start, end);
        
        // Get name (BindingIdentifier)
        self.context = self.context.clone().with_parent("TSTypeParameter", "name");
        self.context.is_binding_context = true;
        let name_value = estree.get("name").ok_or_else(|| ConversionError::MissingField {
            field: "name".to_string(),
            node_type: "TSTypeParameter".to_string(),
            span: error_span,
        })?;
        let estree_id = EstreeIdentifier::from_json(name_value)
            .ok_or_else(|| ConversionError::InvalidFieldType {
                field: "name".to_string(),
                expected: "Identifier".to_string(),
                got: format!("{:?}", name_value),
                span: self.get_node_span(name_value),
            })?;
        let id_kind = convert_identifier(&estree_id, &self.context, self.source_text)?;
        // Extract BindingIdentifier from IdentifierKind
        use oxc_ast::ast::BindingIdentifier;
        use oxc_span::Atom;
        // Verify it's a binding
        if id_kind != IdentifierKind::Binding {
            return Err(ConversionError::InvalidFieldType {
                field: "name".to_string(),
                expected: "BindingIdentifier".to_string(),
                got: format!("{:?}", id_kind),
                span: self.get_node_span(name_value),
            });
        }
        
        let name = Atom::from_in(estree_id.name.as_str(), self.builder.allocator);
        let range = estree_id.range.unwrap_or([0, 0]);
        let name_span = convert_span(self.source_text, range[0] as usize, range[1] as usize);
        let binding_id = self.builder.binding_identifier(name_span, name);
        
        // Get constraint (optional TSType)
        let constraint = if let Some(constraint_value) = estree.get("constraint") {
            self.context = self.context.clone().with_parent("TSTypeParameter", "constraint");
            Some(self.convert_ts_type(constraint_value)?)
        } else {
            None
        };
        
        // Get default (optional TSType)
        let default = if let Some(default_value) = estree.get("default") {
            self.context = self.context.clone().with_parent("TSTypeParameter", "default");
            Some(self.convert_ts_type(default_value)?)
        } else {
            None
        };
        
        // Get in flag
        let r#in = estree.get("in")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        
        // Get out flag
        let out = estree.get("out")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        
        // Get const flag
        let r#const = estree.get("const")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        
        let type_parameter = self.builder.ts_type_parameter(span, binding_id, constraint, default, r#in, out, r#const);
        Ok(type_parameter)
    }

    /// Convert an ESTree TSTypeAnnotation to oxc TSTypeAnnotation.
    fn convert_ts_type_annotation(&mut self, estree: &Value) -> ConversionResult<oxc_allocator::Box<'a, oxc_ast::ast::TSTypeAnnotation<'a>>> {
        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);
        let error_span = (start, end);
        
        // Get typeAnnotation (required TSType)
        self.context = self.context.clone().with_parent("TSTypeAnnotation", "typeAnnotation");
        let type_annotation_value = estree.get("typeAnnotation").ok_or_else(|| ConversionError::MissingField {
            field: "typeAnnotation".to_string(),
            node_type: "TSTypeAnnotation".to_string(),
            span: error_span,
        })?;
        let type_annotation = self.convert_ts_type(type_annotation_value)?;
        
        let ts_type_annotation = self.builder.alloc_ts_type_annotation(span, type_annotation);
        Ok(ts_type_annotation)
    }

    /// Convert an ESTree TSInterfaceHeritage to oxc TSInterfaceHeritage.
    fn convert_ts_interface_heritage(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::TSInterfaceHeritage<'a>> {
        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);
        let error_span = (start, end);
        
        // Get expression (required Expression - typically TSTypeReference)
        self.context = self.context.clone().with_parent("TSInterfaceHeritage", "expression");
        let expression_value = estree.get("expression").ok_or_else(|| ConversionError::MissingField {
            field: "expression".to_string(),
            node_type: "TSInterfaceHeritage".to_string(),
            span: error_span,
        })?;
        let expression = self.convert_expression(expression_value)?;
        
        // Get typeArguments (optional TSTypeParameterInstantiation)
        let type_arguments: Option<oxc_allocator::Box<'a, oxc_ast::ast::TSTypeParameterInstantiation<'a>>> = if let Some(type_args_value) = estree.get("typeArguments") {
            if type_args_value.is_null() {
                None
            } else {
                self.context = self.context.clone().with_parent("TSInterfaceHeritage", "typeArguments");
                Some(self.convert_ts_type_parameter_instantiation(type_args_value)?)
            }
        } else {
            None
        };
        
        let heritage = self.builder.ts_interface_heritage(span, expression, type_arguments);
        Ok(heritage)
    }

    /// Convert an ESTree TSClassImplements to oxc TSClassImplements.
    fn convert_ts_class_implements(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::TSClassImplements<'a>> {
        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);
        let error_span = (start, end);
        
        // Get expression (required TSTypeName - not Expression!)
        self.context = self.context.clone().with_parent("TSClassImplements", "expression");
        let expression_value = estree.get("expression").ok_or_else(|| ConversionError::MissingField {
            field: "expression".to_string(),
            node_type: "TSClassImplements".to_string(),
            span: error_span,
        })?;
        let expression = self.convert_ts_type_name(expression_value)?;
        
        // Get typeArguments (optional TSTypeParameterInstantiation)
        let type_arguments: Option<oxc_allocator::Box<'a, oxc_ast::ast::TSTypeParameterInstantiation<'a>>> = if let Some(type_args_value) = estree.get("typeArguments") {
            if type_args_value.is_null() {
                None
            } else {
                self.context = self.context.clone().with_parent("TSClassImplements", "typeArguments");
                Some(self.convert_ts_type_parameter_instantiation(type_args_value)?)
            }
        } else {
            None
        };
        
        let class_implements = self.builder.ts_class_implements(span, expression, type_arguments);
        Ok(class_implements)
    }

    /// Convert an ESTree Decorator to oxc Decorator.
    fn convert_decorator(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::Decorator<'a>> {
        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);
        
        // Get expression (required Expression)
        self.context = self.context.clone().with_parent("Decorator", "expression");
        let expression_value = estree.get("expression").ok_or_else(|| ConversionError::MissingField {
            field: "expression".to_string(),
            node_type: "Decorator".to_string(),
            span: (start, end),
        })?;
        let expression = self.convert_expression(expression_value)?;
        
        let decorator = self.builder.decorator(span, expression);
        Ok(decorator)
    }

    /// Convert an ESTree TSTypeParameterDeclaration to oxc TSTypeParameterDeclaration.
    fn convert_ts_type_parameter_declaration(&mut self, estree: &Value) -> ConversionResult<oxc_allocator::Box<'a, oxc_ast::ast::TSTypeParameterDeclaration<'a>>> {
        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);
        let error_span = (start, end);
        
        // Get params (array of TSTypeParameter)
        self.context = self.context.clone().with_parent("TSTypeParameterDeclaration", "params");
        let params_value = estree.get("params").ok_or_else(|| ConversionError::MissingField {
            field: "params".to_string(),
            node_type: "TSTypeParameterDeclaration".to_string(),
            span: error_span,
        })?;
        let params_array = params_value.as_array().ok_or_else(|| ConversionError::InvalidFieldType {
            field: "params".to_string(),
            expected: "array".to_string(),
            got: format!("{:?}", params_value),
            span: error_span,
        })?;
        
        let mut type_params = Vec::new_in(self.builder.allocator);
        for param_value in params_array {
            self.context = self.context.clone().with_parent("TSTypeParameterDeclaration", "params");
            let type_param = self.convert_ts_type_parameter(param_value)?;
            type_params.push(type_param);
        }
        
        let type_param_decl = self.builder.alloc_ts_type_parameter_declaration(span, type_params);
        Ok(type_param_decl)
    }

    /// Convert an ESTree TSImportTypeQualifier to oxc TSImportTypeQualifier.
    fn convert_ts_import_type_qualifier(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::TSImportTypeQualifier<'a>> {
        use oxc_ast::ast::TSImportTypeQualifier;

        let node_type_str = estree.get("type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ConversionError::MissingField {
                field: "type".to_string(),
                node_type: "TSImportTypeQualifier".to_string(),
                span: self.get_node_span(estree),
            })?;

        match node_type_str {
            "Identifier" => {
                // Identifier -> IdentifierName
                let ident_name = self.convert_identifier_to_name(estree)?;
                Ok(TSImportTypeQualifier::Identifier(oxc_allocator::Box::new_in(ident_name, self.builder.allocator)))
            }
            "TSQualifiedName" => {
                // TSQualifiedName -> TSImportTypeQualifiedName
                // This is recursive - need to convert left and right separately
                let (start, end) = self.get_node_span(estree);
                let span = Span::new(start, end);
                let error_span = (start, end);
                
                // Get left (TSImportTypeQualifier)
                self.context = self.context.clone().with_parent("TSImportTypeQualifiedName", "left");
                let left_value = estree.get("left").ok_or_else(|| ConversionError::MissingField {
                    field: "left".to_string(),
                    node_type: "TSImportTypeQualifiedName".to_string(),
                    span: error_span,
                })?;
                let left = self.convert_ts_import_type_qualifier(left_value)?;
                
                // Get right (IdentifierName)
                self.context = self.context.clone().with_parent("TSImportTypeQualifiedName", "right");
                let right_value = estree.get("right").ok_or_else(|| ConversionError::MissingField {
                    field: "right".to_string(),
                    node_type: "TSImportTypeQualifiedName".to_string(),
                    span: error_span,
                })?;
                let right = self.convert_identifier_to_name(right_value)?;
                
                let qualified_name = self.builder.alloc_ts_import_type_qualified_name(span, left, right);
                Ok(TSImportTypeQualifier::QualifiedName(qualified_name))
            }
            _ => Err(ConversionError::UnsupportedNodeType {
                node_type: format!("TSImportTypeQualifier variant: {}", node_type_str),
                span: self.get_node_span(estree),
            }),
        }
    }

    /// Convert an ESTree TSTupleElement to oxc TSTupleElement.
    /// TSTupleElement can be a TSType, TSOptionalType, or TSRestType.
    fn convert_ts_tuple_element(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::TSTupleElement<'a>> {
        use oxc_ast::ast::{TSTupleElement, TSType};

        let node_type_str = estree.get("type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ConversionError::MissingField {
                field: "type".to_string(),
                node_type: "TSTupleElement".to_string(),
                span: self.get_node_span(estree),
            })?;

        match node_type_str {
            "TSOptionalType" => {
                // TSOptionalType: [number?]
                self.context = self.context.clone().with_parent("TSOptionalType", "typeAnnotation");
                let type_annotation_value = estree.get("typeAnnotation").ok_or_else(|| ConversionError::MissingField {
                    field: "typeAnnotation".to_string(),
                    node_type: "TSOptionalType".to_string(),
                    span: self.get_node_span(estree),
                })?;
                let type_annotation = self.convert_ts_type(type_annotation_value)?;
                let (start, end) = self.get_node_span(estree);
                let span = Span::new(start, end);
                let optional_type = self.builder.alloc_ts_optional_type(span, type_annotation);
                Ok(TSTupleElement::TSOptionalType(optional_type))
            }
            "TSRestType" => {
                // TSRestType: [...string[]]
                self.context = self.context.clone().with_parent("TSRestType", "typeAnnotation");
                let type_annotation_value = estree.get("typeAnnotation").ok_or_else(|| ConversionError::MissingField {
                    field: "typeAnnotation".to_string(),
                    node_type: "TSRestType".to_string(),
                    span: self.get_node_span(estree),
                })?;
                let type_annotation = self.convert_ts_type(type_annotation_value)?;
                let (start, end) = self.get_node_span(estree);
                let span = Span::new(start, end);
                let rest_type = self.builder.alloc_ts_rest_type(span, type_annotation);
                Ok(TSTupleElement::TSRestType(rest_type))
            }
            _ => {
                // Regular TSType (inherited by TSTupleElement)
                let ts_type = self.convert_ts_type(estree)?;
                // Convert TSType to TSTupleElement
                // Since TSTupleElement inherits from TSType, we can use the conversion
                match ts_type {
                    TSType::TSAnyKeyword(kw) => Ok(TSTupleElement::TSAnyKeyword(kw)),
                    TSType::TSBigIntKeyword(kw) => Ok(TSTupleElement::TSBigIntKeyword(kw)),
                    TSType::TSBooleanKeyword(kw) => Ok(TSTupleElement::TSBooleanKeyword(kw)),
                    TSType::TSIntrinsicKeyword(kw) => Ok(TSTupleElement::TSIntrinsicKeyword(kw)),
                    TSType::TSNeverKeyword(kw) => Ok(TSTupleElement::TSNeverKeyword(kw)),
                    TSType::TSNullKeyword(kw) => Ok(TSTupleElement::TSNullKeyword(kw)),
                    TSType::TSNumberKeyword(kw) => Ok(TSTupleElement::TSNumberKeyword(kw)),
                    TSType::TSObjectKeyword(kw) => Ok(TSTupleElement::TSObjectKeyword(kw)),
                    TSType::TSStringKeyword(kw) => Ok(TSTupleElement::TSStringKeyword(kw)),
                    TSType::TSSymbolKeyword(kw) => Ok(TSTupleElement::TSSymbolKeyword(kw)),
                    TSType::TSUndefinedKeyword(kw) => Ok(TSTupleElement::TSUndefinedKeyword(kw)),
                    TSType::TSUnknownKeyword(kw) => Ok(TSTupleElement::TSUnknownKeyword(kw)),
                    TSType::TSVoidKeyword(kw) => Ok(TSTupleElement::TSVoidKeyword(kw)),
                    TSType::TSThisType(tt) => Ok(TSTupleElement::TSThisType(tt)),
                    TSType::TSArrayType(arr) => Ok(TSTupleElement::TSArrayType(arr)),
                    TSType::TSConditionalType(cond) => Ok(TSTupleElement::TSConditionalType(cond)),
                    TSType::TSConstructorType(ctor) => Ok(TSTupleElement::TSConstructorType(ctor)),
                    TSType::TSFunctionType(func) => Ok(TSTupleElement::TSFunctionType(func)),
                    TSType::TSImportType(imp) => Ok(TSTupleElement::TSImportType(imp)),
                    TSType::TSIndexedAccessType(idx) => Ok(TSTupleElement::TSIndexedAccessType(idx)),
                    TSType::TSInferType(inf) => Ok(TSTupleElement::TSInferType(inf)),
                    TSType::TSIntersectionType(int) => Ok(TSTupleElement::TSIntersectionType(int)),
                    TSType::TSLiteralType(lit) => Ok(TSTupleElement::TSLiteralType(lit)),
                    TSType::TSMappedType(map) => Ok(TSTupleElement::TSMappedType(map)),
                    TSType::TSNamedTupleMember(mem) => Ok(TSTupleElement::TSNamedTupleMember(mem)),
                    TSType::TSTemplateLiteralType(tmpl) => Ok(TSTupleElement::TSTemplateLiteralType(tmpl)),
                    TSType::TSTupleType(tup) => Ok(TSTupleElement::TSTupleType(tup)),
                    TSType::TSTypeLiteral(lit) => Ok(TSTupleElement::TSTypeLiteral(lit)),
                    TSType::TSTypeOperatorType(op) => Ok(TSTupleElement::TSTypeOperatorType(op)),
                    TSType::TSTypePredicate(pred) => Ok(TSTupleElement::TSTypePredicate(pred)),
                    TSType::TSTypeQuery(qry) => Ok(TSTupleElement::TSTypeQuery(qry)),
                    TSType::TSTypeReference(ref_) => Ok(TSTupleElement::TSTypeReference(ref_)),
                    TSType::TSUnionType(uni) => Ok(TSTupleElement::TSUnionType(uni)),
                    TSType::TSParenthesizedType(par) => Ok(TSTupleElement::TSParenthesizedType(par)),
                    TSType::JSDocNullableType(null) => Ok(TSTupleElement::JSDocNullableType(null)),
                    TSType::JSDocNonNullableType(nonnull) => Ok(TSTupleElement::JSDocNonNullableType(nonnull)),
                    TSType::JSDocUnknownType(unk) => Ok(TSTupleElement::JSDocUnknownType(unk)),
                }
            }
        }
    }

    /// Convert an ESTree TSSignature to oxc TSSignature.
    /// TSSignature can be TSIndexSignature, TSPropertySignature, TSCallSignatureDeclaration,
    /// TSConstructSignatureDeclaration, or TSMethodSignature.
    fn convert_ts_signature(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::TSSignature<'a>> {
        use oxc_ast::ast::TSSignature;

        let node_type_str = estree.get("type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ConversionError::MissingField {
                field: "type".to_string(),
                node_type: "TSSignature".to_string(),
                span: self.get_node_span(estree),
            })?;

        match node_type_str {
            "TSPropertySignature" => {
                // TSPropertySignature: { x: number }
                let (start, end) = self.get_node_span(estree);
                let span = Span::new(start, end);
                let error_span = (start, end);
                
                // Get key (PropertyKey)
                self.context = self.context.clone().with_parent("TSPropertySignature", "key");
                let key_value = estree.get("key").ok_or_else(|| ConversionError::MissingField {
                    field: "key".to_string(),
                    node_type: "TSPropertySignature".to_string(),
                    span: error_span,
                })?;
                let key = self.convert_property_key(key_value)?;
                
                // Get computed flag
                let computed = key_value.get("computed")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                
                // Get typeAnnotation (optional)
                let type_annotation = if let Some(type_ann_value) = estree.get("typeAnnotation") {
                    self.context = self.context.clone().with_parent("TSPropertySignature", "typeAnnotation");
                    let ts_type = self.convert_ts_type(type_ann_value)?;
                    let type_ann_span = self.get_node_span(type_ann_value);
                    Some(oxc_allocator::Box::new_in(
                        oxc_ast::ast::TSTypeAnnotation {
                            span: Span::new(type_ann_span.0, type_ann_span.1),
                            type_annotation: ts_type,
                        },
                        self.builder.allocator,
                    ))
                } else {
                    None
                };
                
                // Get optional flag
                let optional = estree.get("optional")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                
                // Get readonly flag
                let readonly = estree.get("readonly")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                
                let property_sig = self.builder.ts_signature_property_signature(span, computed, optional, readonly, key, type_annotation);
                Ok(property_sig)
            }
            "TSIndexSignature" => {
                // TSIndexSignature: { [key: string]: number }
                let (start, end) = self.get_node_span(estree);
                let span = Span::new(start, end);
                let error_span = (start, end);
                
                // Get parameters (TSIndexSignatureName[])
                self.context = self.context.clone().with_parent("TSIndexSignature", "parameters");
                let params_value = estree.get("parameters").ok_or_else(|| ConversionError::MissingField {
                    field: "parameters".to_string(),
                    node_type: "TSIndexSignature".to_string(),
                    span: error_span,
                })?;
                let params_array = params_value.as_array().ok_or_else(|| ConversionError::InvalidFieldType {
                    field: "parameters".to_string(),
                    expected: "array".to_string(),
                    got: format!("{:?}", params_value),
                    span: error_span,
                })?;
                
                let mut parameters = Vec::new_in(self.builder.allocator);
                for param_value in params_array {
                    // TSIndexSignatureName is typically an Identifier with required type annotation
                    let param_name_ident = self.convert_identifier_to_name(param_value)?;
                    let param_span = self.get_node_span(param_value);
                    
                    // Get typeAnnotation (required in oxc AST, but may be missing in ESTree)
                    let type_annotation = if let Some(type_ann_value) = param_value.get("typeAnnotation") {
                        self.context = self.context.clone().with_parent("TSIndexSignatureName", "typeAnnotation");
                        let ts_type = self.convert_ts_type(type_ann_value)?;
                        let type_ann_span = self.get_node_span(type_ann_value);
                        oxc_allocator::Box::new_in(
                            oxc_ast::ast::TSTypeAnnotation {
                                span: Span::new(type_ann_span.0, type_ann_span.1),
                                type_annotation: ts_type,
                            },
                            self.builder.allocator,
                        )
                    } else {
                        // Create a default TSAnyKeyword type annotation if missing
                        let any_span = Span::new(param_span.1, param_span.1);
                        let any_keyword = self.builder.alloc_ts_any_keyword(any_span);
                        oxc_allocator::Box::new_in(
                            oxc_ast::ast::TSTypeAnnotation {
                                span: any_span,
                                type_annotation: oxc_ast::ast::TSType::TSAnyKeyword(any_keyword),
                            },
                            self.builder.allocator,
                        )
                    };
                    
                    let index_sig_name = oxc_ast::ast::TSIndexSignatureName {
                        span: Span::new(param_span.0, param_span.1),
                        name: param_name_ident.name,
                        type_annotation,
                    };
                    parameters.push(index_sig_name);
                }
                
                // Get typeAnnotation (required)
                self.context = self.context.clone().with_parent("TSIndexSignature", "typeAnnotation");
                let type_ann_value = estree.get("typeAnnotation").ok_or_else(|| ConversionError::MissingField {
                    field: "typeAnnotation".to_string(),
                    node_type: "TSIndexSignature".to_string(),
                    span: error_span,
                })?;
                let ts_type = self.convert_ts_type(type_ann_value)?;
                let type_ann_span = self.get_node_span(type_ann_value);
                let type_annotation = oxc_allocator::Box::new_in(
                    oxc_ast::ast::TSTypeAnnotation {
                        span: Span::new(type_ann_span.0, type_ann_span.1),
                        type_annotation: ts_type,
                    },
                    self.builder.allocator,
                );
                
                // Get readonly flag
                let readonly = estree.get("readonly")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                
                // Get static flag
                let r#static = estree.get("static")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                
                let index_sig = self.builder.alloc_ts_index_signature(span, parameters, type_annotation, readonly, r#static);
                Ok(TSSignature::TSIndexSignature(index_sig))
            }
            "TSCallSignatureDeclaration" => {
                // TSCallSignatureDeclaration: () => void
                let (start, end) = self.get_node_span(estree);
                let span = Span::new(start, end);
                let error_span = (start, end);
                
                // Get typeParameters (optional)
                let type_parameters: Option<oxc_allocator::Box<'a, oxc_ast::ast::TSTypeParameterDeclaration<'a>>> = if let Some(type_params_value) = estree.get("typeParameters") {
                    if type_params_value.is_null() {
                        None
                    } else {
                        self.context = self.context.clone().with_parent("TSCallSignatureDeclaration", "typeParameters");
                        Some(self.convert_ts_type_parameter_declaration(type_params_value)?)
                    }
                } else {
                    None
                };
                
                // Get thisParam (optional, skipped in ESTree)
                let this_param: Option<oxc_allocator::Box<'a, oxc_ast::ast::TSThisParameter<'a>>> = None;
                
                // Get params (FormalParameters) - similar to TSFunctionType
                self.context = self.context.clone().with_parent("TSCallSignatureDeclaration", "params");
                let params_value = estree.get("params").ok_or_else(|| ConversionError::MissingField {
                    field: "params".to_string(),
                    node_type: "TSCallSignatureDeclaration".to_string(),
                    span: error_span,
                })?;
                let params_array = params_value.as_array().ok_or_else(|| ConversionError::InvalidFieldType {
                    field: "params".to_string(),
                    expected: "array".to_string(),
                    got: format!("{:?}", params_value),
                    span: error_span,
                })?;
                
                // Convert parameters (same logic as TSFunctionType)
                let mut params_vec = Vec::new_in(self.builder.allocator);
                let mut rest_param: Option<oxc_allocator::Box<'a, oxc_ast::ast::BindingRestElement<'a>>> = None;
                
                for (idx, param_value) in params_array.iter().enumerate() {
                    let param_context = self.context.clone().with_parent("TSCallSignatureDeclaration", "params");
                    self.context = param_context;
                    self.context.is_binding_context = true;
                    
                    use oxc_estree::deserialize::{EstreeNode, EstreeNodeType};
                    
                    let node_type = <Value as EstreeNode>::get_type(param_value).ok_or_else(|| ConversionError::MissingField {
                        field: "type".to_string(),
                        node_type: "TSCallSignatureDeclaration.params".to_string(),
                        span: self.get_node_span(param_value),
                    })?;
                    
                    if node_type == EstreeNodeType::RestElement {
                        if idx != params_array.len() - 1 {
                            return Err(ConversionError::InvalidFieldType {
                                field: "params".to_string(),
                                expected: "RestElement must be last parameter".to_string(),
                                got: format!("RestElement at index {}", idx),
                                span: self.get_node_span(param_value),
                            });
                        }
                        let rest = self.convert_rest_element_to_binding_rest(param_value)?;
                        rest_param = Some(rest);
                    } else {
                        let formal_param = self.convert_to_formal_parameter(param_value)?;
                        params_vec.push(formal_param);
                    }
                }
                
                let params_span = if let Some(first_param) = params_array.first() {
                    let (start, _) = self.get_node_span(first_param);
                    let (_, end) = params_array.last().map(|p| self.get_node_span(p)).unwrap_or((start, start));
                    Span::new(start, end)
                } else {
                    Span::new(start, start)
                };
                
                let formal_params = self.builder.alloc_formal_parameters(params_span, oxc_ast::ast::FormalParameterKind::Signature, params_vec, rest_param);
                
                // Get returnType (optional)
                let return_type = if let Some(return_type_value) = estree.get("returnType") {
                    self.context = self.context.clone().with_parent("TSCallSignatureDeclaration", "returnType");
                    let ts_type = self.convert_ts_type(return_type_value)?;
                    let return_type_span = self.get_node_span(return_type_value);
                    Some(oxc_allocator::Box::new_in(
                        oxc_ast::ast::TSTypeAnnotation {
                            span: Span::new(return_type_span.0, return_type_span.1),
                            type_annotation: ts_type,
                        },
                        self.builder.allocator,
                    ))
                } else {
                    None
                };
                
                let call_signature = self.builder.alloc_ts_call_signature_declaration(span, type_parameters, this_param, formal_params, return_type);
                Ok(TSSignature::TSCallSignatureDeclaration(call_signature))
            }
            "TSConstructSignatureDeclaration" => {
                // TSConstructSignatureDeclaration: new () => void
                let (start, end) = self.get_node_span(estree);
                let span = Span::new(start, end);
                let error_span = (start, end);
                
                // Get typeParameters (optional)
                let type_parameters: Option<oxc_allocator::Box<'a, oxc_ast::ast::TSTypeParameterDeclaration<'a>>> = if let Some(type_params_value) = estree.get("typeParameters") {
                    if type_params_value.is_null() {
                        None
                    } else {
                        self.context = self.context.clone().with_parent("TSConstructSignatureDeclaration", "typeParameters");
                        Some(self.convert_ts_type_parameter_declaration(type_params_value)?)
                    }
                } else {
                    None
                };
                
                // Get params (FormalParameters) - similar to TSFunctionType
                self.context = self.context.clone().with_parent("TSConstructSignatureDeclaration", "params");
                let params_value = estree.get("params").ok_or_else(|| ConversionError::MissingField {
                    field: "params".to_string(),
                    node_type: "TSConstructSignatureDeclaration".to_string(),
                    span: error_span,
                })?;
                let params_array = params_value.as_array().ok_or_else(|| ConversionError::InvalidFieldType {
                    field: "params".to_string(),
                    expected: "array".to_string(),
                    got: format!("{:?}", params_value),
                    span: error_span,
                })?;
                
                // Convert parameters (same logic as TSFunctionType)
                let mut params_vec = Vec::new_in(self.builder.allocator);
                let mut rest_param: Option<oxc_allocator::Box<'a, oxc_ast::ast::BindingRestElement<'a>>> = None;
                
                for (idx, param_value) in params_array.iter().enumerate() {
                    let param_context = self.context.clone().with_parent("TSConstructSignatureDeclaration", "params");
                    self.context = param_context;
                    self.context.is_binding_context = true;
                    
                    use oxc_estree::deserialize::{EstreeNode, EstreeNodeType};
                    
                    let node_type = <Value as EstreeNode>::get_type(param_value).ok_or_else(|| ConversionError::MissingField {
                        field: "type".to_string(),
                        node_type: "TSConstructSignatureDeclaration.params".to_string(),
                        span: self.get_node_span(param_value),
                    })?;
                    
                    if node_type == EstreeNodeType::RestElement {
                        if idx != params_array.len() - 1 {
                            return Err(ConversionError::InvalidFieldType {
                                field: "params".to_string(),
                                expected: "RestElement must be last parameter".to_string(),
                                got: format!("RestElement at index {}", idx),
                                span: self.get_node_span(param_value),
                            });
                        }
                        let rest = self.convert_rest_element_to_binding_rest(param_value)?;
                        rest_param = Some(rest);
                    } else {
                        let formal_param = self.convert_to_formal_parameter(param_value)?;
                        params_vec.push(formal_param);
                    }
                }
                
                let params_span = if let Some(first_param) = params_array.first() {
                    let (start, _) = self.get_node_span(first_param);
                    let (_, end) = params_array.last().map(|p| self.get_node_span(p)).unwrap_or((start, start));
                    Span::new(start, end)
                } else {
                    Span::new(start, start)
                };
                
                let formal_params = self.builder.alloc_formal_parameters(params_span, oxc_ast::ast::FormalParameterKind::Signature, params_vec, rest_param);
                
                // Get returnType (optional)
                let return_type = if let Some(return_type_value) = estree.get("returnType") {
                    self.context = self.context.clone().with_parent("TSConstructSignatureDeclaration", "returnType");
                    let ts_type = self.convert_ts_type(return_type_value)?;
                    let return_type_span = self.get_node_span(return_type_value);
                    Some(oxc_allocator::Box::new_in(
                        oxc_ast::ast::TSTypeAnnotation {
                            span: Span::new(return_type_span.0, return_type_span.1),
                            type_annotation: ts_type,
                        },
                        self.builder.allocator,
                    ))
                } else {
                    None
                };
                
                let construct_signature = self.builder.alloc_ts_construct_signature_declaration(span, type_parameters, formal_params, return_type);
                Ok(TSSignature::TSConstructSignatureDeclaration(construct_signature))
            }
            "TSMethodSignature" => {
                // TSMethodSignature: bar(a: number): string;
                let (start, end) = self.get_node_span(estree);
                let span = Span::new(start, end);
                let error_span = (start, end);
                
                // Get key (PropertyKey)
                self.context = self.context.clone().with_parent("TSMethodSignature", "key");
                let key_value = estree.get("key").ok_or_else(|| ConversionError::MissingField {
                    field: "key".to_string(),
                    node_type: "TSMethodSignature".to_string(),
                    span: error_span,
                })?;
                let key = self.convert_property_key(key_value)?;
                
                // Get computed flag
                let computed = estree.get("computed")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                
                // Get optional flag
                let optional = estree.get("optional")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                
                // Get kind (TSMethodSignatureKind: Method, Get, Set)
                let kind_str = estree.get("kind")
                    .and_then(|v| v.as_str())
                    .unwrap_or("method");
                let kind = match kind_str {
                    "method" => oxc_ast::ast::TSMethodSignatureKind::Method,
                    "get" => oxc_ast::ast::TSMethodSignatureKind::Get,
                    "set" => oxc_ast::ast::TSMethodSignatureKind::Set,
                    _ => {
                        return Err(ConversionError::InvalidFieldType {
                            field: "kind".to_string(),
                            expected: "method, get, or set".to_string(),
                            got: kind_str.to_string(),
                            span: error_span,
                        });
                    }
                };
                
                // Get typeParameters (optional)
                let type_parameters: Option<oxc_allocator::Box<'a, oxc_ast::ast::TSTypeParameterDeclaration<'a>>> = if let Some(type_params_value) = estree.get("typeParameters") {
                    if type_params_value.is_null() {
                        None
                    } else {
                        self.context = self.context.clone().with_parent("TSMethodSignature", "typeParameters");
                        Some(self.convert_ts_type_parameter_declaration(type_params_value)?)
                    }
                } else {
                    None
                };
                
                // Get thisParam (optional, skipped in ESTree)
                let this_param: Option<oxc_allocator::Box<'a, oxc_ast::ast::TSThisParameter<'a>>> = None;
                
                // Get params (FormalParameters) - similar to TSFunctionType
                self.context = self.context.clone().with_parent("TSMethodSignature", "params");
                let params_value = estree.get("params").ok_or_else(|| ConversionError::MissingField {
                    field: "params".to_string(),
                    node_type: "TSMethodSignature".to_string(),
                    span: error_span,
                })?;
                let params_array = params_value.as_array().ok_or_else(|| ConversionError::InvalidFieldType {
                    field: "params".to_string(),
                    expected: "array".to_string(),
                    got: format!("{:?}", params_value),
                    span: error_span,
                })?;
                
                // Convert parameters (same logic as TSFunctionType)
                let mut params_vec = Vec::new_in(self.builder.allocator);
                let mut rest_param: Option<oxc_allocator::Box<'a, oxc_ast::ast::BindingRestElement<'a>>> = None;
                
                for (idx, param_value) in params_array.iter().enumerate() {
                    let param_context = self.context.clone().with_parent("TSMethodSignature", "params");
                    self.context = param_context;
                    self.context.is_binding_context = true;
                    
                    use oxc_estree::deserialize::{EstreeNode, EstreeNodeType};
                    
                    let node_type = <Value as EstreeNode>::get_type(param_value).ok_or_else(|| ConversionError::MissingField {
                        field: "type".to_string(),
                        node_type: "TSMethodSignature.params".to_string(),
                        span: self.get_node_span(param_value),
                    })?;
                    
                    if node_type == EstreeNodeType::RestElement {
                        if idx != params_array.len() - 1 {
                            return Err(ConversionError::InvalidFieldType {
                                field: "params".to_string(),
                                expected: "RestElement must be last parameter".to_string(),
                                got: format!("RestElement at index {}", idx),
                                span: self.get_node_span(param_value),
                            });
                        }
                        let rest = self.convert_rest_element_to_binding_rest(param_value)?;
                        rest_param = Some(rest);
                    } else {
                        let formal_param = self.convert_to_formal_parameter(param_value)?;
                        params_vec.push(formal_param);
                    }
                }
                
                let params_span = if let Some(first_param) = params_array.first() {
                    let (start, _) = self.get_node_span(first_param);
                    let (_, end) = params_array.last().map(|p| self.get_node_span(p)).unwrap_or((start, start));
                    Span::new(start, end)
                } else {
                    Span::new(start, start)
                };
                
                let formal_params = self.builder.alloc_formal_parameters(params_span, oxc_ast::ast::FormalParameterKind::Signature, params_vec, rest_param);
                
                // Get returnType (optional)
                let return_type = if let Some(return_type_value) = estree.get("returnType") {
                    self.context = self.context.clone().with_parent("TSMethodSignature", "returnType");
                    let ts_type = self.convert_ts_type(return_type_value)?;
                    let return_type_span = self.get_node_span(return_type_value);
                    Some(oxc_allocator::Box::new_in(
                        oxc_ast::ast::TSTypeAnnotation {
                            span: Span::new(return_type_span.0, return_type_span.1),
                            type_annotation: ts_type,
                        },
                        self.builder.allocator,
                    ))
                } else {
                    None
                };
                
                let method_signature = self.builder.alloc_ts_method_signature(span, key, computed, optional, kind, type_parameters, this_param, formal_params, return_type);
                Ok(TSSignature::TSMethodSignature(method_signature))
            }
            _ => Err(ConversionError::UnsupportedNodeType {
                node_type: format!("Unknown TSSignature variant: {}", node_type_str),
                span: self.get_node_span(estree),
            }),
        }
    }

    /// Convert an ESTree TSTypeAssertion to oxc Expression.
    fn convert_ts_type_assertion(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::Expression<'a>> {
        use oxc_ast::ast::Expression;

        // Get expression
        self.context = self.context.clone().with_parent("TSTypeAssertion", "expression");
        let expr_value = estree.get("expression").ok_or_else(|| ConversionError::MissingField {
            field: "expression".to_string(),
            node_type: "TSTypeAssertion".to_string(),
            span: self.get_node_span(estree),
        })?;
        let expression = self.convert_expression(expr_value)?;

        // Get typeAnnotation (TSType)
        self.context = self.context.clone().with_parent("TSTypeAssertion", "typeAnnotation");
        let type_annotation_value = estree.get("typeAnnotation").ok_or_else(|| ConversionError::MissingField {
            field: "typeAnnotation".to_string(),
            node_type: "TSTypeAssertion".to_string(),
            span: self.get_node_span(estree),
        })?;
        let type_annotation = self.convert_ts_type(type_annotation_value)?;

        // Build TSTypeAssertion
        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);
        let ts_type_assertion = self.builder.alloc_ts_type_assertion(span, type_annotation, expression);
        Ok(Expression::TSTypeAssertion(ts_type_assertion))
    }

    /// Convert an ESTree node to oxc AssignmentTarget.
    fn convert_to_assignment_target(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::AssignmentTarget<'a>> {
        use oxc_ast::ast::{AssignmentTarget, IdentifierReference};
        use oxc_estree::deserialize::{convert_identifier, EstreeIdentifier, EstreeNode, EstreeNodeType, IdentifierKind};
        use oxc_span::Atom;

        // Debug: check if estree is actually a JSON object
        if !estree.is_object() {
            return Err(ConversionError::InvalidFieldType {
                field: "assignment_target".to_string(),
                expected: "object".to_string(),
                got: format!("{:?}", estree),
                span: self.get_node_span(estree),
            });
        }
        let node_type = <Value as EstreeNode>::get_type(estree).ok_or_else(|| {
            // Debug: check what we actually have
            let has_type = estree.get("type").is_some();
            let type_str = estree.get("type").and_then(|v| v.as_str()).unwrap_or("none");
            ConversionError::MissingField {
                field: "type".to_string(),
                node_type: format!("convert_to_assignment_target (is_object: {}, has_type: {}, type: {})", estree.is_object(), has_type, type_str),
                span: self.get_node_span(estree),
            }
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
            EstreeNodeType::MemberExpression => {
                // MemberExpression can be converted to AssignmentTarget
                // Convert as expression first, then check if it's a member expression variant
                use oxc_ast::ast::Expression;
                let expr = self.convert_expression(estree)?;
                
                // Match the expression to see if it's a member expression variant
                match expr {
                    Expression::ComputedMemberExpression(member) => {
                        Ok(AssignmentTarget::ComputedMemberExpression(member))
                    }
                    Expression::StaticMemberExpression(member) => {
                        Ok(AssignmentTarget::StaticMemberExpression(member))
                    }
                    Expression::PrivateFieldExpression(member) => {
                        Ok(AssignmentTarget::PrivateFieldExpression(member))
                    }
                    _ => Err(ConversionError::UnsupportedNodeType {
                        node_type: format!("AssignmentTarget from MemberExpression: expected member expression variant, got {:?}", expr),
                        span: self.get_node_span(estree),
                    }),
                }
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
                // Create an Elision node for sparse array elements
                // We need to estimate the span - in ESTree, elisions don't have explicit ranges
                // We'll use a minimal span (1 byte) to represent the comma
                // The actual position would need to be calculated from the source text
                // For now, use a placeholder span that will be adjusted if needed
                let (array_start, _array_end) = self.get_node_span(estree);
                // Use a minimal span for elision (represents the comma)
                // In practice, the span should be calculated from source text position
                let elision_span = Span::new(array_start, array_start + 1);
                let elision = oxc_ast::ast::Elision { span: elision_span };
                elements.push(ArrayExpressionElement::Elision(elision));
                continue;
            }

            self.context = self.context.clone().with_parent("ArrayExpression", "elements");
            // Check if it's a SpreadElement or regular expression
            let elem_type = <Value as EstreeNode>::get_type(elem_value);
            if let Some(EstreeNodeType::SpreadElement) = elem_type {
                let spread = self.convert_spread_element(elem_value)?;
                elements.push(ArrayExpressionElement::SpreadElement(spread));
            } else {
                // Try as expression
                let expr = self.convert_expression(elem_value)?;
                elements.push(ArrayExpressionElement::from(expr));
            }
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
            let arg = self.convert_to_argument(arg_value)?;
            args.push(arg);
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

