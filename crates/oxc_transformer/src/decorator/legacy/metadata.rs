/// Emitting decorator metadata.
///
/// This plugin is used to emit decorator metadata for legacy decorators by
/// the `__metadata` helper.
///
/// ## Example
///
/// Input:
/// ```ts
/// class Demo {
///   @LogMethod
///   public foo(bar: number) {}
///
///   @Prop
///   prop: string = "hello";
/// }
/// ```
///
/// Output:
/// ```js
/// class Demo {
///   foo(bar) {}
///   prop = "hello";
/// }
/// babelHelpers.decorate([
///   LogMethod,
///   babelHelpers.decorateParam(0, babelHelpers.decorateMetadata("design:type", Function)),
///   babelHelpers.decorateParam(0, babelHelpers.decorateMetadata("design:paramtypes", [Number])),
///   babelHelpers.decorateParam(0, babelHelpers.decorateMetadata("design:returntype", void 0))
/// ], Demo.prototype, "foo", null);
/// babelHelpers.decorate([Prop, babelHelpers.decorateMetadata("design:type", String)], Demo.prototype, "prop", void 0);
/// ```
///
/// ## Implementation
///
/// Implementation based on https://github.com/microsoft/TypeScript/blob/d85767abfd83880cea17cea70f9913e9c4496dcc/src/compiler/transformers/ts.ts#L1119-L1136
///
/// ## Limitations
///
/// ### Compared to TypeScript
///
/// We do not have TypeScript's type-checker, so we cannot statically classify a
/// cross-file reference (`String` / `Number` / `Object`) the way tsc does.
/// Instead we emit the `typeof X === "undefined" ? Object : X` guard used by
/// SWC and Babel — matching the established ecosystem default.
///
/// For example:
///
/// Input:
/// ```ts
/// import { Foo } from "./mod";
/// class Cls {
///   @dec
///   p: Foo = ""
/// }
/// ```
///
/// TypeScript Output:
/// ```js
/// __decorate([
///   dec,
///   __metadata("design:type", typeof (_a = typeof Foo !== "undefined" && Foo) === "function" ? _a : Object)
/// ], Cls.prototype, "p", void 0);
/// ```
///
/// OXC Output:
/// ```js
/// babelHelpers.decorate([
///   dec,
///   babelHelpers.decorateMetadata("design:type", typeof Foo === "undefined" ? Object : Foo)
/// ], Cls.prototype, "p", void 0);
/// ```
///
/// The OXC form returns the actual binding (including enum objects) when
/// defined — see [#14740](https://github.com/oxc-project/oxc/issues/14740) for
/// the design rationale.
///
/// ## References
/// * TypeScript's [emitDecoratorMetadata](https://www.typescriptlang.org/tsconfig#emitDecoratorMetadata)
use oxc_allocator::Box as ArenaBox;
use oxc_ast::ast::*;
use oxc_data_structures::stack::SparseStack;
use oxc_semantic::{Reference, ReferenceFlags, SymbolId};
use oxc_span::{ContentEq, SPAN};
use oxc_traverse::{MaybeBoundIdentifier, Traverse};
use rustc_hash::FxHashMap;

use crate::{
    Helper, common::helper_loader::helper_call_expr, context::TraverseCtx,
    decorator::DecoratorOptions, state::TransformState, utils::ast_builder::create_property_access,
};

/// Type of an enum inferred from its members
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EnumType {
    /// All members are string literals or template literals with string-only expressions
    String,
    /// All members are numeric, bigint, unary numeric, or auto-incremented
    Number,
    /// Mixed types or computed values
    Object,
}

/// Metadata for decorated methods
pub(super) struct MethodMetadata<'a> {
    /// The `design:type` metadata expression
    pub r#type: Expression<'a>,
    /// The `design:paramtypes` metadata expression
    pub param_types: Expression<'a>,
    /// The `design:returntype` metadata expression (optional, omitted for getters/setters)
    pub return_type: Option<Expression<'a>>,
}

