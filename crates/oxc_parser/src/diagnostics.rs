use oxc_diagnostics::{LabeledSpan, OxcDiagnostic};
use oxc_span::Span;

#[cold]
pub fn redeclaration(x0: &str, span1: Span, span2: Span) -> OxcDiagnostic {
    OxcDiagnostic::new(format!("Identifier `{x0}` has already been declared")).with_labels([
        LabeledSpan::new_with_span(Some(format!("`{x0}` has already been declared here")), span1),
        LabeledSpan::new_with_span(Some("It can not be redeclared here".to_string()), span2),
    ])
}

#[cold]
pub fn overlong_source() -> OxcDiagnostic {
    OxcDiagnostic::new("Source length exceeds 4 GiB limit")
}

#[cold]
pub fn flow(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::new("Flow is not supported").with_labels([span0.into()])
}

#[cold]
pub fn unexpected_token(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::new("Unexpected token").with_labels([span0.into()])
}

#[cold]
pub fn expect_token(x0: &str, x1: &str, span2: Span) -> OxcDiagnostic {
    OxcDiagnostic::new(format!("Expected `{x0}` but found `{x1}`"))
        .with_labels([LabeledSpan::new_with_span(Some(format!("`{x0}` expected")), span2)])
}

#[cold]
pub fn invalid_escape_sequence(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::new("Invalid escape sequence").with_labels([span0.into()])
}

#[cold]
pub fn unicode_escape_sequence(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::new("Invalid Unicode escape sequence").with_labels([span0.into()])
}

#[cold]
pub fn invalid_character(x0: char, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::new(format!("Invalid Character `{x0}`")).with_labels([span1.into()])
}

#[cold]
pub fn invalid_number_end(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::new("Invalid characters after number").with_labels([span0.into()])
}

#[cold]
pub fn unterminated_multi_line_comment(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::new("Unterminated multiline comment").with_labels([span0.into()])
}

#[cold]
pub fn unterminated_string(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::new("Unterminated string").with_labels([span0.into()])
}

#[cold]
pub fn reg_exp_flag(x0: char, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::new(format!("Unexpected flag {x0} in regular expression literal"))
        .with_labels([span1.into()])
}

#[cold]
pub fn reg_exp_flag_twice(x0: char, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::new(format!("Flag {x0} is mentioned twice in regular expression literal"))
        .with_labels([span1.into()])
}

#[cold]
pub fn unexpected_end(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::new("Unexpected end of file").with_labels([span0.into()])
}

#[cold]
pub fn unterminated_reg_exp(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::new("Unterminated regular expression").with_labels([span0.into()])
}

#[cold]
pub fn invalid_number(x0: &str, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::new(format!("Invalid Number {x0}")).with_labels([span1.into()])
}

#[cold]
pub fn escaped_keyword(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::new("Keywords cannot contain escape characters").with_labels([span0.into()])
}

#[cold]
pub fn auto_semicolon_insertion(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::new(
        "Expected a semicolon or an implicit semicolon after a statement, but found none",
    )
    .with_help("Try insert a semicolon here")
    .with_labels([span0.into()])
}

#[cold]
pub fn lineterminator_before_arrow(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::new("Line terminator not permitted before arrow").with_labels([span0.into()])
}

#[cold]
pub fn invalid_destrucuring_declaration(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::new("Missing initializer in destructuring declaration")
        .with_labels([span0.into()])
}

#[cold]
pub fn missinginitializer_in_const(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::new("Missing initializer in const declaration").with_labels([span0.into()])
}

#[cold]
pub fn lexical_declaration_single_statement(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::new("Lexical declaration cannot appear in a single-statement context")
        .with_help("Wrap this declaration in a block statement")
        .with_labels([span0.into()])
}

#[cold]
pub fn async_function_declaration(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::new("Async functions can only be declared at the top level or inside a block")
        .with_labels([span0.into()])
}

#[cold]
pub fn generator_function_declaration(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::new("Generators can only be declared at the top level or inside a block")
        .with_labels([span0.into()])
}

#[cold]
pub fn await_expression(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::new(
        "`await` is only allowed within async functions and at the top levels of modules",
    )
    .with_labels([span0.into()])
}

#[cold]
pub fn yield_expression(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::new("A 'yield' expression is only allowed in a generator body.")
        .with_labels([span0.into()])
}

#[cold]
pub fn class_declaration(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::new("Invalid class declaration")
        .with_help("Classes can only be declared at top level or inside a block")
        .with_labels([span0.into()])
}

#[cold]
pub fn binding_rest_element_last(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::new("A rest element must be last in a destructuring pattern")
        .with_labels([span0.into()])
}

#[cold]
pub fn rest_parameter_last(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::new("A rest parameter must be last in a parameter list")
        .with_labels([span0.into()])
}

#[cold]
pub fn spread_last_element(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::new("Spread must be last element").with_labels([span0.into()])
}

#[cold]
pub fn binding_rest_element_trailing_comma(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::new("Unexpected trailing comma after rest element").with_labels([span0.into()])
}

#[cold]
pub fn invalid_binding_rest_element(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::new("Invalid rest element")
        .with_help("Expected identifier in rest element")
        .with_labels([span0.into()])
}

#[cold]
pub fn a_rest_parameter_cannot_be_optional(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::new("A rest parameter cannot be optional").with_labels([span0.into()])
}

#[cold]
pub fn invalid_assignment(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::new("Cannot assign to this expression").with_labels([span0.into()])
}

#[cold]
pub fn new_optional_chain(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::new("Optional chaining cannot appear in the callee of new expressions")
        .with_labels([span0.into()])
}

#[cold]
pub fn for_loop_async_of(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::new("The left-hand side of a `for...of` statement may not be `async`")
        .with_labels([span0.into()])
}

