use oxc_allocator::{ArenaBox, ArenaVec, CloneIn, GetAllocator};
use oxc_ast::ast::*;
use oxc_span::{SPAN, Span};

use crate::{
    IsolatedDeclarations,
    diagnostics::{
        function_must_have_explicit_return_type, implicitly_adding_undefined_to_type,
        parameter_must_have_explicit_type,
    },
    formal_parameter_binding_pattern::FormalParameterBindingPattern,
};

impl<'a> IsolatedDeclarations<'a> {
    pub(crate) fn transform_function(
        &self,
        func: &Function<'a>,
        declare: Option<bool>,
    ) -> ArenaBox<'a, Function<'a>> {
        let return_type = self.infer_function_return_type(func);
        if return_type.is_none() {
            self.error(function_must_have_explicit_return_type(get_function_span(func)));
        }
        let params = self.transform_formal_parameters(&func.params, false);
        Function::boxed(
            func.span,
            func.r#type,
            func.id.clone_in(self.allocator()),
            false,
            false,
            declare.unwrap_or_else(|| self.is_declare()),
            func.type_parameters.clone_in(self.allocator()),
            func.this_param.clone_in(self.allocator()),
            params,
            return_type,
            None,
            self,
        )
    }

    pub(crate) fn transform_formal_parameter(
        &self,
        param: &FormalParameter<'a>,
        is_remaining_params_have_required: bool,
        in_private_constructor: bool,
    ) -> Option<FormalParameter<'a>> {
        let pattern = &param.pattern;
        if param.initializer.is_some()
            && pattern.is_destructuring_pattern()
            && param.type_annotation.is_none()
        {
            self.error(parameter_must_have_explicit_type(param.span));
            return None;
        }

        let is_assignment_pattern = param.initializer.is_some();
        let mut pattern = if let BindingPattern::AssignmentPattern(pattern) = &param.pattern {
            pattern.left.clone_in(self.allocator())
        } else {
            param.pattern.clone_in(self.allocator())
        };

        FormalParameterBindingPattern::remove_assignments_from_kind(&mut pattern);

        if is_assignment_pattern
            || param.type_annotation.is_none()
            || (param.optional && param.has_modifier())
        {
            let type_annotation = param
                .type_annotation
                .as_ref()
                .map(|type_annotation| type_annotation.type_annotation.clone_in(self.allocator()))
                .or_else(|| {
                    let new_type = self.infer_type_from_formal_parameter(param);
                    // A private parameter property on a private constructor needs no
                    // explicit type: the constructor signature is collapsed to
                    // `private constructor();` and the class member is emitted as
                    // `private readonly name;` with no type annotation.
                    let is_elided_private_param = in_private_constructor
                        && param.accessibility.is_some_and(TSAccessibility::is_private);
                    if new_type.is_none() && !is_elided_private_param {
                        self.error(parameter_must_have_explicit_type(param.span));
                    }
                    new_type
                })
                .map(|ts_type| {
                    // If a defaulted parameter is followed by a required parameter, declaration
                    // emit may need to add `undefined` to its type because the parameter cannot be
                    // marked optional. Preserve annotations whose resolved type is unknown: unlike
                    // TypeScript, isolated declaration emit does not have a type checker available
                    // to determine whether they already include `undefined`.
                    if (is_remaining_params_have_required
                        || (param.optional && param.has_modifier()))
                        && !has_explicit_undefined_union_member(&ts_type)
                    {
                        let presence = undefined_presence(&ts_type);
                        let can_add = can_add_undefined(&ts_type);
                        match (presence, can_add) {
                            (UndefinedPresence::Present, _)
                            | (UndefinedPresence::Unresolved, false) => {}
                            (_, true) => {
                                // Adding `undefined` is either required or redundant after
                                // TypeScript resolves the type.
                                let undefined = TSType::new_ts_undefined_keyword(SPAN, self);
                                let ts_type = if let TSType::TSUnionType(mut union) = ts_type {
                                    union.types.push(undefined);
                                    TSType::TSUnionType(union)
                                } else {
                                    TSType::new_ts_union_type(SPAN, [ts_type, undefined], self)
                                };
                                return TSTypeAnnotation::boxed(SPAN, ts_type, self);
                            }
                            (_, false) => {
                                self.error(implicitly_adding_undefined_to_type(param.span));
                            }
                        }
                    }

                    TSTypeAnnotation::boxed(SPAN, ts_type, self)
                });

            let optional =
                param.optional || (!is_remaining_params_have_required && is_assignment_pattern);
            return Some(FormalParameter::new(
                param.span,
                [],
                // `pattern` is already an owned, freshly-cloned binding (see above) and is
                // not used afterwards, so move it in directly instead of cloning again.
                pattern,
                type_annotation,
                None,
                optional,
                None,
                false,
                false,
                self,
            ));
        }

