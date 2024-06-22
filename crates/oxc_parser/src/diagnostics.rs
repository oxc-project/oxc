use oxc_diagnostics::{LabeledSpan, OxcDiagnostic};
use oxc_span::Span;

#[cold]
pub fn redeclaration(x0: &str, span1: Span, span2: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("Identifier `{x0}` has already been declared")).with_labels([
        LabeledSpan::new_with_span(Some(format!("`{x0}` has already been declared here")), span1),
        LabeledSpan::new_with_span(Some("It can not be redeclared here".to_string()), span2),
    ])
}

#[cold]
pub fn overlong_source() -> OxcDiagnostic {
    OxcDiagnostic::error("Source length exceeds 4 GiB limit")
}

#[cold]
pub fn flow(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Flow is not supported").with_labels([span0.into()])
}

#[cold]
pub fn unexpected_token(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Unexpected token").with_labels([span0.into()])
}

#[cold]
pub fn expect_token(x0: &str, x1: &str, span2: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("Expected `{x0}` but found `{x1}`"))
        .with_labels([LabeledSpan::new_with_span(Some(format!("`{x0}` expected")), span2)])
}

#[cold]
pub fn invalid_escape_sequence(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Invalid escape sequence").with_labels([span0.into()])
}

#[cold]
pub fn unicode_escape_sequence(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Invalid Unicode escape sequence").with_labels([span0.into()])
}

#[cold]
pub fn invalid_character(x0: char, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("Invalid Character `{x0}`")).with_labels([span1.into()])
}

#[cold]
pub fn invalid_number_end(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Invalid characters after number").with_labels([span0.into()])
}

#[cold]
pub fn unterminated_multi_line_comment(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Unterminated multiline comment").with_labels([span0.into()])
}

#[cold]
pub fn unterminated_string(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Unterminated string").with_labels([span0.into()])
}

#[cold]
pub fn reg_exp_flag(x0: char, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("Unexpected flag {x0} in regular expression literal"))
        .with_labels([span1.into()])
}

#[cold]
pub fn reg_exp_flag_twice(x0: char, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("Flag {x0} is mentioned twice in regular expression literal"))
        .with_labels([span1.into()])
}

#[cold]
pub fn unexpected_end(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Unexpected end of file").with_labels([span0.into()])
}

#[cold]
pub fn unterminated_reg_exp(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Unterminated regular expression").with_labels([span0.into()])
}

#[cold]
pub fn invalid_number(x0: &str, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("Invalid Number {x0}")).with_labels([span1.into()])
}

#[cold]
pub fn escaped_keyword(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Keywords cannot contain escape characters").with_labels([span0.into()])
}

#[cold]
pub fn auto_semicolon_insertion(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(
        "Expected a semicolon or an implicit semicolon after a statement, but found none",
    )
    .with_help("Try insert a semicolon here")
    .with_labels([span0.into()])
}

#[cold]
pub fn lineterminator_before_arrow(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Line terminator not permitted before arrow").with_labels([span0.into()])
}

#[cold]
pub fn invalid_destrucuring_declaration(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Missing initializer in destructuring declaration")
        .with_labels([span0.into()])
}

#[cold]
pub fn missinginitializer_in_const(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Missing initializer in const declaration").with_labels([span0.into()])
}

#[cold]
pub fn lexical_declaration_single_statement(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Lexical declaration cannot appear in a single-statement context")
        .with_help("Wrap this declaration in a block statement")
        .with_labels([span0.into()])
}

#[cold]
pub fn async_function_declaration(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Async functions can only be declared at the top level or inside a block")
        .with_labels([span0.into()])
}

#[cold]
pub fn generator_function_declaration(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Generators can only be declared at the top level or inside a block")
        .with_labels([span0.into()])
}

#[cold]
pub fn await_expression(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(
        "`await` is only allowed within async functions and at the top levels of modules",
    )
    .with_labels([span0.into()])
}

#[cold]
pub fn yield_expression(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("A 'yield' expression is only allowed in a generator body.")
        .with_labels([span0.into()])
}

#[cold]
pub fn class_declaration(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Invalid class declaration")
        .with_help("Classes can only be declared at top level or inside a block")
        .with_labels([span0.into()])
}

#[cold]
pub fn binding_rest_element_last(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("A rest element must be last in a destructuring pattern")
        .with_labels([span0.into()])
}

#[cold]
pub fn rest_parameter_last(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("A rest parameter must be last in a parameter list")
        .with_labels([span0.into()])
}

#[cold]
pub fn spread_last_element(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Spread must be last element").with_labels([span0.into()])
}

#[cold]
pub fn binding_rest_element_trailing_comma(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Unexpected trailing comma after rest element").with_labels([span0.into()])
}

#[cold]
pub fn invalid_binding_rest_element(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Invalid rest element")
        .with_help("Expected identifier in rest element")
        .with_labels([span0.into()])
}

#[cold]
pub fn a_rest_parameter_cannot_be_optional(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("A rest parameter cannot be optional").with_labels([span0.into()])
}

#[cold]
pub fn invalid_assignment(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Cannot assign to this expression").with_labels([span0.into()])
}

#[cold]
pub fn new_optional_chain(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Optional chaining cannot appear in the callee of new expressions")
        .with_labels([span0.into()])
}

#[cold]
pub fn for_loop_async_of(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("The left-hand side of a `for...of` statement may not be `async`")
        .with_labels([span0.into()])
}

#[cold]
pub fn for_await(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("await can only be used in conjunction with `for...of` statements")
        .with_labels([span0.into()])
}

#[cold]
pub fn new_dynamic_import(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Cannot use new with dynamic import").with_labels([span0.into()])
}

