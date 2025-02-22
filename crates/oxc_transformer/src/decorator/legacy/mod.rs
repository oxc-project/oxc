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

use metadata::LegacyDecoratorMetadata;
use oxc_allocator::{Address, GetAddress, Vec as ArenaVec};
use oxc_ast::{NONE, Visit, VisitMut, ast::*};
use oxc_semantic::{ScopeFlags, SymbolFlags};
use oxc_span::SPAN;
use oxc_syntax::operator::AssignmentOperator;
use oxc_traverse::{BoundIdentifier, Traverse, TraverseCtx};

use crate::{Helper, TransformCtx, utils::ast_builder::create_prototype_member};

struct ClassDecoratorInfo {
    /// `@dec class C {}` or `class C { constructor(@dec p) {} }`
    class_or_constructor_parameter_is_decorated: bool,
    /// `class C { @dec m() {} }`
    class_element_is_decorated: bool,
    /// `class C { @(#a in C ? dec() : dec2()) prop = 0; }`
    has_private_in_expression_in_decorator: bool,
}

pub struct LegacyDecorator<'a, 'ctx> {
    emit_decorator_metadata: bool,
    metadata: LegacyDecoratorMetadata<'a, 'ctx>,
    ctx: &'ctx TransformCtx<'a>,
}

impl<'a, 'ctx> LegacyDecorator<'a, 'ctx> {
    pub fn new(emit_decorator_metadata: bool, ctx: &'ctx TransformCtx<'a>) -> Self {
        Self { emit_decorator_metadata, metadata: LegacyDecoratorMetadata::new(ctx), ctx }
    }
}

