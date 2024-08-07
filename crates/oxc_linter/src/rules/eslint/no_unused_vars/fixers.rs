use oxc_ast::{ast::VariableDeclarator, AstKind};
use oxc_semantic::{AstNode, AstNodeId};
use oxc_span::{CompactStr, GetSpan};
use regex::Regex;

use super::{NoUnusedVars, Symbol};
use crate::fixer::{Fix, RuleFix, RuleFixer};

impl NoUnusedVars {
    pub(super) fn rename_or_remove_var_declaration<'a>(
        &self,
        fixer: RuleFixer<'_, 'a>,
        symbol: &Symbol<'_, 'a>,
        decl: &VariableDeclarator<'a>,
        decl_id: AstNodeId,
    ) -> RuleFix<'a> {
        let Some(AstKind::VariableDeclaration(declaration)) =
            symbol.nodes().parent_node(decl_id).map(AstNode::kind)
        else {
            panic!("VariableDeclarator nodes should always be direct children of VariableDeclaration nodes");
        };

        // `true` even if references aren't considered a usage.
        let has_references = symbol.has_references();

        // we can delete variable declarations that aren't referenced anywhere
        if !has_references {
            let has_neighbors = declaration.declarations.len() > 1;

            // `let x = 1;` the whole declaration can be removed
            if !has_neighbors {
                return fixer.delete(declaration).dangerously();
            }

            let own_position =
                declaration.declarations.iter().position(|d| symbol == &d.id).expect(
                    "VariableDeclarator not found within its own parent VariableDeclaration",
                );
            let mut delete_range = decl.span();
            let mut has_left = false;
            let mut has_right = false;

            // `let x = 1, y = 2, z = 3;` -> `let x = 1, y = 2, z = 3;`
            //             ^^^^^                         ^^^^^^^
            if let Some(right_neighbor) = declaration.declarations.get(own_position + 1) {
                delete_range.end = right_neighbor.span.start;
                has_right = true;
            }

            // `let x = 1, y = 2, z = 3;` -> `let x = 1, y = 2, z = 3;`
            //             ^^^^^                       ^^^^^^^
            if own_position > 0 {
                if let Some(left_neighbor) = declaration.declarations.get(own_position - 1) {
                    delete_range.start = left_neighbor.span.end;
                    has_left = true;
                }
            }

            // both left and right neighbors are present, so we need to
            // re-insert a comma
            // `let x = 1, y = 2, z = 3;`
            //           ^^^^^^^^^
            if has_left && has_right {
                return fixer.replace(delete_range, ", ").dangerously();
            }

            return fixer.delete(&delete_range).dangerously();
        }

        // otherwise, try to rename the variable to match the unused variable
        // pattern
        if let Some(new_name) = self.get_unused_var_name(symbol) {
            return symbol.rename(&new_name).dangerously();
        }

        fixer.noop()
    }

    fn get_unused_var_name(&self, symbol: &Symbol<'_, '_>) -> Option<CompactStr> {
        let pat = self.vars_ignore_pattern.as_ref().map(Regex::as_str)?;

        let ignored_name: String = match pat {
            // TODO: suppport more patterns
            "^_" => format!("_{}", symbol.name()),
            _ => return None,
        };

        // adjust name to avoid conflicts
        let scopes = symbol.scopes();
        let scope_id = symbol.scope_id();
        let mut i = 0;
        let mut new_name = ignored_name.clone();
        while scopes.has_binding(scope_id, &new_name) {
            new_name = format!("{ignored_name}{i}");
            i += 1;
        }

        Some(new_name.into())
    }
}

impl<'s, 'a> Symbol<'s, 'a> {
    fn rename(&self, new_name: &CompactStr) -> RuleFix<'a> {
        let mut fixes: Vec<Fix<'a>> = vec![];
        let decl_span = self.symbols().get_span(self.id());
        fixes.push(Fix::new(new_name.clone(), decl_span));

        for reference in self.references() {
            match self.nodes().get_node(reference.node_id()).kind() {
                AstKind::IdentifierReference(id) => {
                    fixes.push(Fix::new(new_name.clone(), id.span()));
                }
                AstKind::TSTypeReference(ty) => {
                    fixes.push(Fix::new(new_name.clone(), ty.type_name.span()));
                }
                // we found a reference to an unknown node and we don't know how
                // to replace it, so we abort the whole process
                _ => return Fix::empty().into(),
            }
        }

        return RuleFix::from(fixes)
            .with_message(format!("Rename '{}' to '{new_name}'", self.name()));
    }
}
