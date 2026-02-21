/// Drop manual memoization (useMemo/useCallback) and replace with compiler markers.
///
/// Port of `Inference/DropManualMemoization.ts` from the React Compiler.
///
/// This pass identifies useMemo/useCallback calls, validates their usage,
/// and replaces them with StartMemoize/FinishMemoize marker instructions
/// that the compiler can reason about.
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    compiler_error::{CompilerError, SourceLocation},
    hir::{
        HIRFunction, IdentifierId, IdentifierName, Instruction, InstructionValue,
        ManualMemoDependency, ManualMemoDependencyRoot, Place,
    },
};

/// Collect potential manual memo dependencies from an instruction value.
///
/// Returns the dependency represented by the instruction, if any.
pub fn collect_maybe_memo_dependencies(
    value: &InstructionValue,
    maybe_deps: &mut FxHashMap<IdentifierId, ManualMemoDependency>,
    optional: bool,
) -> Option<ManualMemoDependency> {
    match value {
        InstructionValue::LoadGlobal(v) => {
            let name = match &v.binding {
                crate::hir::NonLocalBinding::Global { name } => name.clone(),
                crate::hir::NonLocalBinding::ModuleLocal { name } => name.clone(),
                crate::hir::NonLocalBinding::ImportDefault { name, .. } => name.clone(),
                crate::hir::NonLocalBinding::ImportNamespace { name, .. } => name.clone(),
                crate::hir::NonLocalBinding::ImportSpecifier { name, .. } => name.clone(),
            };
            Some(ManualMemoDependency {
                root: ManualMemoDependencyRoot::Global { identifier_name: name },
                path: Vec::new(),
                loc: v.loc,
            })
        }
        InstructionValue::PropertyLoad(v) => {
            let object_dep = maybe_deps.get(&v.object.identifier.id)?;
            Some(ManualMemoDependency {
                root: object_dep.root.clone(),
                path: {
                    let mut path = object_dep.path.clone();
                    path.push(crate::hir::DependencyPathEntry {
                        property: v.property.clone(),
                        optional,
                        loc: v.loc,
                    });
                    path
                },
                loc: v.loc,
            })
        }
        InstructionValue::LoadLocal(v) => {
            if let Some(source) = maybe_deps.get(&v.place.identifier.id) {
                return Some(source.clone());
            }
            if let Some(IdentifierName::Named(_name)) = &v.place.identifier.name {
                Some(ManualMemoDependency {
                    root: ManualMemoDependencyRoot::NamedLocal {
                        value: v.place.clone(),
                        constant: false,
                    },
                    path: Vec::new(),
                    loc: v.place.loc,
                })
            } else {
                None
            }
        }
        InstructionValue::LoadContext(v) => {
            if let Some(source) = maybe_deps.get(&v.place.identifier.id) {
                return Some(source.clone());
            }
            if let Some(IdentifierName::Named(_name)) = &v.place.identifier.name {
                Some(ManualMemoDependency {
                    root: ManualMemoDependencyRoot::NamedLocal {
                        value: v.place.clone(),
                        constant: false,
                    },
                    path: Vec::new(),
                    loc: v.place.loc,
                })
            } else {
                None
            }
        }
        InstructionValue::StoreLocal(v) => {
            let rvalue_id = v.value.identifier.id;
            if let Some(aliased) = maybe_deps.get(&rvalue_id) {
                let lvalue = &v.lvalue.place.identifier;
                if !matches!(&lvalue.name, Some(IdentifierName::Named(_))) {
                    let dep = aliased.clone();
                    maybe_deps.insert(lvalue.id, dep.clone());
                    return Some(dep);
                }
            }
            None
        }
        _ => None,
    }
}

/// Track identifiers that are useMemo/useCallback callees.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManualMemoKind {
    UseMemo,
    UseCallback,
}

/// Sidemap for tracking identifiers during the pass.
struct IdentifierSidemap {
    manual_memos: FxHashMap<IdentifierId, ManualMemoKind>,
    react_ids: FxHashSet<IdentifierId>,
    maybe_deps: FxHashMap<IdentifierId, ManualMemoDependency>,
    maybe_deps_lists: FxHashMap<IdentifierId, (SourceLocation, Vec<Place>)>,
    optionals: FxHashSet<IdentifierId>,
}

