//! All registered lint rules.
//!
//! New rules need be added to these `mod` statements and also the macro at the bottom.
//!
//! These modules are declared manually because `cargo fmt` stops formatting these files with they
//! are inside a proc macro.

/// <https://github.com/import-js/eslint-plugin-import>
mod import {
    pub mod default;
    pub mod named;
    pub mod no_cycle;
    pub mod no_self_import;
}

mod deepscan {
    pub mod bad_array_method_on_arguments;
    pub mod bad_bitwise_operator;
    pub mod bad_comparison_sequence;
    pub mod bad_min_max_func;
    pub mod bad_remove_event_listener;
    pub mod missing_throw;
    pub mod number_arg_out_of_range;
    pub mod uninvoked_array_callback;
}

mod eslint {
    pub mod array_callback_return;
    pub mod constructor_super;
    pub mod eq_eq_eq;
    pub mod for_direction;
    pub mod getter_return;
    pub mod no_array_constructor;
    pub mod no_async_promise_executor;
    pub mod no_bitwise;
    pub mod no_caller;
    pub mod no_case_declarations;
    pub mod no_class_assign;
    pub mod no_compare_neg_zero;
    pub mod no_cond_assign;
    pub mod no_console;
    pub mod no_const_assign;
    pub mod no_constant_binary_expression;
    pub mod no_constant_condition;
    pub mod no_control_regex;
    pub mod no_debugger;
    pub mod no_delete_var;
    pub mod no_dupe_class_members;
    pub mod no_dupe_else_if;
    pub mod no_dupe_keys;
    pub mod no_duplicate_case;
    pub mod no_empty;
    pub mod no_empty_character_class;
    pub mod no_empty_pattern;
    pub mod no_eval;
    pub mod no_ex_assign;
    pub mod no_extra_boolean_cast;
    pub mod no_fallthrough;
    pub mod no_func_assign;
    pub mod no_global_assign;
    pub mod no_import_assign;
    pub mod no_inner_declarations;
    pub mod no_loss_of_precision;
    pub mod no_mixed_operators;
    pub mod no_new_symbol;
    pub mod no_obj_calls;
    pub mod no_prototype_builtins;
    pub mod no_redeclare;
    pub mod no_return_await;
    pub mod no_self_assign;
    pub mod no_self_compare;
    pub mod no_setter_return;
    pub mod no_shadow_restricted_names;
    pub mod no_sparse_arrays;
    pub mod no_undef;
    pub mod no_unsafe_finally;
    pub mod no_unsafe_negation;
    pub mod no_unsafe_optional_chaining;
    pub mod no_unused_labels;
    pub mod no_useless_catch;
    pub mod no_useless_escape;
    pub mod require_yield;
    pub mod use_isnan;
    pub mod valid_typeof;
}

mod typescript {
    pub mod adjacent_overload_signatures;
    pub mod ban_ts_comment;
    pub mod ban_types;
    pub mod consistent_type_exports;
    pub mod no_duplicate_enum_values;
    pub mod no_empty_interface;
    pub mod no_explicit_any;
    pub mod no_extra_non_null_assertion;
    pub mod no_misused_new;
    pub mod no_namespace;
    pub mod no_non_null_asserted_optional_chain;
    pub mod no_this_alias;
    pub mod no_unnecessary_type_constraint;
    pub mod no_unsafe_declaration_merging;
    pub mod no_var_requires;
    pub mod prefer_as_const;
}

mod jest {
    pub mod expect_expect;
    pub mod no_alias_methods;
    pub mod no_commented_out_tests;
    pub mod no_conditional_expect;
    pub mod no_confusing_set_timeout;
    pub mod no_disabled_tests;
    pub mod no_done_callback;
    pub mod no_export;
    pub mod no_focused_tests;
    pub mod no_identical_title;
    pub mod no_interpolation_in_snapshots;
    pub mod no_jasmine_globals;
    pub mod no_mocks_import;
    pub mod no_standalone_expect;
    pub mod no_test_prefixes;
    pub mod valid_describe_callback;
    pub mod valid_expect;
    pub mod valid_title;
}

