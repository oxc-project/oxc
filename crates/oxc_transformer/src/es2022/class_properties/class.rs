//! ES2022: Class Properties
//! Transform of class itself.

use oxc_allocator::{Address, GetAddress};
use oxc_ast::{ast::*, NONE};
use oxc_span::SPAN;
use oxc_syntax::{
    node::NodeId,
    reference::ReferenceFlags,
    scope::ScopeFlags,
    symbol::{SymbolFlags, SymbolId},
};
use oxc_traverse::{BoundIdentifier, TraverseCtx};

use crate::common::helper_loader::Helper;

use super::{
    constructor::InstanceInitsInsertLocation,
    private_props::{PrivateProp, PrivateProps},
    utils::{create_assignment, create_variable_declaration, exprs_into_stmts},
    ClassBindings, ClassProperties, FxIndexMap,
};

impl<'a, 'ctx> ClassProperties<'a, 'ctx> {
    /// Transform class expression.
    // `#[inline]` so that compiler sees that `expr` is an `Expression::ClassExpression`.
    // Main guts of transform is broken out into `transform_class_expression_start` and
    // `transform_class_expression_finish` to keep this function as small as possible.
    // Want it to be inlined into `enter_expression` and for `enter_expression` to be inlined into parent.
    #[inline]
    pub(super) fn transform_class_expression(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let Expression::ClassExpression(class) = expr else { unreachable!() };

        let class_address = class.address();
        let expr_count = self.transform_class_expression_start(class, class_address, ctx);
        if expr_count > 0 {
            self.transform_class_expression_finish(expr, expr_count, ctx);
        }
    }

    fn transform_class_expression_start(
        &mut self,
        class: &mut Class<'a>,
        class_address: Address,
        ctx: &mut TraverseCtx<'a>,
    ) -> usize {
        // Check this class isn't being visited twice
        if *self.class_expression_addresses_stack.last() == class_address {
            // This class has already been transformed, and we're now encountering it again
            // in the sequence expression which was substituted for it. So don't transform it again!
            // Returning 0 tells `enter_expression` not to call `transform_class_expression_finish` either.
            self.class_expression_addresses_stack.pop();
            return 0;
        }

        self.is_declaration = false;

        self.transform_class(class, ctx);

        // Return number of expressions to be inserted before/after the class
        let mut expr_count = self.insert_before.len() + self.insert_after_exprs.len();

        let private_props = self.private_props_stack.last();
        if let Some(private_props) = private_props {
            expr_count += private_props.props.len();
        }

        if expr_count > 0 {
            // We're going to replace class expression with a sequence expression
            // `(..., _C = class C {}, ..., _C)`, so this class will be visited again.
            // Store the `Address` of class in stack. This will cause bail-out when we re-visit it.
            self.class_expression_addresses_stack.push(class_address);
        }

        expr_count
    }

