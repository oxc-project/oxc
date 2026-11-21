//! Legacy decorator
//!
//! This plugin transforms legacy decorators by calling `_decorate` and `_decorateParam` helpers
//! to apply decorators.
//!
//! ## Examples
//!
//! Input:
//! ```ts
//! @dec
//! class Class {
//!   @dec
//!   prop = 0;
//!
//!   @dec
//!   method(@dec param) {}
//! }
//! ```
//!
//! Output:
//! ```js
//! let Class = class Class {
//!   prop = 0;
//!   method(param) {}
//! };
//!
//! _decorate([dec], Class.prototype, "method", null);
//!
//! _decorate([
//!   _decorateParam(0, dec)
//! ], Class.prototype, "method", null);
//!
//! Class = _decorate([dec], Class);
//! ```
//!
//! ## Implementation
//!
//! Implementation based on [TypeScript Experimental Decorators](https://github.com/microsoft/TypeScript/blob/d85767abfd83880cea17cea70f9913e9c4496dcc/src/compiler/transformers/legacyDecorators.ts).
//!
//! For testing, we have copied over all legacy decorator test cases from [TypeScript](https://github.com/microsoft/TypeScript/blob/d85767abfd83880cea17cea70f9913e9c4496dcc/tests/cases/conformance/decorators),
//! where the test cases are located in `./tasks/transform_conformance/tests/legacy-decorators/test/fixtures`.
//!
//! ## References:
//! * TypeScript Experimental Decorators documentation: <https://www.typescriptlang.org/docs/handbook/decorators.html>

mod metadata;

use std::{borrow::Cow, cell::Cell, mem};

use oxc_allocator::{
    Address, ArenaVec, CloneIn, GetAddress, GetAllocator, ReplaceWith, TakeIn, UnstableAddress,
};
use oxc_ast::{StaticPropertyName, ast::*};
use oxc_ast_visit::{VisitJs, VisitMut, walk_js};
use oxc_data_structures::stack::NonEmptyStack;
use oxc_semantic::{ReferenceFlags, ScopeFlags, ScopeId, Scoping, SymbolFlags, SymbolId};
use oxc_span::SPAN;
use oxc_syntax::operator::AssignmentOperator;
use oxc_traverse::{Ancestor, BoundIdentifier, Traverse, ast_operations::get_var_name_from_node};
use rustc_hash::FxHashMap;

use crate::{
    Helper,
    common::{
        duplicate::duplicate_expression, helper_loader::helper_call_expr,
        var_declarations::VarDeclarationsStore,
    },
    context::TraverseCtx,
    decorator::DecoratorOptions,
    state::TransformState,
    utils::ast_builder::{
        create_accessor_method, create_assignment, create_export_default_class_reference,
        create_export_named_class_reference, create_prototype_member,
    },
};
use metadata::LegacyDecoratorMetadata;

struct ClassDecoratedData<'a> {
    // Class binding. When a class is without binding, it will be `_default`,
    binding: BoundIdentifier<'a>,
    // Alias binding exist when the class body contains a reference that refers to class itself.
    alias_binding: Option<BoundIdentifier<'a>>,
}

/// Class decorations state for the current class being processed.
#[derive(Default)]
struct ClassDecorations<'a> {
    /// Flag indicating whether the current class needs to transform or not,
    /// `false` if the class is an expression or `declare`.
    should_transform: bool,
    /// Number of member decorations parked at the end of `class.decorators`, which are taken
    /// back on class exit. See [`LegacyDecorator::transform_member_decorators`].
    parked_member_decorations: usize,
    /// Member decorations that go in a static block, because a decorator refers to a private
    /// name of the class. The block is added in [`LegacyDecorator::enter_class_body`].
    member_decorations_for_block: Vec<Statement<'a>>,
    /// Scope of the static block that holds the member decorations, once added.
    member_decorations_block: Option<ScopeId>,
    /// Binding for the current class being processed.
    /// Generated on class entry when a class member is decorated.
    class_binding: Option<BoundIdentifier<'a>>,
}

pub struct LegacyDecorator<'a> {
    emit_decorator_metadata: bool,
    metadata: LegacyDecoratorMetadata<'a>,
    /// Decorated class data exists when a class or constructor is decorated.
    ///
    /// The data assigned in [`Self::transform_class`] and used in places where statements contain
    /// a decorated class declaration because decorated class needs to transform into `let c = class c {}`.
    /// The class and constructor decorators are transformed in `exit_class` rather than in
    /// [`Self::exit_statement`], because the `class-properties` plugin also transforms the class in
    /// `exit_class`, after which the decorators' nodes might be lost. Member decorators are
    /// transformed on class entry, see [`Self::transform_member_decorators`].
    class_decorated_data: Option<ClassDecoratedData<'a>>,
    /// Transformed decorators, they will be inserted in the statements at [`Self::exit_class_at_end`].
    decorations: FxHashMap<Address, Vec<Statement<'a>>>,
    /// Stack for managing nested class decoration state.
    /// Each level represents the decoration state for a class in the hierarchy,
    /// with the top being the currently processed class.
    class_decorations_stack: NonEmptyStack<ClassDecorations<'a>>,
}

impl LegacyDecorator<'_> {
    pub fn new(options: DecoratorOptions) -> Self {
        Self {
            emit_decorator_metadata: options.emit_decorator_metadata,
            metadata: LegacyDecoratorMetadata::new(options),
            class_decorated_data: None,
            decorations: FxHashMap::default(),
            class_decorations_stack: NonEmptyStack::new(ClassDecorations::default()),
        }
    }
}

impl<'a> Traverse<'a, TransformState<'a>> for LegacyDecorator<'a> {
    #[inline]
    fn exit_program(&mut self, node: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.emit_decorator_metadata {
            self.metadata.exit_program(node, ctx);
        }

        debug_assert!(
            self.class_decorations_stack.is_exhausted(),
            "All class decorations should have been popped."
        );
    }

    #[inline]
    fn enter_statements(
        &mut self,
        stmts: &mut ArenaVec<'a, Statement<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if self.emit_decorator_metadata {
            self.metadata.enter_statements(stmts, ctx);
        }
    }

    #[inline]
    fn enter_class(&mut self, class: &mut Class<'a>, ctx: &mut TraverseCtx<'a>) {
        // Lower `accessor` properties to private backing fields + get/set pairs.
        // Must run before `enter_class_body` so that es2022 class-properties can
        // transform the newly created private backing fields.
        Self::lower_accessor_properties(class, ctx);

        // Runs before member decorations are parked on `class.decorators`, as it checks whether
        // the class is decorated.
        if self.emit_decorator_metadata {
            self.metadata.enter_class(class, ctx);
        }

        let mut decorations = ClassDecorations {
            should_transform: !(class.is_expression() || class.declare),
            ..ClassDecorations::default()
        };
        self.transform_member_decorators(class, &mut decorations, ctx);
        self.class_decorations_stack.push(decorations);
    }

    /// Add the static block for member decorations that refer to a private name of the class.
    ///
    /// The block is added here rather than on class entry, after the class-properties and
    /// class-static-block transforms have entered the class body, so that they don't treat it as
    /// a static block written in the source (like other code added on class exit). Its contents
    /// are still visited in the class body.
    fn enter_class_body(&mut self, body: &mut ClassBody<'a>, ctx: &mut TraverseCtx<'a>) {
        let decorations = self.class_decorations_stack.last_mut();
        if decorations.member_decorations_for_block.is_empty() {
            return;
        }
        let stmts = mem::take(&mut decorations.member_decorations_for_block);
        let scope_id = Self::insert_decorations_into_class_static_block(body, stmts, ctx);
        self.class_decorations_stack.last_mut().member_decorations_block = Some(scope_id);
    }

    #[inline]
    fn exit_class(&mut self, class: &mut Class<'a>, ctx: &mut TraverseCtx<'a>) {
        self.transform_class(class, ctx);
    }

    // `#[inline]` because this is a hot path
    #[inline]
    fn exit_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        match stmt {
            Statement::ClassDeclaration(_) => self.transform_class_statement(stmt, ctx),
            Statement::ExportDeclaration(_) => {
                self.transform_export_named_class(stmt, ctx);
            }
            Statement::ExportDefaultDeclaration(_) => {
                self.transform_export_default_class(stmt, ctx);
            }
            _ => {}
        }
    }
}

