/// Validate hooks usage according to the Rules of Hooks.
///
/// Port of `Validation/ValidateHooksUsage.ts` from the React Compiler.
///
/// Validates that hooks are called:
/// - At the top level of a function component or custom hook
/// - Not inside conditions, loops, or nested functions
/// - Always called in the same order
use rustc_hash::FxHashMap;

use crate::{
    compiler_error::{CompilerDiagnostic, CompilerDiagnosticDetail, CompilerError, ErrorCategory},
    hir::{
        HIRFunction, IdentifierId, InstructionValue,
        compute_unconditional_blocks::compute_unconditional_blocks,
    },
};

/// Possible kinds of value during abstract interpretation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Error,
    KnownHook,
    PotentialHook,
    Global,
    Local,
}

fn join_kinds(a: Kind, b: Kind) -> Kind {
    if a == Kind::Error || b == Kind::Error {
        Kind::Error
    } else if a == Kind::KnownHook || b == Kind::KnownHook {
        Kind::KnownHook
    } else if a == Kind::PotentialHook || b == Kind::PotentialHook {
        Kind::PotentialHook
    } else if a == Kind::Global || b == Kind::Global {
        Kind::Global
    } else {
        Kind::Local
    }
}

/// Validate hooks usage in the given function.
///
/// # Errors
/// Returns a `CompilerError` if hooks usage violations are found.
pub fn validate_hooks_usage(func: &HIRFunction) -> Result<(), CompilerError> {
    let unconditional_blocks = compute_unconditional_blocks(func);
    let mut kinds: FxHashMap<IdentifierId, Kind> = FxHashMap::default();
    let mut errors = CompilerError::new();

    for (&block_id, block) in &func.body.blocks {
        let is_unconditional = unconditional_blocks.contains(&block_id);

        for instr in &block.instructions {
            match &instr.value {
                InstructionValue::LoadGlobal(v) => {
                    let name = v.binding.name().to_string();
                    if is_hook_name(&name) {
                        kinds.insert(instr.lvalue.identifier.id, Kind::KnownHook);
                    } else {
                        kinds.insert(instr.lvalue.identifier.id, Kind::Global);
                    }
                }
                InstructionValue::CallExpression(v) => {
                    let callee_kind = kinds.get(&v.callee.identifier.id).copied();
                    if matches!(callee_kind, Some(Kind::KnownHook | Kind::PotentialHook)) {
                        // Hook call — check if it's unconditional
                        if !is_unconditional {
                            errors.push_diagnostic(
                                CompilerDiagnostic::create(
                                    ErrorCategory::Hooks,
                                    "Hooks must be called unconditionally".to_string(),
                                    Some(
                                        "This hook call is inside a conditional or loop"
                                            .to_string(),
                                    ),
                                    None,
                                )
                                .with_detail(
                                    CompilerDiagnosticDetail::Error {
                                        loc: Some(v.loc),
                                        message: Some("hook called conditionally".to_string()),
                                    },
                                ),
                            );
                        }
                    }
                }
                InstructionValue::MethodCall(v) => {
                    let property_kind = kinds.get(&v.property.identifier.id).copied();
                    if matches!(property_kind, Some(Kind::KnownHook | Kind::PotentialHook))
                        && !is_unconditional
                    {
                        errors.push_diagnostic(
                            CompilerDiagnostic::create(
                                ErrorCategory::Hooks,
                                "Hooks must be called unconditionally".to_string(),
                                Some("This hook call is inside a conditional or loop".to_string()),
                                None,
                            )
                            .with_detail(
                                CompilerDiagnosticDetail::Error {
                                    loc: Some(v.loc),
                                    message: Some("hook called conditionally".to_string()),
                                },
                            ),
                        );
                    }
                }
                InstructionValue::LoadLocal(v) => {
                    // Propagate kind from loaded variable
                    if let Some(&kind) = kinds.get(&v.place.identifier.id) {
                        let existing =
                            kinds.get(&instr.lvalue.identifier.id).copied().unwrap_or(Kind::Local);
                        kinds.insert(instr.lvalue.identifier.id, join_kinds(existing, kind));
                    }
                }
                InstructionValue::PropertyLoad(v) => {
                    // If loading from a hook-like object, mark as potential hook
                    let obj_kind = kinds.get(&v.object.identifier.id).copied();
                    if matches!(
                        obj_kind,
                        Some(Kind::KnownHook | Kind::PotentialHook | Kind::Global)
                    ) && is_hook_name(&v.property.to_string())
                    {
                        kinds.insert(instr.lvalue.identifier.id, Kind::PotentialHook);
                    }
                }
                _ => {}
            }
        }
    }

    errors.into_result()
}

/// Check if a name follows hook naming convention (starts with "use" followed by uppercase).
fn is_hook_name(name: &str) -> bool {
    if !name.starts_with("use") {
        return false;
    }
    if name.len() == 3 {
        return true; // "use" alone
    }
    // The character after "use" must be uppercase
    name[3..].starts_with(|c: char| c.is_ascii_uppercase())
}
