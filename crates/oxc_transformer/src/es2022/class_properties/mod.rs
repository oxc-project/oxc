//! ES2022: Class Properties
//!
//! This plugin transforms class properties to initializers inside class constructor.
//!
//! > This plugin is included in `preset-env`, in ES2022
//!
//! ## Example
//!
//! Input:
//! ```js
//! class C {
//!   foo = 123;
//!   #bar = 456;
//!   method() {
//!     let bar = this.#bar;
//!     this.#bar = bar + 1;
//!   }
//! }
//!
//! let x = 123;
//! class D extends S {
//!   foo = x;
//!   constructor(x) {
//!     if (x) {
//!       let s = super(x);
//!     } else {
//!       super(x);
//!     }
//!   }
//! }
//! ```
//!
//! Output:
//! ```js
//! var _bar = /*#__PURE__*/ new WeakMap();
//! class C {
//!   constructor() {
//!     babelHelpers.defineProperty(this, "foo", 123);
//!     babelHelpers.classPrivateFieldInitSpec(this, _bar, 456);
//!   }
//!   method() {
//!     let bar = babelHelpers.classPrivateFieldGet2(_bar, this);
//!     babelHelpers.classPrivateFieldSet2(_bar, this, bar + 1);
//!   }
//! }
//!
//! let x = 123;
//! class D extends S {
//!   constructor(_x) {
//!     if (_x) {
//!       let s = (super(_x), babelHelpers.defineProperty(this, "foo", x));
//!     } else {
//!       super(_x);
//!       babelHelpers.defineProperty(this, "foo", x);
//!     }
//!   }
//! }
//! ```
//!
//! ## Options
//!
//! ### `loose`
//!
//! This option can also be enabled with `CompilerAssumptions::set_public_class_fields`.
//!
//! When `true`, class properties are compiled to use an assignment expression instead of
//! `_defineProperty` helper.
//!
//! #### Example
//!
//! Input:
//! ```js
//! class C {
//!   foo = 123;
//! }
//! ```
//!
//! With `loose: false` (default):
//!
//! ```js
//! class C {
//!   constructor() {
//!     babelHelpers.defineProperty(this, "foo", 123);
//!   }
//! }
//! ```
//!
//! With `loose: true`:
//!
//! ```js
//! class C {
//!   constructor() {
//!     this.foo = 123;
//!   }
//! }
//! ```
//!
//! ## Implementation
//!
//! WORK IN PROGRESS. INCOMPLETE.
//!
//! ### Reference implementation
//!
//! Implementation based on [@babel/plugin-transform-class-properties](https://babel.dev/docs/babel-plugin-transform-class-properties).
//!
//! I (@overlookmotel) wrote this transform without reference to Babel's internal implementation,
//! but aiming to reproduce Babel's output, guided by Babel's test suite.
//!
//! ### Divergence from Babel
//!
//! In a few places, our implementation diverges from Babel, notably inserting property initializers
//! into constructor of a class with multiple `super()` calls (see comments in [`constructor`] module).
//!
//! ### High level overview
//!
//! Transform happens in 3 phases:
//!
//! 1. Check if class contains properties or static blocks, to determine if any transform is necessary
//!    (in [`ClassProperties::transform_class`]).
//! 2. Extract class property declarations and static blocks from class and insert in class constructor
//!    (instance properties) or before/after the class (static properties + static blocks)
//!    (in [`ClassProperties::transform_class`]).
//! 3. Transform private property usages (`this.#prop`)
//!    (in [`ClassProperties::transform_private_field_expression`] and other visitors).
//!
//! Implementation is split into several files:
//!
//! * `mod.rs`:                Setup and visitor.
//! * `class.rs`:              Transform of class body.
//! * `prop_decl.rs`:          Transform of property declarations (instance and static).
//! * `constructor.rs`:        Insertion of property initializers into class constructor.
//! * `instance_prop_init.rs`: Transform of instance property initializers.
//! * `static_prop_init.rs`:   Transform of static property initializers.
//! * `static_block.rs`:       Transform of static blocks.
//! * `computed_key.rs`:       Transform of property/method computed keys.
//! * `private_field.rs`:      Transform of private fields (`this.#prop`).
//! * `super.rs`:              Transform `super` expressions.
//! * `class_details.rs`:      Structures containing details of classes and private properties.
//! * `class_bindings.rs`:     Structure containing bindings for class name and temp var.
//! * `utils.rs`:              Utility functions.
//!
//! ## References
//!
//! * Babel plugin implementation:
//!   * <https://github.com/babel/babel/tree/v7.26.2/packages/babel-plugin-transform-class-properties>
//!   * <https://github.com/babel/babel/blob/v7.26.2/packages/babel-helper-create-class-features-plugin/src/index.ts>
//!   * <https://github.com/babel/babel/blob/v7.26.2/packages/babel-helper-create-class-features-plugin/src/fields.ts>
//! * Class properties TC39 proposal: <https://github.com/tc39/proposal-class-fields>

