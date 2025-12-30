use std::borrow::Cow;

use oxc_diagnostics::OxcDiagnostic;
use oxc_span::Span;

#[cold]
fn ts_error<M: Into<Cow<'static, str>>>(code: &'static str, message: M) -> OxcDiagnostic {
    OxcDiagnostic::error(message).with_error_code("TS", code)
}

#[cold]
pub fn redeclaration(x0: &str, span1: Span, span2: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("Identifier `{x0}` has already been declared")).with_labels([
        span1.label(format!("`{x0}` has already been declared here")),
        span2.label("It can not be redeclared here"),
    ])
}

#[cold]
pub fn undefined_export(x0: &str, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("Export '{x0}' is not defined")).with_label(span1)
}

#[cold]
pub fn class_static_block_await(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Cannot use await in class static initialization block").with_label(span)
}

#[cold]
pub fn reserved_keyword(x0: &str, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("The keyword '{x0}' is reserved")).with_label(span1)
}

#[cold]
pub fn unexpected_identifier_assign(x0: &str, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("Cannot assign to '{x0}' in strict mode")).with_label(span1)
}

#[cold]
pub fn invalid_let_declaration(x0: &str, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!(
        "`let` cannot be declared as a variable name inside of a `{x0}` declaration"
    ))
    .with_label(span1)
}

#[cold]
pub fn unexpected_arguments(x0: &str, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("'arguments' is not allowed in {x0}"))
        .with_label(span1)
        .with_help("Assign the 'arguments' variable to a temporary variable outside")
}

#[cold]
pub fn private_not_in_class(x0: &str, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("Private identifier '#{x0}' is not allowed outside class bodies"))
        .with_label(span1)
}

#[cold]
pub fn private_field_undeclared(x0: &str, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("Private field '{x0}' must be declared in an enclosing class"))
        .with_label(span1)
}

#[cold]
pub fn legacy_octal(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("'0'-prefixed octal literals and octal escape sequences are deprecated")
        .with_help("for octal literals use the '0o' prefix instead")
        .with_label(span)
}

#[cold]
pub fn leading_zero_decimal(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Decimals with leading zeros are not allowed in strict mode")
        .with_help("remove the leading zero")
        .with_label(span)
}

#[cold]
pub fn non_octal_decimal_escape_sequence(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Invalid escape sequence")
        .with_help("\\8 and \\9 are not allowed in strict mode")
        .with_label(span)
}

#[cold]
pub fn illegal_use_strict(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(
        "Illegal 'use strict' directive in function with non-simple parameter list",
    )
    .with_label(span)
    .with_help(
        "Wrap this function with an IIFE with a 'use strict' directive that returns this function",
    )
}

#[cold]
pub fn top_level(x0: &str, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!(
        "'{x0}' declaration can only be used at the top level of a module"
    ))
    .with_label(span1)
}

#[cold]
pub fn module_code(x0: &str, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("Cannot use {x0} outside a module")).with_label(span1)
}

#[cold]
pub fn new_target(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Unexpected new.target expression")
        .with_help("new.target is only allowed in constructors and functions invoked using the `new` operator")
        .with_label(span)
}

#[cold]
pub fn import_meta(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Unexpected import.meta expression")
        .with_help("import.meta is only allowed in module code")
        .with_label(span)
}

#[cold]
pub fn function_declaration_strict(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Invalid function declaration")
        .with_help(
            "In strict mode code, functions can only be declared at top level or inside a block",
        )
        .with_label(span)
}

#[cold]
pub fn function_declaration_non_strict(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Invalid function declaration")
        .with_help("In non-strict mode code, functions can only be declared at top level, inside a block, or as the body of an if statement")
        .with_label(span)
}

#[cold]
pub fn with_statement(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("'with' statements are not allowed").with_label(span)
}

#[cold]
pub fn invalid_label_jump_target(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Jump target cannot cross function boundary.").with_label(span)
}

#[cold]
pub fn invalid_label_target(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Use of undefined label").with_label(span)
}

#[cold]
pub fn invalid_label_non_iteration(x0: &str, span1: Span, span2: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("A `{x0}` statement can only jump to a label of an enclosing `for`, `while` or `do while` statement."))
        .with_labels([
            span1.label("This is an non-iteration statement"),
            span2.label("for this label")
        ])
}

#[cold]
pub fn invalid_break(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Illegal break statement")
        .with_help("A `break` statement can only be used within an enclosing iteration or switch statement.")
        .with_label(span)
}

#[cold]
pub fn invalid_continue(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Illegal continue statement: no surrounding iteration statement")
        .with_help("A `continue` statement can only be used within an enclosing `for`, `while` or `do while` ")
        .with_label(span)
}

#[cold]
pub fn label_redeclaration(x0: &str, span1: Span, span2: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("Label `{x0}` has already been declared")).with_labels([
        span1.label(format!("`{x0}` has already been declared here")),
        span2.label("It can not be redeclared here"),
    ])
}

#[cold]
pub fn multiple_declaration_in_for_loop_head(x0: &str, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!(
        "Only a single declaration is allowed in a `for...{x0}` statement"
    ))
    .with_label(span1)
}

#[cold]
pub fn unexpected_initializer_in_for_loop_head(x0: &str, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("{x0} loop variable declaration may not have an initializer"))
        .with_label(span1)
}

#[cold]
pub fn duplicate_constructor(span: Span, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Multiple constructor implementations are not allowed.").with_labels([
        span.label("constructor has already been declared here"),
        span1.label("it cannot be redeclared here"),
    ])
}

#[cold]
pub fn require_class_name(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("A class name is required.").with_label(span)
}

#[cold]
pub fn super_without_derived_class(span: Span, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("'super' can only be referenced in a derived class.")
        .with_help("either remove this super, or extend the class")
        .with_labels([span.into(), span1.label("class does not have `extends`")])
}

