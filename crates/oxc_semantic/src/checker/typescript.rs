use std::borrow::Cow;

use rustc_hash::FxHashMap;

#[allow(clippy::wildcard_imports)]
use oxc_ast::{ast::*, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::{Atom, GetSpan, Span};
use oxc_syntax_operations::{BoundNames, PropName};

use crate::{builder::SemanticBuilder, diagnostics::redeclaration};

fn ts_error<M: Into<Cow<'static, str>>>(code: &'static str, message: M) -> OxcDiagnostic {
    OxcDiagnostic::error(message).with_error_code("TS", code)
}

fn empty_type_parameter_list(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Type parameter list cannot be empty.").with_label(span)
}

pub fn check_ts_type_parameter_declaration(
    declaration: &TSTypeParameterDeclaration<'_>,
    ctx: &SemanticBuilder<'_>,
) {
    if declaration.params.is_empty() {
        ctx.error(empty_type_parameter_list(declaration.span));
    }
}
/// '?' at the end of a type is not valid TypeScript syntax. Did you mean to write 'number | null | undefined'?(17019)
#[allow(clippy::needless_pass_by_value)]
fn jsdoc_type_in_annotation(
    modifier: char,
    is_start: bool,
    span: Span,
    suggested_type: Cow<str>,
) -> OxcDiagnostic {
    let (code, start_or_end) = if is_start { ("17020", "start") } else { ("17019", "end") };

    ts_error(
        code,
        format!("'{modifier}' at the {start_or_end} of a type is not valid TypeScript syntax.",),
    )
    .with_label(span)
    .with_help(format!("Did you mean to write '{suggested_type}'?"))
}

pub fn check_ts_type_annotation(annotation: &TSTypeAnnotation<'_>, ctx: &SemanticBuilder<'_>) {
    let (modifier, is_start, span_with_illegal_modifier) = match &annotation.type_annotation {
        TSType::JSDocNonNullableType(ty) => ('!', !ty.postfix, ty.span()),
        TSType::JSDocNullableType(ty) => ('?', !ty.postfix, ty.span()),
        _ => {
            return;
        }
    };

    let valid_type_span = if is_start {
        span_with_illegal_modifier.shrink_left(1)
    } else {
        span_with_illegal_modifier.shrink_right(1)
    };

    let suggestion = if modifier == '?' {
        Cow::Owned(format!("{} | null | undefined", &ctx.source_text[valid_type_span]))
    } else {
        Cow::Borrowed(&ctx.source_text[valid_type_span])
    };

    ctx.error(jsdoc_type_in_annotation(modifier, is_start, span_with_illegal_modifier, suggestion));
}

/// Initializers are not allowed in ambient contexts. ts(1039)
fn initializer_in_ambient_context(init_span: Span) -> OxcDiagnostic {
    ts_error("1039", "Initializers are not allowed in ambient contexts.").with_label(init_span)
}

pub fn check_variable_declaration(decl: &VariableDeclaration, ctx: &SemanticBuilder<'_>) {
    if decl.declare {
        for var in &decl.declarations {
            if let Some(init) = &var.init {
                ctx.error(initializer_in_ambient_context(init.span()));
            }
        }
    }
}

fn unexpected_optional(span: Span, type_annotation: Option<&str>) -> OxcDiagnostic {
    let d = OxcDiagnostic::error("Unexpected `?` operator").with_label(span);
    if let Some(ty) = type_annotation {
        d.with_help(format!("If you want an optional type, use `{ty} | undefined` instead."))
    } else {
        d
    }
}

#[expect(clippy::cast_possible_truncation)]
fn find_char(span: Span, source_text: &str, c: char) -> Option<Span> {
    let Some(offset) = span.source_text(source_text).find(c) else {
        debug_assert!(
            false,
            "Flag {c} not found in source text. This is likely indicates a bug in the parser.",
        );
        return None;
    };
    let offset = span.start + offset as u32;
    Some(Span::new(offset, offset))
}

pub fn check_variable_declarator(decl: &VariableDeclarator, ctx: &SemanticBuilder<'_>) {
    // Check for `let x?: number;`
    if decl.id.optional {
        // NOTE: BindingPattern spans cover the identifier _and_ the type annotation.
        let ty = decl
            .id
            .type_annotation
            .as_ref()
            .map(|ty| ty.type_annotation.span())
            .map(|span| &ctx.source_text[span]);
        if let Some(span) = find_char(decl.span, ctx.source_text, '?') {
            ctx.error(unexpected_optional(span, ty));
        }
    }
    if decl.definite {
        // Check for `let x!: number = 1;`
        //                 ^
        let Some(span) = find_char(decl.span, ctx.source_text, '!') else { return };
        if decl.init.is_some() {
            let error = ts_error(
                "1263",
                "Declarations with initializers cannot also have definite assignment assertions.",
            )
            .with_label(span);
            ctx.error(error);
        } else if decl.id.type_annotation.is_none() {
            let error = ts_error(
                "1264",
                "Declarations with definite assignment assertions must also have type annotations.",
            )
            .with_label(span);
            ctx.error(error);
        }
    }
}

