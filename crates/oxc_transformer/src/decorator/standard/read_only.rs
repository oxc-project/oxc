//! Writes to decorated private methods.

use rustc_hash::FxHashSet;

use oxc_ast::ast::*;
use oxc_ast_visit::{Visit, walk};
use oxc_diagnostics::OxcDiagnostic;
use oxc_str::Ident;

use crate::context::TraverseCtx;

use super::StandardDecoratorState;

impl<'a> StandardDecoratorState<'a> {
    /// Report a write to a decorated private method, which Babel rejects at compile time.
    ///
    /// Undecorated private methods are read-only too, but class-properties lowers writes to them
    /// into a runtime `_readOnlyError`. A decorated method's value is the one its decorators
    /// return, which this lowering stores without a setter, so it is rejected here instead.
    pub(crate) fn report_decorated_private_method_writes(
        body: &ClassBody<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let names = body
            .body
            .iter()
            .filter_map(|element| match element {
                ClassElement::MethodDefinition(method)
                    if method.kind == MethodDefinitionKind::Method
                        && !method.decorators.is_empty() =>
                {
                    method.key.private_name()
                }
                _ => None,
            })
            .collect::<FxHashSet<_>>();
        if names.is_empty() {
            return;
        }
        let mut finder = PrivateMethodWriteFinder { names, errors: vec![] };
        // Walk, not visit: this class's own declarations must not count as shadowing.
        walk::walk_class_body(&mut finder, body);
        for error in finder.errors {
            ctx.state.error(error);
        }
    }
}

struct PrivateMethodWriteFinder<'a> {
    names: FxHashSet<Ident<'a>>,
    errors: Vec<OxcDiagnostic>,
}

impl<'a> Visit<'a> for PrivateMethodWriteFinder<'a> {
    /// Covers assignments, updates, destructuring targets and `for-in`/`for-of` heads.
    fn visit_simple_assignment_target(&mut self, target: &SimpleAssignmentTarget<'a>) {
        if let SimpleAssignmentTarget::PrivateFieldExpression(field) = target
            && self.names.contains(&field.field.name)
        {
            self.errors.push(
                OxcDiagnostic::error(format!(
                    "Decorated private methods are read-only, but \"#{}\" is updated via this expression.",
                    field.field.name
                ))
                .with_label(field.span),
            );
        }
        walk::walk_simple_assignment_target(self, target);
    }

    /// A nested class which declares the same private name shadows it.
    fn visit_class_body(&mut self, body: &ClassBody<'a>) {
        let shadowed = body
            .body
            .iter()
            .filter_map(|element| element.property_key().and_then(PropertyKey::private_name))
            .filter(|name| self.names.remove(name))
            .collect::<Vec<_>>();
        walk::walk_class_body(self, body);
        self.names.extend(shadowed);
    }
}
