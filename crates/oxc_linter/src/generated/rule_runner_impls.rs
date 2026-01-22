// Auto-generated code, DO NOT EDIT DIRECTLY!
// To regenerate: `cargo run -p oxc_linter_codegen`

#![allow(clippy::needless_pass_by_value)]

use oxc_ast::AstType;
use oxc_semantic::AstTypesBitset;

use crate::rule::{RuleRunFunctionsImplemented, RuleRunner};

impl RuleRunner
    for crate::rules::import::consistent_type_specifier_style::ConsistentTypeSpecifierStyle
{
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::ImportDeclaration]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::import::default::Default {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::import::export::Export {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::import::exports_last::ExportsLast {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::import::extensions::Extensions {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Unknown;
}

impl RuleRunner for crate::rules::import::first::First {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::import::group_exports::GroupExports {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::import::max_dependencies::MaxDependencies {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::import::named::Named {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::import::namespace::Namespace {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::import::no_absolute_path::NoAbsolutePath {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression, AstType::ImportDeclaration]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::import::no_amd::NoAmd {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::import::no_anonymous_default_export::NoAnonymousDefaultExport {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::ExportDefaultDeclaration]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::import::no_commonjs::NoCommonjs {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::CallExpression,
        AstType::ComputedMemberExpression,
        AstType::PrivateFieldExpression,
        AstType::StaticMemberExpression,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::import::no_cycle::NoCycle {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::import::no_default_export::NoDefaultExport {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::import::no_duplicates::NoDuplicates {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::import::no_dynamic_require::NoDynamicRequire {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression, AstType::ImportExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::import::no_empty_named_blocks::NoEmptyNamedBlocks {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::ImportDeclaration]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::import::no_mutable_exports::NoMutableExports {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::ExportDefaultDeclaration,
        AstType::ExportNamedDeclaration,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::import::no_named_as_default::NoNamedAsDefault {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::import::no_named_as_default_member::NoNamedAsDefaultMember {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::import::no_named_default::NoNamedDefault {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::import::no_named_export::NoNamedExport {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::ExportAllDeclaration,
        AstType::ExportNamedDeclaration,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::import::no_namespace::NoNamespace {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::import::no_self_import::NoSelfImport {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::import::no_unassigned_import::NoUnassignedImport {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::ExpressionStatement,
        AstType::ImportDeclaration,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::import::no_webpack_loader_syntax::NoWebpackLoaderSyntax {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression, AstType::ImportDeclaration]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::import::prefer_default_export::PreferDefaultExport {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::import::unambiguous::Unambiguous {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::eslint::accessor_pairs::AccessorPairs {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::CallExpression,
        AstType::ClassBody,
        AstType::ObjectExpression,
        AstType::TSInterfaceBody,
        AstType::TSTypeLiteral,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::array_callback_return::ArrayCallbackReturn {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::ArrowFunctionExpression, AstType::Function]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::arrow_body_style::ArrowBodyStyle {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::ArrowFunctionExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::block_scoped_var::BlockScopedVar {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::VariableDeclaration]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::capitalized_comments::CapitalizedComments {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::eslint::class_methods_use_this::ClassMethodsUseThis {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::AccessorProperty,
        AstType::MethodDefinition,
        AstType::PropertyDefinition,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::complexity::Complexity {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::ArrowFunctionExpression,
        AstType::Function,
        AstType::PropertyDefinition,
        AstType::StaticBlock,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::constructor_super::ConstructorSuper {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::Class]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::curly::Curly {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::DoWhileStatement,
        AstType::ForInStatement,
        AstType::ForOfStatement,
        AstType::ForStatement,
        AstType::IfStatement,
        AstType::WhileStatement,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::default_case::DefaultCase {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::SwitchStatement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::default_case_last::DefaultCaseLast {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::SwitchStatement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::default_param_last::DefaultParamLast {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::ArrowFunctionExpression, AstType::Function]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::eqeqeq::Eqeqeq {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::BinaryExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::for_direction::ForDirection {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::ForStatement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::func_names::FuncNames {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::Function]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::func_style::FuncStyle {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::eslint::getter_return::GetterReturn {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::ArrowFunctionExpression, AstType::Function]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::grouped_accessor_pairs::GroupedAccessorPairs {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::ClassBody,
        AstType::ObjectExpression,
        AstType::TSInterfaceBody,
        AstType::TSTypeLiteral,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::guard_for_in::GuardForIn {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::ForInStatement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::id_length::IdLength {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::BindingIdentifier,
        AstType::IdentifierName,
        AstType::PrivateIdentifier,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::init_declarations::InitDeclarations {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::VariableDeclaration]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::max_classes_per_file::MaxClassesPerFile {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::eslint::max_depth::MaxDepth {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::max_lines::MaxLines {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::eslint::max_lines_per_function::MaxLinesPerFunction {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::ArrowFunctionExpression, AstType::Function]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::max_nested_callbacks::MaxNestedCallbacks {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::ArrowFunctionExpression, AstType::Function]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::max_params::MaxParams {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::ArrowFunctionExpression, AstType::Function]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::max_statements::MaxStatements {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::eslint::new_cap::NewCap {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression, AstType::NewExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_alert::NoAlert {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_array_constructor::NoArrayConstructor {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression, AstType::NewExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_async_promise_executor::NoAsyncPromiseExecutor {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::NewExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_await_in_loop::NoAwaitInLoop {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::AwaitExpression,
        AstType::ForOfStatement,
        AstType::VariableDeclaration,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_bitwise::NoBitwise {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::AssignmentExpression,
        AstType::BinaryExpression,
        AstType::UnaryExpression,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_caller::NoCaller {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::StaticMemberExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_case_declarations::NoCaseDeclarations {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::SwitchCase]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_class_assign::NoClassAssign {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::Class]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_compare_neg_zero::NoCompareNegZero {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::BinaryExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_cond_assign::NoCondAssign {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::AssignmentExpression,
        AstType::ConditionalExpression,
        AstType::DoWhileStatement,
        AstType::ForStatement,
        AstType::IfStatement,
        AstType::WhileStatement,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_console::NoConsole {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::ComputedMemberExpression,
        AstType::StaticMemberExpression,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_const_assign::NoConstAssign {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::VariableDeclarator]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner
    for crate::rules::eslint::no_constant_binary_expression::NoConstantBinaryExpression
{
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::BinaryExpression, AstType::LogicalExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_constant_condition::NoConstantCondition {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::ConditionalExpression,
        AstType::DoWhileStatement,
        AstType::ForStatement,
        AstType::IfStatement,
        AstType::WhileStatement,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_constructor_return::NoConstructorReturn {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::ReturnStatement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_continue::NoContinue {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::ContinueStatement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_control_regex::NoControlRegex {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::CallExpression,
        AstType::NewExpression,
        AstType::RegExpLiteral,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_debugger::NoDebugger {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::DebuggerStatement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_delete_var::NoDeleteVar {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::UnaryExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_div_regex::NoDivRegex {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::RegExpLiteral]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_dupe_class_members::NoDupeClassMembers {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::eslint::no_dupe_else_if::NoDupeElseIf {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::IfStatement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_dupe_keys::NoDupeKeys {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::ObjectExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_duplicate_case::NoDuplicateCase {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::SwitchStatement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_duplicate_imports::NoDuplicateImports {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::eslint::no_else_return::NoElseReturn {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::IfStatement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_empty::NoEmpty {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::BlockStatement, AstType::SwitchStatement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_empty_character_class::NoEmptyCharacterClass {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::RegExpLiteral]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_empty_function::NoEmptyFunction {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::FunctionBody]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_empty_pattern::NoEmptyPattern {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::ArrayPattern, AstType::ObjectPattern]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_empty_static_block::NoEmptyStaticBlock {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::StaticBlock]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_eq_null::NoEqNull {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::BinaryExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_eval::NoEval {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::CallExpression,
        AstType::Program,
        AstType::ThisExpression,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_ex_assign::NoExAssign {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CatchParameter]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_extend_native::NoExtendNative {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::eslint::no_extra_bind::NoExtraBind {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_extra_boolean_cast::NoExtraBooleanCast {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression, AstType::UnaryExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_extra_label::NoExtraLabel {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::BreakStatement, AstType::ContinueStatement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_fallthrough::NoFallthrough {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::SwitchStatement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_func_assign::NoFuncAssign {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::Function]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_global_assign::NoGlobalAssign {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::eslint::no_implicit_coercion::NoImplicitCoercion {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::AssignmentExpression,
        AstType::BinaryExpression,
        AstType::TemplateLiteral,
        AstType::UnaryExpression,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_import_assign::NoImportAssign {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::ImportDeclaration]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_inline_comments::NoInlineComments {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::eslint::no_inner_declarations::NoInnerDeclarations {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::Function, AstType::VariableDeclaration]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_invalid_regexp::NoInvalidRegexp {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression, AstType::NewExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_irregular_whitespace::NoIrregularWhitespace {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::eslint::no_iterator::NoIterator {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::ComputedMemberExpression,
        AstType::PrivateFieldExpression,
        AstType::StaticMemberExpression,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_label_var::NoLabelVar {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::LabeledStatement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_labels::NoLabels {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::BreakStatement,
        AstType::ContinueStatement,
        AstType::LabeledStatement,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_lone_blocks::NoLoneBlocks {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::BlockStatement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_lonely_if::NoLonelyIf {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::IfStatement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_loop_func::NoLoopFunc {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::ArrowFunctionExpression, AstType::Function]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_loss_of_precision::NoLossOfPrecision {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::NumericLiteral]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_magic_numbers::NoMagicNumbers {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::BigIntLiteral, AstType::NumericLiteral]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner
    for crate::rules::eslint::no_misleading_character_class::NoMisleadingCharacterClass
{
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::CallExpression,
        AstType::NewExpression,
        AstType::RegExpLiteral,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_multi_assign::NoMultiAssign {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::AssignmentExpression,
        AstType::PropertyDefinition,
        AstType::VariableDeclarator,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_multi_str::NoMultiStr {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::StringLiteral]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_negated_condition::NoNegatedCondition {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::ConditionalExpression, AstType::IfStatement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_nested_ternary::NoNestedTernary {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::ConditionalExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_new::NoNew {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::NewExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_new_func::NoNewFunc {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::CallExpression,
        AstType::ComputedMemberExpression,
        AstType::NewExpression,
        AstType::PrivateFieldExpression,
        AstType::StaticMemberExpression,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_new_native_nonconstructor::NoNewNativeNonconstructor {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::NewExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_new_wrappers::NoNewWrappers {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::NewExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_nonoctal_decimal_escape::NoNonoctalDecimalEscape {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::StringLiteral]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_obj_calls::NoObjCalls {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression, AstType::NewExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_object_constructor::NoObjectConstructor {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression, AstType::NewExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_param_reassign::NoParamReassign {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::FormalParameter]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_plusplus::NoPlusplus {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::UpdateExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_promise_executor_return::NoPromiseExecutorReturn {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::NewExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_proto::NoProto {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::ComputedMemberExpression,
        AstType::PrivateFieldExpression,
        AstType::StaticMemberExpression,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_prototype_builtins::NoPrototypeBuiltins {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_redeclare::NoRedeclare {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::eslint::no_regex_spaces::NoRegexSpaces {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::CallExpression,
        AstType::NewExpression,
        AstType::RegExpLiteral,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_restricted_globals::NoRestrictedGlobals {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::IdentifierReference]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_restricted_imports::NoRestrictedImports {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::TSImportEqualsDeclaration]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Unknown;
}

impl RuleRunner for crate::rules::eslint::no_return_assign::NoReturnAssign {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::AssignmentExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_script_url::NoScriptUrl {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::StringLiteral, AstType::TemplateLiteral]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_self_assign::NoSelfAssign {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::AssignmentExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_self_compare::NoSelfCompare {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::BinaryExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_sequences::NoSequences {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::SequenceExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_setter_return::NoSetterReturn {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::ReturnStatement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_shadow_restricted_names::NoShadowRestrictedNames {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::eslint::no_sparse_arrays::NoSparseArrays {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::ArrayExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_template_curly_in_string::NoTemplateCurlyInString {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::StringLiteral]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_ternary::NoTernary {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::ConditionalExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_this_before_super::NoThisBeforeSuper {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::eslint::no_throw_literal::NoThrowLiteral {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::ThrowStatement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_unassigned_vars::NoUnassignedVars {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::VariableDeclarator]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_undef::NoUndef {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::eslint::no_undefined::NoUndefined {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::BindingIdentifier,
        AstType::IdentifierReference,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_unexpected_multiline::NoUnexpectedMultiline {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::BinaryExpression,
        AstType::CallExpression,
        AstType::ComputedMemberExpression,
        AstType::TaggedTemplateExpression,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_unneeded_ternary::NoUnneededTernary {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::ConditionalExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_unreachable::NoUnreachable {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::eslint::no_unsafe_finally::NoUnsafeFinally {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::BreakStatement,
        AstType::ContinueStatement,
        AstType::ReturnStatement,
        AstType::ThrowStatement,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_unsafe_negation::NoUnsafeNegation {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::BinaryExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_unsafe_optional_chaining::NoUnsafeOptionalChaining {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::ArrayExpression,
        AstType::AssignmentExpression,
        AstType::AssignmentPattern,
        AstType::AssignmentTargetWithDefault,
        AstType::BinaryExpression,
        AstType::CallExpression,
        AstType::Class,
        AstType::ComputedMemberExpression,
        AstType::ForOfStatement,
        AstType::NewExpression,
        AstType::PrivateFieldExpression,
        AstType::StaticMemberExpression,
        AstType::TaggedTemplateExpression,
        AstType::UnaryExpression,
        AstType::VariableDeclarator,
        AstType::WithStatement,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_unused_expressions::NoUnusedExpressions {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::ExpressionStatement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_unused_labels::NoUnusedLabels {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner
    for crate::rules::eslint::no_unused_private_class_members::NoUnusedPrivateClassMembers
{
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::eslint::no_unused_vars::NoUnusedVars {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::eslint::no_useless_backreference::NoUselessBackreference {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::CallExpression,
        AstType::NewExpression,
        AstType::RegExpLiteral,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_useless_call::NoUselessCall {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_useless_catch::NoUselessCatch {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::TryStatement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_useless_computed_key::NoUselessComputedKey {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::BindingProperty,
        AstType::MethodDefinition,
        AstType::ObjectProperty,
        AstType::PropertyDefinition,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_useless_concat::NoUselessConcat {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::BinaryExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_useless_constructor::NoUselessConstructor {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::MethodDefinition]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_useless_escape::NoUselessEscape {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::RegExpLiteral,
        AstType::StringLiteral,
        AstType::TemplateLiteral,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_useless_rename::NoUselessRename {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::ExportNamedDeclaration,
        AstType::ImportSpecifier,
        AstType::ObjectAssignmentTarget,
        AstType::ObjectPattern,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_useless_return::NoUselessReturn {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::ReturnStatement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_var::NoVar {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::VariableDeclaration]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_void::NoVoid {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::UnaryExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::no_warning_comments::NoWarningComments {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::eslint::no_with::NoWith {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::WithStatement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::operator_assignment::OperatorAssignment {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::AssignmentExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::prefer_destructuring::PreferDestructuring {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::AssignmentExpression,
        AstType::VariableDeclarator,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner
    for crate::rules::eslint::prefer_exponentiation_operator::PreferExponentiationOperator
{
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::prefer_numeric_literals::PreferNumericLiterals {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::prefer_object_has_own::PreferObjectHasOwn {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::prefer_object_spread::PreferObjectSpread {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::prefer_promise_reject_errors::PreferPromiseRejectErrors {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression, AstType::NewExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::prefer_rest_params::PreferRestParams {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::IdentifierReference]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::prefer_spread::PreferSpread {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::prefer_template::PreferTemplate {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::BinaryExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::preserve_caught_error::PreserveCaughtError {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::TryStatement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::radix::Radix {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::require_await::RequireAwait {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::FunctionBody]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::require_yield::RequireYield {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::Function]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::sort_imports::SortImports {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::eslint::sort_keys::SortKeys {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::ObjectExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::sort_vars::SortVars {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::VariableDeclaration]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::symbol_description::SymbolDescription {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::unicode_bom::UnicodeBom {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::eslint::use_isnan::UseIsnan {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::BinaryExpression,
        AstType::CallExpression,
        AstType::SwitchCase,
        AstType::SwitchStatement,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::valid_typeof::ValidTypeof {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::UnaryExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::vars_on_top::VarsOnTop {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::VariableDeclaration]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::eslint::yoda::Yoda {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::BinaryExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner
    for crate::rules::typescript::adjacent_overload_signatures::AdjacentOverloadSignatures
{
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::BlockStatement,
        AstType::Class,
        AstType::FunctionBody,
        AstType::Program,
        AstType::TSInterfaceDeclaration,
        AstType::TSModuleBlock,
        AstType::TSTypeLiteral,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::typescript::array_type::ArrayType {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::TSArrayType,
        AstType::TSAsExpression,
        AstType::TSConditionalType,
        AstType::TSIndexedAccessType,
        AstType::TSMappedType,
        AstType::TSSatisfiesExpression,
        AstType::TSTypeAliasDeclaration,
        AstType::TSTypeAnnotation,
        AstType::TSTypeParameterInstantiation,
        AstType::TSTypeReference,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::typescript::await_thenable::AwaitThenable {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Unknown;
}

impl RuleRunner for crate::rules::typescript::ban_ts_comment::BanTsComment {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::typescript::ban_tslint_comment::BanTslintComment {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::typescript::ban_types::BanTypes {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::TSTypeLiteral, AstType::TSTypeReference]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner
    for crate::rules::typescript::consistent_generic_constructors::ConsistentGenericConstructors
{
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::FormalParameter,
        AstType::PropertyDefinition,
        AstType::VariableDeclarator,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner
    for crate::rules::typescript::consistent_indexed_object_style::ConsistentIndexedObjectStyle
{
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner
    for crate::rules::typescript::consistent_type_definitions::ConsistentTypeDefinitions
{
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::ExportDefaultDeclaration,
        AstType::TSInterfaceDeclaration,
        AstType::TSTypeAliasDeclaration,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::typescript::consistent_type_imports::ConsistentTypeImports {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner
    for crate::rules::typescript::explicit_function_return_type::ExplicitFunctionReturnType
{
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::ArrowFunctionExpression, AstType::Function]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner
    for crate::rules::typescript::explicit_module_boundary_types::ExplicitModuleBoundaryTypes
{
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::ExportDefaultDeclaration,
        AstType::ExportNamedDeclaration,
        AstType::TSExportAssignment,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::typescript::no_array_delete::NoArrayDelete {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Unknown;
}

impl RuleRunner for crate::rules::typescript::no_base_to_string::NoBaseToString {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Unknown;
}

impl RuleRunner
    for crate::rules::typescript::no_confusing_non_null_assertion::NoConfusingNonNullAssertion
{
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::AssignmentExpression,
        AstType::BinaryExpression,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner
    for crate::rules::typescript::no_confusing_void_expression::NoConfusingVoidExpression
{
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Unknown;
}

impl RuleRunner for crate::rules::typescript::no_deprecated::NoDeprecated {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Unknown;
}

impl RuleRunner for crate::rules::typescript::no_duplicate_enum_values::NoDuplicateEnumValues {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::TSEnumBody]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner
    for crate::rules::typescript::no_duplicate_type_constituents::NoDuplicateTypeConstituents
{
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Unknown;
}

impl RuleRunner for crate::rules::typescript::no_dynamic_delete::NoDynamicDelete {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::UnaryExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::typescript::no_empty_interface::NoEmptyInterface {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::TSInterfaceDeclaration]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::typescript::no_empty_object_type::NoEmptyObjectType {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::TSInterfaceDeclaration,
        AstType::TSTypeLiteral,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::typescript::no_explicit_any::NoExplicitAny {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::TSAnyKeyword]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::typescript::no_extra_non_null_assertion::NoExtraNonNullAssertion {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::CallExpression,
        AstType::ComputedMemberExpression,
        AstType::StaticMemberExpression,
        AstType::TSNonNullExpression,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::typescript::no_extraneous_class::NoExtraneousClass {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::Class]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::typescript::no_floating_promises::NoFloatingPromises {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Unknown;
}

impl RuleRunner for crate::rules::typescript::no_for_in_array::NoForInArray {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Unknown;
}

impl RuleRunner for crate::rules::typescript::no_implied_eval::NoImpliedEval {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Unknown;
}

impl RuleRunner for crate::rules::typescript::no_import_type_side_effects::NoImportTypeSideEffects {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::ImportDeclaration]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::typescript::no_inferrable_types::NoInferrableTypes {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::AccessorProperty,
        AstType::ArrowFunctionExpression,
        AstType::Function,
        AstType::PropertyDefinition,
        AstType::VariableDeclarator,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner
    for crate::rules::typescript::no_meaningless_void_operator::NoMeaninglessVoidOperator
{
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Unknown;
}

impl RuleRunner for crate::rules::typescript::no_misused_new::NoMisusedNew {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::Class,
        AstType::TSInterfaceDeclaration,
        AstType::TSMethodSignature,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::typescript::no_misused_promises::NoMisusedPromises {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Unknown;
}

impl RuleRunner for crate::rules::typescript::no_misused_spread::NoMisusedSpread {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Unknown;
}

impl RuleRunner for crate::rules::typescript::no_mixed_enums::NoMixedEnums {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Unknown;
}

impl RuleRunner for crate::rules::typescript::no_namespace::NoNamespace {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::TSModuleDeclaration]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::typescript::no_non_null_asserted_nullish_coalescing::NoNonNullAssertedNullishCoalescing {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[AstType::LogicalExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::typescript::no_non_null_asserted_optional_chain::NoNonNullAssertedOptionalChain {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[AstType::TSNonNullExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::typescript::no_non_null_assertion::NoNonNullAssertion {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::TSNonNullExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner
    for crate::rules::typescript::no_redundant_type_constituents::NoRedundantTypeConstituents
{
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Unknown;
}

impl RuleRunner for crate::rules::typescript::no_require_imports::NoRequireImports {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::CallExpression,
        AstType::TSImportEqualsDeclaration,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::typescript::no_restricted_types::NoRestrictedTypes {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::TSAnyKeyword,
        AstType::TSBigIntKeyword,
        AstType::TSBooleanKeyword,
        AstType::TSClassImplements,
        AstType::TSInterfaceHeritage,
        AstType::TSNeverKeyword,
        AstType::TSNullKeyword,
        AstType::TSNumberKeyword,
        AstType::TSObjectKeyword,
        AstType::TSStringKeyword,
        AstType::TSSymbolKeyword,
        AstType::TSTupleType,
        AstType::TSTypeLiteral,
        AstType::TSTypeReference,
        AstType::TSUndefinedKeyword,
        AstType::TSUnknownKeyword,
        AstType::TSVoidKeyword,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::typescript::no_this_alias::NoThisAlias {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::AssignmentExpression,
        AstType::VariableDeclarator,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::typescript::no_unnecessary_boolean_literal_compare::NoUnnecessaryBooleanLiteralCompare {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Unknown;
}

impl RuleRunner for crate::rules::typescript::no_unnecessary_parameter_property_assignment::NoUnnecessaryParameterPropertyAssignment {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[AstType::MethodDefinition]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::typescript::no_unnecessary_template_expression::NoUnnecessaryTemplateExpression {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Unknown;
}

impl RuleRunner
    for crate::rules::typescript::no_unnecessary_type_arguments::NoUnnecessaryTypeArguments
{
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Unknown;
}

impl RuleRunner
    for crate::rules::typescript::no_unnecessary_type_assertion::NoUnnecessaryTypeAssertion
{
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Unknown;
}

impl RuleRunner
    for crate::rules::typescript::no_unnecessary_type_constraint::NoUnnecessaryTypeConstraint
{
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::TSTypeParameterDeclaration]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::typescript::no_unsafe_argument::NoUnsafeArgument {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Unknown;
}

impl RuleRunner for crate::rules::typescript::no_unsafe_assignment::NoUnsafeAssignment {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Unknown;
}

impl RuleRunner for crate::rules::typescript::no_unsafe_call::NoUnsafeCall {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Unknown;
}

impl RuleRunner
    for crate::rules::typescript::no_unsafe_declaration_merging::NoUnsafeDeclarationMerging
{
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::Class, AstType::TSInterfaceDeclaration]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::typescript::no_unsafe_enum_comparison::NoUnsafeEnumComparison {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Unknown;
}

impl RuleRunner for crate::rules::typescript::no_unsafe_function_type::NoUnsafeFunctionType {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::TSClassImplements,
        AstType::TSInterfaceHeritage,
        AstType::TSTypeReference,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::typescript::no_unsafe_member_access::NoUnsafeMemberAccess {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Unknown;
}

impl RuleRunner for crate::rules::typescript::no_unsafe_return::NoUnsafeReturn {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Unknown;
}

impl RuleRunner for crate::rules::typescript::no_unsafe_type_assertion::NoUnsafeTypeAssertion {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Unknown;
}

impl RuleRunner for crate::rules::typescript::no_unsafe_unary_minus::NoUnsafeUnaryMinus {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Unknown;
}

impl RuleRunner for crate::rules::typescript::no_useless_empty_export::NoUselessEmptyExport {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::ExportNamedDeclaration]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::typescript::no_var_requires::NoVarRequires {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::typescript::no_wrapper_object_types::NoWrapperObjectTypes {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::TSClassImplements,
        AstType::TSInterfaceHeritage,
        AstType::TSTypeReference,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner
    for crate::rules::typescript::non_nullable_type_assertion_style::NonNullableTypeAssertionStyle
{
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Unknown;
}

impl RuleRunner for crate::rules::typescript::only_throw_error::OnlyThrowError {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Unknown;
}

impl RuleRunner for crate::rules::typescript::prefer_as_const::PreferAsConst {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::PropertyDefinition,
        AstType::TSAsExpression,
        AstType::VariableDeclarator,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::typescript::prefer_enum_initializers::PreferEnumInitializers {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::TSEnumBody]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::typescript::prefer_for_of::PreferForOf {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::ForStatement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::typescript::prefer_function_type::PreferFunctionType {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::ExportDefaultDeclaration,
        AstType::TSInterfaceDeclaration,
        AstType::TSTypeAliasDeclaration,
        AstType::TSTypeAnnotation,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::typescript::prefer_includes::PreferIncludes {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Unknown;
}

impl RuleRunner for crate::rules::typescript::prefer_literal_enum_member::PreferLiteralEnumMember {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::TSEnumMember]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::typescript::prefer_namespace_keyword::PreferNamespaceKeyword {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::TSModuleDeclaration]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::typescript::prefer_nullish_coalescing::PreferNullishCoalescing {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Unknown;
}

impl RuleRunner for crate::rules::typescript::prefer_optional_chain::PreferOptionalChain {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Unknown;
}

impl RuleRunner
    for crate::rules::typescript::prefer_promise_reject_errors::PreferPromiseRejectErrors
{
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Unknown;
}

impl RuleRunner
    for crate::rules::typescript::prefer_reduce_type_parameter::PreferReduceTypeParameter
{
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Unknown;
}

impl RuleRunner for crate::rules::typescript::prefer_return_this_type::PreferReturnThisType {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Unknown;
}

impl RuleRunner for crate::rules::typescript::prefer_ts_expect_error::PreferTsExpectError {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::typescript::promise_function_async::PromiseFunctionAsync {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Unknown;
}

impl RuleRunner
    for crate::rules::typescript::related_getter_setter_pairs::RelatedGetterSetterPairs
{
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Unknown;
}

impl RuleRunner for crate::rules::typescript::require_array_sort_compare::RequireArraySortCompare {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Unknown;
}

impl RuleRunner for crate::rules::typescript::require_await::RequireAwait {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Unknown;
}

impl RuleRunner for crate::rules::typescript::restrict_plus_operands::RestrictPlusOperands {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Unknown;
}

impl RuleRunner
    for crate::rules::typescript::restrict_template_expressions::RestrictTemplateExpressions
{
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Unknown;
}

impl RuleRunner for crate::rules::typescript::return_await::ReturnAwait {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Unknown;
}

impl RuleRunner for crate::rules::typescript::strict_boolean_expressions::StrictBooleanExpressions {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Unknown;
}

impl RuleRunner
    for crate::rules::typescript::switch_exhaustiveness_check::SwitchExhaustivenessCheck
{
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Unknown;
}

impl RuleRunner for crate::rules::typescript::triple_slash_reference::TripleSlashReference {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::typescript::unbound_method::UnboundMethod {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Unknown;
}

impl RuleRunner for crate::rules::typescript::use_unknown_in_catch_callback_variable::UseUnknownInCatchCallbackVariable {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Unknown;
}

impl RuleRunner for crate::rules::jest::consistent_test_it::ConsistentTestIt {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::jest::expect_expect::ExpectExpect {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnJestNode;
}

impl RuleRunner for crate::rules::jest::max_expects::MaxExpects {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::jest::max_nested_describe::MaxNestedDescribe {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::jest::no_alias_methods::NoAliasMethods {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnJestNode;
}

impl RuleRunner for crate::rules::jest::no_commented_out_tests::NoCommentedOutTests {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::jest::no_conditional_expect::NoConditionalExpect {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnJestNode;
}

impl RuleRunner for crate::rules::jest::no_conditional_in_test::NoConditionalInTest {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::ConditionalExpression,
        AstType::IfStatement,
        AstType::LogicalExpression,
        AstType::SwitchStatement,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::jest::no_confusing_set_timeout::NoConfusingSetTimeout {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::jest::no_deprecated_functions::NoDeprecatedFunctions {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::ComputedMemberExpression,
        AstType::PrivateFieldExpression,
        AstType::StaticMemberExpression,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::jest::no_disabled_tests::NoDisabledTests {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnJestNode;
}

impl RuleRunner for crate::rules::jest::no_done_callback::NoDoneCallback {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnJestNode;
}

impl RuleRunner for crate::rules::jest::no_duplicate_hooks::NoDuplicateHooks {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::jest::no_export::NoExport {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::jest::no_focused_tests::NoFocusedTests {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnJestNode;
}

impl RuleRunner for crate::rules::jest::no_hooks::NoHooks {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnJestNode;
}

impl RuleRunner for crate::rules::jest::no_identical_title::NoIdenticalTitle {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::jest::no_interpolation_in_snapshots::NoInterpolationInSnapshots {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnJestNode;
}

impl RuleRunner for crate::rules::jest::no_jasmine_globals::NoJasmineGlobals {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::AssignmentExpression,
        AstType::CallExpression,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Unknown;
}

impl RuleRunner for crate::rules::jest::no_large_snapshots::NoLargeSnapshots {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::jest::no_mocks_import::NoMocksImport {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::jest::no_restricted_jest_methods::NoRestrictedJestMethods {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnJestNode;
}

impl RuleRunner for crate::rules::jest::no_restricted_matchers::NoRestrictedMatchers {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnJestNode;
}

impl RuleRunner for crate::rules::jest::no_standalone_expect::NoStandaloneExpect {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::jest::no_test_prefixes::NoTestPrefixes {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnJestNode;
}

impl RuleRunner for crate::rules::jest::no_test_return_statement::NoTestReturnStatement {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression, AstType::Function]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::jest::no_untyped_mock_factory::NoUntypedMockFactory {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnJestNode;
}

impl RuleRunner for crate::rules::jest::padding_around_test_blocks::PaddingAroundTestBlocks {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnJestNode;
}

impl RuleRunner for crate::rules::jest::prefer_called_with::PreferCalledWith {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnJestNode;
}

impl RuleRunner for crate::rules::jest::prefer_comparison_matcher::PreferComparisonMatcher {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnJestNode;
}

impl RuleRunner for crate::rules::jest::prefer_each::PreferEach {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::jest::prefer_equality_matcher::PreferEqualityMatcher {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnJestNode;
}

impl RuleRunner for crate::rules::jest::prefer_expect_resolves::PreferExpectResolves {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnJestNode;
}

impl RuleRunner for crate::rules::jest::prefer_hooks_in_order::PreferHooksInOrder {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::jest::prefer_hooks_on_top::PreferHooksOnTop {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::jest::prefer_jest_mocked::PreferJestMocked {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::TSAsExpression, AstType::TSTypeAssertion]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::jest::prefer_lowercase_title::PreferLowercaseTitle {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnJestNode;
}

impl RuleRunner for crate::rules::jest::prefer_mock_promise_shorthand::PreferMockPromiseShorthand {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::jest::prefer_spy_on::PreferSpyOn {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::AssignmentExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::jest::prefer_strict_equal::PreferStrictEqual {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnJestNode;
}

impl RuleRunner for crate::rules::jest::prefer_to_be::PreferToBe {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnJestNode;
}

impl RuleRunner for crate::rules::jest::prefer_to_contain::PreferToContain {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnJestNode;
}

impl RuleRunner for crate::rules::jest::prefer_to_have_been_called::PreferToHaveBeenCalled {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnJestNode;
}

impl RuleRunner
    for crate::rules::jest::prefer_to_have_been_called_times::PreferToHaveBeenCalledTimes
{
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnJestNode;
}

impl RuleRunner for crate::rules::jest::prefer_to_have_length::PreferToHaveLength {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnJestNode;
}

impl RuleRunner for crate::rules::jest::prefer_todo::PreferTodo {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnJestNode;
}

impl RuleRunner for crate::rules::jest::require_hook::RequireHook {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression, AstType::Program]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::jest::require_to_throw_message::RequireToThrowMessage {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnJestNode;
}

impl RuleRunner for crate::rules::jest::require_top_level_describe::RequireTopLevelDescribe {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::jest::valid_describe_callback::ValidDescribeCallback {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnJestNode;
}

impl RuleRunner for crate::rules::jest::valid_expect::ValidExpect {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnJestNode;
}

impl RuleRunner for crate::rules::jest::valid_title::ValidTitle {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnJestNode;
}

impl RuleRunner for crate::rules::react::button_has_type::ButtonHasType {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression, AstType::JSXOpeningElement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::react::checked_requires_onchange_or_readonly::CheckedRequiresOnchangeOrReadonly {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[AstType::CallExpression, AstType::JSXOpeningElement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::react::exhaustive_deps::ExhaustiveDeps {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::react::forbid_dom_props::ForbidDomProps {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::JSXOpeningElement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::react::forbid_elements::ForbidElements {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression, AstType::JSXOpeningElement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::react::forward_ref_uses_ref::ForwardRefUsesRef {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::react::iframe_missing_sandbox::IframeMissingSandbox {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression, AstType::JSXOpeningElement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::react::jsx_boolean_value::JsxBooleanValue {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::JSXOpeningElement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::react::jsx_curly_brace_presence::JsxCurlyBracePresence {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::JSXElement, AstType::JSXFragment]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::react::jsx_filename_extension::JsxFilenameExtension {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::react::jsx_fragments::JsxFragments {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::JSXElement, AstType::JSXFragment]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::react::jsx_handler_names::JsxHandlerNames {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::JSXAttribute]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::react::jsx_key::JsxKey {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::ArrayExpression,
        AstType::JSXElement,
        AstType::JSXFragment,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::react::jsx_max_depth::JsxMaxDepth {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::JSXElement, AstType::JSXFragment]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::react::jsx_no_comment_textnodes::JsxNoCommentTextnodes {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::JSXText]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner
    for crate::rules::react::jsx_no_constructed_context_values::JsxNoConstructedContextValues
{
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::JSXOpeningElement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::react::jsx_no_duplicate_props::JsxNoDuplicateProps {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::JSXOpeningElement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::react::jsx_no_script_url::JsxNoScriptUrl {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::JSXOpeningElement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::react::jsx_no_target_blank::JsxNoTargetBlank {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::JSXOpeningElement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::react::jsx_no_undef::JsxNoUndef {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::JSXOpeningElement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::react::jsx_no_useless_fragment::JsxNoUselessFragment {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::JSXElement, AstType::JSXFragment]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::react::jsx_pascal_case::JsxPascalCase {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::JSXOpeningElement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::react::jsx_props_no_spread_multi::JsxPropsNoSpreadMulti {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::JSXOpeningElement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::react::jsx_props_no_spreading::JsxPropsNoSpreading {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::JSXSpreadAttribute]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::react::no_array_index_key::NoArrayIndexKey {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression, AstType::JSXElement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::react::no_children_prop::NoChildrenProp {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression, AstType::JSXAttribute]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::react::no_danger::NoDanger {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression, AstType::JSXElement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::react::no_danger_with_children::NoDangerWithChildren {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression, AstType::JSXElement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::react::no_did_mount_set_state::NoDidMountSetState {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::react::no_direct_mutation_state::NoDirectMutationState {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::AssignmentExpression,
        AstType::UpdateExpression,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::react::no_find_dom_node::NoFindDomNode {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::react::no_is_mounted::NoIsMounted {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::react::no_namespace::NoNamespace {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression, AstType::JSXOpeningElement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner
    for crate::rules::react::no_redundant_should_component_update::NoRedundantShouldComponentUpdate
{
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::Class]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::react::no_render_return_value::NoRenderReturnValue {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::react::no_set_state::NoSetState {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::react::no_string_refs::NoStringRefs {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::ComputedMemberExpression,
        AstType::JSXAttribute,
        AstType::PrivateFieldExpression,
        AstType::StaticMemberExpression,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::react::no_this_in_sfc::NoThisInSfc {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::ThisExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::react::no_unescaped_entities::NoUnescapedEntities {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::JSXText]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::react::no_unknown_property::NoUnknownProperty {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::JSXOpeningElement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::react::no_unsafe::NoUnsafe {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::MethodDefinition, AstType::ObjectProperty]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::react::no_will_update_set_state::NoWillUpdateSetState {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::react::only_export_components::OnlyExportComponents {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::react::prefer_es6_class::PreferEs6Class {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression, AstType::Class]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::react::react_in_jsx_scope::ReactInJsxScope {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::JSXFragment, AstType::JSXOpeningElement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::react::require_render_return::RequireRenderReturn {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::ArrowFunctionExpression, AstType::Function]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::react::rules_of_hooks::RulesOfHooks {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::react::self_closing_comp::SelfClosingComp {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::JSXElement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::react::state_in_constructor::StateInConstructor {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::AssignmentExpression,
        AstType::PropertyDefinition,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::react::style_prop_object::StylePropObject {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression, AstType::JSXElement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::react::void_dom_elements_no_children::VoidDomElementsNoChildren {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression, AstType::JSXElement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::react_perf::jsx_no_jsx_as_prop::JsxNoJsxAsProp {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Unknown;
}

impl RuleRunner for crate::rules::react_perf::jsx_no_new_array_as_prop::JsxNoNewArrayAsProp {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Unknown;
}

impl RuleRunner for crate::rules::react_perf::jsx_no_new_function_as_prop::JsxNoNewFunctionAsProp {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Unknown;
}

impl RuleRunner for crate::rules::react_perf::jsx_no_new_object_as_prop::JsxNoNewObjectAsProp {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Unknown;
}

impl RuleRunner for crate::rules::unicorn::catch_error_name::CatchErrorName {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression, AstType::CatchParameter]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::consistent_assert::ConsistentAssert {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::ImportDeclaration]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::consistent_date_clone::ConsistentDateClone {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::NewExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner
    for crate::rules::unicorn::consistent_empty_array_spread::ConsistentEmptyArraySpread
{
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::ConditionalExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner
    for crate::rules::unicorn::consistent_existence_index_check::ConsistentExistenceIndexCheck
{
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::BinaryExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::consistent_function_scoping::ConsistentFunctionScoping {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::ArrowFunctionExpression, AstType::Function]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::empty_brace_spaces::EmptyBraceSpaces {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::BlockStatement,
        AstType::Class,
        AstType::FunctionBody,
        AstType::ObjectExpression,
        AstType::StaticBlock,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::error_message::ErrorMessage {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression, AstType::NewExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::escape_case::EscapeCase {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::RegExpLiteral,
        AstType::StringLiteral,
        AstType::TemplateLiteral,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::explicit_length_check::ExplicitLengthCheck {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::StaticMemberExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::filename_case::FilenameCase {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::unicorn::new_for_builtins::NewForBuiltins {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression, AstType::NewExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::no_abusive_eslint_disable::NoAbusiveEslintDisable {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::unicorn::no_accessor_recursion::NoAccessorRecursion {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::ThisExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::no_anonymous_default_export::NoAnonymousDefaultExport {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::AssignmentExpression,
        AstType::ExportDefaultDeclaration,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::no_array_callback_reference::NoArrayCallbackReference {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::no_array_for_each::NoArrayForEach {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner
    for crate::rules::unicorn::no_array_method_this_argument::NoArrayMethodThisArgument
{
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::no_array_reduce::NoArrayReduce {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::no_array_reverse::NoArrayReverse {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::no_array_sort::NoArraySort {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::no_await_expression_member::NoAwaitExpressionMember {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::ComputedMemberExpression,
        AstType::PrivateFieldExpression,
        AstType::StaticMemberExpression,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::no_await_in_promise_methods::NoAwaitInPromiseMethods {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::no_console_spaces::NoConsoleSpaces {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::no_document_cookie::NoDocumentCookie {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::AssignmentExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::no_empty_file::NoEmptyFile {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::unicorn::no_hex_escape::NoHexEscape {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::RegExpLiteral,
        AstType::StringLiteral,
        AstType::TemplateLiteral,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::no_immediate_mutation::NoImmediateMutation {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::ExpressionStatement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::no_instanceof_array::NoInstanceofArray {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::BinaryExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::no_instanceof_builtins::NoInstanceofBuiltins {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::BinaryExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::no_invalid_fetch_options::NoInvalidFetchOptions {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression, AstType::NewExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner
    for crate::rules::unicorn::no_invalid_remove_event_listener::NoInvalidRemoveEventListener
{
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::no_length_as_slice_end::NoLengthAsSliceEnd {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::no_lonely_if::NoLonelyIf {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::IfStatement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::no_magic_array_flat_depth::NoMagicArrayFlatDepth {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner
    for crate::rules::unicorn::no_negation_in_equality_check::NoNegationInEqualityCheck
{
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::BinaryExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::no_nested_ternary::NoNestedTernary {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::ConditionalExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::no_new_array::NoNewArray {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::NewExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::no_new_buffer::NoNewBuffer {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::NewExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::no_null::NoNull {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::NullLiteral]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner
    for crate::rules::unicorn::no_object_as_default_parameter::NoObjectAsDefaultParameter
{
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::FormalParameter]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::no_process_exit::NoProcessExit {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner
    for crate::rules::unicorn::no_single_promise_in_promise_methods::NoSinglePromiseInPromiseMethods
{
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::no_static_only_class::NoStaticOnlyClass {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::Class]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::no_thenable::NoThenable {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::AssignmentExpression,
        AstType::CallExpression,
        AstType::ExportNamedDeclaration,
        AstType::MethodDefinition,
        AstType::ObjectExpression,
        AstType::PropertyDefinition,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::no_this_assignment::NoThisAssignment {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::AssignmentExpression,
        AstType::VariableDeclarator,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::no_typeof_undefined::NoTypeofUndefined {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::BinaryExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner
    for crate::rules::unicorn::no_unnecessary_array_flat_depth::NoUnnecessaryArrayFlatDepth
{
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner
    for crate::rules::unicorn::no_unnecessary_array_splice_count::NoUnnecessaryArraySpliceCount
{
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::no_unnecessary_await::NoUnnecessaryAwait {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::AwaitExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::no_unnecessary_slice_end::NoUnnecessarySliceEnd {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner
    for crate::rules::unicorn::no_unreadable_array_destructuring::NoUnreadableArrayDestructuring
{
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::ArrayAssignmentTarget, AstType::ArrayPattern]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::no_unreadable_iife::NoUnreadableIife {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner
    for crate::rules::unicorn::no_useless_collection_argument::NoUselessCollectionArgument
{
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::NewExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner
    for crate::rules::unicorn::no_useless_error_capture_stack_trace::NoUselessErrorCaptureStackTrace
{
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner
    for crate::rules::unicorn::no_useless_fallback_in_spread::NoUselessFallbackInSpread
{
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::LogicalExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::no_useless_length_check::NoUselessLengthCheck {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::LogicalExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner
    for crate::rules::unicorn::no_useless_promise_resolve_reject::NoUselessPromiseResolveReject
{
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::no_useless_spread::NoUselessSpread {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::ArrayExpression, AstType::ObjectExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::no_useless_switch_case::NoUselessSwitchCase {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::SwitchStatement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::no_useless_undefined::NoUselessUndefined {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression, AstType::IdentifierReference]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::no_zero_fractions::NoZeroFractions {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::NumericLiteral]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::number_literal_case::NumberLiteralCase {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::BigIntLiteral, AstType::NumericLiteral]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::numeric_separators_style::NumericSeparatorsStyle {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::BigIntLiteral, AstType::NumericLiteral]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::prefer_add_event_listener::PreferAddEventListener {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::AssignmentExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::prefer_array_find::PreferArrayFind {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::AssignmentExpression,
        AstType::CallExpression,
        AstType::ComputedMemberExpression,
        AstType::VariableDeclarator,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::prefer_array_flat::PreferArrayFlat {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::prefer_array_flat_map::PreferArrayFlatMap {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::prefer_array_index_of::PreferArrayIndexOf {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::prefer_array_some::PreferArraySome {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::BinaryExpression, AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::prefer_at::PreferAt {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::CallExpression,
        AstType::ComputedMemberExpression,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::prefer_bigint_literals::PreferBigintLiterals {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::prefer_blob_reading_methods::PreferBlobReadingMethods {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::prefer_class_fields::PreferClassFields {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::Class]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::prefer_classlist_toggle::PreferClasslistToggle {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::ConditionalExpression, AstType::IfStatement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::prefer_code_point::PreferCodePoint {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::prefer_date_now::PreferDateNow {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::AssignmentExpression,
        AstType::BinaryExpression,
        AstType::CallExpression,
        AstType::UnaryExpression,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::prefer_default_parameters::PreferDefaultParameters {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::AssignmentExpression,
        AstType::VariableDeclaration,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::prefer_dom_node_append::PreferDomNodeAppend {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::prefer_dom_node_dataset::PreferDomNodeDataset {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::prefer_dom_node_remove::PreferDomNodeRemove {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::prefer_dom_node_text_content::PreferDomNodeTextContent {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::IdentifierName,
        AstType::IdentifierReference,
        AstType::StaticMemberExpression,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::prefer_event_target::PreferEventTarget {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::IdentifierReference]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::prefer_global_this::PreferGlobalThis {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::IdentifierReference]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::prefer_includes::PreferIncludes {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::BinaryExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::prefer_keyboard_event_key::PreferKeyboardEventKey {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::BindingProperty,
        AstType::StaticMemberExpression,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::prefer_logical_operator_over_ternary::PreferLogicalOperatorOverTernary {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[AstType::ConditionalExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::prefer_math_min_max::PreferMathMinMax {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::ConditionalExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::prefer_math_trunc::PreferMathTrunc {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::AssignmentExpression,
        AstType::BinaryExpression,
        AstType::UnaryExpression,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::prefer_modern_dom_apis::PreferModernDomApis {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::prefer_modern_math_apis::PreferModernMathApis {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::BinaryExpression, AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner
    for crate::rules::unicorn::prefer_native_coercion_functions::PreferNativeCoercionFunctions
{
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::ArrowFunctionExpression, AstType::Function]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::prefer_negative_index::PreferNegativeIndex {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::prefer_node_protocol::PreferNodeProtocol {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::CallExpression,
        AstType::ExportNamedDeclaration,
        AstType::ImportDeclaration,
        AstType::ImportExpression,
        AstType::TSImportEqualsDeclaration,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::prefer_number_properties::PreferNumberProperties {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::CallExpression,
        AstType::ComputedMemberExpression,
        AstType::IdentifierReference,
        AstType::PrivateFieldExpression,
        AstType::StaticMemberExpression,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::prefer_object_from_entries::PreferObjectFromEntries {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner
    for crate::rules::unicorn::prefer_optional_catch_binding::PreferOptionalCatchBinding
{
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CatchParameter]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::prefer_prototype_methods::PreferPrototypeMethods {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::prefer_query_selector::PreferQuerySelector {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::prefer_reflect_apply::PreferReflectApply {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::prefer_regexp_test::PreferRegexpTest {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::prefer_response_static_json::PreferResponseStaticJson {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::NewExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::prefer_set_has::PreferSetHas {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::VariableDeclarator]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::prefer_set_size::PreferSetSize {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::StaticMemberExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::prefer_spread::PreferSpread {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::prefer_string_raw::PreferStringRaw {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::StringLiteral]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::prefer_string_replace_all::PreferStringReplaceAll {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::prefer_string_slice::PreferStringSlice {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner
    for crate::rules::unicorn::prefer_string_starts_ends_with::PreferStringStartsEndsWith
{
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::prefer_string_trim_start_end::PreferStringTrimStartEnd {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::prefer_structured_clone::PreferStructuredClone {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::prefer_top_level_await::PreferTopLevelAwait {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::prefer_type_error::PreferTypeError {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::ThrowStatement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::require_array_join_separator::RequireArrayJoinSeparator {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::require_module_attributes::RequireModuleAttributes {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::ExportAllDeclaration,
        AstType::ExportNamedDeclaration,
        AstType::ImportDeclaration,
        AstType::ImportExpression,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::require_module_specifiers::RequireModuleSpecifiers {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::ExportNamedDeclaration,
        AstType::ImportDeclaration,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::require_number_to_fixed_digits_argument::RequireNumberToFixedDigitsArgument {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner
    for crate::rules::unicorn::require_post_message_target_origin::RequirePostMessageTargetOrigin
{
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::switch_case_braces::SwitchCaseBraces {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::SwitchStatement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner
    for crate::rules::unicorn::text_encoding_identifier_case::TextEncodingIdentifierCase
{
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::JSXText, AstType::StringLiteral]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::unicorn::throw_new_error::ThrowNewError {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::jsx_a11y::alt_text::AltText {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::JSXOpeningElement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::jsx_a11y::anchor_ambiguous_text::AnchorAmbiguousText {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::JSXElement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::jsx_a11y::anchor_has_content::AnchorHasContent {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::JSXElement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::jsx_a11y::anchor_is_valid::AnchorIsValid {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::JSXElement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner
    for crate::rules::jsx_a11y::aria_activedescendant_has_tabindex::AriaActivedescendantHasTabindex
{
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::JSXOpeningElement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::jsx_a11y::aria_props::AriaProps {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::JSXAttribute]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::jsx_a11y::aria_proptypes::AriaProptypes {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::JSXAttribute]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::jsx_a11y::aria_role::AriaRole {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::JSXElement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::jsx_a11y::aria_unsupported_elements::AriaUnsupportedElements {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::JSXOpeningElement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::jsx_a11y::autocomplete_valid::AutocompleteValid {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::JSXOpeningElement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::jsx_a11y::click_events_have_key_events::ClickEventsHaveKeyEvents {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::JSXOpeningElement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::jsx_a11y::heading_has_content::HeadingHasContent {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::JSXOpeningElement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::jsx_a11y::html_has_lang::HtmlHasLang {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::JSXOpeningElement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::jsx_a11y::iframe_has_title::IframeHasTitle {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::JSXOpeningElement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::jsx_a11y::img_redundant_alt::ImgRedundantAlt {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::JSXOpeningElement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner
    for crate::rules::jsx_a11y::label_has_associated_control::LabelHasAssociatedControl
{
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::JSXElement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::jsx_a11y::lang::Lang {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::JSXOpeningElement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::jsx_a11y::media_has_caption::MediaHasCaption {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::JSXOpeningElement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::jsx_a11y::mouse_events_have_key_events::MouseEventsHaveKeyEvents {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::JSXOpeningElement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::jsx_a11y::no_access_key::NoAccessKey {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::JSXOpeningElement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::jsx_a11y::no_aria_hidden_on_focusable::NoAriaHiddenOnFocusable {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::JSXOpeningElement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::jsx_a11y::no_autofocus::NoAutofocus {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::JSXElement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::jsx_a11y::no_distracting_elements::NoDistractingElements {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::JSXOpeningElement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::jsx_a11y::no_noninteractive_tabindex::NoNoninteractiveTabindex {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::JSXOpeningElement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::jsx_a11y::no_redundant_roles::NoRedundantRoles {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::JSXOpeningElement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner
    for crate::rules::jsx_a11y::no_static_element_interactions::NoStaticElementInteractions
{
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::JSXOpeningElement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::jsx_a11y::prefer_tag_over_role::PreferTagOverRole {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::JSXOpeningElement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::jsx_a11y::role_has_required_aria_props::RoleHasRequiredAriaProps {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::JSXOpeningElement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::jsx_a11y::role_supports_aria_props::RoleSupportsAriaProps {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::JSXOpeningElement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::jsx_a11y::scope::Scope {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::JSXOpeningElement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::jsx_a11y::tabindex_no_positive::TabindexNoPositive {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::JSXOpeningElement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::oxc::approx_constant::ApproxConstant {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::NumericLiteral]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::oxc::bad_array_method_on_arguments::BadArrayMethodOnArguments {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::oxc::bad_bitwise_operator::BadBitwiseOperator {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::AssignmentExpression,
        AstType::BinaryExpression,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::oxc::bad_char_at_comparison::BadCharAtComparison {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::oxc::bad_comparison_sequence::BadComparisonSequence {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::BinaryExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::oxc::bad_min_max_func::BadMinMaxFunc {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::oxc::bad_object_literal_comparison::BadObjectLiteralComparison {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::BinaryExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::oxc::bad_replace_all_arg::BadReplaceAllArg {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::oxc::branches_sharing_code::BranchesSharingCode {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::IfStatement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::oxc::const_comparisons::ConstComparisons {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::BinaryExpression, AstType::LogicalExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::oxc::double_comparisons::DoubleComparisons {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::LogicalExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::oxc::erasing_op::ErasingOp {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::BinaryExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::oxc::misrefactored_assign_op::MisrefactoredAssignOp {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::AssignmentExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::oxc::missing_throw::MissingThrow {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::NewExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::oxc::no_accumulating_spread::NoAccumulatingSpread {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::SpreadElement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::oxc::no_async_await::NoAsyncAwait {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::ArrowFunctionExpression, AstType::Function]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::oxc::no_async_endpoint_handlers::NoAsyncEndpointHandlers {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::oxc::no_barrel_file::NoBarrelFile {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::oxc::no_const_enum::NoConstEnum {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::TSEnumDeclaration]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::oxc::no_map_spread::NoMapSpread {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::oxc::no_optional_chaining::NoOptionalChaining {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::ChainExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::oxc::no_rest_spread_properties::NoRestSpreadProperties {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::BindingRestElement,
        AstType::ObjectAssignmentTarget,
        AstType::SpreadElement,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::oxc::no_this_in_exported_function::NoThisInExportedFunction {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::ExportDefaultDeclaration,
        AstType::ExportNamedDeclaration,
        AstType::ExportSpecifier,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::oxc::number_arg_out_of_range::NumberArgOutOfRange {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::oxc::only_used_in_recursion::OnlyUsedInRecursion {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::ArrowFunctionExpression, AstType::Function]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::oxc::uninvoked_array_callback::UninvokedArrayCallback {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::NewExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::nextjs::google_font_display::GoogleFontDisplay {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::JSXOpeningElement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::nextjs::google_font_preconnect::GoogleFontPreconnect {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::JSXOpeningElement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::nextjs::inline_script_id::InlineScriptId {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::ImportDefaultSpecifier]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::nextjs::next_script_for_ga::NextScriptForGa {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::JSXOpeningElement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::nextjs::no_assign_module_variable::NoAssignModuleVariable {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::VariableDeclaration]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::nextjs::no_async_client_component::NoAsyncClientComponent {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::nextjs::no_before_interactive_script_outside_document::NoBeforeInteractiveScriptOutsideDocument {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[AstType::JSXOpeningElement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::nextjs::no_css_tags::NoCssTags {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::JSXOpeningElement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::nextjs::no_document_import_in_page::NoDocumentImportInPage {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::ImportDeclaration]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::nextjs::no_duplicate_head::NoDuplicateHead {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::ImportDeclaration]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::nextjs::no_head_element::NoHeadElement {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::JSXOpeningElement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::nextjs::no_head_import_in_document::NoHeadImportInDocument {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::ImportDeclaration]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::nextjs::no_html_link_for_pages::NoHtmlLinkForPages {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::JSXOpeningElement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::nextjs::no_img_element::NoImgElement {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::JSXOpeningElement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::nextjs::no_page_custom_font::NoPageCustomFont {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::JSXOpeningElement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::nextjs::no_script_component_in_head::NoScriptComponentInHead {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::ImportDeclaration]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::nextjs::no_styled_jsx_in_document::NoStyledJsxInDocument {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::JSXOpeningElement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::nextjs::no_sync_scripts::NoSyncScripts {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::JSXOpeningElement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::nextjs::no_title_in_document_head::NoTitleInDocumentHead {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::ImportDeclaration]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::nextjs::no_typos::NoTypos {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::ExportNamedDeclaration]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::nextjs::no_unwanted_polyfillio::NoUnwantedPolyfillio {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::JSXOpeningElement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::jsdoc::check_access::CheckAccess {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::jsdoc::check_property_names::CheckPropertyNames {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::jsdoc::check_tag_names::CheckTagNames {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::jsdoc::empty_tags::EmptyTags {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::jsdoc::implements_on_classes::ImplementsOnClasses {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::ArrowFunctionExpression, AstType::Function]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::jsdoc::no_defaults::NoDefaults {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::ArrowFunctionExpression, AstType::Function]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::jsdoc::require_param::RequireParam {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::ArrowFunctionExpression, AstType::Function]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::jsdoc::require_param_description::RequireParamDescription {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::ArrowFunctionExpression, AstType::Function]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::jsdoc::require_param_name::RequireParamName {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::ArrowFunctionExpression, AstType::Function]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::jsdoc::require_param_type::RequireParamType {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::ArrowFunctionExpression, AstType::Function]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::jsdoc::require_property::RequireProperty {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::jsdoc::require_property_description::RequirePropertyDescription {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::jsdoc::require_property_name::RequirePropertyName {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::jsdoc::require_property_type::RequirePropertyType {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::jsdoc::require_returns::RequireReturns {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::jsdoc::require_returns_description::RequireReturnsDescription {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::ArrowFunctionExpression, AstType::Function]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::jsdoc::require_returns_type::RequireReturnsType {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::ArrowFunctionExpression, AstType::Function]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::jsdoc::require_yields::RequireYields {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::Function, AstType::YieldExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::promise::always_return::AlwaysReturn {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::promise::avoid_new::AvoidNew {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::NewExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::promise::catch_or_return::CatchOrReturn {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::ExpressionStatement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::promise::no_callback_in_promise::NoCallbackInPromise {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::promise::no_multiple_resolved::NoMultipleResolved {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::NewExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::promise::no_nesting::NoNesting {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::promise::no_new_statics::NoNewStatics {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::NewExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::promise::no_promise_in_callback::NoPromiseInCallback {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::promise::no_return_in_finally::NoReturnInFinally {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::promise::no_return_wrap::NoReturnWrap {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::promise::param_names::ParamNames {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::NewExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::promise::prefer_await_to_callbacks::PreferAwaitToCallbacks {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::ArrowFunctionExpression,
        AstType::CallExpression,
        AstType::Function,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::promise::prefer_await_to_then::PreferAwaitToThen {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::promise::prefer_catch::PreferCatch {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::ExpressionStatement]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::promise::spec_only::SpecOnly {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::ComputedMemberExpression,
        AstType::PrivateFieldExpression,
        AstType::StaticMemberExpression,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::promise::valid_params::ValidParams {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::vitest::consistent_each_for::ConsistentEachFor {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnJestNode;
}

impl RuleRunner for crate::rules::vitest::consistent_test_filename::ConsistentTestFilename {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::vitest::consistent_vitest_vi::ConsistentVitestVi {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression, AstType::ImportDeclaration]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::vitest::hoisted_apis_on_top::HoistedApisOnTop {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnJestNode;
}

impl RuleRunner for crate::rules::vitest::no_conditional_tests::NoConditionalTests {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnJestNode;
}

impl RuleRunner for crate::rules::vitest::no_import_node_test::NoImportNodeTest {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner
    for crate::rules::vitest::no_unneeded_async_expect_function::NoUnneededAsyncExpectFunction
{
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnJestNode;
}

impl RuleRunner for crate::rules::vitest::prefer_called_once::PreferCalledOnce {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnJestNode;
}

impl RuleRunner for crate::rules::vitest::prefer_called_times::PreferCalledTimes {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnJestNode;
}

impl RuleRunner
    for crate::rules::vitest::prefer_describe_function_title::PreferDescribeFunctionTitle
{
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnJestNode;
}

impl RuleRunner for crate::rules::vitest::prefer_to_be_falsy::PreferToBeFalsy {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnJestNode;
}

impl RuleRunner for crate::rules::vitest::prefer_to_be_object::PreferToBeObject {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnJestNode;
}

impl RuleRunner for crate::rules::vitest::prefer_to_be_truthy::PreferToBeTruthy {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnJestNode;
}

impl RuleRunner for crate::rules::vitest::require_local_test_context_for_concurrent_snapshots::RequireLocalTestContextForConcurrentSnapshots {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnJestNode;
}

impl RuleRunner for crate::rules::vitest::warn_todo::WarnTodo {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnJestNode;
}

impl RuleRunner for crate::rules::node::global_require::GlobalRequire {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::node::no_exports_assign::NoExportsAssign {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::AssignmentExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::node::no_new_require::NoNewRequire {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::NewExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::node::no_process_env::NoProcessEnv {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::ComputedMemberExpression,
        AstType::StaticMemberExpression,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::vue::define_emits_declaration::DefineEmitsDeclaration {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::vue::define_props_declaration::DefinePropsDeclaration {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::vue::define_props_destructuring::DefinePropsDestructuring {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::vue::max_props::MaxProps {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::vue::no_arrow_functions_in_watch::NoArrowFunctionsInWatch {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::CallExpression,
        AstType::ExportDefaultDeclaration,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner
    for crate::rules::vue::no_deprecated_destroyed_lifecycle::NoDeprecatedDestroyedLifecycle
{
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::ExportDefaultDeclaration]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::vue::no_export_in_script_setup::NoExportInScriptSetup {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::vue::no_import_compiler_macros::NoImportCompilerMacros {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::ImportDeclaration]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::vue::no_lifecycle_after_await::NoLifecycleAfterAwait {
    const NODE_TYPES: Option<&AstTypesBitset> = Some(&AstTypesBitset::from_types(&[
        AstType::CallExpression,
        AstType::ExportDefaultDeclaration,
    ]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::vue::no_multiple_slot_args::NoMultipleSlotArgs {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::vue::no_required_prop_with_default::NoRequiredPropWithDefault {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::vue::no_this_in_before_route_enter::NoThisInBeforeRouteEnter {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::ExportDefaultDeclaration]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::vue::prefer_import_from_vue::PreferImportFromVue {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::vue::require_default_export::RequireDefaultExport {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::vue::require_typed_ref::RequireTypedRef {
    const NODE_TYPES: Option<&AstTypesBitset> =
        Some(&AstTypesBitset::from_types(&[AstType::CallExpression]));
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Run;
}

impl RuleRunner for crate::rules::vue::valid_define_emits::ValidDefineEmits {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}

impl RuleRunner for crate::rules::vue::valid_define_props::ValidDefineProps {
    const NODE_TYPES: Option<&AstTypesBitset> = None;
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::RunOnce;
}
