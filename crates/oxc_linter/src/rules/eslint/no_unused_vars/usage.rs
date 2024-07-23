//! This module contains logic for checking if any [`Reference`]s to a
//! [`Symbol`] are considered a usage.

#[allow(clippy::wildcard_imports)]
use oxc_ast::{ast::*, AstKind};
use oxc_semantic::{Reference, ScopeId, SymbolFlags};

use super::{binding_pattern::CheckBinding, options::NoUnusedVarsOptions, NoUnusedVars, Symbol};

impl NoUnusedVars {
    pub(super) fn is_used(&self, symbol: &Symbol<'_, '_>) -> bool {
        // Order matters. We want to call cheap/high "yield" functions first.
        symbol.is_exported() || symbol.has_usages(self)
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
    pub fn has_usages(&self, options: &NoUnusedVarsOptions) -> bool {
        let do_reassignment_checks = self.is_assignable();
        let do_self_call_check = self.is_maybe_callable();

        for reference in self.references() {
            if reference.is_write() {
                if do_reassignment_checks
                    && (self.is_assigned_to_ignored_destructure(reference, options)
                        || self.is_used_in_for_of_loop(reference))
                {
                    return true;
                }
                // references can be both reads & writes.
                if !reference.is_read() {
                    continue;
                }
            }

            if reference.is_type() {
                return true;
            }

            if do_reassignment_checks && self.is_self_reassignment(reference) {
                continue;
            }

            if do_self_call_check && self.is_self_call(reference) {
                continue;
            }

            return true;
        }

        false
    }

    fn is_used_in_for_of_loop(&self, reference: &Reference) -> bool {
        for parent in self.nodes().iter_parents(reference.node_id()) {
            match parent.kind() {
                AstKind::ParenthesizedExpression(_)
                | AstKind::IdentifierReference(_)
                | AstKind::SimpleAssignmentTarget(_)
                | AstKind::AssignmentTarget(_) => continue,
                AstKind::ForInStatement(ForInStatement { body, .. })
                | AstKind::ForOfStatement(ForOfStatement { body, .. }) => match body {
                    Statement::ReturnStatement(_) => return true,
                    Statement::BlockStatement(b) => {
                        return b
                            .body
                            .first()
                            .is_some_and(|s| matches!(s, Statement::ReturnStatement(_)))
                    }
                    _ => return false,
                },
                // AstKind::ForInStatement(_) | AstKind::ForOfStatement(_) => return true,
                _ => return false,
            }
        }

        false
    }

    /// Does this variable have a name that is ignored by the destructuring
    /// pattern, and is also assigned inside a destructure?
    ///
    /// ```ts
    /// let a, _b;
    /// [a, _b] = [1, 2];
    /// //  ^^ this should be ignored
    ///
    /// console.log(a)
    /// ```
    fn is_assigned_to_ignored_destructure(
        &self,
        reference: &Reference,
        options: &NoUnusedVarsOptions,
    ) -> bool {
        for parent in self.nodes().iter_parents(reference.node_id()) {
            match parent.kind() {
                AstKind::IdentifierReference(_)
                | AstKind::SimpleAssignmentTarget(_)
                | AstKind::AssignmentExpression(_) => continue,
                AstKind::AssignmentPattern(pattern) => {
                    if let Some(res) = pattern.left.check_unused_binding_pattern(options, self) {
                        return res.is_ignore();
                    }
                }
                AstKind::AssignmentTarget(target) => {
                    if let Some(res) = target.check_unused_binding_pattern(options, self) {
                        return res.is_ignore();
                    }
                }
                _ => {
                    return false;
                }
            }
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
        for node in self.nodes().iter_parents(reference.node_id()).skip(1) {
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
                // expression is over, save cycles by breaking
                // todo: do we need to check if variable is used as iterator in
                // loops?
                AstKind::Argument(_)
                | AstKind::ForInStatement(_)
                | AstKind::ForOfStatement(_)
                | AstKind::WhileStatement(_)
                | AstKind::Function(_)
                | AstKind::ExpressionStatement(_) => {
                    break;
                }
                // AstKind::Function(f) if !f.is_expression() => {
                //     break;
                // }
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

    fn is_self_call(&self, reference: &Reference) -> bool {
        let mut nodes = self.iter_relevant_parents(reference.node_id());
        let Some(ref_node) = nodes.next() else {
            return false;
        };
        if !matches!(ref_node.kind(), AstKind::CallExpression(_) | AstKind::NewExpression(_)) {
            return false;
        }
        let call_scope_id = ref_node.scope_id();
        let allegedly_own_scope_id = self.scope_id();
        let name = self.name();
        let decl_scope_id = self
            .scopes()
            .ancestors(allegedly_own_scope_id)
            .find(|scope_id| self.scopes().get_binding(*scope_id, name).is_some());
        let Some(decl_scope_id) = decl_scope_id else {
            return false;
        };
        if call_scope_id == decl_scope_id {
            return false;
        }

        for scope_id in self.scopes().ancestors(call_scope_id) {
            if scope_id == decl_scope_id {
                return true;
            }

            if self.is_inside_storable_function(scope_id, decl_scope_id) {
                return false;
            }
        }
        false
    }

    fn is_inside_storable_function(&self, scope_id: ScopeId, decl_scope_id: ScopeId) -> bool {
        let parents = self.iter_relevant_parents(self.scopes().get_node_id(scope_id));

        for callback_argument_or_fn_assignment in parents {
            match callback_argument_or_fn_assignment.kind() {
                AstKind::IfStatement(_)
                | AstKind::WhileStatement(_)
                | AstKind::ForStatement(_)
                | AstKind::ForInStatement(_)
                | AstKind::ForOfStatement(_)
                // | AstKind::Function(_)
                | AstKind::ArrowFunctionExpression(_)
                | AstKind::ExpressionStatement(_) => {
                    continue;
                }
                AstKind::Function(f) => {
                    if f.id.as_ref().is_some_and(|id| self == id) {
                        return false;
                    }
                    continue;
                }
                AstKind::Argument(_) => {
                    // return parents.clone().next().is_some_and(|node| {
                    return self.iter_relevant_parents(callback_argument_or_fn_assignment.id()).next().is_some_and(|node| {
                        // matches!(node.kind(), AstKind::CallExpression(_))
                        matches!(node.kind(), AstKind::CallExpression(call) if call.callee.get_identifier_reference().map_or(true, |identifier| self != identifier))
                    });
                }
                AstKind::SimpleAssignmentTarget(_)
                | AstKind::YieldExpression(_)
                | AstKind::TaggedTemplateExpression(_)
                | AstKind::TemplateLiteral(_)
                | AstKind::AssignmentExpression(_) => {
                    return true
                }
                AstKind::FunctionBody(_) => {
                    for parent in self.nodes().iter_parents(callback_argument_or_fn_assignment.id()).skip(1) {
                        if parent.scope_id() == decl_scope_id {
                            return false;
                        }
                        match parent.kind() {
                            AstKind::Function(f) => {
                                // could be `unused = function() { unused() }`
                                // note: else needed, otherwise rustfmt panics
                                #[allow(clippy::redundant_else)]
                                if f.is_expression() {
                                    let Some(parent) = self.iter_parents().next() else {
                                        return false;
                                    };
                                    if parent.scope_id() <= decl_scope_id {
                                        return false;
                                    }
                                    let AstKind::AssignmentExpression(assignment) = parent.kind() else {
                                        return true;
                                    };
                                    let Some(id) = assignment.left.get_identifier() else {
                                        return true;
                                    };
                                    return id != self.name();
                                } else {
                                    // none means an anonymous fn, which will not be a
                                    // self call. If some, check that it's not the same
                                    return !f.id.as_ref().is_some_and(|id| self == id);
                                }
                            }
                            // to get the identifier for an arrow function, we need
                            // to go to the parent and look at the assignment target
                            // or binding pattern
                            AstKind::ArrowFunctionExpression(_) => {
                                continue;
                            }
                            AstKind::VariableDeclarator(v) => {
                                return !v.id.get_binding_identifier().is_some_and(|id| self == id)
                            }
                            _ => return false,
                        }
                    }
                    unreachable!()
                }
                _ => {
                    return false;
                }
            }
        }
        false
    }
}
