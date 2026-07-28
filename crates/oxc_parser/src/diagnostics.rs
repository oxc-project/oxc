use std::borrow::Cow;

use oxc_ast::ast::REGEXP_FLAGS_LIST;
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::Span;

use crate::modifiers::{Modifier, ModifierKind, ModifierKinds};

trait DiagnosticExt {
    fn with_allowed_modifier_help(self, allowed: Option<ModifierKinds>) -> Self;
}

impl DiagnosticExt for OxcDiagnostic {
    fn with_allowed_modifier_help(self, allowed: Option<ModifierKinds>) -> Self {
        if let Some(allowed) = allowed {
            match allowed.count() {
                0 => self.with_help("No modifiers are allowed here."),
                1 => self.with_help(format!("Only '{allowed}' modifier is allowed here.")),
                _ => self.with_help(format!("Allowed modifiers are: {allowed}")),
            }
        } else {
            self
        }
    }
}

#[inline]
fn ts_error<M>(code: &'static str, message: M) -> OxcDiagnostic
where
    M: Into<Cow<'static, str>>,
{
    OxcDiagnostic::error(message).with_error_code("TS", code)
}

/// Declares every parser diagnostic as a variant of [`ParserDiagnostic`].
///
/// Each entry `name(args) => body` generates three things that stay in sync:
/// 1. an enum variant `ParserDiagnostic::name { args }` holding only the POD arguments,
/// 2. a `#[cold]` match arm in [`ParserDiagnostic::into_diagnostic`] holding `body`
///    (the eager `OxcDiagnostic` construction — boxing, label strings, `format!`),
/// 3. an `#[inline]` constructor `pub fn name(args) -> ParserDiagnostic` that just
///    writes the variant.
///
/// Diagnostics are therefore allocation-free until materialized once at parse exit.
macro_rules! parser_diagnostics {
    (
        $(
            $name:ident ( $($pname:ident : $pty:ty),* $(,)? ) => $body:expr
        );* $(;)?
    ) => {
        /// A parser or lexer diagnostic, stored in deferred (unmaterialized) form.
        ///
        /// During parsing only the POD arguments are kept, so pushing, cloning
        /// (checkpoint), and discarding (rewind) an error never allocates. The eager
        /// [`OxcDiagnostic`] is built once, at parse exit, via [`Self::into_diagnostic`].
        #[derive(Debug, Clone)]
        #[expect(non_camel_case_types)]
        pub enum ParserDiagnostic<'a> {
            $(
                $name { $($pname : $pty),* },
            )*
            /// An already-materialized diagnostic (e.g. forwarded from the regex parser).
            Eager(Box<OxcDiagnostic>),
        }

        impl<'a> ParserDiagnostic<'a> {
            /// Materialize into an owned [`OxcDiagnostic`]. Called once per diagnostic
            /// at parse exit, so all the formatting/allocation work lives here.
            #[cold]
            pub fn into_diagnostic(self) -> OxcDiagnostic {
                match self {
                    $(
                        Self::$name { $($pname),* } => $body,
                    )*
                    Self::Eager(diagnostic) => *diagnostic,
                }
            }
        }

        $(
            #[inline]
            pub fn $name<'a>($($pname : $pty),*) -> ParserDiagnostic<'a> {
                ParserDiagnostic::$name { $($pname),* }
            }
        )*
    };
}

