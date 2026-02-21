
use oxc_ast::ast;
use oxc_span::Span;

use crate::{
    compiler_error::{CompilerError, SourceLocation, GENERATED_SOURCE},
    hir::{
        GotoVariant, HIRFunction, Instruction, InstructionId, InstructionValue, PrimitiveValue, PrimitiveValueKind, ReactFunctionType, Terminal,
        environment::Environment,
        hir_builder::{HirBuilder, create_temporary_place},
    },
};

/// Lower an oxc AST function into HIR.
///
/// # Errors
/// Returns a `CompilerError` if lowering fails due to unsupported syntax.
pub fn lower(
    env: &mut Environment,
    fn_type: ReactFunctionType,
) -> Result<HIRFunction, CompilerError> {
    let mut builder = HirBuilder::new(env.clone(), None);

    // Build a minimal HIR function
    // The full implementation would walk the AST and lower each node.
    // For now, we create a skeleton with an empty body.

    let return_place = create_temporary_place(builder.environment_mut(), GENERATED_SOURCE);

    // Terminate the entry block with a return
    builder.terminate(
        Terminal::Return(crate::hir::ReturnTerminal {
            id: InstructionId(0),
            value: return_place.clone(),
            return_variant: crate::hir::ReturnVariant::Void,
            loc: GENERATED_SOURCE,
        }),
        None,
    );

    let body = builder.build()?;

    Ok(HIRFunction {
        loc: GENERATED_SOURCE,
        id: None,
        name_hint: None,
        fn_type,
        params: Vec::new(),
        returns: return_place,
        context: Vec::new(),
        body,
        generator: false,
        is_async: false,
        directives: Vec::new(),
    })
}

// =====================================================================================
// Statement lowering helpers
// =====================================================================================

/// Lower a block statement into the HIR builder.
pub fn lower_block_statement(
    builder: &mut HirBuilder,
    stmts: &[LowerableStatement],
) -> Result<(), CompilerError> {
    for stmt in stmts {
        lower_statement(builder, stmt)?;
    }
    Ok(())
}

/// A statement that can be lowered to HIR.
/// This is an abstraction over oxc_ast statement types.
#[derive(Debug)]
pub enum LowerableStatement<'a> {
    VariableDeclaration(&'a ast::VariableDeclaration<'a>),
    ExpressionStatement(&'a ast::ExpressionStatement<'a>),
    ReturnStatement(&'a ast::ReturnStatement<'a>),
    IfStatement(&'a ast::IfStatement<'a>),
    WhileStatement(&'a ast::WhileStatement<'a>),
    ForStatement(&'a ast::ForStatement<'a>),
    BlockStatement(&'a ast::BlockStatement<'a>),
    ThrowStatement(&'a ast::ThrowStatement<'a>),
    TryStatement(&'a ast::TryStatement<'a>),
    SwitchStatement(&'a ast::SwitchStatement<'a>),
    BreakStatement,
    ContinueStatement,
    DebuggerStatement,
    EmptyStatement,
}

