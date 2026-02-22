/// Validate no JSX in try statements.
///
/// Port of `Validation/ValidateNoJSXInTryStatement.ts` from the React Compiler.
///
/// Validates against using try/catch to handle errors from child components.
/// React Error Boundaries should be used instead.
///
/// Developers may not be aware of error boundaries and lazy evaluation of JSX,
/// leading them to use patterns such as `let el; try { el = <Component /> } catch { ... }`
/// to attempt to catch rendering errors. Such code will fail to catch errors in rendering.
use crate::{
    compiler_error::{CompilerDiagnostic, CompilerDiagnosticDetail, CompilerError, ErrorCategory},
    hir::{BlockId, HIRFunction, InstructionValue, Terminal},
};

/// Validate no JSX in try statements.
///
/// Tracks active try blocks and checks all blocks that are reachable from a try
/// terminal's `block` until the corresponding `handler` block is reached.
pub fn validate_no_jsx_in_try_statement(func: &HIRFunction) -> CompilerError {
    let mut errors = CompilerError::new();
    let mut active_try_blocks: Vec<BlockId> = Vec::new();

    for block in func.body.blocks.values() {
        // Remove any active try block whose handler is the current block.
        // This means we've reached the catch handler, so we're no longer
        // "inside" that try block.
        active_try_blocks.retain(|id| *id != block.id);

        if !active_try_blocks.is_empty() {
            for instr in &block.instructions {
                match &instr.value {
                    InstructionValue::JsxExpression(_) | InstructionValue::JsxFragment(_) => {
                        errors.push_diagnostic(
                            CompilerDiagnostic::create(
                                ErrorCategory::ErrorBoundaries,
                                "Avoid constructing JSX within try/catch".to_string(),
                                Some(
                                    "React does not immediately render components when JSX is \
                                     rendered, so any errors from this component will not be \
                                     caught by the try/catch. To catch errors in rendering a \
                                     given component, wrap that component in an error boundary. \
                                     (https://react.dev/reference/react/Component\
                                     #catching-rendering-errors-with-an-error-boundary)"
                                        .to_string(),
                                ),
                                None,
                            )
                            .with_detail(
                                CompilerDiagnosticDetail::Error {
                                    loc: Some(instr.value.loc()),
                                    message: Some(
                                        "Avoid constructing JSX within try/catch".to_string(),
                                    ),
                                },
                            ),
                        );
                    }
                    _ => {}
                }
            }
        }

        if let Terminal::Try(try_term) = &block.terminal {
            active_try_blocks.push(try_term.handler);
        }
    }

    errors
}