impl<'a> LegacyDecorator<'a> {
    /// Lower `accessor` properties to private backing fields + get/set pairs.
    ///
    /// `accessor prop: T = val` becomes:
    /// ```js
    /// #prop_accessor_storage = val;
    /// get prop() { return this.#prop_accessor_storage; }
    /// set prop(value) { this.#prop_accessor_storage = value; }
    /// ```
    fn lower_accessor_properties(class: &mut Class<'a>, ctx: &mut TraverseCtx<'a>) {
        if !class
            .body
            .body
            .iter()
            .any(|e| matches!(e, ClassElement::AccessorProperty(p) if !p.r#type.is_abstract()))
        {
            return;
        }

        // For static accessors, use the class name instead of `this` to ensure
        // correct behavior when accessed via subclasses.
        // Named class: `C.#storage`, Anonymous class expression: `_a.#storage`
        let has_static_accessor = class.body.body.iter().any(|e| {
            matches!(e, ClassElement::AccessorProperty(p) if p.r#static && !p.r#type.is_abstract())
        });
        let static_class_binding = if has_static_accessor {
            Some(if let Some(ident) = class.id.as_ref() {
                BoundIdentifier::from_binding_ident(ident)
            } else {
                ctx.generate_uid("class", class.scope_id(), SymbolFlags::Class)
            })
        } else {
            None
        };

        let class_scope_id = class.scope_id();
        let mut new_body = ArenaVec::with_capacity_in(class.body.body.len() * 3, ctx);

        for element in class.body.body.drain(..) {
            let ClassElement::AccessorProperty(accessor) = element else {
                new_body.push(element);
                continue;
            };
            if accessor.r#type.is_abstract() {
                new_body.push(ClassElement::AccessorProperty(accessor));
                continue;
            }

            let mut accessor = accessor.unbox();
            let is_static = accessor.r#static;

            // Transfer decorators to the getter so legacy decorator transform can process them.
            let decorators = std::mem::replace(&mut accessor.decorators, ArenaVec::new_in(ctx));
            // Transfer the type annotation to the getter's return type so that
            // `emitDecoratorMetadata` can derive `design:type` from it. Without this,
            // the lowered getter is untyped and `design:type` falls through to `Object`.
            let type_annotation = accessor.type_annotation.take();

            // For computed keys, duplicate the key expression so getter and setter
            // each get their own reference:
            //   `accessor [expr] = val` →
            //   `get [_expr = expr]() { ... } set [expr](value) { ... }`
            // Note: Legacy decorators do not support private identifiers,
            // so we only handle static identifiers and computed keys.
            let (storage_name, getter_key, setter_key) = if accessor.key.is_computed() {
                let key_expr = accessor.key.into_expression();
                let (assignment, reference) = duplicate_expression(key_expr, true, ctx);

                // Use raw identifier name when possible to avoid collisions from
                // `get_var_name_from_node` stripping leading underscores.
                let key_name: Cow<'_, str> = match &reference {
                    Expression::Identifier(ident) => Cow::Borrowed(ident.name.as_str()),
                    _ => Cow::Owned(get_var_name_from_node(&reference)),
                };
                let getter_key = PropertyKey::from(assignment);
                let setter_key = PropertyKey::from(reference);
                let storage_name =
                    Str::from_strs_array_in(["_", &key_name, "_computed_accessor_storage"], ctx);
                (storage_name, getter_key, setter_key)
            } else {
                // Use `name()` to get the raw property name, avoiding `get_var_name_from_node`
                // which strips leading underscores (e.g. `prop` and `_prop` both become "prop").
                let key_name = accessor
                    .key
                    .name()
                    .and_then(StaticPropertyName::into_cow_str)
                    .unwrap_or_else(|| Cow::Owned(get_var_name_from_node(&accessor.key)));
                let storage_name =
                    Str::from_strs_array_in(["_", &key_name, "_accessor_storage"], ctx);
                let getter_key = accessor.key.clone_in(ctx.ast.allocator());
                let setter_key = accessor.key.clone_in(ctx.ast.allocator());
                (storage_name, getter_key, setter_key)
            };

            // For static accessors, use class name reference; for instance accessors, use `this`.
            let object_binding = if is_static { static_class_binding.as_ref() } else { None };

            // 1. Private backing field: `#<name>_accessor_storage = <value>`
            new_body.push(ClassElement::new_property_definition(
                SPAN,
                PropertyDefinitionType::PropertyDefinition,
                [],
                PropertyKey::new_private_identifier(SPAN, storage_name, ctx),
                None,
                accessor.value.take(),
                is_static,
                false,
                false,
                false,
                false,
                false,
                None,
                ctx,
            ));

            // 2. Getter: `get <name>() { return <object>.#<name>_accessor_storage; }`
            new_body.push(create_accessor_method(
                decorators,
                getter_key,
                MethodDefinitionKind::Get,
                is_static,
                storage_name,
                object_binding,
                class_scope_id,
                type_annotation,
                ctx,
            ));

            // 3. Setter: `set <name>(value) { <object>.#<name>_accessor_storage = value; }`
            new_body.push(create_accessor_method(
                ArenaVec::new_in(ctx),
                setter_key,
                MethodDefinitionKind::Set,
                is_static,
                storage_name,
                object_binding,
                class_scope_id,
                None,
                ctx,
            ));
        }

        // For anonymous class expressions with static accessors, assign the class
        // to the generated binding: `var _class; const c = _class = class { ... }`
        if let Some(binding) = &static_class_binding
            && class.id.is_none()
        {
            class.id = Some(binding.create_binding_identifier(ctx));
        }

        class.body.body = new_body;
    }

    /// Transform the decorators of class members (fields, methods, getters and setters, and the
    /// parameters of methods) into `_decorate(...)` calls on class entry, and take them off the
    /// members.
    ///
    /// Doing this before the TypeScript and class-properties transforms see the class means they
    /// only see undecorated members. They can erase a decorated field without an initializer like
    /// any other, and rewrite fields without losing a decoration or its computed key. Computed keys
    /// are replaced with an assignment to a temp var (`[_a = foo()]`), so they stay in the class and
    /// are still evaluated once, in order.
    ///
    /// The other transforms must still visit the calls, as decorators can contain TypeScript
    /// syntax, private names, and other syntax to lower. So the calls are parked where they will be
    /// evaluated:
    ///
    /// * Usually after the class. The calls are appended to `class.decorators`, which are visited
    ///   outside the class, and taken back on class exit (see [`Self::take_member_decorations`]).
    /// * In a static block at the end of the class body if a decorator refers to a private name,
    ///   such as `#x in obj` or `obj.#x`, which can only be evaluated inside the class.
    ///
    /// TypeScript applies every instance member decoration first, then every static one.
    fn transform_member_decorators(
        &mut self,
        class: &mut Class<'a>,
        decorations: &mut ClassDecorations<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        // Legacy decorators are not applied to class expressions or `declare` classes, so their
        // member decorators are dropped.
        if !decorations.should_transform {
            Self::drop_member_decorators(class);
            return;
        }

        let mut instance_calls = vec![];
        let mut static_calls = vec![];
        let mut private_name_detector = PrivateNameDetector::new(&class.body);
        let class_id = class.id.as_ref();
        let class_scope_id = class.scope_id();

        for element in &mut class.body.body {
            let (is_static, key, erased, descriptor, decorators) = match element {
                ClassElement::PropertyDefinition(prop) => {
                    if prop.decorators.is_empty() {
                        continue;
                    }
                    let decorators = prop.decorators.take_in(ctx);
                    private_name_detector.visit_decorators(&decorators);
                    let metadata = self.emit_decorator_metadata.then(|| {
                        self.metadata
                            .create_design_type_metadata(prop.type_annotation.as_ref(), ctx)
                    });
                    let decorators = Self::create_decorations_array(
                        decorators
                            .into_iter()
                            .map(|decorator| decorator.expression)
                            .chain(metadata),
                        ctx,
                    );
                    // `void 0` tells `_decorate` to invoke `Object.defineProperty` directly.
                    let descriptor = Expression::new_void_0(SPAN, ctx);
                    // TypeScript erases `declare` and abstract fields, and with them a computed
                    // key rewritten to `[_a = foo()]`. Like TypeScript, pass the key expression
                    // to `_decorate` instead.
                    let erased = prop.declare
                        || prop.r#type == PropertyDefinitionType::TSAbstractPropertyDefinition;
                    (prop.r#static, &mut prop.key, erased, descriptor, decorators)
                }
                ClassElement::MethodDefinition(method) => {
                    // TypeScript erases methods without a body, such as overloads.
                    if method.value.is_typescript_syntax() {
                        continue;
                    }
                    // The constructor's parameters are decorated with the class, on class exit.
                    if method.kind.is_constructor() {
                        continue;
                    }
                    private_name_detector.visit_method_decorators(method);
                    let Some(decorators) = self.get_all_decorators_of_class_method(method, ctx)
                    else {
                        continue;
                    };
                    // `null` tells `_decorate` to invoke `Object.getOwnPropertyDescriptor` directly.
                    let descriptor = Expression::new_null_literal(SPAN, ctx);
                    (method.r#static, &mut method.key, false, descriptor, decorators)
                }
                _ => continue,
            };

            let class_binding = decorations.class_binding.get_or_insert_with(|| {
                if let Some(ident) = class_id {
                    BoundIdentifier::from_binding_ident(ident)
                } else {
                    let outer_scope_id = ctx.scoping().scope_parent_id(class_scope_id).unwrap();
                    ctx.generate_uid("default", outer_scope_id, SymbolFlags::Class)
                }
            });
            let prefix = Self::get_class_member_prefix(class_binding, is_static, ctx);
            let name = if erased && let Some(key) = key.as_expression_mut() {
                // References in the key of a `declare` field are type-only so far.
                ValueReferences { scoping: ctx.scoping_mut() }.visit_expression(key);
                key.take_in(ctx)
            } else {
                Self::get_name_of_property_key(key, ctx)
            };
            let call = Self::create_decorate_call(decorators, prefix, name, descriptor, ctx);
            if is_static { static_calls.push(call) } else { instance_calls.push(call) }
        }

        if instance_calls.is_empty() && static_calls.is_empty() {
            return;
        }
        instance_calls.append(&mut static_calls);

        if private_name_detector.found {
            decorations.member_decorations_for_block = instance_calls
                .into_iter()
                .map(|call| Statement::new_expression_statement(SPAN, call, ctx))
                .collect();
        } else {
            // The calls are evaluated after the class.
            let outer_scope_id = ctx.scoping().scope_parent_id(class.scope_id()).unwrap();
            let mut reparenter = ChildScopeReparenter::new(outer_scope_id, ctx);
            for call in &instance_calls {
                reparenter.visit_expression(call);
            }
            decorations.parked_member_decorations = instance_calls.len();
            class
                .decorators
                .extend(instance_calls.into_iter().map(|call| Decorator::new(SPAN, call, ctx)));
        }
    }

    /// Drop the decorators of class members, except the constructor's parameters, which are
    /// handled on class exit.
    fn drop_member_decorators(class: &mut Class<'a>) {
        for element in &mut class.body.body {
            match element {
                ClassElement::PropertyDefinition(prop) => prop.decorators.clear(),
                ClassElement::MethodDefinition(method)
                    if !method.kind.is_constructor() && !method.value.is_typescript_syntax() =>
                {
                    method.decorators.clear();
                    for param in &mut method.value.params.items {
                        param.decorators.clear();
                    }
                }
                _ => {}
            }
        }
    }

    /// Take back the member decorations parked on `class.decorators` by
    /// [`Self::transform_member_decorators`], once the other transforms have visited them.
    fn take_member_decorations(
        class: &mut Class<'a>,
        parked_member_decorations: usize,
        ctx: &TraverseCtx<'a>,
    ) -> Vec<Statement<'a>> {
        if parked_member_decorations == 0 {
            return vec![];
        }
        let start = class.decorators.len() - parked_member_decorations;
        class
            .decorators
            .drain(start..)
            .map(|decorator| Statement::new_expression_statement(SPAN, decorator.expression, ctx))
            .collect()
    }

    /// Transforms a statement that is a class declaration
    ///
    ///
    /// Input:
    /// ```ts
    /// @dec
    /// class Class {
    ///   method(@dec param) {}
    /// }
    /// ```
    ///
    /// Output:
    /// ```js
    /// let Class = class Class {
    ///   method(param) { }
    /// };
    ///
    /// _decorate([
    ///   _decorateParam(0, dec)
    /// ], Class.prototype, "method", null);
    ///
    /// Class = _decorate([
    ///   dec
    /// ], Class);
    /// ```
    // `#[inline]` so that compiler sees that `stmt` is a `Statement::ClassDeclaration`.
    #[inline]
    fn transform_class_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        let Statement::ClassDeclaration(class) = stmt else { unreachable!() };

        let Some(ClassDecoratedData { binding, alias_binding }) = self.class_decorated_data.take()
        else {
            return;
        };

        let new_stmt =
            Self::transform_class_decorated(class, &binding, alias_binding.as_ref(), ctx);

        ctx.state.statement_injector.move_insertions(stmt, &new_stmt);
        *stmt = new_stmt;
    }

    /// Transforms a statement that is a export default class declaration
    ///
    /// Input:
    /// ```ts
    /// @dec
    /// export default class Class {
    ///   method(@dec param) {}
    /// }
    /// ```
    ///
    /// Output:
    /// ```js
    /// let Class = class Class {
    ///   method(param) { }
    /// };
    ///
    /// _decorate([
    ///   _decorateParam(0, dec)
    /// ], Class.prototype, "method", null);
    ///
    /// Class = _decorate([
    ///   dec
    /// ], Class);
    ///
    /// export default Class;
    /// ```
    // `#[inline]` so that compiler sees that `stmt` is a `Statement::ExportDefaultDeclaration`.
    #[inline]
    fn transform_export_default_class(
        &mut self,
        stmt: &mut Statement<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let Statement::ExportDefaultDeclaration(export) = stmt else { unreachable!() };
        let ExportDefaultDeclarationKind::ClassDeclaration(class) = &mut export.declaration else {
            return;
        };
        let Some(ClassDecoratedData { binding, alias_binding }) = self.class_decorated_data.take()
        else {
            return;
        };

        let new_stmt =
            Self::transform_class_decorated(class, &binding, alias_binding.as_ref(), ctx);

        // `export default Class`
        let export_default_class_reference = create_export_default_class_reference(&binding, ctx);
        ctx.state.statement_injector.move_insertions(stmt, &new_stmt);
        ctx.state.statement_injector.insert_after(&new_stmt, export_default_class_reference);
        *stmt = new_stmt;
    }

    /// Transforms a statement that is a export named class declaration
    ///
    /// Input:
    /// ```ts
    /// @dec
    /// export class Class {
    ///   method(@dec param) {}
    /// }
    /// ```
    ///
    /// Output:
    /// ```js
    /// let Class = class Class {
    ///   method(param) { }
    /// };
    ///
    /// _decorate([
    ///   _decorateParam(0, dec)
    /// ], Class.prototype, "method", null);
    ///
    /// Class = _decorate([
    ///   dec
    /// ], Class);
    ///
    /// export { Class };
    /// ```
    // `#[inline]` so that compiler sees that `stmt` is a `Statement::ExportDeclaration`.
    #[inline]
    fn transform_export_named_class(
        &mut self,
        stmt: &mut Statement<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let Statement::ExportDeclaration(export) = stmt else { unreachable!() };
        let Declaration::ClassDeclaration(class) = &mut export.declaration else { return };

        let Some(ClassDecoratedData { binding, alias_binding }) = self.class_decorated_data.take()
        else {
            return;
        };

        let new_stmt =
            Self::transform_class_decorated(class, &binding, alias_binding.as_ref(), ctx);

        // `export { Class }`
        let export_class_reference = create_export_named_class_reference(&binding, ctx);
        ctx.state.statement_injector.move_insertions(stmt, &new_stmt);
        ctx.state.statement_injector.insert_after(&new_stmt, export_class_reference);
        *stmt = new_stmt;
    }

    fn transform_class(&mut self, class: &mut Class<'a>, ctx: &mut TraverseCtx<'a>) {
        let current_class_decorations = self.class_decorations_stack.pop();
        let decoration_stmts = Self::take_member_decorations(
            class,
            current_class_decorations.parked_member_decorations,
            ctx,
        );

        // Legacy decorator does not allow in class expression.
        if current_class_decorations.should_transform {
            let class_or_constructor_parameter_is_decorated =
                Self::check_class_has_decorated(class);

            if class_or_constructor_parameter_is_decorated {
                self.transform_class_declaration_with_class_decorators(
                    class,
                    current_class_decorations,
                    decoration_stmts,
                    ctx,
                );
                return;
            } else if current_class_decorations.class_binding.is_some() {
                // A class member is decorated.
                self.transform_class_declaration_without_class_decorators(
                    class,
                    current_class_decorations,
                    decoration_stmts,
                    ctx,
                );
            }
        } else {
            debug_assert!(
                current_class_decorations.class_binding.is_none(),
                "Legacy decorator does not allow class expression, so that it should not have class binding."
            );
        }

        if self.emit_decorator_metadata {
            let metadata = self.metadata.pop_constructor_metadata();
            debug_assert!(
                metadata.is_none(),
                "`pop_constructor_metadata` should be `None` because there are no class decorators, so no metadata was generated."
            );
        }
    }

    /// Transforms a decorated class declaration and appends the resulting statements. If
    /// the class requires an alias to avoid issues with double-binding, the alias is returned.
    fn transform_class_declaration_with_class_decorators(
        &mut self,
        class: &mut Class<'a>,
        current_class_decorations: ClassDecorations<'a>,
        mut decoration_stmts: Vec<Statement<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        // When we emit an ES6 class that has a class decorator, we must tailor the
        // emit to certain specific cases.
        //
        // In the simplest case, we emit the class declaration as a let declaration, and
        // evaluate decorators after the close of the class body:
        //
        //  [Example 1]
        //  ---------------------------------------------------------------------
        //  TypeScript                      | Javascript
        //  ---------------------------------------------------------------------
        //  @dec                            | let C = class C {
        //  class C {                       | }
        //  }                               | C = _decorate([dec], C);
        //  ---------------------------------------------------------------------
        //  @dec                            | let C = class C {
        //  export class C {                | }
        //  }                               | C = _decorate([dec], C);
        //                                  | export { C };
        //  ---------------------------------------------------------------------
        //
        // If a class declaration contains a reference to itself *inside* of the class body,
        // this introduces two bindings to the class: One outside of the class body, and one
        // inside of the class body. If we apply decorators as in [Example 1] above, there
        // is the possibility that the decorator `dec` will return a new value for the
        // constructor, which would result in the binding inside of the class no longer
        // pointing to the same reference as the binding outside of the class.
        //
        // As a result, we must instead rewrite all references to the class *inside* of the
        // class body to instead point to a local temporary alias for the class:
        //
        //  [Example 2]
        //  ---------------------------------------------------------------------
        //  TypeScript                      | Javascript
        //  ---------------------------------------------------------------------
        //  @dec                            | let C = C_1 = class C {
        //  class C {                       |   static x() { return C_1.y; }
        //    static x() { return C.y; }    | }
        //    static y = 1;                 | C.y = 1;
        //  }                               | C = C_1 = _decorate([dec], C);
        //                                  | var C_1;
        //  ---------------------------------------------------------------------
        //  @dec                            | let C = class C {
        //  export class C {                |   static x() { return C_1.y; }
        //    static x() { return C.y; }    | }
        //    static y = 1;                 | C.y = 1;
        //  }                               | C = C_1 = _decorate([dec], C);
        //                                  | export { C };
        //                                  | var C_1;
        //  ---------------------------------------------------------------------
        //
        // If a class declaration is the default export of a module, we instead emit
        // the export after the decorated declaration:
        //
        //  [Example 3]
        //  ---------------------------------------------------------------------
        //  TypeScript                      | Javascript
        //  ---------------------------------------------------------------------
        //  @dec                            | let default_1 = class {
        //  export default class {          | }
        //  }                               | default_1 = _decorate([dec], default_1);
        //                                  | export default default_1;
        //  ---------------------------------------------------------------------
        //  @dec                            | let C = class C {
        //  export default class C {        | }
        //  }                               | C = _decorate([dec], C);
        //                                  | export default C;
        //  ---------------------------------------------------------------------
        //
        // If the class declaration is the default export and a reference to itself
        // inside of the class body, we must emit both an alias for the class *and*
        // move the export after the declaration:
        //
        //  [Example 4]
        //  ---------------------------------------------------------------------
        //  TypeScript                      | Javascript
        //  ---------------------------------------------------------------------
        //  @dec                            | let C = class C {
        //  export default class C {        |   static x() { return C_1.y; }
        //    static x() { return C.y; }    | }
        //    static y = 1;                 | C.y = 1;
        //  }                               | C = C_1 = _decorate([dec], C);
        //                                  | export default C;
        //                                  | var C_1;
        //  ---------------------------------------------------------------------
        //

        // TODO(improve-on-typescript): we can take the class id without keeping it as-is.
        // Now: `class C {}` -> `let C = class C {}`
        // After: `class C {}` -> `let C = class {}`
        let class_binding = class.id.as_ref().map(|ident| {
            let binding_span = ident.span;
            let new_class_binding = ctx.generate_spanned_binding(
                ident.name,
                binding_span,
                class.scope_id(),
                SymbolFlags::Class,
            );
            let old_class_symbol_id = ident.symbol_id.replace(Some(new_class_binding.symbol_id));
            let old_class_symbol_id = old_class_symbol_id.expect("class always has a symbol id");

            ctx.scoping_mut().set_symbol_span(old_class_symbol_id, binding_span);
            *ctx.scoping_mut().symbol_flags_mut(old_class_symbol_id) =
                SymbolFlags::BlockScopedVariable;
            BoundIdentifier::new(ident.name, old_class_symbol_id)
        });
        let inner_class_symbol_id = class.id.as_ref().map(BindingIdentifier::symbol_id);
        let class_alias_binding = class_binding.as_ref().and_then(|id| {
            ClassReferenceChanger::new(
                id.clone(),
                inner_class_symbol_id.zip(current_class_decorations.member_decorations_block),
                ctx,
            )
            .get_class_alias_if_needed(&mut class.body)
        });

        let ClassDecorations {
            class_binding: class_binding_tmp,
            parked_member_decorations: _,
            member_decorations_for_block: _,
            member_decorations_block: _,
            should_transform: _,
        } = current_class_decorations;

        let class_binding = class_binding.unwrap_or_else(|| {
            // `class_binding_tmp` maybe already generated a default class binding for unnamed classes, so use it.
            class_binding_tmp
                .unwrap_or_else(|| ctx.generate_uid_in_current_scope("default", SymbolFlags::Class))
        });

        let constructor_decoration = self.transform_decorators_of_class_and_constructor(
            class,
            &class_binding,
            class_alias_binding.as_ref(),
            ctx,
        );

        let class_alias_with_this_assignment =
            if ctx.state.class_lowering_of(class.scope_id()).lowers_fields() {
                None
            } else {
                // If we're emitting to ES2022 or later then we need to reassign the class alias before
                // static initializers are evaluated.
                // <https://github.com/microsoft/TypeScript/blob/b86ab7dbe0eb2f1c9a624486d72590d638495c97/src/compiler/transformers/legacyDecorators.ts#L345-L366>
                class_alias_binding.as_ref().and_then(|class_alias_binding| {
                    let has_static_field_or_block = class.body.body.iter().any(|element| {
                        matches!(element, ClassElement::StaticBlock(_))
                            || matches!(element, ClassElement::PropertyDefinition(prop)
                                    if prop.r#static
                            )
                    });

                    if has_static_field_or_block {
                        // `_Class = this`;
                        let class_alias_with_this_assignment = Statement::new_expression_statement(
                            SPAN,
                            create_assignment(
                                class_alias_binding,
                                Expression::new_this_expression(SPAN, ctx),
                                SPAN,
                                ctx,
                            ),
                            ctx,
                        );
                        let body = [class_alias_with_this_assignment];
                        let scope_id = ctx.create_child_scope(
                            class.scope_id(),
                            ScopeFlags::StrictMode | ScopeFlags::ClassStaticBlock,
                        );
                        let element =
                            ClassElement::new_static_block_with_scope_id(SPAN, body, scope_id, ctx);
                        Some(element)
                    } else {
                        None
                    }
                })
            };

        let address = match ctx.parent() {
            parent @ (Ancestor::ExportDefaultDeclarationDeclaration(_)
            | Ancestor::ExportDeclarationDeclaration(_)) => parent.address(),
            // `Class` is always stored in a `Box`, so has a stable memory location
            _ => class.unstable_address(),
        };

        decoration_stmts.push(constructor_decoration);
        self.decorations.entry(address).or_default().append(&mut decoration_stmts);
        self.class_decorated_data = Some(ClassDecoratedData {
            binding: class_binding,
            // If the class alias has reassigned to `this` in the static block, then
            // don't assign `class` to the class alias again.
            //
            // * class_alias_with_this_assignment is `None`:
            //   `Class = _Class = class Class {}`
            // * class_alias_with_this_assignment is `Some`:
            //   `Class = class Class { static { _Class = this; } }`
            alias_binding: if class_alias_with_this_assignment.is_none() {
                class_alias_binding
            } else {
                None
            },
        });

        if let Some(class_alias_with_this_assignment) = class_alias_with_this_assignment {
            class.body.body.insert(0, class_alias_with_this_assignment);
        }
    }

    /// Transform class to a [`VariableDeclarator`], whose binding name is the same as class.
    ///
    /// * `alias_binding` is `None`: `class C {}` -> `let C = class C {}`
    /// * `alias_binding` is `Some`: `class C {}` -> `let C = _C = class C {}`
    fn transform_class_decorated(
        class: &mut Class<'a>,
        binding: &BoundIdentifier<'a>,
        alias_binding: Option<&BoundIdentifier<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Statement<'a> {
        let span = class.span;
        let binding_span = ctx.scoping().symbol_span(binding.symbol_id);
        class.r#type = ClassType::ClassExpression;
        let initializer = Self::get_class_initializer(
            Expression::ClassExpression(class.take_in_box(ctx)),
            alias_binding,
            ctx,
        );
        let declarator = VariableDeclarator::new(
            SPAN,
            binding.create_spanned_binding_pattern(binding_span, ctx),
            None,
            Some(initializer),
            false,
            ctx,
        );
        let var_declaration = Declaration::new_variable_declaration(
            span,
            VariableDeclarationKind::Let,
            [declarator],
            false,
            ctx,
        );
        Statement::from(var_declaration)
    }

    /// Transforms a non-decorated class declaration.
    fn transform_class_declaration_without_class_decorators(
        &mut self,
        class: &mut Class<'a>,
        current_class_decorations: ClassDecorations<'a>,
        mut decoration_stmts: Vec<Statement<'a>>,
        ctx: &TraverseCtx<'a>,
    ) {
        let ClassDecorations {
            class_binding,
            parked_member_decorations: _,
            member_decorations_for_block: _,
            member_decorations_block: _,
            should_transform: _,
        } = current_class_decorations;

        let Some(class_binding) = class_binding else {
            unreachable!(
                "Always has a class binding because there are decorators in class elements,
                so that it has been added in `transform_member_decorators`"
            );
        };

        // No class id, add one by using the class binding
        if class.id.is_none() {
            class.id = Some(class_binding.create_binding_identifier(ctx));
        }

        // Empty when the member decorations are in a static block.
        if !decoration_stmts.is_empty() {
            let stmt_address = match ctx.parent() {
                parent @ (Ancestor::ExportDefaultDeclarationDeclaration(_)
                | Ancestor::ExportDeclarationDeclaration(_)) => parent.address(),
                // `Class` is always stored in a `Box`, so has a stable memory location
                _ => class.unstable_address(),
            };
            self.decorations.entry(stmt_address).or_default().append(&mut decoration_stmts);
        }
    }

    /// Transform the decorators of class and constructor method.
    ///
    /// Input:
    /// ```ts
    /// @dec
    /// class Class {
    ///   method(@dec param) {}
    /// }
    /// ```
    ///
    /// These decorators transform into:
    /// ```rust,ignore
    /// _decorate([
    ///   _decorateParam(0, dec)
    ///   ], Class.prototype, "method", null);
    ///
    /// Class = _decorate([
    ///   dec
    /// ], Class);
    /// ```
    fn transform_decorators_of_class_and_constructor(
        &mut self,
        class: &mut Class<'a>,
        class_binding: &BoundIdentifier<'a>,
        class_alias_binding: Option<&BoundIdentifier<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Statement<'a> {
        // Find first constructor method from the class
        let constructor = class.body.body.iter_mut().find_map(|element| match element {
            ClassElement::MethodDefinition(method) if method.kind.is_constructor() => Some(method),
            _ => None,
        });

        let decorations = if let Some(constructor) = constructor {
            // Constructor cannot have decorators, swap decorators of class and constructor to use
            // `get_all_decorators_of_class_method` to get all decorators of the class and constructor params
            mem::swap(&mut class.decorators, &mut constructor.decorators);
            //  constructor.decorators
            self.get_all_decorators_of_class_method(constructor, ctx)
                .expect("At least one decorator")
        } else {
            debug_assert!(
                !self.emit_decorator_metadata || self.metadata.pop_constructor_metadata().is_none(),
                "`pop_constructor_metadata` should be `None` because there is no `constructor`, so no metadata was generated."
            );
            Self::create_decorations_array(
                class.decorators.drain(..).map(|decorator| decorator.expression),
                ctx,
            )
        };

        // `Class = _decorate(decorations, Class)`
        let arguments = ArenaVec::from_array_in(
            [
                Argument::from(decorations),
                Argument::from(class_binding.create_read_expression(ctx)),
            ],
            ctx,
        );
        let helper = helper_call_expr(Helper::Decorate, arguments, ctx);
        let operator = AssignmentOperator::Assign;
        let left = class_binding.create_write_target(ctx);
        let right = Self::get_class_initializer(helper, class_alias_binding, ctx);
        let assignment = Expression::new_assignment_expression(SPAN, operator, left, right, ctx);
        Statement::new_expression_statement(SPAN, assignment, ctx)
    }

    /// Insert all decorations into a static block of a class because there is a
    /// private-in expression in the decorator.
    ///
    /// Input:
    /// ```ts
    /// class Class {
    ///   #a =0;
    ///   @(#a in Class ? dec() : dec2())
    ///   prop = 0;
    /// }
    /// ```
    ///
    /// Output:
    /// ```js
    /// class Class {
    ///   #a = 0;
    ///   prop = 0;
    ///   static {
    ///     _decorate([
    ///         (#a in Class ? dec() : dec2())
    ///     ], Class.prototype, "prop", void 0);
    ///   }
    /// }
    /// ```
    fn insert_decorations_into_class_static_block(
        body: &mut ClassBody<'a>,
        decorations: Vec<Statement<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) -> ScopeId {
        // Called on class body entry, so the current scope is the class's.
        let scope_id = ctx
            .create_child_scope_of_current(ScopeFlags::StrictMode | ScopeFlags::ClassStaticBlock);
        let mut reparenter = ChildScopeReparenter::new(scope_id, ctx);
        for stmt in &decorations {
            reparenter.visit_statement(stmt);
        }
        let decorations = ArenaVec::from_iter_in(decorations, ctx);
        let element =
            ClassElement::new_static_block_with_scope_id(SPAN, decorations, scope_id, ctx);
        body.body.push(element);
        scope_id
    }

    /// Transforms the decorators of the parameters of a class method.
    #[expect(clippy::cast_precision_loss, clippy::unused_self)]
    fn transform_decorators_of_parameters(
        &self,
        decorations: &mut ArenaVec<'a, ArrayExpressionElement<'a>>,
        params: &mut FormalParameters<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        for (index, param) in &mut params.items.iter_mut().enumerate() {
            if param.decorators.is_empty() {
                continue;
            }
            decorations.extend(param.decorators.drain(..).map(|decorator| {
                // (index, decorator)
                let index = Expression::new_numeric_literal(SPAN, index as f64, None, ctx);
                let arguments = ArenaVec::from_array_in(
                    [Argument::from(index), Argument::from(decorator.expression)],
                    ctx,
                );
                // _decorateParam(index, decorator)
                ArrayExpressionElement::from(helper_call_expr(
                    Helper::DecorateParam,
                    arguments,
                    ctx,
                ))
            }));
        }
    }

    /// Injects the class decorator statements after class-properties plugin has run, ensuring that
    /// all transformed fields are injected before the class decorator statements.
    pub fn exit_class_at_end(&mut self, _class: &mut Class<'a>, ctx: &mut TraverseCtx<'a>) {
        for (address, stmts) in mem::take(&mut self.decorations) {
            ctx.state.statement_injector.insert_many_after(&address, stmts);
        }
    }

    /// Converts decorator expressions to [`Expression::ArrayExpression`].
    fn create_decorations_array(
        decorations: impl Iterator<Item = Expression<'a>>,
        ctx: &TraverseCtx<'a>,
    ) -> Expression<'a> {
        let decorations =
            ArenaVec::from_iter_in(decorations.map(ArrayExpressionElement::from), ctx);
        Expression::new_array_expression(SPAN, decorations, ctx)
    }

