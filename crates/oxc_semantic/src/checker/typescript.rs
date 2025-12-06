use std::borrow::Cow;

use itertools::Itertools;
use rustc_hash::FxHashMap;

use oxc_ast::{AstKind, ast::*};
use oxc_diagnostics::OxcDiagnostic;
use oxc_ecmascript::BoundNames;
use oxc_span::{Atom, GetSpan, Span};

use crate::{builder::SemanticBuilder, diagnostics::redeclaration};

fn ts_error<M: Into<Cow<'static, str>>>(code: &'static str, message: M) -> OxcDiagnostic {
    OxcDiagnostic::error(message).with_error_code("TS", code)
}

fn can_only_appear_on_a_type_parameter_of_a_class_interface_or_type_alias(
    modifier: &str,
    span: Span,
) -> OxcDiagnostic {
    ts_error("1274", format!("'{modifier}' modifier can only appear on a type parameter of a class, interface or type alias."))
        .with_label(span)
}

pub fn check_ts_type_parameter<'a>(param: &TSTypeParameter<'a>, ctx: &SemanticBuilder<'a>) {
    check_type_name_is_reserved(&param.name, ctx, "Type parameter");
    if param.r#in || param.out {
        let is_allowed_node = matches!(
            // skip parent TSTypeParameterDeclaration
            ctx.nodes.ancestor_kinds(ctx.current_node_id).nth(1),
            Some(
                AstKind::TSInterfaceDeclaration(_)
                    | AstKind::Class(_)
                    | AstKind::TSTypeAliasDeclaration(_)
            )
        );
        if !is_allowed_node {
            if param.r#in {
                ctx.error(can_only_appear_on_a_type_parameter_of_a_class_interface_or_type_alias(
                    "in", param.span,
                ));
            }
            if param.out {
                ctx.error(can_only_appear_on_a_type_parameter_of_a_class_interface_or_type_alias(
                    "out", param.span,
                ));
            }
        }
    }
}

/// '?' at the end of a type is not valid TypeScript syntax. Did you mean to write 'number | null | undefined'?(17019)
fn jsdoc_type_in_annotation(
    modifier: char,
    is_start: bool,
    span: Span,
    suggested_type: &str,
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

    let suggestion = &ctx.source_text[valid_type_span];
    let suggestion = if modifier == '?' {
        Cow::Owned(format!("{suggestion} | null | undefined"))
    } else {
        Cow::Borrowed(suggestion)
    };

    ctx.error(jsdoc_type_in_annotation(
        modifier,
        is_start,
        span_with_illegal_modifier,
        &suggestion,
    ));
}

pub fn check_ts_type_alias_declaration<'a>(
    decl: &TSTypeAliasDeclaration<'a>,
    ctx: &SemanticBuilder<'a>,
) {
    check_type_name_is_reserved(&decl.id, ctx, "Type alias");
}

fn required_parameter_after_optional_parameter(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("A required parameter cannot follow an optional parameter.")
        .with_label(span)
}

pub fn check_formal_parameters(params: &FormalParameters, ctx: &SemanticBuilder<'_>) {
    if params.kind == FormalParameterKind::Signature && params.items.len() > 1 {
        check_duplicate_bound_names(params, ctx);
    }

    let mut has_optional = false;

    for param in &params.items {
        // function a(optional?: number, required: number) { }
        if param.pattern.optional {
            has_optional = true;
        } else if has_optional && !param.pattern.kind.is_assignment_pattern() {
            ctx.error(required_parameter_after_optional_parameter(param.span));
        }
    }
}

fn check_duplicate_bound_names<'a, T: BoundNames<'a>>(bound_names: &T, ctx: &SemanticBuilder<'_>) {
    let mut idents: FxHashMap<Atom<'a>, Span> = FxHashMap::default();
    bound_names.bound_names(&mut |ident| {
        if let Some(old_span) = idents.insert(ident.name, ident.span) {
            ctx.error(redeclaration(&ident.name, old_span, ident.span));
        }
    });
}

