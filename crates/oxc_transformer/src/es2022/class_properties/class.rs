//! ES2022: Class Properties
//! Transform of class itself.

use oxc_allocator::{Address, GetAddress};
use oxc_ast::{ast::*, NONE};
use oxc_span::SPAN;
use oxc_syntax::{
    reference::ReferenceFlags,
    symbol::{SymbolFlags, SymbolId},
};
use oxc_traverse::{BoundIdentifier, TraverseCtx};

use crate::common::helper_loader::Helper;

use super::{super::ClassStaticBlock, ClassBindings};
use super::{
    private_props::{PrivateProp, PrivateProps},
    utils::{
        create_assignment, create_underscore_ident_name, create_variable_declaration,
        exprs_into_stmts,
    },
    ClassProperties, FxIndexMap,
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

        // Insert `_prop = new WeakMap()` expressions for private instance props.
        // Babel has these always go first, regardless of order of class elements.
        // Also insert `var _prop;` temp var declarations for private static props.
        let private_props = self.private_props_stack.last();
        if let Some(private_props) = private_props {
            let mut weakmap_symbol_id = None;
            exprs.extend(private_props.props.values().filter_map(|prop| {
                // Insert `var _prop;` declaration.
                // Do it here rather than when binding was created to maintain same order of `var`
                // declarations as Babel. `c = class C { #x = 1; static y = 2; }` -> `var _C, _x;`
                self.ctx.var_declarations.insert_var(&prop.binding, None, ctx);

                if prop.is_static {
                    return None;
                }

                // `_prop = new WeakMap()`
                Some(create_assignment(
                    &prop.binding,
                    create_new_weakmap(&mut weakmap_symbol_id, ctx),
                    ctx,
                ))
            }));
        }

        // Insert computed key initializers
        exprs.extend(self.insert_before.drain(..));

        // Insert class + static property assignments + static blocks
        let class_expr = ctx.ast.move_expression(expr);
        if let Some(binding) = &self.class_bindings.temp {
            // Insert `var _Class` statement, if it wasn't already in `transform_class`
            if !self.temp_var_is_created {
                self.ctx.var_declarations.insert_var(binding, None, ctx);
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

        self.transform_class_declaration_impl(class, stmt_address, ctx);
    }

    /// Transform `export default class {}`.
    ///
    /// Separate function as this is only circumstance where have to deal with anonymous class declaration,
    /// and it's an uncommon case (can only be 1 per file).
    // TODO: This method is now defunct. Can just have 1 `transform_class_declaration` function.
    pub(super) fn transform_class_export_default(
        &mut self,
        class: &mut Class<'a>,
        stmt_address: Address,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.transform_class_declaration_impl(class, stmt_address, ctx);
    }

    fn transform_class_declaration_impl(
        &mut self,
        class: &mut Class<'a>,
        stmt_address: Address,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.is_declaration = true;

        self.transform_class(class, ctx);

        // TODO: Run other transforms on inserted statements. How?

        if let Some(temp_binding) = &self.class_bindings.temp {
            // Binding for class name is required
            if let Some(ident) = &class.id {
                // Insert `var _Class` statement, if it wasn't already in `transform_class`
                if !self.temp_var_is_created {
                    self.ctx.var_declarations.insert_var(temp_binding, None, ctx);
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
            // TODO: Only call `insert_many_before` if some private *instance* props
            let mut weakmap_symbol_id = None;
            self.ctx.statement_injector.insert_many_before(
                &stmt_address,
                private_props.props.values().filter_map(|prop| {
                    if prop.is_static {
                        return None;
                    }

                    // `var _prop = new WeakMap()`
                    Some(create_variable_declaration(
                        &prop.binding,
                        create_new_weakmap(&mut weakmap_symbol_id, ctx),
                        ctx,
                    ))
                }),
            );
        }

        if !self.insert_after_stmts.is_empty() {
            self.ctx
                .statement_injector
                .insert_many_after(&stmt_address, self.insert_after_stmts.drain(..));
        }
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
        let mut constructor_index = None;
        let mut index_not_including_removed = 0;
        for element in &class.body.body {
            match element {
                ClassElement::PropertyDefinition(prop) => {
                    // TODO: Throw error if property has decorators

                    // Create binding for private property key
                    if let PropertyKey::PrivateIdentifier(ident) = &prop.key {
                        // Note: Current scope is outside class.
                        let binding = ctx.generate_uid_in_current_scope(
                            ident.name.as_str(),
                            SymbolFlags::FunctionScopedVariable,
                        );
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
                        constructor_index = Some(index_not_including_removed);
                    }
                }
                ClassElement::AccessorProperty(_) | ClassElement::TSIndexSignature(_) => {
                    // TODO: Need to handle these?
                }
            }

            index_not_including_removed += 1;
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
                self.ctx.var_declarations.insert_var(&temp_binding, None, ctx);
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

        // Extract properties and static blocks from class body + substitute computed method keys
        let mut instance_inits = Vec::with_capacity(instance_prop_count);
        class.body.body.retain_mut(|element| {
            match element {
                ClassElement::PropertyDefinition(prop) => {
                    if prop.r#static {
                        self.convert_static_property(prop, ctx);
                    } else {
                        self.convert_instance_property(prop, &mut instance_inits, ctx);
                    }
                    false
                }
                ClassElement::StaticBlock(block) => {
                    if self.transform_static_blocks {
                        self.convert_static_block(block, ctx);
                        false
                    } else {
                        true
                    }
                }
                ClassElement::MethodDefinition(method) => {
                    self.substitute_temp_var_for_method_computed_key(method, ctx);
                    true
                }
                ClassElement::AccessorProperty(_) | ClassElement::TSIndexSignature(_) => {
                    // TODO: Need to handle these?
                    true
                }
            }
        });

        // Insert instance initializers into constructor
        if !instance_inits.is_empty() {
            // TODO: Re-parent any scopes within initializers.
            if let Some(constructor_index) = constructor_index {
                // Existing constructor - amend it
                self.insert_inits_into_constructor(class, instance_inits, constructor_index, ctx);
            } else {
                // No constructor - create one
                Self::insert_constructor(class, instance_inits, ctx);
            }
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

    /// Convert instance property to initialization expression.
    /// Property `foo = 123;` -> Expression `this.foo = 123` or `_defineProperty(this, "foo", 123)`.
    fn convert_instance_property(
        &mut self,
        prop: &mut PropertyDefinition<'a>,
        instance_inits: &mut Vec<Expression<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        // Get value
        let value = match &mut prop.value {
            Some(value) => ctx.ast.move_expression(value),
            None => ctx.ast.void_0(SPAN),
        };

        let init_expr = if let PropertyKey::PrivateIdentifier(ident) = &mut prop.key {
            self.create_private_instance_init_assignment(ident, value, ctx)
        } else {
            // Convert to assignment or `_defineProperty` call, depending on `loose` option
            let this = ctx.ast.expression_this(SPAN);
            self.create_init_assignment(prop, value, this, false, ctx)
        };
        instance_inits.push(init_expr);
    }

    /// Convert static property to initialization expression.
    /// Property `static foo = 123;` -> Expression `C.foo = 123` or `_defineProperty(C, "foo", 123)`.
    fn convert_static_property(
        &mut self,
        prop: &mut PropertyDefinition<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        // Get value, and transform it to replace `this` with reference to class name,
        // and transform class property accesses (`object.#prop`)
        let value = match &mut prop.value {
            Some(value) => {
                self.transform_static_initializer(value, ctx);
                ctx.ast.move_expression(value)
            }
            None => ctx.ast.void_0(SPAN),
        };

        if let PropertyKey::PrivateIdentifier(ident) = &mut prop.key {
            self.insert_private_static_init_assignment(ident, value, ctx);
        } else {
            // Convert to assignment or `_defineProperty` call, depending on `loose` option
            let class_binding = if self.is_declaration {
                // Class declarations always have a name except `export default class {}`.
                // For default export, binding is created when static prop found in 1st pass.
                self.class_bindings.name.as_ref().unwrap()
            } else {
                // Binding is created when static prop found in 1st pass.
                self.class_bindings.temp.as_ref().unwrap()
            };

            let assignee = class_binding.create_read_expression(ctx);
            let init_expr = self.create_init_assignment(prop, value, assignee, true, ctx);
            self.insert_expr_after_class(init_expr, ctx);
        }
    }

    /// `assignee.foo = value` or `_defineProperty(assignee, "foo", value)`
    fn create_init_assignment(
        &mut self,
        prop: &mut PropertyDefinition<'a>,
        value: Expression<'a>,
        assignee: Expression<'a>,
        is_static: bool,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        if self.set_public_class_fields {
            // `assignee.foo = value`
            self.create_init_assignment_loose(prop, value, assignee, is_static, ctx)
        } else {
            // `_defineProperty(assignee, "foo", value)`
            self.create_init_assignment_not_loose(prop, value, assignee, ctx)
        }
    }

    /// `this.foo = value` or `_Class.foo = value`
    fn create_init_assignment_loose(
        &mut self,
        prop: &mut PropertyDefinition<'a>,
        value: Expression<'a>,
        assignee: Expression<'a>,
        is_static: bool,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        // In-built static props `name` and `length` need to be set with `_defineProperty`
        let needs_define = |name| is_static && (name == "name" || name == "length");

        let left = match &mut prop.key {
            PropertyKey::StaticIdentifier(ident) => {
                if needs_define(&ident.name) {
                    return self.create_init_assignment_not_loose(prop, value, assignee, ctx);
                }
                ctx.ast.member_expression_static(SPAN, assignee, ident.as_ref().clone(), false)
            }
            PropertyKey::StringLiteral(str_lit) if needs_define(&str_lit.value) => {
                return self.create_init_assignment_not_loose(prop, value, assignee, ctx);
            }
            key @ match_expression!(PropertyKey) => {
                // TODO: This can also be a numeric key (non-computed). Maybe other key types?
                let key = self.create_computed_key_temp_var(key.to_expression_mut(), ctx);
                ctx.ast.member_expression_computed(SPAN, assignee, key, false)
            }
            PropertyKey::PrivateIdentifier(_) => {
                // Handled in `convert_instance_property` and `convert_static_property`
                unreachable!();
            }
        };

        // TODO: Should this have span of the original `PropertyDefinition`?
        ctx.ast.expression_assignment(
            SPAN,
            AssignmentOperator::Assign,
            AssignmentTarget::from(left),
            value,
        )
    }

    /// `_defineProperty(this, "foo", value)` or `_defineProperty(_Class, "foo", value)`
    fn create_init_assignment_not_loose(
        &mut self,
        prop: &mut PropertyDefinition<'a>,
        value: Expression<'a>,
        assignee: Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let key = match &mut prop.key {
            PropertyKey::StaticIdentifier(ident) => {
                ctx.ast.expression_string_literal(ident.span, ident.name.clone(), None)
            }
            key @ match_expression!(PropertyKey) => {
                // TODO: This can also be a numeric key (non-computed). Maybe other key types?
                self.create_computed_key_temp_var(key.to_expression_mut(), ctx)
            }
            PropertyKey::PrivateIdentifier(_) => {
                // Handled in `convert_instance_property` and `convert_static_property`
                unreachable!();
            }
        };

        let arguments = ctx.ast.vec_from_array([
            Argument::from(assignee),
            Argument::from(key),
            Argument::from(value),
        ]);
        // TODO: Should this have span of the original `PropertyDefinition`?
        self.ctx.helper_call_expr(Helper::DefineProperty, SPAN, arguments, ctx)
    }

    /// Create `_classPrivateFieldInitSpec(this, _prop, value)` to be inserted into class constructor.
    fn create_private_instance_init_assignment(
        &mut self,
        ident: &PrivateIdentifier<'a>,
        value: Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let private_props = self.private_props_stack.last().unwrap();
        let prop = &private_props.props[&ident.name];
        let arguments = ctx.ast.vec_from_array([
            Argument::from(ctx.ast.expression_this(SPAN)),
            Argument::from(prop.binding.create_read_expression(ctx)),
            Argument::from(value),
        ]);
        // TODO: Should this have span of original `PropertyDefinition`?
        self.ctx.helper_call_expr(Helper::ClassPrivateFieldInitSpec, SPAN, arguments, ctx)
    }

    /// Insert after class:
    /// * Class declaration: `var _prop = {_: value};`
    /// * Class expression: `_prop = {_: value}`
    fn insert_private_static_init_assignment(
        &mut self,
        ident: &PrivateIdentifier<'a>,
        value: Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        // `_prop = {_: value}`
        let property = ctx.ast.object_property_kind_object_property(
            SPAN,
            PropertyKind::Init,
            PropertyKey::StaticIdentifier(ctx.ast.alloc(create_underscore_ident_name(ctx))),
            value,
            false,
            false,
            false,
        );
        let obj = ctx.ast.expression_object(SPAN, ctx.ast.vec1(property), None);

        // Insert after class
        let private_props = self.private_props_stack.last().unwrap();
        let prop = &private_props.props[&ident.name];

        if self.is_declaration {
            // `var _prop = {_: value};`
            let var_decl = create_variable_declaration(&prop.binding, obj, ctx);
            self.insert_after_stmts.push(var_decl);
        } else {
            // `_prop = {_: value}`
            let assignment = create_assignment(&prop.binding, obj, ctx);
            self.insert_after_exprs.push(assignment);
        }
    }

    /// Substitute temp var for method computed key.
    /// `class C { [x()]() {} }` -> `let _x; _x = x(); class C { [_x]() {} }`
    /// This transform is only required if class has properties or a static block.
    fn substitute_temp_var_for_method_computed_key(
        &mut self,
        method: &mut MethodDefinition<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let Some(key) = method.key.as_expression_mut() else { return };

        // TODO: Don't alter key if it's provable evaluating it has no side effects.
        // TODO(improve-on-babel): It's unnecessary to create temp vars for method keys unless:
        // 1. Properties also have computed keys.
        // 2. Some of those properties' computed keys have side effects and require temp vars.
        // 3. At least one property satisfying the above is after this method,
        //    or class contains a static block which is being transformed
        //    (static blocks are always evaluated after computed keys, regardless of order)
        method.key = PropertyKey::from(self.create_computed_key_temp_var(key, ctx));
    }

    /// Convert static block to `Expression`.
    ///
    /// `static { x = 1; }` -> `x = 1`
    /// `static { x = 1; y = 2; } -> `(() => { x = 1; y = 2; })()`
    ///
    /// TODO: Add tests for this if there aren't any already.
    /// Include tests for evaluation order inc that static block goes before class expression
    /// unless also static properties, or static block uses class name.
    fn convert_static_block(&mut self, block: &mut StaticBlock<'a>, ctx: &mut TraverseCtx<'a>) {
        // TODO: Convert `this` and references to class name.
        // `x = class C { static { this.C = C; } }` -> `x = (_C = class C {}, _C.C = _C, _C)`
        // TODO: Scope of static block contents becomes outer scope, not scope of class.

        // If class expression, assignment in static block moves to a position where it's read from.
        // e.g.: `x` here now has read+write `ReferenceFlags`:
        // `C = class C { static { x = 1; } }` -> `C = (_C = class C {}, x = 1, _C)`
        let expr = ClassStaticBlock::convert_block_to_expression(block, ctx);
        self.insert_expr_after_class(expr, ctx);
    }

    /// Insert an expression after the class.
    fn insert_expr_after_class(&mut self, expr: Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.is_declaration {
            self.insert_after_stmts.push(ctx.ast.statement_expression(SPAN, expr));
        } else {
            self.insert_after_exprs.push(expr);
        }
    }

    /// Convert computed property/method key to a temp var.
    ///
    /// Transformation is:
    /// * Class declaration:
    ///   `class C { [x()] = 1; }` -> `let _x; _x = x(); class C { constructor() { this[_x] = 1; } }`
    /// * Class expression:
    ///   `C = class { [x()] = 1; }` -> `let _x; C = (_x = x(), class C { constructor() { this[_x] = 1; } })`
    ///
    /// This function:
    /// * Creates the `let _x;` statement and inserts it.
    /// * Creates the `_x = x()` assignments.
    /// * Inserts assignments before class declaration, or adds to `state` if class expression.
    /// * Returns `_x`.
    fn create_computed_key_temp_var(
        &mut self,
        key: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let key = ctx.ast.move_expression(key);

        // Bound vars and literals do not need temp var - return unchanged.
        // e.g. `let x = 'x'; class C { [x] = 1; }` or `class C { ['x'] = 1; }`
        // TODO: Do fuller analysis to detect expressions which cannot have side effects e.g. `'x' + 'y'`.
        let cannot_have_side_effects = match &key {
            Expression::BooleanLiteral(_)
            | Expression::NullLiteral(_)
            | Expression::NumericLiteral(_)
            | Expression::BigIntLiteral(_)
            | Expression::RegExpLiteral(_)
            | Expression::StringLiteral(_)
            | Expression::ThisExpression(_) => true,
            Expression::Identifier(ident) => {
                // Cannot have side effects if is bound.
                // Additional check that the var is not mutated is required for cases like
                // `let x = 1; class { [x] = 1; [++x] = 2; }`
                // `++x` is hoisted to before class in output, so `x` in 1st key would get the wrong
                // value unless it's hoisted out too.
                // TODO: Add an exec test for this odd case.
                // TODO(improve-on-babel): That case is rare.
                // Test for it in first pass over class elements, and avoid temp vars where possible.
                match ctx.symbols().get_reference(ident.reference_id()).symbol_id() {
                    Some(symbol_id) => {
                        ctx.symbols().get_flags(symbol_id).intersects(SymbolFlags::ConstVariable)
                            || ctx
                                .symbols()
                                .get_resolved_references(symbol_id)
                                .all(|reference| !reference.is_write())
                    }
                    None => false,
                }
            }
            _ => false,
        };
        if cannot_have_side_effects {
            return key;
        }

        // We entered transform via `enter_expression` or `enter_statement`,
        // so `ctx.current_scope_id()` is the scope outside the class
        let parent_scope_id = ctx.current_scope_id();
        // TODO: Handle if is a class expression defined in a function's params.
        let binding =
            ctx.generate_uid_based_on_node(&key, parent_scope_id, SymbolFlags::BlockScopedVariable);

        self.ctx.var_declarations.insert_let(&binding, None, ctx);

        let assignment = create_assignment(&binding, key, ctx);
        self.insert_before.push(assignment);

        binding.create_read_expression(ctx)
    }
}

/// Create `new WeakMap()` expression.
///
/// Takes an `&mut Option<Option<SymbolId>>` which is updated after looking up the binding for `WeakMap`.
/// * `None` = Not looked up yet.
/// * `Some(None)` = Has been looked up, and `WeakMap` is unbound.
/// * `Some(Some(symbol_id))` = Has been looked up, and `WeakMap` has a local binding.
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