    /// Insert expressions before/after the class.
    /// `C = class { [x()] = 1; static y = 2 };`
    /// -> `C = (_x = x(), _Class = class C { constructor() { this[_x] = 1; } }, _Class.y = 2, _Class)`
    fn transform_class_expression_finish(
        &mut self,
        expr: &mut Expression<'a>,
        mut expr_count: usize,
        ctx: &mut TraverseCtx<'a>,
    ) {
        // TODO: Name class if had no name, and name is statically knowable (as in example above).
        // If class name shadows var which is referenced within class, rename that var.
        // `var C = class { prop = C }; var C2 = C;`
        // -> `var _C = class C { constructor() { this.prop = _C; } }; var C2 = _C;`
        // This is really difficult as need to rename all references too to that binding too,
        // which can be very far above the class in AST, when it's a `var`.
        // Maybe for now only add class name if it doesn't shadow a var used within class?

        // TODO: Deduct static private props from `expr_count`.
        // Or maybe should store count and increment it when create private static props?
        // They're probably pretty rare, so it'll be rarely used.
        expr_count += 1 + usize::from(self.class_bindings.temp.is_some());

        let mut exprs = ctx.ast.vec_with_capacity(expr_count);

        // Insert `_prop = new WeakMap()` expressions for private instance props
        // (or `_prop = _classPrivateFieldLooseKey("prop")` if loose mode).
        // Babel has these always go first, regardless of order of class elements.
        // Also insert `var _prop;` temp var declarations for private static props.
        let private_props = self.private_props_stack.last();
        if let Some(private_props) = private_props {
            // Insert `var _prop;` declarations here rather than when binding was created to maintain
            // same order of `var` declarations as Babel.
            // `c = class C { #x = 1; static y = 2; }` -> `var _C, _x;`
            // TODO(improve-on-babel): Simplify this.
            if self.private_fields_as_properties {
                exprs.extend(private_props.props.iter().map(|(name, prop)| {
                    // Insert `var _prop;` declaration
                    self.ctx.var_declarations.insert_var(&prop.binding, ctx);

                    // `_prop = _classPrivateFieldLooseKey("prop")`
                    let value = self.create_private_prop_key_loose(name, ctx);
                    create_assignment(&prop.binding, value, ctx)
                }));
            } else {
                let mut weakmap_symbol_id = None;
                exprs.extend(private_props.props.values().filter_map(|prop| {
                    // Insert `var _prop;` declaration
                    self.ctx.var_declarations.insert_var(&prop.binding, ctx);

                    if prop.is_static {
                        return None;
                    }

                    // `_prop = new WeakMap()`
                    let value = create_new_weakmap(&mut weakmap_symbol_id, ctx);
                    Some(create_assignment(&prop.binding, value, ctx))
                }));
            }
        }

        // Insert computed key initializers
        exprs.extend(self.insert_before.drain(..));

        // Insert class + static property assignments + static blocks
        let class_expr = ctx.ast.move_expression(expr);
        if let Some(binding) = &self.class_bindings.temp {
            // Insert `var _Class` statement, if it wasn't already in `transform_class`
            if !self.temp_var_is_created {
                self.ctx.var_declarations.insert_var(binding, ctx);
            }

            // `_Class = class {}`
            let assignment = create_assignment(binding, class_expr, ctx);
            exprs.push(assignment);
            // Add static property assignments + static blocks
            exprs.extend(self.insert_after_exprs.drain(..));
            // `_Class`
            exprs.push(binding.create_read_expression(ctx));
        } else {
            // Add static blocks (which didn't reference class name)
            // TODO: If class has `extends` clause, and it may have side effects, then static block contents
            // goes after class expression, and temp var is called `_temp` not `_Class`.
            // `let C = class extends Unbound { static { x = 1; } };`
            // -> `var _temp; let C = ((_temp = class C extends Unbound {}), (x = 1), _temp);`
            // `let C = class extends Bound { static { x = 1; } };`
            // -> `let C = ((x = 1), class C extends Bound {});`
            exprs.extend(self.insert_after_exprs.drain(..));

            exprs.push(class_expr);
        }

        *expr = ctx.ast.expression_sequence(SPAN, exprs);
    }