fn unexpected_type_annotation(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Unexpected type annotation").with_label(span)
}

pub fn check_array_pattern<'a>(pattern: &ArrayPattern<'a>, ctx: &SemanticBuilder<'a>) {
    for element in &pattern.elements {
        if let Some(element) = element.as_ref()
            && let Some(type_annotation) = &element.type_annotation
        {
            ctx.error(unexpected_type_annotation(type_annotation.span));
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
    check_ts_module_or_global_declaration(decl.span, ctx);
    check_ts_export_assignment_in_module_decl(decl, ctx);
}

pub fn check_ts_global_declaration<'a>(decl: &TSGlobalDeclaration<'a>, ctx: &SemanticBuilder<'a>) {
    check_ts_module_or_global_declaration(decl.span, ctx);
}

fn check_ts_module_or_global_declaration(span: Span, ctx: &SemanticBuilder<'_>) {
    // skip current node
    for node in ctx.nodes.ancestors(ctx.current_node_id) {
        match node.kind() {
            AstKind::Program(_)
            | AstKind::TSModuleBlock(_)
            | AstKind::TSModuleDeclaration(_)
            | AstKind::TSGlobalDeclaration(_) => {
                break;
            }
            m if m.is_module_declaration() => {
                // We need to check the parent of the parent
            }
            _ => {
                ctx.error(not_allowed_namespace_declaration(span));
            }
        }
    }
}

fn enum_member_must_have_initializer(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Enum member must have initializer.").with_label(span)
}

pub fn check_ts_enum_declaration<'a>(decl: &TSEnumDeclaration<'a>, ctx: &SemanticBuilder<'a>) {
    let mut need_initializer = false;

    decl.body.members.iter().for_each(|member| {
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
            ctx.error(enum_member_must_have_initializer(member.span));
        }
    });

    check_type_name_is_reserved(&decl.id, ctx, "Enum");
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

fn constructor_implementation_missing(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Constructor implementation is missing.").with_label(span)
}

fn function_implementation_missing(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(
        "Function implementation is missing or not immediately following the declaration.",
    )
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

    if !class.r#declare && !ctx.in_declare_scope() {
        for (a, b) in class.body.body.iter().map(Some).chain(vec![None]).tuple_windows() {
            if let Some(ClassElement::MethodDefinition(a)) = a
                && !a.r#type.is_abstract()
                && !a.optional
                && a.value.r#type == FunctionType::TSEmptyBodyFunctionExpression
                && b.is_none_or(|b| match b {
                    ClassElement::StaticBlock(_)
                    | ClassElement::PropertyDefinition(_)
                    | ClassElement::AccessorProperty(_)
                    | ClassElement::TSIndexSignature(_) => true,
                    ClassElement::MethodDefinition(b) => b.key.static_name() != a.key.static_name(),
                })
            {
                if a.kind.is_constructor() {
                    ctx.error(constructor_implementation_missing(a.key.span()));
                } else {
                    ctx.error(function_implementation_missing(a.key.span()));
                }
            }
        }
    }
    if let Some(id) = &class.id {
        check_type_name_is_reserved(id, ctx, "Class");
    }
}

pub fn check_ts_interface_declaration<'a>(
    decl: &TSInterfaceDeclaration<'a>,
    ctx: &SemanticBuilder<'a>,
) {
    check_type_name_is_reserved(&decl.id, ctx, "Interface");
}

