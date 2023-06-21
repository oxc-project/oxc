oxc_macros::declare_all_lint_rules! {
    array_callback_return,
    constructor_super,
    eq_eq_eq,
    for_direction,
    getter_return,
    no_debugger,
    no_dupe_keys,
    no_duplicate_case,
    no_array_constructor,
    no_async_promise_executor,
    no_caller,
    no_class_assign,
    no_const_assign,
    no_function_assign,
    no_dupe_class_members,
    no_empty,
    no_empty_pattern,
    no_eval,
    no_new_symbol,
    no_self_compare,
    no_shadow_restricted_names,
    no_mixed_operators,
    no_constant_binary_expression,
    no_constant_condition,
    no_compare_neg_zero,
    no_unsafe_negation,
    no_unused_labels,
    no_bitwise,
    deepscan::uninvoked_array_callback,
    deepscan::bad_bitwise_operator,
    deepscan::bad_comparison_sequence,
    deepscan::bad_array_method_on_arguments,
    deepscan::missing_throw,
    deepscan::bad_min_max_func,
    use_isnan,
    valid_typeof,
    typescript::isolated_declaration
}

#[cfg(test)]
mod test {
    use super::RULES;

    #[test]
    fn ensure_documentation() {
        assert!(!RULES.is_empty());
        for rule in RULES.iter() {
            assert!(rule.documentation().is_some_and(|s| !s.is_empty()), "{}", rule.name());
        }
    }
}
