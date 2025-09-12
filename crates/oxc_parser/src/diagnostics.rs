use std::borrow::Cow;

use oxc_diagnostics::OxcDiagnostic;
use oxc_span::Span;

use crate::modifiers::{Modifier, ModifierKind};

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
pub fn unexpected_jsx_end(span: Span, a: char, b: &str) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("Unexpected token. Did you mean `{{'{a}'}}` or `&{b};`?"))
        .with_label(span)
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

// 'extends' clause already seen. ts(1172)
#[cold]
pub fn extends_clause_already_seen(span: Span) -> OxcDiagnostic {
    ts_error("1172", "'extends' clause already seen").with_label(span)
}

// 'extends' clause must precede 'implements' clause. ts(1173)
#[cold]
pub fn extends_clause_must_precede_implements(span: Span) -> OxcDiagnostic {
    ts_error("1173", "'extends' clause must precede 'implements' clause").with_label(span)
}

// 'implements' clause already seen. ts(1175)
#[cold]
pub fn implements_clause_already_seen(span: Span) -> OxcDiagnostic {
    ts_error("1175", "'implements' clause already seen").with_label(span)
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
pub fn rest_element_trailing_comma(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("A rest parameter or binding pattern may not have a trailing comma.")
        .with_label(span)
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
pub fn invalid_lhs_assignment(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(
        "The left-hand side of an assignment expression must be a variable or a property access.",
    )
    .with_label(span)
}

#[cold]
pub fn new_optional_chain(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Optional chaining cannot appear in the callee of new expressions")
        .with_label(span)
}

#[cold]
pub fn invalid_new_optional_chain(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Invalid optional chain from new expression.").with_label(span)
}

#[cold]
pub fn decorator_optional(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Expression must be enclosed in parentheses to be used as a decorator.")
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
pub fn identifier_expected(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Identifier expected.").with_label(span)
}

#[cold]
pub fn identifier_reserved_word(span: Span, reserved: &str) -> OxcDiagnostic {
    OxcDiagnostic::error(format!(
        "Identifier expected. '{reserved}' is a reserved word that cannot be used here."
    ))
    .with_label(span)
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
pub fn ts_constructor_type_parameter(span: Span) -> OxcDiagnostic {
    ts_error("1092", "Type parameters cannot appear on a constructor declaration").with_label(span)
}

#[cold]
pub fn ts_arrow_function_this_parameter(span: Span) -> OxcDiagnostic {
    ts_error("2730", "An arrow function cannot have a `this` parameter.").with_label(span)
}

#[cold]
pub fn ts_empty_type_parameter_list(span: Span) -> OxcDiagnostic {
    ts_error("1098", "Type parameter list cannot be empty.").with_label(span)
}

#[cold]
pub fn ts_empty_type_argument_list(span: Span) -> OxcDiagnostic {
    ts_error("1099", "Type argument list cannot be empty.").with_label(span)
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
pub fn v8_intrinsic_spread_elem(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("V8 runtime calls cannot have spread elements as arguments")
        .with_label(span)
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

#[cold]
pub fn jsx_element_no_match(span: Span, span1: Span, name: &str) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("Expected corresponding JSX closing tag for '{name}'."))
        .with_labels([span, span1])
}

#[cold]
pub fn cover_initialized_name(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Invalid assignment in object literal")
.with_help("Did you mean to use a ':'? An '=' can only follow a property name when the containing object literal is part of a destructuring pattern.")
.with_label(span)
}

#[cold]
pub fn duplicate_export(x0: &str, span1: Span, span2: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("Duplicated export '{x0}'")).with_labels([
        span1.label("Export has already been declared here"),
        span2.label("It cannot be redeclared here"),
    ])
}

#[cold]
pub fn import_meta(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("The only valid meta property for import is import.meta").with_label(span)
}

#[cold]
pub fn new_target(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("The only valid meta property for new is new.target").with_label(span)
}

#[cold]
pub fn private_in_private(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Unexpected right-hand side of private-in expression").with_label(span)
}

#[cold]
pub fn import_arguments(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Dynamic imports can only accept a module specifier and an optional set of attributes as arguments").with_label(span)
}

#[cold]
pub fn rest_element_property_name(span: Span) -> OxcDiagnostic {
    ts_error("2566", "A rest element cannot have a property name.").with_label(span)
}

#[cold]
pub fn a_rest_element_cannot_have_an_initializer(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("A rest element cannot have an initializer.").with_label(span)
}

#[cold]
pub fn import_requires_a_specifier(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("import() requires a specifier.").with_label(span)
}

#[cold]
pub fn modifier_cannot_be_used_here(modifier: &Modifier) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("'{}' modifier cannot be used here.", modifier.kind))
        .with_label(modifier.span)
}

#[cold]
pub fn modifier_only_on_property_declaration_or_index_signature(
    modifier: &Modifier,
) -> OxcDiagnostic {
    ts_error(
        "1024",
        format!(
            "'{}' modifier can only appear on a property declaration or index signature.",
            modifier.kind
        ),
    )
    .with_label(modifier.span)
}

#[cold]
pub fn accessibility_modifier_already_seen(modifier: &Modifier) -> OxcDiagnostic {
    ts_error("1028", "Accessibility modifier already seen.")
        .with_label(modifier.span)
        .with_help("Remove the duplicate modifier.")
}

#[cold]
pub fn modifier_must_precede_other_modifier(
    modifier: &Modifier,
    other_modifier: ModifierKind,
) -> OxcDiagnostic {
    ts_error(
        "1029",
        format!("'{}' modifier must precede '{}' modifier.", modifier.kind, other_modifier),
    )
    .with_label(modifier.span)
}

#[cold]
pub fn modifier_already_seen(modifier: &Modifier) -> OxcDiagnostic {
    ts_error("1030", format!("'{}' modifier already seen.", modifier.kind))
        .with_label(modifier.span)
        .with_help("Remove the duplicate modifier.")
}

pub fn cannot_appear_on_class_elements(modifier: &Modifier) -> OxcDiagnostic {
    ts_error(
        "1031",
        format!("'{}' modifier cannot appear on class elements of this kind.", modifier.kind),
    )
    .with_label(modifier.span)
}

pub fn cannot_appear_on_a_type_member(modifier: &Modifier) -> OxcDiagnostic {
    ts_error("1070", format!("'{}' modifier cannot appear on a type member.", modifier.kind))
        .with_label(modifier.span)
}

#[cold]
pub fn cannot_appear_on_a_type_parameter(modifier: &Modifier) -> OxcDiagnostic {
    ts_error("1273", format!("'{}' modifier cannot be used on a type parameter.", modifier.kind))
        .with_label(modifier.span)
}

pub fn cannot_appear_on_a_parameter(modifier: &Modifier) -> OxcDiagnostic {
    ts_error("1090", format!("'{}' modifier cannot appear on a parameter.", modifier.kind))
        .with_label(modifier.span)
}

pub fn cannot_appear_on_an_index_signature(modifier: &Modifier) -> OxcDiagnostic {
    ts_error("1071", format!("'{}' modifier cannot appear on an index signature.", modifier.kind))
        .with_label(modifier.span)
}

pub fn accessor_modifier(modifier: &Modifier) -> OxcDiagnostic {
    ts_error(
        "1243",
        format!("'accessor' modifier cannot be used with '{}' modifier.", modifier.kind),
    )
    .with_label(modifier.span)
}

#[cold]
pub fn readonly_in_array_or_tuple_type(span: Span) -> OxcDiagnostic {
    ts_error("1354", "'readonly' type modifier is only permitted on array and tuple literal types.")
        .with_label(span)
}

#[cold]
pub fn accessibility_modifier_on_private_property(modifier: &Modifier) -> OxcDiagnostic {
    ts_error("18010", "An accessibility modifier cannot be used with a private identifier.")
        .with_label(modifier.span)
}

#[cold]
pub fn type_modifier_on_named_type_import(span: Span) -> OxcDiagnostic {
    ts_error("2206", "The 'type' modifier cannot be used on a named import when 'import type' is used on its import statement.")
             .with_label(span)
}

#[cold]
pub fn type_modifier_on_named_type_export(span: Span) -> OxcDiagnostic {
    ts_error("2207", "The 'type' modifier cannot be used on a named export when 'export type' is used on its export statement.")
         .with_label(span)
}

#[cold]
pub fn computed_property_names_not_allowed_in_enums(span: Span) -> OxcDiagnostic {
    ts_error("1164", "Computed property names are not allowed in enums.").with_label(span)
}

#[cold]
pub fn enum_member_cannot_have_numeric_name(span: Span) -> OxcDiagnostic {
    ts_error("2452", "An enum member cannot have a numeric name.").with_label(span)
}

#[cold]
pub fn index_signature_one_parameter(span: Span) -> OxcDiagnostic {
    ts_error("1096", "An index signature must have exactly one parameter.").with_label(span)
}

#[cold]
pub fn mixed_coalesce(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Logical expressions and coalesce expressions cannot be mixed")
        .with_help("Wrap either expression by parentheses")
        .with_label(span)
}

#[cold]
pub fn unexpected_exponential(x0: &str, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Unexpected exponentiation expression")
        .with_help(format!("Wrap {x0} expression in parentheses to enforce operator precedence"))
        .with_label(span1)
}

#[cold]
pub fn import_equals_can_only_be_used_in_typescript_files(span: Span) -> OxcDiagnostic {
    ts_error("8002", "'import ... =' can only be used in TypeScript files.").with_label(span)
}

#[cold]
pub fn index_signature_question_mark(span: Span) -> OxcDiagnostic {
    ts_error("1019", "An index signature parameter cannot have a question mark.").with_label(span)
}

#[cold]
pub fn index_signature_type_annotation(span: Span) -> OxcDiagnostic {
    ts_error("1021", "An index signature must have a type annotation.").with_label(span)
}

#[cold]
pub fn unexpected_export(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Unexpected export.").with_label(span)
}

#[cold]
pub fn decorators_in_export_and_class(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Decorators may not appear after 'export' or 'export default' if they also appear before 'export'.").with_label(span)
}

#[cold]
pub fn decorators_are_not_valid_here(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Decorators are not valid here.").with_label(span)
}

#[cold]
pub fn decorator_on_overload(span: Span) -> OxcDiagnostic {
    ts_error("1249", "A decorator can only decorate a method implementation, not an overload.")
        .with_label(span)
}

#[cold]
pub fn as_in_ts(span: Span) -> OxcDiagnostic {
    ts_error("8037", "Type assertion expressions can only be used in TypeScript files.")
        .with_label(span)
}

#[cold]
pub fn satisfies_in_ts(span: Span) -> OxcDiagnostic {
    ts_error("8016", "Type satisfaction expressions can only be used in TypeScript files.")
        .with_label(span)
}

#[cold]
pub fn optional_and_rest_tuple_member(span: Span) -> OxcDiagnostic {
    ts_error("5085", "A tuple member cannot be both optional and rest.").with_label(span)
}

#[cold]
pub fn optional_after_tuple_member_name(span: Span) -> OxcDiagnostic {
    ts_error("5086", "A labeled tuple element is declared as optional with a question mark after the name and before the colon, rather than after the type.").with_label(span)
}

#[cold]
pub fn rest_after_tuple_member_name(span: Span) -> OxcDiagnostic {
    ts_error("5087", "A labeled tuple element is declared as rest with a '...' before the name, rather than before the type.").with_label(span)
}

#[cold]
pub fn parameter_modifiers_in_ts(modifier: &Modifier) -> OxcDiagnostic {
    ts_error("8012", "Parameter modifiers can only be used in TypeScript files.")
        .with_label(modifier.span)
}

#[cold]
pub fn implementation_in_ambient(span: Span) -> OxcDiagnostic {
    ts_error("1183", "An implementation cannot be declared in ambient contexts.").with_label(span)
}

#[cold]
pub fn interface_implements(span: Span) -> OxcDiagnostic {
    ts_error("1176", "Interface declaration cannot have 'implements' clause.").with_label(span)
}

#[cold]
pub fn interface_extend(span: Span) -> OxcDiagnostic {
    ts_error(
        "2499",
        "An interface can only extend an identifier/qualified-name with optional type arguments.",
    )
    .with_label(span)
}

#[cold]
pub fn reg_exp_flag_u_and_v(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(
        "The 'u' and 'v' regular expression flags cannot be enabled at the same time",
    )
    .with_label(span)
}

#[cold]
pub fn setter_with_parameters(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("A 'set' accessor must have exactly one parameter.").with_label(span)
}

#[cold]
pub fn setter_with_rest_parameter(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("A 'set' accessor cannot have rest parameter.").with_label(span)
}
#[cold]
pub fn setter_with_assignment_pattern(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("A 'set' accessor cannot have an initializer.").with_label(span)
}

#[cold]
pub fn getter_parameters(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("A 'get' accessor must not have any formal parameters.").with_label(span)
}

#[cold]
pub fn variable_declarator_definite(span: Span) -> OxcDiagnostic {
    ts_error(
        "1263",
        "Declarations with initializers cannot also have definite assignment assertions.",
    )
    .with_label(span)
}

#[cold]
pub fn variable_declarator_definite_type_assertion(span: Span) -> OxcDiagnostic {
    ts_error(
        "1264",
        "Declarations with definite assignment assertions must also have type annotations.",
    )
    .with_label(span)
}

#[cold]
pub fn invalid_rest_assignment_target(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Invalid rest operator's argument.").with_label(span)
}

#[cold]
pub fn modifiers_cannot_appear_here(span: Span) -> OxcDiagnostic {
    ts_error("1184", "Modifiers cannot appear here.").with_label(span)
}