/// ```ts
/// function checkTypeNameIsReserved(name: Identifier, message: DiagnosticMessage): void {
///     // TS 1.0 spec (April 2014): 3.6.1
///     // The predefined type keywords are reserved and cannot be used as names of user defined types.
///     switch (name.escapedText) {
///         case "any":
///         case "unknown":
///         case "never":
///         case "number":
///         case "bigint":
///         case "boolean":
///         case "string":
///         case "symbol":
///         case "void":
///         case "object":
///         case "undefined":
///             error(name, message, name.escapedText as string);
///     }
/// }
/// ```
fn check_type_name_is_reserved<'a>(
    id: &BindingIdentifier<'a>,
    ctx: &SemanticBuilder<'a>,
    syntax_name: &str,
) {
    match id.name.as_str() {
        "any" | "unknown" | "never" | "number" | "bigint" | "boolean" | "string" | "symbol"
        | "void" | "object" | "undefined" => {
            ctx.error(reserved_type_name(id.span, id.name.as_str(), syntax_name));
        }
        _ => {}
    }
}

fn reserved_type_name(span: Span, reserved_name: &str, syntax_name: &str) -> OxcDiagnostic {
    ts_error("2414", format!("{syntax_name} name cannot be '{reserved_name}'")).with_label(span)
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
        }
    }

    let is_empty_body = method.value.r#type == FunctionType::TSEmptyBodyFunctionExpression;
    // Illegal to have `constructor(public foo);`
    if method.kind.is_constructor() && is_empty_body {
        for param in &method.value.params.items {
            if param.has_modifier() {
                ctx.error(parameter_property_only_in_constructor_impl(param.span));
            }
        }
    }

    // Illegal to have `get foo();` or `set foo(a)`
    if method.kind.is_accessor() && is_empty_body && !is_abstract && !is_declare {
        ctx.error(accessor_without_body(method.key.span()));
    }
}

pub fn check_object_property(prop: &ObjectProperty, ctx: &SemanticBuilder<'_>) {
    if let Expression::FunctionExpression(func) = &prop.value
        && prop.kind.is_accessor()
        && matches!(func.r#type, FunctionType::TSEmptyBodyFunctionExpression)
    {
        ctx.error(accessor_without_body(prop.key.span()));
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

fn jsx_expressions_may_not_use_the_comma_operator(span: Span) -> OxcDiagnostic {
    ts_error("18007", "JSX expressions may not use the comma operator")
        .with_help("Did you mean to write an array?")
        .with_label(span)
}

pub fn check_jsx_expression_container(
    container: &JSXExpressionContainer,
    ctx: &SemanticBuilder<'_>,
) {
    if matches!(container.expression, JSXExpression::SequenceExpression(_)) {
        ctx.error(jsx_expressions_may_not_use_the_comma_operator(container.expression.span()));
    }
}

fn ts_export_assignment_cannot_be_used_with_other_exports(span: Span) -> OxcDiagnostic {
    ts_error("2309", "An export assignment cannot be used in a module with other exported elements")
        .with_label(span)
        .with_help("If you want to use `export =`, remove other `export`s and put all of them to the right hand value of `export =`. If you want to use `export`s, remove `export =` statement.")
}

pub fn check_ts_export_assignment_in_program<'a>(program: &Program<'a>, ctx: &SemanticBuilder<'a>) {
    if !ctx.source_type.is_typescript() {
        return;
    }
    check_ts_export_assignment_in_statements(&program.body, ctx);
}

fn check_ts_export_assignment_in_module_decl<'a>(
    module_decl: &TSModuleDeclaration<'a>,
    ctx: &SemanticBuilder<'a>,
) {
    let Some(body) = &module_decl.body else {
        return;
    };
    match body {
        TSModuleDeclarationBody::TSModuleDeclaration(nested) => {
            check_ts_export_assignment_in_module_decl(nested, ctx);
        }
        TSModuleDeclarationBody::TSModuleBlock(block) => {
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
                if export_decl.declaration.is_none() && export_decl.specifiers.is_empty() {
                    continue;
                }
                has_other_exports = true;
            }
            Statement::ExportDefaultDeclaration(_) | Statement::ExportAllDeclaration(_) => {
                has_other_exports = true;
            }
            _ => {}
        }
    }

    if has_other_exports {
        for span in export_assignment_spans {
            ctx.error(ts_export_assignment_cannot_be_used_with_other_exports(span));
        }
    }
}