        Some(FormalParameter::new(
            param.span,
            [],
            pattern,
            param.type_annotation.clone_in(self.allocator()),
            None,
            param.optional,
            None,
            false,
            false,
            self,
        ))
    }

    pub(crate) fn transform_formal_parameters(
        &self,
        params: &ArenaBox<'a, FormalParameters<'a>>,
        in_private_constructor: bool,
    ) -> ArenaBox<'a, FormalParameters<'a>> {
        if params.kind.is_signature() || (params.rest.is_none() && params.items.is_empty()) {
            return params.clone_in(self.allocator());
        }

        let items = ArenaVec::from_iter_in(
            params
                .items
                .iter()
                .enumerate()
                .filter(|(_, item)| !in_private_constructor || item.has_modifier())
                .filter_map(|(index, item)| {
                    let is_remaining_params_have_required = params
                        .items
                        .iter()
                        .skip(index)
                        .any(|item| !(item.optional || item.initializer.is_some()));
                    self.transform_formal_parameter(
                        item,
                        is_remaining_params_have_required,
                        in_private_constructor,
                    )
                }),
            self,
        );

        if let Some(rest) = &params.rest
            && rest.type_annotation.is_none()
        {
            self.error(parameter_must_have_explicit_type(rest.span));
        }

        let rest = params.rest.as_ref().map(|rest| {
            let mut rest = rest.clone_in(self.allocator());
            FormalParameterBindingPattern::remove_assignments_from_kind(&mut rest.rest.argument);
            rest
        });

        FormalParameters::boxed(params.span, FormalParameterKind::Signature, items, rest, self)
    }
}

pub fn get_function_span(func: &Function<'_>) -> Span {
    func.id.as_ref().map_or_else(|| Span::empty(func.params.span.start), |id| id.span)
}

/// Syntax-only categories that remain distinct while folding unions and intersections.
#[derive(Clone, Copy)]
enum UndefinedPresence {
    Present,
    Absent,
    /// `any` accepts `undefined` on its own but makes an intersection checker-dependent.
    Any,
    /// `never` excludes `undefined` and absorbs every other intersection operand.
    Never,
    /// `unknown` behaves like `Absent` for declaration emit but is the identity element of an
    /// intersection.
    UnknownKeyword,
    /// `void` excludes `undefined` on its own but retains it when intersected with `undefined`.
    Void,
    /// Resolving this annotation requires checker information.
    Unresolved,
}

impl UndefinedPresence {
    /// Combine the classifications of two union members.
    fn union(self, other: Self) -> Self {
        use UndefinedPresence::{Absent, Any, Never, Present, UnknownKeyword, Unresolved, Void};
        match (self, other) {
            (Unresolved, _) | (_, Unresolved) => Unresolved,
            (Any, _) | (_, Any) => Any,
            (UnknownKeyword, _) | (_, UnknownKeyword) => UnknownKeyword,
            (Present, _) | (_, Present) => Present,
            (Void, _) | (_, Void) => Void,
            (Absent, _) | (_, Absent) => Absent,
            (Never, Never) => Never,
        }
    }

    /// Combine the classifications of two intersection members.
    fn intersection(self, other: Self) -> Self {
        use UndefinedPresence::{Absent, Any, Never, Present, UnknownKeyword, Unresolved, Void};
        match (self, other) {
            (Never, _) | (_, Never) => Never,
            (Unresolved | Any, _) | (_, Unresolved | Any) => Unresolved,
            (Absent, _) | (_, Absent) => Absent,
            (Present, _) | (_, Present) => Present,
            (Void, _) | (_, Void) => Void,
            (UnknownKeyword, UnknownKeyword) => UnknownKeyword,
        }
    }
}