    /// Get all decorators of a class method.
    ///
    /// ```ts
    /// class Class {
    ///   @dec
    ///   method(@dec param) {}
    /// }
    /// ```
    ///
    /// Returns:
    /// ```js
    /// [
    ///   dec,
    ///   _decorateParam(0, dec)
    /// ]
    /// ```
    fn get_all_decorators_of_class_method(
        &mut self,
        method: &mut MethodDefinition<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<Expression<'a>> {
        let params = &mut method.value.params;
        let param_decoration_count =
            params.items.iter().fold(0, |acc, param| acc + param.decorators.len());
        let method_decoration_count = method.decorators.len() + param_decoration_count;

        if method_decoration_count == 0 {
            if self.emit_decorator_metadata && method.kind.is_constructor() {
                debug_assert!(
                    self.metadata.pop_constructor_metadata().is_none(),
                    "No method decorators, so `pop_constructor_metadata` should be `None`"
                );
            }
            return None;
        }

        let mut decorations = ArenaVec::with_capacity_in(method_decoration_count, ctx);

        // Method decorators should always be injected before all other decorators
        decorations.extend(
            method
                .decorators
                .take_in(ctx)
                .into_iter()
                .map(|decorator| ArrayExpressionElement::from(decorator.expression)),
        );

        // The decorators of params are always inserted at the end if any.
        if param_decoration_count > 0 {
            self.transform_decorators_of_parameters(&mut decorations, params, ctx);
        }

        if self.emit_decorator_metadata {
            // `decorateMetadata` should always be injected after param decorators
            if method.kind.is_constructor() {
                if let Some(metadata) = self.metadata.pop_constructor_metadata() {
                    decorations.push(ArrayExpressionElement::from(metadata));
                }
            } else {
                let metadata = self.metadata.method_metadata(method, ctx);
                decorations.push(ArrayExpressionElement::from(metadata.r#type));
                decorations.push(ArrayExpressionElement::from(metadata.param_types));
                if let Some(return_type) = metadata.return_type {
                    decorations.push(ArrayExpressionElement::from(return_type));
                }
            }
        }

        Some(Expression::new_array_expression(SPAN, decorations, ctx))
    }

    /// * class_alias_binding is `Some`: `Class = _Class = expr`
    /// * class_alias_binding is `None`: `Class = expr`
    fn get_class_initializer(
        expr: Expression<'a>,
        class_alias_binding: Option<&BoundIdentifier<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        if let Some(class_alias_binding) = class_alias_binding {
            let left = class_alias_binding.create_write_target(ctx);
            Expression::new_assignment_expression(SPAN, AssignmentOperator::Assign, left, expr, ctx)
        } else {
            expr
        }
    }

    /// Check if a class or its constructor parameters have decorators.
    fn check_class_has_decorated(class: &Class<'a>) -> bool {
        if !class.decorators.is_empty() {
            return true;
        }

        class.body.body.iter().any(|element| {
            matches!(element,
                ClassElement::MethodDefinition(method) if method.kind.is_constructor() &&
                    Self::class_method_parameter_is_decorated(&method.value)
            )
        })
    }

    /// Check if a class method parameter is decorated.
    fn class_method_parameter_is_decorated(func: &Function<'a>) -> bool {
        func.params.items.iter().any(|param| !param.decorators.is_empty())
    }

    /// * is_static is `true`: `Class`
    /// * is_static is `false`: `Class.prototype`
    fn get_class_member_prefix(
        class_binding: &BoundIdentifier<'a>,
        is_static: bool,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let ident = class_binding.create_read_expression(ctx);
        if is_static { ident } else { create_prototype_member(ident, SPAN, ctx) }
    }

    /// Get the name of the property key.
    ///
    /// * StaticIdentifier: `a = 0;` -> `a`
    /// * PrivateIdentifier: `#a = 0;` -> `""`
    /// * Literal key:
    ///    * StaticStringLiteral: `"a" = 0;` -> `"a"`
    ///    * StaticNumericLiteral: `1 = 0;` -> `1`
    ///    * StaticBigIntLiteral: `1n = 0;` -> `1n`
    /// * Computed property key:
    ///  * Copiable key:
    ///    * NumericLiteral: `[1] = 0;` -> `1`
    ///    * StringLiteral: `["a"] = 0;` -> `"a"`
    ///    * TemplateLiteral: `[`a`] = 0;` -> `a`
    ///    * NullLiteral: `[null] = 0;` -> `null`
    ///  * Non-copiable key:
    ///    * `[a()] = 0;` mutates the key to `[_a = a()] = 0;` and returns `_a`
    fn get_name_of_property_key(
        key: &mut PropertyKey<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        match key {
            PropertyKey::StaticIdentifier(ident) => {
                Expression::new_string_literal(SPAN, ident.name, None, ctx)
            }
            // Legacy decorators do not support private key
            PropertyKey::PrivateIdentifier(_) => {
                Expression::new_string_literal(SPAN, "", None, ctx)
            }
            // Copiable literals
            PropertyKey::StaticNumericLiteral(literal) | PropertyKey::NumericLiteral(literal) => {
                Expression::NumericLiteral(literal.clone_in(ctx.allocator()))
            }
            PropertyKey::StaticStringLiteral(literal) | PropertyKey::StringLiteral(literal) => {
                Expression::StringLiteral(literal.clone_in(ctx.allocator()))
            }
            PropertyKey::StaticBigIntLiteral(literal) => {
                Expression::BigIntLiteral(literal.clone_in(ctx.allocator()))
            }
            PropertyKey::TemplateLiteral(literal) if literal.expressions.is_empty() => {
                let quasis = literal.quasis.clone_in(ctx.allocator());
                Expression::new_template_literal(SPAN, quasis, [], ctx)
            }
            PropertyKey::NullLiteral(_) => Expression::new_null_literal(SPAN, ctx),
            match_expression!(PropertyKey) => {
                // ```ts
                // Input:
                // class Test {
                //  static [a()] = 0;
                // }

                // Output:
                // ```js
                // let _a;
                // class Test {
                //   static [_a = a()] = 0;
                // ```

                let key = key.to_expression_mut();

                // Create a unique binding for the computed property key, and insert it outside of the class
                let binding = VarDeclarationsStore::create_uid_var_based_on_node(key, ctx);
                let operator = AssignmentOperator::Assign;
                let left = binding.create_write_target(ctx);
                key.replace_with(|right| {
                    Expression::new_assignment_expression(SPAN, operator, left, right, ctx)
                });
                binding.create_read_expression(ctx)
            }
        }
    }

    /// `_decorate([...decorators], Class, name, descriptor)`
    fn create_decorate_call(
        decorations: Expression<'a>,
        prefix: Expression<'a>,
        name: Expression<'a>,
        descriptor: Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let arguments = ArenaVec::from_array_in(
            [
                Argument::from(decorations),
                Argument::from(prefix),
                Argument::from(name),
                Argument::from(descriptor),
            ],
            ctx,
        );
        helper_call_expr(Helper::Decorate, arguments, ctx)
    }
}

/// Visitor to detect if a decorator refers to a private name of the decorated class, such as
/// `#x in obj` or `obj.#x`.
///
/// Such decorators must be evaluated inside the class, like TypeScript does. Private names of an
/// enclosing class are accessible after the class too, where `this` is unchanged.
struct PrivateNameDetector<'a> {
    /// Private names declared by the class, without those shadowed by a nested class.
    names: Vec<Ident<'a>>,
    found: bool,
}

impl<'a> PrivateNameDetector<'a> {
    fn new(body: &ClassBody<'a>) -> Self {
        Self { names: Self::declared_names(body).collect(), found: false }
    }

