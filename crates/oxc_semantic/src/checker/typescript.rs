use std::{borrow::Cow, collections::hash_map::Entry};

use itertools::Itertools;
use rustc_hash::FxHashMap;

use oxc_ast::{AstKind, ast::*};
use oxc_ecmascript::BoundNames;
use oxc_span::{GetSpan, Span};
use oxc_str::{JSStr, Str};

use crate::{builder::SemanticBuilder, diagnostics};

pub fn check_jsdoc_nullable_type(ty: &JSDocNullableType<'_>, ctx: &SemanticBuilder<'_>) {
    let mut union = SuggestedUnion::default();
    union.add(&ty.type_annotation);
    if !union.is_never_or_void() {
        union.null |= !ty.postfix;
        union.undefined = true;
    }
    let suggestion = union.to_source(ctx.source_text);
    ctx.error(diagnostics::jsdoc_type_modifier('?', !ty.postfix, ty.span, &suggestion));
}

pub fn check_jsdoc_non_nullable_type(ty: &JSDocNonNullableType<'_>, ctx: &SemanticBuilder<'_>) {
    let mut union = SuggestedUnion::default();
    union.add(&ty.type_annotation);
    let suggestion = union.to_source(ctx.source_text);
    ctx.error(diagnostics::jsdoc_type_modifier('!', !ty.postfix, ty.span, &suggestion));
}

/// Approximates the type which TypeScript suggests for a JSDoc type modifier with
/// `strictNullChecks`: `T?` is `T | undefined`, `?T` is `T | null | undefined`, and `T!` is `T`.
#[derive(Default)]
struct SuggestedUnion<'a, 'b> {
    types: Vec<&'b TSType<'a>>,
    null: bool,
    undefined: bool,
    any: bool,
    unknown: bool,
}

impl<'a, 'b> SuggestedUnion<'a, 'b> {
    fn add(&mut self, ty: &'b TSType<'a>) {
        match ty {
            TSType::TSParenthesizedType(ty) => self.add(&ty.type_annotation),
            TSType::JSDocNonNullableType(ty) => self.add(&ty.type_annotation),
            TSType::JSDocNullableType(ty) => {
                self.add(&ty.type_annotation);
                self.null = true;
            }
            TSType::TSUnionType(ty) => ty.types.iter().for_each(|ty| self.add(ty)),
            TSType::TSNullKeyword(_) => self.null = true,
            TSType::TSUndefinedKeyword(_) => self.undefined = true,
            TSType::TSAnyKeyword(_) => self.any = true,
            TSType::TSUnknownKeyword(_) => self.unknown = true,
            TSType::TSNeverKeyword(_) => {}
            _ => self.types.push(ty),
        }
    }

    fn is_never_or_void(&self) -> bool {
        !(self.null || self.undefined || self.any || self.unknown)
            && matches!(self.types.as_slice(), [] | [TSType::TSVoidKeyword(_)])
    }

    fn to_source(&self, source_text: &str) -> String {
        if self.any {
            return "any".to_string();
        }
        if self.unknown {
            return "unknown".to_string();
        }
        let is_union = self.types.len() + usize::from(self.null) + usize::from(self.undefined) > 1;
        let types = self.types.iter().map(|ty| {
            let text = &source_text[ty.span()];
            let needs_parens = is_union
                && matches!(
                    ty,
                    TSType::TSFunctionType(_)
                        | TSType::TSConstructorType(_)
                        | TSType::TSConditionalType(_)
                        | TSType::TSIntersectionType(_)
                );
            if needs_parens { Cow::Owned(format!("({text})")) } else { Cow::Borrowed(text) }
        });
        let nullish = [self.null.then_some("null"), self.undefined.then_some("undefined")]
            .into_iter()
            .flatten()
            .map(Cow::Borrowed);
        let suggestion = types.chain(nullish).join(" | ");
        if suggestion.is_empty() { "never".to_string() } else { suggestion }
    }
}

pub fn check_jsdoc_unknown_type(ty: &JSDocUnknownType, ctx: &SemanticBuilder<'_>) {
    ctx.error(diagnostics::jsdoc_type(ty.span));
}

