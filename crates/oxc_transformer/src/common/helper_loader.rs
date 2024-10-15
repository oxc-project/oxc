//! Utility transform to load helper functions
//!
//! This module provides functionality to load helper functions in different modes.
//! It supports runtime, external, and inline (not yet implemented) modes for loading helper functions.
//!
//! ## Usage
//!
//! You can call [`HelperLoaderStore::load`] to load a helper function and use it in your CallExpression.
//!
//! ```rs
//! let callee = self.ctx.helper_loader.load("helperName");
//! let call = self.ctx.ast.call_expression(callee, ...arguments);
//! ```
//!
//! And also you can call [`HelperLoaderStore::call`] directly to load and call a helper function.
//!
//! ```rs
//! let call_expression = self.ctx.helper_loader.call("helperName", ...arguments);
//! ```
//!
//! ## Modes
//!
//! ### Runtime ([`HelperLoaderMode::Runtime`])
//!
//! Uses `@babel/runtime` as a dependency, importing helper functions from the runtime.
//!
//! Generated code example:
//!
//! ```js
//! import helperName from "@babel/runtime/helpers/helperName";
//! helperName(...arguments);
//! ```
//!
//! Based on [@babel/plugin-transform-runtime](https://github.com/babel/babel/tree/main/packages/babel-plugin-transform-runtime).
//!
//! ### External ([`HelperLoaderMode::External`])
//!
//! Uses helper functions from a global `babelHelpers` variable. This is the default mode for testing.
//!
//! Generated code example:
//!
//! ```js
//! babelHelpers.helperName(...arguments);
//! ```
//!
//! Based on [@babel/plugin-external-helpers](https://github.com/babel/babel/tree/main/packages/babel-plugin-external-helpers).
//!
//! ### Inline ([`HelperLoaderMode::Inline`])
//!
//! > Note: This mode is not currently implemented.
//!
//! Inline helper functions are inserted directly into the top of program.
//!
//! Generated code example:
//!
//! ```js
//! function helperName(...arguments) { ... } // Inlined helper function
//! helperName(...arguments);
//! ```
//!
//! Based on [@babel/helper](https://github.com/babel/babel/tree/main/packages/babel-helpers).

use std::{borrow::Cow, cell::RefCell};

use rustc_hash::FxHashMap;
use serde::Deserialize;

use oxc_allocator::Vec;
use oxc_ast::ast::{Argument, CallExpression, Expression, Program, TSTypeParameterInstantiation};
use oxc_semantic::{ReferenceFlags, SymbolFlags};
use oxc_span::{Atom, SPAN};
use oxc_traverse::{BoundIdentifier, Traverse, TraverseCtx};

use crate::TransformCtx;

/// Defines the mode for loading helper functions.
#[derive(Default, Clone, Copy, Debug, Deserialize)]
pub enum HelperLoaderMode {
    /// Inline mode: Helper functions are directly inserted into the program.
    ///
    /// Note: This mode is not currently implemented.
    ///
    /// Example output:
    /// ```js
    /// function helperName(...arguments) { ... } // Inlined helper function
    /// helperName(...arguments);
    /// ```
    Inline,
    /// External mode: Helper functions are accessed from a global `babelHelpers` object.
    ///
    /// This is the default mode used in Babel tests.
    ///
    /// Example output:
    /// ```js
    /// babelHelpers.helperName(...arguments);
    /// ```
    External,
    /// Runtime mode: Helper functions are imported from a runtime package.
    ///
    /// This mode is similar to how @babel/plugin-transform-runtime works.
    /// It's the default mode for this implementation.
    ///
    /// Example output:
    /// ```js
    /// import helperName from "@babel/runtime/helpers/helperName";
    /// helperName(...arguments);
    /// ```
    #[default]
    Runtime,
}

#[derive(Clone, Debug, Deserialize)]
pub struct HelperLoaderOptions {
    #[serde(default = "default_as_module_name")]
    /// The module name to import helper functions from.
    /// Default: `@babel/runtime`
    pub module_name: Cow<'static, str>,
    pub mode: HelperLoaderMode,
}

impl Default for HelperLoaderOptions {
    fn default() -> Self {
        Self { module_name: default_as_module_name(), mode: HelperLoaderMode::default() }
    }
}

fn default_as_module_name() -> Cow<'static, str> {
    Cow::Borrowed("@babel/runtime")
}

