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
use oxc_index::IndexSlice;
use smallvec::SmallVec;

use crate::diagnostics;
use crate::react_compiler_hir::{BlockId, FunctionId, HirFunction, InstructionValue, Terminal};

pub fn validate_no_jsx_in_try_statement<'a>(
    func: &HirFunction<'a>,
    functions: &IndexSlice<FunctionId, [HirFunction<'a>]>,
) -> Diagnostics {
    let mut error = Diagnostics::new();
    let has_memo_callbacks = functions.iter().any(|func| func.is_memo_callback);
    let mut worklist: SmallVec<[_; 1]> = smallvec::smallvec![(func, false)];

    while let Some((func, in_outer_try)) = worklist.pop() {
        let mut active_try_blocks: Vec<BlockId> = Vec::new();

        for (_block_id, block) in &func.body.blocks {
            // Remove completed try blocks (retainWhere equivalent)
            active_try_blocks.retain(|id| *id != block.id);
            let in_try = in_outer_try || !active_try_blocks.is_empty();

            if in_try || has_memo_callbacks {
                for &instr_id in &block.instructions {
                    let instr = &func.instructions[instr_id.index()];
                    match &instr.value {
                        InstructionValue::FunctionExpression { lowered_func, .. } => {
                            let callback = &functions[lowered_func.func];
                            // Unlike the TS implementation, some inline useMemo callbacks
                            // retain their scope. Their bodies still execute during render,
                            // including within an enclosing try block. Returned handlers do not.
                            if callback.is_memo_callback {
                                worklist.push((callback, in_try));
                            }
                        }
                        InstructionValue::JsxExpression { opening_name_span, span, .. }
                            if in_try =>
                        {
                            error.push(diagnostics::jsx_in_try(opening_name_span.or(*span)));
                        }
                        InstructionValue::JsxFragment { opening_span, span, .. } if in_try => {
                            error.push(diagnostics::jsx_in_try(opening_span.or(*span)));
                        }
                        _ => {}
                    }
                }
            }

            if let Terminal::Try { handler, .. } = &block.terminal {
                active_try_blocks.push(*handler);
            }
        }
    }

    error
}
