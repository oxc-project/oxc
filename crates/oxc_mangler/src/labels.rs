use oxc_ast::AstKind;
use oxc_semantic::Semantic;
use oxc_str::CompactStr;
use oxc_syntax::{keyword::is_reserved_keyword, node::NodeId};
use rustc_hash::FxHashMap;

use crate::{base54, debug_name};

/// Resolve labels in their own lexical namespace. Semantic traversal already recorded
/// the label tokens in preorder, so declarations precede their references and nested
/// declarations. Only walk ancestors of labels, never the rest of the program.
pub fn mangle_labels(semantic: &mut Semantic<'_>, debug: bool) {
    let mut slots: FxHashMap<NodeId, usize> = FxHashMap::default();
    let mut names = Vec::new();
    let mut candidate = 0;
    let nodes = semantic.nodes();

    for &id in semantic.label_identifiers() {
        let AstKind::LabelIdentifier(label) = nodes.kind(id) else { unreachable!() };
        let declaration = matches!(nodes.parent_kind(id), AstKind::LabeledStatement(_));
        let mut slot = declaration.then_some(0);
        // For a declaration, skip its own LabeledStatement to find the enclosing label.
        for ancestor in nodes.ancestor_kinds(id).skip(usize::from(declaration)) {
            match ancestor {
                AstKind::Function(_)
                | AstKind::ArrowFunctionExpression(_)
                | AstKind::StaticBlock(_) => break,
                AstKind::LabeledStatement(statement) => {
                    if declaration {
                        slot = Some(slots[&statement.label.node_id.get()] + 1);
                        break;
                    }
                    if statement.label.name == label.name {
                        slot = slots.get(&statement.label.node_id.get()).copied();
                        break;
                    }
                }
                _ => {}
            }
        }
        // Invalid unresolved control flow has no target to rename.
        let Some(slot) = slot else { continue };
        if slot == names.len() {
            let name = loop {
                let name = if debug {
                    CompactStr::new(debug_name(candidate).as_str())
                } else {
                    CompactStr::new(base54(candidate).as_str())
                };
                candidate += 1;
                // Include strict-mode, generator, and async restrictions regardless of
                // context, so the same depth always gets the same legal name.
                if !is_reserved_keyword(&name) {
                    break name;
                }
            };
            names.push(name);
        }
        slots.insert(id, slot);
    }

    semantic.scoping_mut().set_mangled_label_names(
        slots.into_iter().map(|(id, slot)| (id, names[slot].clone())).collect(),
    );
}