impl<'a> Traverse<'a> for LegacyDecorator<'a, '_> {
    // `#[inline]` because this is a hot path
    #[inline]
    fn enter_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        match stmt {
            Statement::ClassDeclaration(_) => self.transform_class(stmt, ctx),
            Statement::ExportNamedDeclaration(_) => {
                self.transform_export_named_class(stmt, ctx);
            }
            Statement::ExportDefaultDeclaration(_) => {
                self.transform_export_default_class(stmt, ctx);
            }
            _ => {}
        };
    }

    #[inline]
    fn enter_class(&mut self, class: &mut Class<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.emit_decorator_metadata {
            self.metadata.enter_class(class, ctx);
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
}

impl<'a> LegacyDecorator<'a, '_> {
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
    fn transform_class(&self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        let Statement::ClassDeclaration(class) = stmt else { unreachable!() };

        let stmt_address = class.address();
        if let Some((_, new_stmt)) = self.transform_class_impl(class, stmt_address, ctx) {
            *stmt = new_stmt;
        }
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
    fn transform_export_default_class(&self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        let Statement::ExportDefaultDeclaration(export) = stmt else { unreachable!() };
        let stmt_address = export.address();
        let ExportDefaultDeclarationKind::ClassDeclaration(class) = &mut export.declaration else {
            return;
        };

        let Some((class_binding, new_stmt)) = self.transform_class_impl(class, stmt_address, ctx)
        else {
            return;
        };
        *stmt = new_stmt;

        // `export default Class`
        let export_default_class_reference =
            Self::create_export_default_class_reference(&class_binding, ctx);
        self.ctx.statement_injector.insert_after(stmt, export_default_class_reference);
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
    // `#[inline]` so that compiler sees that `stmt` is a `Statement::ExportNamedDeclaration`.
    #[inline]
    fn transform_export_named_class(&self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        let Statement::ExportNamedDeclaration(export) = stmt else { unreachable!() };
        let stmt_address = export.address();
        let Some(Declaration::ClassDeclaration(class)) = &mut export.declaration else { return };

        let Some((class_binding, new_stmt)) = self.transform_class_impl(class, stmt_address, ctx)
        else {
            return;
        };
        *stmt = new_stmt;

        // `export { Class }`
        let export_class_reference = Self::create_export_named_class_reference(&class_binding, ctx);
        self.ctx.statement_injector.insert_after(stmt, export_class_reference);
    }

    fn transform_class_impl(
        &self,
        class: &mut Class<'a>,
        stmt_address: Address,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<(BoundIdentifier<'a>, Statement<'a>)> {
        let ClassDecoratorInfo {
            class_or_constructor_parameter_is_decorated,
            class_element_is_decorated,
            has_private_in_expression_in_decorator,
        } = Self::check_class_has_decorated(class);

        if class_or_constructor_parameter_is_decorated {
            return Some(self.transform_class_declaration_with_class_decorators(
                class,
                has_private_in_expression_in_decorator,
                ctx,
            ));
        } else if class_element_is_decorated {
            self.transform_class_declaration_without_class_decorators(
                class,
                stmt_address,
                has_private_in_expression_in_decorator,
                ctx,
            );
        }

        // No decorators found
        None
    }

    /// Transforms a decorated class declaration and appends the resulting statements. If
    /// the class requires an alias to avoid issues with double-binding, the alias is returned.
    fn transform_class_declaration_with_class_decorators(
        &self,
        class: &mut Class<'a>,
        has_private_in_expression_in_decorator: bool,
        ctx: &mut TraverseCtx<'a>,
    ) -> (BoundIdentifier<'a>, Statement<'a>) {
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

        let span = class.span;
        // TODO(improve-on-typescript): we can take the class id without keeping it as-is.
        // Now: `class C {}` -> `let C = class C {}`
        // After: `class C {}` -> `let C = class {}`
        let class_binding = class.id.as_ref().map(|ident| {
            let new_class_binding =
                ctx.generate_binding(ident.name, class.scope_id(), SymbolFlags::Class);
            let old_class_symbol_id = ident.symbol_id.replace(Some(new_class_binding.symbol_id));
            let old_class_symbol_id = old_class_symbol_id.expect("class always has a symbol id");

            *ctx.symbols_mut().get_flags_mut(old_class_symbol_id) =
                SymbolFlags::BlockScopedVariable;
            BoundIdentifier::new(ident.name, old_class_symbol_id)
        });
        let class_alias_binding = class_binding.as_ref().and_then(|id| {
            ClassReferenceChanger::new(id.clone(), ctx, self.ctx)
                .get_class_alias_if_needed(&mut class.body)
        });
        let class_binding = class_binding
            .unwrap_or_else(|| ctx.generate_uid_in_current_scope("default", SymbolFlags::Class));

        let constructor_decoration = self.transform_decorators_of_class_and_constructor(
            class,
            &class_binding,
            class_alias_binding.as_ref(),
            ctx,
        );
        let mut decoration_stmts =
            self.transform_decorators_of_class_elements(class, &class_binding, ctx);

        if has_private_in_expression_in_decorator {
            let stmts = mem::replace(&mut decoration_stmts, ctx.ast.vec());
            Self::insert_decorations_into_class_static_block(class, stmts, ctx);
        }

        decoration_stmts.push(constructor_decoration);

        // `class C {}` -> `let C = class {}`
        class.r#type = ClassType::ClassExpression;
        let initializer = Self::get_class_initializer(
            Expression::ClassExpression(ctx.ast.alloc(ctx.ast.move_class(class))),
            class_alias_binding.as_ref(),
            ctx,
        );
        let declarator = ctx.ast.variable_declarator(
            SPAN,
            VariableDeclarationKind::Let,
            class_binding.create_binding_pattern(ctx),
            Some(initializer),
            false,
        );
        let var_declaration = ctx.ast.declaration_variable(
            span,
            VariableDeclarationKind::Let,
            ctx.ast.vec1(declarator),
            false,
        );
        let statement = Statement::from(var_declaration);

        self.ctx.statement_injector.insert_many_after(&statement, decoration_stmts);

        (class_binding, statement)
    }

    /// Transforms a non-decorated class declaration.
    fn transform_class_declaration_without_class_decorators(
        &self,
        class: &mut Class<'a>,
        stmt_address: Address,
        has_private_in_expression_in_decorator: bool,
        ctx: &mut TraverseCtx<'a>,
    ) {
        // `export default class {}`
        let class_binding = if let Some(ident) = &class.id {
            BoundIdentifier::from_binding_ident(ident)
        } else {
            let class_binding = ctx.generate_uid_in_current_scope("default", SymbolFlags::Class);
            class.id.replace(class_binding.create_binding_identifier(ctx));
            class_binding
        };

        let decoration_stmts =
            self.transform_decorators_of_class_elements(class, &class_binding, ctx);

        if has_private_in_expression_in_decorator {
            Self::insert_decorations_into_class_static_block(class, decoration_stmts, ctx);
        } else {
            self.ctx.statement_injector.insert_many_after(&stmt_address, decoration_stmts);
        }
    }

    /// Transform decorators of [`ClassElement::MethodDefinition`],
    /// [`ClassElement::PropertyDefinition`] and [`ClassElement::AccessorProperty`].
    fn transform_decorators_of_class_elements(
        &self,
        class: &mut Class<'a>,
        class_binding: &BoundIdentifier<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> ArenaVec<'a, Statement<'a>> {
        let mut decoration_stmts = ctx.ast.vec_with_capacity(class.body.body.len());

        for element in &mut class.body.body {
            let (is_static, key, descriptor, decorations) = match element {
                ClassElement::MethodDefinition(method) => {
                    let Some(decorations) = self.get_all_decorators_of_class_method(method, ctx)
                    else {
                        continue;
                    };

                    // We emit `null` here to indicate to `_decorate` that it can invoke `Object.getOwnPropertyDescriptor` directly.
                    // We have this extra argument here so that we can inject an explicit property descriptor at a later date.
                    let descriptor = ctx.ast.expression_null_literal(SPAN);

                    (method.r#static, &mut method.key, descriptor, decorations)
                }
                ClassElement::PropertyDefinition(prop) if !prop.decorators.is_empty() => {
                    let decorations = Self::convert_decorators_to_array_expression(
                        prop.decorators.drain(..),
                        ctx,
                    );

                    // We emit `void 0` here to indicate to `_decorate` that it can invoke `Object.defineProperty` directly, but that it
                    // should not invoke `Object.getOwnPropertyDescriptor`.
                    let descriptor = ctx.ast.void_0(SPAN);

                    (prop.r#static, &mut prop.key, descriptor, decorations)
                }
                ClassElement::AccessorProperty(accessor) => {
                    let decorations = Self::convert_decorators_to_array_expression(
                        accessor.decorators.drain(..),
                        ctx,
                    );

                    // We emit `null` here to indicate to `_decorate` that it can invoke `Object.getOwnPropertyDescriptor` directly.
                    // We have this extra argument here so that we can inject an explicit property descriptor at a later date.
                    let descriptor = ctx.ast.expression_null_literal(SPAN);

                    (accessor.r#static, &mut accessor.key, descriptor, decorations)
                }
                _ => {
                    continue;
                }
            };

            // `Class` or `Class.prototype`
            let prefix = Self::get_class_member_prefix(class_binding, is_static, ctx);
            let name = self.get_name_of_property_key(key, ctx);
            // `_decorator([...decorators], Class, name, descriptor)`
            let decorator_stmt = self.create_decorator(decorations, prefix, name, descriptor, ctx);
            decoration_stmts.push(decorator_stmt);
        }

        decoration_stmts
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
    /// ```
    /// _decorate([
    ///   _decorateParam(0, dec)
    ///   ], Class.prototype, "method", null);
    ///
    /// Class = _decorate([
    ///   dec
    /// ], Class);
    /// ```
    fn transform_decorators_of_class_and_constructor(
        &self,
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
            Self::convert_decorators_to_array_expression(class.decorators.drain(..), ctx)
        };

        // `Class = _decorate(decorations, Class)`
        let arguments = ctx.ast.vec_from_array([
            Argument::from(decorations),
            Argument::from(class_binding.create_read_expression(ctx)),
        ]);
        let helper = self.ctx.helper_call_expr(Helper::Decorate, SPAN, arguments, ctx);
        let operator = AssignmentOperator::Assign;
        let left = class_binding.create_write_target(ctx);
        let right = Self::get_class_initializer(helper, class_alias_binding, ctx);
        let assignment = ctx.ast.expression_assignment(SPAN, operator, left, right);
        ctx.ast.statement_expression(SPAN, assignment)
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
        decorations: ArenaVec<'a, Statement<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let scope_id = ctx.create_child_scope(class.scope_id(), ScopeFlags::ClassStaticBlock);
        let static_block = ctx.ast.alloc_static_block_with_scope_id(SPAN, decorations, scope_id);
        let element = ClassElement::StaticBlock(static_block);
        class.body.body.push(element);
    }

    /// Transforms the decorators of the parameters of a class method.
    #[expect(clippy::cast_precision_loss)]
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
                let index = ctx.ast.expression_numeric_literal(
                    SPAN,
                    index as f64,
                    None,
                    NumberBase::Decimal,
                );
                let arguments = ctx
                    .ast
                    .vec_from_array([Argument::from(index), Argument::from(decorator.expression)]);
                // _decorateParam(index, decorator)
                ArrayExpressionElement::from(self.ctx.helper_call_expr(
                    Helper::DecorateParam,
                    decorator.span,
                    arguments,
                    ctx,
                ))
            }));
        }
    }

    /// Converts a vec of [`Decorator`] to [`Expression::ArrayExpression`].
    fn convert_decorators_to_array_expression(
        decorators_iter: impl Iterator<Item = Decorator<'a>>,
        ctx: &TraverseCtx<'a>,
    ) -> Expression<'a> {
        let decorations = ctx.ast.vec_from_iter(
            decorators_iter.map(|decorator| ArrayExpressionElement::from(decorator.expression)),
        );
        ctx.ast.expression_array(SPAN, decorations, None)
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
        &self,
        method: &mut MethodDefinition<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<Expression<'a>> {
        let params = &mut method.value.params;
        let param_decoration_count =
            params.items.iter().fold(0, |acc, param| acc + param.decorators.len());
        let method_decoration_count = method.decorators.len() + param_decoration_count;

        if method_decoration_count == 0 {
            return None;
        }

        let mut decorations = ctx.ast.vec_with_capacity(method_decoration_count);
        decorations.extend(
            method
                .decorators
                .drain(..)
                .map(|decorator| ArrayExpressionElement::from(decorator.expression)),
        );

        // The decorators of params are always inserted at the end if any.
        if param_decoration_count > 0 {
            self.transform_decorators_of_parameters(&mut decorations, params, ctx);
        }

        Some(ctx.ast.expression_array(SPAN, decorations, None))
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
            ctx.ast.expression_assignment(SPAN, AssignmentOperator::Assign, left, expr)
        } else {
            expr
        }
    }

    /// Check if a class or its elements have decorators.
    fn check_class_has_decorated(class: &Class<'a>) -> ClassDecoratorInfo {
        let mut class_or_constructor_parameter_is_decorated = !class.decorators.is_empty();
        let mut class_element_is_decorated = false;
        let mut has_private_in_expression_in_decorator = false;

        for element in &class.body.body {
            match element {
                ClassElement::MethodDefinition(method) if method.kind.is_constructor() => {
                    class_or_constructor_parameter_is_decorated |=
                        Self::class_method_parameter_is_decorated(&method.value);

                    if class_or_constructor_parameter_is_decorated
                        && !has_private_in_expression_in_decorator
                    {
                        has_private_in_expression_in_decorator =
                            PrivateInExpressionDetector::has_private_in_expression_in_method_decorator(method);
                    }
                }
                ClassElement::MethodDefinition(method) => {
                    class_element_is_decorated |= !method.decorators.is_empty()
                        || Self::class_method_parameter_is_decorated(&method.value);

                    if class_element_is_decorated && !has_private_in_expression_in_decorator {
                        has_private_in_expression_in_decorator =
                            PrivateInExpressionDetector::has_private_in_expression_in_method_decorator(method);
                    }
                }
                ClassElement::PropertyDefinition(prop) => {
                    class_element_is_decorated |= !prop.decorators.is_empty();

                    if class_element_is_decorated && !has_private_in_expression_in_decorator {
                        has_private_in_expression_in_decorator =
                            PrivateInExpressionDetector::has_private_in_expression(
                                &prop.decorators,
                            );
                    }
                }
                ClassElement::AccessorProperty(accessor) => {
                    class_element_is_decorated |= !accessor.decorators.is_empty();

                    if class_element_is_decorated && !has_private_in_expression_in_decorator {
                        has_private_in_expression_in_decorator =
                            PrivateInExpressionDetector::has_private_in_expression(
                                &accessor.decorators,
                            );
                    }
                }
                _ => {}
            }
        }

        ClassDecoratorInfo {
            class_or_constructor_parameter_is_decorated,
            class_element_is_decorated,
            has_private_in_expression_in_decorator,
        }
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
        if is_static { ident } else { create_prototype_member(ident, ctx) }
    }

    /// Get the name of the property key.
    ///
    /// * StaticIdentifier: `a = 0;` -> `a`
    /// * PrivateIdentifier: `#a = 0;` -> `""`
    /// * Computed property key:
    ///  * Copiable key:
    ///    * NumericLiteral: `[1] = 0;` -> `1`
    ///    * StringLiteral: `["a"] = 0;` -> `"a"`
    ///    * TemplateLiteral: `[`a`] = 0;` -> `a`
    ///    * NullLiteral: `[null] = 0;` -> `null`
    ///  * Non-copiable key:
    ///    * `[a()] = 0;` mutates the key to `[_a = a()] = 0;` and returns `_a`
    fn get_name_of_property_key(
        &self,
        key: &mut PropertyKey<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        match key {
            PropertyKey::StaticIdentifier(ident) => {
                ctx.ast.expression_string_literal(SPAN, ident.name, None)
            }
            // Legacy decorators do not support private key
            PropertyKey::PrivateIdentifier(_) => ctx.ast.expression_string_literal(SPAN, "", None),
            // Copiable literals
            PropertyKey::NumericLiteral(literal) => {
                Expression::NumericLiteral(ctx.ast.alloc(literal.clone()))
            }
            PropertyKey::StringLiteral(literal) => {
                Expression::StringLiteral(ctx.ast.alloc(literal.clone()))
            }
            PropertyKey::TemplateLiteral(literal) if literal.expressions.is_empty() => {
                let quasis = ctx.ast.vec_from_iter(literal.quasis.iter().cloned());
                ctx.ast.expression_template_literal(SPAN, quasis, ctx.ast.vec())
            }
            PropertyKey::NullLiteral(_) => ctx.ast.expression_null_literal(SPAN),
            _ => {
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

                // Create a unique binding for the computed property key, and insert it outside of the class
                let binding = self.ctx.var_declarations.create_uid_var_based_on_node(key, ctx);
                let operator = AssignmentOperator::Assign;
                let left = binding.create_read_write_target(ctx);
                let right = ctx.ast.move_expression(key.to_expression_mut());
                let key_expr = ctx.ast.expression_assignment(SPAN, operator, left, right);
                *key = PropertyKey::from(key_expr);
                binding.create_read_expression(ctx)
            }
        }
    }

    /// `_decorator([...decorators], Class, name, descriptor)`
    fn create_decorator(
        &self,
        decorations: Expression<'a>,
        prefix: Expression<'a>,
        name: Expression<'a>,
        descriptor: Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Statement<'a> {
        let arguments = ctx.ast.vec_from_array([
            Argument::from(decorations),
            Argument::from(prefix),
            Argument::from(name),
            Argument::from(descriptor),
        ]);
        let helper = self.ctx.helper_call_expr(Helper::Decorate, SPAN, arguments, ctx);
        ctx.ast.statement_expression(SPAN, helper)
    }

    /// `export default Class`
    fn create_export_default_class_reference(
        class_binding: &BoundIdentifier<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Statement<'a> {
        let export_default_class_reference = ctx.ast.module_declaration_export_default_declaration(
            SPAN,
            ExportDefaultDeclarationKind::Identifier(
                ctx.ast.alloc(class_binding.create_read_reference(ctx)),
            ),
            ctx.ast.module_export_name_identifier_name(SPAN, "default"),
        );
        Statement::from(export_default_class_reference)
    }

    /// `export { Class }`
    fn create_export_named_class_reference(
        class_binding: &BoundIdentifier<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Statement<'a> {
        let kind = ImportOrExportKind::Value;
        let local = ModuleExportName::IdentifierReference(class_binding.create_read_reference(ctx));
        let exported = ctx.ast.module_export_name_identifier_name(SPAN, class_binding.name);
        let specifiers = ctx.ast.vec1(ctx.ast.export_specifier(SPAN, local, exported, kind));
        let export_class_reference = ctx
            .ast
            .module_declaration_export_named_declaration(SPAN, None, specifiers, None, kind, NONE);
        Statement::from(export_class_reference)
    }
}