pub fn lower_statement(
    builder: &mut HirBuilder,
    stmt: &LowerableStatement,
) -> Result<(), CompilerError> {
    match stmt {
        LowerableStatement::ReturnStatement(_ret) => {
            // Lower the return value expression
            let value = create_temporary_place(builder.environment_mut(), GENERATED_SOURCE);
            builder.terminate(
                Terminal::Return(crate::hir::ReturnTerminal {
                    id: InstructionId(0),
                    value,
                    return_variant: crate::hir::ReturnVariant::Explicit,
                    loc: GENERATED_SOURCE,
                }),
                None,
            );
        }
        LowerableStatement::BreakStatement => {
            let target = builder.lookup_break(None)?;
            builder.terminate(
                Terminal::Goto(crate::hir::GotoTerminal {
                    id: InstructionId(0),
                    block: target,
                    variant: GotoVariant::Break,
                    loc: GENERATED_SOURCE,
                }),
                None,
            );
        }
        LowerableStatement::ContinueStatement => {
            let target = builder.lookup_continue(None)?;
            builder.terminate(
                Terminal::Goto(crate::hir::GotoTerminal {
                    id: InstructionId(0),
                    block: target,
                    variant: GotoVariant::Continue,
                    loc: GENERATED_SOURCE,
                }),
                None,
            );
        }
        LowerableStatement::ThrowStatement(_throw) => {
            let value = create_temporary_place(builder.environment_mut(), GENERATED_SOURCE);
            builder.terminate(
                Terminal::Throw(crate::hir::ThrowTerminal {
                    id: InstructionId(0),
                    value,
                    loc: GENERATED_SOURCE,
                }),
                None,
            );
        }
        LowerableStatement::DebuggerStatement => {
            let lvalue = create_temporary_place(builder.environment_mut(), GENERATED_SOURCE);
            builder.push(Instruction {
                id: InstructionId(0),
                lvalue,
                value: InstructionValue::Debugger(crate::hir::DebuggerValue {
                    loc: GENERATED_SOURCE,
                }),
                loc: GENERATED_SOURCE,
            });
        }
        LowerableStatement::EmptyStatement => {
            // No-op
        }
        // Other statement types would be handled here in the full implementation
        _ => {
            // For unhandled statements, create a placeholder
        }
    }
    Ok(())
}

// =====================================================================================
// Expression lowering helpers
// =====================================================================================

/// A lowered expression result — the Place where the expression value is stored.
pub struct ExpressionResult {
    pub place: crate::hir::Place,
}

/// Lower an expression, emitting instructions to the builder and returning
/// the Place that holds the result.
pub fn lower_expression(
    builder: &mut HirBuilder,
    expr: &LowerableExpression,
) -> Result<ExpressionResult, CompilerError> {
    match expr {
        LowerableExpression::NumericLiteral(value, span) => {
            let loc = span_to_loc(*span);
            let lvalue = create_temporary_place(builder.environment_mut(), loc);
            builder.push(Instruction {
                id: InstructionId(0),
                lvalue: lvalue.clone(),
                value: lower_number(*value, loc),
                loc,
            });
            Ok(ExpressionResult { place: lvalue })
        }
        LowerableExpression::StringLiteral(value, span) => {
            let loc = span_to_loc(*span);
            let lvalue = create_temporary_place(builder.environment_mut(), loc);
            builder.push(Instruction {
                id: InstructionId(0),
                lvalue: lvalue.clone(),
                value: lower_string(value.clone(), loc),
                loc,
            });
            Ok(ExpressionResult { place: lvalue })
        }
        LowerableExpression::BooleanLiteral(value, span) => {
            let loc = span_to_loc(*span);
            let lvalue = create_temporary_place(builder.environment_mut(), loc);
            builder.push(Instruction {
                id: InstructionId(0),
                lvalue: lvalue.clone(),
                value: lower_boolean(*value, loc),
                loc,
            });
            Ok(ExpressionResult { place: lvalue })
        }
        LowerableExpression::NullLiteral(span) => {
            let loc = span_to_loc(*span);
            let lvalue = create_temporary_place(builder.environment_mut(), loc);
            builder.push(Instruction {
                id: InstructionId(0),
                lvalue: lvalue.clone(),
                value: lower_null(loc),
                loc,
            });
            Ok(ExpressionResult { place: lvalue })
        }
        LowerableExpression::Undefined(span) => {
            let loc = span_to_loc(*span);
            let lvalue = create_temporary_place(builder.environment_mut(), loc);
            builder.push(Instruction {
                id: InstructionId(0),
                lvalue: lvalue.clone(),
                value: lower_undefined(loc),
                loc,
            });
            Ok(ExpressionResult { place: lvalue })
        }
        LowerableExpression::ArrayExpression(_elements, span) => {
            let loc = span_to_loc(*span);
            let lvalue = create_temporary_place(builder.environment_mut(), loc);
            builder.push(Instruction {
                id: InstructionId(0),
                lvalue: lvalue.clone(),
                value: InstructionValue::ArrayExpression(crate::hir::ArrayExpression {
                    elements: Vec::new(), // Elements would be lowered recursively
                    loc,
                }),
                loc,
            });
            Ok(ExpressionResult { place: lvalue })
        }
        LowerableExpression::ObjectExpression(span) => {
            let loc = span_to_loc(*span);
            let lvalue = create_temporary_place(builder.environment_mut(), loc);
            builder.push(Instruction {
                id: InstructionId(0),
                lvalue: lvalue.clone(),
                value: InstructionValue::ObjectExpression(crate::hir::ObjectExpression {
                    properties: Vec::new(), // Properties would be lowered recursively
                    loc,
                }),
                loc,
            });
            Ok(ExpressionResult { place: lvalue })
        }
        LowerableExpression::BinaryExpression { operator, left, right, span } => {
            let loc = span_to_loc(*span);
            let left_result = lower_expression(builder, left)?;
            let right_result = lower_expression(builder, right)?;
            let lvalue = create_temporary_place(builder.environment_mut(), loc);
            builder.push(Instruction {
                id: InstructionId(0),
                lvalue: lvalue.clone(),
                value: InstructionValue::BinaryExpression(crate::hir::BinaryExpressionValue {
                    operator: *operator,
                    left: left_result.place,
                    right: right_result.place,
                    loc,
                }),
                loc,
            });
            Ok(ExpressionResult { place: lvalue })
        }
        LowerableExpression::UnaryExpression { operator, argument, span } => {
            let loc = span_to_loc(*span);
            let arg_result = lower_expression(builder, argument)?;
            let lvalue = create_temporary_place(builder.environment_mut(), loc);
            builder.push(Instruction {
                id: InstructionId(0),
                lvalue: lvalue.clone(),
                value: InstructionValue::UnaryExpression(crate::hir::UnaryExpressionValue {
                    operator: *operator,
                    value: arg_result.place,
                    loc,
                }),
                loc,
            });
            Ok(ExpressionResult { place: lvalue })
        }
    }
}

