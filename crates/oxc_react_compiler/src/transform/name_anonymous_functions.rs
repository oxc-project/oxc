/// Name anonymous functions based on their usage context.
///
/// Port of `Transform/NameAnonymousFunctions.ts` from the React Compiler.
///
/// Assigns descriptive names to anonymous function expressions based on how
/// they are used (e.g., assigned to a variable, passed as a callback, used as
/// a JSX prop).
use rustc_hash::FxHashMap;

use crate::hir::{
    HIRFunction, IdentifierId, IdentifierName, InstructionValue, JsxAttribute, JsxTag,
};

/// Name anonymous functions in the given HIR function.
pub fn name_anonymous_functions(func: &mut HIRFunction) {
    let parent_name = match &func.id {
        Some(name) => name.clone(),
        None => return,
    };

    let functions = collect_anonymous_functions(func);
    for node in &functions {
        visit_node(node, &format!("{parent_name}["));
    }
}

struct FunctionNode {
    generated_name: Option<String>,
    original_name: Option<String>,
    inner: Vec<FunctionNode>,
}

fn visit_node(node: &FunctionNode, prefix: &str) {
    if let Some(ref gen_name) = node.generated_name {
        let _name = format!("{prefix}{gen_name}]");
        // In the full implementation, we'd set the nameHint on the function
    }
    let next_prefix = format!(
        "{prefix}{} > ",
        node.generated_name
            .as_deref()
            .or(node.original_name.as_deref())
            .unwrap_or("<anonymous>")
    );
    for inner in &node.inner {
        visit_node(inner, &next_prefix);
    }
}

fn collect_anonymous_functions(func: &HIRFunction) -> Vec<FunctionNode> {
    let mut functions: FxHashMap<IdentifierId, usize> = FxHashMap::default();
    let mut names: FxHashMap<IdentifierId, String> = FxHashMap::default();
    let mut nodes: Vec<FunctionNode> = Vec::new();

    for block in func.body.blocks.values() {
        for instr in &block.instructions {
            match &instr.value {
                InstructionValue::LoadGlobal(v) => {
                    let name = match &v.binding {
                        crate::hir::NonLocalBinding::Global { name } => name.clone(),
                        crate::hir::NonLocalBinding::ModuleLocal { name } => name.clone(),
                        crate::hir::NonLocalBinding::ImportDefault { name, .. } => name.clone(),
                        crate::hir::NonLocalBinding::ImportNamespace { name, .. } => name.clone(),
                        crate::hir::NonLocalBinding::ImportSpecifier { name, .. } => name.clone(),
                    };
                    names.insert(instr.lvalue.identifier.id, name);
                }
                InstructionValue::LoadLocal(v) => {
                    if let Some(IdentifierName::Named(name)) = &v.place.identifier.name {
                        names.insert(instr.lvalue.identifier.id, name.clone());
                    }
                }
                InstructionValue::LoadContext(v) => {
                    if let Some(IdentifierName::Named(name)) = &v.place.identifier.name {
                        names.insert(instr.lvalue.identifier.id, name.clone());
                    }
                }
                InstructionValue::PropertyLoad(v) => {
                    if let Some(obj_name) = names.get(&v.object.identifier.id) {
                        let prop = v.property.to_string();
                        names.insert(instr.lvalue.identifier.id, format!("{obj_name}.{prop}"));
                    }
                }
                InstructionValue::FunctionExpression(v) => {
                    let inner = collect_anonymous_functions(&v.lowered_func.func);
                    let node = FunctionNode {
                        generated_name: None,
                        original_name: v.name.clone(),
                        inner,
                    };
                    let idx = nodes.len();
                    nodes.push(node);
                    if v.name.is_none() {
                        functions.insert(instr.lvalue.identifier.id, idx);
                    }
                }
                InstructionValue::StoreLocal(v) => {
                    if let Some(&idx) = functions.get(&v.value.identifier.id)
                        && let Some(IdentifierName::Named(name)) = &v.lvalue.place.identifier.name
                            && nodes[idx].generated_name.is_none() {
                                nodes[idx].generated_name = Some(name.clone());
                                functions.remove(&v.value.identifier.id);
                            }
                }
                InstructionValue::StoreContext(v) => {
                    if let Some(&idx) = functions.get(&v.value.identifier.id)
                        && let Some(IdentifierName::Named(name)) = &v.lvalue_place.identifier.name
                            && nodes[idx].generated_name.is_none() {
                                nodes[idx].generated_name = Some(name.clone());
                                functions.remove(&v.value.identifier.id);
                            }
                }
                InstructionValue::CallExpression(v) => {
                    let callee_name = names.get(&v.callee.identifier.id).cloned();
                    if let Some(callee_name) = callee_name {
                        for arg in &v.args {
                            if let crate::hir::CallArg::Place(place) = arg
                                && let Some(&idx) = functions.get(&place.identifier.id)
                                    && nodes[idx].generated_name.is_none() {
                                        nodes[idx].generated_name =
                                            Some(format!("{callee_name}()"));
                                        functions.remove(&place.identifier.id);
                                    }
                        }
                    }
                }
                InstructionValue::JsxExpression(v) => {
                    let element_name = match &v.tag {
                        JsxTag::BuiltIn(tag) => Some(tag.name.clone()),
                        JsxTag::Place(p) => names.get(&p.identifier.id).cloned(),
                    };
                    for attr in &v.props {
                        if let JsxAttribute::Attribute { name: attr_name, place } = attr
                            && let Some(&idx) = functions.get(&place.identifier.id)
                                && nodes[idx].generated_name.is_none() {
                                    let prop_name = match &element_name {
                                        Some(el) => format!("<{el}>.{attr_name}"),
                                        None => attr_name.clone(),
                                    };
                                    nodes[idx].generated_name = Some(prop_name);
                                    functions.remove(&place.identifier.id);
                                }
                    }
                }
                _ => {}
            }
        }
    }

    nodes
}