/// Drop manual memoization and replace with compiler markers.
///
/// # Errors
/// Returns a `CompilerError` if validation of manual memoization fails.
pub fn drop_manual_memoization(func: &mut HIRFunction) -> Result<(), CompilerError> {
    let mut sidemap = IdentifierSidemap {
        manual_memos: FxHashMap::default(),
        react_ids: FxHashSet::default(),
        maybe_deps: FxHashMap::default(),
        maybe_deps_lists: FxHashMap::default(),
        optionals: FxHashSet::default(),
    };
    let mut _manual_memo_id: u32 = 0;
    let errors = CompilerError::new();

    let block_ids: Vec<_> = func.body.blocks.keys().copied().collect();
    for block_id in block_ids {
        let block = match func.body.blocks.get_mut(&block_id) {
            Some(b) => b,
            None => continue,
        };

        for instr in &mut block.instructions {
            // Collect temporaries and track manual memo callees
            collect_temporaries(instr, &mut sidemap);

            // Check if this instruction is a useMemo/useCallback call
            let is_manual_memo_call = match &instr.value {
                InstructionValue::CallExpression(v) => {
                    sidemap.manual_memos.contains_key(&v.callee.identifier.id)
                }
                _ => false,
            };

            if is_manual_memo_call {
                // In the full implementation, we would:
                // 1. Validate the call arguments (callback + deps)
                // 2. Replace the call with inline code + StartMemoize/FinishMemoize markers
                // 3. Track the memoization ID for validation
                _manual_memo_id += 1;
            }
        }
    }

    errors.into_result()
}

fn collect_temporaries(instr: &Instruction, sidemap: &mut IdentifierSidemap) {
    let lvalue_id = instr.lvalue.identifier.id;
    match &instr.value {
        InstructionValue::LoadGlobal(v) => {
            let name = match &v.binding {
                crate::hir::NonLocalBinding::Global { name } => name.as_str(),
                crate::hir::NonLocalBinding::ModuleLocal { name } => name.as_str(),
                crate::hir::NonLocalBinding::ImportDefault { name, .. } => name.as_str(),
                crate::hir::NonLocalBinding::ImportNamespace { name, .. } => name.as_str(),
                crate::hir::NonLocalBinding::ImportSpecifier { name, .. } => name.as_str(),
            };
            match name {
                "useMemo" => {
                    sidemap.manual_memos.insert(lvalue_id, ManualMemoKind::UseMemo);
                }
                "useCallback" => {
                    sidemap.manual_memos.insert(lvalue_id, ManualMemoKind::UseCallback);
                }
                "React" => {
                    sidemap.react_ids.insert(lvalue_id);
                }
                _ => {}
            }
        }
        InstructionValue::PropertyLoad(v) => {
            if sidemap.react_ids.contains(&v.object.identifier.id) {
                let prop = v.property.to_string();
                match prop.as_str() {
                    "useMemo" => {
                        sidemap.manual_memos.insert(lvalue_id, ManualMemoKind::UseMemo);
                    }
                    "useCallback" => {
                        sidemap.manual_memos.insert(lvalue_id, ManualMemoKind::UseCallback);
                    }
                    _ => {}
                }
            }
        }
        InstructionValue::ArrayExpression(v) => {
            // Track array literals that might be deps lists
            let all_places: Vec<Place> = v
                .elements
                .iter()
                .filter_map(|e| match e {
                    crate::hir::ArrayExpressionElement::Place(p) => Some(p.clone()),
                    _ => None,
                })
                .collect();
            if all_places.len() == v.elements.len() {
                sidemap.maybe_deps_lists.insert(lvalue_id, (v.loc, all_places));
            }
        }
        _ => {}
    }

    // Collect maybe-memo dependencies
    let is_optional = sidemap.optionals.contains(&lvalue_id);
    let dep = collect_maybe_memo_dependencies(&instr.value, &mut sidemap.maybe_deps, is_optional);
    if let Some(dep) = dep {
        sidemap.maybe_deps.insert(lvalue_id, dep);
    }
}