    /// Transform class declaration.
    pub(super) fn transform_class_declaration(
        &mut self,
        class: &mut Class<'a>,
        stmt_address: Address,
        ctx: &mut TraverseCtx<'a>,
    ) {
        // Ignore TS class declarations
        // TODO: Is this correct?
        // TODO: If remove this check, remove from `transform_class_on_exit` too.
        if class.declare {
            return;
        }

        self.is_declaration = true;

        self.transform_class(class, ctx);

        // TODO: Run other transforms on inserted statements. How?

        if let Some(temp_binding) = &self.class_bindings.temp {
            // Binding for class name is required
            if let Some(ident) = &class.id {
                // Insert `var _Class` statement, if it wasn't already in `transform_class`
                if !self.temp_var_is_created {
                    self.ctx.var_declarations.insert_var(temp_binding, ctx);
                }

                // Insert `_Class = Class` after class.
                // TODO(improve-on-babel): Could just insert `var _Class = Class;` after class,
                // rather than separate `var _Class` declaration.
                let class_name =
                    BoundIdentifier::from_binding_ident(ident).create_read_expression(ctx);
                let expr = create_assignment(temp_binding, class_name, ctx);
                let stmt = ctx.ast.statement_expression(SPAN, expr);
                self.insert_after_stmts.insert(0, stmt);
            } else {
                // Class must be default export `export default class {}`, as all other class declarations
                // always have a name. Set class name.
                *ctx.symbols_mut().get_flags_mut(temp_binding.symbol_id) = SymbolFlags::Class;
                class.id = Some(temp_binding.create_binding_identifier(ctx));
            }
        }

        // Insert expressions before/after class
        if !self.insert_before.is_empty() {
            self.ctx.statement_injector.insert_many_before(
                &stmt_address,
                exprs_into_stmts(self.insert_before.drain(..), ctx),
            );
        }

        if let Some(private_props) = self.private_props_stack.last() {
            if self.private_fields_as_properties {
                self.ctx.statement_injector.insert_many_before(
                    &stmt_address,
                    private_props.props.iter().map(|(name, prop)| {
                        // `var _prop = _classPrivateFieldLooseKey("prop");`
                        let value = self.create_private_prop_key_loose(name, ctx);
                        create_variable_declaration(&prop.binding, value, ctx)
                    }),
                );
            } else {
                // TODO: Only call `insert_many_before` if some private *instance* props
                let mut weakmap_symbol_id = None;
                self.ctx.statement_injector.insert_many_before(
                    &stmt_address,
                    private_props.props.values().filter_map(|prop| {
                        if prop.is_static {
                            return None;
                        }

                        // `var _prop = new WeakMap();`
                        let value = create_new_weakmap(&mut weakmap_symbol_id, ctx);
                        Some(create_variable_declaration(&prop.binding, value, ctx))
                    }),
                );
            }
        }

        if !self.insert_after_stmts.is_empty() {
            self.ctx
                .statement_injector
                .insert_many_after(&stmt_address, self.insert_after_stmts.drain(..));
        }
    }

    /// `_classPrivateFieldLooseKey("prop")`
    fn create_private_prop_key_loose(
        &self,
        name: &Atom<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        self.ctx.helper_call_expr(
            Helper::ClassPrivateFieldLooseKey,
            SPAN,
            ctx.ast.vec1(Argument::from(ctx.ast.expression_string_literal(
                SPAN,
                name.clone(),
                None,
            ))),
            ctx,
        )
    }

