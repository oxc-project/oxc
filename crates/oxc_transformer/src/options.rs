use oxc_syntax::assumptions::CompilerAssumptions;

use crate::{
    es2015::ArrowFunctionsOptions, es2020::NullishCoalescingOperatorOptions,
    react_jsx::ReactJsxOptions,
};

#[derive(Debug, Default, Clone)]
pub struct TransformOptions {
    pub target: TransformTarget,
    pub assumptions: CompilerAssumptions,

    pub react_jsx: Option<ReactJsxOptions>,

    // es2022
    pub class_static_block: bool,
    // es2021
    pub logical_assignment_operators: bool,
    // es2020
    pub nullish_coalescing_operator: Option<NullishCoalescingOperatorOptions>,
    // es2019
    pub optional_catch_binding: bool,
    // es2016
    pub exponentiation_operator: bool,
    // es2015
    pub duplicate_keys: bool,
    pub function_name: bool,
    pub arrow_functions: Option<ArrowFunctionsOptions>,
    pub shorthand_properties: bool,
    pub sticky_regex: bool,
    pub template_literals: bool,
    pub property_literals: bool,
    pub babel_8_breaking: Option<bool>,
    pub instanceof: bool,
}

/// See <https://www.typescriptlang.org/tsconfig#target>
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub enum TransformTarget {
    ES3,
    ES5,
    ES2015,
    ES2016,
    ES2018,
    ES2019,
    ES2020,
    ES2021,
    ES2022,
    ES2024,
    #[default]
    ESNext,
}
