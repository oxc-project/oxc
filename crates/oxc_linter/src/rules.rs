oxc_macros::declare_all_lint_rules! {
    deepscan::bad_array_method_on_arguments,
    deepscan::bad_bitwise_operator,
    deepscan::bad_comparison_sequence,
    deepscan::bad_min_max_func,
    deepscan::bad_remove_event_listener,
    deepscan::missing_throw,
    deepscan::number_arg_out_of_range,
    deepscan::uninvoked_array_callback,
    eslint::array_callback_return,
    eslint::constructor_super,
    eslint::eq_eq_eq,
    eslint::for_direction,
    eslint::getter_return,
    eslint::no_array_constructor,
    eslint::no_async_promise_executor,
    eslint::no_bitwise,
    eslint::no_caller,
    eslint::no_case_declarations,
    eslint::no_class_assign,
    eslint::no_compare_neg_zero,
    eslint::no_const_assign,
    eslint::no_constant_binary_expression,
    eslint::no_constant_condition,
    eslint::no_debugger,
    eslint::no_delete_var,
    eslint::no_dupe_class_members,
    eslint::no_dupe_keys,
    eslint::no_duplicate_case,
    eslint::no_empty,
    eslint::no_empty_pattern,
    eslint::no_eval,
    eslint::no_ex_assign,
    eslint::no_func_assign,
    eslint::no_mixed_operators,
    eslint::no_new_symbol,
    eslint::no_self_compare,
    eslint::no_setter_return,
    eslint::no_shadow_restricted_names,
    eslint::no_unsafe_negation,
    eslint::no_unused_labels,
    eslint::use_isnan,
    eslint::valid_typeof,
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