pub struct LegacyDecoratorMetadata<'a> {
    /// Stack of method metadata.
    ///
    /// Only the method that needs to be pushed onto a stack is the method metadata,
    /// which should be inserted after all real decorators. However, method parameters
    /// will be processed before the metadata generation, so we need to temporarily store
    /// them in a stack and pop them when in exit_method_definition.
    method_metadata_stack: SparseStack<MethodMetadata<'a>>,
    /// Stack of constructor metadata expressions, each expression
    /// is the `design:paramtypes`.
    ///
    /// Same as `method_metadata_stack`, but for constructors. Because the constructor is specially treated
    /// in the class, we need to handle it in `exit_class` rather than `exit_method_definition`.
    constructor_metadata_stack: SparseStack<Expression<'a>>,
    enum_types: FxHashMap<SymbolId, EnumType>,
    strict_null_checks: bool,
}

impl LegacyDecoratorMetadata<'_> {
    pub fn new(options: DecoratorOptions) -> Self {
        LegacyDecoratorMetadata {
            method_metadata_stack: SparseStack::new(),
            constructor_metadata_stack: SparseStack::new(),
            enum_types: FxHashMap::default(),
            strict_null_checks: options.strict_null_checks,
        }
    }
}

impl<'a> Traverse<'a, TransformState<'a>> for LegacyDecoratorMetadata<'a> {
    #[inline]
    fn exit_program(&mut self, _program: &mut Program<'a>, _ctx: &mut TraverseCtx<'a>) {
        debug_assert!(
            self.method_metadata_stack.is_exhausted(),
            "All method metadata should have been popped."
        );
        debug_assert!(
            self.constructor_metadata_stack.is_exhausted(),
            "All constructor metadata should have been popped."
        );
    }

    // `#[inline]` because this is a hot path and most `Statement`s are not `TSEnumDeclaration`s.
    // We want to avoid overhead of a function call for the common case.
    #[inline]
    fn enter_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        // Collect enum types here instead of in `enter_ts_enum_declaration` because the TypeScript
        // plugin transforms enum declarations in `enter_statement`, and we need to collect the
        // enum type before it gets transformed.
        if let Statement::TSEnumDeclaration(decl) = stmt {
            self.collect_enum_type(decl, ctx);
        }
    }

    fn enter_class(&mut self, class: &mut Class<'a>, ctx: &mut TraverseCtx<'a>) {
        let should_transform = !(class.is_expression() || class.declare);

        let constructor = class.body.body.iter_mut().find_map(|item| match item {
            ClassElement::MethodDefinition(method)
                if method.kind.is_constructor() && method.value.body.is_some() =>
            {
                Some(method)
            }
            _ => None,
        });

        let metadata = if should_transform
            && let Some(constructor) = constructor
            && !(class.decorators.is_empty()
                && constructor.value.params.items.iter().all(|param| param.decorators.is_empty()))
        {
            let serialized_type =
                self.serialize_parameters_types_of_node(&constructor.value.params, ctx);

            Some(self.create_metadata("design:paramtypes", serialized_type, ctx))
        } else {
            None
        };

        self.constructor_metadata_stack.push(metadata);
    }

    fn enter_method_definition(
        &mut self,
        method: &mut MethodDefinition<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if method.kind.is_constructor() {
            // Handle constructor in `enter_class`
            return;
        }

        let is_typescript_syntax = method.value.is_typescript_syntax();
        let is_decorated = !is_typescript_syntax
            && (!method.decorators.is_empty()
                || method.value.params.items.iter().any(|param| !param.decorators.is_empty()));

        let metadata = is_decorated.then(|| {
            // TypeScript only emits `design:returntype` for regular methods,
            // not for getters or setters.

            let (design_type, return_type) = if method.kind.is_get() {
                // For getters, the design type is the return type (or `Object` if untyped)
                (self.serialize_type_annotation(method.value.return_type.as_ref(), ctx), None)
            } else if method.kind.is_set()
                && let Some(param) = method.value.params.items.first()
            {
                // For setters, the design type is the type of the first parameter
                (self.serialize_parameter_types_of_node(param, ctx), None)
            } else {
                // For methods, the design type is always `Function`
                (
                    Self::global_function(ctx),
                    Some(self.serialize_return_type_of_node(&method.value, ctx)),
                )
            };

            let param_types = self.serialize_parameters_types_of_node(&method.value.params, ctx);

            MethodMetadata {
                r#type: self.create_metadata("design:type", design_type, ctx),
                param_types: self.create_metadata("design:paramtypes", param_types, ctx),
                return_type: return_type.map(|t| self.create_metadata("design:returntype", t, ctx)),
            }
        });

        self.method_metadata_stack.push(metadata);
    }

    #[inline]
    fn enter_property_definition(
        &mut self,
        prop: &mut PropertyDefinition<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if prop.decorators.is_empty() {
            return;
        }
        prop.decorators.push(self.create_design_type_metadata(prop.type_annotation.as_ref(), ctx));
    }

    #[inline]
    fn enter_accessor_property(
        &mut self,
        prop: &mut AccessorProperty<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if !prop.decorators.is_empty() {
            prop.decorators
                .push(self.create_design_type_metadata(prop.type_annotation.as_ref(), ctx));
        }
    }
}