#[cold]
pub fn for_await(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::new("await can only be used in conjunction with `for...of` statements")
        .with_labels([span0.into()])
}

#[cold]
pub fn new_dynamic_import(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::new("Cannot use new with dynamic import").with_labels([span0.into()])
}

#[cold]
pub fn private_name_constructor(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::new("Classes can't have an element named '#constructor'")
        .with_labels([span0.into()])
}

#[cold]
pub fn static_prototype(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::new("Classes may not have a static property named prototype")
        .with_labels([span0.into()])
}

#[cold]
pub fn constructor_getter_setter(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::new("Constructor can't have get/set modifier").with_labels([span0.into()])
}

#[cold]
pub fn constructor_async(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::new("Constructor can't be an async method").with_labels([span0.into()])
}

#[cold]
pub fn identifier_async(x0: &str, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::new(format!("Cannot use `{x0}` as an identifier in an async context"))
        .with_labels([span1.into()])
}

#[cold]
pub fn identifier_generator(x0: &str, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::new(format!("Cannot use `{x0}` as an identifier in a generator context"))
        .with_labels([span1.into()])
}

#[cold]
pub fn constructor_generator(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::new("Constructor can't be a generator").with_labels([span0.into()])
}

#[cold]
pub fn field_constructor(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::new("Classes can't have a field named 'constructor'").with_labels([span0.into()])
}

#[cold]
pub fn export_lone_surrogate(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::new("An export name cannot include a unicode lone surrogate")
        .with_labels([span0.into()])
}

#[cold]
pub fn export_named_string(x0: &str, x1: &str, span2: Span) -> OxcDiagnostic {
    OxcDiagnostic::new("A string literal cannot be used as an exported binding without `from`")
        .with_help(format!("Did you mean `export {{ {x0} as {x1} }} from 'some-module'`?"))
        .with_labels([span2.into()])
}

#[cold]
pub fn export_reserved_word(x0: &str, x1: &str, span2: Span) -> OxcDiagnostic {
    OxcDiagnostic::new("A reserved word cannot be used as an exported binding without `from`")
        .with_help(format!("Did you mean `export {{ {x0} as {x1} }} from 'some-module'`?"))
        .with_labels([span2.into()])
}

#[cold]
pub fn template_literal(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::new("Bad escape sequence in untagged template literal")
        .with_labels([span0.into()])
}

#[cold]
pub fn empty_parenthesized_expression(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::new("Empty parenthesized expression").with_labels([span0.into()])
}

#[cold]
pub fn illegal_newline(x0: &str, span1: Span, span2: Span) -> OxcDiagnostic {
    OxcDiagnostic::new(format!("Illegal newline after {x0}")).with_labels([
        LabeledSpan::new_with_span(Some(format!("{x0} starts here")), span1),
        LabeledSpan::new_with_span(Some("A newline is not expected here".to_string()), span2),
    ])
}

#[cold]
pub fn optional_chain_tagged_template(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::new("Tagged template expressions are not permitted in an optional chain")
        .with_labels([span0.into()])
}

#[cold]
pub fn ts_constructor_this_parameter(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::new("TS2681: A constructor cannot have a `this` parameter.")
        .with_labels([span0.into()])
}

#[cold]
pub fn ts_arrow_function_this_parameter(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::new("TS2730: An arrow function cannot have a `this` parameter.")
        .with_labels([span0.into()])
}

#[cold]
pub fn unexpected_super(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::new("'super' can only be used with function calls or in property accesses")
        .with_help("replace with `super()` or `super.prop` or `super[prop]`")
        .with_labels([span0.into()])
}

#[cold]
pub fn expect_function_name(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::new("Expected function name")
        .with_help("Function name is required in function declaration or named export")
        .with_labels([span0.into()])
}

#[cold]
pub fn expect_catch_finally(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::new("Missing catch or finally clause").with_labels([span0.into()])
}

#[cold]
pub fn a_set_accessor_cannot_have_a_return_type_annotation(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::new("TS1095: A 'set' accessor cannot have a return type annotation")
        .with_labels([span0.into()])
}

#[cold]
pub fn return_statement_only_in_function_body(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::new("TS1108: A 'return' statement can only be used within a function body")
        .with_labels([span0.into()])
}

#[cold]
pub fn jsx_expressions_may_not_use_the_comma_operator(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::new("TS18007: JSX expressions may not use the comma operator.")
        .with_help("Did you mean to write an array?")
        .with_labels([span0.into()])
}

#[cold]
pub fn line_terminator_before_using_declaration(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::new("Line terminator not permitted before using declaration.")
        .with_labels([span0.into()])
}

#[cold]
pub fn await_in_using_declaration(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::new("Await is not allowed in using declarations.").with_labels([span0.into()])
}

#[cold]
pub fn invalid_identifier_in_using_declaration(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::new("Using declarations may not have binding patterns.")
        .with_labels([span0.into()])
}

#[cold]
pub fn await_using_declaration_not_allowed_in_for_in_statement(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::new(
        "The left-hand side of a for...in statement cannot be an await using declaration.",
    )
    .with_labels([span0.into()])
}

#[cold]
pub fn using_declaration_not_allowed_in_for_in_statement(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::new("The left-hand side of a for...in statement cannot be an using declaration.")
        .with_labels([span0.into()])
}

#[cold]
pub fn using_declarations_must_be_initialized(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::new("Using declarations must have an initializer.").with_labels([span0.into()])
}

#[cold]
pub fn static_constructor(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::new("TS1089: `static` modifier cannot appear on a constructor declaration.")
        .with_labels([span0.into()])
}

#[cold]
pub fn no_line_break_is_allowed_before_arrow(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::new("No line break is allowed before '=>'.").with_labels([span0.into()])
}
