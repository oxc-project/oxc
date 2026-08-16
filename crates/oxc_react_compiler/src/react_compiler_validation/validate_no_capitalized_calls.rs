use cow_utils::CowUtils;
use rustc_hash::{FxHashMap, FxHashSet};

use oxc_diagnostics::OxcDiagnostic;
use oxc_str::Ident;

use crate::diagnostics;
use crate::react_compiler_hir::environment::Environment;
use crate::react_compiler_hir::{HirFunction, IdentifierId, InstructionValue, PropertyLiteral};

/// Validates that capitalized functions are not called directly (they should be rendered as JSX).
///
/// Port of ValidateNoCapitalizedCalls.ts.
pub fn validate_no_capitalized_calls(
    func: &HirFunction,
    env: &mut Environment,
) -> Result<(), OxcDiagnostic> {
    // Build the allow list from global registry keys + config entries
    let mut allow_list: FxHashSet<String> = env.globals().keys().map(str::to_owned).collect();
    if let Some(config_entries) = &env.config.validate_no_capitalized_calls {
        for entry in config_entries {
            allow_list.insert(entry.clone());
        }
    }

    let mut capital_load_globals: FxHashMap<IdentifierId, Ident> = FxHashMap::default();
    let mut capitalized_properties: FxHashMap<IdentifierId, Ident> = FxHashMap::default();

    for (_block_id, block) in &func.body.blocks {
        for &instr_id in &block.instructions {
            let instr = &func.instructions[instr_id.index()];
            let lvalue_id = instr.lvalue.identifier;
            let value = &instr.value;

            match value {
                InstructionValue::LoadGlobal { binding, .. } => {
                    let name = binding.name();
                    if !name.is_empty()
                        && name.starts_with(|c: char| c.is_ascii_uppercase())
                        // We don't want to flag CONSTANTS()
                        && name.as_str() != name.cow_to_uppercase()
                        && !allow_list.contains(name.as_str())
                    {
                        capital_load_globals.insert(lvalue_id, name);
                    }
                }
                InstructionValue::CallExpression { callee, span, .. } => {
                    let callee_id = callee.identifier;
                    if let Some(callee_name) = capital_load_globals.get(&callee_id) {
                        env.record_error(diagnostics::capitalized_call(callee_name, *span))?;
                        continue;
                    }
                }
                InstructionValue::PropertyLoad {
                    property: PropertyLiteral::String(prop_name),
                    ..
                } => {
                    if prop_name.starts_with(|c: char| c.is_ascii_uppercase()) {
                        capitalized_properties.insert(lvalue_id, *prop_name);
                    }
                }
                InstructionValue::MethodCall { property, span, .. } => {
                    let property_id = property.identifier;
                    if let Some(prop_name) = capitalized_properties.get(&property_id) {
                        env.record_error(diagnostics::capitalized_call(prop_name, *span))?;
                    }
                }
                _ => {}
            }
        }
    }
    Ok(())
}
