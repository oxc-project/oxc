//! Utility to load helper functions.
//!
//! This module provides functionality to load helper functions in different modes.
//! It supports runtime, external, and inline (not yet implemented) modes for loading helper functions.
//!
//! ## Usage
//!
//! You can call [`helper_load`] to load a helper function and use it in your CallExpression.
//!
//! ```rs
//! let callee = helper_load(Helper::ObjectSpread2, ctx);
//! let call = ctx.ast.call_expression(callee, ...arguments);
//! ```
//!
//! And also you can call [`helper_call`] directly to load and call a helper function.
//!
//! ```rs
//! let call_expression = helper_call(Helper::ObjectSpread2, ...arguments, ctx);
//! ```
//!
//! ## Modes
//!
//! ### Runtime ([`HelperLoaderMode::Runtime`])
//!
//! Uses `@oxc-project/runtime` as a dependency, importing helper functions from the runtime.
//!
//! Generated code example:
//!
//! ```js
//! import helperName from "@oxc-project/runtime/helpers/helperName";
//! helperName(...arguments);
//! ```
//!
//! Based on [@babel/plugin-transform-runtime](https://github.com/babel/babel/tree/v7.26.2/packages/babel-plugin-transform-runtime).
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
//! Based on [@babel/plugin-external-helpers](https://github.com/babel/babel/tree/v7.26.2/packages/babel-plugin-external-helpers).
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
//! Based on [@babel/helper](https://github.com/babel/babel/tree/v7.26.2/packages/babel-helpers).
//!
//! ## Implementation
//!
//! Unlike other "common" utilities, this one has no transformer. It adds imports to the program
//! via `ModuleImports` transform.

use std::borrow::Cow;

use rustc_hash::FxHashMap;
use serde::Deserialize;

use oxc_allocator::{ArenaBox, ArenaVec};
use oxc_ast::ast::{Argument, CallExpression, Expression, IdentifierName};
use oxc_semantic::{ReferenceFlags, SymbolFlags};
use oxc_span::SPAN;
use oxc_str::{Str, static_ident};
use oxc_traverse::BoundIdentifier;

use crate::context::TraverseCtx;

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
    /// import helperName from "@oxc-project/runtime/helpers/helperName";
    /// helperName(...arguments);
    /// ```
    #[default]
    Runtime,
}

/// Helper loader options.
#[derive(Clone, Debug, Deserialize)]
pub struct HelperLoaderOptions {
    #[serde(default = "default_as_module_name")]
    /// The module name to import helper functions from.
    /// Default: `@oxc-project/runtime`
    pub module_name: Cow<'static, str>,
    /// Strategy used to resolve helper calls.
    pub mode: HelperLoaderMode,
}

impl Default for HelperLoaderOptions {
    fn default() -> Self {
        Self { module_name: default_as_module_name(), mode: HelperLoaderMode::default() }
    }
}

fn default_as_module_name() -> Cow<'static, str> {
    Cow::Borrowed("@oxc-project/runtime")
}

/// Defines [`Helper`], [`Helper::ALL`] and [`Helper::name`] from a single list,
/// so a new helper is always listed and named.
macro_rules! define_helpers {
    ($($variant:ident => $name:literal,)+) => {
        /// Available helpers.
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
        pub enum Helper {
            $(
                #[doc = concat!("Runtime helper `", $name, "`.")]
                $variant,
            )+
        }

        impl Helper {
            /// Every helper the transformer can emit, in declaration order.
            pub const ALL: &[Self] = &[$(Self::$variant,)+];

            /// Canonical helper export name used by runtime/external helper loaders.
            pub const fn name(self) -> &'static str {
                match self {
                    $(Self::$variant => $name,)+
                }
            }
        }
    };
}

define_helpers! {
    AwaitAsyncGenerator => "awaitAsyncGenerator",
    AsyncGeneratorDelegate => "asyncGeneratorDelegate",
    AsyncIterator => "asyncIterator",
    AsyncToGenerator => "asyncToGenerator",
    ObjectSpread2 => "objectSpread2",
    WrapAsyncGenerator => "wrapAsyncGenerator",
    Extends => "extends",
    ObjectDestructuringEmpty => "objectDestructuringEmpty",
    ObjectWithoutProperties => "objectWithoutProperties",
    ToPropertyKey => "toPropertyKey",
    DefineProperty => "defineProperty",
    ClassPrivateFieldInitSpec => "classPrivateFieldInitSpec",
    ClassPrivateMethodInitSpec => "classPrivateMethodInitSpec",
    ClassPrivateFieldGet2 => "classPrivateFieldGet2",
    ClassPrivateFieldSet2 => "classPrivateFieldSet2",
    AssertClassBrand => "assertClassBrand",
    ToSetter => "toSetter",
    ClassPrivateFieldLooseKey => "classPrivateFieldLooseKey",
    ClassPrivateFieldLooseBase => "classPrivateFieldLooseBase",
    SuperPropGet => "superPropGet",
    SuperPropSet => "superPropSet",
    ReadOnlyError => "readOnlyError",
    WriteOnlyError => "writeOnlyError",
    CheckInRHS => "checkInRHS",
    Decorate => "decorate",
    DecorateParam => "decorateParam",
    DecorateMetadata => "decorateMetadata",
    UsingCtx => "usingCtx",
    TaggedTemplateLiteral => "taggedTemplateLiteral",
    SetFunctionName => "setFunctionName",
}

