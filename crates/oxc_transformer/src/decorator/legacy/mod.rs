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

use std::mem;

use oxc_allocator::{
    Address, ArenaVec, CloneIn, GetAddress, GetAllocator, ReplaceWith, TakeIn, UnstableAddress,
};
use oxc_ast::ast::*;
use oxc_ast_visit::{VisitJs, VisitMut};
use oxc_data_structures::stack::NonEmptyStack;
use oxc_semantic::{ScopeFlags, SymbolFlags, SymbolId};
use oxc_span::SPAN;
use oxc_syntax::operator::AssignmentOperator;
use oxc_traverse::{Ancestor, BoundIdentifier, Traverse};
use rustc_hash::FxHashMap;

use crate::{
    Helper,
    common::{helper_loader::helper_call_expr, var_declarations::VarDeclarationsStore},
    context::TraverseCtx,
    decorator::DecoratorOptions,
    state::TransformState,
    utils::{
        ast_builder::{
            create_assignment, create_export_default_class_reference,
            create_export_named_class_reference, create_prototype_member,
        },
        decorated_class::{
            class_body_has_auto_accessors, legacy_decorators_reassign_class_binding,
        },
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
    /// Decoration statements accumulated for the current class.
    /// These will be applied when the class processing is complete.
    decoration_stmts: Vec<Statement<'a>>,
    /// Decoration statements of static members, kept apart from `decoration_stmts`
    /// because TypeScript applies every instance member decorator first, then
    /// every static member decorator. See [`ClassDecorations::append_static_decorations`].
    static_decoration_stmts: Vec<Statement<'a>>,
    /// Binding for the current class being processed.
    /// Generated on-demand when the first decorator needs it.
    class_binding: Option<BoundIdentifier<'a>>,
    /// Flag indicating whether the current class has a private `in` expression in any decorator.
    /// This affects where decorations are placed (in static block vs after class).
    class_has_private_in_expression_in_decorator: bool,
}

impl ClassDecorations<'_> {
    fn with_should_transform(mut self, should_transform: bool) -> Self {
        self.should_transform = should_transform;
        self
    }

    /// Move the static member decorations after the instance member ones,
    /// the order TypeScript applies them in.
    fn append_static_decorations(&mut self) {
        self.decoration_stmts.append(&mut self.static_decoration_stmts);
    }
}

