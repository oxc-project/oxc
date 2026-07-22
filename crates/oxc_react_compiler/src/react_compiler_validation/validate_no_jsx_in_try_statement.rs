// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

//! Validates against constructing JSX within try/catch blocks.
//!
//! Developers may not be aware of error boundaries and lazy evaluation of JSX, leading them
//! to use patterns such as `let el; try { el = <Component /> } catch { ... }` to attempt to
//! catch rendering errors. Such code will fail to catch errors in rendering, but developers
//! may not realize this right away.
//!
//! This validation pass errors for JSX created within a try block. JSX is allowed within a
//! catch statement, unless that catch is itself nested inside an outer try.
//!
//! Port of ValidateNoJSXInTryStatement.ts.

use oxc_diagnostics::Diagnostics;

use crate::diagnostics::ErrorCategory;
use crate::react_compiler_hir::{BlockId, HirFunction, InstructionValue, Terminal};

pub fn validate_no_jsx_in_try_statement(func: &HirFunction) -> Diagnostics {
    let mut active_try_blocks: Vec<BlockId> = Vec::new();
    let mut error = Diagnostics::new();

    for (_block_id, block) in &func.body.blocks {
        // Remove completed try blocks (retainWhere equivalent)
        active_try_blocks.retain(|id| *id != block.id);

        if !active_try_blocks.is_empty() {
            for &instr_id in &block.instructions {
                let instr = &func.instructions[instr_id.index()];
                match &instr.value {
                    InstructionValue::JsxExpression { span, .. }
                    | InstructionValue::JsxFragment { span, .. } => {
                        error.push(
                            ErrorCategory::ErrorBoundaries
                                .diagnostic("Avoid constructing JSX within try/catch")
                                .with_help(
                                    "React does not immediately render components when JSX is rendered, so any errors from this component will not be caught by the try/catch. To catch errors in rendering a given component, wrap that component in an error boundary. (https://react.dev/reference/react/Component#catching-rendering-errors-with-an-error-boundary)",
                                )
                                .with_labels(
                                    span.map(|s| s.label("Avoid constructing JSX within try/catch")),
                                ),
                        );
                    }
                    _ => {}
                }
            }
        }

        if let Terminal::Try { handler, .. } = &block.terminal {
            active_try_blocks.push(*handler);
        }
    }

    error
}
