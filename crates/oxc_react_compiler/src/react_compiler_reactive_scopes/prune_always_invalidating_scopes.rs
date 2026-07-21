// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

//! PruneAlwaysInvalidatingScopes
//!
//! Some instructions will *always* produce a new value, and unless memoized will *always*
//! invalidate downstream reactive scopes. This pass finds such values and prunes downstream
//! memoization.
//!
//! Corresponds to `src/ReactiveScopes/PruneAlwaysInvalidatingScopes.ts`.

use std::mem::take;

use rustc_hash::FxHashSet;

use oxc_diagnostics::OxcDiagnostic;

use crate::react_compiler_hir::{
    IdentifierId, InstructionValue, PrunedReactiveScopeBlock, ReactiveFunction,
    ReactiveInstruction, ReactiveScopeBlock, ReactiveStatement, ReactiveValue,
    environment::Environment,
};

use crate::react_compiler_reactive_scopes::visitors::{
    ReactiveFunctionTransform, ReactiveFunctionVisitor, Transformed, transform_reactive_function,
    visit_reactive_function,
};

/// Prunes scopes that always invalidate because they depend on unmemoized
/// always-invalidating values.
/// TS: `pruneAlwaysInvalidatingScopes`
pub fn prune_always_invalidating_scopes<'a>(
    func: &mut ReactiveFunction<'a>,
    env: &Environment<'a>,
) -> Result<(), OxcDiagnostic> {
    let mut resource_initializers = FxHashSet::default();
    if env.has_resource_declarations() {
        visit_reactive_function(
            func,
            &ResourceInitializerCollector { env },
            &mut resource_initializers,
        );
    }
    let mut transform = Transform {
        env,
        resource_initializer_values: resource_initializers.clone(),
        always_invalidating_values: resource_initializers.clone(),
        unmemoized_values: resource_initializers,
    };
    let mut state = false; // withinScope
    transform_reactive_function(func, &mut transform, &mut state)
}

struct ResourceInitializerCollector<'a, 'e> {
    env: &'e Environment<'a>,
}

impl<'a, 'e> ReactiveFunctionVisitor<'a> for ResourceInitializerCollector<'a, 'e> {
    type State = FxHashSet<IdentifierId>;

    fn env(&self) -> &Environment<'a> {
        self.env
    }

    fn visit_instruction(&self, instruction: &ReactiveInstruction<'a>, state: &mut Self::State) {
        if let ReactiveValue::Instruction(InstructionValue::StoreLocal { value, lvalue, .. }) =
            &instruction.value
            && self.env.is_resource_declaration(lvalue.place.identifier)
        {
            state.insert(value.identifier);
        }
        self.traverse_instruction(instruction, state);
    }
}

struct Transform<'a, 'e> {
    env: &'e Environment<'a>,
    resource_initializer_values: FxHashSet<IdentifierId>,
    always_invalidating_values: FxHashSet<IdentifierId>,
    unmemoized_values: FxHashSet<IdentifierId>,
}

impl<'a, 'e> ReactiveFunctionTransform<'a> for Transform<'a, 'e> {
    type State = bool; // withinScope

    fn env(&self) -> &Environment<'a> {
        self.env
    }

    fn transform_instruction(
        &mut self,
        instruction: &mut ReactiveInstruction<'a>,
        within_scope: &mut bool,
    ) -> Result<Transformed<ReactiveStatement<'a>>, OxcDiagnostic> {
        self.visit_instruction(instruction, within_scope)?;

        let lvalue = &instruction.lvalue;
        match &instruction.value {
            ReactiveValue::Instruction(
                InstructionValue::ArrayExpression { .. }
                | InstructionValue::ObjectExpression { .. }
                | InstructionValue::JsxExpression { .. }
                | InstructionValue::JsxFragment { .. }
                | InstructionValue::NewExpression { .. },
            ) => {
                if let Some(lv) = lvalue {
                    self.always_invalidating_values.insert(lv.identifier);
                    if !*within_scope {
                        self.unmemoized_values.insert(lv.identifier);
                    }
                }
            }
            ReactiveValue::Instruction(InstructionValue::StoreLocal {
                value: store_value,
                lvalue: store_lvalue,
                ..
            }) => {
                if self.env.is_resource_declaration(store_lvalue.place.identifier) {
                    // Resource acquisition and registration must happen on every
                    // execution of the declaration. Mark both sides so a scope
                    // producing the initializer cannot memoize it independently.
                    self.always_invalidating_values.insert(store_value.identifier);
                    self.always_invalidating_values.insert(store_lvalue.place.identifier);
                    self.unmemoized_values.insert(store_value.identifier);
                    self.unmemoized_values.insert(store_lvalue.place.identifier);
                }
                if self.always_invalidating_values.contains(&store_value.identifier) {
                    self.always_invalidating_values.insert(store_lvalue.place.identifier);
                }
                if self.unmemoized_values.contains(&store_value.identifier) {
                    self.unmemoized_values.insert(store_lvalue.place.identifier);
                }
            }
            ReactiveValue::Instruction(InstructionValue::LoadLocal { place, .. }) => {
                if let Some(lv) = lvalue {
                    if self.always_invalidating_values.contains(&place.identifier) {
                        self.always_invalidating_values.insert(lv.identifier);
                    }
                    if self.unmemoized_values.contains(&place.identifier) {
                        self.unmemoized_values.insert(lv.identifier);
                    }
                }
            }
            _ => {}
        }
        Ok(Transformed::Keep)
    }

    fn transform_scope(
        &mut self,
        scope: &mut ReactiveScopeBlock<'a>,
        _within_scope: &mut bool,
    ) -> Result<Transformed<ReactiveStatement<'a>>, OxcDiagnostic> {
        let mut within_scope = true;
        self.visit_scope(scope, &mut within_scope)?;

        let scope_id = scope.scope;
        let scope_data = &self.env.scopes[scope_id];

        if scope_data.declarations.iter().any(|(_, declaration)| {
            self.resource_initializer_values.contains(&declaration.identifier)
        }) {
            return Ok(Transformed::Replace(ReactiveStatement::PrunedScope(
                PrunedReactiveScopeBlock {
                    scope: scope.scope,
                    instructions: take(&mut scope.instructions),
                },
            )));
        }

        for dep in &scope_data.dependencies {
            if self.unmemoized_values.contains(&dep.identifier) {
                // This scope depends on an always-invalidating value, prune it
                // Propagate always-invalidating and unmemoized to declarations/reassignments
                let decl_ids: Vec<IdentifierId> =
                    scope_data.declarations.iter().map(|(_, decl)| decl.identifier).collect();
                let reassign_ids: Vec<IdentifierId> = scope_data.reassignments.clone();

                for id in &decl_ids {
                    if self.always_invalidating_values.contains(id) {
                        self.unmemoized_values.insert(*id);
                    }
                }
                for id in &reassign_ids {
                    if self.always_invalidating_values.contains(id) {
                        self.unmemoized_values.insert(*id);
                    }
                }

                return Ok(Transformed::Replace(ReactiveStatement::PrunedScope(
                    PrunedReactiveScopeBlock {
                        scope: scope.scope,
                        instructions: take(&mut scope.instructions),
                    },
                )));
            }
        }
        Ok(Transformed::Keep)
    }
}