    fn declared_names<'b>(body: &'b ClassBody<'a>) -> impl Iterator<Item = Ident<'a>> + 'b {
        body.body.iter().filter_map(|element| {
            let key = match element {
                ClassElement::PropertyDefinition(prop) => &prop.key,
                ClassElement::MethodDefinition(method) => &method.key,
                ClassElement::AccessorProperty(accessor) => &accessor.key,
                ClassElement::StaticBlock(_) | ClassElement::TSIndexSignature(_) => return None,
            };
            key.private_name()
        })
    }

    fn visit_method_decorators(&mut self, method: &MethodDefinition<'a>) {
        self.visit_decorators(&method.decorators);
        for param in &method.value.params.items {
            self.visit_decorators(&param.decorators);
        }
    }
}

impl<'a> VisitJs<'a> for PrivateNameDetector<'a> {
    fn visit_decorators(&mut self, decorators: &ArenaVec<'a, Decorator<'a>>) {
        if self.names.is_empty() || self.found {
            return;
        }
        for decorator in decorators {
            self.visit_expression(&decorator.expression);
        }
    }

    fn visit_private_identifier(&mut self, ident: &PrivateIdentifier<'a>) {
        if self.names.contains(&ident.name) {
            self.found = true;
        }
    }

    fn visit_class(&mut self, class: &Class<'a>) {
        let shadowed = Self::declared_names(&class.body).any(|name| self.names.contains(&name));
        if !shadowed {
            walk_js::walk_class(self, class);
            return;
        }
        let names = self.names.clone();
        let declared = Self::declared_names(&class.body).collect::<Vec<_>>();
        self.names.retain(|name| !declared.contains(name));
        walk_js::walk_class(self, class);
        self.names = names;
    }
}