pub struct LegacyDecorator<'a> {
    emit_decorator_metadata: bool,
    metadata: LegacyDecoratorMetadata<'a>,
    /// Decorated class data exists when a class or constructor is decorated.
    ///
    /// The data assigned in [`Self::transform_class`] and used in places where statements contain
    /// a decorated class declaration because decorated class needs to transform into `let c = class c {}`.
    /// The reason we why don't transform class in [`Self::exit_statement`] is the decorator transform
    /// must run first. Since the `class-properties` plugin transforms class in `exit_class`, so that
    /// we have to transforms decorators to `exit_class` otherwise after class is being transformed by
    /// `class-properties` plugin, the decorators' nodes might be lost.
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

        let should_transform = !(class.is_expression() || class.declare);
        // Class-properties must see the same binding that decorators will use after the class.
        if class.id.is_none() && legacy_decorators_reassign_class_binding(class) {
            let outer_scope_id = ctx.scoping().scope_parent_id(class.scope_id()).unwrap();
            let binding = ctx.generate_uid("default", outer_scope_id, SymbolFlags::Class);
            class.id = Some(binding.create_binding_identifier(ctx));
        }

        self.class_decorations_stack
            .push(ClassDecorations::default().with_should_transform(should_transform));

        if self.emit_decorator_metadata {
            self.metadata.enter_class(class, ctx);
        }
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

    #[inline]
    fn enter_method_definition(
        &mut self,
        node: &mut MethodDefinition<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if self.emit_decorator_metadata {
            self.metadata.enter_method_definition(node, ctx);
        }
    }

    #[inline]
    fn enter_accessor_property(
        &mut self,
        node: &mut AccessorProperty<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if self.emit_decorator_metadata {
            self.metadata.enter_accessor_property(node, ctx);
        }
    }

    #[inline]
    fn enter_property_definition(
        &mut self,
        node: &mut PropertyDefinition<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if self.emit_decorator_metadata {
            self.metadata.enter_property_definition(node, ctx);
        }
    }

    #[inline]
    fn exit_method_definition(
        &mut self,
        method: &mut MethodDefinition<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        // `constructor` will handle in `transform_decorators_of_class_and_constructor`.
        if method.kind.is_constructor() {
            return;
        }

        if let Some(decorations) = self.get_all_decorators_of_class_method(method, ctx) {
            // We emit `null` here to indicate to `_decorate` that it can invoke `Object.getOwnPropertyDescriptor` directly.
            let descriptor = Expression::new_null_literal(SPAN, ctx);
            self.handle_decorated_class_element(
                method.r#static,
                &mut method.key,
                true,
                descriptor,
                decorations,
                ctx,
            );
        }
    }

    #[inline]
    fn exit_property_definition(
        &mut self,
        prop: &mut PropertyDefinition<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if prop.decorators.is_empty() {
            return;
        }

        let decorations =
            Self::convert_decorators_to_array_expression(prop.decorators.drain(..), ctx);

        // We emit `void 0` here to indicate to `_decorate` that it can invoke `Object.defineProperty` directly.
        let descriptor = Expression::new_void_0(SPAN, ctx);
        // The class doesn't evaluate the key of a `declare` field, which TypeScript removes. It has
        // already replaced a key with side effects by a temp var, which it assigns in order with the
        // other class keys, so an identifier or literal key is passed to `_decorate` as is.
        let key_is_evaluated = !(prop.declare
            && prop
                .key
                .as_expression()
                .is_some_and(|key| matches!(key, Expression::Identifier(_)) || key.is_literal()));
        self.handle_decorated_class_element(
            prop.r#static,
            &mut prop.key,
            key_is_evaluated,
            descriptor,
            decorations,
            ctx,
        );
    }

    #[inline]
    fn exit_accessor_property(
        &mut self,
        accessor: &mut AccessorProperty<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if accessor.decorators.is_empty() {
            return;
        }

        let decorations =
            Self::convert_decorators_to_array_expression(accessor.decorators.drain(..), ctx);
        // We emit `null` here to indicate to `_decorate` that it can invoke `Object.getOwnPropertyDescriptor` directly.
        let descriptor = Expression::new_null_literal(SPAN, ctx);
        self.handle_decorated_class_element(
            accessor.r#static,
            &mut accessor.key,
            true,
            descriptor,
            decorations,
            ctx,
        );
    }

    fn enter_decorator(&mut self, node: &mut Decorator<'a>, _ctx: &mut TraverseCtx<'a>) {
        let current_class = self.class_decorations_stack.last_mut();
        if current_class.should_transform
            && !current_class.class_has_private_in_expression_in_decorator
        {
            current_class.class_has_private_in_expression_in_decorator =
                PrivateInExpressionDetector::has_private_in_expression(&node.expression);
        }
    }
}

