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
        for quasi_value in quasis_array {
            self.context = self.context.clone().with_parent("TemplateLiteral", "quasis");
            let template_element = self.convert_template_element(quasi_value)?;
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
        for quasi_elem_value in quasis_array {
            self.context = self.context.clone().with_parent("TaggedTemplateExpression", "quasi.quasis");
            let template_element = self.convert_template_element(quasi_elem_value)?;
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

        let type_args: Option<oxc_allocator::Box<'a, oxc_ast::ast::TSTypeParameterInstantiation<'a>>> = None;
        let tagged = self.builder.alloc_tagged_template_expression(span, tag, type_args, quasi);
        Ok(Expression::TaggedTemplateExpression(tagged))
    }

    /// Convert an ESTree TemplateElement to oxc TemplateElement.
    fn convert_template_element(&mut self, estree: &Value) -> ConversionResult<oxc_ast::ast::TemplateElement<'a>> {
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

        // Get tail
        let tail = estree.get("tail").and_then(|v| v.as_bool()).unwrap_or(false);

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
        for param_value in params_array {
            // TODO: Convert param to FormalParameter
            // For now, skip params
        }

        let (params_start, params_end) = self.get_node_span(params_value);
        let params_span = Span::new(params_start, params_end);
        let rest: Option<oxc_allocator::Box<'a, oxc_ast::ast::BindingRestElement<'a>>> = None;
        let params = self.builder.formal_parameters(params_span, FormalParameterKind::ArrowFormalParameters, param_items, rest);
        let params_box = oxc_allocator::Box::new_in(params, self.builder.allocator);

        // Get body - can be Expression or BlockStatement
        self.context = self.context.clone().with_parent("ArrowFunctionExpression", "body");
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

        let type_params: Option<oxc_allocator::Box<'a, oxc_ast::ast::TSTypeParameterDeclaration<'a>>> = None;
        let return_type: Option<oxc_allocator::Box<'a, oxc_ast::ast::TSTypeAnnotation<'a>>> = None;
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
        for param_value in params_array {
            // TODO: Convert param to FormalParameter
            // For now, skip params
        }

        let (params_start, params_end) = self.get_node_span(params_value);
        let params_span = Span::new(params_start, params_end);
        let rest: Option<oxc_allocator::Box<'a, oxc_ast::ast::BindingRestElement<'a>>> = None;
        let params = self.builder.formal_parameters(params_span, FormalParameterKind::FormalParameter, param_items, rest);
        let params_box = oxc_allocator::Box::new_in(params, self.builder.allocator);

        // Get body
        self.context = self.context.clone().with_parent("Function", "body");
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
        let type_params: Option<oxc_allocator::Box<'a, oxc_ast::ast::TSTypeParameterDeclaration<'a>>> = None;
        let this_param: Option<oxc_allocator::Box<'a, oxc_ast::ast::TSThisParameter<'a>>> = None;
        let return_type: Option<oxc_allocator::Box<'a, oxc_ast::ast::TSTypeAnnotation<'a>>> = None;
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
            // For now, treat all arguments as expressions
            // TODO: Handle SpreadElement separately
            let arg_expr = self.convert_expression(arg_value)?;
            args.push(Argument::from(arg_expr));
        }

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        let type_args: Option<oxc_allocator::Box<'a, oxc_ast::ast::TSTypeParameterInstantiation<'a>>> = None;
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

        // Get decorators (optional, empty for now)
        let decorators = Vec::new_in(self.builder.allocator);

        // Get implements (optional, empty for now - TypeScript only)
        let implements = Vec::new_in(self.builder.allocator);

        // Get abstract and declare (optional, false for now)
        let r#abstract = estree.get("abstract").and_then(|v| v.as_bool()).unwrap_or(false);
        let declare = estree.get("declare").and_then(|v| v.as_bool()).unwrap_or(false);

        let (start, end) = self.get_node_span(estree);
        let span = Span::new(start, end);

        let type_params: Option<oxc_allocator::Box<'a, oxc_ast::ast::TSTypeParameterDeclaration<'a>>> = None;
        let super_type_args: Option<oxc_allocator::Box<'a, oxc_ast::ast::TSTypeParameterInstantiation<'a>>> = None;
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

        // Get accessibility (TypeScript) - None for now
        let accessibility: Option<oxc_ast::ast::TSAccessibility> = None;

        // Get decorators (empty for now)
        let decorators = Vec::new_in(self.builder.allocator);

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

        // Get type_annotation (TypeScript) - None for now
        let type_annotation: Option<oxc_allocator::Box<'a, oxc_ast::ast::TSTypeAnnotation<'a>>> = None;

        // Get accessibility (TypeScript) - None for now
        let accessibility: Option<oxc_ast::ast::TSAccessibility> = None;

        // Get decorators (empty for now)
        let decorators = Vec::new_in(self.builder.allocator);

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

        // Get type_annotation (TypeScript) - None for now
        let type_annotation: Option<oxc_allocator::Box<'a, oxc_ast::ast::TSTypeAnnotation<'a>>> = None;

        // Get accessibility (TypeScript) - None for now
        let accessibility: Option<oxc_ast::ast::TSAccessibility> = None;

        // Get decorators (empty for now)
        let decorators = Vec::new_in(self.builder.allocator);

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

        // Get attributes/with_clause (optional, None for now)
        let with_clause: Option<oxc_allocator::Box<'a, oxc_ast::ast::WithClause<'a>>> = None;

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
                // Convert to Declaration - this is complex, for now return None
                // TODO: Implement declaration conversion
                None
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

        // Get attributes/with_clause (optional, None for now)
        let with_clause: Option<oxc_allocator::Box<'a, oxc_ast::ast::WithClause<'a>>> = None;

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
        let type_parameters: Option<oxc_allocator::Box<'a, oxc_ast::ast::TSTypeParameterDeclaration<'a>>> = None;

        // Get extends (optional, empty for now)
        let extends = Vec::new_in(self.builder.allocator);

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
        let body_array = body_value.as_array().ok_or_else(|| ConversionError::InvalidFieldType {
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
        let members_array = members_value.as_array().ok_or_else(|| ConversionError::InvalidFieldType {
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
        let type_parameters: Option<oxc_allocator::Box<'a, oxc_ast::ast::TSTypeParameterDeclaration<'a>>> = None;

        // Get typeAnnotation (required)
        self.context = self.context.clone().with_parent("TSTypeAliasDeclaration", "typeAnnotation");
        let _type_annotation_value = estree.get("typeAnnotation").ok_or_else(|| ConversionError::MissingField {
            field: "typeAnnotation".to_string(),
            node_type: "TSTypeAliasDeclaration".to_string(),
            span: self.get_node_span(estree),
        })?;
        // For now, return error as TSType conversion is complex
        // TODO: Implement TSType conversion
        return Err(ConversionError::UnsupportedNodeType {
            node_type: "TSTypeAliasDeclaration (TSType conversion not yet implemented)".to_string(),
            span: self.get_node_span(estree),
        });
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

        // Get body (optional, empty for now)
        let body: Option<TSModuleDeclarationBody> = {
            // For now, create an empty TSModuleBlock
            let empty_span = Span::new(0, 0);
            let directives = Vec::new_in(self.builder.allocator);
            let body_statements = Vec::new_in(self.builder.allocator);
            let module_block = self.builder.ts_module_block(empty_span, directives, body_statements);
            Some(TSModuleDeclarationBody::TSModuleBlock(oxc_allocator::Box::new_in(module_block, self.builder.allocator)))
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
        
        // For now, only handle ExternalModuleReference (string literal)
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
            // TODO: Handle TSTypeName variants
            return Err(ConversionError::UnsupportedNodeType {
                node_type: "TSImportEqualsDeclaration.moduleReference (only ExternalModuleReference supported)".to_string(),
                span: self.get_node_span(estree),
            });
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

        // Get typeAnnotation (TSType - not yet implemented, return error for now)
        // TODO: Implement TSType conversion
        let _type_annotation_value = estree.get("typeAnnotation").ok_or_else(|| ConversionError::MissingField {
            field: "typeAnnotation".to_string(),
            node_type: "TSAsExpression".to_string(),
            span: self.get_node_span(estree),
        })?;

        // For now, return error as TSType conversion is complex
        return Err(ConversionError::UnsupportedNodeType {
            node_type: "TSAsExpression (TSType conversion not yet implemented)".to_string(),
            span: self.get_node_span(estree),
        });
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

        // Get typeAnnotation (TSType - not yet implemented, return error for now)
        // TODO: Implement TSType conversion
        let _type_annotation_value = estree.get("typeAnnotation").ok_or_else(|| ConversionError::MissingField {
            field: "typeAnnotation".to_string(),
            node_type: "TSSatisfiesExpression".to_string(),
            span: self.get_node_span(estree),
        })?;

        // For now, return error as TSType conversion is complex
        return Err(ConversionError::UnsupportedNodeType {
            node_type: "TSSatisfiesExpression (TSType conversion not yet implemented)".to_string(),
            span: self.get_node_span(estree),
        });
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

        // Get typeArguments (TSTypeParameterInstantiation - not yet implemented, return error for now)
        // TODO: Implement TSTypeParameterInstantiation conversion
        let _type_args_value = estree.get("typeArguments").ok_or_else(|| ConversionError::MissingField {
            field: "typeArguments".to_string(),
            node_type: "TSInstantiationExpression".to_string(),
            span: self.get_node_span(estree),
        })?;

        // For now, return error as TSTypeParameterInstantiation conversion is complex
        return Err(ConversionError::UnsupportedNodeType {
            node_type: "TSInstantiationExpression (TSTypeParameterInstantiation conversion not yet implemented)".to_string(),
            span: self.get_node_span(estree),
        });
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

        // Get typeAnnotation (TSType - not yet implemented, return error for now)
        // TODO: Implement TSType conversion
        let _type_annotation_value = estree.get("typeAnnotation").ok_or_else(|| ConversionError::MissingField {
            field: "typeAnnotation".to_string(),
            node_type: "TSTypeAssertion".to_string(),
            span: self.get_node_span(estree),
        })?;

        // For now, return error as TSType conversion is complex
        return Err(ConversionError::UnsupportedNodeType {
            node_type: "TSTypeAssertion (TSType conversion not yet implemented)".to_string(),
            span: self.get_node_span(estree),
        });
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