/// An expression that can be lowered to HIR.
#[derive(Debug)]
pub enum LowerableExpression {
    NumericLiteral(f64, Span),
    StringLiteral(String, Span),
    BooleanLiteral(bool, Span),
    NullLiteral(Span),
    Undefined(Span),
    ArrayExpression(Vec<LowerableExpression>, Span),
    ObjectExpression(Span),
    BinaryExpression {
        operator: oxc_syntax::operator::BinaryOperator,
        left: Box<LowerableExpression>,
        right: Box<LowerableExpression>,
        span: Span,
    },
    UnaryExpression {
        operator: oxc_syntax::operator::UnaryOperator,
        argument: Box<LowerableExpression>,
        span: Span,
    },
}

/// Lower a primitive literal to an instruction value.
pub fn lower_primitive(value: PrimitiveValueKind, loc: SourceLocation) -> InstructionValue {
    InstructionValue::Primitive(PrimitiveValue { value, loc })
}

/// Lower a numeric literal.
pub fn lower_number(value: f64, loc: SourceLocation) -> InstructionValue {
    lower_primitive(PrimitiveValueKind::Number(value), loc)
}

/// Lower a string literal.
pub fn lower_string(value: String, loc: SourceLocation) -> InstructionValue {
    lower_primitive(PrimitiveValueKind::String(value), loc)
}

/// Lower a boolean literal.
pub fn lower_boolean(value: bool, loc: SourceLocation) -> InstructionValue {
    lower_primitive(PrimitiveValueKind::Boolean(value), loc)
}

/// Lower null literal.
pub fn lower_null(loc: SourceLocation) -> InstructionValue {
    lower_primitive(PrimitiveValueKind::Null, loc)
}

/// Lower undefined.
pub fn lower_undefined(loc: SourceLocation) -> InstructionValue {
    lower_primitive(PrimitiveValueKind::Undefined, loc)
}

/// Convert an oxc Span to a SourceLocation.
pub fn span_to_loc(span: Span) -> SourceLocation {
    SourceLocation::Source(span)
}