    /// Main guts of the transform.
    fn transform_class(&mut self, class: &mut Class<'a>, ctx: &mut TraverseCtx<'a>) {
        // TODO(improve-on-babel): If outer scope is sloppy mode, all code which is moved to outside
        // the class should be wrapped in an IIFE with `'use strict'` directive. Babel doesn't do this.

        // TODO: If static blocks transform is disabled, it's possible to get incorrect execution order.
        // ```js
        // class C {
        //     static x = console.log('x');
        //     static {
        //         console.log('block');
        //     }
        //     static y = console.log('y');
        // }
        // ```
        // This logs "x", "block", "y". But in transformed output it'd be "block", "x", "y".
        // Maybe force transform of static blocks if any static properties?
        // Or alternatively could insert static property initializers into static blocks.

        // Check if class has any properties and get index of constructor (if class has one)
        let mut instance_prop_count = 0;
        let mut has_static_prop = false;
        let mut has_static_block = false;
        // TODO: Store `FxIndexMap`s in a pool and re-use them
        let mut private_props = FxIndexMap::default();
        let mut constructor = None;
        for element in class.body.body.iter_mut() {
            match element {
                ClassElement::PropertyDefinition(prop) => {
                    // TODO: Throw error if property has decorators

                    // Create binding for private property key
                    if let PropertyKey::PrivateIdentifier(ident) = &prop.key {
                        // Note: Current scope is outside class.
                        let binding = ctx.generate_uid_in_current_hoist_scope(&ident.name);
                        private_props.insert(
                            ident.name.clone(),
                            PrivateProp { binding, is_static: prop.r#static },
                        );
                    }

                    if prop.r#static {
                        has_static_prop = true;
                    } else {
                        instance_prop_count += 1;
                    }

                    continue;
                }
                ClassElement::StaticBlock(_) => {
                    // Static block only necessitates transforming class if it's being transformed
                    if self.transform_static_blocks {
                        has_static_block = true;
                        continue;
                    }
                }
                ClassElement::MethodDefinition(method) => {
                    if method.kind == MethodDefinitionKind::Constructor
                        && method.value.body.is_some()
                    {
                        constructor = Some(method);
                    }
                }
                ClassElement::AccessorProperty(_) | ClassElement::TSIndexSignature(_) => {
                    // TODO: Need to handle these?
                }
            }
        }

        // Exit if nothing to transform
        if instance_prop_count == 0 && !has_static_prop && !has_static_block {
            self.private_props_stack.push(None);
            return;
        }

        // Initialize class binding vars.
        // Static prop in class expression or anonymous `export default class {}` always require
        // temp var for class. Static prop in class declaration doesn't.
        let mut class_name_binding = class.id.as_ref().map(BoundIdentifier::from_binding_ident);

        let need_temp_var = has_static_prop && (!self.is_declaration || class.id.is_none());
        self.temp_var_is_created = need_temp_var;

        let class_temp_binding = if need_temp_var {
            let temp_binding = ClassBindings::create_temp_binding(class_name_binding.as_ref(), ctx);
            if self.is_declaration {
                // Anonymous `export default class {}`. Set class name binding to temp var.
                // Actual class name will be set to this later.
                class_name_binding = Some(temp_binding.clone());
            } else {
                // Create temp var `var _Class;` statement.
                // TODO(improve-on-babel): Inserting the temp var `var _Class` statement here is only
                // to match Babel's output. It'd be simpler just to insert it at the end and get rid of
                // `temp_var_is_created` that tracks whether it's done already or not.
                self.ctx.var_declarations.insert_var(&temp_binding, ctx);
            }
            Some(temp_binding)
        } else {
            None
        };

        self.class_bindings = ClassBindings::new(class_name_binding, class_temp_binding);

        // Add entry to `private_props_stack`
        if private_props.is_empty() {
            self.private_props_stack.push(None);
        } else {
            // `class_bindings.temp` in the `PrivateProps` entry is the temp var (if one has been created).
            // Private fields in static prop initializers use the temp var in the transpiled output
            // e.g. `_assertClassBrand(_Class, obj, _prop)`.
            // At end of this function, if it's a class declaration, we set `class_bindings.temp` to be
            // the binding for the class name, for when the class body is visited, because in the class
            // body, private fields use the class name
            // e.g. `_assertClassBrand(Class, obj, _prop)` (note `Class` not `_Class`).
            self.private_props_stack.push(Some(PrivateProps {
                props: private_props,
                class_bindings: self.class_bindings.clone(),
                is_declaration: self.is_declaration,
            }));
        }

        // Determine where to insert instance property initializers in constructor
        let instance_inits_insert_location = if instance_prop_count == 0 {
            // No instance prop initializers to insert
            None
        } else if let Some(constructor) = constructor {
            // Existing constructor
            let constructor = constructor.value.as_mut();
            if class.super_class.is_some() {
                let (insert_scopes, insert_location) =
                    Self::replace_super_in_constructor(constructor, ctx);
                self.instance_inits_scope_id = insert_scopes.insert_in_scope_id;
                self.instance_inits_constructor_scope_id = insert_scopes.constructor_scope_id;
                Some(insert_location)
            } else {
                let constructor_scope_id = constructor.scope_id();
                self.instance_inits_scope_id = constructor_scope_id;
                // Only record `constructor_scope_id` if constructor's scope has some bindings.
                // If it doesn't, no need to check for shadowed symbols in instance prop initializers,
                // because no bindings to clash with.
                self.instance_inits_constructor_scope_id =
                    if ctx.scopes().get_bindings(constructor_scope_id).is_empty() {
                        None
                    } else {
                        Some(constructor_scope_id)
                    };
                Some(InstanceInitsInsertLocation::ExistingConstructor(0))
            }
        } else {
            // No existing constructor - create scope for one
            let constructor_scope_id = ctx.scopes_mut().add_scope(
                Some(class.scope_id()),
                NodeId::DUMMY,
                ScopeFlags::Function | ScopeFlags::Constructor | ScopeFlags::StrictMode,
            );
            self.instance_inits_scope_id = constructor_scope_id;
            self.instance_inits_constructor_scope_id = None;
            Some(InstanceInitsInsertLocation::NewConstructor)
        };

        // Extract properties and static blocks from class body + substitute computed method keys
        let mut instance_inits = Vec::with_capacity(instance_prop_count);
        let mut constructor_index = 0;
        let mut index_not_including_removed = 0;
        class.body.body.retain_mut(|element| {
            match element {
                ClassElement::PropertyDefinition(prop) => {
                    if prop.r#static {
                        self.convert_static_property(prop, ctx);
                    } else {
                        self.convert_instance_property(prop, &mut instance_inits, ctx);
                    }
                    return false;
                }
                ClassElement::StaticBlock(block) => {
                    if self.transform_static_blocks {
                        self.convert_static_block(block, ctx);
                        return false;
                    }
                }
                ClassElement::MethodDefinition(method) => {
                    if method.kind == MethodDefinitionKind::Constructor {
                        if method.value.body.is_some() {
                            constructor_index = index_not_including_removed;
                        }
                    } else {
                        self.substitute_temp_var_for_method_computed_key(method, ctx);
                    }
                }
                ClassElement::AccessorProperty(_) | ClassElement::TSIndexSignature(_) => {
                    // TODO: Need to handle these?
                }
            }

            index_not_including_removed += 1;

            true
        });