impl<'a> LegacyDecoratorMetadata<'a> {
    /// Collects enum type information for decorator metadata generation.
    fn collect_enum_type(&mut self, decl: &TSEnumDeclaration<'a>, ctx: &TraverseCtx<'a>) {
        let symbol_id = decl.id.symbol_id();

        // Optimization:
        // If the enum doesn't have any type references, that implies that no decorators
        // refer to this enum, so there is no need to infer its type.
        let has_type_reference =
            ctx.scoping().get_resolved_references(symbol_id).any(Reference::is_type);
        if has_type_reference {
            let enum_type = Self::infer_enum_type(&decl.body.members);
            self.enum_types.insert(symbol_id, enum_type);
        }
    }

    /// Infer the type of an enum based on its members
    fn infer_enum_type(members: &[TSEnumMember<'a>]) -> EnumType {
        let mut enum_type = EnumType::Object;

        for member in members {
            if let Some(init) = &member.initializer {
                match init {
                    Expression::StringLiteral(_) | Expression::TemplateLiteral(_)
                        if enum_type != EnumType::Number =>
                    {
                        enum_type = EnumType::String;
                    }
                    // TS considers `+x`, `-x`, `~x` to be `Number` type, no matter what `x` is.
                    // All other unary expressions (`!x`, `void x`, `typeof x`, `delete x`) are illegal in enum initializers,
                    // so we can ignore those cases here and just say all `UnaryExpression`s are numeric.
                    // Bigint literals are also illegal in enum initializers, so we don't need to consider them here.
                    Expression::NumericLiteral(_) | Expression::UnaryExpression(_)
                        if enum_type != EnumType::String =>
                    {
                        enum_type = EnumType::Number;
                    }
                    // For other expressions, we can't determine the type statically
                    _ => return EnumType::Object,
                }
            } else {
                // No initializer means numeric (auto-incrementing from previous member)
                if enum_type == EnumType::String {
                    return EnumType::Object;
                }
                enum_type = EnumType::Number;
            }
        }

        enum_type
    }

    pub fn pop_method_metadata(&mut self) -> Option<MethodMetadata<'a>> {
        self.method_metadata_stack.pop()
    }