/// Visitor to detect if a private-in expression is present in a decorator
#[derive(Default)]
struct PrivateInExpressionDetector {
    has_private_in_expression: bool,
}

impl Visit<'_> for PrivateInExpressionDetector {
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
    fn has_private_in_expression(decorators: &ArenaVec<'_, Decorator<'_>>) -> bool {
        let mut detector = Self::default();
        detector.visit_decorators(decorators);
        detector.has_private_in_expression
    }

    fn has_private_in_expression_in_method_decorator(method: &MethodDefinition<'_>) -> bool {
        let mut detector = Self::default();
        detector.visit_decorators(&method.decorators);
        if detector.has_private_in_expression {
            return true;
        }
        method.value.params.items.iter().any(|param| {
            detector.visit_decorators(&param.decorators);
            detector.has_private_in_expression
        })
    }
}

/// Visitor to change references to the class to a local alias
/// <https://github.com/microsoft/TypeScript/blob/8da951cbb629b648753454872df4e1754982aef1/src/compiler/transformers/legacyDecorators.ts#L770-L783>
struct ClassReferenceChanger<'a, 'ctx> {
    class_binding: BoundIdentifier<'a>,
    // `Some` if there are references to the class inside the class body
    class_alias_binding: Option<BoundIdentifier<'a>>,
    ctx: &'ctx mut TraverseCtx<'a>,
    transformer_ctx: &'ctx TransformCtx<'a>,
}