parser_diagnostics! {
    redeclaration(name: &'a str, declare_span: Span, redeclare_span: Span) => {
        OxcDiagnostic::error(format!("Identifier `{name}` has already been declared")).with_labels([
            declare_span.label(format!("`{name}` has already been declared here")),
            redeclare_span.label("It can not be redeclared here"),
        ])
    };

    overlong_source() => {
        OxcDiagnostic::error("Source length exceeds 4 GiB limit")
    };

    file_appears_to_be_binary() => {
        ts_error("1490", "File appears to be binary.")
    };

    flow(span: Span) => {
        OxcDiagnostic::error("Flow is not supported").with_label(span)
    };

    unexpected_token(span: Span) => {
        OxcDiagnostic::error("Unexpected token").with_label(span)
    };

    // 'abstract' modifier can only appear on a class, method, or property declaration. (1242)
    illegal_abstract_modifier(span: Span) => {
        ts_error(
            "1242",
            "'abstract' modifier can only appear on a class, method, or property declaration.",
        )
        .with_label(span)
    };

    private_identifier_in_property_name(name: &'a str, span: Span) => {
        OxcDiagnostic::error(format!("Private identifier '#{name}' is not allowed in property names"))
            .with_label(span)
    };

    html_comment_in_module(span: Span) => {
        OxcDiagnostic::error("HTML comments are not allowed in modules").with_label(span)
    };

    merge_conflict_marker(start_span: Span, middle_span: Option<Span>, end_span: Option<Span>) => {
        let mut diagnostic = OxcDiagnostic::error("Encountered diff marker")
            .and_label(
                start_span.primary_label(
                    "between this marker and `=======` is the code that we're merging into",
                ),
            )
            .with_help(
                "Conflict markers indicate that a merge was started but could not be completed due to \
             merge conflicts.\n\
             To resolve a conflict, keep only the code you want and then delete the lines containing \
             conflict markers.\n\
             If you're having merge conflicts after pulling new code, the top section is the code you \
             already had and the bottom section is the remote code.\n\
             If you're in the middle of a rebase, the top section is the code being rebased onto and \
             the bottom section is the code coming from the current commit being rebased.\n\
             If you have nested conflicts, resolve the outermost conflict first.",
            );

        if let Some(middle) = middle_span {
            diagnostic = diagnostic
                .and_label(middle.label("between this marker and `>>>>>>>` is the incoming code"));
        } else {
            // Incomplete conflict - missing middle or end markers
            diagnostic = diagnostic.with_help(
                "This conflict marker appears to be incomplete (missing `=======` or `>>>>>>>`).\n\
             Check if the conflict markers were accidentally modified or partially deleted.",
            );
        }

        if let Some(end) = end_span {
            diagnostic = diagnostic.and_label(end.label("this marker concludes the conflict region"));
        }

        diagnostic
    };

    jsx_in_non_jsx(span: Span) => {
        OxcDiagnostic::error("Unexpected JSX expression")
            .with_label(span)
            .with_help("JSX syntax is disabled and should be enabled via the parser options")
    };

    expect_token(expected: &'a str, found: &'a str, span: Span) => {
        OxcDiagnostic::error(format!("Expected `{expected}` but found `{found}`"))
            .with_label(span.label(format!("`{expected}` expected")))
    };

    expect_closing(expected_closing: &'a str, actual: &'a str, span: Span, opening_span: Span) => {
        OxcDiagnostic::error(format!("Expected `{expected_closing}` but found `{actual}`")).with_labels(
            [
                span.primary_label(format!("`{expected_closing}` expected")),
                opening_span.label("Opened here"),
            ],
        )
    };

    expect_closing_or_separator(expected_closing: &'a str, expected_separator: &'a str, actual: &'a str, span: Span, opening_span: Span) => {
        OxcDiagnostic::error(format!(
            "Expected `{expected_separator}` or `{expected_closing}` but found `{actual}`"
        ))
        .with_labels([
            span.primary_label(format!("`{expected_separator}` or `{expected_closing}` expected")),
            opening_span.label("Opened here"),
        ])
    };

    expect_conditional_alternative(found: &'a str, span: Span, question_span: Span) => {
        OxcDiagnostic::error(format!("Expected `:` but found `{found}`")).with_labels([
            span.primary_label("`:` expected"),
            question_span.label("Conditional starts here"),
        ])
    };

    unexpected_trailing_comma(name: &'static str, span: Span) => {
        OxcDiagnostic::error(format!("{name} may not have a trailing comma."))
            .with_label(span)
            .with_help("Remove the trailing comma here")
    };

    invalid_escape_sequence(span: Span) => {
        OxcDiagnostic::error("Invalid escape sequence").with_label(span)
    };

    unicode_escape_sequence(span: Span) => {
        OxcDiagnostic::error("Invalid Unicode escape sequence").with_label(span)
    };

    invalid_character(c: char, span: Span) => {
        OxcDiagnostic::error(format!("Invalid Character `{c}`")).with_label(span)
    };

    invalid_number_end(span: Span) => {
        OxcDiagnostic::error("Invalid characters after number").with_label(span)
    };

    unterminated_multi_line_comment(span: Span) => {
        OxcDiagnostic::error("Unterminated multiline comment").with_label(span)
    };

    unterminated_string(span: Span) => {
        OxcDiagnostic::error("Unterminated string").with_label(span)
    };

    reg_exp_flag(flag: char, span: Span) => {
        OxcDiagnostic::error(format!("Unexpected flag {flag} in regular expression literal"))
            .with_label(span)
            .with_help(format!("The allowed flags are `{REGEXP_FLAGS_LIST}`"))
    };

    reg_exp_flag_twice(flag: char, span: Span) => {
        OxcDiagnostic::error(format!("Flag {flag} is mentioned twice in regular expression literal"))
            .with_label(span)
            .with_help("Remove the duplicated flag here")
    };

    unexpected_end(span: Span) => {
        OxcDiagnostic::error("Unexpected end of file").with_label(span)
    };

    unexpected_jsx_end(span: Span, ch: char, entity: &'a str) => {
        OxcDiagnostic::error(format!("Unexpected token. Did you mean `{{'{ch}'}}` or `&{entity};`?"))
            .with_label(span)
    };

    unterminated_reg_exp(span: Span) => {
        OxcDiagnostic::error("Unterminated regular expression").with_label(span)
    };

    invalid_number(number: &'a str, span: Span) => {
        OxcDiagnostic::error(format!("Invalid Number {number}")).with_label(span)
    };

    escaped_keyword(span: Span) => {
        OxcDiagnostic::error("Keywords cannot contain escape characters").with_label(span)
    };

    auto_semicolon_insertion(span: Span) => {
        OxcDiagnostic::error(
            "Expected a semicolon or an implicit semicolon after a statement, but found none",
        )
        .with_help("Try inserting a semicolon here")
        .with_label(span)
    };

    lineterminator_before_arrow(span: Span) => {
        OxcDiagnostic::error("Line terminator not permitted before arrow")
            .with_label(span)
            .with_help("Remove the line break before here")
    };

    invalid_destructuring_declaration(span: Span) => {
        OxcDiagnostic::error("Missing initializer in destructuring declaration")
            .with_label(span)
            .with_help("Add an initializer (e.g. ` = undefined`) here")
    };

    missing_initializer_in_const(span: Span) => {
        OxcDiagnostic::error("Missing initializer in const declaration")
            .with_label(span)
            .with_help("Add an initializer (e.g. ` = undefined`) here")
    };

    lexical_declaration_single_statement(span: Span) => {
        OxcDiagnostic::error("Lexical declaration cannot appear in a single-statement context")
            .with_help("Wrap this declaration in a block statement")
            .with_label(span)
    };

    export_assignment_in_namespace(span: Span) => {
        ts_error("1063", "An export assignment cannot be used in a namespace.").with_label(span)
    };

    import_in_namespace(span: Span) => {
        ts_error("1147", "Import declarations in a namespace cannot reference a module.")
            .with_label(span)
    };

    export_in_namespace(span: Span) => {
        ts_error("1194", "Export declarations are not permitted in a namespace.").with_label(span)
    };

    default_export_in_namespace(span: Span) => {
        ts_error("1319", "A default export can only be used in an ECMAScript-style module.")
            .with_label(span)
    };

    global_export_in_namespace(span: Span) => {
        ts_error("1316", "Global module exports may only appear at top level.").with_label(span)
    };

    statement_in_ambient_context(span: Span) => {
        ts_error("1036", "Statements are not allowed in ambient contexts.").with_label(span)
    };

    declare_in_ambient_context(span: Span) => {
        ts_error("1038", "A 'declare' modifier cannot be used in an already ambient context.")
            .with_label(span)
    };

    declaration_single_statement(span: Span) => {
        OxcDiagnostic::error("Declaration cannot appear in a single-statement context")
            .with_help("Wrap this declaration in a block statement")
            .with_label(span)
    };

    const_type_parameter(span: Span) => {
        ts_error(
            "1277",
            "'const' modifier can only appear on a type parameter of a function, method or class",
        )
        .with_label(span)
    };

    async_function_declaration(span: Span) => {
        OxcDiagnostic::error("Async functions can only be declared at the top level or inside a block")
            .with_label(span)
    };

    generator_function_declaration(span: Span) => {
        OxcDiagnostic::error("Generators can only be declared at the top level or inside a block")
            .with_label(span)
    };

    await_expression(span: Span) => {
        OxcDiagnostic::error(
            "`await` is only allowed within async functions and at the top levels of modules",
        )
        .with_label(span)
        .with_help("Either remove this `await` or add the `async` keyword to the enclosing function")
    };

    for_await_statement(span: Span) => {
        OxcDiagnostic::error(
            "`for await` loops are only allowed within async functions and at the top levels of modules",
        )
        .with_label(span)
        .with_help("Either remove this `await` or add the `async` keyword to the enclosing function")
    };

    yield_expression(span: Span) => {
        OxcDiagnostic::error("A 'yield' expression is only allowed in a generator body.")
            .with_label(span)
            .with_help("Either remove this `yield` or change the enclosing function to a generator function (`function*`)")
    };

    class_declaration(span: Span) => {
        OxcDiagnostic::error("Invalid class declaration")
            .with_help("Classes can only be declared at top level or inside a block")
            .with_label(span)
    };

    // 'extends' clause already seen. ts(1172)
    extends_clause_already_seen(span: Span) => {
        ts_error("1172", "'extends' clause already seen").with_label(span)
    };

    // 'extends' clause must precede 'implements' clause. ts(1173)
    extends_clause_must_precede_implements(span: Span, implements_span: Span) => {
        ts_error("1173", "'extends' clause must precede 'implements' clause")
            .with_labels([
                implements_span.label("'implements' clause found here"),
                span.primary_label("'extends' clause found here"),
            ])
            .with_help("Move the 'extends' clause before the 'implements' clause")
    };

    // Classes can only extend a single class. ts(1174)
    classes_can_only_extend_single_class(span: Span) => {
        ts_error("1174", "Classes can only extend a single class.")
            .with_label(span)
            .with_help("Remove the extra base class or use interfaces for multiple inheritance")
    };

    // 'implements' clause already seen. ts(1175)
    implements_clause_already_seen(span: Span, seen_span: Span) => {
        ts_error("1175", "'implements' clause already seen")
            .with_labels([seen_span, span])
            .with_help("Merge the two 'implements' clauses into one by a ','")
    };

    // A class member cannot have the 'const' keyword. ts(1248)
    const_class_member(span: Span) => {
        ts_error("1248", "A class member cannot have the 'const' keyword.")
            .with_help("Did you mean `readonly`?")
            .with_label(span)
    };

    // A required element cannot follow an optional element. ts(1257)
    required_element_cannot_follow_optional_element(span: Span, optional_span: Span) => {
        ts_error("1257", "A required element cannot follow an optional element.").with_labels([
            span.label("Required element here"),
            optional_span.label("Optional element seen here"),
        ])
    };

    // A rest element cannot follow another rest element. ts(1265)
    rest_element_cannot_follow_another_rest_element(seen_span: Span, span: Span) => {
        ts_error("1265", "A rest element cannot follow another rest element.")
            .with_labels([span.label("Second rest element here"), seen_span.label("First seen here")])
    };

    // An optional element cannot follow a rest element. ts(1266)
    optional_element_cannot_follow_rest_element(span: Span, rest_span: Span) => {
        ts_error("1266", "An optional element cannot follow a rest element.").with_labels([
            span.label("Optional element here"),
            rest_span.label("Rest element seen here"),
        ])
    };

    // A type-only import can specify a default import or named bindings, but not both. ts(1363)
    type_only_import_default_and_named(specifier_span: Span) => {
        ts_error(
            "1363",
            "A type-only import can specify a default import or named bindings, but not both.",
        )
        .with_label(specifier_span)
    };

    binding_rest_element_last(span: Span) => {
        OxcDiagnostic::error("A rest element must be last in a destructuring pattern").with_label(span)
    };

    rest_parameter_last(span: Span) => {
        OxcDiagnostic::error("A rest parameter must be last in a parameter list").with_label(span)
    };

    spread_last_element(span: Span) => {
        OxcDiagnostic::error("Spread must be last element").with_label(span)
    };

    invalid_binding_rest_element(span: Span) => {
        OxcDiagnostic::error("Invalid rest element target in destructuring pattern")
            .with_help("Expected an identifier, like `...rest`.")
            .with_label(span)
    };

    a_rest_parameter_cannot_be_optional(span: Span) => {
        ts_error("1047", "A rest parameter cannot be optional")
            .with_label(span)
            .with_help("Remove this `?`. The default value is an empty array")
    };

    invalid_assignment(span: Span) => {
        OxcDiagnostic::error("Cannot assign to this expression").with_label(span)
    };

    assignment_is_not_simple(span: Span) => {
        OxcDiagnostic::error("Invalid left-hand side in assignment").with_label(span)
    };

    invalid_lhs_assignment(span: Span) => {
        OxcDiagnostic::error(
            "The left-hand side of an assignment expression must be a variable or a property access.",
        )
        .with_label(span)
    };

    new_optional_chain(span: Span) => {
        OxcDiagnostic::error("Optional chaining cannot appear in the callee of new expressions")
            .with_label(span)
    };

    invalid_new_optional_chain(span: Span) => {
        OxcDiagnostic::error("Invalid optional chain from new expression.").with_label(span)
    };

    decorator_optional(span: Span) => {
        OxcDiagnostic::error("Expression must be enclosed in parentheses to be used as a decorator.")
            .with_label(span)
    };

    for_loop_async_of(span: Span) => {
        OxcDiagnostic::error("The left-hand side of a `for...of` statement may not be `async`")
            .with_label(span)
            .with_help("Did you mean to use a for await...of statement?")
    };

    for_loop_let_reserved_word(span: Span) => {
        OxcDiagnostic::error("The left-hand side of a `for...of` statement may not start with `let`")
            .with_label(span)
    };

    for_await(span: Span) => {
        OxcDiagnostic::error("await can only be used in conjunction with `for...of` statements")
            .with_label(span)
            .with_help("Did you mean to use a for...of statement?")
    };

    new_dynamic_import(span: Span) => {
        OxcDiagnostic::error("Cannot use new with dynamic import")
            .with_label(span)
            .with_help("Wrap this with parenthesis")
    };

    new_super(span: Span) => {
        OxcDiagnostic::error("'new super()' is not allowed").with_label(span)
    };

    private_name_constructor(span: Span) => {
        OxcDiagnostic::error("Classes can't have an element named '#constructor'").with_label(span)
    };

    static_prototype(span: Span) => {
        OxcDiagnostic::error("Classes may not have a static property named 'prototype'")
            .with_label(span)
    };

    constructor_getter_setter(span: Span) => {
        OxcDiagnostic::error("Constructor can't have get/set modifier").with_label(span)
    };

    constructor_async(span: Span) => {
        OxcDiagnostic::error("Constructor can't be an async method").with_label(span)
    };

    optional_accessor_property(span: Span) => {
        ts_error("1276", "An 'accessor' property cannot be declared optional.")
            .with_label(span)
            .with_help("Remove this `?`")
    };

    constructor_accessor(span: Span) => {
        OxcDiagnostic::error("Classes may not have a field named 'constructor'").with_label(span)
    };

    optional_definite_property(span: Span) => {
        // NOTE: could not find an error code when tsc parses this; its parser panics.
        OxcDiagnostic::error("A property cannot be both optional and definite.")
            .with_label(span)
            .with_help("Remove either the `?` or the `!`")
    };

    definite_assignment_assertion_not_permitted(span: Span) => {
        ts_error("1255", "A definite assignment assertion '!' is not permitted in this context.")
            .with_label(span)
    };

    identifier_async(name: &'a str, span: Span) => {
        OxcDiagnostic::error(format!("Cannot use `{name}` as an identifier in an async context"))
            .with_label(span)
    };

    identifier_generator(name: &'a str, span: Span, looks_like_expression: bool) => {
        let diagnostic = OxcDiagnostic::error(format!(
            "Cannot use `{name}` as an identifier in a generator context"
        ))
        .with_label(span);
        if looks_like_expression {
            diagnostic.with_help(format!(
                "Wrap this in parentheses if you want to use a `{name}` expression here"
            ))
        } else {
            diagnostic
        }
    };

    identifier_expected(span: Span) => {
        OxcDiagnostic::error("Identifier expected.").with_label(span)
    };

    identifier_reserved_word(span: Span, reserved: &'a str) => {
        OxcDiagnostic::error(format!(
            "Identifier expected. '{reserved}' is a reserved word that cannot be used here."
        ))
        .with_label(span)
    };

    constructor_generator(span: Span) => {
        OxcDiagnostic::error("Constructor can't be a generator").with_label(span)
    };

    declare_constructor(span: Span) => {
        ts_error("1031", "'declare' modifier cannot appear on a constructor declaration.")
            .with_label(span)
    };

    constructor_return_type(span: Span) => {
        ts_error("1093", "Type annotation cannot appear on a constructor declaration.").with_label(span)
    };

    field_constructor(span: Span) => {
        OxcDiagnostic::error("Classes can't have a field named 'constructor'").with_label(span)
    };

    export_lone_surrogate(span: Span) => {
        OxcDiagnostic::error("An export name cannot include a unicode lone surrogate").with_label(span)
    };

    export_named_string(local: &'a str, exported: &'a str, span: Span) => {
        OxcDiagnostic::error("A string literal cannot be used as an exported binding without `from`")
            .with_help(format!("Did you mean `export {{ {local} as {exported} }} from 'some-module'`?"))
            .with_label(span)
    };

    export_reserved_word(local: &'a str, exported: &'a str, span: Span) => {
        OxcDiagnostic::error("A reserved word cannot be used as an exported binding without `from`")
            .with_help(format!("Did you mean `export {{ {local} as {exported} }} from 'some-module'`?"))
            .with_label(span)
    };

    template_literal(span: Span) => {
        OxcDiagnostic::error("Bad escape sequence in untagged template literal").with_label(span)
    };

    empty_parenthesized_expression(span: Span) => {
        OxcDiagnostic::error("Empty parenthesized expression").with_label(span)
    };

    illegal_newline(token: &'a str, token_span: Span, newline_span: Span) => {
        OxcDiagnostic::error(format!("Illegal newline after {token}")).with_labels([
            token_span.label(format!("{token} starts here")),
            newline_span.label("A newline is not expected here"),
        ])
    };

    optional_chain_tagged_template(span: Span) => {
        OxcDiagnostic::error("Tagged template expressions are not permitted in an optional chain")
            .with_label(span)
    };

    ts_constructor_this_parameter(span: Span) => {
        ts_error("2681", "A constructor cannot have a `this` parameter.").with_label(span)
    };

    ts_constructor_type_parameter(span: Span) => {
        ts_error("1092", "Type parameters cannot appear on a constructor declaration")
            .with_label(span)
            .with_help("Instead, add type parameters to the class itself")
    };

    ts_arrow_function_this_parameter(span: Span) => {
        ts_error("2730", "An arrow function cannot have a `this` parameter.")
            .with_label(span)
            .with_help("Arrow function does not bind `this` and inherits `this` from the outer scope")
    };

    ts_empty_type_parameter_list(span: Span) => {
        ts_error("1098", "Type parameter list cannot be empty.").with_label(span)
    };

    ts_empty_type_argument_list(span: Span) => {
        ts_error("1099", "Type argument list cannot be empty.").with_label(span)
    };

    ts_instantiation_expression_cannot_be_followed_by_property_access(span: Span) => {
        ts_error("1477", "An instantiation expression cannot be followed by a property access.")
            .with_label(span)
    };

    ts_string_literal_expected(span: Span) => {
        ts_error("1141", "String literal expected.").with_label(span)
    };

    unexpected_super(span: Span) => {
        OxcDiagnostic::error("'super' can only be used with function calls or in property accesses")
            .with_help("Replace with `super()` or `super.prop` or `super[prop]`")
            .with_label(span)
    };

    super_private(span: Span) => {
        OxcDiagnostic::error("Private fields cannot be accessed on super").with_label(span)
    };

    expect_function_name(span: Span) => {
        OxcDiagnostic::error("Expected function name")
            .with_help("Function name is required in function declaration or named export")
            .with_label(span)
    };

    expect_catch_finally(span: Span) => {
        OxcDiagnostic::error("Missing catch or finally clause")
            .with_label(span)
            .with_help("Either unwrap this try block or add catch / finally clause")
    };

    v8_intrinsic_spread_elem(span: Span) => {
        OxcDiagnostic::error("V8 runtime calls cannot have spread elements as arguments")
            .with_label(span)
    };

    a_set_accessor_cannot_have_a_return_type_annotation(span: Span) => {
        ts_error("1095", "A 'set' accessor cannot have a return type annotation.").with_label(span)
    };

    accessor_cannot_have_type_parameters(span: Span) => {
        ts_error("1094", "An accessor cannot have type parameters.").with_label(span)
    };

    return_statement_only_in_function_body(span: Span) => {
        ts_error("1108", "A 'return' statement can only be used within a function body.")
            .with_label(span)
    };

    return_statement_in_class_static_block(span: Span) => {
        ts_error("18041", "A 'return' statement cannot be used inside a class static block.")
            .with_label(span)
    };

    invalid_identifier_in_using_declaration(span: Span) => {
        OxcDiagnostic::error("Using declarations may not have binding patterns.").with_label(span)
    };

    await_using_declaration_not_allowed_in_for_in_statement(span: Span) => {
        OxcDiagnostic::error(
            "The left-hand side of a for...in statement cannot be an await using declaration.",
        )
        .with_label(span)
        .with_help("Did you mean to use a for...of statement?")
    };

    using_declaration_not_allowed_in_for_in_statement(span: Span) => {
        OxcDiagnostic::error(
            "The left-hand side of a for...in statement cannot be an using declaration.",
        )
        .with_label(span)
        .with_help("Did you mean to use a for...of statement?")
    };

    using_declarations_must_be_initialized(span: Span) => {
        OxcDiagnostic::error("Using declarations must have an initializer.")
            .with_label(span)
            .with_help("Add an initializer (e.g. ` = undefined`) here")
    };

    using_declaration_cannot_be_exported(identifier: &'a str, span: Span) => {
        OxcDiagnostic::error("Using declarations cannot be exported directly.")
            .with_label(span)
            .with_help(format!("Remove the `export` here and add `export {{ {identifier} }}` as a separate statement to export the declaration"))
    };

    using_declaration_not_allowed_in_switch_bare_case(span: Span) => {
        OxcDiagnostic::error("'using' declaration cannot appear in the bare case statement.")
            .with_label(span)
            .with_help("Wrap this declaration in a block statement")
    };

    await_using_declaration_not_allowed_in_switch_bare_case(span: Span) => {
        OxcDiagnostic::error("'await using' declaration cannot appear in the bare case statement.")
            .with_label(span)
            .with_help("Wrap this declaration in a block statement")
    };

    using_declarations_not_allowed_in_ambient_contexts(span: Span) => {
        ts_error("1545", "'using' declarations are not allowed in ambient contexts.").with_label(span)
    };

    await_using_declarations_not_allowed_in_ambient_contexts(span: Span) => {
        ts_error("1546", "'await using' declarations are not allowed in ambient contexts.")
            .with_label(span)
    };

    jsx_element_no_match(opening_span: Span, closing_span: Span, name: &'a str) => {
        OxcDiagnostic::error(format!("Expected corresponding JSX closing tag for '{name}'."))
            .with_labels([
                closing_span.primary_label(format!("Expected `</{name}>`")),
                opening_span.label("Opened here"),
            ])
    };

    jsx_fragment_no_match(opening_span: Span, closing_span: Span) => {
        OxcDiagnostic::error("Expected corresponding closing tag for JSX fragment.").with_labels([
            closing_span.primary_label("Expected `</>`"),
            opening_span.label("Opened here"),
        ])
    };

    adjacent_jsx_elements(span: Span) => {
        OxcDiagnostic::error("Adjacent JSX elements must be wrapped in an enclosing tag.")
            .with_help("Did you want a JSX fragment `<>...</>`?")
            .with_label(span)
    };

    cover_initialized_name(span: Span) => {
        OxcDiagnostic::error("Invalid assignment in object literal")
    .with_help("Did you mean to use a ':'? An '=' can only follow a property name when the containing object literal is part of a destructuring pattern.")
    .with_label(span)
    };

    invalid_import_property(span: Span) => {
        OxcDiagnostic::error(
            "The only valid property accesses on import are `import.meta`, `import.source()`, and `import.defer()`",
        )
        .with_label(span)
    };

    new_target(span: Span) => {
        OxcDiagnostic::error("The only valid meta property for new is new.target").with_label(span)
    };

    new_target_outside_function(span: Span) => {
        OxcDiagnostic::error("Unexpected new.target expression")
            .with_help(
                "new.target is only allowed in constructors, functions, and class field initializers",
            )
            .with_label(span)
    };

    switch_multiple_default_clause(first_default: Span, other_default: Span) => {
        ts_error("1113", "A 'default' clause cannot appear more than once in a 'switch' statement.")
            .with_labels([
                first_default.label("First 'default' clause is here."),
                other_default.label("Another 'default' clause cannot appear here."),
            ])
    };

    import_meta(span: Span) => {
        OxcDiagnostic::error("Unexpected import.meta expression")
            .with_help("import.meta is only allowed in module code")
            .with_label(span)
    };

    private_in_private(span: Span) => {
        OxcDiagnostic::error("Unexpected right-hand side of private-in expression").with_label(span)
    };

    unexpected_private_identifier(span: Span) => {
        OxcDiagnostic::error("Unexpected private identifier").with_label(span)
    };

    import_arguments(span: Span) => {
        OxcDiagnostic::error("Dynamic imports can only accept a module specifier and an optional set of attributes as arguments").with_label(span)
    };

    dynamic_import_argument_spread(span: Span) => {
        ts_error("1325", "Argument of dynamic import cannot be a spread element.").with_label(span)
    };

    rest_element_property_name(span: Span) => {
        ts_error("2566", "A rest element cannot have a property name.").with_label(span)
    };

    a_rest_element_cannot_have_an_initializer(span: Span) => {
        OxcDiagnostic::error("A rest element cannot have an initializer.").with_label(span)
    };

    a_rest_parameter_cannot_have_an_initializer(span: Span) => {
        OxcDiagnostic::error("A rest parameter cannot have an initializer.").with_label(span)
    };

    import_requires_a_specifier(span: Span) => {
        OxcDiagnostic::error("import() requires a specifier.").with_label(span)
    };

    modifier_cannot_be_used_here(modifier: Modifier, allowed: Option<ModifierKinds>) => {
        OxcDiagnostic::error(format!("'{}' modifier cannot be used here.", modifier.kind))
            .with_label(modifier.span())
            .with_allowed_modifier_help(allowed)
    };

    modifier_only_on_property_declaration_or_index_signature(modifier: Modifier, allowed: Option<ModifierKinds>) => {
        ts_error(
            "1024",
            format!(
                "'{}' modifier can only appear on a property declaration or index signature.",
                modifier.kind
            ),
        )
        .with_label(modifier.span())
        .with_allowed_modifier_help(allowed)
    };

    accessibility_modifier_already_seen(modifier: Modifier) => {
        ts_error("1028", "Accessibility modifier already seen.")
            .with_label(modifier.span())
            .with_help("Remove the duplicate modifier.")
    };

    modifier_must_precede_other_modifier(modifier: Modifier, other_modifier: ModifierKind) => {
        ts_error(
            "1029",
            format!("'{}' modifier must precede '{}' modifier.", modifier.kind, other_modifier),
        )
        .with_label(modifier.span())
    };

    modifier_cannot_be_used_with_other_modifier(span: Span, modifier: ModifierKind, other_modifier: ModifierKind) => {
        ts_error(
            "1243",
            format!("'{modifier}' modifier cannot be used with '{other_modifier}' modifier."),
        )
        .with_label(span)
    };

    modifier_cannot_be_used_in_ambient_context(span: Span, modifier: ModifierKind) => {
        ts_error("1040", format!("'{modifier}' modifier cannot be used in an ambient context."))
            .with_label(span)
    };

    modifier_already_seen(modifier: Modifier) => {
        ts_error("1030", format!("'{}' modifier already seen.", modifier.kind))
            .with_label(modifier.span())
            .with_help("Remove the duplicate modifier.")
    };

    cannot_appear_on_class_elements(modifier: Modifier, allowed: Option<ModifierKinds>) => {
        ts_error(
            "1031",
            format!("'{}' modifier cannot appear on class elements of this kind.", modifier.kind),
        )
        .with_label(modifier.span())
        .with_allowed_modifier_help(allowed)
    };

    cannot_appear_on_a_type_member(modifier: Modifier, allowed: Option<ModifierKinds>) => {
        ts_error("1070", format!("'{}' modifier cannot appear on a type member.", modifier.kind))
            .with_label(modifier.span())
            .with_allowed_modifier_help(allowed)
    };

    cannot_appear_on_a_type_parameter(modifier: Modifier, allowed: Option<ModifierKinds>) => {
        ts_error("1273", format!("'{}' modifier cannot be used on a type parameter.", modifier.kind))
            .with_label(modifier.span())
            .with_allowed_modifier_help(allowed)
    };

    a_parameter_cannot_have_question_mark_and_initializer(span: Span) => {
        ts_error("1015", "A parameter cannot have a question mark and an initializer.").with_label(span)
    };

    can_only_appear_on_a_type_parameter_of_a_class_interface_or_type_alias(modifier: ModifierKind, span: Span) => {
        ts_error("1274", format!("'{modifier}' modifier can only appear on a type parameter of a class, interface or type alias."))
            .with_label(span)
    };

    cannot_appear_on_a_parameter(modifier: Modifier, allowed: Option<ModifierKinds>) => {
        ts_error("1090", format!("'{}' modifier cannot appear on a parameter.", modifier.kind))
            .with_label(modifier.span())
            .with_allowed_modifier_help(allowed)
    };

    parameter_property_cannot_be_binding_pattern(span: Span) => {
        ts_error("1187", "A parameter property may not be declared using a binding pattern.")
            .with_label(span)
    };

    constructor_cannot_be_parameter_property_name(span: Span) => {
        ts_error("2398", "'constructor' cannot be used as a parameter property name.").with_label(span)
    };

    cannot_appear_on_an_index_signature(modifier: Modifier, allowed: Option<ModifierKinds>) => {
        ts_error("1071", format!("'{}' modifier cannot appear on an index signature.", modifier.kind))
            .with_label(modifier.span())
            .with_allowed_modifier_help(allowed)
    };

    accessor_modifier(modifier: Modifier, allowed: Option<ModifierKinds>) => {
        ts_error(
            "1243",
            format!("'accessor' modifier cannot be used with '{}' modifier.", modifier.kind),
        )
        .with_label(modifier.span())
        .with_allowed_modifier_help(allowed.map(|a| a.without(ModifierKind::Accessor)))
    };

    readonly_in_array_or_tuple_type(span: Span) => {
        ts_error("1354", "'readonly' type modifier is only permitted on array and tuple literal types.")
            .with_label(span)
    };

    accessibility_modifier_on_private_property(modifier: Modifier, _allowed: Option<ModifierKinds>) => {
        ts_error("18010", "An accessibility modifier cannot be used with a private identifier.")
            .with_label(modifier.span())
            .with_help("Private identifiers are enforced at runtime, while accessibility modifiers only affect type checking, so using both is redundant.")
    };

    type_modifier_on_named_type_import(span: Span) => {
        ts_error("2206", "The 'type' modifier cannot be used on a named import when 'import type' is used on its import statement.")
            .with_label(span)
            .with_help("Remove this 'type' modifier")
    };

    type_modifier_on_named_type_export(span: Span) => {
        ts_error("2207", "The 'type' modifier cannot be used on a named export when 'export type' is used on its export statement.")
             .with_label(span)
             .with_help("Remove this 'type' modifier")
    };

    computed_property_names_not_allowed_in_enums(span: Span) => {
        ts_error("1164", "Computed property names are not allowed in enums.").with_label(span)
    };

    enum_member_cannot_have_numeric_name(span: Span) => {
        ts_error("2452", "An enum member cannot have a numeric name.").with_label(span)
    };

    index_signature_one_parameter(span: Span) => {
        ts_error("1096", "An index signature must have exactly one parameter.").with_label(span)
    };

    index_signature_parameter_type(span: Span) => {
        ts_error(
            "1268",
            "An index signature parameter type must be 'string', 'number', 'symbol', or a template \
             literal type.",
        )
        .with_label(span)
    };

    index_signature_parameter_literal_type(span: Span) => {
        ts_error(
            "1337",
            "An index signature parameter type cannot be a literal type or generic type. Consider \
             using a mapped object type instead.",
        )
        .with_label(span)
    };

    mixed_coalesce(span: Span) => {
        OxcDiagnostic::error("Logical expressions and coalesce expressions cannot be mixed")
            .with_help("Wrap either expression by parentheses")
            .with_label(span)
    };

    unary_exponentiation_left_operand(operator: &'a str, span: Span) => {
        OxcDiagnostic::error(format!(
            "A unary expression with the '{operator}' operator cannot be used as the left operand of an exponentiation expression"
        ))
        .with_help("Wrap the unary expression in parentheses to make the precedence explicit")
        .with_label(span)
    };

    type_assertion_exponentiation_left_operand(span: Span) => {
        OxcDiagnostic::error(
            "A type assertion cannot be used as the left operand of an exponentiation expression",
        )
        .with_help("Wrap the type assertion in parentheses to make the precedence explicit")
        .with_label(span)
    };

    import_equals_can_only_be_used_in_typescript_files(span: Span) => {
        ts_error("8002", "'import ... =' can only be used in TypeScript files.")
            .with_label(span)
            .with_help("TypeScript transforms 'import ... =' to 'const ... ='")
    };

    index_signature_question_mark(span: Span) => {
        ts_error("1019", "An index signature parameter cannot have a question mark.").with_label(span)
    };

    index_signature_type_annotation(span: Span) => {
        ts_error("1021", "An index signature must have a type annotation.").with_label(span)
    };

    abstract_method_cannot_have_implementation(name: &'a str, span: Span) => {
        ts_error(
            "1245",
            format!("Method '{name}' cannot have an implementation because it is marked abstract."),
        )
        .with_label(span)
    };

    abstract_property_cannot_have_initializer(name: &'a str, span: Span) => {
        ts_error(
            "1267",
            format!("Property '{name}' cannot have an initializer because it is marked abstract."),
        )
        .with_label(span)
    };

    abstract_with_private_identifier(span: Span) => {
        ts_error("18019", "'abstract' modifier cannot be used with a private identifier.")
            .with_label(span)
    };

    required_parameter_after_optional_parameter(span: Span) => {
        ts_error("1016", "A required parameter cannot follow an optional parameter.").with_label(span)
    };

    jsx_expressions_may_not_use_the_comma_operator(span: Span) => {
        ts_error("18007", "JSX expressions may not use the comma operator")
            .with_help("Did you mean to write an array?")
            .with_label(span)
    };

    import_alias_cannot_use_import_type(span: Span) => {
        ts_error("1392", "An import alias cannot use 'import type'").with_label(span)
    };

    reserved_type_name(span: Span, reserved_name: &'a str, syntax_name: &'a str) => {
        let code = match syntax_name {
            "Type parameter" => "2368",
            "Interface" => "2427",
            "Enum" => "2431",
            "Type alias" => "2457",
            // "Class" and any other declaration form
            _ => "2414",
        };
        ts_error(code, format!("{syntax_name} name cannot be '{reserved_name}'")).with_label(span)
    };

    abstract_accessor_cannot_have_implementation(name: &'a str, span: Span) => {
        ts_error(
            "1318",
            format!("Accessor '{name}' cannot have an implementation because it is marked abstract."),
        )
        .with_label(span)
    };

    unexpected_export(span: Span) => {
        OxcDiagnostic::error("Unexpected export.").with_label(span)
    };

    decorators_in_export_and_class(span: Span) => {
        OxcDiagnostic::error("Decorators may not appear after 'export' or 'export default' if they also appear before 'export'.").with_label(span)
    };

    decorators_are_not_valid_here(span: Span) => {
        OxcDiagnostic::error("Decorators are not valid here.").with_label(span)
    };

    decorator_on_overload(span: Span) => {
        ts_error("1249", "A decorator can only decorate a method implementation, not an overload.")
            .with_label(span)
            .with_help("Move this after all the overloads")
    };

    type_arguments_in_ts(span: Span) => {
        ts_error("8011", "Type arguments can only be used in TypeScript files.")
            .with_label(span)
    };

    as_in_ts(span: Span) => {
        ts_error("8016", "Type assertion expressions can only be used in TypeScript files.")
            .with_label(span)
    };

    satisfies_in_ts(span: Span) => {
        ts_error("8037", "Type satisfaction expressions can only be used in TypeScript files.")
            .with_label(span)
    };

    optional_and_rest_tuple_member(span: Span) => {
        ts_error("5085", "A tuple member cannot be both optional and rest.").with_label(span)
    };

    optional_after_tuple_member_name(span: Span) => {
        ts_error("5086", "A labeled tuple element is declared as optional with a question mark after the name and before the colon, rather than after the type.").with_label(span)
    };

    rest_after_tuple_member_name(span: Span) => {
        ts_error("5087", "A labeled tuple element is declared as rest with a '...' before the name, rather than before the type.").with_label(span)
    };

    parameter_modifiers_in_ts(modifier: Modifier, allowed: Option<ModifierKinds>) => {
        ts_error("8012", "Parameter modifiers can only be used in TypeScript files.")
            .with_label(modifier.span())
            .with_allowed_modifier_help(allowed)
    };

    implements_clause_in_ts(span: Span) => {
        ts_error("8005", "'implements' clauses can only be used in TypeScript files.").with_label(span)
    };

    implementation_in_ambient(span: Span) => {
        ts_error("1183", "An implementation cannot be declared in ambient contexts.").with_label(span)
    };

    generator_in_ambient_context(span: Span) => {
        ts_error("1221", "Generators are not allowed in an ambient context.").with_label(span)
    };

    overload_signature_generator(span: Span) => {
        ts_error("1222", "An overload signature cannot be declared as a generator.").with_label(span)
    };

    initializers_not_allowed_in_ambient_contexts(span: Span) => {
        ts_error("1039", "Initializers are not allowed in ambient contexts.").with_label(span)
    };

    interface_implements(span: Span) => {
        ts_error("1176", "Interface declaration cannot have 'implements' clause.").with_label(span)
    };

    interface_extend(span: Span) => {
        ts_error(
            "2499",
            "An interface can only extend an identifier/qualified-name with optional type arguments.",
        )
        .with_label(span)
    };

    reg_exp_flag_u_and_v(span: Span) => {
        ts_error("1502", "The 'u' and 'v' regular expression flags cannot be enabled at the same time")
            .with_label(span)
            .with_help("v flag enables additional syntax over u flag")
    };

    setter_with_parameters(span: Span, parameters_count: usize) => {
        ts_error("1049", "A 'set' accessor must have exactly one parameter.")
            .with_label(span)
            .with_help(if parameters_count == 0 {
                "Add a parameter here"
            } else {
                "Remove parameters except the first one here"
            })
    };

    setter_with_rest_parameter(span: Span) => {
        OxcDiagnostic::error("A 'set' accessor cannot have rest parameter.").with_label(span)
    };

    setter_with_initializer(span: Span) => {
        OxcDiagnostic::error("A 'set' accessor cannot have an initializer.").with_label(span)
    };

    getter_parameters(span: Span) => {
        OxcDiagnostic::error("A 'get' accessor must not have any formal parameters.")
            .with_label(span)
            .with_help("Remove these parameters here")
    };

    setter_with_optional_parameter(span: Span) => {
        ts_error("1051", "A 'set' accessor cannot have an optional parameter.").with_label(span)
    };

    accessor_cannot_have_this_parameter(span: Span) => {
        ts_error("2784", "'get' and 'set' accessors cannot declare 'this' parameters.").with_label(span)
    };

    variable_declarator_definite(span: Span) => {
        ts_error(
            "1263",
            "Declarations with initializers cannot also have definite assignment assertions.",
        )
        .with_label(span)
    };

    variable_declarator_definite_type_assertion(span: Span) => {
        ts_error(
            "1264",
            "Declarations with definite assignment assertions must also have type annotations.",
        )
        .with_label(span)
    };

    invalid_assignment_target_default_value_operator(span: Span) => {
        OxcDiagnostic::error("Only '=' operator can be used for specifying default value.")
            .with_label(span)
            .with_help("Replace this operator with `=`.")
    };

    invalid_rest_assignment_target(span: Span) => {
        OxcDiagnostic::error("Invalid rest element target in destructuring assignment")
            .with_help("Expected an identifier or member expression, like `...rest` or `...obj.prop`.")
            .with_label(span)
    };

    modifiers_cannot_appear_here(modifier: Modifier, _unused: Option<ModifierKinds>) => {
        ts_error("1184", "Modifiers cannot appear here.").with_label(modifier.span())
    };

    expect_function_body(span: Span) => {
        OxcDiagnostic::error("Expected function body")
            .with_label(span)
            .with_help("Add a function body (`{}`).")
    };

    expect_switch_clause(span: Span) => {
        OxcDiagnostic::error("Expected switch clause")
            .with_label(span.label("`case` or `default` clause expected here"))
            .with_help("If this is intended to be the condition for the switch statement, add `case` before it.")
    };

    unexpected_optional_declaration(span: Span) => {
        OxcDiagnostic::error("Optional declaration is not allowed here")
            .with_label(span)
            .with_help("Remove the `?`")
    };

    identifier_expected_after_question_dot(span: Span) => {
        OxcDiagnostic::error("Identifier expected after '?.'")
            .with_label(span)
            .with_help("Add an identifier after '?.'")
    };

    identifier_expected_jsx_no_hyphen(span: Span) => {
        OxcDiagnostic::error("Identifiers in JSX cannot contain hyphens")
            .with_label(span)
            .with_help("Remove the hyphen from the identifier")
    };

    jsx_attribute_value_empty_expression(span: Span) => {
        ts_error("17000", "JSX attributes must only be assigned a non-empty 'expression'.")
            .with_label(span)
    };

    import_attribute_value_must_be_string_literal(span: Span) => {
        OxcDiagnostic::error("Only string literals are allowed as module attribute values.")
            .with_label(span)
            .with_help("Wrap this with quotes")
    };

    // TS18058
    default_import_not_allowed_in_defer(span: Span) => {
        ts_error("18058", "Default imports are not allowed in a deferred import.").with_label(span)
    };

    // TS18059
    named_import_not_allowed_in_defer(span: Span) => {
        ts_error("18059", "Named imports are not allowed in a deferred import.").with_label(span)
    };

    only_default_import_allowed_in_source_phase(span: Span) => {
        OxcDiagnostic::error("Only a single default import is allowed in a source phase import.")
            .with_label(span)
    };

    ts_import_type_options_expected_with(span: Span) => {
        OxcDiagnostic::error("Expected 'with' in import type options").with_label(span)
    };

    ts_import_type_options_invalid_key(span: Span) => {
        OxcDiagnostic::error("Import attributes keys must be identifier or string literal.")
            .with_label(span)
    };

    ts_import_type_options_no_spread(span: Span) => {
        OxcDiagnostic::error("Spread elements are not allowed in import type options.").with_label(span)
    };

    // This syntax is reserved in files with the .mts or .cts extension. Use an `as` expression instead. ts(7059)
    jsx_type_assertion_in_mts_cts(span: Span) => {
        ts_error(
            "7059",
            "This syntax is reserved in files with the .mts or .cts extension. Use an `as` expression instead.",
        )
        .with_label(span)
    };

    // This syntax is reserved in files with the .mts or .cts extension. Add a trailing comma or explicit constraint. ts(7060)
    jsx_type_parameter_in_mts_cts(span: Span) => {
        ts_error(
            "7060",
            "This syntax is reserved in files with the .mts or .cts extension. Add a trailing comma or explicit constraint.",
        )
        .with_label(span)
    };
}

/// A rest parameter or binding pattern may not have a trailing comma.
///
/// Thin alias for [`unexpected_trailing_comma`] with a fixed subject; returns that diagnostic's
/// variant directly rather than materializing and re-deferring.
#[inline]
pub fn rest_element_trailing_comma<'a>(span: Span) -> ParserDiagnostic<'a> {
    unexpected_trailing_comma("A rest parameter or binding pattern", span)
}

impl From<OxcDiagnostic> for ParserDiagnostic<'_> {
    #[inline]
    fn from(diagnostic: OxcDiagnostic) -> Self {
        ParserDiagnostic::Eager(Box::new(diagnostic))
    }
}

#[cold]
pub fn duplicate_export(name: &str, declared_span: Span, redeclared_span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("Duplicated export '{name}'")).with_labels([
        declared_span.label("Export has already been declared here"),
        redeclared_span.label("It cannot be redeclared here"),
    ])
}

#[cold]
pub fn duplicate_default_export(spans: Vec<Span>) -> OxcDiagnostic {
    ts_error("2528", "A module cannot have multiple default exports.").with_labels(spans)
}