pub fn check_ts_type_predicate(predicate: &TSTypePredicate<'_>, ctx: &SemanticBuilder<'_>) {
    if !is_allowed_type_predicate_position(ctx) {
        ctx.error(diagnostics::type_predicate_only_in_return_type(predicate.span));
    }
}

fn is_allowed_type_predicate_position(ctx: &SemanticBuilder<'_>) -> bool {
    let mut ancestors = ctx.ancestry().ancestor_kinds();

    let Some(AstKind::TSTypeAnnotation(_)) = ancestors.next() else {
        return false;
    };

    match ancestors.next() {
        Some(AstKind::Function(_)) => match ancestors.next() {
            Some(AstKind::MethodDefinition(method)) => method.kind.is_method(),
            Some(AstKind::ObjectProperty(property)) => !property.kind.is_accessor(),
            _ => true,
        },
        Some(
            AstKind::ArrowFunctionExpression(_)
            | AstKind::TSFunctionType(_)
            | AstKind::TSCallSignatureDeclaration(_),
        ) => true,
        Some(AstKind::TSMethodSignature(signature)) => {
            signature.kind == TSMethodSignatureKind::Method
        }
        _ => false,
    }
}

pub fn check_ts_infer_type<'a>(infer_type: &TSInferType<'a>, ctx: &SemanticBuilder<'a>) {
    let is_in_conditional_extends_clause = ctx.ancestry().ancestor_kinds().any(|kind| {
        kind.as_ts_conditional_type().is_some_and(|conditional| {
            conditional.extends_type.span().contains_inclusive(infer_type.span)
        })
    });

    if !is_in_conditional_extends_clause {
        ctx.error(diagnostics::infer_declaration_only_permitted_in_extends_clause(infer_type.span));
    }
}

pub fn check_formal_parameters(params: &FormalParameters, ctx: &SemanticBuilder<'_>) {
    if params.kind == FormalParameterKind::Signature && params.items.len() > 1 {
        check_duplicate_bound_names(params, ctx);
    }
}

fn check_duplicate_bound_names<'a, T: BoundNames<'a>>(bound_names: &T, ctx: &SemanticBuilder<'_>) {
    let mut idents: FxHashMap<Str<'a>, Span> = FxHashMap::default();
    bound_names.bound_names(&mut |ident| {
        if let Some(old_span) = idents.insert(ident.name.into(), ident.span) {
            ctx.error(diagnostics::redeclaration(&ident.name, old_span, ident.span));
        }
    });
}

pub fn check_ts_external_module_declaration<'a>(
    decl: &TSExternalModuleDeclaration<'a>,
    ctx: &SemanticBuilder<'a>,
) {
    check_ambient_module_declaration(decl.span, ctx);
    if let Some(body) = &decl.body {
        check_ts_export_assignment_in_statements(&body.body, ctx);
    }
}

pub fn check_ts_namespace_declaration<'a>(
    decl: &TSNamespaceDeclaration<'a>,
    ctx: &SemanticBuilder<'a>,
) {
    check_ts_module_or_global_declaration(decl.span, ctx);
    check_ts_export_assignment_in_namespace_decl(decl, ctx);
}

pub fn check_ts_global_declaration<'a>(decl: &TSGlobalDeclaration<'a>, ctx: &SemanticBuilder<'a>) {
    check_ts_module_or_global_declaration(decl.span, ctx);

    if !decl.declare && !ctx.in_ambient_context() {
        ctx.error(diagnostics::global_scope_augmentation_should_have_declare_modifier(
            decl.global_span,
        ));
    }
}

fn check_ts_module_or_global_declaration(span: Span, ctx: &SemanticBuilder<'_>) {
    // skip current node
    for kind in ctx.ancestry().ancestor_kinds() {
        match kind {
            AstKind::Program(_)
            | AstKind::TSModuleBlock(_)
            | AstKind::TSExternalModuleDeclaration(_)
            | AstKind::TSNamespaceDeclaration(_)
            | AstKind::TSGlobalDeclaration(_) => {
                break;
            }
            m if m.is_module_declaration() => {
                // We need to check the parent of the parent
            }
            _ => {
                ctx.error(diagnostics::not_allowed_namespace_declaration(span));
                break;
            }
        }
    }
}