impl<'a> LegacyDecorator<'a> {
    fn lower_accessor_properties(class: &mut Class<'a>, ctx: &mut TraverseCtx<'a>) {
        if class.declare || !class_body_has_auto_accessors(&class.body) {
            return;
        }
        let has_static_accessor = class.body.body.iter().any(|element| {
            matches!(element, ClassElement::AccessorProperty(prop) if prop.r#static && !prop.r#type.is_abstract())
        });
        if has_static_accessor && class.id.is_none() {
            let binding = ctx.generate_uid("class", class.scope_id(), SymbolFlags::Class);
            class.id = Some(binding.create_binding_identifier(ctx));
        }
        let binding = class.id.as_ref().map(BoundIdentifier::from_binding_ident);
        super::accessor::lower_auto_accessors(
            &mut class.body,
            class.scope_id.get().unwrap(),
            binding.as_ref(),
            false,
            ctx,
        );
    }

    /// Helper method to handle a decorated class element (method, property, or accessor).
    /// Accumulates decoration statements in the current decoration stack.
    fn handle_decorated_class_element(
        &mut self,
        is_static: bool,
        key: &mut PropertyKey<'a>,
        key_is_evaluated: bool,
        descriptor: Expression<'a>,
        decorations: Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let current_class = self.class_decorations_stack.last_mut();

        if !current_class.should_transform {
            return;
        }

        // Get current class binding from stack
        let class_binding = current_class.class_binding.get_or_insert_with(|| {
            let Ancestor::ClassBody(class) = ctx.ancestor(1) else {
                unreachable!("The grandparent of a class element is always a class.");
            };
            if let Some(ident) = class.id() {
                BoundIdentifier::from_binding_ident(ident)
            } else {
                let class_scope_id = class.scope_id().get().unwrap();
                let outer_scope_id = ctx.scoping().scope_parent_id(class_scope_id).unwrap();
                ctx.generate_uid("default", outer_scope_id, SymbolFlags::Class)
            }
        });

        let prefix = Self::get_class_member_prefix(class_binding, is_static, ctx);
        let name = self.get_name_of_property_key(key, key_is_evaluated, ctx);
        let decorator_stmt = self.create_decorator(decorations, prefix, name, descriptor, ctx);

        // Push to current decoration stack
        let current_class = self.class_decorations_stack.last_mut();
        if is_static {
            current_class.static_decoration_stmts.push(decorator_stmt);
        } else {
            current_class.decoration_stmts.push(decorator_stmt);
        }
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
        let mut current_class_decorations = self.class_decorations_stack.pop();
        current_class_decorations.append_static_decorations();

        // Legacy decorator does not allow in class expression.
        if current_class_decorations.should_transform {
            if legacy_decorators_reassign_class_binding(class) {
                self.transform_class_declaration_with_class_decorators(
                    class,
                    current_class_decorations,
                    ctx,
                );
                return;
            } else if !current_class_decorations.decoration_stmts.is_empty() {
                self.transform_class_declaration_without_class_decorators(
                    class,
                    current_class_decorations,
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
        // `enter_class` names anonymous classes that have class or constructor-parameter
        // decorators, so the class always has an id here.
        let class_binding = {
            let ident =
                class.id.as_ref().expect("decorated class should have been named in `enter_class`");
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
        };
        let class_alias_binding =
            ClassReferenceChanger::get_class_alias_if_needed(&class_binding, &mut class.body, ctx);

        let ClassDecorations {
            class_binding: _,
            mut decoration_stmts,
            static_decoration_stmts: _,
            class_has_private_in_expression_in_decorator,
            should_transform: _,
        } = current_class_decorations;

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

        // Member decorations that use a private-in expression must stay inside the class body
        // to access the private name, so they move into a static block at the end of the class.
        // The class decorator still applies after the class, like TypeScript does.
        if class_has_private_in_expression_in_decorator {
            let mut decorations = mem::take(&mut decoration_stmts);
            // Inside the class body, the class name refers to the class's own binding
            // rather than the outer one, which is still uninitialized when the static block runs.
            // If class fields are lowered, the static block moves back out of the class, and
            // class-properties replaces references to the outer binding with a temp var instead.
            if !ctx.state.class_lowering_of(class.scope_id()).lowers_fields() {
                let inner_class_binding = BoundIdentifier::from_binding_ident(
                    class.id.as_ref().expect("decorated class always has an id here"),
                );
                ClassReferenceChanger::rebind(
                    &mut decorations,
                    class_binding.symbol_id,
                    inner_class_binding,
                    ctx,
                );
            }
            Self::insert_decorations_into_class_static_block(class, decorations, ctx);
        }

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
        ctx: &mut TraverseCtx<'a>,
    ) {
        let ClassDecorations {
            class_binding,
            mut decoration_stmts,
            static_decoration_stmts: _,
            class_has_private_in_expression_in_decorator,
            should_transform: _,
        } = current_class_decorations;

        let Some(class_binding) = class_binding else {
            unreachable!(
                "Always has a class binding because there are decorators in class elements,
                so that it has been added in `handle_decorated_class_element`"
            );
        };

        // No class id, add one by using the class binding
        if class.id.is_none() {
            class.id = Some(class_binding.create_binding_identifier(ctx));
        }

        if class_has_private_in_expression_in_decorator {
            Self::insert_decorations_into_class_static_block(class, decoration_stmts, ctx);
        } else {
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
            Self::convert_decorators_to_array_expression(class.decorators.drain(..), ctx)
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
        class: &mut Class<'a>,
        decorations: Vec<Statement<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let scope_id = ctx.create_child_scope(class.scope_id(), ScopeFlags::ClassStaticBlock);
        ctx.move_scopes_of_statements(&decorations, scope_id);
        let decorations = ArenaVec::from_iter_in(decorations, ctx);
        let element =
            ClassElement::new_static_block_with_scope_id(SPAN, decorations, scope_id, ctx);
        class.body.body.push(element);
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
            // The statements are inserted next to the class, in the current scope.
            ctx.move_scopes_of_statements(&stmts, ctx.current_scope_id());
            ctx.state.statement_injector.insert_many_after(&address, stmts);
        }
    }

    /// Converts a vec of [`Decorator`] to [`Expression::ArrayExpression`].
    fn convert_decorators_to_array_expression(
        decorators_iter: impl Iterator<Item = Decorator<'a>>,
        ctx: &TraverseCtx<'a>,
    ) -> Expression<'a> {
        let decorations = ArenaVec::from_iter_in(
            decorators_iter.map(|decorator| ArrayExpressionElement::from(decorator.expression)),
            ctx,
        );
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
            if self.emit_decorator_metadata {
                if method.kind.is_constructor() {
                    debug_assert!(
                        self.metadata.pop_constructor_metadata().is_none(),
                        "No method decorators, so `pop_constructor_metadata` should be `None`"
                    );
                } else {
                    debug_assert!(
                        self.metadata.pop_method_metadata().is_none(),
                        "No method decorators, so `pop_method_metadata` should be `None`"
                    );
                }
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
            } else if let Some(metadata) = self.metadata.pop_method_metadata() {
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
    ///
    /// When the class doesn't evaluate `key`, it's moved into the decoration instead:
    /// `@dec declare [_x];` -> `_decorate([dec], C.prototype, _x, void 0)`.
    #[expect(clippy::unused_self)]
    fn get_name_of_property_key(
        &self,
        key: &mut PropertyKey<'a>,
        key_is_evaluated: bool,
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
            key @ match_expression!(PropertyKey) if !key_is_evaluated => {
                key.to_expression_mut().take_in(ctx)
            }
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

    /// `_decorator([...decorators], Class, name, descriptor)`
    #[expect(clippy::unused_self)]
    fn create_decorator(
        &self,
        decorations: Expression<'a>,
        prefix: Expression<'a>,
        name: Expression<'a>,
        descriptor: Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Statement<'a> {
        let arguments = ArenaVec::from_array_in(
            [
                Argument::from(decorations),
                Argument::from(prefix),
                Argument::from(name),
                Argument::from(descriptor),
            ],
            ctx,
        );
        let helper = helper_call_expr(Helper::Decorate, arguments, ctx);
        Statement::new_expression_statement(SPAN, helper, ctx)
    }
}

/// Visitor to detect if a private-in expression is present in a decorator
#[derive(Default)]
struct PrivateInExpressionDetector {
    has_private_in_expression: bool,
}

impl VisitJs<'_> for PrivateInExpressionDetector {
    fn visit_private_in_expression(&mut self, _it: &PrivateInExpression<'_>) {
        self.has_private_in_expression = true;
    }

    fn visit_decorators(&mut self, decorators: &ArenaVec<'_, Decorator<'_>>) {
        for decorator in decorators {
            self.visit_expression(&decorator.expression);
            // Early exit if a private-in expression is found
            if self.has_private_in_expression {
                break;
            }
        }
    }
}

impl PrivateInExpressionDetector {
    fn has_private_in_expression(expression: &Expression<'_>) -> bool {
        let mut detector = Self::default();
        detector.visit_expression(expression);
        detector.has_private_in_expression
    }
}

/// Visitor to change references to the class to another binding: the given one, or a local
/// alias created for the first reference found.
/// <https://github.com/microsoft/TypeScript/blob/8da951cbb629b648753454872df4e1754982aef1/src/compiler/transformers/legacyDecorators.ts#L770-L783>
struct ClassReferenceChanger<'a, 'ctx> {
    class_symbol_id: SymbolId,
    class_name: Ident<'a>,
    /// The binding references are changed to. If `None`, an alias is created on first use.
    target_binding: Option<BoundIdentifier<'a>>,
    ctx: &'ctx mut TraverseCtx<'a>,
}

impl<'a, 'ctx> ClassReferenceChanger<'a, 'ctx> {
    /// Change references to the class in its body to a local alias, if there are any.
    fn get_class_alias_if_needed(
        class_binding: &BoundIdentifier<'a>,
        body: &mut ClassBody<'a>,
        ctx: &'ctx mut TraverseCtx<'a>,
    ) -> Option<BoundIdentifier<'a>> {
        let mut changer = Self {
            class_symbol_id: class_binding.symbol_id,
            class_name: class_binding.name,
            target_binding: None,
            ctx,
        };
        changer.visit_class_body(body);
        changer.target_binding
    }

    /// Change references to the class in decorations moved into the class body to `target`.
    fn rebind(
        stmts: &mut [Statement<'a>],
        class_symbol_id: SymbolId,
        target: BoundIdentifier<'a>,
        ctx: &'ctx mut TraverseCtx<'a>,
    ) {
        let mut changer =
            Self { class_symbol_id, class_name: target.name, target_binding: Some(target), ctx };
        for stmt in stmts {
            changer.visit_statement(stmt);
        }
    }
}

impl<'a> VisitMut<'a> for ClassReferenceChanger<'a, '_> {
    #[inline]
    fn visit_identifier_reference(&mut self, ident: &mut IdentifierReference<'a>) {
        let symbol_id = self.ctx.scoping().get_reference(ident.reference_id()).symbol_id();
        if symbol_id != Some(self.class_symbol_id) {
            return;
        }

        // Rebind the reference, keeping its `ReferenceId` and flags
        let target = self.target_binding.get_or_insert_with(|| {
            VarDeclarationsStore::create_uid_var(&self.class_name, self.ctx)
        });
        self.ctx.rebind_identifier_reference(ident, target);
    }
}