impl Helper {
    /// Whether calling this helper is side-effect free.
    pub const fn pure(self) -> bool {
        matches!(self, Self::ClassPrivateFieldLooseKey)
    }
}

/// Stores the state of the helper loader in `TransformState`.
pub struct HelperLoaderStore<'a> {
    module_name: Cow<'static, str>,
    mode: HelperLoaderMode,
    /// Loaded helpers, determined what helpers are loaded and what imports should be added.
    loaded_helpers: FxHashMap<Helper, BoundIdentifier<'a>>,
    pub(crate) used_helpers: FxHashMap<Helper, String>,
}

impl HelperLoaderStore<'_> {
    pub fn new(options: &HelperLoaderOptions) -> Self {
        Self {
            module_name: options.module_name.clone(),
            mode: options.mode,
            loaded_helpers: FxHashMap::default(),
            used_helpers: FxHashMap::default(),
        }
    }
}

/// Load and call a helper function and return a `CallExpression`.
///
/// This is a free function to avoid borrow conflicts when accessing state through `ctx.state`.
///
/// Uses `SPAN` for the call expression since it's synthesized and has no original source position.
pub fn helper_call<'a>(
    helper: Helper,
    arguments: ArenaVec<'a, Argument<'a>>,
    ctx: &mut TraverseCtx<'a>,
) -> ArenaBox<'a, CallExpression<'a>> {
    let callee = helper_load(helper, ctx);
    let pure = helper.pure();
    CallExpression::boxed_with_pure(SPAN, callee, None, arguments, false, pure, ctx)
}

/// Same as [`helper_call`], but returns a `CallExpression` wrapped in an `Expression`.
///
/// This is a free function to avoid borrow conflicts when accessing state through `ctx.state`.
pub fn helper_call_expr<'a>(
    helper: Helper,
    arguments: ArenaVec<'a, Argument<'a>>,
    ctx: &mut TraverseCtx<'a>,
) -> Expression<'a> {
    let callee = helper_load(helper, ctx);
    let pure = helper.pure();
    Expression::new_call_expression_with_pure(SPAN, callee, None, arguments, false, pure, ctx)
}

/// Load a helper function and return a callee expression.
///
/// This is a free function to avoid borrow conflicts when accessing state through `ctx.state`.
pub fn helper_load<'a>(helper: Helper, ctx: &mut TraverseCtx<'a>) -> Expression<'a> {
    let source = ctx.state.helper_loader.get_runtime_source(helper, ctx);
    ctx.state.helper_loader.used_helpers.entry(helper).or_insert_with(|| source.to_string());

    let mode = ctx.state.helper_loader.mode;
    match mode {
        HelperLoaderMode::Runtime => {
            // Check if helper is already loaded
            let existing = ctx.state.helper_loader.loaded_helpers.get(&helper).cloned();
            if let Some(binding) = existing {
                return binding.create_read_expression(ctx);
            }

            // Generate new binding
            let is_module = ctx.state.source_type.is_module();
            let flag =
                if is_module { SymbolFlags::Import } else { SymbolFlags::FunctionScopedVariable };
            let binding = ctx.generate_uid_in_root_scope(helper.name(), flag);

            ctx.state.module_imports.add_default_import(source, binding.clone(), false);
            ctx.state.helper_loader.loaded_helpers.insert(helper, binding.clone());

            binding.create_read_expression(ctx)
        }
        HelperLoaderMode::External => HelperLoaderStore::transform_for_external_helper(helper, ctx),
        HelperLoaderMode::Inline => {
            unreachable!("Inline helpers are not supported yet");
        }
    }
}

// Internal methods
impl<'a> HelperLoaderStore<'a> {
    // Construct string directly in arena without an intermediate temp allocation
    fn get_runtime_source(&self, helper: Helper, ctx: &TraverseCtx<'a>) -> Str<'a> {
        Str::from_strs_array_in([&self.module_name, "/helpers/", helper.name()], ctx)
    }

    fn transform_for_external_helper(helper: Helper, ctx: &mut TraverseCtx<'a>) -> Expression<'a> {
        let helper_var = static_ident!("babelHelpers");
        let symbol_id = ctx.scoping().find_binding(ctx.current_scope_id(), helper_var);
        let object = ctx.create_ident_expr(SPAN, helper_var, symbol_id, ReferenceFlags::Read);
        let property = IdentifierName::new(SPAN, helper.name(), ctx);
        Expression::new_static_member_expression(SPAN, object, property, false, ctx)
    }
}
