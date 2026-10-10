// All 59 of tsgolint's rules (v7.0.2002), each ported in rules/<rule>.rs; see docs/porting.md.
// not_ported() remains for new upstream rules: a stub's create() returns it and the rule tests skip it.

use crate::rule::Rule;

pub mod await_thenable;
pub mod consistent_return;
pub mod consistent_type_exports;
pub mod dot_notation;
pub mod no_array_delete;
pub mod no_base_to_string;
pub mod no_confusing_void_expression;
pub mod no_deprecated;
pub mod no_duplicate_type_constituents;
pub mod no_floating_promises;
pub mod no_for_in_array;
pub mod no_generated_empty_object_type;
pub mod no_implied_eval;
pub mod no_meaningless_void_operator;
pub mod no_misused_promises;
pub mod no_misused_spread;
pub mod no_mixed_enums;
pub mod no_redundant_type_constituents;
pub mod no_unnecessary_boolean_literal_compare;
pub mod no_unnecessary_condition;
pub mod no_unnecessary_qualifier;
pub mod no_unnecessary_template_expression;
pub mod no_unnecessary_type_arguments;
pub mod no_unnecessary_type_assertion;
pub mod no_unnecessary_type_conversion;
pub mod no_unnecessary_type_parameters;
pub mod no_unsafe_argument;
pub mod no_unsafe_assignment;
pub mod no_unsafe_call;
pub mod no_unsafe_enum_comparison;
pub mod no_unsafe_member_access;
pub mod no_unsafe_return;
pub mod no_unsafe_type_assertion;
pub mod no_unsafe_unary_minus;
pub mod no_useless_default_assignment;
pub mod non_nullable_type_assertion_style;
pub mod only_throw_error;
pub mod prefer_find;
pub mod prefer_includes;
pub mod prefer_nullish_coalescing;
pub mod prefer_optional_chain;
pub mod prefer_promise_reject_errors;
pub mod prefer_readonly;
pub mod prefer_readonly_parameter_types;
pub mod prefer_reduce_type_parameter;
pub mod prefer_regexp_exec;
pub mod prefer_return_this_type;
pub mod prefer_string_starts_ends_with;
pub mod promise_function_async;
pub mod related_getter_setter_pairs;
pub mod require_array_sort_compare;
pub mod require_await;
pub mod restrict_plus_operands;
pub mod restrict_template_expressions;
pub mod return_await;
pub mod strict_boolean_expressions;
pub mod strict_void_return;
pub mod switch_exhaustiveness_check;
pub mod unbound_method;
pub mod use_unknown_in_catch_callback_variable;

pub const RULE_NAMES: &[&str] = &[
    "await-thenable",
    "consistent-return",
    "consistent-type-exports",
    "dot-notation",
    "no-array-delete",
    "no-base-to-string",
    "no-confusing-void-expression",
    "no-deprecated",
    "no-duplicate-type-constituents",
    "no-floating-promises",
    "no-for-in-array",
    "no-generated-empty-object-type",
    "no-implied-eval",
    "no-meaningless-void-operator",
    "no-misused-promises",
    "no-misused-spread",
    "no-mixed-enums",
    "no-redundant-type-constituents",
    "no-unnecessary-boolean-literal-compare",
    "no-unnecessary-condition",
    "no-unnecessary-qualifier",
    "no-unnecessary-template-expression",
    "no-unnecessary-type-arguments",
    "no-unnecessary-type-assertion",
    "no-unnecessary-type-conversion",
    "no-unnecessary-type-parameters",
    "no-unsafe-argument",
    "no-unsafe-assignment",
    "no-unsafe-call",
    "no-unsafe-enum-comparison",
    "no-unsafe-member-access",
    "no-unsafe-return",
    "no-unsafe-type-assertion",
    "no-unsafe-unary-minus",
    "no-useless-default-assignment",
    "non-nullable-type-assertion-style",
    "only-throw-error",
    "prefer-find",
    "prefer-includes",
    "prefer-nullish-coalescing",
    "prefer-optional-chain",
    "prefer-promise-reject-errors",
    "prefer-readonly",
    "prefer-readonly-parameter-types",
    "prefer-reduce-type-parameter",
    "prefer-regexp-exec",
    "prefer-return-this-type",
    "prefer-string-starts-ends-with",
    "promise-function-async",
    "related-getter-setter-pairs",
    "require-array-sort-compare",
    "require-await",
    "restrict-plus-operands",
    "restrict-template-expressions",
    "return-await",
    "strict-boolean-expressions",
    "strict-void-return",
    "switch-exhaustiveness-check",
    "unbound-method",
    "use-unknown-in-catch-callback-variable",
];

/// Prefix of the error a stub rule's create() returns (the rule tests skip such rules).
pub const NOT_PORTED_PREFIX: &str = "rule not yet ported to tsrslint: ";