/// Visitor to move the scopes in decorator expressions to where the expressions end up, such as
/// after the class or in a static block. Only the outermost scopes are moved, the scopes within
/// them keep their parents.
///
/// Code in a class is strict, so moved scopes lose their strict mode when the new parent is
/// sloppy, unless they are strict on their own.
struct ChildScopeReparenter<'s> {
    scoping: &'s mut Scoping,
    parent_scope_id: ScopeId,
    make_sloppy: bool,
}

impl<'s> ChildScopeReparenter<'s> {
    fn new(parent_scope_id: ScopeId, ctx: &'s mut TraverseCtx<'_>) -> Self {
        let scoping = ctx.scoping_mut();
        let make_sloppy = !scoping.scope_flags(parent_scope_id).is_strict_mode();
        Self { scoping, parent_scope_id, make_sloppy }
    }

    fn reparent_scope(&mut self, scope_id: &Cell<Option<ScopeId>>) {
        self.scoping.change_scope_parent_id(scope_id.get().unwrap(), Some(self.parent_scope_id));
    }
}

impl<'a> VisitJs<'a> for ChildScopeReparenter<'_> {
    fn visit_function(&mut self, func: &Function<'a>, flags: ScopeFlags) {
        self.reparent_scope(&func.scope_id);
        if self.make_sloppy {
            SloppyScopes { scoping: self.scoping }.visit_function(func, flags);
        }
    }

    fn visit_arrow_function_expression(&mut self, func: &ArrowFunctionExpression<'a>) {
        self.reparent_scope(&func.scope_id);
        if self.make_sloppy {
            SloppyScopes { scoping: self.scoping }.visit_arrow_function_expression(func);
        }
    }

    fn visit_class(&mut self, class: &Class<'a>) {
        // `decorators` is outside `Class`'s scope and can contain scopes itself
        self.visit_decorators(&class.decorators);
        self.reparent_scope(&class.scope_id);
    }
}