fn check_ambient_module_declaration(span: Span, ctx: &SemanticBuilder<'_>) {
    let mut ancestors =
        ctx.ancestry().ancestor_kinds().filter(|kind| !kind.is_module_declaration());

    match ancestors.next() {
        Some(AstKind::Program(_)) => {}
        Some(AstKind::TSModuleBlock(_)) => match ancestors.next() {
            Some(AstKind::TSExternalModuleDeclaration(_))
                if ctx.source_type.is_script()
                    && matches!(ancestors.next(), Some(AstKind::Program(_))) =>
            {
                // Module augmentations may be nested directly in a top-level ambient module.
            }
            Some(
                AstKind::TSExternalModuleDeclaration(_)
                | AstKind::TSNamespaceDeclaration(_)
                | AstKind::TSGlobalDeclaration(_),
            ) => {
                ctx.error(diagnostics::nested_ambient_module_declaration(span));
            }
            _ => ctx.error(diagnostics::not_allowed_ambient_module_declaration(span)),
        },
        Some(
            AstKind::TSExternalModuleDeclaration(_)
            | AstKind::TSNamespaceDeclaration(_)
            | AstKind::TSGlobalDeclaration(_),
        ) => {
            ctx.error(diagnostics::nested_ambient_module_declaration(span));
        }
        _ => ctx.error(diagnostics::not_allowed_ambient_module_declaration(span)),
    }
}

pub fn check_ts_enum_declaration<'a>(decl: &TSEnumDeclaration<'a>, ctx: &SemanticBuilder<'a>) {
    let mut need_initializer = false;
    // Enum member symbols store UTF-8 names, so the binder declares no symbol for a name with a
    // lone surrogate and cannot report its redeclaration.
    let mut lone_surrogate_names = FxHashMap::<JSStr<'a>, Span>::default();

    decl.body.members.iter().for_each(|member| {
        let name = match &member.id {
            TSEnumMemberName::Identifier(_) => None,
            TSEnumMemberName::String(lit) | TSEnumMemberName::ComputedString(lit) => {
                Some(lit.value)
            }
            TSEnumMemberName::ComputedTemplateString(template) => template.single_quasi(),
        };
        if let Some(name) = name.filter(|name| name.as_str().is_none()) {
            let span = member.id.span();
            match lone_surrogate_names.entry(name) {
                Entry::Occupied(first) => ctx.error(diagnostics::redeclaration(
                    &name.display().to_string(),
                    *first.get(),
                    span,
                )),
                Entry::Vacant(entry) => {
                    entry.insert(span);
                }
            }
        }

        #[expect(clippy::unnested_or_patterns)]
        if let Some(initializer) = &member.initializer {
            need_initializer = !matches!(
                initializer.without_parentheses(),
                // A = 1
                Expression::NumericLiteral(_)
                    // B = A
                    | Expression::Identifier(_)
                    // C = E.D
                    | match_member_expression!(Expression)
                    // D = 1 + 2
                    | Expression::BinaryExpression(_)
                    // E = -1
                    | Expression::UnaryExpression(_)
            );
        } else if need_initializer {
            ctx.error(diagnostics::enum_member_must_have_initializer(member.span));
        }
    });
}

