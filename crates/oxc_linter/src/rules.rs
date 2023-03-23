pub mod early_error {
    pub mod javascript;
}

oxc_macros::declare_all_lint_rules! {
    constructor_super,
    eq_eq_eq,
    for_direction,
    no_debugger,
    no_dupe_keys,
    no_duplicate_case,
    no_array_constructor,
    no_async_promise_executor,
    no_caller,
    no_class_assign,
    no_const_assign,
    no_dupe_class_members,
    no_empty,
    no_empty_pattern,
    no_self_compare,
    no_mixed_operators,
    no_constant_binary_expression,
    no_compare_neg_zero,
    no_unsafe_negation,
    deepscan::uninvoked_array_callback,
    use_isnan,
}
