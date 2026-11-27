use rustc_hash::FxHashSet;

use oxc_allocator::{ArenaVec, CloneIn, GetAllocator, ReplaceWith, TakeIn};
use oxc_ast::ast::*;
use oxc_ast_visit::{Visit, VisitJs, walk};
use oxc_semantic::{ReferenceFlags, ScopeFlags, ScopeId, Scoping, SymbolFlags};
use oxc_span::{SPAN, Span};
use oxc_str::{Ident, Str, static_ident};
use oxc_syntax::operator::{AssignmentOperator, UnaryOperator};
use oxc_traverse::BoundIdentifier;

use crate::{
    common::{
        class_lowering::ClassLowering,
        computed_key::{create_computed_key_temp_var, key_needs_temp_var},
        duplicate::duplicate_expression,
    },
    context::TraverseCtx,
    utils::ast_builder::{
        create_class_constructor, create_class_method, create_this_property_access,
        create_this_property_assignment, create_variable_declaration, take_literal_property_key,
    },
};

use super::TypeScript;

/// What happens to a property declaration when its initializer moves out of the class body.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Declaration {
    /// The declaration is removed, along with its key.
    Remove,
    /// The declaration stays, and the class still evaluates its key.
    Keep,
    /// The declaration stays as a `declare` field with a copy of its key, for legacy decorators.
    Placeholder,
}