/// Visitor to turn type-only references into value reads, for an expression that moves out of a
/// type-only position.
struct ValueReferences<'s> {
    scoping: &'s mut Scoping,
}

impl<'a> VisitJs<'a> for ValueReferences<'_> {
    fn visit_identifier_reference(&mut self, ident: &IdentifierReference<'a>) {
        let flags = self.scoping.get_reference_mut(ident.reference_id()).flags_mut();
        if flags.is_value_as_type() {
            *flags = ReferenceFlags::Read;
        }
    }
}

/// Visitor to remove strict mode from scopes that are no longer in strict code.
struct SloppyScopes<'s> {
    scoping: &'s mut Scoping,
}

impl<'a> VisitJs<'a> for SloppyScopes<'_> {
    fn enter_scope(&mut self, _flags: ScopeFlags, scope_id: &Cell<Option<ScopeId>>) {
        *self.scoping.scope_flags_mut(scope_id.get().unwrap()) -= ScopeFlags::StrictMode;
    }

    fn visit_function(&mut self, func: &Function<'a>, flags: ScopeFlags) {
        if !func.has_use_strict_directive() {
            walk_js::walk_function(self, func, flags);
        }
    }

    fn visit_arrow_function_expression(&mut self, func: &ArrowFunctionExpression<'a>) {
        if !func.has_use_strict_directive() {
            walk_js::walk_arrow_function_expression(self, func);
        }
    }

    // Classes are always strict.
    fn visit_class(&mut self, _class: &Class<'a>) {}
}

