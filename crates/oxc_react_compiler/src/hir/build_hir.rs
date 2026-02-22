use oxc_ast::ast;
use oxc_span::Span;

use crate::{
    compiler_error::{CompilerError, GENERATED_SOURCE, SourceLocation},
    hir::{
        GotoVariant, HIRFunction, Instruction, InstructionId, InstructionValue, PrimitiveValue,
        PrimitiveValueKind, ReactFunctionType, Terminal,
        environment::Environment,
        hir_builder::{HirBuilder, create_temporary_place},
    },
};

/// Lower an oxc AST function into HIR.
///
/// # Errors
/// Returns a `CompilerError` if lowering fails due to unsupported syntax.
pub fn lower(env: &Environment, fn_type: ReactFunctionType) -> Result<HIRFunction, CompilerError> {
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
        env: env.clone(),
        params: Vec::new(),
        returns: return_place,
        context: Vec::new(),
        body,
        generator: false,
        is_async: false,
        directives: Vec::new(),
        aliasing_effects: None,
    })
}

// =====================================================================================
// Statement lowering helpers
// =====================================================================================

/// Lower a block statement into the HIR builder.
///
/// # Errors
/// Returns a `CompilerError` if any statement in the block cannot be lowered.
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

/// # Errors
/// Returns a `CompilerError` if the statement cannot be lowered.
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
                effects: None,
                loc: GENERATED_SOURCE,
            });
        }
        // Empty statements and unhandled statement types are no-ops
        _ => {}
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
///
/// # Errors
/// Returns a `CompilerError` if the expression cannot be lowered.
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
                effects: None,
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
                effects: None,
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
                effects: None,
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
                effects: None,
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
                effects: None,
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
                effects: None,
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
                effects: None,
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
                effects: None,
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
                effects: None,
                loc,
            });
            Ok(ExpressionResult { place: lvalue })
        }
        LowerableExpression::CallExpression { callee, arguments, span } => {
            let loc = span_to_loc(*span);
            let callee_result = lower_expression(builder, callee)?;
            let mut args = Vec::new();
            for arg in arguments {
                let arg_result = lower_expression(builder, arg)?;
                args.push(crate::hir::CallArg::Place(arg_result.place));
            }
            let lvalue = create_temporary_place(builder.environment_mut(), loc);
            builder.push(Instruction {
                id: InstructionId(0),
                lvalue: lvalue.clone(),
                value: InstructionValue::CallExpression(crate::hir::CallExpression {
                    callee: callee_result.place,
                    args,
                    loc,
                }),
                effects: None,
                loc,
            });
            Ok(ExpressionResult { place: lvalue })
        }
        LowerableExpression::NewExpression { callee, arguments, span } => {
            let loc = span_to_loc(*span);
            let callee_result = lower_expression(builder, callee)?;
            let mut args = Vec::new();
            for arg in arguments {
                let arg_result = lower_expression(builder, arg)?;
                args.push(crate::hir::CallArg::Place(arg_result.place));
            }
            let lvalue = create_temporary_place(builder.environment_mut(), loc);
            builder.push(Instruction {
                id: InstructionId(0),
                lvalue: lvalue.clone(),
                value: InstructionValue::NewExpression(crate::hir::NewExpression {
                    callee: callee_result.place,
                    args,
                    loc,
                }),
                effects: None,
                loc,
            });
            Ok(ExpressionResult { place: lvalue })
        }
        LowerableExpression::PropertyAccess { object, property, span } => {
            let loc = span_to_loc(*span);
            let obj_result = lower_expression(builder, object)?;
            let lvalue = create_temporary_place(builder.environment_mut(), loc);
            builder.push(Instruction {
                id: InstructionId(0),
                lvalue: lvalue.clone(),
                value: InstructionValue::PropertyLoad(crate::hir::PropertyLoad {
                    object: obj_result.place,
                    property: crate::hir::types::PropertyLiteral::String(property.clone()),
                    loc,
                }),
                effects: None,
                loc,
            });
            Ok(ExpressionResult { place: lvalue })
        }
        LowerableExpression::ComputedPropertyAccess { object, property, span } => {
            let loc = span_to_loc(*span);
            let obj_result = lower_expression(builder, object)?;
            let prop_result = lower_expression(builder, property)?;
            let lvalue = create_temporary_place(builder.environment_mut(), loc);
            builder.push(Instruction {
                id: InstructionId(0),
                lvalue: lvalue.clone(),
                value: InstructionValue::ComputedLoad(crate::hir::ComputedLoad {
                    object: obj_result.place,
                    property: prop_result.place,
                    loc,
                }),
                effects: None,
                loc,
            });
            Ok(ExpressionResult { place: lvalue })
        }
        LowerableExpression::AwaitExpression { argument, span } => {
            let loc = span_to_loc(*span);
            let arg_result = lower_expression(builder, argument)?;
            let lvalue = create_temporary_place(builder.environment_mut(), loc);
            builder.push(Instruction {
                id: InstructionId(0),
                lvalue: lvalue.clone(),
                value: InstructionValue::Await(crate::hir::AwaitValue {
                    value: arg_result.place,
                    loc,
                }),
                effects: None,
                loc,
            });
            Ok(ExpressionResult { place: lvalue })
        }
        LowerableExpression::RegExpLiteral { pattern, flags, span } => {
            let loc = span_to_loc(*span);
            let lvalue = create_temporary_place(builder.environment_mut(), loc);
            builder.push(Instruction {
                id: InstructionId(0),
                lvalue: lvalue.clone(),
                value: InstructionValue::RegExpLiteral(crate::hir::RegExpLiteral {
                    pattern: pattern.clone(),
                    flags: flags.clone(),
                    loc,
                }),
                effects: None,
                loc,
            });
            Ok(ExpressionResult { place: lvalue })
        }
        LowerableExpression::TemplateLiteral { quasis, expressions, span } => {
            let loc = span_to_loc(*span);
            let mut subexprs = Vec::new();
            for expr in expressions {
                let result = lower_expression(builder, expr)?;
                subexprs.push(result.place);
            }
            let quasi_values: Vec<crate::hir::TemplateLiteralQuasi> = quasis
                .iter()
                .map(|(raw, cooked)| crate::hir::TemplateLiteralQuasi {
                    raw: raw.clone(),
                    cooked: cooked.clone(),
                })
                .collect();
            let lvalue = create_temporary_place(builder.environment_mut(), loc);
            builder.push(Instruction {
                id: InstructionId(0),
                lvalue: lvalue.clone(),
                value: InstructionValue::TemplateLiteral(crate::hir::TemplateLiteral {
                    subexprs,
                    quasis: quasi_values,
                    loc,
                }),
                effects: None,
                loc,
            });
            Ok(ExpressionResult { place: lvalue })
        }
        LowerableExpression::LoadGlobal(name, span) => {
            let loc = span_to_loc(*span);
            let lvalue = create_temporary_place(builder.environment_mut(), loc);
            builder.push(Instruction {
                id: InstructionId(0),
                lvalue: lvalue.clone(),
                value: InstructionValue::LoadGlobal(crate::hir::LoadGlobal {
                    binding: crate::hir::NonLocalBinding::Global { name: name.clone() },
                    loc,
                }),
                effects: None,
                loc,
            });
            Ok(ExpressionResult { place: lvalue })
        }
        LowerableExpression::Identifier(name, span) => {
            let loc = span_to_loc(*span);
            // For now, treat as a global load; full implementation would
            // resolve through scope chain
            let lvalue = create_temporary_place(builder.environment_mut(), loc);
            builder.push(Instruction {
                id: InstructionId(0),
                lvalue: lvalue.clone(),
                value: InstructionValue::LoadGlobal(crate::hir::LoadGlobal {
                    binding: crate::hir::NonLocalBinding::Global { name: name.clone() },
                    loc,
                }),
                effects: None,
                loc,
            });
            Ok(ExpressionResult { place: lvalue })
        }
        LowerableExpression::ConditionalExpression { test, consequent, alternate, span } => {
            // Conditional expressions are lowered to if-then-else blocks in HIR.
            // For now, lower each sub-expression and return the test result.
            // The full implementation creates branch/ternary terminals.
            let _loc = span_to_loc(*span);
            let test_result = lower_expression(builder, test)?;
            let _consequent_result = lower_expression(builder, consequent)?;
            let _alternate_result = lower_expression(builder, alternate)?;
            // Full implementation creates a TernaryTerminal here.
            Ok(test_result)
        }
        LowerableExpression::AssignmentExpression { left: _, right, span } => {
            let _loc = span_to_loc(*span);
            let right_result = lower_expression(builder, right)?;
            // Full implementation handles different LHS types
            Ok(right_result)
        }
        LowerableExpression::SpreadElement { argument, span: _ } => {
            lower_expression(builder, argument)
        }
        LowerableExpression::JsxElement { tag, span } => {
            let loc = span_to_loc(*span);
            let lvalue = create_temporary_place(builder.environment_mut(), loc);
            builder.push(Instruction {
                id: InstructionId(0),
                lvalue: lvalue.clone(),
                value: InstructionValue::JsxExpression(crate::hir::JsxExpression {
                    tag: crate::hir::JsxTag::BuiltIn(crate::hir::BuiltinTag {
                        name: tag.clone(),
                        loc,
                    }),
                    props: Vec::new(),
                    children: None,
                    loc,
                    opening_loc: loc,
                    closing_loc: loc,
                }),
                effects: None,
                loc,
            });
            Ok(ExpressionResult { place: lvalue })
        }
        LowerableExpression::FunctionExpression { span, .. }
        | LowerableExpression::ArrowFunctionExpression { span, .. } => {
            let loc = span_to_loc(*span);
            let lvalue = create_temporary_place(builder.environment_mut(), loc);
            // Function expressions require full recursive lowering of the body.
            // For now, create a placeholder UnsupportedNode.
            builder.push(Instruction {
                id: InstructionId(0),
                lvalue: lvalue.clone(),
                value: InstructionValue::UnsupportedNode(crate::hir::UnsupportedNode { loc }),
                effects: None,
                loc,
            });
            Ok(ExpressionResult { place: lvalue })
        }
    }
}

