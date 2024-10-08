use std::borrow::Cow;

use oxc_diagnostics::OxcDiagnostic;
use oxc_span::Span;

use crate::modifiers::Modifier;

#[inline]
fn ts_error<C, M>(code: C, message: M) -> OxcDiagnostic
where
    C: Into<Cow<'static, str>>,
    M: Into<Cow<'static, str>>,
{
    OxcDiagnostic::error(message).with_error_code("TS", code)
}

#[cold]
pub fn redeclaration(x0: &str, declare_span: Span, redeclare_span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("Identifier `{x0}` has already been declared")).with_labels([
        declare_span.label(format!("`{x0}` has already been declared here")),
        redeclare_span.label("It can not be redeclared here"),
    ])
}

#[cold]
pub fn overlong_source() -> OxcDiagnostic {
    OxcDiagnostic::error("Source length exceeds 4 GiB limit")
}

#[cold]
pub fn flow(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Flow is not supported").with_label(span)
}

#[cold]
pub fn unexpected_token(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Unexpected token").with_label(span)
}

#[cold]
pub fn expect_token(x0: &str, x1: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("Expected `{x0}` but found `{x1}`"))
        .with_label(span.label(format!("`{x0}` expected")))
}

#[cold]
pub fn invalid_escape_sequence(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Invalid escape sequence").with_label(span)
}

#[cold]
pub fn unicode_escape_sequence(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Invalid Unicode escape sequence").with_label(span)
}

#[cold]
pub fn invalid_character(x0: char, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("Invalid Character `{x0}`")).with_label(span1)
}

#[cold]
pub fn invalid_number_end(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Invalid characters after number").with_label(span)
}

#[cold]
pub fn unterminated_multi_line_comment(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Unterminated multiline comment").with_label(span)
}

#[cold]
pub fn unterminated_string(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Unterminated string").with_label(span)
}

#[cold]
pub fn reg_exp_flag(x0: char, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("Unexpected flag {x0} in regular expression literal"))
        .with_label(span1)
}

#[cold]
pub fn reg_exp_flag_twice(x0: char, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("Flag {x0} is mentioned twice in regular expression literal"))
        .with_label(span1)
}

#[cold]
pub fn unexpected_end(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Unexpected end of file").with_label(span)
}

#[cold]
pub fn unterminated_reg_exp(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Unterminated regular expression").with_label(span)
}

#[cold]
pub fn invalid_number(x0: &str, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("Invalid Number {x0}")).with_label(span1)
}

#[cold]
pub fn escaped_keyword(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Keywords cannot contain escape characters").with_label(span)
}

#[cold]
pub fn auto_semicolon_insertion(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(
        "Expected a semicolon or an implicit semicolon after a statement, but found none",
    )
    .with_help("Try insert a semicolon here")
    .with_label(span)
}

#[cold]
pub fn lineterminator_before_arrow(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Line terminator not permitted before arrow").with_label(span)
}

#[cold]
pub fn invalid_destrucuring_declaration(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Missing initializer in destructuring declaration").with_label(span)
}

#[cold]
pub fn missinginitializer_in_const(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Missing initializer in const declaration").with_label(span)
}

#[cold]
pub fn lexical_declaration_single_statement(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Lexical declaration cannot appear in a single-statement context")
        .with_help("Wrap this declaration in a block statement")
        .with_label(span)
}

#[cold]
pub fn async_function_declaration(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Async functions can only be declared at the top level or inside a block")
        .with_label(span)
}

#[cold]
pub fn generator_function_declaration(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Generators can only be declared at the top level or inside a block")
        .with_label(span)
}

#[cold]
pub fn await_expression(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(
        "`await` is only allowed within async functions and at the top levels of modules",
    )
    .with_label(span)
}

#[cold]
pub fn yield_expression(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("A 'yield' expression is only allowed in a generator body.")
        .with_label(span)
}

#[cold]
pub fn class_declaration(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Invalid class declaration")
        .with_help("Classes can only be declared at top level or inside a block")
        .with_label(span)
}

/// A class member cannot have the 'const' keyword. ts(1248)
#[cold]
pub fn const_class_member(span: Span) -> OxcDiagnostic {
    ts_error("1248", "A class member cannot have the 'const' keyword.")
        .with_help("Did you mean `readonly`?")
        .with_label(span)
}

#[cold]
pub fn binding_rest_element_last(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("A rest element must be last in a destructuring pattern").with_label(span)
}