use indexmap::IndexMap;
use rustc_hash::{FxBuildHasher, FxHashMap};
use serde::Deserialize;

use oxc_allocator::{Address, GetAddress};
use oxc_ast::ast::*;
use oxc_data_structures::stack::NonEmptyStack;
use oxc_span::Atom;
use oxc_syntax::{scope::ScopeId, symbol::SymbolId};
use oxc_traverse::{Traverse, TraverseCtx};

use crate::TransformCtx;

mod class;
mod class_bindings;
mod class_details;
mod computed_key;
mod constructor;
mod instance_prop_init;
mod private_field;
mod prop_decl;
mod static_block;
mod static_prop_init;
mod supers;
mod utils;
use class_bindings::ClassBindings;
use class_details::{ClassDetails, ClassesStack, PrivateProp, ResolvedPrivateProp};

type FxIndexMap<K, V> = IndexMap<K, V, FxBuildHasher>;

#[derive(Debug, Default, Clone, Copy, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ClassPropertiesOptions {
    pub(crate) loose: bool,
}

/// Class properties transform.
///
/// See [module docs] for details.
///
/// [module docs]: self
pub struct ClassProperties<'a, 'ctx> {
    // Options
    //
    /// If `true`, set properties with `=`, instead of `_defineProperty` helper.
    set_public_class_fields: bool,
    /// If `true`, record private properties as string keys
    private_fields_as_properties: bool,
    /// If `true`, transform static blocks.
    transform_static_blocks: bool,

    ctx: &'ctx TransformCtx<'a>,

    // State during whole AST
    //
    /// Stack of classes.
    /// Pushed to when entering a class, popped when exiting.
    // TODO: The way stack is used is not perfect, because pushing to/popping from it in
    // `enter_expression` / `exit_expression`. If another transform replaces the class,
    // then stack will get out of sync.
    // TODO: Should push to the stack only when entering class body, because `#x` in class `extends`
    // clause resolves to `#x` in *outer* class, not the current class.
    classes_stack: ClassesStack<'a>,

    /// Addresses of class expressions being processed, to prevent same class being visited twice.
    /// Have to use a stack because the revisit doesn't necessarily happen straight after the first visit.
    /// e.g. `c = class C { [class D {}] = 1; }` -> `c = (_D = class D {}, class C { ... })`
    class_expression_addresses_stack: NonEmptyStack<Address>,

    // State during transform of class
    //
    /// `true` for class declaration, `false` for class expression
    is_declaration: bool,
    /// `true` if temp var for class has been inserted
    temp_var_is_created: bool,
    /// Scope that instance init initializers will be inserted into
    instance_inits_scope_id: ScopeId,
    /// `ScopeId` of class constructor, if instance init initializers will be inserted into constructor.
    /// Used for checking for variable name clashes.
    /// e.g. `let x; class C { prop = x; constructor(x) {} }` - `x` in constructor needs to be renamed
    instance_inits_constructor_scope_id: Option<ScopeId>,
    /// `SymbolId`s in constructor which clash with instance prop initializers
    clashing_constructor_symbols: FxHashMap<SymbolId, Atom<'a>>,
    /// Expressions to insert before class
    insert_before: Vec<Expression<'a>>,
    /// Expressions to insert after class expression
    insert_after_exprs: Vec<Expression<'a>>,
    /// Statements to insert after class declaration
    insert_after_stmts: Vec<Statement<'a>>,
}