mod react {
    pub mod jsx_key;
    pub mod jsx_no_comment_text_nodes;
    pub mod jsx_no_duplicate_props;
    pub mod jsx_no_useless_fragment;
    pub mod no_children_prop;
    pub mod no_dangerously_set_inner_html;
    pub mod no_unescaped_entities;
}

mod unicorn {
    pub mod catch_error_name;
    pub mod error_message;
    pub mod filename_case;
    pub mod no_console_spaces;
    pub mod no_instanceof_array;
    pub mod no_thenable;
    pub mod no_unnecessary_await;
    pub mod prefer_array_flat_map;
    pub mod throw_new_error;
}

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
    eslint::no_cond_assign,
    eslint::no_console,
    eslint::no_const_assign,
    eslint::no_constant_binary_expression,
    eslint::no_constant_condition,
    eslint::no_control_regex,
    eslint::no_debugger,
    eslint::no_delete_var,
    eslint::no_dupe_class_members,
    eslint::no_dupe_else_if,
    eslint::no_dupe_keys,
    eslint::no_duplicate_case,
    eslint::no_empty_character_class,
    eslint::no_empty_pattern,
    eslint::no_empty,
    eslint::no_eval,
    eslint::no_ex_assign,
    eslint::no_extra_boolean_cast,
    eslint::no_fallthrough,
    eslint::no_func_assign,
    eslint::no_global_assign,
    eslint::no_import_assign,
    eslint::no_inner_declarations,
    eslint::no_loss_of_precision,
    eslint::no_mixed_operators,
    eslint::no_new_symbol,
    eslint::no_obj_calls,
    eslint::no_prototype_builtins,
    eslint::no_redeclare,
    eslint::no_return_await,
    eslint::no_self_assign,
    eslint::no_self_compare,
    eslint::no_setter_return,
    eslint::no_shadow_restricted_names,
    eslint::no_sparse_arrays,
    eslint::no_undef,
    eslint::no_unsafe_finally,
    eslint::no_unsafe_negation,
    eslint::no_unsafe_optional_chaining,
    eslint::no_unused_labels,
    eslint::no_useless_catch,
    eslint::no_useless_escape,
    eslint::require_yield,
    eslint::use_isnan,
    eslint::valid_typeof,
    typescript::adjacent_overload_signatures,
    typescript::ban_ts_comment,
    typescript::ban_types,
    typescript::consistent_type_exports,
    typescript::no_duplicate_enum_values,
    typescript::no_empty_interface,
    typescript::no_explicit_any,
    typescript::no_extra_non_null_assertion,
    typescript::no_non_null_asserted_optional_chain,
    typescript::no_unnecessary_type_constraint,
    typescript::no_unsafe_declaration_merging,
    typescript::no_misused_new,
    typescript::no_this_alias,
    typescript::no_namespace,
    typescript::no_var_requires,
    typescript::prefer_as_const,
    jest::no_disabled_tests,
    jest::no_test_prefixes,
    jest::no_focused_tests,
    jest::valid_describe_callback,
    jest::valid_expect,
    jest::no_commented_out_tests,
    jest::expect_expect,
    jest::no_alias_methods,
    jest::no_conditional_expect,
    jest::no_confusing_set_timeout,
    jest::no_done_callback,
    jest::no_interpolation_in_snapshots,
    jest::no_jasmine_globals,
    jest::no_mocks_import,
    jest::no_export,
    jest::no_standalone_expect,
    jest::no_identical_title,
    jest::valid_title,
    unicorn::catch_error_name,
    unicorn::error_message,
    unicorn::filename_case,
    unicorn::no_console_spaces,
    unicorn::no_instanceof_array,
    unicorn::no_unnecessary_await,
    unicorn::no_thenable,
    unicorn::throw_new_error,
    unicorn::prefer_array_flat_map,
    react::jsx_key,
    react::jsx_no_comment_text_nodes,
    react::jsx_no_duplicate_props,
    react::no_unescaped_entities,
    react::jsx_no_useless_fragment,
    react::no_children_prop,
    react::no_dangerously_set_inner_html,
    import::named,
    import::no_cycle,
    import::no_self_import,
    import::default
}
