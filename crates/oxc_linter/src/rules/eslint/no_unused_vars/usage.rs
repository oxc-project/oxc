#[allow(clippy::wildcard_imports)]
use oxc_ast::{ast::*, AstKind};
use oxc_semantic::{Reference, SymbolFlags};

use super::{NoUnusedVars, Symbol};

impl NoUnusedVars {
    pub(super) fn is_used(&self, symbol: &Symbol<'_, '_>) -> bool {
        // Order matters. We want to call cheap/high "yield" functions first.
        symbol.is_exported() || symbol.has_usages()
    }
}

impl<'s, 'a> Symbol<'s, 'a> {
    const fn is_maybe_callable(&self) -> bool {
        !self.flags().intersects(SymbolFlags::Import.union(SymbolFlags::CatchVariable))
    }

    const fn is_assignable(&self) -> bool {
        self.flags().intersects(SymbolFlags::Variable)
    }

    /// Check if this [`Symbol`] has an [`Reference`]s that are considered a usage.
    pub fn has_usages(&self) -> bool {
        let do_self_reassignment_check = self.is_assignable();
        let do_self_call_check = self.is_maybe_callable();

        for reference in self.references() {
            if reference.is_write() {
                continue;
            }

            if reference.is_type() {
                return true;
            }

            if do_self_reassignment_check && self.is_self_reassignment(reference) {
                continue;
            }

            if do_self_call_check && self.is_self_call(reference) {
                continue;
            }

            return true;
        }

        false
    }

    fn is_self_reassignment(&self, reference: &Reference) -> bool {
        if reference.symbol_id().is_none() {
            debug_assert!(
                false,
                "is_self_reassignment() should only be called on resolved symbol references"
            );
            return true;
        }
        let mut is_used_by_others = true;
        let name = self.name();
        for node in self.iter_parents() {
            match node.kind() {
                // references used in declaration of another variable are definitely
                // used by others
                AstKind::VariableDeclarator(v) => {
                    // let a = a; is a syntax error, even if `a` is shadowed.
                    debug_assert!(
                        v.id.kind.get_identifier().map_or_else(|| true, |id| id != name),
                        "While traversing {name}'s reference's parent nodes, found {name}'s declaration. This algorithm assumes that variable declarations do not appear in references."
                    );
                    // definitely used, short-circuit
                    return false;
                }
                // When symbol is being assigned a new value, we flag the reference
                // as only affecting itself until proven otherwise.
                AstKind::UpdateExpression(_) | AstKind::SimpleAssignmentTarget(_) => {
                    is_used_by_others = false;
                }
                // RHS usage when LHS != reference's symbol is definitely used by
                // others
                AstKind::AssignmentExpression(AssignmentExpression { left, .. }) => {
                    match left {
                        AssignmentTarget::AssignmentTargetIdentifier(id) => {
                            if id.name == name {
                                is_used_by_others = false;
                            } else {
                                return false; // we can short-circuit
                            }
                        }
                        // variable is being used to index another variable, this is
                        // always a usage
                        // todo: check self index?
                        match_member_expression!(AssignmentTarget) => return false,
                        _ => {
                            // debug_assert!(false, "is_self_update only supports AssignmentTargetIdentifiers right now. Please update this function. Found {t:#?}",);
                        }
                    }
                }
                AstKind::Argument(_) => {
                    break;
                }
                // expression is over, save cycles by breaking
                // todo: do we need to check if variable is used as iterator in loops?
                AstKind::ForInStatement(_)
                | AstKind::ForOfStatement(_)
                | AstKind::WhileStatement(_)
                | AstKind::Function(_)
                | AstKind::ExpressionStatement(_) => {
                    break;
                }
                // function* foo() {
                //    let a = 1;
                //    a = yield a // <- still considered used b/c it's propagated to the caller
                // }
                AstKind::YieldExpression(_) => return false,
                _ => { /* continue up tree */ }
            }
        }

        !is_used_by_others
    }

    fn is_self_call(&self, _reference: &Reference) -> bool {
        false // todo
    }
}