pub fn check_class<'a>(class: &Class<'a>, ctx: &SemanticBuilder<'a>) {
    if !class.r#abstract {
        for elem in &class.body.body {
            if elem.is_abstract() {
                let span = elem.property_key().map_or_else(|| elem.span(), GetSpan::span);
                ctx.error(diagnostics::abstract_elem_in_concrete_class(elem.is_property(), span));
            }
        }
    }

    if !ctx.in_ambient_context() {
        let mut is_in_overload_group = false;
        for (a, b) in class.body.body.iter().map(Some).chain(std::iter::once(None)).tuple_windows()
        {
            if let Some(ClassElement::MethodDefinition(a)) = a
                && !a.r#type.is_abstract()
                && !a.optional
                && a.value.r#type == FunctionType::TSEmptyBodyFunctionExpression
            {
                let next_is_same = b.is_some_and(|b| {
                    matches!(b,
                        ClassElement::MethodDefinition(b)
                            if b.key.static_name() == a.key.static_name()
                    )
                });
                if next_is_same {
                    is_in_overload_group = true;
                } else if a.key.static_name().is_some() || is_in_overload_group {
                    // Report error for:
                    // 1. Methods with static names that are not followed by an implementation
                    // 2. The last overload in a computed-name overload group (e.g. [Symbol.iterator])
                    if a.kind.is_constructor() {
                        ctx.error(diagnostics::constructor_implementation_missing(a.key.span()));
                    } else {
                        ctx.error(diagnostics::function_implementation_missing(a.key.span()));
                    }
                    is_in_overload_group = false;
                } else {
                    is_in_overload_group = false;
                }
            } else {
                is_in_overload_group = false;
            }
        }
    }
}

pub fn check_method_definition<'a>(method: &MethodDefinition<'a>, ctx: &SemanticBuilder<'a>) {
    let is_abstract = method.r#type.is_abstract();

    let is_empty_body = method.value.r#type == FunctionType::TSEmptyBodyFunctionExpression;
    // Illegal to have `constructor(public foo);`
    if method.kind.is_constructor() && is_empty_body {
        for param in &method.value.params.items {
            if param.has_modifier() {
                ctx.error(diagnostics::parameter_property_only_in_constructor_impl(param.span));
            }
        }
    }

    // Illegal to have `get foo();` or `set foo(a)`
    if method.kind.is_accessor() && is_empty_body && !is_abstract && !ctx.in_ambient_context() {
        ctx.error(diagnostics::accessor_without_body(method.key.span()));
    }
}

pub fn check_for_statement_left(left: &ForStatementLeft, is_for_in: bool, ctx: &SemanticBuilder) {
    let ForStatementLeft::VariableDeclaration(decls) = left else {
        return;
    };

    for decl in &decls.declarations {
        if decl.type_annotation.is_some() {
            let span = decl.id.span();
            ctx.error(diagnostics::type_annotation_in_for_left(span, is_for_in));
        }
    }
}

pub fn check_ts_export_assignment_in_program<'a>(program: &Program<'a>, ctx: &SemanticBuilder<'a>) {
    if !ctx.source_type.is_typescript() {
        return;
    }
    check_ts_export_assignment_in_statements(&program.body, ctx);
}

fn check_ts_export_assignment_in_namespace_decl<'a>(
    namespace_decl: &TSNamespaceDeclaration<'a>,
    ctx: &SemanticBuilder<'a>,
) {
    match &namespace_decl.body {
        TSNamespaceDeclarationBody::TSNamespaceDeclaration(nested) => {
            check_ts_export_assignment_in_namespace_decl(nested, ctx);
        }
        TSNamespaceDeclarationBody::TSModuleBlock(block) => {
            check_ts_export_assignment_in_statements(&block.body, ctx);
        }
    }
}

fn check_ts_export_assignment_in_statements<'a>(
    statements: &[Statement<'a>],
    ctx: &SemanticBuilder<'a>,
) {
    let mut export_assignment_spans = vec![];
    let mut has_other_exports = false;

    for stmt in statements {
        match stmt {
            Statement::TSExportAssignment(export_assignment) => {
                export_assignment_spans.push(export_assignment.span);
            }
            Statement::ExportNamedDeclaration(export_decl) => {
                // ignore `export {}`
                has_other_exports |= !export_decl.specifiers.is_empty();
            }
            Statement::ExportFromDeclaration(export_decl) => {
                // Preserve the existing treatment of `export {} from "mod"`.
                has_other_exports |= !export_decl.specifiers.is_empty();
            }
            Statement::ExportDeclaration(_)
            | Statement::ExportDefaultDeclaration(_)
            | Statement::ExportAllDeclaration(_) => {
                has_other_exports = true;
            }
            _ => {}
        }
    }

    if has_other_exports {
        for span in export_assignment_spans {
            ctx.error(diagnostics::ts_export_assignment_cannot_be_used_with_other_exports(span));
        }
    }
}