    pub fn pop_constructor_metadata(&mut self) -> Option<Expression<'a>> {
        self.constructor_metadata_stack.pop()
    }

    fn serialize_type_annotation(
        &mut self,
        type_annotation: Option<&ArenaBox<'a, TSTypeAnnotation<'a>>>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        if let Some(type_annotation) = type_annotation {
            self.serialize_type_node(&type_annotation.type_annotation, ctx)
        } else {
            Self::global_object(ctx)
        }
    }

    /// Serializes a type node for use with decorator type metadata.
    ///
    /// Types are serialized in the following fashion:
    /// - Void types point to "undefined" (e.g. "void 0")
    /// - Function and Constructor types point to the global "Function" constructor.
    /// - Interface types with a call or construct signature types point to the global
    ///   "Function" constructor.
    /// - Array and Tuple types point to the global "Array" constructor.
    /// - Type predicates and booleans point to the global "Boolean" constructor.
    /// - String literal types and strings point to the global "String" constructor.
    /// - Enum and number types point to the global "Number" constructor.
    /// - Symbol types point to the global "Symbol" constructor.
    /// - Type references to classes (or class-like variables) point to the constructor for the class.
    /// - Anything else points to the global "Object" constructor.
    fn serialize_type_node(
        &mut self,
        node: &TSType<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        match &node {
            TSType::TSVoidKeyword(_)
            | TSType::TSUndefinedKeyword(_)
            | TSType::TSNullKeyword(_)
            | TSType::TSNeverKeyword(_) => ctx.ast.void_0(SPAN),
            TSType::TSFunctionType(_) | TSType::TSConstructorType(_) => Self::global_function(ctx),
            TSType::TSArrayType(_) | TSType::TSTupleType(_) => Self::global_array(ctx),
            TSType::TSTypePredicate(t) => {
                if t.asserts {
                    ctx.ast.void_0(SPAN)
                } else {
                    Self::global_boolean(ctx)
                }
            }
            TSType::TSBooleanKeyword(_) => Self::global_boolean(ctx),
            TSType::TSTemplateLiteralType(_) | TSType::TSStringKeyword(_) => {
                Self::global_string(ctx)
            }
            TSType::TSLiteralType(literal) => {
                Self::serialize_literal_of_literal_type_node(&literal.literal, ctx)
            }
            TSType::TSNumberKeyword(_) => Self::global_number(ctx),
            TSType::TSBigIntKeyword(_) => Self::global_bigint(ctx),
            TSType::TSSymbolKeyword(_) => Self::global_symbol(ctx),
            TSType::TSTypeReference(t) => {
                self.serialize_type_reference_node(&t.type_name, ctx)
            }
            TSType::TSIntersectionType(t) => {
                self.serialize_union_or_intersection_constituents(t.types.iter(), /* is_intersection */ true, ctx)
            }
            TSType::TSUnionType(t) => {
                self.serialize_union_or_intersection_constituents(t.types.iter(), /* is_intersection */ false, ctx)
            }
            TSType::TSConditionalType(t) => {
                self.serialize_union_or_intersection_constituents(
                    [&t.true_type, &t.false_type].into_iter(),
                    false,
                    ctx
                )
            }
            TSType::TSTypeOperatorType(operator)
                if operator.operator == TSTypeOperatorOperator::Readonly =>
            {
                self.serialize_type_node(&operator.type_annotation, ctx)
            }
            TSType::JSDocNullableType(t) => {
                self.serialize_type_node(&t.type_annotation, ctx)
            }
            TSType::JSDocNonNullableType(t) => {
                self.serialize_type_node(&t.type_annotation, ctx)
            }
            TSType::TSParenthesizedType(t) => {
                self.serialize_type_node(&t.type_annotation, ctx)
            }
            TSType::TSObjectKeyword(_)
            // Fallback to `Object`
            | TSType::TSTypeQuery(_) | TSType::TSIndexedAccessType(_) | TSType::TSMappedType(_)
            | TSType::TSTypeLiteral(_) | TSType::TSAnyKeyword(_) | TSType::TSUnknownKeyword(_)
            | TSType::TSThisType(_) | TSType::TSImportType(_) | TSType::TSTypeOperatorType(_)
            // Not allowed to be used in the start of type annotations, fallback to `Object`
            | TSType::TSInferType(_) | TSType::TSIntrinsicKeyword(_) | TSType::TSNamedTupleMember(_)
            | TSType::JSDocUnknownType(_) => Self::global_object(ctx),
        }
    }

    /// Serializes the type of a node for use with decorator type metadata.
    fn serialize_parameters_types_of_node(
        &mut self,
        params: &FormalParameters<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let mut elements =
            ctx.ast.vec_with_capacity(params.items.len() + usize::from(params.rest.is_some()));
        elements.extend(params.items.iter().map(|param| {
            ArrayExpressionElement::from(self.serialize_parameter_types_of_node(param, ctx))
        }));

        if let Some(rest) = &params.rest {
            elements.push(ArrayExpressionElement::from(
                self.serialize_type_annotation(rest.type_annotation.as_ref(), ctx),
            ));
        }
        ctx.ast.expression_array(SPAN, elements)
    }

    fn serialize_parameter_types_of_node(
        &mut self,
        param: &FormalParameter<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let type_annotation = param.type_annotation.as_ref();
        self.serialize_type_annotation(type_annotation, ctx)
    }

    /// Serializes the return type of a node for use with decorator type metadata.
    fn serialize_return_type_of_node(
        &mut self,
        func: &Function<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        if func.r#async {
            Self::global_promise(ctx)
        } else if let Some(return_type) = &func.return_type {
            self.serialize_type_node(&return_type.type_annotation, ctx)
        } else {
            ctx.ast.void_0(SPAN)
        }
    }

    /// Serializes a type reference for `design:type` / `design:paramtypes` / `design:returntype`.
    ///
    /// Matches the emit shape used by SWC and Babel:
    ///
    /// - `X` → `typeof X === "undefined" ? Object : X`
    /// - `A.B` → `typeof A === "undefined" || typeof A.B === "undefined" ? Object : A.B`
    /// - `A.B.C` → `typeof A === "undefined" || typeof A.B === "undefined" || typeof A.B.C === "undefined" ? Object : A.B.C`
    ///
    /// Local enums are pre-classified through [`Self::enum_types`] and emit `String` /
    /// `Number` / `Object` directly (matching SWC and tsc).
    /// Type-only references and `this`-typed entity names emit `Object` without a
    /// runtime check (matching tsc).
    ///
    /// See [issue #14740](https://github.com/oxc-project/oxc/issues/14740): the guard
    /// short-circuits to `Object` when the binding is missing, but evaluates to the
    /// actual runtime binding when present — including enum objects, so consumers like
    /// `type-graphql` and `typeorm` can introspect them via `Object.values()`.
    fn serialize_type_reference_node(
        &self,
        name: &TSTypeName<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let Some((root_ident, properties)) = Self::decompose_entity_name(name) else {
            // `this`-typed entity name — no runtime binding to check.
            return Self::global_object(ctx);
        };

        let symbol_id = ctx.scoping().get_reference(root_ident.reference_id()).symbol_id();

        // Local enum fast path: classify by member shape and emit the primitive globally.
        if properties.is_empty()
            && let Some(symbol_id) = symbol_id
            && let Some(enum_type) = self.enum_types.get(&symbol_id)
        {
            return match enum_type {
                EnumType::String => Self::global_string(ctx),
                EnumType::Number => Self::global_number(ctx),
                EnumType::Object => Self::global_object(ctx),
            };
        }

        // `ReadonlyArray<T>` is a TS-only utility type; tsc emits `Array`. Skip
        // when shadowed by any local declaration so the user's binding wins.
        if properties.is_empty() && root_ident.name == "ReadonlyArray" && symbol_id.is_none() {
            return Self::global_array(ctx);
        }

        // Type-only references have no runtime binding either.
        if Self::is_type_symbol(symbol_id, ctx) {
            return Self::global_object(ctx);
        }

        Self::build_undefined_guard(root_ident, &properties, ctx)
    }

    /// Walk a [`TSTypeName`] from leaf to root, returning the root identifier and the
    /// ordered list of property names (root → leaf). Returns `None` if the entity name
    /// is rooted at a `this`-type, which has no static identifier to guard.
    fn decompose_entity_name<'b>(
        name: &'b TSTypeName<'a>,
    ) -> Option<(&'b IdentifierReference<'a>, Vec<&'b str>)> {
        let mut properties: Vec<&str> = vec![];
        let mut current = name;
        loop {
            match current {
                TSTypeName::IdentifierReference(ident) => {
                    properties.reverse();
                    return Some((ident, properties));
                }
                TSTypeName::QualifiedName(q) => {
                    properties.push(q.right.name.as_str());
                    current = &q.left;
                }
                TSTypeName::ThisExpression(_) => return None,
            }
        }
    }

    /// Build the SWC-style undefined-fallback guard for the given root + property path.
    ///
    /// OR-chains `typeof <prefix> === "undefined"` for every prefix from root to leaf.
    /// Left-to-right short-circuit keeps the chain safe even when an intermediate is
    /// undefined — the later `typeof <prefix>.foo` is never evaluated.
    fn build_undefined_guard(
        root: &IdentifierReference<'a>,
        properties: &[&str],
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let binding = MaybeBoundIdentifier::from_identifier_reference(root, ctx);
        let ref_flags = Self::get_reference_flags(&binding, ctx);

        let root_expr = binding.create_expression(ref_flags, ctx);
        let mut test = Self::typeof_undefined(root_expr, ctx);
        for i in 1..=properties.len() {
            let prefix = Self::build_path(&binding, ref_flags, &properties[..i], ctx);
            let next = Self::typeof_undefined(prefix, ctx);
            test = ctx.ast.expression_logical(SPAN, test, LogicalOperator::Or, next);
        }

        let alternate = Self::build_path(&binding, ref_flags, properties, ctx);
        ctx.ast.expression_conditional(SPAN, test, Self::global_object(ctx), alternate)
    }

    fn build_path(
        binding: &MaybeBoundIdentifier<'a>,
        ref_flags: ReferenceFlags,
        properties: &[&str],
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let mut expr = binding.create_expression(ref_flags, ctx);
        for prop in properties {
            expr = create_property_access(SPAN, expr, prop, ctx);
        }
        expr
    }

    fn typeof_undefined(expr: Expression<'a>, ctx: &TraverseCtx<'a>) -> Expression<'a> {
        let typeof_expr = ctx.ast.expression_unary(SPAN, UnaryOperator::Typeof, expr);
        let undefined_str = ctx.ast.expression_string_literal(SPAN, "undefined", None);
        ctx.ast.expression_binary(SPAN, typeof_expr, BinaryOperator::StrictEquality, undefined_str)
    }

    fn serialize_literal_of_literal_type_node(
        literal: &TSLiteral<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        match literal {
            TSLiteral::BooleanLiteral(_) => Self::global_boolean(ctx),
            TSLiteral::NumericLiteral(_) => Self::global_number(ctx),
            TSLiteral::BigIntLiteral(_) => Self::global_bigint(ctx),
            TSLiteral::StringLiteral(_) | TSLiteral::TemplateLiteral(_) => Self::global_string(ctx),
            TSLiteral::UnaryExpression(expr) => match expr.argument {
                Expression::NumericLiteral(_) => Self::global_number(ctx),
                Expression::StringLiteral(_) => Self::global_string(ctx),
                // Cannot be a type annotation
                _ => unreachable!(),
            },
        }
    }

    fn serialize_union_or_intersection_constituents<'t>(
        &mut self,
        types: impl Iterator<Item = &'t TSType<'a>>,
        is_intersection: bool,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a>
    where
        'a: 't,
    {
        let mut serialized_type = None;

        for t in types {
            let t = t.without_parenthesized();
            match t {
                TSType::TSNeverKeyword(_) => {
                    if is_intersection {
                        // Reduce to `never` in an intersection
                        return ctx.ast.void_0(SPAN);
                    }
                    // Elide `never` in a union
                    continue;
                }
                // Elide `null` and `undefined` in a union when strictNullChecks is off
                TSType::TSNullKeyword(_) | TSType::TSUndefinedKeyword(_)
                    if !is_intersection && !self.strict_null_checks =>
                {
                    continue;
                }
                TSType::TSUnknownKeyword(_) => {
                    if !is_intersection {
                        // Reduce to `unknown` in a union
                        return Self::global_object(ctx);
                    }
                    // Elide `unknown` in an intersection
                    continue;
                }
                TSType::TSAnyKeyword(_) => {
                    return Self::global_object(ctx);
                }
                // Unlike TypeScript, we don't have a way to determine what the referent is,
                // so return `Object` early, because once have a type reference, the final
                // type is `Object` anyway.
                TSType::TSTypeReference(_) => return Self::global_object(ctx),
                _ => {}
            }

            let serialized_constituent = self.serialize_type_node(t, ctx);
            if matches!(&serialized_constituent, Expression::Identifier(ident) if ident.name == "Object")
            {
                // One of the individual is global object, return immediately
                return serialized_constituent;
            }

            // If there exists union that is not `void 0` expression, check if the the common type is identifier.
            // anything more complex and we will just default to Object
            if let Some(serialized_type) = &serialized_type {
                // Different types
                if !Self::equate_serialized_type_nodes(serialized_type, &serialized_constituent) {
                    return Self::global_object(ctx);
                }
            } else {
                // Initialize the union type
                serialized_type = Some(serialized_constituent);
            }
        }

        // If we were able to find common type, use it
        serialized_type.unwrap_or_else(|| {
            // Fallback is only hit if all union constituents are null/undefined/never
            ctx.ast.void_0(SPAN)
        })
    }

    /// Compares two serialized type nodes for equality.
    ///
    /// <https://github.com/microsoft/TypeScript/blob/d85767abfd83880cea17cea70f9913e9c4496dcc/src/compiler/transformers/typeSerializer.ts#L449-L484>
    #[inline]
    fn equate_serialized_type_nodes(a: &Expression<'a>, b: &Expression<'a>) -> bool {
        a.content_eq(b)
    }

    #[inline]
    fn is_type_symbol(symbol_id: Option<oxc_semantic::SymbolId>, ctx: &TraverseCtx<'a>) -> bool {
        symbol_id.is_some_and(|symbol_id| ctx.scoping().symbol_flags(symbol_id).is_type())
    }

    fn get_reference_flags(
        binding: &MaybeBoundIdentifier<'a>,
        ctx: &TraverseCtx<'a>,
    ) -> ReferenceFlags {
        if let Some(symbol_id) = binding.symbol_id {
            // Type symbols have filtered out in [`serialize_entity_name_as_expression_fallback`].
            debug_assert!(ctx.scoping().symbol_flags(symbol_id).is_value());
            // `design::*type` would be called by `reflect-metadata` APIs, use `Read` flag
            // to avoid TypeScript remove it because only used as types.
            ReferenceFlags::Read
        } else {
            // Unresolved reference
            ReferenceFlags::Type | ReferenceFlags::Read
        }
    }

    #[inline]
    fn create_global_identifier(ident: &'static str, ctx: &mut TraverseCtx<'a>) -> Expression<'a> {
        ctx.create_unbound_ident_expr(SPAN, ctx.ast.ident(ident), ReferenceFlags::Read)
    }

    #[inline]
    fn global_object(ctx: &mut TraverseCtx<'a>) -> Expression<'a> {
        Self::create_global_identifier("Object", ctx)
    }

    #[inline]
    fn global_function(ctx: &mut TraverseCtx<'a>) -> Expression<'a> {
        Self::create_global_identifier("Function", ctx)
    }

    #[inline]
    fn global_array(ctx: &mut TraverseCtx<'a>) -> Expression<'a> {
        Self::create_global_identifier("Array", ctx)
    }

    #[inline]
    fn global_boolean(ctx: &mut TraverseCtx<'a>) -> Expression<'a> {
        Self::create_global_identifier("Boolean", ctx)
    }

    #[inline]
    fn global_string(ctx: &mut TraverseCtx<'a>) -> Expression<'a> {
        Self::create_global_identifier("String", ctx)
    }

    #[inline]
    fn global_number(ctx: &mut TraverseCtx<'a>) -> Expression<'a> {
        Self::create_global_identifier("Number", ctx)
    }

    #[inline]
    fn global_bigint(ctx: &mut TraverseCtx<'a>) -> Expression<'a> {
        Self::create_global_identifier("BigInt", ctx)
    }

    #[inline]
    fn global_symbol(ctx: &mut TraverseCtx<'a>) -> Expression<'a> {
        Self::create_global_identifier("Symbol", ctx)
    }

    #[inline]
    fn global_promise(ctx: &mut TraverseCtx<'a>) -> Expression<'a> {
        Self::create_global_identifier("Promise", ctx)
    }

    // `_metadata(key, value)
    #[expect(clippy::unused_self)]
    fn create_metadata(
        &self,
        key: &'a str,
        value: Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let arguments = ctx.ast.vec_from_array([
            Argument::from(ctx.ast.expression_string_literal(SPAN, key, None)),
            Argument::from(value),
        ]);
        helper_call_expr(Helper::DecorateMetadata, arguments, ctx)
    }

    // `_metadata(key, value)
    fn create_metadata_decorate(
        &self,
        key: &'a str,
        value: Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Decorator<'a> {
        ctx.ast.decorator(SPAN, self.create_metadata(key, value, ctx))
    }

    /// `_metadata("design:type", type)`
    fn create_design_type_metadata(
        &mut self,
        type_annotation: Option<&ArenaBox<'a, TSTypeAnnotation<'a>>>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Decorator<'a> {
        let serialized_type = self.serialize_type_annotation(type_annotation, ctx);
        self.create_metadata_decorate("design:type", serialized_type, ctx)
    }
}
