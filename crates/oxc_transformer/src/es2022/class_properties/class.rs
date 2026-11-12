//! ES2022: Class Properties
//! Transform of class itself.

use oxc_allocator::{Address, ArenaVec, GetAddress, TakeIn, UnstableAddress};
use oxc_ast::ast::*;
use oxc_span::SPAN;
use oxc_str::{Ident, static_ident};
use oxc_syntax::{
    reference::ReferenceFlags,
    symbol::{SymbolFlags, SymbolId},
};
use oxc_traverse::{Ancestor, BoundIdentifier};

use crate::{
    common::helper_loader::{Helper, helper_call_expr},
    context::TraverseCtx,
    decorator::standard::{ClassNameRewriter, DecoratorElementKind, StandardDecoratorState},
    utils::{
        ast_builder::{create_assignment, create_new_weakset, create_variable_declaration},
        decorated_class::class_body_has_decorators,
    },
};

use super::{
    ClassBindings, ClassDetails, ClassProperties, IdentIndexMap, PrivateProp, PrivateProps,
    utils::exprs_into_stmts,
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

impl<'a> ClassProperties<'a> {
    /// Perform first phase of transformation of class.
    ///
    /// This is the only entry point into the transform upon entering class body.
    ///
    /// First we check if any transforms are necessary, and exit if not.
    ///
    /// If transform is required:
    /// * Build a hashmap of private property keys.
    /// * Push `ClassDetails` containing info about the class to `classes_stack`.
    /// * Extract instance property initializers (public or private) from class body and insert into
    ///   class constructor.
    /// * Temporarily replace computed keys of instance properties with assignments to temp vars.
    ///   `class C { [foo()] = 123; }` -> `class C { [_foo = foo()]; }`
    pub(super) fn transform_class_body_on_entry(
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

        // Get basic details about class
        let is_declaration = *class.r#type() == ClassType::ClassDeclaration;
        let class_has_decorators = !class.decorators().is_empty();
        let mut class_name_binding = class.id().as_ref().map(BoundIdentifier::from_binding_ident);
        let class_scope_id = class.scope_id().get().unwrap();
        let has_super_class = class.heritage().is_some();
        let class_address = class.address();
        let private_method_helpers_in_class_sequence =
            !is_declaration && Self::should_emit_private_method_helpers_in_class_sequence(ctx);

        // Standard decorators. A decorated class has all fields, private names and static blocks lowered.
        let is_decorated = self.decorator_state.enabled()
            && (class_has_decorators || class_body_has_decorators(body));
        // `accessor` is not supported by any engine, so lower it in every class.
        let has_auto_accessor = self.decorator_state.enabled()
            && body
                .body
                .iter()
                .any(|e| matches!(e, ClassElement::AccessorProperty(p) if !p.r#type.is_abstract()));
        if self.decorator_state.enabled() {
            for element in &mut body.body {
                let (key, computed, value) = match element {
                    ClassElement::PropertyDefinition(prop) => {
                        let prop = &mut **prop;
                        (&mut prop.key, prop.computed, &prop.value)
                    }
                    ClassElement::AccessorProperty(prop) => {
                        let prop = &mut **prop;
                        (&mut prop.key, prop.computed, &prop.value)
                    }
                    _ => continue,
                };
                if let Some(value) = value {
                    self.decorator_state.capture_class_initializer_name(key, computed, value, ctx);
                }
            }
        }
        if self.only_decorated_classes && !is_decorated && !has_auto_accessor {
            // Register private names only when they can shadow a lowered name in an enclosing
            // class. Other unresolved names belong to untouched classes and are left as-is.
            let mut private_names = IdentIndexMap::default();
            if self.private_field_count != 0 {
                for element in &body.body {
                    let key = match element {
                        ClassElement::PropertyDefinition(prop) => &prop.key,
                        ClassElement::MethodDefinition(method) => &method.key,
                        ClassElement::AccessorProperty(prop) => &prop.key,
                        _ => continue,
                    };
                    if let PropertyKey::PrivateIdentifier(ident) = key {
                        private_names.insert(ident.name, ());
                    }
                }
            }
            self.classes_stack.push(ClassDetails {
                is_declaration,
                private_method_helpers_in_class_sequence: false,
                is_transform_required: false,
                private_props: PrivateProps::Native(private_names),
                bindings: ClassBindings::dummy(false),
                decorators: None,
                has_auto_accessor,
            });
            return;
        }
        let static_accessor_binding = (has_auto_accessor
            && body.body.iter().any(|element| {
                matches!(element, ClassElement::AccessorProperty(prop) if prop.r#static && !prop.r#type.is_abstract())
            }))
        .then(|| {
            if is_decorated {
                let name = class_name_binding.as_ref().map_or("Class", |binding| binding.name.as_str());
                ctx.generate_uid(name, ctx.current_block_scope_id(), SymbolFlags::BlockScopedVariable)
            } else {
                ClassBindings::create_temp_binding(
                    class_name_binding.as_ref(),
                    ctx.current_hoist_scope_id(),
                    ctx,
                )
            }
        });
        if is_decorated {
            StandardDecoratorState::coerce_decorated_keys(body, ctx);
        }
        if is_decorated || has_auto_accessor {
            self.decorator_state.lower_auto_accessors(
                body,
                class_scope_id,
                static_accessor_binding.as_ref(),
                ctx,
            );
        }
        let decorator_state = if is_decorated {
            Some(self.decorator_state.prepare_class_decorators(body, ctx))
        } else {
            None
        };

        // Point references to the class name inside a decorated class declaration at a new
        // `let _Foo` binding, which the class decorator can reassign. `class Foo {}` statement
        // is converted to `let _Foo = class Foo {}` in exit phase.
        if is_decorated
            && is_declaration
            && let Some(name_binding) = &class_name_binding
        {
            let outer_scope_id = ctx.scoping().scope_parent_id(class_scope_id).unwrap();
            let inner = ClassNameRewriter::new(
                name_binding.symbol_id,
                name_binding.name,
                None,
                Some(outer_scope_id),
                ctx,
            )
            .rewrite(body);
            if let Some(inner) = inner {
                self.decorator_state.record_declaration_rewrite(
                    class_address,
                    (inner.clone(), name_binding.clone()),
                );
                class_name_binding = Some(inner);
            }
        }
        let instance_method_extras =
            decorator_state.as_ref().is_some_and(|state| state.instance_method_extras);

        // Check if class has any properties, private methods, or static blocks
        let mut instance_prop_count = 0;
        let mut has_static_prop = false;
        let mut has_instance_private_method = false;
        let mut has_static_private_method_or_static_block = false;
        // TODO: Store `IdentIndexMap`s in a pool and re-use them
        let mut private_props: IdentIndexMap<'a, PrivateProp<'a>> = IdentIndexMap::default();
        for element in &mut body.body {
            match element {
                ClassElement::PropertyDefinition(prop) => {
                    // TODO: Throw error if property has decorators

                    // Ignore `declare` properties as they don't have any runtime effect,
                    // and will be removed in the TypeScript transform later
                    if prop.declare {
                        continue;
                    }

                    // Create binding for private property key
                    if let Some(name) = prop.key.private_name() {
                        // Note: Current scope is outside class.
                        let binding = if is_decorated {
                            ctx.generate_uid(
                                &name,
                                ctx.current_block_scope_id(),
                                SymbolFlags::BlockScopedVariable,
                            )
                        } else {
                            ctx.generate_uid_in_current_hoist_scope(&name)
                        };
                        private_props
                            .insert(name, PrivateProp::new(binding, prop.r#static, None, false));
                    }

                    if prop.r#static {
                        has_static_prop = true;
                    } else {
                        instance_prop_count += 1;
                    }
                }
                ClassElement::StaticBlock(_) => {
                    // Static block only necessitates transforming class if it's being transformed
                    if self.transform_static_blocks || is_decorated || has_auto_accessor {
                        has_static_private_method_or_static_block = true;
                    }
                }
                ClassElement::MethodDefinition(method) => {
                    if let Some(private_name) = method.key.private_name() {
                        if method.r#static {
                            has_static_private_method_or_static_block = true;
                        } else {
                            has_instance_private_method = true;
                        }

                        let name = match method.kind {
                            MethodDefinitionKind::Method => private_name.as_str(),
                            MethodDefinitionKind::Get => &format!("get_{private_name}"),
                            MethodDefinitionKind::Set => &format!("set_{private_name}"),
                            MethodDefinitionKind::Constructor => unreachable!(),
                        };
                        let (scope_id, flags) = if private_method_helpers_in_class_sequence {
                            if is_decorated {
                                (ctx.current_block_scope_id(), SymbolFlags::BlockScopedVariable)
                            } else {
                                (ctx.current_hoist_scope_id(), SymbolFlags::FunctionScopedVariable)
                            }
                        } else {
                            (ctx.current_block_scope_id(), SymbolFlags::Function)
                        };
                        let binding = ctx.generate_uid(name, scope_id, flags);

                        if let Some(prop) = private_props.get_mut(&private_name) {
                            // If there's already a binding for this private property,
                            // it's a setter or getter, so store the binding in `binding2`.
                            prop.set_binding2(binding);
                        } else {
                            private_props.insert(
                                private_name,
                                PrivateProp::new(
                                    binding,
                                    method.r#static,
                                    Some(method.kind),
                                    false,
                                ),
                            );
                        }
                    }
                }
                ClassElement::AccessorProperty(prop) => {
                    // TODO: Not sure what we should do here.
                    // Only added this to prevent panics in TS conformance tests.
                    if let Some(name) = prop.key.private_name() {
                        let dummy_binding = BoundIdentifier::new(Ident::empty(), SymbolId::new(0));
                        private_props.insert(
                            name,
                            PrivateProp::new(dummy_binding, prop.r#static, None, true),
                        );
                    }
                }
                ClassElement::TSIndexSignature(_) => {
                    // TODO: Need to handle this?
                }
            }
        }

        if is_decorated {
            for prop in private_props.values_mut() {
                prop.uses_weakmap = prop.is_static && !prop.is_method();
            }
        }
        self.private_field_count += private_props.len();

        // Exit if nothing to transform
        if instance_prop_count == 0
            && !has_static_prop
            && !has_static_private_method_or_static_block
            && !has_instance_private_method
            && !is_decorated
        {
            self.classes_stack.push(ClassDetails {
                is_declaration,
                private_method_helpers_in_class_sequence,
                is_transform_required: false,
                private_props: PrivateProps::Lowered(private_props),
                bindings: ClassBindings::dummy(self.private_fields_as_properties),
                decorators: None,
                has_auto_accessor,
            });
            return;
        }

        // Initialize class binding vars.
        // Static prop in class expression or anonymous `export default class {}` always require
        // temp var for class. Static prop in class declaration doesn't.
        let need_temp_var = is_decorated
            || static_accessor_binding.is_some()
            || (has_static_prop && (!is_declaration || class_name_binding.is_none()));

        let outer_hoist_scope_id = ctx.current_hoist_scope_id();
        let class_temp_binding = if need_temp_var {
            let temp_binding = if let Some(binding) = static_accessor_binding {
                binding
            } else if is_decorated {
                let name =
                    class_name_binding.as_ref().map_or("Class", |binding| binding.name.as_str());
                ctx.generate_uid(
                    name,
                    ctx.current_block_scope_id(),
                    SymbolFlags::BlockScopedVariable,
                )
            } else {
                ClassBindings::create_temp_binding(
                    class_name_binding.as_ref(),
                    outer_hoist_scope_id,
                    ctx,
                )
            };
            if is_declaration && class_name_binding.is_none() {
                // Anonymous `export default class {}`. Set class name binding to temp var.
                // Actual class name will be set to this later.
                class_name_binding = Some(temp_binding.clone());
            } else {
                // Create temp var `var _Class;` statement.
                // TODO(improve-on-babel): Inserting the temp var `var _Class` statement here is only
                // to match Babel's output. It'd be simpler just to insert it at the end and get rid of
                // `temp_var_is_created` that tracks whether it's done already or not.
                if is_decorated {
                    ctx.state.var_declarations.insert_let(&temp_binding, None, &ctx.ast);
                } else {
                    ctx.state.var_declarations.insert_var(&temp_binding, &ctx.ast);
                }
            }
            Some(temp_binding)
        } else {
            None
        };

        let class_brand_binding = has_instance_private_method.then(|| {
            // `_Class_brand`
            let name = class_name_binding.as_ref().map_or_else(|| "Class", |binding| &binding.name);
            let name = &format!("_{name}_brand");
            if is_decorated {
                ctx.generate_uid(
                    name,
                    ctx.current_block_scope_id(),
                    SymbolFlags::BlockScopedVariable,
                )
            } else {
                ctx.generate_uid_in_current_hoist_scope(name)
            }
        });

        let static_private_fields_use_temp = !is_declaration;
        let mut class_bindings = ClassBindings::new(
            class_name_binding,
            class_temp_binding,
            class_brand_binding,
            outer_hoist_scope_id,
            static_private_fields_use_temp,
            need_temp_var,
            self.private_fields_as_properties && !is_decorated,
        );

        if is_decorated {
            class_bindings.force_private_spec = true;
            if private_props.values().any(|prop| prop.is_static && prop.is_method()) {
                let brand = ctx.generate_uid(
                    "staticBrand",
                    ctx.current_block_scope_id(),
                    SymbolFlags::BlockScopedVariable,
                );
                ctx.state.var_declarations.insert_let(&brand, None, &ctx.ast);
                class_bindings.decorator_static_brand = Some(brand);
            }
        }

        // Add entry to `classes_stack`
        self.classes_stack.push(ClassDetails {
            is_declaration,
            private_method_helpers_in_class_sequence,
            is_transform_required: true,
            private_props: PrivateProps::Lowered(private_props),
            bindings: class_bindings,
            decorators: decorator_state,
            has_auto_accessor,
        });

        // Class expression: references to class name inside class body use temp var for class
        if is_decorated && !is_declaration {
            let bindings = &self.current_class().bindings;
            if let (Some(name), Some(temp)) = (&bindings.name, &bindings.temp) {
                let (symbol_id, name, temp) = (name.symbol_id, name.name, temp.clone());
                ClassNameRewriter::new(symbol_id, name, Some(temp), None, ctx).rewrite(body);
            }
        }

        // Exit if no instance properties (public or private)
        if instance_prop_count == 0 && !has_instance_private_method && !instance_method_extras {
            return;
        }

        // Extract instance properties initializers.
        //
        // We leave the properties themselves in place, but take the initializers.
        // `class C { prop = 123; }` -> `class { prop; }`
        // Leave them in place to avoid shifting up all the elements twice.
        // We already have to do that in exit phase, so better to do it all at once.
        //
        // Also replace any instance property computed keys with an assignment to temp var.
        // `class C { [foo()] = 123; }` -> `class C { [_foo = foo()]; }`
        // Those assignments will be moved to before class in exit phase of the transform.
        // -> `_foo = foo(); class C {}`
        let mut instance_inits =
            Vec::with_capacity(instance_prop_count + usize::from(has_instance_private_method));

        // Install private methods before extra initializers can access them.
        if has_instance_private_method {
            instance_inits.push(self.create_class_private_method_init_spec(ctx));
        }
        if instance_method_extras {
            let this = Expression::new_this_expression(SPAN, ctx);
            instance_inits.push(
                self.current_class()
                    .decorators
                    .as_ref()
                    .unwrap()
                    .run_initializers_call(5, this, ctx),
            );
        }

        let mut constructor_index = None;
        for (index, element) in body.body.iter_mut().enumerate() {
            #[expect(clippy::match_same_arms)]
            match element {
                ClassElement::PropertyDefinition(prop) => {
                    // Ignore `declare` properties as they don't have any runtime effect,
                    // and will be removed in the TypeScript transform later
                    if !prop.r#static && !prop.declare {
                        if self.decorator_state.enabled() {
                            self.decorator_state.record_instance_field_for_decorators(prop, ctx);
                        }
                        let decorated_index = self.decorator_state.decorated_field_index(prop);
                        if let Some(index) = decorated_index {
                            let value = prop.value.take();
                            prop.value = Some(
                                self.current_class()
                                    .decorators
                                    .as_ref()
                                    .unwrap()
                                    .wrap_decorated_field_value(value, index, ctx),
                            );
                        }
                        self.convert_instance_property(prop, &mut instance_inits, ctx);
                        if let Some(index) = decorated_index {
                            instance_inits.push(
                                self.current_class()
                                    .decorators
                                    .as_ref()
                                    .unwrap()
                                    .decorated_field_extra_initializers(index, ctx),
                            );
                        }
                    }
                }
                ClassElement::MethodDefinition(method) => {
                    if method.kind == MethodDefinitionKind::Constructor
                        && method.value.body.is_some()
                    {
                        constructor_index = Some(index);
                    }
                }
                ClassElement::AccessorProperty(_) | ClassElement::TSIndexSignature(_) => {
                    // TODO: Need to handle these?
                }
                ClassElement::StaticBlock(_) => {}
            }
        }

        self.insert_instance_initializers(
            body,
            instance_inits,
            constructor_index,
            class_scope_id,
            has_super_class,
            ctx,
        );
    }

    /// Transform class declaration on exit.
    ///
    /// This is the exit phase of the transform. Only applies to class *declarations*.
    /// Class *expressions* are handled in [`ClassProperties::transform_class_expression_on_exit`] below.
    /// Both functions do much the same, `transform_class_expression_on_exit` inserts items as expressions,
    /// whereas here we insert as statements.
    ///
    /// At this point, other transforms have had a chance to run on the class body, so we can move
    /// parts of the code out to before/after the class now.
    ///
    /// * Transform static properties and insert after class.
    /// * Transform static blocks and insert after class.
    /// * Extract computed key assignments and insert them before class.
    /// * Remove all properties and static blocks from class body.
    ///
    /// Items needing insertion before/after class are inserted as statements.
    ///
    /// `class C { static [foo()] = 123; static { bar(); } }`
    /// -> `_foo = foo(); class C {}; C[_foo] = 123; bar();`
    pub(super) fn transform_class_declaration_on_exit(
        &mut self,
        class: &mut Class<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        // Ignore TS class declarations
        // TODO: Is this correct?
        if class.declare {
            return;
        }

        // Leave class expressions to `transform_class_expression_on_exit`
        let class_details = self.current_class();
        if !class_details.is_declaration {
            return;
        }

        // Finish transform
        if class_details.is_transform_required {
            self.transform_class_declaration_on_exit_impl(class, ctx);
        } else {
            debug_assert!(class_details.bindings.temp.is_none());
        }
        // Pop off stack. We're done!
        let class_details = self.classes_stack.pop();
        if let Some(private_props) = class_details.private_props.lowered() {
            // Note: `private_props` can be non-empty even if `is_transform_required == false`,
            // if class contains private accessors, which we don't transform yet
            self.private_field_count -= private_props.len();
        }
    }

    fn transform_class_declaration_on_exit_impl(
        &mut self,
        class: &mut Class<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        // Static properties and static blocks
        self.current_class_mut().bindings.static_private_fields_use_temp = true;

        if self.current_class().decorators.is_some() {
            let name = (class.id.is_none()
                && matches!(ctx.parent(), Ancestor::ExportDefaultDeclarationDeclaration(_)))
            .then(|| Expression::new_string_literal(SPAN, "default", None, ctx));
            self.decorator_state.begin_class_exit(name);
        }

        // Transform static properties, remove static and instance properties, and move computed keys
        // to before class
        self.transform_class_elements(class, ctx);

        // Computed keys move outside the class. Reparent their first-level scopes as well,
        // including functions containing brand checks against the lexical alias.
        if !self.insert_before.is_empty() {
            let keys = std::mem::take(&mut self.insert_before);
            self.reparent_initializers_scope(&keys, ctx.current_scope_id(), None, ctx);
            self.insert_before = keys;
        }

        // Insert temp var for class if required. Name class if required.
        let class_details = self.classes_stack.last_mut();
        let shadowed_name_declaration = if let Some(binding) = &class_details.bindings.shadowed_name
        {
            // Computed keys run before the class exists. Start with `undefined` so brand checks
            // in keys can reject other objects without reading an uninitialized lexical binding.
            // The `let` must be fresh for each evaluation of a declaration in a loop.
            let declarator = VariableDeclarator::new(
                SPAN,
                binding.create_binding_pattern(ctx),
                None,
                None,
                false,
                ctx,
            );
            let declaration = Statement::new_variable_declaration(
                SPAN,
                VariableDeclarationKind::Let,
                [declarator],
                false,
                ctx,
            );

            // A preserved static block runs during class evaluation, before after-class statements.
            // Initialize the alias at the start of the first block, before its user code can call
            // a method that uses it. Otherwise initialize before the extracted static initializers.
            if let Some(block) = class.body.body.iter_mut().find_map(|element| match element {
                ClassElement::StaticBlock(block) => Some(block),
                _ => None,
            }) {
                let init = Expression::new_this_expression(SPAN, ctx);
                let expr = create_assignment(binding, init, SPAN, ctx);
                block.body.insert(0, Statement::new_expression_statement(SPAN, expr, ctx));
            } else {
                let init =
                    class_details.bindings.name.as_ref().unwrap().create_read_expression(ctx);
                let expr = create_assignment(binding, init, SPAN, ctx);
                let stmt = Statement::new_expression_statement(SPAN, expr, ctx);
                self.insert_after_stmts.insert(0, stmt);
            }
            Some(declaration)
        } else {
            None
        };
        if let Some(temp_binding) = &class_details.bindings.temp {
            // Binding for class name is required
            if let Some(ident) = &class.id {
                // For an anonymous class, `temp_var_is_created` assumes the class itself will bind
                // the temp. If another transform changed the class ID, declare the temp separately.
                if !class_details.bindings.temp_var_is_created
                    || (class_details
                        .bindings
                        .name
                        .as_ref()
                        .is_some_and(|name| name.symbol_id == temp_binding.symbol_id)
                        && ident.symbol_id() != temp_binding.symbol_id)
                {
                    class_details.declare_temp(temp_binding, ctx);
                }

                // Insert `_Class = Class` after class.
                // TODO(improve-on-babel): Could just insert `var _Class = Class;` after class,
                // rather than separate `var _Class` declaration.
                let class_name_binding = class_details
                    .bindings
                    .name
                    .as_ref()
                    .expect("class declaration with a temp always has a name binding");
                // Preserve a later name assigned to an anonymous class. Named classes use the
                // outer binding captured on entry, rather than a rewritten inner class binding.
                let class_name = if class_name_binding.symbol_id == temp_binding.symbol_id {
                    BoundIdentifier::from_binding_ident(ident).create_read_expression(ctx)
                } else {
                    class_name_binding.create_read_expression(ctx)
                };
                let expr = create_assignment(temp_binding, class_name, SPAN, ctx);
                let stmt = Statement::new_expression_statement(SPAN, expr, ctx);
                self.insert_after_stmts.insert(0, stmt);
            } else {
                // Class must be default export `export default class {}`, as all other class declarations
                // always have a name. Set class name.
                *ctx.scoping_mut().symbol_flags_mut(temp_binding.symbol_id) = SymbolFlags::Class;
                class.id = Some(temp_binding.create_binding_identifier(ctx));
            }
        }

        // Insert statements before/after class
        let stmt_address = match ctx.parent() {
            parent @ (Ancestor::ExportDefaultDeclarationDeclaration(_)
            | Ancestor::ExportDeclarationDeclaration(_)) => parent.address(),
            // `Class` is always stored in a `Box`, so has a stable memory location
            _ => class.unstable_address(),
        };

        if !self.insert_before.is_empty() {
            ctx.state.statement_injector.insert_many_before(
                &stmt_address,
                exprs_into_stmts(self.insert_before.drain(..), &ctx.ast),
            );
        }

        if let Some(private_props) = class_details.private_props.lowered() {
            if class_details.bindings.private_fields_as_properties() {
                let props_to_init: Vec<_> = private_props
                    .iter()
                    .filter_map(|(&name, prop)| {
                        // TODO: Output `var _C_brand = new WeakSet();` for private instance method
                        if prop.is_method() || prop.is_accessor {
                            return None;
                        }
                        Some((name, prop.binding.clone()))
                    })
                    .collect();
                if !props_to_init.is_empty() {
                    let mut stmts = Vec::with_capacity(props_to_init.len());
                    for (name, binding) in props_to_init {
                        // `var _prop = _classPrivateFieldLooseKey("prop");`
                        let value = Self::create_private_prop_key_loose(name, ctx);
                        stmts.push(create_variable_declaration(&binding, value, ctx));
                    }
                    ctx.state.statement_injector.insert_many_before(&stmt_address, stmts);
                }
            } else {
                let mut weakmap_symbol_id = None;
                let mut has_method = false;
                let mut stmts = Vec::new();
                for prop in private_props.values() {
                    if (prop.is_static && !prop.uses_weakmap)
                        || (prop.is_method() && has_method)
                        || prop.is_accessor
                    {
                        continue;
                    }
                    if prop.is_method() {
                        // `var _C_brand = new WeakSet();`
                        has_method = true;
                        let binding = class_details.bindings.brand();
                        let value = create_new_weakset(ctx);
                        stmts.push(class_details.create_temp_declaration(binding, value, ctx));
                    } else {
                        // `var _prop = new WeakMap();`
                        let value = create_new_weakmap(&mut weakmap_symbol_id, ctx);
                        stmts.push(class_details.create_temp_declaration(
                            &prop.binding,
                            value,
                            ctx,
                        ));
                    }
                }
                if !stmts.is_empty() {
                    ctx.state.statement_injector.insert_many_before(&stmt_address, stmts);
                }
            }
        }

        if let Some(declaration) = shadowed_name_declaration {
            // `insert_many_before` prepends, so do this after inserting computed key evaluations.
            ctx.state.statement_injector.insert_many_before(&stmt_address, [declaration]);
        }

        if !self.insert_after_stmts.is_empty() {
            ctx.state
                .statement_injector
                .insert_many_after(&stmt_address, self.insert_after_stmts.drain(..));
        }
    }

    /// Transform class expression on exit.
    ///
    /// This is the exit phase of the transform. Only applies to class *expressions*.
    /// Class *expressions* are handled in [`ClassProperties::transform_class_declaration_on_exit`] above.
    /// Both functions do much the same, `transform_class_declaration_on_exit` inserts items as statements,
    /// whereas here we insert as expressions.
    ///
    /// At this point, other transforms have had a chance to run on the class body, so we can move
    /// parts of the code out to before/after the class now.
    ///
    /// * Transform static properties and insert after class.
    /// * Transform static blocks and insert after class.
    /// * Extract computed key assignments and insert them before class.
    /// * Remove all properties and static blocks from class body.
    ///
    /// Items needing insertion before/after class are inserted as expressions around, surrounding class
    /// in a [`SequenceExpression`].
    ///
    /// `let C = class { static [foo()] = 123; static { bar(); } };`
    /// -> `let C = (_foo = foo(), _Class = class {}, _Class[_foo] = 123, bar(), _Class);`
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

        // Finish transform
        let class_details = self.current_class();
        if class_details.is_transform_required {
            self.transform_class_expression_on_exit_impl(expr, ctx);
        } else {
            debug_assert!(class_details.bindings.temp.is_none());
        }

        // Pop off stack. We're done!
        let class_details = self.classes_stack.pop();
        if let Some(private_props) = class_details.private_props.lowered() {
            // Note: `private_props` can be non-empty even if `is_transform_required == false`,
            // if class contains private accessors, which we don't transform yet
            self.private_field_count -= private_props.len();
        }
    }

    fn transform_class_expression_on_exit_impl(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let Expression::ClassExpression(class) = expr else { unreachable!() };
        let mut inferred_name = if class.id.is_none()
            && (self.current_class().decorators.is_some() || self.current_class().has_auto_accessor)
        {
            self.decorator_state.infer_class_expression_name(class.address(), ctx)
        } else {
            None
        };
        if self.current_class().decorators.is_some() {
            let name = if class.decorators.is_empty() { None } else { inferred_name.take() };
            self.decorator_state.begin_class_exit(name);
        }

        // Transform static properties, remove static and instance properties, and move computed keys
        // to before class
        self.transform_class_elements(class, ctx);

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

        // TODO: Deduct static private props and private methods from `expr_count`.
        // Or maybe should store count and increment it when create private static props or private methods?
        // They're probably pretty rare, so it'll be rarely used.
        let class_details = self.classes_stack.last();

        let private_method_helpers_in_class_sequence =
            class_details.private_method_helpers_in_class_sequence;
        let mut expr_count = self.insert_before.len() + self.insert_after_exprs.len();
        if private_method_helpers_in_class_sequence {
            // Private method helpers are emitted into the class-expression sequence (see below).
            expr_count += self.insert_after_stmts.len();
        }
        if let Some(private_props) = class_details.private_props.lowered() {
            expr_count += private_props.len();
        }

        // Exit if no expressions to insert before or after class
        if expr_count == 0 {
            return;
        }

        expr_count += 1 + usize::from(class_details.bindings.temp.is_some());

        let mut exprs = ArenaVec::with_capacity_in(expr_count, ctx);

        // Insert `_prop = new WeakMap()` expressions for private instance props
        // (or `_prop = _classPrivateFieldLooseKey("prop")` if loose mode).
        // Babel has these always go first, regardless of order of class elements.
        // Also insert `var _prop;` temp var declarations for private static props.
        if let Some(private_props) = class_details.private_props.lowered() {
            // Insert `var _prop;` declarations here rather than when binding was created to maintain
            // same order of `var` declarations as Babel.
            // `c = class C { #x = 1; static y = 2; }` -> `var _C, _x;`
            // TODO(improve-on-babel): Simplify this.
            if class_details.bindings.private_fields_as_properties() {
                exprs.extend(private_props.iter().filter_map(|(&name, prop)| {
                    // TODO: Output `_C_brand = new WeakSet()` for private instance method
                    if prop.is_method() || prop.is_accessor {
                        return None;
                    }

                    // Insert `var _prop;` declaration
                    ctx.state.var_declarations.insert_var(&prop.binding, &ctx.ast);

                    // `_prop = _classPrivateFieldLooseKey("prop")`
                    let value = Self::create_private_prop_key_loose(name, ctx);
                    Some(create_assignment(&prop.binding, value, SPAN, ctx))
                }));
            } else {
                let mut weakmap_symbol_id = None;
                let mut has_method = false;
                exprs.extend(private_props.values().filter_map(|prop| {
                    if prop.is_accessor {
                        return None;
                    }

                    if prop.is_method() {
                        if prop.is_static || has_method {
                            return None;
                        }
                        has_method = true;
                        // `_C_brand = new WeakSet()`
                        let binding = class_details.bindings.brand();
                        class_details.declare_temp(binding, ctx);
                        let value = create_new_weakset(ctx);
                        return Some(create_assignment(binding, value, SPAN, ctx));
                    }

                    // Insert `var _prop;` declaration
                    class_details.declare_temp(&prop.binding, ctx);

                    if prop.is_static && !prop.uses_weakmap {
                        return None;
                    }

                    // `_prop = new WeakMap()`
                    let value = create_new_weakmap(&mut weakmap_symbol_id, ctx);
                    Some(create_assignment(&prop.binding, value, SPAN, ctx))
                }));
            }
        }

        // Insert private methods.
        if private_method_helpers_in_class_sequence {
            // A concise arrow has no statement list to inject a function declaration into.
            // Emit `_m = function () {}` into the class sequence instead. Adding `var _m`
            // expands the concise arrow to a block, so the binding is fresh for every call.
            for stmt in self.insert_after_stmts.drain(..) {
                let Statement::FunctionDeclaration(mut func) = stmt else {
                    unreachable!(
                        "class expression private methods are always function declarations"
                    );
                };
                let id = func.id.take().expect("private method binding always has an id");
                let binding = BoundIdentifier::from_binding_ident(&id);
                class_details.declare_temp(&binding, ctx);
                func.r#type = FunctionType::FunctionExpression;
                let assignment =
                    create_assignment(&binding, Expression::FunctionExpression(func), SPAN, ctx);
                exprs.push(assignment);
            }
        } else if !self.insert_after_stmts.is_empty() {
            // Find `Address` of statement containing class expression.
            let mut stmt_address = Address::DUMMY;
            for ancestor in ctx.ancestors() {
                if ancestor.is_parent_of_statement() {
                    break;
                }
                stmt_address = ancestor.address();
            }

            ctx.state
                .statement_injector
                .insert_many_after(&stmt_address, self.insert_after_stmts.drain(..));
        }

        // Insert computed key initializers
        exprs.extend(self.insert_before.drain(..));

        // Insert class + static property assignments + static blocks
        if let Some(binding) = &class_details.bindings.temp {
            // Insert `var _Class` statement, if it wasn't already in entry phase
            if !class_details.bindings.temp_var_is_created {
                ctx.state.var_declarations.insert_var(binding, &ctx.ast);
            }

            // `_Class = class {}`
            let class_expr = StandardDecoratorState::set_inferred_class_name(
                expr.take_in(ctx),
                inferred_name,
                ctx,
            );
            let assignment = create_assignment(binding, class_expr, SPAN, ctx);

            if exprs.is_empty() && self.insert_after_exprs.is_empty() {
                // No need to wrap in sequence if no static property
                // and static blocks
                *expr = assignment;
                return;
            }

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

            if exprs.is_empty() {
                return;
            }

            let class_expr = StandardDecoratorState::set_inferred_class_name(
                expr.take_in(ctx),
                inferred_name,
                ctx,
            );
            exprs.push(class_expr);
        }

        debug_assert!(exprs.len() > 1);
        debug_assert!(exprs.len() <= expr_count);

        *expr = Expression::new_sequence_expression(SPAN, exprs, ctx);
    }

    /// Transform class elements.
    ///
    /// This is part of the exit phase of transform, performed on both class declarations
    /// and class expressions.
    ///
    /// At this point, other transforms have had a chance to run on the class body, so we can move
    /// parts of the code out to before/after the class now.
    ///
    /// * Transform static properties and insert after class.
    /// * Transform static blocks and insert after class.
    /// * Transform private methods and insert after class.
    /// * Extract computed key assignments and insert them before class.
    /// * Remove all properties, private methods and static blocks from class body.
    fn transform_class_elements(&mut self, class: &mut Class<'a>, ctx: &mut TraverseCtx<'a>) {
        let is_decorated = self.current_class().decorators.is_some();
        if is_decorated {
            self.decorator_state.take_class_decorators(class, &mut self.insert_before, ctx);
        }
        let transform_static_blocks =
            self.transform_static_blocks || is_decorated || self.current_class().has_auto_accessor;

        let mut class_methods = vec![];
        class.body.body.retain_mut(|element| {
            match element {
                ClassElement::PropertyDefinition(prop_box) => {
                    let decorated_index = self.decorator_state.decorated_field_index(prop_box);
                    let prop_address = prop_box.address();
                    let prop = &mut **prop_box;
                    debug_assert!(
                        !prop.declare,
                        "`declare` property should have been removed in the TypeScript plugin's `exit_class`"
                    );
                    // Element decorators are evaluated before element's computed key
                    let private_name = match &prop.key {
                        PropertyKey::PrivateIdentifier(ident) => Some(ident.name),
                        _ => None,
                    };
                    let decorators = if is_decorated && !prop.decorators.is_empty() {
                        Some(StandardDecoratorState::take_element_decorators(&mut prop.decorators, &prop.key, &mut self.insert_before, ctx))
                    } else {
                        None
                    };

                    if prop.r#static {
                        if decorators.is_some() {
                            StandardDecoratorState::memoize_decorated_key(&mut prop.key, &mut self.insert_before, ctx);
                        }
                        let key = decorators.as_ref().map(|_| StandardDecoratorState::decorator_key_expr(&prop.key, ctx));
                        if let Some(index) = decorated_index {
                            let value = prop.value.take();
                            prop.value = Some(self.current_class().decorators.as_ref().unwrap().wrap_decorated_field_value(value, index, ctx));
                        }
                        self.convert_static_property(prop, ctx);
                        if let Some(index) = decorated_index {
                            let mut extra = self.current_class().decorators.as_ref().unwrap().decorated_field_extra_initializers(index, ctx);
                            self.transform_static_initializer(&mut extra, ctx);
                            self.insert_expr_after_class(extra, ctx);
                        }
                        if let (Some(decorators), Some(key)) = (&decorators, key) {
                            self.push_decorate_element(
                                DecoratorElementKind::Field,
                                true,
                                key,
                                decorators,
                                private_name,
                                ctx,
                            );
                        }
                    } else {
                        if let Some(decorators) = &decorators {
                            let key = self.decorator_state.take_instance_key(prop_address)
                                .unwrap_or_else(|| StandardDecoratorState::decorator_key_expr(&prop.key, ctx));
                            self.push_decorate_element(
                                DecoratorElementKind::Field,
                                false,
                                key,
                                decorators,
                                private_name,
                                ctx,
                            );
                        }
                        if prop.computed {
                            self.extract_instance_prop_computed_key(prop, ctx);
                        }
                    }
                    return false;
                }
                ClassElement::StaticBlock(block) => {
                    if transform_static_blocks {
                        self.convert_static_block(block, ctx);
                        return false;
                    }
                }
                ClassElement::MethodDefinition(method) => {
                    let kind = self.decorator_state.method_decorator_kind(method);
                    let method_ref = &mut **method;
                    let decorators = if is_decorated && !method_ref.decorators.is_empty() {
                        Some(StandardDecoratorState::take_element_decorators(&mut method_ref.decorators, &method_ref.key, &mut self.insert_before, ctx))
                    } else {
                        None
                    };
                    let private_name = match &method.key {
                        PropertyKey::PrivateIdentifier(ident) => Some(ident.name),
                        _ => None,
                    };
                    self.substitute_temp_var_for_method_computed_key(method, ctx);
                    if let Some(decorators) = &decorators {
                        let key = StandardDecoratorState::decorator_key_expr(&method.key, ctx);
                        self.push_decorate_element(kind, method.r#static, key, decorators, private_name, ctx);
                    }
                    if let Some(statement) = self.convert_private_method(method, ctx) {
                        class_methods.push(statement);
                        return false;
                    }
                }
                ClassElement::AccessorProperty(_) | ClassElement::TSIndexSignature(_) => {
                    // TODO: Need to handle these?
                }
            }

            true
        });

        // All methods are moved to after the class, but need to be before static properties
        // TODO(improve-on-babel): Insertion order doesn't matter, and it more clear to insert according to
        // definition order.
        let class_methods_count = class_methods.len();
        self.insert_after_stmts.splice(0..0, class_methods);

        if is_decorated {
            self.decorator_state.fold_element_decorators_into_heritage(
                class,
                &mut self.insert_before,
                ctx,
            );
        }

        // Standard decorators are applied after class definition, before static members are initialized.
        if is_decorated {
            let (exprs, class_extras) = self.build_decorator_sequence(class, ctx);
            if self.current_class().is_declaration {
                let stmts = exprs
                    .into_iter()
                    .map(|expr| Statement::new_expression_statement(SPAN, expr, ctx));
                self.insert_after_stmts.splice(class_methods_count..class_methods_count, stmts);
                if let Some(expr) = class_extras {
                    self.insert_after_stmts
                        .push(Statement::new_expression_statement(SPAN, expr, ctx));
                }
            } else {
                self.insert_after_exprs.splice(0..0, exprs);
                if let Some(expr) = class_extras {
                    self.insert_after_exprs.push(expr);
                }
            }
        }
    }

    /// Flag that static private fields should be transpiled using temp binding,
    /// while in this static property or static block.
    ///
    /// We turn `static_private_fields_use_temp` on and off when entering exiting static context.
    ///
    /// Static private fields reference class name (not temp var) in class declarations.
    /// `class Class { static #privateProp; method() { return obj.#privateProp; } }`
    /// -> `method() { return _assertClassBrand(Class, obj, _privateProp)._; }`
    ///                                         ^^^^^
    ///
    /// But in static properties and static blocks, they use the temp var.
    /// `class Class { static #privateProp; static publicProp = obj.#privateProp; }`
    /// -> `Class.publicProp = _assertClassBrand(_Class, obj, _privateProp)._`
    ///                                          ^^^^^^
    ///
    /// Also see comments on `ClassBindings`.
    ///
    /// Note: If declaration is `export default class {}` with no name, and class has static props,
    /// then class has had name binding created already in `transform_class`.
    /// So name binding is always `Some`.
    pub(super) fn flag_entering_static_property_or_block(&mut self) {
        // No need to check if class is a declaration, because `static_private_fields_use_temp`
        // is always `true` for class expressions anyway
        self.current_class_mut().bindings.static_private_fields_use_temp = true;
    }

    /// Flag that static private fields should be transpiled using name binding again
    /// as we're exiting this static property or static block.
    /// (see [`ClassProperties::flag_entering_static_property_or_block`])
    pub(super) fn flag_exiting_static_property_or_block(&mut self) {
        // Flag that transpiled static private props use name binding in class declarations
        let class_details = self.current_class_mut();
        if class_details.is_declaration {
            class_details.bindings.static_private_fields_use_temp = false;
        }
    }

    /// `_classPrivateFieldLooseKey("prop")`
    fn create_private_prop_key_loose(name: Ident<'a>, ctx: &mut TraverseCtx<'a>) -> Expression<'a> {
        helper_call_expr(
            Helper::ClassPrivateFieldLooseKey,
            ArenaVec::from_value_in(Argument::new_string_literal(SPAN, name, None, ctx), ctx),
            ctx,
        )
    }

    /// Insert an expression after the class.
    pub(super) fn insert_expr_after_class(&mut self, expr: Expression<'a>, ctx: &TraverseCtx<'a>) {
        if self.current_class().is_declaration {
            self.insert_after_stmts.push(Statement::new_expression_statement(SPAN, expr, ctx));
        } else {
            self.insert_after_exprs.push(expr);
        }
    }
}

/// Create `new WeakMap()` expression.
///
/// Takes an `&mut Option<Option<SymbolId>>` which is updated after looking up the binding for `WeakMap`.
///
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
    let weak_map = static_ident!("WeakMap");
    let symbol_id = *symbol_id
        .get_or_insert_with(|| ctx.scoping().find_binding(ctx.current_scope_id(), weak_map));
    let ident = ctx.create_ident_expr(SPAN, weak_map, symbol_id, ReferenceFlags::Read);
    Expression::new_new_expression_with_pure(SPAN, ident, None, [], true, ctx)
}