/// An expression that can be lowered to HIR.
#[derive(Debug)]
pub enum LowerableExpression {
    // Literals
    NumericLiteral(f64, Span),
    StringLiteral(String, Span),
    BooleanLiteral(bool, Span),
    NullLiteral(Span),
    Undefined(Span),
    RegExpLiteral {
        pattern: String,
        flags: String,
        span: Span,
    },
    TemplateLiteral {
        quasis: Vec<(String, Option<String>)>,
        expressions: Vec<LowerableExpression>,
        span: Span,
    },

    // Compound expressions
    ArrayExpression(Vec<LowerableExpression>, Span),
    ObjectExpression(Span),

    // Operators
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

    // Calls
    CallExpression {
        callee: Box<LowerableExpression>,
        arguments: Vec<LowerableExpression>,
        span: Span,
    },
    NewExpression {
        callee: Box<LowerableExpression>,
        arguments: Vec<LowerableExpression>,
        span: Span,
    },

    // Property access
    PropertyAccess {
        object: Box<LowerableExpression>,
        property: String,
        span: Span,
    },
    ComputedPropertyAccess {
        object: Box<LowerableExpression>,
        property: Box<LowerableExpression>,
        span: Span,
    },

    // Assignment
    AssignmentExpression {
        left: Box<LowerableExpression>,
        right: Box<LowerableExpression>,
        span: Span,
    },

    // Other
    AwaitExpression {
        argument: Box<LowerableExpression>,
        span: Span,
    },
    ConditionalExpression {
        test: Box<LowerableExpression>,
        consequent: Box<LowerableExpression>,
        alternate: Box<LowerableExpression>,
        span: Span,
    },
    SpreadElement {
        argument: Box<LowerableExpression>,
        span: Span,
    },

    // Identifiers / globals
    Identifier(String, Span),
    LoadGlobal(String, Span),

    // JSX
    JsxElement {
        tag: String,
        span: Span,
    },

    // Function expressions
    FunctionExpression {
        name: Option<String>,
        is_async: bool,
        is_generator: bool,
        span: Span,
    },
    ArrowFunctionExpression {
        is_async: bool,
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
