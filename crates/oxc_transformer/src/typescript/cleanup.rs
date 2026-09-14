//! Record semantic data in erased syntax before its AST nodes are discarded.

use oxc_ast::ast::{
    ArrowFunctionExpression, BindingIdentifier, FormalParameters, Function, IdentifierReference,
    Program,
};
use oxc_ast_visit::{Visit, walk};
use oxc_ecmascript::BoundNames;
use oxc_semantic::Scoping;
use oxc_span::Span;
use oxc_syntax::{
    reference::ReferenceId,
    scope::{ScopeFlags, ScopeId},
    symbol::SymbolId,
};
use rustc_hash::FxHashSet;

use crate::context::TraverseCtx;

#[derive(Default)]
pub struct TypeScriptCleanup {
    declarations: FxHashSet<(SymbolId, Span)>,
    references: FxHashSet<ReferenceId>,
}

impl TypeScriptCleanup {
    pub fn finish(self, program: &Program<'_>, ctx: &mut TraverseCtx<'_>) {
        let scoping = ctx.scoping_mut();
        scoping.remove_references(&self.references);
        let mut parameters = ParameterReferences {
            scoping,
            function: None,
            environments: Vec::new(),
            inaccessible: FxHashSet::default(),
        };
        parameters.visit_program(program);
        let inaccessible = parameters.inaccessible;
        scoping.delete_typescript_bindings_with(
            |id, span| self.declarations.contains(&(id, span)),
            |reference, symbol| !inaccessible.contains(&(reference, symbol)),
        );
    }
}

/// All removals are deferred until import/export retention has finished.
pub(super) struct Erase<'c, 'a>(pub &'c mut TraverseCtx<'a>);

impl<'a> Visit<'a> for Erase<'_, 'a> {
    fn visit_binding_identifier(&mut self, ident: &BindingIdentifier<'a>) {
        if let Some(id) = ident.symbol_id.get() {
            self.0.state.typescript_cleanup.declarations.insert((id, ident.span));
        }
    }

    fn visit_identifier_reference(&mut self, ident: &IdentifierReference<'a>) {
        if let Some(id) = ident.reference_id.get() {
            self.0.state.typescript_cleanup.references.insert(id);
        }
    }
}

/// Function bodies share their semantic scope with parameters, but their bindings
/// are inaccessible from parameter initializers (including nested closures).
/// Derive those exclusions from surviving syntax so this also handles references
/// and scopes introduced by other transforms without permanent semantic metadata.
struct ParameterReferences<'s> {
    scoping: &'s Scoping,
    function: Option<(ScopeId, Option<SymbolId>)>,
    environments: Vec<(ScopeId, FxHashSet<SymbolId>)>,
    inaccessible: FxHashSet<(ReferenceId, SymbolId)>,
}

impl<'a> Visit<'a> for ParameterReferences<'_> {
    fn visit_function(&mut self, function: &Function<'a>, flags: ScopeFlags) {
        let name = function
            .id
            .as_ref()
            .filter(|_| function.is_expression())
            .and_then(|id| id.symbol_id.get());
        let previous = self.function.replace((function.scope_id.get().unwrap(), name));
        walk::walk_function(self, function, flags);
        self.function = previous;
    }

    fn visit_arrow_function_expression(&mut self, function: &ArrowFunctionExpression<'a>) {
        let previous = self.function.replace((function.scope_id.get().unwrap(), None));
        walk::walk_arrow_function_expression(self, function);
        self.function = previous;
    }

    fn visit_formal_parameters(&mut self, parameters: &FormalParameters<'a>) {
        let (scope, name) = self.function.unwrap();
        let mut bindings = FxHashSet::default();
        bindings.extend(name);
        parameters.bound_names(&mut |id| {
            bindings.extend(id.symbol_id.get());
        });
        self.environments.push((scope, bindings));
        walk::walk_formal_parameters(self, parameters);
        self.environments.pop();
    }

    fn visit_identifier_reference(&mut self, ident: &IdentifierReference<'a>) {
        let Some(reference) = ident.reference_id.get() else {
            return;
        };
        for (scope, parameters) in &self.environments {
            if let Some(symbol) = self.scoping.get_binding(*scope, ident.name)
                && !parameters.contains(&symbol)
            {
                self.inaccessible.insert((reference, symbol));
            }
        }
    }
}