        // Insert instance initializers into constructor, or create constructor if there is none
        if let Some(instance_inits_insert_location) = instance_inits_insert_location {
            self.insert_instance_inits(
                class,
                instance_inits,
                &instance_inits_insert_location,
                self.instance_inits_scope_id,
                constructor_index,
                ctx,
            );
        }

        // Update class bindings prior to traversing class body and insertion of statements/expressions
        // before/after the class. See comments on `ClassBindings`.
        if let Some(private_props) = self.private_props_stack.last_mut() {
            // Transfer state of `temp` binding from `private_props_stack` to `self`.
            // A temp binding may have been created while transpiling private fields in
            // static prop initializers.
            // TODO: Do this where `class_bindings.temp` is consumed instead?
            if let Some(temp_binding) = &private_props.class_bindings.temp {
                self.class_bindings.temp = Some(temp_binding.clone());
            }

            // Static private fields reference class name (not temp var) in class declarations.
            // `class Class { static #prop; method() { return obj.#prop; } }`
            // -> `method() { return _assertClassBrand(Class, obj, _prop)._; }`
            // (note `Class` in `_assertClassBrand(Class, ...)`, not `_Class`)
            // So set "temp" binding to actual class name while visiting class body.
            // Note: If declaration is `export default class {}` with no name, and class has static props,
            // then class has had name binding created already above. So name binding is always `Some`.
            if self.is_declaration {
                private_props.class_bindings.temp = private_props.class_bindings.name.clone();
            }
        }
    }

    /// Pop from private props stack.
    // `#[inline]` because this is function is so small
    #[inline]
    pub(super) fn transform_class_on_exit(&mut self, class: &Class) {
        // Ignore TS class declarations
        // TODO: Is this correct?
        if class.declare {
            return;
        }

        self.private_props_stack.pop();
    }

    /// Insert an expression after the class.
    pub(super) fn insert_expr_after_class(
        &mut self,
        expr: Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if self.is_declaration {
            self.insert_after_stmts.push(ctx.ast.statement_expression(SPAN, expr));
        } else {
            self.insert_after_exprs.push(expr);
        }
    }
}

/// Create `new WeakMap()` expression.
///
/// Takes an `&mut Option<Option<SymbolId>>` which is updated after looking up the binding for `WeakMap`.
/// * `None` = Not looked up yet.
/// * `Some(None)` = Has been looked up, and `WeakMap` is unbound.
/// * `Some(Some(symbol_id))` = Has been looked up, and `WeakMap` has a local binding.
///
/// This is an optimization to avoid looking up the symbol for `WeakMap` over and over when defining
/// multiple private properties.
#[expect(clippy::option_option)]
fn create_new_weakmap<'a>(
    symbol_id: &mut Option<Option<SymbolId>>,
    ctx: &mut TraverseCtx<'a>,
) -> Expression<'a> {
    let symbol_id = *symbol_id
        .get_or_insert_with(|| ctx.scopes().find_binding(ctx.current_scope_id(), "WeakMap"));
    let ident = ctx.create_ident_expr(SPAN, Atom::from("WeakMap"), symbol_id, ReferenceFlags::Read);
    ctx.ast.expression_new(SPAN, ident, ctx.ast.vec(), NONE)
}