/// Visitor to change references to the class to a local alias
/// <https://github.com/microsoft/TypeScript/blob/8da951cbb629b648753454872df4e1754982aef1/src/compiler/transformers/legacyDecorators.ts#L770-L783>
struct ClassReferenceChanger<'a, 'ctx> {
    class_binding: BoundIdentifier<'a>,
    /// The class's inner name binding, and the scope of the static block holding the member
    /// decorations. Like TypeScript, member decorations refer to the class itself rather than
    /// to the alias, through its inner name. See [`ClassDecorations::member_decorations_block`].
    member_decorations_block: Option<(SymbolId, ScopeId)>,
    in_member_decorations_block: bool,
    // `Some` if there are references to the class inside the class body
    class_alias_binding: Option<BoundIdentifier<'a>>,
    ctx: &'ctx mut TraverseCtx<'a>,
}

impl<'a, 'ctx> ClassReferenceChanger<'a, 'ctx> {
    fn new(
        class_binding: BoundIdentifier<'a>,
        member_decorations_block: Option<(SymbolId, ScopeId)>,
        ctx: &'ctx mut TraverseCtx<'a>,
    ) -> Self {
        Self {
            class_binding,
            member_decorations_block,
            in_member_decorations_block: false,
            class_alias_binding: None,
            ctx,
        }
    }