impl<'a, 'ctx> ClassProperties<'a, 'ctx> {
    pub fn new(
        options: ClassPropertiesOptions,
        transform_static_blocks: bool,
        ctx: &'ctx TransformCtx<'a>,
    ) -> Self {
        // TODO: Raise error if these 2 options are inconsistent
        let set_public_class_fields = options.loose || ctx.assumptions.set_public_class_fields;
        // TODO: Raise error if these 2 options are inconsistent
        let private_fields_as_properties =
            options.loose || ctx.assumptions.private_fields_as_properties;

        Self {
            set_public_class_fields,
            private_fields_as_properties,
            transform_static_blocks,
            ctx,
            classes_stack: ClassesStack::default(),
            class_expression_addresses_stack: NonEmptyStack::new(Address::DUMMY),
            // Temporary values - overwritten when entering class
            is_declaration: false,
            temp_var_is_created: false,
            instance_inits_scope_id: ScopeId::new(0),
            instance_inits_constructor_scope_id: None,
            // `Vec`s and `FxHashMap`s which are reused for every class being transformed
            clashing_constructor_symbols: FxHashMap::default(),
            insert_before: vec![],
            insert_after_exprs: vec![],
            insert_after_stmts: vec![],
        }
    }
}

impl<'a, 'ctx> Traverse<'a> for ClassProperties<'a, 'ctx> {
    // `#[inline]` because this is a hot path
    #[inline]
    fn enter_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        // IMPORTANT: If add any other visitors here to handle private fields,
        // also need to add them to visitor in `static_prop.rs`.
        match expr {
            // `class {}`
            Expression::ClassExpression(_) => {
                self.transform_class_expression(expr, ctx);
            }
            // `object.#prop`
            Expression::PrivateFieldExpression(_) => {
                self.transform_private_field_expression(expr, ctx);
            }
            // `object.#prop()`
            Expression::CallExpression(_) => {
                self.transform_call_expression(expr, ctx);
            }
            // `object.#prop = value`, `object.#prop += value`, `object.#prop ??= value` etc
            Expression::AssignmentExpression(_) => {
                self.transform_assignment_expression(expr, ctx);
            }
            // `object.#prop++`, `--object.#prop`
            Expression::UpdateExpression(_) => {
                self.transform_update_expression(expr, ctx);
            }
            // `object?.#prop`
            Expression::ChainExpression(_) => {
                self.transform_chain_expression(expr, ctx);
            }
            // `delete object?.#prop.xyz`
            Expression::UnaryExpression(_) => {
                self.transform_unary_expression(expr, ctx);
            }
            // "object.#prop`xyz`"
            Expression::TaggedTemplateExpression(_) => {
                self.transform_tagged_template_expression(expr, ctx);
            }
            _ => {}
        }
    }

    // `#[inline]` because this is a hot path
    #[inline]
    fn enter_assignment_target(
        &mut self,
        target: &mut AssignmentTarget<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.transform_assignment_target(target, ctx);
    }

    // `#[inline]` because this is a hot path
    #[inline]
    fn enter_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        match stmt {
            // `class C {}`
            Statement::ClassDeclaration(class) => {
                let stmt_address = class.address();
                self.transform_class_declaration(class, stmt_address, ctx);
            }
            // `export class C {}`
            Statement::ExportNamedDeclaration(decl) => {
                let stmt_address = decl.address();
                if let Some(Declaration::ClassDeclaration(class)) = &mut decl.declaration {
                    self.transform_class_declaration(class, stmt_address, ctx);
                }
            }
            // `export default class {}`
            Statement::ExportDefaultDeclaration(decl) => {
                let stmt_address = decl.address();
                if let ExportDefaultDeclarationKind::ClassDeclaration(class) = &mut decl.declaration
                {
                    self.transform_class_declaration(class, stmt_address, ctx);
                }
            }
            _ => {}
        }
    }

    // `#[inline]` because `transform_class_on_exit` is so small
    #[inline]
    fn exit_class(&mut self, class: &mut Class<'a>, _ctx: &mut TraverseCtx<'a>) {
        self.transform_class_on_exit(class);
    }
}
