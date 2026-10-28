//! React performance rules, compiled independently of the linter and other plugins.

use oxc_linter_core::{AstNode, LintContext, context, fixer, rule, utils as ast_util};

pub mod jsx_no_jsx_as_prop;
pub mod jsx_no_new_array_as_prop;
pub mod jsx_no_new_function_as_prop;
pub mod jsx_no_new_object_as_prop;

mod utils;

mod generated {
    mod rule_runner_impls;
}

#[cfg(test)]
use oxc_linter::tester;