impl<'a> TypeScript<'a> {
    /// Transform class fields, and constructor parameters that includes modifiers into `this` assignments.
    ///
    /// This transformation is doing 2 things:
    ///
    /// 1. Convert constructor parameters that include modifier to `this` assignments and insert them
    ///    after the super call in the constructor body.
    ///
    /// Same as `Self::convert_constructor_params` does, the reason why we still need that method because
    /// this method only runs when `set_public_class_fields` is `true` and the `class_properties` plugin
    /// is disabled. The other branches (`class_properties` enabled, or `set_public_class_fields: false`
    /// — see [`Self::add_parameter_property_fields`]) reach the parameter-property assignments via
    /// `convert_constructor_params` directly. Merging them together would add unnecessary checks for
    /// the param-only paths.
    ///
    /// 2. Convert class fields to `this` assignments in the constructor body.
    ///
    /// > This transformation only works when `set_public_class_fields` is `true`,
    /// > and the fields have initializers, which is to align with the behavior of TypeScript's
    /// > `useDefineForClassFields: false` option.
    ///
    /// Input:
    /// ```ts
    /// class C {
    ///   x = 1;
    ///   [y] = 2;
    /// }
    /// ```
    ///
    /// Output:
    /// ```js
    /// let _y;
    /// class C {
    ///   static {
    ///     _y = y;
    ///   }
    ///   constructor() {
    ///     this.x = 1;
    ///     this[_y] = 2;
    ///   }
    /// }
    /// ```
    ///
    /// The computed key transformation behavior is the same as `TypeScript`.
    /// Computed key assignments are inserted into a static block, unlike Babel which inserts them before class.
    /// We follow `TypeScript` just for simplicity, because `Babel` handles class expressions and class declarations
    /// differently, which is quite troublesome to implement.
    /// Anyway, `TypeScript` is the source of truth for the typescript transformation.
    ///
    /// A static block has its own `this`, `super`, `arguments` and `new.target`, and can't contain
    /// `yield` or `await`. When a key uses them, the keys stay in a key of a temporary static method,
    /// which the static block deletes instead:
    ///
    /// Input:
    /// ```ts
    /// function* f() {
    ///   return class {
    ///     static [yield "a"] = 1;
    ///   };
    /// }
    /// ```
    ///
    /// Output:
    /// ```js
    /// var _computedKey = Symbol();
    /// function* f() {
    ///   let _yield$a;
    ///   return class {
    ///     static {
    ///       delete this[_computedKey];
    ///     }
    ///     static {
    ///       this[_yield$a] = 1;
    ///     }
    ///     static [(_yield$a = yield "a", _computedKey)]() {}
    ///   };
    /// }
    /// ```
    ///
    /// For static properties, we convert them to static blocks.
    ///
    /// Input:
    /// ```ts
    /// class C {
    ///   static x = 1;
    ///   static [y] = 2;
    /// }
    /// ```
    ///
    /// Output:
    /// ```js
    /// let _y;
    /// class C {
    ///   static {
    ///     this.x = 1;
    ///   }
    ///   static {
    ///     this[_y] = 2;
    ///   }
    /// }
    /// ```
    ///
    /// The transformation way is also the same as `TypeScript`, the advantage from the implementation is that
    /// we don't need extra transformation for static properties, the output is the same as instance properties
    /// transformation, and the greatest advantage is we don't need to care about `this` usage in static block.
    ///
    /// A decorated field whose declaration is removed stays as a `declare` field with a side-effect
    /// free key until legacy decorators lower it: `@dec static [x()] = 1;` -> `@dec declare static [_x];`
    ///
    /// Fields that `lowering` keeps for a downstream transform stay as written, and so do instance
    /// fields before [`ClassLowering::first_movable_instance_field`].
    pub(super) fn transform_class_fields(
        &mut self,
        class: &mut Class<'a>,
        lowering: ClassLowering,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let first_movable_index = lowering.first_movable_instance_field(class);
        // When any non-private instance field has an initializer, all instance field
        // initializers (including private) must be hoisted to the constructor to preserve
        // execution order. This matches TypeScript's `WillHoistInitializersToConstructor`:
        // https://github.com/microsoft/TypeScript/blob/7b8cb3bdf8/src/compiler/transformers/classFields.ts#L339
        let will_hoist_initializers_to_constructor =
            class.body.body[first_movable_index..].iter().any(|e| {
                matches!(e, ClassElement::PropertyDefinition(prop)
                    if !prop.r#static && !prop.declare && !prop.key.is_private_identifier() && prop.value.is_some()
                )
            });

        // Static blocks created below must be parented to the class body scope,
        // not the current traversal scope (which is the scope enclosing the class).
        let class_scope_id = class.scope_id();

        let lowers_decorated_declare_fields = self.lowers_decorated_declare_fields(class);
        let mut constructor_index = None;
        let mut property_assignments = Vec::new();
        let mut computed_key_assignments = Vec::new();
        // A decorated static field gets its static block inserted after it, so `position` indexes
        // the growing class body, and `index` is the element's index before any insertion.
        let mut position = 0;
        let mut inserted_blocks = 0;
        while let Some(element) = class.body.body.get_mut(position) {
            let index = position - inserted_blocks;
            let mut block_after = None;
            let kept = lowering.keeps_field(element);
            match element {
                ClassElement::PropertyDefinition(prop)
                    if lowers_decorated_declare_fields
                        && !kept
                        && prop.declare
                        && !prop.decorators.is_empty() =>
                {
                    Self::convert_decorated_declare_field_key(
                        prop,
                        &mut computed_key_assignments,
                        ctx,
                    );
                }
                // Skip static private properties — converting them to static blocks
                // would lose the private field declaration.
                ClassElement::PropertyDefinition(prop)
                    if !kept
                        && (prop.r#static || index >= first_movable_index)
                        && (!prop.key.is_private_identifier()
                            || (!prop.r#static && will_hoist_initializers_to_constructor)) =>
                {
                    // Legacy decorators lower decorated fields on class exit. A decorated field
                    // whose declaration is removed here stays until then as a `declare` field,
                    // which `TypeScriptAnnotations::exit_class` removes after them.
                    let decorated =
                        !prop.decorators.is_empty() && !prop.key.is_private_identifier();
                    if let Some(value) = prop.value.take() {
                        // A static property is replaced by a static block below, and an instance
                        // property is removed later when `remove_class_fields_without_initializer`
                        // is `true`. Otherwise, the instance property declaration stays in the
                        // class body (without its initializer), so its key is still evaluated
                        // during class definition.
                        let declaration =
                            if !prop.r#static && !self.remove_class_fields_without_initializer {
                                Declaration::Keep
                            } else if decorated {
                                Declaration::Placeholder
                            } else {
                                Declaration::Remove
                            };
                        let assignment = self.convert_property_definition(
                            &mut prop.key,
                            value,
                            declaration,
                            &mut computed_key_assignments,
                            ctx,
                        );
                        prop.declare |= declaration == Declaration::Placeholder;
                        if prop.r#static {
                            // Convert static property to static block
                            // `class C { static x = 1; }` -> `class C { static { this.x = 1; } }`
                            // `class C { static [x] = 1; }` -> `let _x; class C { static { this[_x] = 1; } }`
                            let body = ArenaVec::from_value_in(assignment, ctx);
                            let block = Self::create_class_static_block(body, class_scope_id, ctx);
                            if prop.declare {
                                block_after = Some(block);
                            } else {
                                *element = block;
                            }
                        } else {
                            property_assignments.push(assignment);
                        }
                    } else if self.remove_class_fields_without_initializer && decorated {
                        // `@dec [x()];` -> `@dec declare [_x];`, and `_x = x()` is evaluated in
                        // order with the other moved keys.
                        if !prop.declare
                            && let Some(key) = prop.key.as_expression_mut()
                        {
                            Self::move_key_to_temp_var(key, &mut computed_key_assignments, ctx);
                        }
                        prop.declare = true;
                    } else if self.remove_class_fields_without_initializer
                        && !prop.declare
                        && let Some(key) = prop.key.as_expression_mut()
                    {
                        // A `declare` field is erased along with its key, so the key never runs.
                        // `TypeScript` uses `isSimpleInlineableExpression` to check if the key needs to be kept.
                        // There is a little difference that we treat `BigIntLiteral` and `RegExpLiteral` can be kept, and
                        // `IdentifierReference` without symbol is not kept.
                        // https://github.com/microsoft/TypeScript/blob/8c62e08448e0ec76203bd519dd39608dbcb31705/src/compiler/transformers/classFields.ts#L2720
                        if key_needs_temp_var(key, ctx) {
                            // When `remove_class_fields_without_initializer` is true, the property without initializer
                            // would be removed in the `transform_class_on_exit`. We need to make sure the computed key
                            // keeps and is evaluated in the same order as the original class field in static block.
                            computed_key_assignments.push(key.take_in(ctx));
                        }
                    } else if !prop.declare && prop.key.is_computed() {
                        // The property declaration without initializer is kept in the class body, so its
                        // computed key is evaluated during class definition. Any pending computed key
                        // assignments from previous elements must be evaluated before it.
                        Self::convert_computed_key(
                            &mut prop.key,
                            &mut computed_key_assignments,
                            ctx,
                        );
                    }
                }
                ClassElement::MethodDefinition(method) => {
                    if method.kind == MethodDefinitionKind::Constructor {
                        constructor_index = Some(position);
                    } else if method.key.is_computed() {
                        // A non-computed key (e.g. `"m"() {}`) has no side effects, and can't hold
                        // a sequence expression. Pending assignments go to the next computed key
                        // or the static block instead.
                        Self::convert_computed_key(
                            &mut method.key,
                            &mut computed_key_assignments,
                            ctx,
                        );
                    }
                }
                ClassElement::AccessorProperty(accessor) if accessor.key.is_computed() => {
                    Self::convert_computed_key(
                        &mut accessor.key,
                        &mut computed_key_assignments,
                        ctx,
                    );
                }
                // A computed key that stays in the class still evaluates after the keys before it.
                ClassElement::PropertyDefinition(prop)
                    if !prop.declare && prop.key.is_computed() =>
                {
                    Self::convert_computed_key(&mut prop.key, &mut computed_key_assignments, ctx);
                }
                _ => (),
            }
            if let Some(block) = block_after {
                class.body.body.insert(position + 1, block);
                position += 1;
                inserted_blocks += 1;
            }
            position += 1;
        }

        let computed_key_assignment_static_block = self.create_computed_key_static_block(
            class,
            computed_key_assignments,
            &property_assignments,
            lowering,
            ctx,
        );

        // Blocks are only inserted after the visited element, and the carrier is appended,
        // so `constructor_index` is still valid.
        let constructor = constructor_index.map(|position| {
            let ClassElement::MethodDefinition(method) = &mut class.body.body[position] else {
                unreachable!()
            };
            &mut method.value
        });

        if let Some(constructor) = constructor {
            let params = &constructor.params.items;

            let params_assignment = Self::convert_constructor_params(params, ctx);
            property_assignments.splice(0..0, params_assignment);

            if property_assignments.is_empty() {
                // No property/parameter assignments to inject, but computed-key temp
                // inits still need their static block (e.g. `static [expr] = value` +
                // empty `constructor()`), otherwise the temp var is left uninitialized.
                if let Some(element) = computed_key_assignment_static_block {
                    class.body.body.insert(0, element);
                }
                return;
            }

            // `constructor {}` is guaranteed that it is `Some`.
            let constructor_body_statements = &mut constructor.body.as_mut().unwrap().statements;
            let super_call_position = Self::get_super_call_position(constructor_body_statements);

            // Insert the assignments after the `super()` call
            constructor_body_statements
                .splice(super_call_position..super_call_position, property_assignments);

            // Insert the static block after the constructor if there is a constructor
            if let Some(element) = computed_key_assignment_static_block {
                class.body.body.insert(0, element);
            }
        } else if !property_assignments.is_empty() {
            // If there is no constructor, we need to create a default constructor
            // that initializes the public fields
            // TODO: should use `ctx.insert_scope_below_statements`, but it only accept an `ArenaVec` rather than std `Vec`.
            let scope_id = ctx.create_child_scope(
                class_scope_id,
                ScopeFlags::StrictMode | ScopeFlags::Function | ScopeFlags::Constructor,
            );
            let ctor = create_class_constructor(
                property_assignments,
                class.heritage.is_some(),
                scope_id,
                ctx,
            );

            // Insert the static block at the beginning of the class body if there is no constructor
            if let Some(element) = computed_key_assignment_static_block {
                class.body.body.splice(0..0, [ctor, element]);
            } else {
                // TODO(improve-on-babel): Could push constructor onto end of elements, instead of inserting as first
                class.body.body.insert(0, ctor);
            }
        } else if let Some(element) = computed_key_assignment_static_block {
            class.body.body.insert(0, element);
        }
    }

    /// Insert an uninitialized class field declaration for each constructor parameter property.
    ///
    /// Aligns with TypeScript's output under `useDefineForClassFields: true`, which is the
    /// default since TypeScript 4.0. Runs when [`crate::CompilerAssumptions::set_public_class_fields`]
    /// is `false` (the inverse of the `useDefineForClassFields: false` mapping).
    ///
    /// Input:
    /// ```ts
    /// class C {
    ///   constructor(public x = 1) {}
    /// }
    /// ```
    ///
    /// Output:
    /// ```js
    /// class C {
    ///   x;
    ///   constructor(x = 1) {
    ///     this.x = x;
    ///   }
    /// }
    /// ```
    ///
    /// Skips parameter properties whose name matches an existing instance field declaration on
    /// the same class (matching TypeScript's dedup). Only static-identifier instance fields
    /// count — static, private, computed, and `declare`-only fields don't suppress the new field.
    ///
    /// `properties` comes from [`Self::collect_parameter_properties`] on class entry, before
    /// parameter modifiers are erased. Fields are added on class exit, after fields without
    /// initializers are removed, so neither the removal nor a removed field affects them.
    pub(super) fn add_parameter_property_fields(
        class: &mut Class<'a>,
        properties: &[(Span, Ident<'a>)],
        ctx: &TraverseCtx<'a>,
    ) {
        let mut declared_names = Self::collect_instance_field_names(class);
        let fields = ArenaVec::from_iter_in(
            properties
                .iter()
                .filter(|(_, name)| declared_names.insert(*name))
                .map(|&(span, name)| Self::build_uninitialized_field(span, name, ctx)),
            ctx,
        );

        class.body.body.splice(0..0, fields);
    }

    /// Append the span and name of each constructor parameter property to `properties`.
    ///
    /// Safe to read `id.name` directly here: this runs on class entry, before any rename.
    /// `convert_constructor_params` uses the span trick because it also runs alongside
    /// class-properties.
    pub(super) fn collect_parameter_properties(
        class: &Class<'a>,
        properties: &mut Vec<(Span, Ident<'a>)>,
    ) {
        if let Some(params) = Self::find_constructor_params(class) {
            properties.extend(
                params
                    .iter()
                    .filter(|param| param.has_modifier())
                    .filter_map(|param| param.pattern.get_binding_identifier())
                    .map(|id| (id.span, id.name)),
            );
        }
    }

    /// Returns the body-bearing constructor's parameter list, or `None` if the class has none.
    fn find_constructor_params<'b>(
        class: &'b Class<'a>,
    ) -> Option<&'b ArenaVec<'a, FormalParameter<'a>>> {
        class.body.body.iter().find_map(|element| {
            if let ClassElement::MethodDefinition(method) = element
                && method.kind.is_constructor()
                && method.value.body.is_some()
            {
                Some(&method.value.params.items)
            } else {
                None
            }
        })
    }

    /// Collect names of instance field declarations keyed by a static identifier.
    ///
    /// Static, private, computed, and `declare`-only fields don't suppress a parameter-property
    /// field — they're different bindings or erased before emit. Plain instance-field collisions
    /// are a TS `Duplicate identifier` error, so this dedup is mostly defensive.
    fn collect_instance_field_names(class: &Class<'a>) -> FxHashSet<Ident<'a>> {
        class
            .body
            .body
            .iter()
            .filter_map(|element| match element {
                ClassElement::PropertyDefinition(prop)
                    if !prop.r#static
                        && !prop.declare
                        && let PropertyKey::StaticIdentifier(ident) = &prop.key =>
                {
                    Some(ident.name)
                }
                ClassElement::AccessorProperty(prop)
                    if !prop.r#static
                        && let PropertyKey::StaticIdentifier(ident) = &prop.key =>
                {
                    Some(ident.name)
                }
                _ => None,
            })
            .collect()
    }

    /// Build `<name>;` — an uninitialized public class field declaration.
    fn build_uninitialized_field(
        span: Span,
        name: Ident<'a>,
        ctx: &TraverseCtx<'a>,
    ) -> ClassElement<'a> {
        let key = PropertyKey::new_static_identifier(span, name, ctx);
        ClassElement::new_property_definition(
            span,
            PropertyDefinitionType::PropertyDefinition,
            [],
            key,
            None,
            None,
            false,
            false,
            false,
            false,
            false,
            false,
            None,
            ctx,
        )
    }

    /// Remove fields without initializers, as TypeScript erases them.
    ///
    /// Decorated fields stay, for a downstream transform or for legacy decorators to lower. See
    /// `TransformerImpl::exit_class` for how this interacts with legacy decorators on class exit.
    pub(super) fn remove_uninitialized_class_fields(
        &self,
        class: &mut Class<'a>,
        ctx: &TraverseCtx<'a>,
    ) {
        if !self.remove_class_fields_without_initializer {
            return;
        }

        class.body.body.retain(|element| {
            !matches!(element, ClassElement::PropertyDefinition(prop)
                if self.removes_uninitialized_field(prop, ctx))
        });
    }

    /// Whether [`Self::remove_uninitialized_class_fields`] removes `prop`: a field without an
    /// initializer or decorators, whose key has no side effects.
    fn removes_uninitialized_field(
        &self,
        prop: &PropertyDefinition<'a>,
        ctx: &TraverseCtx<'a>,
    ) -> bool {
        self.remove_class_fields_without_initializer
            && prop.value.is_none()
            && prop.decorators.is_empty()
            && !prop.key.is_private_identifier()
            && prop.key.as_expression().is_none_or(|key| !key_needs_temp_var(key, ctx))
    }

    /// Transform constructor parameters that include modifier to `this` assignments and
    /// insert them after the super call in the constructor body.
    ///
    /// Input:
    /// ```ts
    /// class C {
    ///   constructor(public x, private y) {}
    /// }
    /// ```
    ///
    /// Output:
    /// ```js
    /// class C {
    ///  constructor(x, y) {
    ///   this.x = x;
    ///   this.y = y;
    /// }
    /// ```
    pub(super) fn transform_class_constructor(
        constructor: &mut MethodDefinition<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if !constructor.kind.is_constructor() || constructor.value.body.is_none() {
            return;
        }

        let params = &constructor.value.params.items;
        let assignments = Self::convert_constructor_params(params, ctx).collect::<Vec<_>>();

        let constructor_body_statements = &mut constructor.value.body.as_mut().unwrap().statements;
        let super_call_position = Self::get_super_call_position(constructor_body_statements);

        // Insert the assignments after the `super()` call
        constructor_body_statements.splice(super_call_position..super_call_position, assignments);
    }

    /// Convert property definition to `this` assignment in constructor.
    ///
    /// * Computed key:
    ///   `class C { [x()] = 1; }` -> `let _x; class C { static { _x = x(); } constructor() { this[_x] = 1; } }`
    /// * Static key:
    ///   `class C { x = 1; }` -> `class C { constructor() { this.x = 1; } }`
    ///
    /// With [`Declaration::Keep`], the property declaration stays in the class body, so its key is
    /// still evaluated during class definition. In that case the temp var is assigned inside the
    /// key itself, so the key expression is evaluated exactly once and in source order:
    ///   `class C { [x()] = 1; }` -> `let _x; class C { constructor() { this[_x] = 1; } [_x = x()]; }`
    ///
    /// With [`Declaration::Placeholder`], the key is evaluated like a removed one, and the
    /// declaration keeps a copy: `[x()] = 1` -> `declare [_x];`
    ///
    /// Returns an assignment statement which would be inserted in the constructor body.
    #[expect(clippy::unused_self)]
    fn convert_property_definition(
        &self,
        key: &mut PropertyKey<'a>,
        value: Expression<'a>,
        declaration: Declaration,
        computed_key_assignments: &mut Vec<Expression<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Statement<'a> {
        let keep_key = declaration != Declaration::Remove;
        let member = match key {
            PropertyKey::StaticIdentifier(ident) => {
                create_this_property_access(SPAN, ident.name, ctx)
            }
            PropertyKey::PrivateIdentifier(ident) => {
                // Accessor backing fields: `this.#prop = value`
                let ident = PrivateIdentifier::new(SPAN, ident.name, ctx);
                MemberExpression::new_private_field_expression(
                    SPAN,
                    Expression::new_this_expression(SPAN, ctx),
                    ident,
                    false,
                    ctx,
                )
            }
            PropertyKey::StaticStringLiteral(_)
            | PropertyKey::StaticNumericLiteral(_)
            | PropertyKey::StaticBigIntLiteral(_) => {
                // `class C { 'x' = true; 123 = false; }`
                let new_key = if keep_key {
                    // Keep the original key in the kept declaration, rather than leaving
                    // the `NullLiteral` dummy from `take_in`, which would create a `null` property.
                    // A literal key is side-effect free, so the copy can be used in the assignment.
                    take_literal_property_key(&mut key.clone_in(ctx.allocator()), ctx)
                } else {
                    take_literal_property_key(key, ctx)
                }
                .unwrap();
                MemberExpression::new_computed_member_expression(
                    SPAN,
                    Expression::new_this_expression(SPAN, ctx),
                    new_key,
                    false,
                    ctx,
                )
            }
            key @ match_expression!(PropertyKey) => {
                let key = key.to_expression_mut();
                let new_key = if key_needs_temp_var(key, ctx) {
                    let taken_key = key.take_in(ctx);
                    let (assignment, ident) = create_computed_key_temp_var(taken_key, ctx);
                    if declaration == Declaration::Keep {
                        // All computed keys are evaluated during class definition, before any
                        // static block runs, so assigning the temp var in a static block would
                        // leave it `undefined` when the kept declaration's key is evaluated.
                        // Assign it in the key instead: `[x()] = 1` -> `[_x = x()];`.
                        *key = assignment;
                        Self::prepend_computed_key_assignments(key, computed_key_assignments, ctx);
                        ident
                    } else {
                        computed_key_assignments.push(assignment);
                        if keep_key {
                            let (key_copy, ident) = duplicate_expression(ident, false, ctx);
                            *key = key_copy;
                            ident
                        } else {
                            ident
                        }
                    }
                } else if keep_key {
                    // Keep the original key in the kept declaration, rather than leaving
                    // the `NullLiteral` dummy from `take_in`, which would create a `null` property.
                    // The key is side-effect free here, so it can be duplicated without a temp var.
                    let (original_key, new_key) =
                        duplicate_expression(key.take_in(ctx), false, ctx);
                    *key = original_key;
                    new_key
                } else {
                    key.take_in(ctx)
                };

                MemberExpression::new_computed_member_expression(
                    SPAN,
                    Expression::new_this_expression(SPAN, ctx),
                    new_key,
                    false,
                    ctx,
                )
            }
        };
        let target = AssignmentTarget::from(member);
        Self::create_assignment(target, value, ctx)
    }

    /// Find the position of the `super()` call in the constructor body, otherwise return 0.
    ///
    /// Don't need to handle nested `super()` call because `TypeScript` doesn't allow it.
    pub fn get_super_call_position(statements: &[Statement<'a>]) -> usize {
        // Find the position of the `super()` call in the constructor body.
        // Don't need to handle nested `super()` call because `TypeScript` doesn't allow it.
        statements
            .iter()
            .position(|stmt| {
                matches!(stmt, Statement::ExpressionStatement(stmt)
                        if stmt.expression.is_super_call_expression())
            })
            .map_or(0, |pos| pos + 1)
    }

    /// Convert computed key to sequence expression if there are assignments.
    ///
    /// Input:
    /// ```ts
    /// class C {
    ///   [x()] = 1;
    ///   [y()]() {}
    ///   [x()] = 2;
    /// }
    /// ```
    ///
    /// Output:
    /// ```js
    /// let _x, _x2;
    /// class C {
    ///   constructor() {
    ///     this[_x] = 1;
    ///     this[_x2] = 2;
    ///   }
    ///   static {
    ///     _x2 = x();
    ///   }
    ///   [(_x = x(), y())]() {}
    /// }
    /// ```
    ///
    /// So that computed key keeps running in the same order as the original class field.
    #[inline]
    fn convert_computed_key(
        key: &mut PropertyKey<'a>,
        assignments: &mut Vec<Expression<'a>>,
        ctx: &TraverseCtx<'a>,
    ) {
        if let Some(key) = key.as_expression_mut() {
            Self::prepend_computed_key_assignments(key, assignments, ctx);
        }
    }

    /// Prepend pending computed key `assignments` to `key`: `key` -> `(_x = x(), key)`.
    fn prepend_computed_key_assignments(
        key: &mut Expression<'a>,
        assignments: &mut Vec<Expression<'a>>,
        ctx: &TraverseCtx<'a>,
    ) {
        if assignments.is_empty() {
            return;
        }
        // Create a new expression sequence to insert the assignments into.
        key.replace_with(|original_key| {
            Expression::new_sequence_expression(
                SPAN,
                ArenaVec::from_iter_in(
                    assignments.drain(..).chain(std::iter::once(original_key)),
                    ctx,
                ),
                ctx,
            )
        });
    }

    /// Create `static { _x = x(), _y = y(); }` for computed key `assignments` that no key left in
    /// the class evaluates. Callers insert it first in the class body, so it runs after the class
    /// keys and before static initialization.
    fn create_computed_key_static_block(
        &mut self,
        class: &mut Class<'a>,
        assignments: Vec<Expression<'a>>,
        property_assignments: &[Statement<'a>],
        lowering: ClassLowering,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<ClassElement<'a>> {
        if assignments.is_empty() {
            return None;
        }
        let assignments = ArenaVec::from_iter_in(assignments, ctx);
        let expr = if assignments.iter().any(depends_on_outer_context) {
            self.append_computed_key_carrier(
                class,
                assignments,
                property_assignments,
                lowering,
                ctx,
            )
        } else {
            Some(Expression::new_sequence_expression(SPAN, assignments, ctx))
        };
        expr.map(|expr| {
            let statement = Statement::new_expression_statement(SPAN, expr, ctx);
            Self::create_class_static_block(
                ArenaVec::from_value_in(statement, ctx),
                class.scope_id(),
                ctx,
            )
        })
    }

    /// Evaluate the keys of decorated `declare` fields in order with the other class keys, when
    /// `transform_class_fields` doesn't. See [`Self::convert_decorated_declare_field_key`].
    pub(super) fn convert_decorated_declare_field_keys(
        &mut self,
        class: &mut Class<'a>,
        lowering: ClassLowering,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let is_decorated_declare_field = |element: &ClassElement<'a>| {
            matches!(element, ClassElement::PropertyDefinition(prop)
                if prop.declare && !prop.decorators.is_empty())
                && !lowering.keeps_field(element)
        };
        if !self.lowers_decorated_declare_fields(class)
            || !class.body.body.iter().any(is_decorated_declare_field)
        {
            return;
        }

        let mut computed_key_assignments = Vec::new();
        for element in &mut class.body.body {
            if is_decorated_declare_field(element) {
                let ClassElement::PropertyDefinition(prop) = element else { unreachable!() };
                Self::convert_decorated_declare_field_key(prop, &mut computed_key_assignments, ctx);
                continue;
            }
            let key = match element {
                ClassElement::MethodDefinition(method) => &mut method.key,
                ClassElement::AccessorProperty(accessor) => &mut accessor.key,
                ClassElement::PropertyDefinition(prop)
                    if !prop.declare && !self.removes_uninitialized_field(prop, ctx) =>
                {
                    &mut prop.key
                }
                _ => continue,
            };
            if key.is_computed() {
                Self::convert_computed_key(key, &mut computed_key_assignments, ctx);
            }
        }

        if let Some(block) = self.create_computed_key_static_block(
            class,
            computed_key_assignments,
            &[],
            lowering,
            ctx,
        ) {
            class.body.body.insert(0, block);
        }
    }

    /// Whether legacy decorators lower the decorated `declare` fields of `class`, and so need their
    /// keys. They leave the members of class expressions as they are, and other decorator modes
    /// erase `declare` fields together with their decorators.
    fn lowers_decorated_declare_fields(&self, class: &Class<'a>) -> bool {
        self.legacy_decorators && !class.is_expression()
    }

    /// The class removes a `declare` field, and legacy decorators pass its key to `_decorate` after
    /// the class. A key with side effects is evaluated once, in order with the other class keys:
    ///
    /// `@dec declare [x()]: T;` -> `@dec declare [_x]: T;`, and `_x = x()` is pushed to `assignments`.
    ///
    /// Semantic resolves the key's references as types where they are read, as in any `declare`
    /// field, so they become reads.
    fn convert_decorated_declare_field_key(
        prop: &mut PropertyDefinition<'a>,
        assignments: &mut Vec<Expression<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let Some(key) = prop.key.as_expression_mut() else { return };
        ValueReferenceReader(ctx.scoping_mut()).visit_expression(key);
        Self::move_key_to_temp_var(key, assignments, ctx);
    }

    /// Move a `key` with side effects to a temp var: `x()` -> `_x`, and `_x = x()` is pushed to
    /// `assignments`.
    fn move_key_to_temp_var(
        key: &mut Expression<'a>,
        assignments: &mut Vec<Expression<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if key_needs_temp_var(key, ctx) {
            let (assignment, ident) = create_computed_key_temp_var(key.take_in(ctx), ctx);
            assignments.push(assignment);
            *key = ident;
        }
    }

    /// Evaluate trailing keys after the other class keys and before static initialization.
    fn append_computed_key_carrier(
        &mut self,
        class: &mut Class<'a>,
        mut assignments: ArenaVec<'a, Expression<'a>>,
        property_assignments: &[Statement<'a>],
        lowering: ClassLowering,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<Expression<'a>> {
        let class_scope_id = class.scope_id();
        let mut decorators = ArenaVec::new_in(ctx);
        let (key, cleanup) = if lowering.has_decorators() {
            // Element decorators can inspect the class before its first static block. Evaluate
            // the keys in an inert decorator on a private method, which has no reflective key.
            let mut names = self.eval_private_names.clone();
            names.visit_class_body(&class.body);
            names.visit_expressions(&assignments);
            for statement in property_assignments {
                names.visit_statement(statement);
            }
            let name = names.generate(Str::from_str_in("_computedKey", ctx), ctx);
            let decorator_scope = ctx.create_child_scope(
                class_scope_id,
                ScopeFlags::StrictMode | ScopeFlags::Function | ScopeFlags::Arrow,
            );
            let noop = Expression::new_arrow_function_expression_with_scope_id_and_pure_and_pife(
                SPAN,
                false,
                None,
                FormalParameters::boxed(
                    SPAN,
                    FormalParameterKind::ArrowFormalParameters,
                    [],
                    None,
                    ctx,
                ),
                None,
                ArrowFunctionBody::FunctionBody(FunctionBody::boxed(SPAN, [], [], ctx)),
                decorator_scope,
                false,
                false,
                ctx,
            );
            assignments.push(noop);
            let expr = Expression::new_sequence_expression(SPAN, assignments, ctx);
            decorators.push(Decorator::new(SPAN, expr, ctx));
            (PropertyKey::new_private_identifier(SPAN, name, ctx), None)
        } else {
            let marker = self.computed_key_marker(ctx);
            assignments.push(marker.create_read_expression(ctx));
            let key = Expression::new_sequence_expression(SPAN, assignments, ctx);
            let member = Expression::new_computed_member_expression(
                SPAN,
                Expression::new_this_expression(SPAN, ctx),
                marker.create_read_expression(ctx),
                false,
                ctx,
            );
            let delete = Expression::new_unary_expression(SPAN, UnaryOperator::Delete, member, ctx);
            (PropertyKey::from(key), Some(delete))
        };
        let scope_id =
            ctx.create_child_scope(class_scope_id, ScopeFlags::StrictMode | ScopeFlags::Function);
        let params =
            FormalParameters::boxed(SPAN, FormalParameterKind::FormalParameter, [], None, ctx);
        class.body.body.push(create_class_method(
            SPAN,
            decorators,
            key,
            MethodDefinitionKind::Method,
            params,
            None,
            ArenaVec::new_in(ctx),
            true,
            scope_id,
            ctx,
        ));
        cleanup
    }

    /// `_computedKey`, a symbol that keys the methods from [`Self::append_computed_key_carrier`].
    /// It is created once at the top of the file, where a local `Symbol` binding can't shadow it.
    fn computed_key_marker(&mut self, ctx: &mut TraverseCtx<'a>) -> BoundIdentifier<'a> {
        self.computed_key_marker
            .get_or_insert_with(|| {
                let binding = ctx
                    .generate_uid_in_root_scope("computedKey", SymbolFlags::FunctionScopedVariable);
                let symbol = static_ident!("Symbol");
                let symbol_id = ctx.scoping().find_binding(ctx.scoping().root_scope_id(), symbol);
                let callee = ctx.create_ident_expr(SPAN, symbol, symbol_id, ReferenceFlags::Read);
                let init = Expression::new_call_expression(SPAN, callee, None, [], false, ctx);
                let declaration = create_variable_declaration(&binding, init, ctx);
                ctx.state.top_level_statements.insert_statement(declaration);
                binding
            })
            .clone()
    }

    /// Convert constructor parameters that include modifier to `this` assignments
    pub(super) fn convert_constructor_params(
        params: &ArenaVec<'a, FormalParameter<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) -> impl Iterator<Item = Statement<'a>> {
        let source_text = ctx.state.source_text;
        params
            .iter()
            .filter(|param| param.has_modifier())
            .filter_map(|param| param.pattern.get_binding_identifier())
            .map(|id| {
                // `id.name` may be renamed by clash detection; use span to recover the original source name.
                let prop_name = Ident::from(id.span.source_text(source_text));
                let target = create_this_property_assignment(id.span, prop_name, ctx);
                let value = BoundIdentifier::from_binding_ident(id).create_read_expression(ctx);
                Self::create_assignment(target, value, ctx)
            })
    }

    /// Create `a.b = value`
    fn create_assignment(
        target: AssignmentTarget<'a>,
        value: Expression<'a>,
        ctx: &TraverseCtx<'a>,
    ) -> Statement<'a> {
        Statement::new_expression_statement(
            SPAN,
            Expression::new_assignment_expression(
                SPAN,
                AssignmentOperator::Assign,
                target,
                value,
                ctx,
            ),
            ctx,
        )
    }

    /// Create `static { body }` as a child of `class_scope_id`.
    ///
    /// The enclosing traversal scope is the scope around the class, not the class body,
    /// so pass `class.scope_id()` explicitly to parent the static block correctly.
    #[inline]
    fn create_class_static_block(
        body: ArenaVec<'a, Statement<'a>>,
        class_scope_id: ScopeId,
        ctx: &mut TraverseCtx<'a>,
    ) -> ClassElement<'a> {
        let scope_id = ctx.insert_scope_below_statement_from_scope_id(
            &body[0],
            class_scope_id,
            ScopeFlags::StrictMode | ScopeFlags::ClassStaticBlock,
        );

        ClassElement::new_static_block_with_scope_id(SPAN, body, scope_id, ctx)
    }
}

/// Whether `expr`, from a computed key, means something else or is invalid in a static block.
fn depends_on_outer_context(expr: &Expression<'_>) -> bool {
    let mut finder = OuterContextFinder::default();
    finder.visit_expression(expr);
    finder.found
}

#[derive(Default)]
struct OuterContextFinder {
    found: bool,
}

impl<'a> Visit<'a> for OuterContextFinder {
    fn visit_expression(&mut self, expr: &Expression<'a>) {
        if !self.found {
            walk::walk_expression(self, expr);
        }
    }

    fn visit_this_expression(&mut self, _it: &ThisExpression) {
        self.found = true;
    }

    fn visit_super(&mut self, _it: &Super) {
        self.found = true;
    }

    fn visit_yield_expression(&mut self, _it: &YieldExpression<'a>) {
        self.found = true;
    }

    fn visit_await_expression(&mut self, _it: &AwaitExpression<'a>) {
        self.found = true;
    }

    fn visit_new_target(&mut self, _it: &NewTarget) {
        self.found = true;
    }

    fn visit_identifier_reference(&mut self, it: &IdentifierReference<'a>) {
        if it.name == "arguments" {
            self.found = true;
        }
    }

    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        // Direct `eval` sees the variables and `this` of where it's called
        if !it.optional && it.callee.is_specific_id("eval") {
            self.found = true;
            return;
        }
        walk::walk_call_expression(self, it);
    }

    fn visit_function(&mut self, _it: &Function<'a>, _flags: ScopeFlags) {
        // Functions have their own `this`, `arguments`, `new.target` and `super`,
        // and `yield` and `await` in them belong to them.
    }
}

/// Turns the type references of an expression into reads, skipping those in types (`x as T`,
/// `f<T>()`). References that are already values, such as the `Read | Write` of `x++`, are kept.
struct ValueReferenceReader<'s>(&'s mut Scoping);

impl<'a> Visit<'a> for ValueReferenceReader<'_> {
    fn visit_identifier_reference(&mut self, it: &IdentifierReference<'a>) {
        let flags = self.0.get_reference_mut(it.reference_id()).flags_mut();
        if !flags.is_value() {
            *flags = ReferenceFlags::Read;
        }
    }

    fn visit_ts_type(&mut self, _it: &TSType<'a>) {}
}