impl<'a, 'ctx> ClassReferenceChanger<'a, 'ctx> {
    fn new(
        class_binding: BoundIdentifier<'a>,
        ctx: &'ctx mut TraverseCtx<'a>,
        transformer_ctx: &'ctx TransformCtx<'a>,
    ) -> Self {
        Self { class_binding, class_alias_binding: None, ctx, transformer_ctx }
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
    #[inline]
    fn visit_identifier_reference(&mut self, ident: &mut IdentifierReference<'a>) {
        if self.is_class_reference(ident) {
            *ident = self.get_alias_ident_reference();
        }
    }
}

impl<'a> ClassReferenceChanger<'a, '_> {
    // Check if the identifier reference is a reference to the class
    fn is_class_reference(&self, ident: &IdentifierReference<'a>) -> bool {
        self.ctx
            .symbols()
            .get_reference(ident.reference_id())
            .symbol_id()
            .is_some_and(|symbol_id| self.class_binding.symbol_id == symbol_id)
    }

    fn get_alias_ident_reference(&mut self) -> IdentifierReference<'a> {
        let binding = self.class_alias_binding.get_or_insert_with(|| {
            self.transformer_ctx.var_declarations.create_uid_var(&self.class_binding.name, self.ctx)
        });

        binding.create_read_reference(self.ctx)
    }
}
