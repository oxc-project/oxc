/// Validate no JSX in try statements.
///
/// Port of `Validation/ValidateNoJSXInTryStatement.ts` from the React Compiler.
///
/// Validates against using try/catch to handle errors from child components.
/// React Error Boundaries should be used instead.
use crate::{
    compiler_error::{
        CompilerDiagnostic, CompilerDiagnosticDetail, CompilerError, ErrorCategory,
    },
    hir::{HIRFunction, InstructionValue, Terminal},
};

/// Validate no JSX in try statements.
pub fn validate_no_jsx_in_try_statement(func: &HIRFunction) -> CompilerError {
    let mut errors = CompilerError::new();

    // Find blocks that are inside try blocks
    let mut try_block_ids = Vec::new();
    for block in func.body.blocks.values() {
        if let Terminal::Try(try_term) = &block.terminal {
            try_block_ids.push(try_term.block);
        }
    }

    // Check for JSX inside try blocks
    for &try_block_id in &try_block_ids {
        if let Some(block) = func.body.blocks.get(&try_block_id) {
            for instr in &block.instructions {
                if matches!(
                    &instr.value,
                    InstructionValue::JsxExpression(_) | InstructionValue::JsxFragment(_)
                ) {
                    errors.push_diagnostic(
                        CompilerDiagnostic::create(
                            ErrorCategory::ErrorBoundaries,
                            "Unexpected JSX in try statement".to_string(),
                            Some(
                                "Use React Error Boundaries instead of try/catch for error handling in child components"
                                    .to_string(),
                            ),
                            None,
                        )
                        .with_detail(CompilerDiagnosticDetail::Error {
                            loc: Some(instr.value.loc()),
                            message: Some("JSX expression inside try block".to_string()),
                        }),
                    );
                }
            }
        }
    }

    errors
}
