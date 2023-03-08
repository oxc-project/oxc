pub mod early_error {
    pub mod javascript;
}

oxc_macros::declare_all_lint_rules! {
    constructor_super,
    eq_eq_eq,
    for_direction,
    no_debugger,
    no_array_constructor,
    no_empty,
    no_empty_pattern,
    deepscan::uninvoked_array_callback,
}
