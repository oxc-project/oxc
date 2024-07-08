use oxc_ast::syntax_directed_operations::BoundNames;
#[allow(clippy::wildcard_imports)]
use oxc_ast::{ast::*, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::{Atom, GetSpan, Span};
use rustc_hash::FxHashMap;

use crate::{builder::SemanticBuilder, diagnostics::redeclaration};

fn empty_type_parameter_list(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Type parameter list cannot be empty.").with_label(span0)
}

pub fn check_ts_type_parameter_declaration(
    declaration: &TSTypeParameterDeclaration<'_>,
    ctx: &SemanticBuilder<'_>,
) {
    if declaration.params.is_empty() {
        ctx.error(empty_type_parameter_list(declaration.span));
    }
}

fn unexpected_optional(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Unexpected `?` operator").with_label(span0)
}
#[allow(clippy::cast_possible_truncation)]
pub fn check_variable_declarator(decl: &VariableDeclarator, ctx: &SemanticBuilder<'_>) {
    if decl.id.optional {
        let start = decl.id.span().end;
        let Some(offset) = ctx.source_text[start as usize..].find('?') else { return };
        let offset = start + offset as u32;
        ctx.error(unexpected_optional(Span::new(offset, offset)));
    }
}

fn required_parameter_after_optional_parameter(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("A required parameter cannot follow an optional parameter.")
        .with_label(span0)
}

fn parameter_property_outside_constructor(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("A parameter property is only allowed in a constructor implementation.")
        .with_label(span0)
}

pub fn check_formal_parameters(params: &FormalParameters, ctx: &SemanticBuilder<'_>) {
    if !params.is_empty() && params.kind == FormalParameterKind::Signature {
        check_duplicate_bound_names(params, ctx);
    }

    let is_inside_constructor =
        !params.kind.is_signature() && ctx.current_scope_flags().is_constructor();
    let mut has_optional = false;

    for item in &params.items {
        // function a(optional?: number, required: number) { }
        if has_optional && !item.pattern.optional && !item.pattern.kind.is_assignment_pattern() {
            ctx.error(required_parameter_after_optional_parameter(item.span));
        }
        if item.pattern.optional {
            has_optional = true;
        }

        // function a(public x: number) { }
        if !is_inside_constructor && item.accessibility.is_some() {
            ctx.error(parameter_property_outside_constructor(item.span));
        }
    }
}

fn check_duplicate_bound_names<'a, T: BoundNames<'a>>(bound_names: &T, ctx: &SemanticBuilder<'_>) {
    let mut idents: FxHashMap<Atom<'a>, Span> = FxHashMap::default();
    bound_names.bound_names(&mut |ident| {
        if let Some(old_span) = idents.insert(ident.name.clone(), ident.span) {
            ctx.error(redeclaration(&ident.name, old_span, ident.span));
        }
    });
}

fn unexpected_assignment(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(
        "The left-hand side of an assignment expression must be a variable or a property access.",
    )
    .with_label(span0)
}

pub fn check_simple_assignment_target<'a>(
    target: &SimpleAssignmentTarget<'a>,
    ctx: &SemanticBuilder<'a>,
) {
    if let Some(expression) = target.get_expression() {
        #[allow(clippy::match_same_arms)]
        match expression.get_inner_expression() {
            Expression::Identifier(_) => {}
            match_member_expression!(Expression) => {}
            _ => {
                ctx.error(unexpected_assignment(target.span()));
            }
        }
    }
}

fn unexpected_type_annotation(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Unexpected type annotation").with_label(span0)
}

pub fn check_array_pattern<'a>(pattern: &ArrayPattern<'a>, ctx: &SemanticBuilder<'a>) {
    for element in &pattern.elements {
        let _ = element.as_ref().map(|element| {
            if let Some(type_annotation) = &element.type_annotation {
                ctx.error(unexpected_type_annotation(type_annotation.span));
            }
        });
    }
}

fn not_allowed_namespace_declaration(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(
        "A namespace declaration is only allowed at the top level of a namespace or module.",
    )
    .with_label(span0)
}

pub fn check_ts_module_declaration<'a>(decl: &TSModuleDeclaration<'a>, ctx: &SemanticBuilder<'a>) {
    // skip current node
    for node in ctx.nodes.iter_parents(ctx.current_node_id).skip(1) {
        match node.kind() {
            AstKind::Program(_) | AstKind::TSModuleBlock(_) | AstKind::TSModuleDeclaration(_) => {
                break;
            }
            AstKind::ExportNamedDeclaration(_) | AstKind::ModuleDeclaration(_) => {
                // export namespace N {}
                // We need to check the parent of the parent
                continue;
            }
            _ => {
                ctx.error(not_allowed_namespace_declaration(decl.span));
            }
        }
    }
}

fn enum_member_must_have_initializer(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Enum member must have initializer.").with_label(span0)
}

pub fn check_ts_enum_declaration<'a>(decl: &TSEnumDeclaration<'a>, ctx: &SemanticBuilder<'a>) {
    let mut need_initializer = false;

    decl.members.iter().for_each(|member| {
        #[allow(clippy::unnested_or_patterns)]
        if let Some(initializer) = &member.initializer {
            need_initializer = !matches!(
                initializer,
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
            ctx.error(enum_member_must_have_initializer(member.span));
        }
    });
}

/// TS(1392)
fn import_alias_cannot_use_import_type(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("TS(1392): An import alias cannot use 'import type'").with_label(span)
}

pub fn check_ts_import_equals_declaration<'a>(
    decl: &TSImportEqualsDeclaration<'a>,
    ctx: &SemanticBuilder<'a>,
) {
    // `import type Foo = require('./foo')` is allowed
    // `import { Foo } from './foo'; import type Bar = Foo.Bar` is not allowed
    if decl.import_kind.is_type() && !decl.module_reference.is_external() {
        ctx.error(import_alias_cannot_use_import_type(decl.span));
    }
}