#[cold]
pub fn private_name_constructor(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Classes can't have an element named '#constructor'")
        .with_labels([span0.into()])
}

#[cold]
pub fn static_prototype(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Classes may not have a static property named prototype")
        .with_labels([span0.into()])
}

#[cold]
pub fn constructor_getter_setter(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Constructor can't have get/set modifier").with_labels([span0.into()])
}

#[cold]
pub fn constructor_async(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Constructor can't be an async method").with_labels([span0.into()])
}

#[cold]
pub fn identifier_async(x0: &str, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("Cannot use `{x0}` as an identifier in an async context"))
        .with_labels([span1.into()])
}

#[cold]
pub fn identifier_generator(x0: &str, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("Cannot use `{x0}` as an identifier in a generator context"))
        .with_labels([span1.into()])
}

#[cold]
pub fn constructor_generator(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Constructor can't be a generator").with_labels([span0.into()])
}

#[cold]
pub fn field_constructor(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Classes can't have a field named 'constructor'")
        .with_labels([span0.into()])
}

#[cold]
pub fn export_lone_surrogate(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("An export name cannot include a unicode lone surrogate")
        .with_labels([span0.into()])
}

#[cold]
pub fn export_named_string(x0: &str, x1: &str, span2: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("A string literal cannot be used as an exported binding without `from`")
        .with_help(format!("Did you mean `export {{ {x0} as {x1} }} from 'some-module'`?"))
        .with_labels([span2.into()])
}

#[cold]
pub fn export_reserved_word(x0: &str, x1: &str, span2: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("A reserved word cannot be used as an exported binding without `from`")
        .with_help(format!("Did you mean `export {{ {x0} as {x1} }} from 'some-module'`?"))
        .with_labels([span2.into()])
}

#[cold]
pub fn template_literal(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Bad escape sequence in untagged template literal")
        .with_labels([span0.into()])
}

#[cold]
pub fn empty_parenthesized_expression(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Empty parenthesized expression").with_labels([span0.into()])
}

#[cold]
pub fn illegal_newline(x0: &str, span1: Span, span2: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("Illegal newline after {x0}")).with_labels([
        LabeledSpan::new_with_span(Some(format!("{x0} starts here")), span1),
        LabeledSpan::new_with_span(Some("A newline is not expected here".to_string()), span2),
    ])
}

#[cold]
pub fn optional_chain_tagged_template(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Tagged template expressions are not permitted in an optional chain")
        .with_labels([span0.into()])
}

#[cold]
pub fn ts_constructor_this_parameter(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("TS2681: A constructor cannot have a `this` parameter.")
        .with_labels([span0.into()])
}

#[cold]
pub fn ts_arrow_function_this_parameter(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("TS2730: An arrow function cannot have a `this` parameter.")
        .with_labels([span0.into()])
}

#[cold]
pub fn unexpected_super(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("'super' can only be used with function calls or in property accesses")
        .with_help("replace with `super()` or `super.prop` or `super[prop]`")
        .with_labels([span0.into()])
}

#[cold]
pub fn expect_function_name(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Expected function name")
        .with_help("Function name is required in function declaration or named export")
        .with_labels([span0.into()])
}

#[cold]
pub fn expect_catch_finally(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Missing catch or finally clause").with_labels([span0.into()])
}

#[cold]
pub fn a_set_accessor_cannot_have_a_return_type_annotation(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("TS1095: A 'set' accessor cannot have a return type annotation")
        .with_labels([span0.into()])
}

#[cold]
pub fn return_statement_only_in_function_body(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("TS1108: A 'return' statement can only be used within a function body")
        .with_labels([span0.into()])
}

#[cold]
pub fn jsx_expressions_may_not_use_the_comma_operator(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("TS18007: JSX expressions may not use the comma operator.")
        .with_help("Did you mean to write an array?")
        .with_labels([span0.into()])
}

#[cold]
pub fn line_terminator_before_using_declaration(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Line terminator not permitted before using declaration.")
        .with_labels([span0.into()])
}

#[cold]
pub fn await_in_using_declaration(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Await is not allowed in using declarations.").with_labels([span0.into()])
}

#[cold]
pub fn invalid_identifier_in_using_declaration(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Using declarations may not have binding patterns.")
        .with_labels([span0.into()])
}

#[cold]
pub fn await_using_declaration_not_allowed_in_for_in_statement(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(
        "The left-hand side of a for...in statement cannot be an await using declaration.",
    )
    .with_labels([span0.into()])
}

#[cold]
pub fn using_declaration_not_allowed_in_for_in_statement(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(
        "The left-hand side of a for...in statement cannot be an using declaration.",
    )
    .with_labels([span0.into()])
}

#[cold]
pub fn using_declarations_must_be_initialized(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Using declarations must have an initializer.").with_labels([span0.into()])
}

#[cold]
pub fn static_constructor(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("TS1089: `static` modifier cannot appear on a constructor declaration.")
        .with_labels([span0.into()])
}

#[cold]
pub fn jsx_element_no_match(span0: Span, span1: Span, name: &str) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("Expected corresponding JSX closing tag for '{name}'."))
        .with_labels([span0.into(), span1.into()])
}

#[cold]
pub fn only_ambient_modules_can_use_quoted_names(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("TS1035: Only ambient modules can use quoted names.").with_label(span0)
}

#[cold]
pub fn ambient_modules_cannot_be_nested_in_other_modules_or_namespaces(
    span0: Span,
) -> OxcDiagnostic {
    OxcDiagnostic::error("TS2435: Ambient modules cannot be nested in other modules or namespaces.")
        .with_label(span0)
}
