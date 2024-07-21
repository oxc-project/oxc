use std::borrow::Cow;

use oxc_ast::syntax_directed_operations::{BoundNames, PropName};
#[allow(clippy::wildcard_imports)]
use oxc_ast::{ast::*, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::{Atom, GetSpan, Span};
use rustc_hash::FxHashMap;

use crate::{builder::SemanticBuilder, diagnostics::redeclaration};

fn ts_error<M: Into<Cow<'static, str>>>(code: &'static str, message: M) -> OxcDiagnostic {
    OxcDiagnostic::error(message).with_error_code("TS", code)
}

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
    ts_error("1392", "An import alias cannot use 'import type'").with_label(span)
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

/// - Abstract properties can only appear within an abstract class. (1253)
/// - Abstract methods can only appear within an abstract class. (1244)
fn abstract_elem_in_concrete_class(is_property: bool, span: Span) -> OxcDiagnostic {
    let (code, elem_kind) = if is_property { ("1253", "properties") } else { ("1244", "methods") };
    ts_error(code, format!("Abstract {elem_kind} can only appear within an abstract class."))
        .with_label(span)
}

pub fn check_class<'a>(class: &Class<'a>, ctx: &SemanticBuilder<'a>) {
    if !class.r#abstract {
        for elem in &class.body.body {
            if elem.is_abstract() {
                let span = elem.property_key().map_or_else(|| elem.span(), GetSpan::span);
                ctx.error(abstract_elem_in_concrete_class(elem.is_property(), span));
            }
        }
    }
}

fn abstract_element_cannot_have_initializer(
    code: &'static str,
    elem_name: &str,
    prop_name: &str,
    span: Span,
    init_or_impl: &str,
) -> OxcDiagnostic {
    ts_error(code, format!("{elem_name} '{prop_name}' cannot have an {init_or_impl} because it is marked abstract."))
    .with_label(span)
}

/// TS(1245): Method 'foo' cannot have an implementation because it is marked abstract.
fn abstract_method_cannot_have_implementation(method_name: &str, span: Span) -> OxcDiagnostic {
    abstract_element_cannot_have_initializer("1245", "Method", method_name, span, "implementation")
}

/// TS(1267): Property 'foo' cannot have an initializer because it is marked abstract.
fn abstract_property_cannot_have_initializer(prop_name: &str, span: Span) -> OxcDiagnostic {
    abstract_element_cannot_have_initializer("1267", "Property", prop_name, span, "initializer")
}

/// TS(1318): Accessor 'foo' cannot have an implementation because it is marked abstract.
///
/// Applies to getters/setters
///
/// > TS's original message, `An abstract accessor cannot have an
/// > implementation.`, is less helpful than the one provided here.
fn abstract_accessor_cannot_have_implementation(accessor_name: &str, span: Span) -> OxcDiagnostic {
    abstract_element_cannot_have_initializer(
        "1318",
        "Accessor",
        accessor_name,
        span,
        "implementation",
    )
}

/// 'abstract' modifier can only appear on a class, method, or property declaration. (1242)
fn illegal_abstract_modifier(span: Span) -> OxcDiagnostic {
    ts_error(
        "1242",
        "'abstract' modifier can only appear on a class, method, or property declaration.",
    )
    .with_label(span)
}

pub fn check_method_definition<'a>(method: &MethodDefinition<'a>, ctx: &SemanticBuilder<'a>) {
    if method.r#type.is_abstract() {
        // constructors cannot be abstract, no matter what
        if method.kind.is_constructor() {
            ctx.error(illegal_abstract_modifier(method.key.span()));
        } else if method.value.body.is_some() {
            // abstract class elements cannot have bodies or initializers
            let (method_name, span) = method.key.prop_name().unwrap_or_else(|| {
                let key_span = method.key.span();
                (&ctx.source_text[key_span], key_span)
            });
            match method.kind {
                MethodDefinitionKind::Method => {
                    ctx.error(abstract_method_cannot_have_implementation(method_name, span));
                }
                MethodDefinitionKind::Get | MethodDefinitionKind::Set => {
                    ctx.error(abstract_accessor_cannot_have_implementation(method_name, span));
                }
                // abstract classes can have concrete methods. Constructors cannot
                // have abstract modifiers, but this gets checked during parsing
                MethodDefinitionKind::Constructor => {}
            }
            ctx.error(abstract_method_cannot_have_implementation(method_name, span));
        }
    }
}

pub fn check_property_definition<'a>(prop: &PropertyDefinition<'a>, ctx: &SemanticBuilder<'a>) {
    if prop.r#type.is_abstract() && prop.value.is_some() {
        let (prop_name, span) = prop.key.prop_name().unwrap_or_else(|| {
            let key_span = prop.key.span();
            (&ctx.source_text[key_span], key_span)
        });
        ctx.error(abstract_property_cannot_have_initializer(prop_name, span));
    }
}
