// Auto-generated code, DO NOT EDIT DIRECTLY!
// To regenerate: `cargo run -p oxc_linter_codegen`

#![allow(clippy::needless_pass_by_value)]

use oxc_ast::AstType;
use oxc_semantic::AstTypesBitset;

use crate::rule::{RuleRunFunctionsImplemented, RuleRunner};

impl RuleRunner for crate::jsx_no_jsx_as_prop::JsxNoJsxAsProp {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::JSXAttribute]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::jsx_no_new_array_as_prop::JsxNoNewArrayAsProp {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::JSXAttribute]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::jsx_no_new_function_as_prop::JsxNoNewFunctionAsProp {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::JSXAttribute]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::jsx_no_new_object_as_prop::JsxNoNewObjectAsProp {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::JSXAttribute]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}