#[cold]
pub fn rest_parameter_last(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("A rest parameter must be last in a parameter list").with_label(span)
}

#[cold]
pub fn spread_last_element(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Spread must be last element").with_label(span)
}

#[cold]
pub fn binding_rest_element_trailing_comma(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Unexpected trailing comma after rest element").with_label(span)
}

#[cold]
pub fn invalid_binding_rest_element(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Invalid rest element")
        .with_help("Expected identifier in rest element")
        .with_label(span)
}

#[cold]
pub fn a_rest_parameter_cannot_be_optional(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("A rest parameter cannot be optional").with_label(span)
}

#[cold]
pub fn invalid_assignment(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Cannot assign to this expression").with_label(span)
}

#[cold]
pub fn new_optional_chain(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Optional chaining cannot appear in the callee of new expressions")
        .with_label(span)
}

#[cold]
pub fn for_loop_async_of(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("The left-hand side of a `for...of` statement may not be `async`")
        .with_label(span)
}

#[cold]
pub fn for_await(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("await can only be used in conjunction with `for...of` statements")
        .with_label(span)
}

#[cold]
pub fn new_dynamic_import(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Cannot use new with dynamic import").with_label(span)
}

#[cold]
pub fn private_name_constructor(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Classes can't have an element named '#constructor'").with_label(span)
}

#[cold]
pub fn static_prototype(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Classes may not have a static property named prototype").with_label(span)
}

#[cold]
pub fn constructor_getter_setter(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Constructor can't have get/set modifier").with_label(span)
}

#[cold]
pub fn constructor_async(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Constructor can't be an async method").with_label(span)
}

#[cold]
pub fn optional_accessor_property(span: Span) -> OxcDiagnostic {
    ts_error("1276", "An 'accessor' property cannot be declared optional.").with_label(span)
}

#[cold]
pub fn optional_definite_property(span: Span) -> OxcDiagnostic {
    // NOTE: could not find an error code when tsc parses this; its parser panics.
    OxcDiagnostic::error("A property cannot be both optional and definite.")
        .with_label(span)
        .with_help("Remove either the `?` or the `!`")
}

#[cold]
pub fn identifier_async(x0: &str, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("Cannot use `{x0}` as an identifier in an async context"))
        .with_label(span1)
}

#[cold]
pub fn identifier_generator(x0: &str, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("Cannot use `{x0}` as an identifier in a generator context"))
        .with_label(span1)
}

#[cold]
pub fn constructor_generator(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Constructor can't be a generator").with_label(span)
}

#[cold]
pub fn field_constructor(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Classes can't have a field named 'constructor'").with_label(span)
}

#[cold]
pub fn export_lone_surrogate(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("An export name cannot include a unicode lone surrogate").with_label(span)
}

#[cold]
pub fn export_named_string(x0: &str, x1: &str, span2: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("A string literal cannot be used as an exported binding without `from`")
        .with_help(format!("Did you mean `export {{ {x0} as {x1} }} from 'some-module'`?"))
        .with_label(span2)
}

#[cold]
pub fn export_reserved_word(x0: &str, x1: &str, span2: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("A reserved word cannot be used as an exported binding without `from`")
        .with_help(format!("Did you mean `export {{ {x0} as {x1} }} from 'some-module'`?"))
        .with_label(span2)
}

#[cold]
pub fn template_literal(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Bad escape sequence in untagged template literal").with_label(span)
}

#[cold]
pub fn empty_parenthesized_expression(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Empty parenthesized expression").with_label(span)
}

#[cold]
pub fn illegal_newline(x0: &str, span1: Span, span2: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("Illegal newline after {x0}")).with_labels([
        span1.label(format!("{x0} starts here")),
        span2.label("A newline is not expected here"),
    ])
}

#[cold]
pub fn optional_chain_tagged_template(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Tagged template expressions are not permitted in an optional chain")
        .with_label(span)
}

#[cold]
pub fn ts_constructor_this_parameter(span: Span) -> OxcDiagnostic {
    ts_error("2681", "A constructor cannot have a `this` parameter.").with_label(span)
}

#[cold]
pub fn ts_arrow_function_this_parameter(span: Span) -> OxcDiagnostic {
    ts_error("2730", "An arrow function cannot have a `this` parameter.").with_label(span)
}

#[cold]
pub fn unexpected_super(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("'super' can only be used with function calls or in property accesses")
        .with_help("replace with `super()` or `super.prop` or `super[prop]`")
        .with_label(span)
}