    fn get_class_alias_if_needed(
        mut self,
        class: &mut ClassBody<'a>,
    ) -> Option<BoundIdentifier<'a>> {
        self.visit_class_body(class);
        self.class_alias_binding
    }
}

impl<'a> VisitMut<'a> for ClassReferenceChanger<'a, '_> {
    fn visit_static_block(&mut self, block: &mut StaticBlock<'a>) {
        let is_member_decorations_block =
            self.member_decorations_block.is_some_and(|(_, scope_id)| scope_id == block.scope_id());
        self.in_member_decorations_block = is_member_decorations_block;
        self.visit_statements(&mut block.body);
        self.in_member_decorations_block = false;
    }

    #[inline]
    fn visit_identifier_reference(&mut self, ident: &mut IdentifierReference<'a>) {
        if !self.is_class_reference(ident) {
            return;
        }
        if self.in_member_decorations_block
            && let Some((inner_symbol_id, _)) = self.member_decorations_block
        {
            let reference_id = ident.reference_id();
            let scoping = self.ctx.scoping_mut();
            scoping.get_reference_mut(reference_id).set_symbol_id(inner_symbol_id);
            scoping.delete_resolved_reference(self.class_binding.symbol_id, reference_id);
            scoping.add_resolved_reference(inner_symbol_id, reference_id);
        } else {
            let reference_id = ident.reference_id();
            self.ctx
                .scoping_mut()
                .delete_resolved_reference(self.class_binding.symbol_id, reference_id);
            *ident = self.get_alias_ident_reference();
        }
    }
}

impl<'a> ClassReferenceChanger<'a, '_> {
    // Check if the identifier reference is a reference to the class
    fn is_class_reference(&self, ident: &IdentifierReference<'a>) -> bool {
        self.ctx
            .scoping()
            .get_reference(ident.reference_id())
            .symbol_id()
            .is_some_and(|symbol_id| self.class_binding.symbol_id == symbol_id)
    }

    fn get_alias_ident_reference(&mut self) -> IdentifierReference<'a> {
        let binding = self.class_alias_binding.get_or_insert_with(|| {
            VarDeclarationsStore::create_uid_var(&self.class_binding.name, self.ctx)
        });

        binding.create_read_reference(self.ctx)
    }
}