pub fn not_ported(name: &str) -> String {
    format!("{NOT_PORTED_PREFIX}{name}")
}

pub fn create_rule(
    name: &str,
    options: Option<&serde_json::Value>,
) -> Result<Box<dyn Rule>, String> {
    match name {
        "await-thenable" => await_thenable::create(options),
        "consistent-return" => consistent_return::create(options),
        "consistent-type-exports" => consistent_type_exports::create(options),
        "dot-notation" => dot_notation::create(options),
        "no-array-delete" => no_array_delete::create(options),
        "no-base-to-string" => no_base_to_string::create(options),
        "no-confusing-void-expression" => no_confusing_void_expression::create(options),
        "no-deprecated" => no_deprecated::create(options),
        "no-duplicate-type-constituents" => no_duplicate_type_constituents::create(options),
        "no-floating-promises" => no_floating_promises::create(options),
        "no-generated-empty-object-type" => no_generated_empty_object_type::create(options),
        "no-for-in-array" => no_for_in_array::create(options),
        "no-implied-eval" => Ok(Box::new(no_implied_eval::NoImpliedEval)),
        "no-meaningless-void-operator" => no_meaningless_void_operator::create(options),
        "no-misused-promises" => no_misused_promises::create(options),
        "no-misused-spread" => no_misused_spread::create(options),
        "no-mixed-enums" => no_mixed_enums::create(options),
        "no-redundant-type-constituents" => no_redundant_type_constituents::create(options),
        "no-unnecessary-boolean-literal-compare" => {
            no_unnecessary_boolean_literal_compare::create(options)
        }
        "no-unnecessary-condition" => no_unnecessary_condition::create(options),
        "no-unnecessary-qualifier" => no_unnecessary_qualifier::create(options),
        "no-unnecessary-template-expression" => no_unnecessary_template_expression::create(options),
        "no-unnecessary-type-arguments" => no_unnecessary_type_arguments::create(options),
        "no-unnecessary-type-assertion" => no_unnecessary_type_assertion::create(options),
        "no-unnecessary-type-conversion" => no_unnecessary_type_conversion::create(options),
        "no-unnecessary-type-parameters" => no_unnecessary_type_parameters::create(options),
        "no-unsafe-argument" => no_unsafe_argument::create(options),
        "no-unsafe-assignment" => no_unsafe_assignment::create(options),
        "no-unsafe-call" => no_unsafe_call::create(options),
        "no-unsafe-enum-comparison" => no_unsafe_enum_comparison::create(options),
        "no-unsafe-member-access" => no_unsafe_member_access::create(options),
        "no-unsafe-return" => no_unsafe_return::create(options),
        "no-unsafe-type-assertion" => no_unsafe_type_assertion::create(options),
        "no-unsafe-unary-minus" => no_unsafe_unary_minus::create(options),
        "no-useless-default-assignment" => no_useless_default_assignment::create(options),
        "non-nullable-type-assertion-style" => non_nullable_type_assertion_style::create(options),
        "only-throw-error" => only_throw_error::create(options),
        "prefer-find" => prefer_find::create(options),
        "prefer-includes" => prefer_includes::create(options),
        "prefer-nullish-coalescing" => prefer_nullish_coalescing::create(options),
        "prefer-optional-chain" => prefer_optional_chain::create(options),
        "prefer-promise-reject-errors" => prefer_promise_reject_errors::create(options),
        "prefer-readonly" => prefer_readonly::create(options),
        "prefer-readonly-parameter-types" => prefer_readonly_parameter_types::create(options),
        "prefer-reduce-type-parameter" => prefer_reduce_type_parameter::create(options),
        "prefer-regexp-exec" => prefer_regexp_exec::create(options),
        "prefer-return-this-type" => prefer_return_this_type::create(options),
        "prefer-string-starts-ends-with" => prefer_string_starts_ends_with::create(options),
        "promise-function-async" => promise_function_async::create(options),
        "related-getter-setter-pairs" => related_getter_setter_pairs::create(options),
        "require-array-sort-compare" => require_array_sort_compare::create(options),
        "require-await" => require_await::create(options),
        "restrict-plus-operands" => restrict_plus_operands::create(options),
        "restrict-template-expressions" => restrict_template_expressions::create(options),
        "return-await" => return_await::create(options),
        "strict-boolean-expressions" => strict_boolean_expressions::create(options),
        "strict-void-return" => strict_void_return::create(options),
        "switch-exhaustiveness-check" => switch_exhaustiveness_check::create(options),
        "unbound-method" => unbound_method::create(options),
        "use-unknown-in-catch-callback-variable" => {
            use_unknown_in_catch_callback_variable::create(options)
        }
        _ => Err(format!("unknown rule: {name}")),
    }
}
