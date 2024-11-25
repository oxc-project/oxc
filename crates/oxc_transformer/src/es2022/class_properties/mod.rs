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
//! * `mod.rs`:         Setup, visitor and ancillary types.
//! * `class.rs`:       Transform of class body.
//! * `constructor.rs`: Insertion of property initializers into class constructor.
//! * `private.rs`:     Transform of private property usages (`this.#prop`).
//! * `utils.rs`:       Utility functions.
//!
//! ## References
//!
//! * Babel plugin implementation:
//!   * <https://github.com/babel/babel/tree/v7.26.2/packages/babel-plugin-transform-class-properties>
//!   * <https://github.com/babel/babel/blob/v7.26.2/packages/babel-helper-create-class-features-plugin/src/index.ts>
//!   * <https://github.com/babel/babel/blob/v7.26.2/packages/babel-helper-create-class-features-plugin/src/fields.ts>
//! * Class properties TC39 proposal: <https://github.com/tc39/proposal-class-fields>

use std::hash::BuildHasherDefault;

use indexmap::IndexMap;
use rustc_hash::FxHasher;
use serde::Deserialize;

use oxc_allocator::{Address, GetAddress};
use oxc_ast::ast::*;
use oxc_data_structures::stack::{NonEmptyStack, SparseStack};
use oxc_traverse::{BoundIdentifier, Traverse, TraverseCtx};

use crate::TransformCtx;

mod class;
mod constructor;
mod private;
mod utils;

type FxIndexMap<K, V> = IndexMap<K, V, BuildHasherDefault<FxHasher>>;

#[derive(Debug, Default, Clone, Copy, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ClassPropertiesOptions {
    #[serde(alias = "loose")]
    pub(crate) set_public_class_fields: bool,
}

/// Class properties transform.
///
/// See [module docs] for details.
///
/// [module docs]: self
pub struct ClassProperties<'a, 'ctx> {
    // Options
    set_public_class_fields: bool,
    static_block: bool,

    ctx: &'ctx TransformCtx<'a>,

    // State during whole AST
    /// Stack of private props.
    /// Pushed to when entering a class (`None` if class has no private props, `Some` if it does).
    /// Entries are a mapping from private prop name to binding for temp var.
    /// This is then used as lookup when transforming e.g. `this.#x`.
    // TODO: The way stack is used is not perfect, because pushing to/popping from it in
    // `enter_expression` / `exit_expression`. If another transform replaces the class,
    // then stack will get out of sync.
    // TODO: Should push to the stack only when entering class body, because `#x` in class `extends`
    // clause resolves to `#x` in *outer* class, not the current class.
    // TODO(improve-on-babel): Order that temp vars are created in is not important. Use `FxHashMap` instead.
    private_props_stack: SparseStack<PrivateProps<'a>>,
    /// Addresses of class expressions being processed, to prevent same class being visited twice.
    /// Have to use a stack because the revisit doesn't necessarily happen straight after the first visit.
    /// e.g. `c = class C { [class D {}] = 1; }` -> `c = (_D = class D {}, class C { ... })`
    class_expression_addresses_stack: NonEmptyStack<Address>,

    // State during transform of class
    /// `true` for class declaration, `false` for class expression
    is_declaration: bool,
    /// Var for class.
    /// e.g. `X` in `class X {}`.
    /// e.g. `_Class` in `_Class = class {}, _Class.x = 1, _Class`
    class_name: ClassName<'a>,
    /// Expressions to insert before class
    insert_before: Vec<Expression<'a>>,
    /// Expressions to insert after class expression
    insert_after_exprs: Vec<Expression<'a>>,
    /// Statements to insert after class declaration
    insert_after_stmts: Vec<Statement<'a>>,
}

/// Representation of binding for class name.
enum ClassName<'a> {
    /// Class has a name. This is the binding.
    Binding(BoundIdentifier<'a>),
    /// Class is anonymous.
    /// This is the name it would have if we need to set class name, in order to reference it.
    Name(&'a str),
}

/// Details of private properties for a class.
struct PrivateProps<'a> {
    /// Private properties for class. Indexed by property name.
    props: FxIndexMap<Atom<'a>, PrivateProp<'a>>,
    /// Binding for class name
    class_name_binding: Option<BoundIdentifier<'a>>,
    /// `true` for class declaration, `false` for class expression
    is_declaration: bool,
}

/// Details of a private property.
struct PrivateProp<'a> {
    binding: BoundIdentifier<'a>,
    is_static: bool,
}

impl<'a, 'ctx> ClassProperties<'a, 'ctx> {
    pub fn new(
        options: ClassPropertiesOptions,
        static_block: bool,
        ctx: &'ctx TransformCtx<'a>,
    ) -> Self {
        // TODO: Raise error if these 2 options are inconsistent
        let set_public_class_fields =
            options.set_public_class_fields || ctx.assumptions.set_public_class_fields;

        Self {
            set_public_class_fields,
            static_block,
            ctx,
            private_props_stack: SparseStack::new(),
            class_expression_addresses_stack: NonEmptyStack::new(Address::DUMMY),
            // Temporary values - overwritten when entering class
            is_declaration: false,
            class_name: ClassName::Name(""),
            // `Vec`s and `FxHashMap`s which are reused for every class being transformed
            insert_before: vec![],
            insert_after_exprs: vec![],
            insert_after_stmts: vec![],
        }
    }
}

impl<'a, 'ctx> Traverse<'a> for ClassProperties<'a, 'ctx> {
    fn enter_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        // Note: `delete this.#prop` is an early syntax error, so no need to handle transforming it
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
                // TODO: `transform_chain_expression` is no-op at present
                self.transform_chain_expression(expr, ctx);
            }
            // "object.#prop`xyz`"
            Expression::TaggedTemplateExpression(_) => {
                // TODO: `transform_tagged_template_expression` is no-op at present
                self.transform_tagged_template_expression(expr, ctx);
            }
            // TODO: `[object.#prop] = value`
            // TODO: `({x: object.#prop} = value)`
            _ => {}
        }
    }

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
                    self.transform_class_export_default(class, stmt_address, ctx);
                }
            }
            _ => {}
        }
    }

    fn exit_class(&mut self, class: &mut Class<'a>, _ctx: &mut TraverseCtx<'a>) {
        self.transform_class_on_exit(class);
    }
}