#[cold]
pub fn unexpected_super_call(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Super calls are not permitted outside constructors or in nested functions inside constructors.")
        .with_label(span)
}

#[cold]
pub fn unexpected_super_reference(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("'super' can only be referenced in members of derived classes or object literal expressions.")
        .with_label(span)
}

#[cold]
pub fn assignment_is_not_simple(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Invalid left-hand side in assignment").with_label(span)
}

#[cold]
pub fn super_private(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Private fields cannot be accessed on super").with_label(span)
}

#[cold]
pub fn delete_of_unqualified(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Delete of an unqualified identifier in strict mode.").with_label(span)
}

#[cold]
pub fn delete_private_field(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("The operand of a 'delete' operator cannot be a private identifier.")
        .with_label(span)
}

#[cold]
pub fn await_or_yield_in_parameter(x0: &str, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("{x0} expression not allowed in formal parameter"))
        .with_label(span1.label(format!("{x0} expression not allowed in formal parameter")))
}

// TypeScript diagnostics

#[cold]
pub fn can_only_appear_on_a_type_parameter_of_a_class_interface_or_type_alias(
    modifier: &str,
    span: Span,
) -> OxcDiagnostic {
    ts_error("1274", format!("'{modifier}' modifier can only appear on a type parameter of a class, interface or type alias."))
        .with_label(span)
}

/// '?' at the end of a type is not valid TypeScript syntax. Did you mean to write 'number | null | undefined'?(17019)
#[cold]
pub fn jsdoc_type_in_annotation(
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

#[cold]
pub fn required_parameter_after_optional_parameter(span: Span) -> OxcDiagnostic {
    ts_error("1016", "A required parameter cannot follow an optional parameter.").with_label(span)
}

#[cold]
pub fn not_allowed_namespace_declaration(span: Span) -> OxcDiagnostic {
    ts_error(
        "1235",
        "A namespace declaration is only allowed at the top level of a namespace or module.",
    )
    .with_label(span)
}

#[cold]
pub fn global_scope_augmentation_should_have_declare_modifier(span: Span) -> OxcDiagnostic {
    ts_error(
        "2670",
        "Augmentations for the global scope should have 'declare' modifier unless they appear in already ambient context.",
    )
    .with_label(span)
}

#[cold]
pub fn enum_member_must_have_initializer(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Enum member must have initializer.").with_label(span)
}

/// TS(1392)
#[cold]
pub fn import_alias_cannot_use_import_type(span: Span) -> OxcDiagnostic {
    ts_error("1392", "An import alias cannot use 'import type'").with_label(span)
}

/// - Abstract properties can only appear within an abstract class. (1253)
/// - Abstract methods can only appear within an abstract class. (1244)
#[cold]
pub fn abstract_elem_in_concrete_class(is_property: bool, span: Span) -> OxcDiagnostic {
    let (code, elem_kind) = if is_property { ("1253", "properties") } else { ("1244", "methods") };
    ts_error(code, format!("Abstract {elem_kind} can only appear within an abstract class."))
        .with_label(span)
}

#[cold]
pub fn constructor_implementation_missing(span: Span) -> OxcDiagnostic {
    ts_error("2390", "Constructor implementation is missing.").with_label(span)
}

#[cold]
pub fn function_implementation_missing(span: Span) -> OxcDiagnostic {
    ts_error(
        "2391",
        "Function implementation is missing or not immediately following the declaration.",
    )
    .with_label(span)
}

#[cold]
pub fn reserved_type_name(span: Span, reserved_name: &str, syntax_name: &str) -> OxcDiagnostic {
    ts_error("2414", format!("{syntax_name} name cannot be '{reserved_name}'")).with_label(span)
}

/// 'abstract' modifier can only appear on a class, method, or property declaration. (1242)
#[cold]
pub fn illegal_abstract_modifier(span: Span) -> OxcDiagnostic {
    ts_error(
        "1242",
        "'abstract' modifier can only appear on a class, method, or property declaration.",
    )
    .with_label(span)
}

/// A parameter property is only allowed in a constructor implementation.ts(2369)
#[cold]
pub fn parameter_property_only_in_constructor_impl(span: Span) -> OxcDiagnostic {
    ts_error("2369", "A parameter property is only allowed in a constructor implementation.")
        .with_label(span)
}

/// Getter or setter without a body. There is no corresponding TS error code,
/// since in TSC this is a parse error.
#[cold]
pub fn accessor_without_body(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Getters and setters must have an implementation.").with_label(span)
}

/// The left-hand side of a 'for...of' statement cannot use a type annotation. (2483)
#[cold]
pub fn type_annotation_in_for_left(span: Span, is_for_in: bool) -> OxcDiagnostic {
    let for_of_or_in = if is_for_in { "for...in" } else { "for...of" };
    ts_error(
        "2483",
        format!(
            "The left-hand side of a '{for_of_or_in}' statement cannot use a type annotation.",
        ),
    ).with_label(span).with_help("This iterator's type will be inferred from the iterable. You can safely remove the type annotation.")
}

#[cold]
pub fn jsx_expressions_may_not_use_the_comma_operator(span: Span) -> OxcDiagnostic {
    ts_error("18007", "JSX expressions may not use the comma operator")
        .with_help("Did you mean to write an array?")
        .with_label(span)
}

#[cold]
pub fn ts_export_assignment_cannot_be_used_with_other_exports(span: Span) -> OxcDiagnostic {
    ts_error("2309", "An export assignment cannot be used in a module with other exported elements")
        .with_label(span)
        .with_help("If you want to use `export =`, remove other `export`s and put all of them to the right hand value of `export =`. If you want to use `export`s, remove `export =` statement.")
}