pub struct HelperLoader<'a, 'ctx> {
    ctx: &'ctx TransformCtx<'a>,
}

impl<'a, 'ctx> HelperLoader<'a, 'ctx> {
    pub fn new(ctx: &'ctx TransformCtx<'a>) -> Self {
        Self { ctx }
    }
}

impl<'a, 'ctx> Traverse<'a> for HelperLoader<'a, 'ctx> {
    fn exit_program(&mut self, _program: &mut Program<'a>, _ctx: &mut TraverseCtx<'a>) {
        self.ctx.helper_loader.add_imports(self.ctx);
    }
}

struct Helper<'a> {
    source: Atom<'a>,
    local: BoundIdentifier<'a>,
}

/// Stores the state of the helper loader in [`TransformCtx`].
pub struct HelperLoaderStore<'a> {
    module_name: Cow<'static, str>,
    mode: HelperLoaderMode,
    /// Loaded helpers, determined what helpers are loaded and what imports should be added.
    loaded_helpers: RefCell<FxHashMap<Atom<'a>, Helper<'a>>>,
}

// Public methods
impl<'a> HelperLoaderStore<'a> {
    pub fn new(options: &HelperLoaderOptions) -> Self {
        Self {
            module_name: options.module_name.clone(),
            mode: options.mode,
            loaded_helpers: RefCell::new(FxHashMap::default()),
        }
    }

    /// Load and call a helper function and return the `CallExpression`.
    #[expect(dead_code)]
    pub fn call(
        &mut self,
        helper_name: Atom<'a>,
        arguments: Vec<'a, Argument<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) -> CallExpression<'a> {
        let callee = self.load(helper_name, ctx);
        ctx.ast.call_expression(
            SPAN,
            callee,
            None::<TSTypeParameterInstantiation<'a>>,
            arguments,
            false,
        )
    }

    /// Same as [`HelperLoaderStore::call`], but returns a `CallExpression` wrapped in an `Expression`.
    #[expect(dead_code)]
    pub fn call_expr(
        &mut self,
        helper_name: Atom<'a>,
        arguments: Vec<'a, Argument<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let callee = self.load(helper_name, ctx);
        ctx.ast.expression_call(
            SPAN,
            callee,
            None::<TSTypeParameterInstantiation<'a>>,
            arguments,
            false,
        )
    }

    /// Load a helper function and return the callee expression.
    pub fn load(&self, helper_name: Atom<'a>, ctx: &mut TraverseCtx<'a>) -> Expression<'a> {
        match self.mode {
            HelperLoaderMode::Runtime => self.transform_for_runtime_helper(helper_name, ctx),
            HelperLoaderMode::External => Self::transform_for_external_helper(helper_name, ctx),
            HelperLoaderMode::Inline => {
                unreachable!("Inline helpers are not supported yet");
            }
        }
    }
}

// Internal methods
impl<'a> HelperLoaderStore<'a> {
    fn transform_for_runtime_helper(
        &self,
        helper_name: Atom<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let mut loaded_helpers = self.loaded_helpers.borrow_mut();
        let helper = loaded_helpers.entry(helper_name).or_insert_with_key(|helper_name| {
            let source = ctx.ast.atom(&format!("{}/helpers/{helper_name}", self.module_name));
            let local = ctx.generate_uid_in_root_scope(helper_name, SymbolFlags::Import);
            Helper { source, local }
        });
        helper.local.create_read_expression(ctx)
    }

    fn transform_for_external_helper(
        helper_name: Atom<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        static HELPER_VAR: &str = "babelHelpers";

        let symbol_id = ctx.scopes().find_binding(ctx.current_scope_id(), HELPER_VAR);
        let ident =
            ctx.create_reference_id(SPAN, Atom::from(HELPER_VAR), symbol_id, ReferenceFlags::Read);
        let object = ctx.ast.expression_from_identifier_reference(ident);
        let property = ctx.ast.identifier_name(SPAN, helper_name);
        Expression::from(ctx.ast.member_expression_static(SPAN, object, property, false))
    }

    fn add_imports(&self, transform_ctx: &TransformCtx<'a>) {
        let mut loaded_helpers = self.loaded_helpers.borrow_mut();
        for (_, helper) in loaded_helpers.drain() {
            transform_ctx.module_imports.add_default_import(helper.source, helper.local, false);
        }
    }
}
