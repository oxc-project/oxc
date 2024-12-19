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
use oxc_traverse::{Ancestor, BoundIdentifier, TraverseCtx};

use crate::{common::helper_loader::Helper, TransformCtx};

use super::{
    constructor::InstanceInitsInsertLocation,
    utils::{create_assignment, create_variable_declaration, exprs_into_stmts},
    ClassBindings, ClassDetails, ClassProperties, FxIndexMap, PrivateProp,
};

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

impl<'a, 'ctx> ClassProperties<'a, 'ctx> {
    pub(super) fn transform_class_body_on_enter(
        &mut self,
        body: &mut ClassBody<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        // Ignore TS class declarations
        // TODO: Is this correct?
        let Ancestor::ClassBody(class) = ctx.parent() else { unreachable!() };
        if *class.declare() {
            return;
        }

        // Get details about class
        let mut class_name_binding = class.id().as_ref().map(BoundIdentifier::from_binding_ident);
        let class_scope_id = class.scope_id().get().unwrap();
        let has_super_class = class.super_class().is_some();

        // Check if class has any properties and get index of constructor (if class has one)
        let mut instance_prop_count = 0;
        let mut has_static_prop = false;
        let mut has_static_block = false;
        // TODO: Store `FxIndexMap`s in a pool and re-use them
        let mut private_props = FxIndexMap::default();
        let mut constructor = None;
        for element in body.body.iter_mut() {
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

        // Determine if is a class declaration
        let is_declaration = match ctx.ancestor(1) {
            Ancestor::ExportDefaultDeclarationDeclaration(_)
            | Ancestor::ExportNamedDeclarationDeclaration(_) => true,
            grandparent => grandparent.is_via_statement(),
        };

        // Exit if nothing to transform
        if instance_prop_count == 0 && !has_static_prop && !has_static_block {
            self.classes_stack.push(ClassDetails {
                is_declaration,
                transform_required: false,
                private_props: None,
                bindings: ClassBindings::default(),
            });
            return;
        }

        // Initialize class binding vars.
        // Static prop in class expression or anonymous `export default class {}` always require
        // temp var for class. Static prop in class declaration doesn't.
        let need_temp_var = has_static_prop && (!is_declaration || class_name_binding.is_none());

        let outer_hoist_scope_id = ctx.current_hoist_scope_id();
        let class_temp_binding = if need_temp_var {
            let temp_binding = ClassBindings::create_temp_binding(
                class_name_binding.as_ref(),
                outer_hoist_scope_id,
                ctx,
            );
            if is_declaration {
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

        let static_private_fields_use_temp = !is_declaration;
        let class_bindings = ClassBindings::new(
            class_name_binding,
            class_temp_binding,
            outer_hoist_scope_id,
            static_private_fields_use_temp,
            need_temp_var,
        );

        // Add entry to `classes_stack`
        self.classes_stack.push(ClassDetails {
            is_declaration,
            transform_required: true,
            private_props: if private_props.is_empty() { None } else { Some(private_props) },
            bindings: class_bindings,
        });

        // Exit if no instance properties (public or private)
        if instance_prop_count == 0 {
            return;
        }

        // Determine where to insert instance property initializers in constructor
        let instance_inits_insert_location = if let Some(constructor) = constructor {
            // Existing constructor
            let constructor = constructor.value.as_mut();
            if has_super_class {
                let (insert_scopes, insert_location) =
                    Self::replace_super_in_constructor(constructor, ctx);
                self.instance_inits_scope_id = insert_scopes.insert_in_scope_id;
                self.instance_inits_constructor_scope_id = insert_scopes.constructor_scope_id;
                insert_location
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
                InstanceInitsInsertLocation::ExistingConstructor(0)
            }
        } else {
            // No existing constructor - create scope for one
            let constructor_scope_id = ctx.scopes_mut().add_scope(
                Some(class_scope_id),
                NodeId::DUMMY,
                ScopeFlags::Function | ScopeFlags::Constructor | ScopeFlags::StrictMode,
            );
            self.instance_inits_scope_id = constructor_scope_id;
            self.instance_inits_constructor_scope_id = None;
            InstanceInitsInsertLocation::NewConstructor
        };

        // Extract instance properties initializers
        let mut instance_inits = Vec::with_capacity(instance_prop_count);
        let mut constructor = None;
        for element in body.body.iter_mut() {
            #[expect(clippy::match_same_arms)]
            match element {
                ClassElement::PropertyDefinition(prop) => {
                    if !prop.r#static {
                        self.convert_instance_property(prop, &mut instance_inits, ctx);
                    }
                }
                ClassElement::MethodDefinition(method) => {
                    if method.kind == MethodDefinitionKind::Constructor
                        && method.value.body.is_some()
                    {
                        constructor = Some(method.value.as_mut());
                    }
                }
                ClassElement::AccessorProperty(_) | ClassElement::TSIndexSignature(_) => {
                    // TODO: Need to handle these?
                }
                ClassElement::StaticBlock(_) => {}
            }
        }

        // Insert instance initializers into constructor, or create constructor if there is none
        match instance_inits_insert_location {
            InstanceInitsInsertLocation::NewConstructor => {
                self.insert_constructor(body, instance_inits, has_super_class, ctx);
            }
            InstanceInitsInsertLocation::ExistingConstructor(stmt_index) => {
                self.insert_inits_into_constructor_as_statements(
                    constructor.as_mut().unwrap(),
                    instance_inits,
                    stmt_index,
                    ctx,
                );
            }
            InstanceInitsInsertLocation::SuperFnInsideConstructor(super_binding) => {
                self.create_super_function_inside_constructor(
                    constructor.as_mut().unwrap(),
                    instance_inits,
                    &super_binding,
                    ctx,
                );
            }
            InstanceInitsInsertLocation::SuperFnOutsideClass(super_binding) => {
                self.create_super_function_outside_constructor(instance_inits, &super_binding, ctx);
            }
        }
    }

    /// Transform class declaration on exit.
    pub(super) fn transform_class_on_exit(
        &mut self,
        class: &mut Class<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        // Ignore TS class declarations
        // TODO: Is this correct?
        if class.declare {
            return;
        }

        // Deal with exit from class expression in `transform_class_expression_on_exit`
        let class_details = self.current_class_mut();
        if !class_details.is_declaration {
            return;
        }

        // TODO: Refactor
        if !class_details.transform_required {
            debug_assert!(class_details.bindings.temp.is_none());
            self.classes_stack.pop();
            return;
        }

        // TODO: Comment explaining why
        class_details.bindings.static_private_fields_use_temp = true;

        // Transform static properties, remove static and instance properties, and move computed keys
        // to before class
        self.transform_class_on_exit_impl(class, ctx);

        // Insert temp var for class if required. Name class if required.
        let class_details = self.classes_stack.last_mut();
        if let Some(temp_binding) = &class_details.bindings.temp {
            // Binding for class name is required
            if let Some(ident) = &class.id {
                // Insert `var _Class` statement, if it wasn't already in `transform_class`
                if !class_details.bindings.temp_var_is_created {
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

        // Insert statements before/after class
        let stmt_address = match ctx.parent() {
            parent @ (Ancestor::ExportDefaultDeclarationDeclaration(_)
            | Ancestor::ExportNamedDeclarationDeclaration(_)) => parent.address(),
            // `Class` is always stored in a `Box`, so has a stable memory location
            _ => Address::from_ptr(class),
        };

        if !self.insert_before.is_empty() {
            self.ctx.statement_injector.insert_many_before(
                &stmt_address,
                exprs_into_stmts(self.insert_before.drain(..), ctx),
            );
        }

        if let Some(private_props) = &class_details.private_props {
            if self.private_fields_as_properties {
                self.ctx.statement_injector.insert_many_before(
                    &stmt_address,
                    private_props.iter().map(|(name, prop)| {
                        // `var _prop = _classPrivateFieldLooseKey("prop");`
                        let value = Self::create_private_prop_key_loose(name, self.ctx, ctx);
                        create_variable_declaration(&prop.binding, value, ctx)
                    }),
                );
            } else {
                // TODO: Only call `insert_many_before` if some private *instance* props
                let mut weakmap_symbol_id = None;
                self.ctx.statement_injector.insert_many_before(
                    &stmt_address,
                    private_props.values().filter_map(|prop| {
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

        // Exit class
        self.classes_stack.pop();
    }

    /// Transform class expression on exit.
    pub(super) fn transform_class_expression_on_exit(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let Expression::ClassExpression(class) = expr else { unreachable!() };

        // Ignore TS class declarations
        // TODO: Is this correct?
        if class.declare {
            return;
        }

        let class_details = self.current_class();
        if !class_details.transform_required {
            debug_assert!(class_details.bindings.temp.is_none());
            self.classes_stack.pop();
            return;
        }

        // Transform static properties, remove static and instance properties, and move computed keys
        // to before class
        self.transform_class_on_exit_impl(class, ctx);

        // Insert expressions before / after class.
        // `C = class { [x()] = 1; static y = 2 };`
        // -> `C = (_x = x(), _Class = class C { constructor() { this[_x] = 1; } }, _Class.y = 2, _Class)`

        // TODO: Name class if had no name, and name is statically knowable (as in example above).
        // If class name shadows var which is referenced within class, rename that var.
        // `var C = class { prop = C }; var C2 = C;`
        // -> `var _C = class C { constructor() { this.prop = _C; } }; var C2 = _C;`
        // This is really difficult as need to rename all references to that binding too,
        // which can be very far above the class in AST, when it's a `var`.
        // Maybe for now only add class name if it doesn't shadow a var used within class?

        // TODO: Deduct static private props from `expr_count`.
        // Or maybe should store count and increment it when create private static props?
        // They're probably pretty rare, so it'll be rarely used.
        let class_details = self.classes_stack.last();

        let mut expr_count = self.insert_before.len() + self.insert_after_exprs.len();
        if let Some(private_props) = &class_details.private_props {
            expr_count += private_props.len();
        }

        // TODO: Refactor
        if expr_count == 0 {
            debug_assert!(class_details.bindings.temp.is_none());
            self.classes_stack.pop();
            return;
        }

        expr_count += 1 + usize::from(class_details.bindings.temp.is_some());

        let mut exprs = ctx.ast.vec_with_capacity(expr_count);

        // Insert `_prop = new WeakMap()` expressions for private instance props
        // (or `_prop = _classPrivateFieldLooseKey("prop")` if loose mode).
        // Babel has these always go first, regardless of order of class elements.
        // Also insert `var _prop;` temp var declarations for private static props.
        if let Some(private_props) = &class_details.private_props {
            // Insert `var _prop;` declarations here rather than when binding was created to maintain
            // same order of `var` declarations as Babel.
            // `c = class C { #x = 1; static y = 2; }` -> `var _C, _x;`
            // TODO(improve-on-babel): Simplify this.
            if self.private_fields_as_properties {
                exprs.extend(private_props.iter().map(|(name, prop)| {
                    // Insert `var _prop;` declaration
                    self.ctx.var_declarations.insert_var(&prop.binding, ctx);

                    // `_prop = _classPrivateFieldLooseKey("prop")`
                    let value = Self::create_private_prop_key_loose(name, self.ctx, ctx);
                    create_assignment(&prop.binding, value, ctx)
                }));
            } else {
                let mut weakmap_symbol_id = None;
                exprs.extend(private_props.values().filter_map(|prop| {
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
        if let Some(binding) = &class_details.bindings.temp {
            // Insert `var _Class` statement, if it wasn't already in `transform_class`
            if !class_details.bindings.temp_var_is_created {
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

        debug_assert!(exprs.len() <= expr_count);

        *expr = ctx.ast.expression_sequence(SPAN, exprs);

        // Exit class
        self.classes_stack.pop();
    }

    // TODO: Rename this method
    fn transform_class_on_exit_impl(&mut self, class: &mut Class<'a>, ctx: &mut TraverseCtx<'a>) {
        class.body.body.retain_mut(|element| {
            match element {
                ClassElement::PropertyDefinition(prop) => {
                    if prop.r#static {
                        self.convert_static_property(prop, ctx);
                    } else if prop.computed {
                        self.extract_instance_prop_computed_key(prop, ctx);
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
                    self.substitute_temp_var_for_method_computed_key(method, ctx);
                }
                ClassElement::AccessorProperty(_) | ClassElement::TSIndexSignature(_) => {
                    // TODO: Need to handle these?
                }
            }

            true
        });
    }

    // TODO: Move these 2 methods to another file?
    // TODO: Update comment below

    /// Flag that static private fields should be transpiled using temp binding,
    /// while in this static property or static block.
    ///
    /// Static private fields reference class name (not temp var) in class declarations.
    /// `class Class { static #prop; method() { return obj.#prop; } }`
    /// -> `method() { return _assertClassBrand(Class, obj, _prop)._; }`
    /// (note `Class` in `_assertClassBrand(Class, ...)`, not `_Class`)
    ///
    /// Also see comments on `ClassBindings`.
    ///
    /// Note: If declaration is `export default class {}` with no name, and class has static props,
    /// then class has had name binding created already in `transform_class`.
    /// So name binding is always `Some`.
    pub(super) fn flag_entering_static_prop_or_block(&mut self) {
        // No need to check if class is a declaration, because `static_private_fields_use_temp`
        // is always `true` for expressions anyway
        self.current_class_mut().bindings.static_private_fields_use_temp = true;
    }

    pub(super) fn flag_exiting_static_prop_or_block(&mut self) {
        // Flag that transpiled static private props use name binding in class declarations
        let class_details = self.current_class_mut();
        if class_details.is_declaration {
            class_details.bindings.static_private_fields_use_temp = false;
        }
    }

    /// `_classPrivateFieldLooseKey("prop")`
    fn create_private_prop_key_loose(
        name: &Atom<'a>,
        transform_ctx: &TransformCtx<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        transform_ctx.helper_call_expr(
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

    /// Insert an expression after the class.
    pub(super) fn insert_expr_after_class(
        &mut self,
        expr: Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if self.current_class().is_declaration {
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