/// Classify whether annotation syntax proves that the parameter type includes `undefined`.
fn undefined_presence(ts_type: &TSType<'_>) -> UndefinedPresence {
    match ts_type {
        TSType::TSUndefinedKeyword(_) => UndefinedPresence::Present,
        TSType::TSAnyKeyword(_) => UndefinedPresence::Any,
        TSType::TSUnknownKeyword(_) => UndefinedPresence::UnknownKeyword,
        TSType::TSBigIntKeyword(_)
        | TSType::TSBooleanKeyword(_)
        | TSType::TSIntrinsicKeyword(_)
        | TSType::TSNullKeyword(_)
        | TSType::TSNumberKeyword(_)
        | TSType::TSObjectKeyword(_)
        | TSType::TSStringKeyword(_)
        | TSType::TSSymbolKeyword(_)
        | TSType::TSLiteralType(_)
        | TSType::TSFunctionType(_)
        | TSType::TSConstructorType(_)
        | TSType::TSArrayType(_)
        | TSType::TSTupleType(_)
        | TSType::TSMappedType(_)
        | TSType::TSTypeLiteral(_)
        | TSType::TSTypeOperatorType(_)
        | TSType::TSTemplateLiteralType(_)
        | TSType::TSThisType(_) => UndefinedPresence::Absent,
        TSType::TSNeverKeyword(_) => UndefinedPresence::Never,
        TSType::TSVoidKeyword(_) => UndefinedPresence::Void,
        TSType::TSParenthesizedType(parenthesized) => {
            undefined_presence(&parenthesized.type_annotation)
        }
        TSType::TSUnionType(union) => union
            .types
            .iter()
            .map(undefined_presence)
            .fold(UndefinedPresence::Never, UndefinedPresence::union),
        // Resolving these types requires type information. Treat them as unknown so declaration
        // emit does not report a false-positive TS9025 for a type that already contains
        // `undefined`. Some variants cannot occur directly as a parameter annotation, but listing
        // them keeps this classification exhaustive as new `TSType` variants are added.
        TSType::TSConditionalType(_)
        | TSType::TSImportType(_)
        | TSType::TSIndexedAccessType(_)
        | TSType::TSInferType(_)
        | TSType::TSNamedTupleMember(_)
        | TSType::TSTypePredicate(_)
        | TSType::TSTypeQuery(_)
        | TSType::TSTypeReference(_)
        | TSType::JSDocNullableType(_)
        | TSType::JSDocNonNullableType(_)
        | TSType::JSDocUnknownType(_) => UndefinedPresence::Unresolved,
        TSType::TSIntersectionType(intersection) => intersection
            .types
            .iter()
            .map(undefined_presence)
            .fold(UndefinedPresence::UnknownKeyword, UndefinedPresence::intersection),
    }
}

/// Whether the annotation itself already has an explicit `undefined` union member.
///
/// Do not descend through intersections: `undefined & string` excludes `undefined`, whereas a
/// parenthesized or nested union member still spells it explicitly.
fn has_explicit_undefined_union_member(ts_type: &TSType<'_>) -> bool {
    match ts_type {
        TSType::TSUndefinedKeyword(_) => true,
        TSType::TSParenthesizedType(parenthesized) => {
            has_explicit_undefined_union_member(&parenthesized.type_annotation)
        }
        TSType::TSUnionType(union) => union.types.iter().any(has_explicit_undefined_union_member),
        _ => false,
    }
}

/// Whether TypeScript can syntactically add `undefined` without using type information.
fn can_add_undefined(ts_type: &TSType<'_>) -> bool {
    if ts_type.is_keyword() {
        return true;
    }

    match ts_type {
        TSType::TSLiteralType(_)
        | TSType::TSFunctionType(_)
        | TSType::TSConstructorType(_)
        | TSType::TSArrayType(_)
        | TSType::TSTupleType(_)
        | TSType::TSTypeLiteral(_)
        | TSType::TSTemplateLiteralType(_)
        | TSType::TSThisType(_) => true,
        TSType::TSParenthesizedType(parenthesized) => {
            can_add_undefined(&parenthesized.type_annotation)
        }
        TSType::TSUnionType(union) => union.types.iter().all(can_add_undefined),
        TSType::TSIntersectionType(intersection) => {
            intersection.types.iter().all(can_add_undefined)
        }
        _ => false,
    }
}