#[cold]
pub fn expect_function_name(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Expected function name")
        .with_help("Function name is required in function declaration or named export")
        .with_label(span)
}

#[cold]
pub fn expect_catch_finally(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Missing catch or finally clause").with_label(span)
}

#[cold]
pub fn a_set_accessor_cannot_have_a_return_type_annotation(span: Span) -> OxcDiagnostic {
    ts_error("1095", " A 'set' accessor cannot have a return type annotation.").with_label(span)
}

#[cold]
pub fn return_statement_only_in_function_body(span: Span) -> OxcDiagnostic {
    ts_error("1108", "A 'return' statement can only be used within a function body.")
        .with_label(span)
}

#[cold]
pub fn jsx_expressions_may_not_use_the_comma_operator(span: Span) -> OxcDiagnostic {
    // OxcDiagnostic::error("TS18007: JSX expressions may not use the comma
    // operator.")
    ts_error("18007", "JSX expressions may not use the comma operator")
        .with_help("Did you mean to write an array?")
        .with_label(span)
}

#[cold]
pub fn line_terminator_before_using_declaration(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Line terminator not permitted before using declaration.").with_label(span)
}

#[cold]
pub fn await_in_using_declaration(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Await is not allowed in using declarations.").with_label(span)
}

#[cold]
pub fn invalid_identifier_in_using_declaration(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Using declarations may not have binding patterns.").with_label(span)
}

#[cold]
pub fn await_using_declaration_not_allowed_in_for_in_statement(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(
        "The left-hand side of a for...in statement cannot be an await using declaration.",
    )
    .with_label(span)
}

#[cold]
pub fn using_declaration_not_allowed_in_for_in_statement(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(
        "The left-hand side of a for...in statement cannot be an using declaration.",
    )
    .with_label(span)
}

#[cold]
pub fn using_declarations_must_be_initialized(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Using declarations must have an initializer.").with_label(span)
}

/// TS(1093)
#[cold]
pub fn static_constructor(span: Span) -> OxcDiagnostic {
    ts_error("1089", "`static` modifier cannot appear on a constructor declaration.")
        .with_label(span)
}

#[cold]
pub fn jsx_element_no_match(span: Span, span1: Span, name: &str) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("Expected corresponding JSX closing tag for '{name}'."))
        .with_labels([span, span1])
}

// ================================= MODIFIERS =================================

#[cold]
pub fn modifier_cannot_be_used_here(modifier: &Modifier) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("'{}' modifier cannot be used here.", modifier.kind))
        .with_label(modifier.span)
}

/// TS(1030)
#[cold]
pub fn modifier_already_seen(modifier: &Modifier) -> OxcDiagnostic {
    // OxcDiagnostic::error(format!("TS1030: '{}' modifier already seen.", modifier.kind))
    ts_error("1030", format!("{}' modifier already seen.", modifier.kind))
        .with_label(modifier.span)
        .with_help("Remove the duplicate modifier.")
}

/// TS(1273)
#[cold]
pub fn cannot_appear_on_a_type_parameter(modifier: &Modifier) -> OxcDiagnostic {
    ts_error("1273", format!("'{}' modifier cannot be used on a type parameter.", modifier.kind))
        .with_label(modifier.span)
}

/// TS(1090)
pub fn cannot_appear_on_a_parameter(modifier: &Modifier) -> OxcDiagnostic {
    ts_error("1090", format!("'{}' modifier cannot appear on a parameter.", modifier.kind))
        .with_label(modifier.span)
}

/// TS(18010)
#[cold]
pub fn accessibility_modifier_on_private_property(modifier: &Modifier) -> OxcDiagnostic {
    ts_error("18010", "An accessibility modifier cannot be used with a private identifier.")
        .with_label(modifier.span)
}

// ================================== TS ENUMS =================================

/// Computed property names are not allowed in enums.ts(1164)
#[cold]
pub fn computed_property_names_not_allowed_in_enums(span: Span) -> OxcDiagnostic {
    ts_error("1164", "Computed property names are not allowed in enums.").with_label(span)
}
/// An enum member cannot have a numeric name.ts(2452)
#[cold]
pub fn enum_member_cannot_have_numeric_name(span: Span) -> OxcDiagnostic {
    ts_error("2452", "An enum member cannot have a numeric name.").with_label(span)
}