// impl<'s, 'a> Symbol<'s, 'a> {
//     fn is_self_reassignment(&self, reference: &Reference) -> bool {
//         let Some(symbol_id) = reference.symbol_id() else {
//             debug_assert!(
//                 false,
//                 "is_self_update() should only be called on resolved symbol references"
//             );
//             return true;
//         };
//         let node_id = reference.node_id();
//         let mut is_used_by_others = true;
//         let name = self.name();
//         for node in self.iter_parents() {
//             println!("kind: {}, used: {is_used_by_others}", node.kind().debug_name());
//             match node.kind() {
//                 // references used in declaration of another variable are definitely
//                 // used by others
//                 AstKind::VariableDeclarator(v) => {
//                     debug_assert!(
//                         v.id.kind.get_identifier().map_or_else(|| true, |id| id != name),
//                         "While traversing {name}'s reference's parent nodes, found {name}'s declaration. This algorithm assumes that variable declarations do not appear in references."
//                     );
//                     // definitely used, short-circuit
//                     return false;
//                 }
//                 // When symbol is being assigned a new value, we flag the reference
//                 // as only affecting itself until proven otherwise.
//                 AstKind::UpdateExpression(_) | AstKind::SimpleAssignmentTarget(_) => {
//                     is_used_by_others = false;
//                 }
//                 // RHS usage when LHS != reference's symbol is definitely used by
//                 // others
//                 AstKind::AssignmentExpression(AssignmentExpression {
//                     left: AssignmentTarget::SimpleAssignmentTarget(target),
//                     ..
//                 }) => {
//                     match target {
//                         SimpleAssignmentTarget::AssignmentTargetIdentifier(id) => {
//                             if id.name == name {
//                                 is_used_by_others = false;
//                             } else {
//                                 return false; // we can short-circuit
//                             }
//                         }
//                         // variable is being used to index another variable, this is
//                         // always a usage
//                         // todo: check self index?
//                         SimpleAssignmentTarget::MemberAssignmentTarget(_) => return false,
//                         _ => {
//                             // debug_assert!(false, "is_self_update only supports AssignmentTargetIdentifiers right now. Please update this function. Found {t:#?}",);
//                         }
//                     }
//                 }
//                 AstKind::Argument(_) => {
//                     break;
//                 }
//                 // expression is over, save cycles by breaking
//                 // todo: do we need to check if variable is used as iterator in loops?
//                 AstKind::ForInStatement(_)
//                 | AstKind::ForOfStatement(_)
//                 | AstKind::WhileStatement(_)
//                 | AstKind::Function(_)
//                 | AstKind::ExpressionStatement(_) => {
//                     break;
//                 }
//                 AstKind::YieldExpression(_) => return false,
//                 _ => { /* continue up tree */ }
//             }
//         }

//         !is_used_by_others
//     }

//     pub fn is_self_call(&self, reference: &Reference) -> bool {
//         let scopes = self.scopes();

//         // determine what scope the call occurred in
//         let node_id = reference.node_id();
//         let node = self
//             .nodes()
//             .iter_parents(node_id)
//             .skip(1)
//             .filter(|n| {
//                 dbg!(n.kind().debug_name());
//                 !matches!(n.kind(), AstKind::ParenthesizedExpression(_))
//             })
//             .nth(0);
//         if !matches!(
//             node.map(|n| {
//                 println!("{}", n.kind().debug_name());
//                 n.kind()
//             }),
//             Some(AstKind::CallExpression(_) | AstKind::NewExpression(_))
//         ) {
//             return false;
//         }

//         let call_scope_id = self.nodes().get_node(node_id).scope_id();
//         // note: most nodes record what scope they were declared in. The
//         // exception is functions and classes, which record the scopes they create.
//         let decl_scope_id = self
//             .scopes()
//             .ancestors(self.scope_id())
//             .find(|scope_id| self.scopes().get_binding(*scope_id, self.name()).is_some())
//             .unwrap();
//         if call_scope_id == decl_scope_id {
//             return false;
//         };

//         let is_called_inside_self = scopes.ancestors(call_scope_id).any(|scope_id| {
//             // let flags = scopes.get_flags(scope_id);
//             // scope_id == decl_scope_id && flags.intersects(ScopeFlags::Function | ScopeFlags::Arrow)
//             scope_id == decl_scope_id
//         });

//         return is_called_inside_self;
//     }
// }