fn required_parameter_after_optional_parameter(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("A required parameter cannot follow an optional parameter.")
        .with_label(span)
}

fn parameter_property_outside_constructor(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("A parameter property is only allowed in a constructor implementation.")
        .with_label(span)
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

fn unexpected_assignment(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(
        "The left-hand side of an assignment expression must be a variable or a property access.",
    )
    .with_label(span)
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

fn unexpected_type_annotation(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Unexpected type annotation").with_label(span)
}

pub fn check_array_pattern<'a>(pattern: &ArrayPattern<'a>, ctx: &SemanticBuilder<'a>) {
    for element in &pattern.elements {
        if let Some(element) = element.as_ref() {
            if let Some(type_annotation) = &element.type_annotation {
                ctx.error(unexpected_type_annotation(type_annotation.span));
            }
        }
    }
}

/// An interface can only extend an identifier/qualified-name with optional type arguments.(2499)
fn invalid_interface_extend(span: Span) -> OxcDiagnostic {
    ts_error(
        "2499",
        "An interface can only extend an identifier/qualified-name with optional type arguments.",
    )
    .with_label(span)
}

pub fn check_ts_interface_declaration<'a>(
    decl: &TSInterfaceDeclaration<'a>,
    ctx: &SemanticBuilder<'a>,
) {
    if let Some(extends) = &decl.extends {
        for extend in extends {
            if !matches!(
                &extend.expression,
                Expression::Identifier(_) | Expression::StaticMemberExpression(_),
            ) {
                ctx.error(invalid_interface_extend(extend.span));
            }
        }
    }
}

fn not_allowed_namespace_declaration(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(
        "A namespace declaration is only allowed at the top level of a namespace or module.",
    )
    .with_label(span)
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

fn enum_member_must_have_initializer(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Enum member must have initializer.").with_label(span)
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
    ts_error(
        code,
        format!(
            "{elem_name} '{prop_name}' cannot have an {init_or_impl} because it is marked abstract."
        ),
    )
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

/// A parameter property is only allowed in a constructor implementation.ts(2369)
fn parameter_property_only_in_constructor_impl(span: Span) -> OxcDiagnostic {
    ts_error("2369", "A parameter property is only allowed in a constructor implementation.")
        .with_label(span)
}

/// Getter or setter without a body. There is no corresponding TS error code,
/// since in TSC this is a parse error.
fn accessor_without_body(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Getters and setters must have an implementation.").with_label(span)
}

pub fn check_method_definition<'a>(method: &MethodDefinition<'a>, ctx: &SemanticBuilder<'a>) {
    let is_abstract = method.r#type.is_abstract();
    let is_declare = ctx.class_table_builder.current_class_id.map_or(
        ctx.source_type.is_typescript_definition(),
        |id| {
            let node_id = ctx.class_table_builder.classes.declarations[id];
            let AstKind::Class(class) = ctx.nodes.get_node(node_id).kind() else {
                #[cfg(debug_assertions)]
                panic!("current_class_id is set, but does not point to a Class node.");
                #[cfg(not(debug_assertions))]
                return ctx.source_type.is_typescript_definition();
            };
            class.declare || ctx.source_type.is_typescript_definition()
        },
    );

    if is_abstract {
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

    let is_empty_body = method.value.r#type == FunctionType::TSEmptyBodyFunctionExpression;
    // Illegal to have `constructor(public foo);`
    if method.kind.is_constructor() && is_empty_body {
        for param in &method.value.params.items {
            if param.accessibility.is_some() {
                ctx.error(parameter_property_only_in_constructor_impl(param.span));
            }
        }
    }

    // Illegal to have `get foo();` or `set foo(a)`
    if method.kind.is_accessor() && is_empty_body && !is_abstract && !is_declare {
        ctx.error(accessor_without_body(method.key.span()));
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

pub fn check_object_property(prop: &ObjectProperty, ctx: &SemanticBuilder<'_>) {
    if let Expression::FunctionExpression(func) = &prop.value {
        if prop.kind.is_accessor()
            && matches!(func.r#type, FunctionType::TSEmptyBodyFunctionExpression)
        {
            ctx.error(accessor_without_body(prop.key.span()));
        }
    }
}

/// The left-hand side of a 'for...of' statement cannot use a type annotation. (2483)
fn type_annotation_in_for_left(span: Span, is_for_in: bool) -> OxcDiagnostic {
    let for_of_or_in = if is_for_in { "for...in" } else { "for...of" };
    ts_error(
        "2483",
        format!(
            "The left-hand side of a '{for_of_or_in}' statement cannot use a type annotation.",
        ),
    ).with_label(span).with_help("This iterator's type will be inferred from the iterable. You can safely remove the type annotation.")
}

pub fn check_for_statement_left(left: &ForStatementLeft, is_for_in: bool, ctx: &SemanticBuilder) {
    let ForStatementLeft::VariableDeclaration(decls) = left else {
        return;
    };

    for decl in &decls.declarations {
        if decl.id.type_annotation.is_some() {
            let span = decl.id.span();
            ctx.error(type_annotation_in_for_left(span, is_for_in));
        }
    }
}
