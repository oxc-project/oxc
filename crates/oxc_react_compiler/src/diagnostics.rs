// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

//! Compiler diagnostics, built directly on [`oxc_diagnostics`].
//!
//! Passes construct [`OxcDiagnostic`]s eagerly via cold functions in this module.
//! Their structured error codes let consumers recover the category for control
//! flow (Invariant/Config checks, panic-threshold severity) without a parallel
//! data model.
//!
//! Errors "thrown" by a pass (TS: exceptions escaping a pass) propagate as a
//! single `Err(OxcDiagnostic)`; errors accumulated on the Environment and
//! returned at the end of the pipeline travel as
//! [`Diagnostics`](oxc_diagnostics::Diagnostics).

use std::fmt::{Debug, Display};

use oxc_diagnostics::{LabeledSpan, OxcDiagnostic, Severity};
use oxc_span::Span;

use crate::options::PanicThreshold;

macro_rules! react_lint_url {
    ($rule:literal) => {
        concat!("https://react.dev/reference/eslint-plugin-react-hooks/lints/", $rule)
    };
}

/// Error categories matching the TS `ErrorCategory` enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    Hooks,
    CapitalizedCalls,
    StaticComponents,
    UseMemo,
    VoidUseMemo,
    PreserveManualMemo,
    MemoDependencies,
    IncompatibleLibrary,
    Immutability,
    Globals,
    Refs,
    EffectExhaustiveDependencies,
    EffectSetState,
    EffectDerivationsOfState,
    ErrorBoundaries,
    Purity,
    RenderSetState,
    Invariant,
    Todo,
    Syntax,
    UnsupportedSyntax,
    Config,
    Gating,
    Suppression,
}

impl ErrorCategory {
    const CODE_SCOPE: &'static str = "react-compiler";

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Hooks => "Hooks",
            Self::CapitalizedCalls => "CapitalizedCalls",
            Self::StaticComponents => "StaticComponents",
            Self::UseMemo => "UseMemo",
            Self::VoidUseMemo => "VoidUseMemo",
            Self::PreserveManualMemo => "PreserveManualMemo",
            Self::MemoDependencies => "MemoDependencies",
            Self::IncompatibleLibrary => "IncompatibleLibrary",
            Self::Immutability => "Immutability",
            Self::Globals => "Globals",
            Self::Refs => "Refs",
            Self::EffectExhaustiveDependencies => "EffectExhaustiveDependencies",
            Self::EffectSetState => "EffectSetState",
            Self::EffectDerivationsOfState => "EffectDerivationsOfState",
            Self::ErrorBoundaries => "ErrorBoundaries",
            Self::Purity => "Purity",
            Self::RenderSetState => "RenderSetState",
            Self::Invariant => "Invariant",
            Self::Todo => "Todo",
            Self::Syntax => "Syntax",
            Self::UnsupportedSyntax => "UnsupportedSyntax",
            Self::Config => "Config",
            Self::Gating => "Gating",
            Self::Suppression => "Suppression",
        }
    }

    /// Displayed severity, matching the TS compiler's `getRuleForCategory()`.
    /// `PreserveManualMemo` displays as an error but does not count towards
    /// `panicThreshold: critical_errors` (see [`has_critical_errors`]).
    const fn severity(self) -> Severity {
        match self {
            Self::IncompatibleLibrary | Self::UnsupportedSyntax | Self::Todo => Severity::Warning,
            _ => Severity::Error,
        }
    }

    /// Canonical guidance for diagnostics in this category.
    const fn documentation_url(self) -> &'static str {
        const REACT_LINTS: &str = "https://react.dev/reference/eslint-plugin-react-hooks";
        match self {
            Self::Hooks => react_lint_url!("rules-of-hooks"),
            Self::CapitalizedCalls | Self::StaticComponents => {
                react_lint_url!("static-components")
            }
            Self::UseMemo | Self::VoidUseMemo => react_lint_url!("use-memo"),
            Self::PreserveManualMemo => react_lint_url!("preserve-manual-memoization"),
            Self::MemoDependencies | Self::EffectExhaustiveDependencies => {
                react_lint_url!("exhaustive-deps")
            }
            Self::IncompatibleLibrary => react_lint_url!("incompatible-library"),
            Self::Immutability => react_lint_url!("immutability"),
            Self::Globals => react_lint_url!("globals"),
            Self::Refs => react_lint_url!("refs"),
            Self::EffectSetState | Self::EffectDerivationsOfState => {
                react_lint_url!("set-state-in-effect")
            }
            Self::ErrorBoundaries => react_lint_url!("error-boundaries"),
            Self::Purity => react_lint_url!("purity"),
            Self::RenderSetState => react_lint_url!("set-state-in-render"),
            Self::Config => react_lint_url!("config"),
            Self::Gating => react_lint_url!("gating"),
            Self::Syntax | Self::UnsupportedSyntax | Self::Todo => {
                react_lint_url!("unsupported-syntax")
            }
            Self::Invariant => "https://github.com/oxc-project/oxc/issues/new/choose",
            Self::Suppression => REACT_LINTS,
        }
    }

    const fn default_help(self) -> &'static str {
        match self {
            Self::Invariant => {
                "Please report this internal React Compiler error to Oxc with a minimal reproduction"
            }
            Self::Config | Self::Gating => "Update the React Compiler configuration and try again",
            Self::Todo | Self::UnsupportedSyntax | Self::Syntax => {
                "Rewrite the highlighted code using syntax supported by React Compiler"
            }
            Self::Suppression => {
                "Remove the suppression and address the reported React rule violation"
            }
            _ => "Rewrite the highlighted code to follow the Rules of React",
        }
    }

    const fn default_note(self) -> &'static str {
        match self {
            Self::Invariant => {
                "This is an internal React Compiler error; the component or hook was not optimized"
            }
            Self::Config | Self::Gating => {
                "React Compiler could not continue with this configuration"
            }
            _ => "React Compiler skipped optimizing this component or hook",
        }
    }

    /// The category whose [`Self::as_str`] is `name`.
    fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "Hooks" => Self::Hooks,
            "CapitalizedCalls" => Self::CapitalizedCalls,
            "StaticComponents" => Self::StaticComponents,
            "UseMemo" => Self::UseMemo,
            "VoidUseMemo" => Self::VoidUseMemo,
            "PreserveManualMemo" => Self::PreserveManualMemo,
            "MemoDependencies" => Self::MemoDependencies,
            "IncompatibleLibrary" => Self::IncompatibleLibrary,
            "Immutability" => Self::Immutability,
            "Globals" => Self::Globals,
            "Refs" => Self::Refs,
            "EffectExhaustiveDependencies" => Self::EffectExhaustiveDependencies,
            "EffectSetState" => Self::EffectSetState,
            "EffectDerivationsOfState" => Self::EffectDerivationsOfState,
            "ErrorBoundaries" => Self::ErrorBoundaries,
            "Purity" => Self::Purity,
            "RenderSetState" => Self::RenderSetState,
            "Invariant" => Self::Invariant,
            "Todo" => Self::Todo,
            "Syntax" => Self::Syntax,
            "UnsupportedSyntax" => Self::UnsupportedSyntax,
            "Config" => Self::Config,
            "Gating" => Self::Gating,
            "Suppression" => Self::Suppression,
            _ => return None,
        })
    }

    /// Whether a diagnostic was built for this category.
    pub fn matches(self, diagnostic: &OxcDiagnostic) -> bool {
        diagnostic.code.scope.as_deref() == Some(Self::CODE_SCOPE)
            && diagnostic.code.number.as_deref() == Some(self.as_str())
    }
}

#[cold]
fn diagnostic(category: ErrorCategory, reason: impl AsRef<str>) -> OxcDiagnostic {
    let diagnostic = match category.severity() {
        Severity::Error => OxcDiagnostic::error(reason.as_ref().to_string()),
        _ => OxcDiagnostic::warn(reason.as_ref().to_string()),
    };
    diagnostic
        .with_error_code(ErrorCategory::CODE_SCOPE, category.as_str())
        .with_help(category.default_help())
        .with_note(category.default_note())
        .with_url(category.documentation_url())
}

fn primary_label(label: impl Into<LabeledSpan>) -> LabeledSpan {
    let label = label.into();
    LabeledSpan::new_primary_with_span(label.label().map(str::to_owned), label.span())
}

#[cold]
pub fn invariant_merge_consecutive_scopes_expected_scope_at_starting_index() -> OxcDiagnostic {
    diagnostic(ErrorCategory::Invariant, "MergeConsecutiveScopes: Expected scope at starting index")
}

#[cold]
pub fn invariant_expected_function_context_empty_outer_function_declarations() -> OxcDiagnostic {
    diagnostic(
        ErrorCategory::Invariant,
        "Expected function context to be empty for outer function declarations",
    )
}

#[cold]
pub fn invariant_expected_function_expression_entry_block_have_zero_predecessors() -> OxcDiagnostic
{
    diagnostic(
        ErrorCategory::Invariant,
        "Expected function expression entry block to have zero predecessors",
    )
}

#[cold]
pub fn invariant_mismatched_loop_scope_expected_loop_got_other() -> OxcDiagnostic {
    diagnostic(ErrorCategory::Invariant, "Mismatched loop scope: expected Loop, got other")
}

#[cold]
pub fn invariant_mismatched_label_scope_expected_label_got_other() -> OxcDiagnostic {
    diagnostic(ErrorCategory::Invariant, "Mismatched label scope: expected Label, got other")
}

#[cold]
pub fn invariant_mismatched_switch_scope_expected_switch_got_other() -> OxcDiagnostic {
    diagnostic(ErrorCategory::Invariant, "Mismatched switch scope: expected Switch, got other")
}

#[cold]
pub fn invariant_expected_loop_or_switch_scope_break() -> OxcDiagnostic {
    diagnostic(ErrorCategory::Invariant, "Expected a loop or switch to be in scope for break")
}

#[cold]
pub fn invariant_continue_may_only_refer_labeled_loop() -> OxcDiagnostic {
    diagnostic(ErrorCategory::Invariant, "Continue may only refer to a labeled loop")
}

#[cold]
pub fn invariant_expected_loop_scope_continue() -> OxcDiagnostic {
    diagnostic(ErrorCategory::Invariant, "Expected a loop to be in scope for continue")
}

#[cold]
pub fn invariant_unexpected_error() -> OxcDiagnostic {
    diagnostic(ErrorCategory::Invariant, "unexpected error")
}

#[cold]
pub fn terminal_successor_references_unknown_block(
    successor: impl Display,
    terminal: impl Display,
    span: Option<Span>,
) -> OxcDiagnostic {
    const REASON: &str = "Terminal successor references unknown block";
    diagnostic(ErrorCategory::Invariant, REASON)
        .with_help(format!("Block bb{successor} does not exist for terminal '{terminal}'"))
        .with_labels(span.map(|span| span.primary_label(REASON)))
}

#[cold]
pub fn invalid_block_nesting(
    parent_start: impl Display,
    parent_end: impl Display,
    current_start: impl Display,
    current_end: impl Display,
) -> OxcDiagnostic {
    diagnostic(ErrorCategory::Invariant, "Invalid nesting in program blocks or scopes").with_help(
        format!(
            "Items overlap but are not nested: {parent_start}:{parent_end}({current_start}:{current_end})"
        ),
    )
}

#[cold]
pub fn expected_predecessor_block_to_exist(
    block: impl Display,
    predecessor: impl Display,
) -> OxcDiagnostic {
    diagnostic(ErrorCategory::Invariant, "Expected predecessor block to exist")
        .with_help(format!("Block {block} references non-existent {predecessor}"))
}

#[cold]
pub fn terminal_successor_does_not_reference_correct_predecessor(
    block: impl Display,
    predecessor: impl Display,
) -> OxcDiagnostic {
    diagnostic(
        ErrorCategory::Invariant,
        "Terminal successor does not reference correct predecessor",
    )
    .with_help(format!(
        "Block bb{block} has bb{predecessor} as a predecessor, but bb{predecessor}'s successors do not include bb{block}"
    ))
}

#[cold]
pub fn expected_all_lvalues_to_be_temporaries(name: &str, span: Option<Span>) -> OxcDiagnostic {
    const REASON: &str = "Expected all lvalues to be temporaries";
    diagnostic(ErrorCategory::Invariant, REASON)
        .with_help(format!("Found named lvalue `{name}`"))
        .with_labels(span.map(|span| span.primary_label(REASON)))
}

#[cold]
pub fn expected_lvalues_to_be_assigned_exactly_once(
    place: impl Display,
    span: Option<Span>,
) -> OxcDiagnostic {
    const REASON: &str = "Expected lvalues to be assigned exactly once";
    diagnostic(ErrorCategory::Invariant, REASON)
        .with_help(format!("Found duplicate assignment of '{place}'"))
        .with_labels(span.map(|span| span.primary_label(REASON)))
}

#[cold]
pub fn invariant_analyze_functions_expected_apply_effects_replaced_more_precise_effects()
-> OxcDiagnostic {
    diagnostic(
        ErrorCategory::Invariant,
        "[AnalyzeFunctions] Expected Apply effects to be replaced with more precise effects",
    )
}

#[cold]
pub fn invariant_expected_node_all_scopes() -> OxcDiagnostic {
    diagnostic(ErrorCategory::Invariant, "Expected a node for all scopes")
}

#[cold]
pub fn invariant_there_should_at_least_one_operand() -> OxcDiagnostic {
    diagnostic(ErrorCategory::Invariant, "there should be at least one operand")
}

#[cold]
pub fn invariant_cycle_detected() -> OxcDiagnostic {
    diagnostic(ErrorCategory::Invariant, "cycle detected")
}

#[cold]
pub fn invariant_can_only_unschedule_last_target() -> OxcDiagnostic {
    diagnostic(ErrorCategory::Invariant, "Can only unschedule the last target")
}

#[cold]
pub fn invariant_unexpected_switch_where_case_already_scheduled_and_block_not_fallthrough()
-> OxcDiagnostic {
    diagnostic(
        ErrorCategory::Invariant,
        "Unexpected 'switch' where a case is already scheduled and block is not the fallthrough",
    )
}

#[cold]
pub fn invariant_unexpected_do_while_where_loop_already_scheduled() -> OxcDiagnostic {
    diagnostic(
        ErrorCategory::Invariant,
        "Unexpected 'do-while' where the loop is already scheduled",
    )
}

#[cold]
pub fn invariant_unexpected_while_where_loop_already_scheduled() -> OxcDiagnostic {
    diagnostic(ErrorCategory::Invariant, "Unexpected 'while' where the loop is already scheduled")
}

#[cold]
pub fn invariant_unexpected_where_loop_already_scheduled() -> OxcDiagnostic {
    diagnostic(ErrorCategory::Invariant, "Unexpected 'for' where the loop is already scheduled")
}

#[cold]
pub fn invariant_unexpected_where_loop_already_scheduled_2() -> OxcDiagnostic {
    diagnostic(ErrorCategory::Invariant, "Unexpected 'for-of' where the loop is already scheduled")
}

#[cold]
pub fn invariant_unexpected_where_loop_already_scheduled_3() -> OxcDiagnostic {
    diagnostic(ErrorCategory::Invariant, "Unexpected 'for-in' where the loop is already scheduled")
}

#[cold]
pub fn invariant_unexpected_label_where_block_already_scheduled() -> OxcDiagnostic {
    diagnostic(ErrorCategory::Invariant, "Unexpected 'label' where the block is already scheduled")
}

#[cold]
pub fn invariant_unexpected_scope_where_block_already_scheduled() -> OxcDiagnostic {
    diagnostic(ErrorCategory::Invariant, "Unexpected 'scope' where the block is already scheduled")
}

#[cold]
pub fn invariant_unexpected_scope_where_block_already_scheduled_2() -> OxcDiagnostic {
    diagnostic(ErrorCategory::Invariant, "Unexpected 'scope' where the block is already scheduled")
}

#[cold]
pub fn invariant_unexpected_branch_where_alternate_already_scheduled() -> OxcDiagnostic {
    diagnostic(
        ErrorCategory::Invariant,
        "Unexpected 'branch' where the alternate is already scheduled",
    )
}

#[cold]
pub fn invariant_unexpected_maybe_throw_visit_value_block_terminal() -> OxcDiagnostic {
    diagnostic(ErrorCategory::Invariant, "Unexpected maybe-throw in visit_value_block_terminal")
}

#[cold]
pub fn todo_support_labeled_statements_combined_value_blocks_not_yet_implemented() -> OxcDiagnostic
{
    diagnostic(
        ErrorCategory::Todo,
        "Support labeled statements combined with value blocks is not yet implemented",
    )
}

#[cold]
pub fn invariant_unsupported_terminal_kind_value_block() -> OxcDiagnostic {
    diagnostic(ErrorCategory::Invariant, "Unsupported terminal kind in value block")
}

#[cold]
pub fn invariant_expected_reactive_scope_implicitly_break_fallthrough() -> OxcDiagnostic {
    diagnostic(
        ErrorCategory::Invariant,
        "Expected reactive scope to implicitly break to fallthrough",
    )
}

#[cold]
pub fn invariant_unexpected_compiled_functions_when_module_scope_opt_out_present() -> OxcDiagnostic
{
    diagnostic(
        ErrorCategory::Invariant,
        "Unexpected compiled functions when module scope opt-out is present",
    )
}

#[cold]
pub fn invariant_analyze_functions_expected_apply_effects_replaced_more_precise_effects_2()
-> OxcDiagnostic {
    diagnostic(
        ErrorCategory::Invariant,
        "[AnalyzeFunctions] Expected Apply effects to be replaced with more precise effects",
    )
}

#[cold]
pub fn invariant_unexpected_unknown_effect() -> OxcDiagnostic {
    diagnostic(ErrorCategory::Invariant, "Unexpected unknown effect")
}

#[cold]
pub fn invariant_ref_type_environment_did_not_converge() -> OxcDiagnostic {
    diagnostic(ErrorCategory::Invariant, "Ref type environment did not converge")
}

#[cold]
pub fn invariant_infer_mutation_aliasing_effects_potential_infinite_loop_value_temporary_place_or_effect_no()
-> OxcDiagnostic {
    diagnostic(
        ErrorCategory::Invariant,
        "[InferMutationAliasingEffects] Potential infinite loop: \
                 A value, temporary place, or effect was not cached properly",
    )
}

#[cold]
pub fn purity_impure_function_call() -> OxcDiagnostic {
    diagnostic(ErrorCategory::Purity, "Impure function call")
}

#[cold]
pub fn todo_codegen_reactive_function_codegen_instruction_value_cannot_declare_variables_value_block()
-> OxcDiagnostic {
    diagnostic(
        ErrorCategory::Todo,
        "(CodegenReactiveFunction::codegenInstructionValue) Cannot declare variables in a value block",
    )
}

#[cold]
pub fn todo_codegen_reactive_function_codegen_instruction_value_handle_conversion_statement_expression()
-> OxcDiagnostic {
    diagnostic(
        ErrorCategory::Todo,
        "(CodegenReactiveFunction::codegenInstructionValue) Handle conversion of statement to expression",
    )
}

#[cold]
pub fn invariant_could_not_find_binding_declaration(
    label: impl Into<oxc_diagnostics::LabeledSpan>,
) -> OxcDiagnostic {
    diagnostic(ErrorCategory::Invariant, "Could not find binding for declaration")
        .with_label(primary_label(label))
}

#[cold]
pub fn invariant_could_not_find_binding_declaration_2(
    label: impl Into<oxc_diagnostics::LabeledSpan>,
) -> OxcDiagnostic {
    diagnostic(ErrorCategory::Invariant, "Could not find binding for declaration")
        .with_label(primary_label(label))
}

#[cold]
pub fn syntax_expected_const_declaration_not_reassigned(
    label: impl Into<oxc_diagnostics::LabeledSpan>,
) -> OxcDiagnostic {
    diagnostic(ErrorCategory::Syntax, "Expected `const` declaration not to be reassigned")
        .with_label(primary_label(label))
}

#[cold]
pub fn todo_build_hir_lower_assignment_handle_computed_properties_object_pattern(
    label: impl Into<oxc_diagnostics::LabeledSpan>,
) -> OxcDiagnostic {
    diagnostic(
        ErrorCategory::Todo,
        "(BuildHIR::lowerAssignment) Handle computed properties in ObjectPattern",
    )
    .with_label(primary_label(label))
}

#[cold]
pub fn todo_expected_reassignment_globals_enable_force_temporaries(
    label: impl Into<oxc_diagnostics::LabeledSpan>,
) -> OxcDiagnostic {
    diagnostic(ErrorCategory::Todo, "Expected reassignment of globals to enable forceTemporaries")
        .with_label(primary_label(label))
}

#[cold]
pub fn todo_expected_reassignment_globals_enable_force_temporaries_2(
    label: impl Into<oxc_diagnostics::LabeledSpan>,
) -> OxcDiagnostic {
    diagnostic(ErrorCategory::Todo, "Expected reassignment of globals to enable forceTemporaries")
        .with_label(primary_label(label))
}

#[cold]
pub fn invariant_member_expression_may_only_appear_assignment_expression(
    label: impl Into<oxc_diagnostics::LabeledSpan>,
) -> OxcDiagnostic {
    diagnostic(
        ErrorCategory::Invariant,
        "MemberExpression may only appear in an assignment expression",
    )
    .with_label(primary_label(label))
}

#[cold]
pub fn todo_build_hir_lower_assignment_handle_private_name_properties_member_expression(
    label: impl Into<oxc_diagnostics::LabeledSpan>,
) -> OxcDiagnostic {
    diagnostic(
        ErrorCategory::Todo,
        "(BuildHIR::lowerAssignment) Handle PrivateName properties in MemberExpression",
    )
    .with_label(primary_label(label))
}

#[cold]
pub fn syntax_expected_const_declaration_not_reassigned_2(
    label: impl Into<oxc_diagnostics::LabeledSpan>,
) -> OxcDiagnostic {
    diagnostic(ErrorCategory::Syntax, "Expected `const` declaration not to be reassigned")
        .with_label(primary_label(label))
}

#[cold]
pub fn todo_expected_reassignment_globals_enable_force_temporaries_3(
    label: impl Into<oxc_diagnostics::LabeledSpan>,
) -> OxcDiagnostic {
    diagnostic(ErrorCategory::Todo, "Expected reassignment of globals to enable forceTemporaries")
        .with_label(primary_label(label))
}

#[cold]
pub fn todo_build_hir_lower_assignment_handle_computed_properties_object_pattern_2(
    label: impl Into<oxc_diagnostics::LabeledSpan>,
) -> OxcDiagnostic {
    diagnostic(
        ErrorCategory::Todo,
        "(BuildHIR::lowerAssignment) Handle computed properties in ObjectPattern",
    )
    .with_label(primary_label(label))
}

#[cold]
pub fn todo_expected_reassignment_globals_enable_force_temporaries_4(
    label: impl Into<oxc_diagnostics::LabeledSpan>,
) -> OxcDiagnostic {
    diagnostic(ErrorCategory::Todo, "Expected reassignment of globals to enable forceTemporaries")
        .with_label(primary_label(label))
}

#[cold]
pub fn todo_expected_reassignment_globals_enable_force_temporaries_5(
    label: impl Into<oxc_diagnostics::LabeledSpan>,
) -> OxcDiagnostic {
    diagnostic(ErrorCategory::Todo, "Expected reassignment of globals to enable forceTemporaries")
        .with_label(primary_label(label))
}

#[cold]
pub fn invariant_build_hir_lower_assignment_could_not_find_binding_declaration(
    label: impl Into<oxc_diagnostics::LabeledSpan>,
) -> OxcDiagnostic {
    diagnostic(
        ErrorCategory::Invariant,
        "(BuildHIR::lowerAssignment) Could not find binding for declaration.",
    )
    .with_label(primary_label(label))
}

#[cold]
pub fn syntax_java_script_import_and_export_statements_may_only_appear_at_top_level_module(
    label: impl Into<oxc_diagnostics::LabeledSpan>,
) -> OxcDiagnostic {
    diagnostic(
        ErrorCategory::Syntax,
        "JavaScript `import` and `export` statements may only appear at the top level of a module",
    )
    .with_label(primary_label(label))
}

#[cold]
pub fn todo_build_hir_lower_statement_handle_var_kinds_variable_declaration(
    label: impl Into<oxc_diagnostics::LabeledSpan>,
) -> OxcDiagnostic {
    diagnostic(
        ErrorCategory::Todo,
        "(BuildHIR::lowerStatement) Handle var kinds in VariableDeclaration",
    )
    .with_label(primary_label(label))
}

#[cold]
pub fn syntax_expect_const_declaration_not_reassigned(
    label: impl Into<oxc_diagnostics::LabeledSpan>,
) -> OxcDiagnostic {
    diagnostic(ErrorCategory::Syntax, "Expect `const` declaration not to be reassigned")
        .with_label(primary_label(label))
}

#[cold]
pub fn invariant_could_not_find_binding_declaration_3(
    label: impl Into<oxc_diagnostics::LabeledSpan>,
) -> OxcDiagnostic {
    diagnostic(ErrorCategory::Invariant, "Could not find binding for declaration")
        .with_label(primary_label(label))
}

#[cold]
pub fn syntax_expected_variable_declaration_identifier_if_no_initializer_provided(
    label: impl Into<oxc_diagnostics::LabeledSpan>,
) -> OxcDiagnostic {
    diagnostic(
        ErrorCategory::Syntax,
        "Expected variable declaration to be an identifier if no initializer was provided",
    )
    .with_label(primary_label(label))
}

#[cold]
pub fn todo_support_functions_unreachable_code_that_may_contain_hoisted_declarations<L, T>(
    labels: T,
) -> OxcDiagnostic
where
    L: Into<oxc_diagnostics::LabeledSpan>,
    T: IntoIterator<Item = L>,
{
    diagnostic(
        ErrorCategory::Todo,
        "Support functions with unreachable code that may contain hoisted declarations",
    )
    .with_labels(labels)
}

#[cold]
pub fn todo_prune_hoisted_contexts_rewrite_hoisted_function_references<L, T>(
    labels: T,
) -> OxcDiagnostic
where
    L: Into<oxc_diagnostics::LabeledSpan>,
    T: IntoIterator<Item = L>,
{
    diagnostic(ErrorCategory::Todo, "[PruneHoistedContexts] Rewrite hoisted function references")
        .with_labels(labels)
}

#[cold]
pub fn invariant_prune_hoisted_contexts_unexpected_hoisted_function<L, T>(
    labels: T,
) -> OxcDiagnostic
where
    L: Into<oxc_diagnostics::LabeledSpan>,
    T: IntoIterator<Item = L>,
{
    diagnostic(ErrorCategory::Invariant, "[PruneHoistedContexts] Unexpected hoisted function")
        .with_labels(labels)
}

#[cold]
pub fn todo_prune_hoisted_contexts_unexpected_kind<L, T>(labels: T) -> OxcDiagnostic
where
    L: Into<oxc_diagnostics::LabeledSpan>,
    T: IntoIterator<Item = L>,
{
    diagnostic(ErrorCategory::Todo, "[PruneHoistedContexts] Unexpected kind").with_labels(labels)
}

#[cold]
pub fn todo_validate_context_variable_lvalues_unhandled_instruction_variant<L, T>(
    labels: T,
) -> OxcDiagnostic
where
    L: Into<oxc_diagnostics::LabeledSpan>,
    T: IntoIterator<Item = L>,
{
    diagnostic(ErrorCategory::Todo, "ValidateContextVariableLValues: unhandled instruction variant")
        .with_labels(labels)
}

#[cold]
pub fn todo_support_destructuring_context_variables<L, T>(labels: T) -> OxcDiagnostic
where
    L: Into<oxc_diagnostics::LabeledSpan>,
    T: IntoIterator<Item = L>,
{
    diagnostic(ErrorCategory::Todo, "Support destructuring of context variables")
        .with_labels(labels)
}

#[cold]
pub fn todo_support_spread_syntax_hook_arguments<L, T>(labels: T) -> OxcDiagnostic
where
    L: Into<oxc_diagnostics::LabeledSpan>,
    T: IntoIterator<Item = L>,
{
    diagnostic(ErrorCategory::Todo, "Support spread syntax for hook arguments").with_labels(labels)
}

#[cold]
pub fn todo_support_spread_syntax_hook_arguments_2<L, T>(labels: T) -> OxcDiagnostic
where
    L: Into<oxc_diagnostics::LabeledSpan>,
    T: IntoIterator<Item = L>,
{
    diagnostic(ErrorCategory::Todo, "Support spread syntax for hook arguments").with_labels(labels)
}

#[cold]
pub fn todo_support_spread_syntax_hook_arguments_3<L, T>(labels: T) -> OxcDiagnostic
where
    L: Into<oxc_diagnostics::LabeledSpan>,
    T: IntoIterator<Item = L>,
{
    diagnostic(ErrorCategory::Todo, "Support spread syntax for hook arguments").with_labels(labels)
}

#[cold]
pub fn todo_build_hir_lower_member_expression_handle_private_field_property<L, T>(
    labels: T,
) -> OxcDiagnostic
where
    L: Into<oxc_diagnostics::LabeledSpan>,
    T: IntoIterator<Item = L>,
{
    diagnostic(
        ErrorCategory::Todo,
        "(BuildHIR::lowerMemberExpression) Handle private field property",
    )
    .with_labels(labels)
}

#[cold]
pub fn todo_build_hir_lower_expression_handle_import_expressions<L, T>(labels: T) -> OxcDiagnostic
where
    L: Into<oxc_diagnostics::LabeledSpan>,
    T: IntoIterator<Item = L>,
{
    diagnostic(ErrorCategory::Todo, "(BuildHIR::lowerExpression) Handle Import expressions")
        .with_labels(labels)
}

#[cold]
pub fn todo_build_hir_lower_expression_handle_private_name_expressions<L, T>(
    labels: T,
) -> OxcDiagnostic
where
    L: Into<oxc_diagnostics::LabeledSpan>,
    T: IntoIterator<Item = L>,
{
    diagnostic(ErrorCategory::Todo, "(BuildHIR::lowerExpression) Handle PrivateName expressions")
        .with_labels(labels)
}

#[cold]
pub fn todo_build_hir_lower_member_expression_handle_private_field_property_2<L, T>(
    labels: T,
) -> OxcDiagnostic
where
    L: Into<oxc_diagnostics::LabeledSpan>,
    T: IntoIterator<Item = L>,
{
    diagnostic(
        ErrorCategory::Todo,
        "(BuildHIR::lowerMemberExpression) Handle private field property",
    )
    .with_labels(labels)
}

#[cold]
pub fn syntax_only_object_properties_can_deleted<L, T>(labels: T) -> OxcDiagnostic
where
    L: Into<oxc_diagnostics::LabeledSpan>,
    T: IntoIterator<Item = L>,
{
    diagnostic(ErrorCategory::Syntax, "Only object properties can be deleted").with_labels(labels)
}

#[cold]
pub fn syntax_expected_sequence_expression_have_at_least_one_expression<L, T>(
    labels: T,
) -> OxcDiagnostic
where
    L: Into<oxc_diagnostics::LabeledSpan>,
    T: IntoIterator<Item = L>,
{
    diagnostic(
        ErrorCategory::Syntax,
        "Expected sequence expression to have at least one expression",
    )
    .with_labels(labels)
}

#[cold]
pub fn todo_build_hir_lower_expression_handle_yield_expression_expressions<L, T>(
    labels: T,
) -> OxcDiagnostic
where
    L: Into<oxc_diagnostics::LabeledSpan>,
    T: IntoIterator<Item = L>,
{
    diagnostic(
        ErrorCategory::Todo,
        "(BuildHIR::lowerExpression) Handle YieldExpression expressions",
    )
    .with_labels(labels)
}

#[cold]
pub fn todo_build_hir_lower_expression_handle_meta_property_expressions_other_than_import_meta<
    L,
    T,
>(
    labels: T,
) -> OxcDiagnostic
where
    L: Into<oxc_diagnostics::LabeledSpan>,
    T: IntoIterator<Item = L>,
{
    diagnostic(
        ErrorCategory::Todo,
        "(BuildHIR::lowerExpression) Handle MetaProperty expressions other than import.meta",
    )
    .with_labels(labels)
}

#[cold]
pub fn todo_build_hir_lower_expression_handle_class_expression_expressions<L, T>(
    labels: T,
) -> OxcDiagnostic
where
    L: Into<oxc_diagnostics::LabeledSpan>,
    T: IntoIterator<Item = L>,
{
    diagnostic(
        ErrorCategory::Todo,
        "(BuildHIR::lowerExpression) Handle ClassExpression expressions",
    )
    .with_labels(labels)
}

#[cold]
pub fn todo_build_hir_lower_expression_handle_super_expressions<L, T>(labels: T) -> OxcDiagnostic
where
    L: Into<oxc_diagnostics::LabeledSpan>,
    T: IntoIterator<Item = L>,
{
    diagnostic(ErrorCategory::Todo, "(BuildHIR::lowerExpression) Handle Super expressions")
        .with_labels(labels)
}

#[cold]
pub fn todo_build_hir_lower_expression_handle_this_expression_expressions<L, T>(
    labels: T,
) -> OxcDiagnostic
where
    L: Into<oxc_diagnostics::LabeledSpan>,
    T: IntoIterator<Item = L>,
{
    diagnostic(ErrorCategory::Todo, "(BuildHIR::lowerExpression) Handle ThisExpression expressions")
        .with_labels(labels)
}

#[cold]
pub fn todo_build_hir_lower_expression_handle_update_expression_variables_captured_within_lambdas<
    L,
    T,
>(
    labels: T,
) -> OxcDiagnostic
where
    L: Into<oxc_diagnostics::LabeledSpan>,
    T: IntoIterator<Item = L>,
{
    diagnostic(
        ErrorCategory::Todo,
        "(BuildHIR::lowerExpression) Handle UpdateExpression to variables captured within lambdas.",
    )
    .with_labels(labels)
}

#[cold]
pub fn todo_update_expression_where_argument_global_not_yet_supported<L, T>(
    labels: T,
) -> OxcDiagnostic
where
    L: Into<oxc_diagnostics::LabeledSpan>,
    T: IntoIterator<Item = L>,
{
    diagnostic(
        ErrorCategory::Todo,
        "UpdateExpression where argument is a global is not yet supported",
    )
    .with_labels(labels)
}

#[cold]
pub fn todo_build_hir_lower_expression_support_update_expression_where_argument_global<L, T>(
    labels: T,
) -> OxcDiagnostic
where
    L: Into<oxc_diagnostics::LabeledSpan>,
    T: IntoIterator<Item = L>,
{
    diagnostic(
        ErrorCategory::Todo,
        "(BuildHIR::lowerExpression) Support UpdateExpression where argument is a global",
    )
    .with_labels(labels)
}

#[cold]
pub fn todo_update_expression_unsupported_argument_type<L, T>(labels: T) -> OxcDiagnostic
where
    L: Into<oxc_diagnostics::LabeledSpan>,
    T: IntoIterator<Item = L>,
{
    diagnostic(ErrorCategory::Todo, "UpdateExpression with unsupported argument type")
        .with_labels(labels)
}

#[cold]
pub fn todo_logical_assignment_operators_not_yet_supported<L, T>(labels: T) -> OxcDiagnostic
where
    L: Into<oxc_diagnostics::LabeledSpan>,
    T: IntoIterator<Item = L>,
{
    diagnostic(
        ErrorCategory::Todo,
        "Logical assignment operators (||=, &&=, ??=) are not yet supported",
    )
    .with_labels(labels)
}

#[cold]
pub fn todo_compound_assignment_complex_pattern_not_yet_supported<L, T>(labels: T) -> OxcDiagnostic
where
    L: Into<oxc_diagnostics::LabeledSpan>,
    T: IntoIterator<Item = L>,
{
    diagnostic(ErrorCategory::Todo, "Compound assignment to complex pattern is not yet supported")
        .with_labels(labels)
}

#[cold]
pub fn todo_unsupported_key_type_object_expression<L, T>(labels: T) -> OxcDiagnostic
where
    L: Into<oxc_diagnostics::LabeledSpan>,
    T: IntoIterator<Item = L>,
{
    diagnostic(ErrorCategory::Todo, "Unsupported key type in ObjectExpression").with_labels(labels)
}

#[cold]
pub fn todo_build_hir_lower_statement_support_throw_statement_inside_try_catch<L, T>(
    labels: T,
) -> OxcDiagnostic
where
    L: Into<oxc_diagnostics::LabeledSpan>,
    T: IntoIterator<Item = L>,
{
    diagnostic(
        ErrorCategory::Todo,
        "(BuildHIR::lowerStatement) Support ThrowStatement inside of try/catch",
    )
    .with_labels(labels)
}

#[cold]
pub fn todo_build_hir_lower_statement_handle_non_variable_initialization_statement<L, T>(
    labels: T,
) -> OxcDiagnostic
where
    L: Into<oxc_diagnostics::LabeledSpan>,
    T: IntoIterator<Item = L>,
{
    diagnostic(
        ErrorCategory::Todo,
        "(BuildHIR::lowerStatement) Handle non-variable initialization in ForStatement",
    )
    .with_labels(labels)
}

#[cold]
pub fn todo_build_hir_lower_statement_handle_empty_test_statement<L, T>(labels: T) -> OxcDiagnostic
where
    L: Into<oxc_diagnostics::LabeledSpan>,
    T: IntoIterator<Item = L>,
{
    diagnostic(ErrorCategory::Todo, "(BuildHIR::lowerStatement) Handle empty test in ForStatement")
        .with_labels(labels)
}

#[cold]
pub fn todo_build_hir_lower_statement_handle_await_loops<L, T>(labels: T) -> OxcDiagnostic
where
    L: Into<oxc_diagnostics::LabeledSpan>,
    T: IntoIterator<Item = L>,
{
    diagnostic(ErrorCategory::Todo, "(BuildHIR::lowerStatement) Handle for-await loops")
        .with_labels(labels)
}

#[cold]
pub fn syntax_expected_at_most_one_default_branch_switch_statement<L, T>(labels: T) -> OxcDiagnostic
where
    L: Into<oxc_diagnostics::LabeledSpan>,
    T: IntoIterator<Item = L>,
{
    diagnostic(ErrorCategory::Syntax, "Expected at most one `default` branch in a switch statement")
        .with_labels(labels)
}

#[cold]
pub fn todo_build_hir_lower_statement_handle_try_statement_without_catch_clause(
    try_span: Option<Span>,
    finally_span: Option<Span>,
) -> OxcDiagnostic {
    let mut diagnostic = diagnostic(
        ErrorCategory::Todo,
        "`try`/`finally` without `catch` is not supported by React Compiler",
    )
    .with_help(
        "React Compiler cannot analyze this control flow. Refactor the cleanup to avoid `finally`, or suppress this warning if this function should remain uncompiled",
    )
    .with_labels(try_span.map(|span| span.primary_label("Unsupported `try` starts here")));
    diagnostic.labels.extend(
        finally_span
            .filter(|finally_span| Some(*finally_span) != try_span)
            .map(|span| span.label("This `finally` clause requires unsupported control flow")),
    );
    diagnostic
}

#[cold]
pub fn todo_build_hir_lower_statement_handle_try_statement_finalizer_finally_clause<L, T>(
    labels: T,
) -> OxcDiagnostic
where
    L: Into<oxc_diagnostics::LabeledSpan>,
    T: IntoIterator<Item = L>,
{
    diagnostic(
        ErrorCategory::Todo,
        "(BuildHIR::lowerStatement) Handle TryStatement with a finalizer ('finally') clause",
    )
    .with_labels(labels)
}

#[cold]
pub fn todo_support_non_trivial_inits<L, T>(labels: T) -> OxcDiagnostic
where
    L: Into<oxc_diagnostics::LabeledSpan>,
    T: IntoIterator<Item = L>,
{
    diagnostic(ErrorCategory::Todo, "Support non-trivial for..in inits").with_labels(labels)
}

#[cold]
pub fn todo_support_non_trivial_inits_2<L, T>(labels: T) -> OxcDiagnostic
where
    L: Into<oxc_diagnostics::LabeledSpan>,
    T: IntoIterator<Item = L>,
{
    diagnostic(ErrorCategory::Todo, "Support non-trivial for..of inits").with_labels(labels)
}

#[cold]
pub fn config_invalid_type_configuration_module<H, L, T>(help: H, labels: T) -> OxcDiagnostic
where
    H: Into<std::borrow::Cow<'static, str>>,
    L: Into<oxc_diagnostics::LabeledSpan>,
    T: IntoIterator<Item = L>,
{
    diagnostic(ErrorCategory::Config, "Invalid type configuration for module")
        .with_help(help)
        .with_labels(labels)
}

#[cold]
pub fn config_invalid_type_configuration_module_2<H, L, T>(help: H, labels: T) -> OxcDiagnostic
where
    H: Into<std::borrow::Cow<'static, str>>,
    L: Into<oxc_diagnostics::LabeledSpan>,
    T: IntoIterator<Item = L>,
{
    diagnostic(ErrorCategory::Config, "Invalid type configuration for module")
        .with_help(help)
        .with_labels(labels)
}

#[cold]
pub fn invariant_expected_temporaries_promoted_named_identifiers_earlier_pass(
    span: Option<Span>,
) -> OxcDiagnostic {
    diagnostic(
        ErrorCategory::Invariant,
        "Expected temporaries to be promoted to named identifiers in an earlier pass",
    )
    .with_labels(span.map(|span| {
        span.primary_label(
            "Expected temporaries to be promoted to named identifiers in an earlier pass",
        )
    }))
}

#[cold]
pub fn invariant_expected_scope_have_at_least_one_declaration(span: Option<Span>) -> OxcDiagnostic {
    diagnostic(ErrorCategory::Invariant, "Expected scope to have at least one declaration")
        .with_labels(
            span.map(|span| span.primary_label("Expected scope to have at least one declaration")),
        )
}

#[cold]
pub fn invariant_expected_early_return_value_promoted_named_variable(
    span: Option<Span>,
) -> OxcDiagnostic {
    diagnostic(
        ErrorCategory::Invariant,
        "Expected early return value to be promoted to a named variable",
    )
    .with_labels(span.map(|span| {
        span.primary_label("Expected early return value to be promoted to a named variable")
    }))
}

#[cold]
pub fn invariant_expected_sequence_expression_init(span: Option<Span>) -> OxcDiagnostic {
    diagnostic(ErrorCategory::Invariant, "Expected a sequence expression init for for..in")
        .with_labels(
            span.map(|span| span.primary_label("Expected a sequence expression init for for..in")),
        )
}

#[cold]
pub fn invariant_expected_sequence_expression_init_2(span: Option<Span>) -> OxcDiagnostic {
    diagnostic(ErrorCategory::Invariant, "Expected a sequence expression init for for..of")
        .with_labels(
            span.map(|span| span.primary_label("Expected a sequence expression init for for..of")),
        )
}

#[cold]
pub fn invariant_expected_single_expression_sequence_expression_init(
    span: Option<Span>,
) -> OxcDiagnostic {
    diagnostic(
        ErrorCategory::Invariant,
        "Expected a single-expression sequence expression init for for..of",
    )
    .with_labels(span.map(|span| {
        span.primary_label("Expected a single-expression sequence expression init for for..of")
    }))
}

#[cold]
pub fn invariant_expected_get_iterator_init(span: Option<Span>) -> OxcDiagnostic {
    diagnostic(ErrorCategory::Invariant, "Expected GetIterator in for..of init")
        .with_labels(span.map(|span| span.primary_label("Expected GetIterator in for..of init")))
}

#[cold]
pub fn invariant_expected_sequence_expression_test(span: Option<Span>) -> OxcDiagnostic {
    diagnostic(ErrorCategory::Invariant, "Expected a sequence expression test for for..of")
        .with_labels(
            span.map(|span| span.primary_label("Expected a sequence expression test for for..of")),
        )
}

#[cold]
pub fn invariant_expected_let_or_const_variable_declaration(span: Option<Span>) -> OxcDiagnostic {
    diagnostic(ErrorCategory::Invariant, "Expected a let or const variable declaration")
        .with_labels(
            span.map(|span| span.primary_label("Expected a let or const variable declaration")),
        )
}

#[cold]
pub fn invariant_expected_variable_declaration(span: Option<Span>) -> OxcDiagnostic {
    diagnostic(ErrorCategory::Invariant, "Expected a variable declaration")
        .with_labels(span.map(|span| span.primary_label("Expected a variable declaration")))
}

#[cold]
pub fn invariant_expected_variable_declaration_init(span: Option<Span>) -> OxcDiagnostic {
    diagnostic(ErrorCategory::Invariant, "Expected a variable declaration in for-init").with_labels(
        span.map(|span| span.primary_label("Expected a variable declaration in for-init")),
    )
}

#[cold]
pub fn invariant_expected_identifier_as_function_declaration_lvalue(
    span: Option<Span>,
) -> OxcDiagnostic {
    diagnostic(ErrorCategory::Invariant, "Expected an identifier as function declaration lvalue")
        .with_labels(span.map(|span| {
            span.primary_label("Expected an identifier as function declaration lvalue")
        }))
}

#[cold]
pub fn invariant_expected_function_value_function_declaration(span: Option<Span>) -> OxcDiagnostic {
    diagnostic(ErrorCategory::Invariant, "Expected a function value for function declaration")
        .with_labels(
            span.map(|span| {
                span.primary_label("Expected a function value for function declaration")
            }),
        )
}

#[cold]
pub fn invariant_expected_function_expression_function_declaration(
    span: Option<Span>,
) -> OxcDiagnostic {
    diagnostic(ErrorCategory::Invariant, "Expected a function expression for function declaration")
        .with_labels(span.map(|span| {
            span.primary_label("Expected a function expression for function declaration")
        }))
}

#[cold]
pub fn invariant_expected_value_reassignment(span: Option<Span>) -> OxcDiagnostic {
    diagnostic(ErrorCategory::Invariant, "Expected a value for reassignment")
        .with_labels(span.map(|span| span.primary_label("Expected a value for reassignment")))
}

#[cold]
pub fn invariant_expected_optional_value_resolve_call_or_member_expression(
    span: Option<Span>,
) -> OxcDiagnostic {
    diagnostic(
        ErrorCategory::Invariant,
        "Expected optional value to resolve to call or member expression",
    )
    .with_labels(span.map(|span| {
        span.primary_label("Expected optional value to resolve to call or member expression")
    }))
}

#[cold]
pub fn invariant_unexpected_default_destructuring_assignment_target(
    span: Option<Span>,
) -> OxcDiagnostic {
    diagnostic(ErrorCategory::Invariant, "Unexpected default in destructuring assignment target")
        .with_labels(span.map(|span| {
            span.primary_label("Unexpected default in destructuring assignment target")
        }))
}

#[cold]
pub fn invariant_expected_identifier_shorthand_destructuring_property(
    span: Option<Span>,
) -> OxcDiagnostic {
    diagnostic(
        ErrorCategory::Invariant,
        "Expected an identifier in shorthand destructuring property",
    )
    .with_labels(span.map(|span| {
        span.primary_label("Expected an identifier in shorthand destructuring property")
    }))
}

#[cold]
pub fn invariant_expected_identifier_shorthand_destructuring_property_2(
    span: Option<Span>,
) -> OxcDiagnostic {
    diagnostic(
        ErrorCategory::Invariant,
        "Expected an identifier in shorthand destructuring property",
    )
    .with_labels(span.map(|span| {
        span.primary_label("Expected an identifier in shorthand destructuring property")
    }))
}

#[cold]
pub fn invariant_expected_simple_assignment_target_update_expression(
    span: Option<Span>,
) -> OxcDiagnostic {
    diagnostic(
        ErrorCategory::Invariant,
        "Expected a simple assignment target for update expression",
    )
    .with_labels(span.map(|span| {
        span.primary_label("Expected a simple assignment target for update expression")
    }))
}

#[cold]
pub fn invariant_expected_object_method_instruction(span: Option<Span>) -> OxcDiagnostic {
    diagnostic(ErrorCategory::Invariant, "Expected ObjectMethod instruction")
        .with_labels(span.map(|span| span.primary_label("Expected ObjectMethod instruction")))
}

#[cold]
pub fn invariant_expected_jsx_tag_identifier_or_string(span: Option<Span>) -> OxcDiagnostic {
    diagnostic(ErrorCategory::Invariant, "Expected JSX tag to be an identifier or string")
        .with_labels(
            span.map(|span| span.primary_label("Expected JSX tag to be an identifier or string")),
        )
}

#[cold]
pub fn invariant_expected_jsx_member_expression_property_string(
    span: Option<Span>,
) -> OxcDiagnostic {
    diagnostic(ErrorCategory::Invariant, "Expected JSX member expression property to be a string")
        .with_labels(span.map(|span| {
            span.primary_label("Expected JSX member expression property to be a string")
        }))
}

#[cold]
pub fn invariant_expected_jsx_member_expression_identifier_or_nested_member_expression(
    span: Option<Span>,
) -> OxcDiagnostic {
    diagnostic(
        ErrorCategory::Invariant,
        "Expected JSX member expression to be an identifier or nested member expression",
    )
    .with_labels(span.map(|span| {
        span.primary_label(
            "Expected JSX member expression to be an identifier or nested member expression",
        )
    }))
}

#[cold]
pub fn invariant_expected_base_instruction_value(span: Option<Span>) -> OxcDiagnostic {
    diagnostic(ErrorCategory::Invariant, "Expected base instruction value")
        .with_labels(span.map(|span| span.primary_label("Expected base instruction value")))
}

#[cold]
pub fn invariant_const_declaration_cannot_referenced_as_expression(
    span: Option<Span>,
) -> OxcDiagnostic {
    diagnostic(ErrorCategory::Invariant, "Const declaration cannot be referenced as an expression")
        .with_labels(span.map(|span| span.primary_label("this is Const")))
}

#[cold]
pub fn invariant_const_declaration_cannot_referenced_as_expression_2(
    span: Option<Span>,
) -> OxcDiagnostic {
    diagnostic(ErrorCategory::Invariant, "Const declaration cannot be referenced as an expression")
        .with_labels(span.map(|span| span.primary_label("this is Let")))
}

#[cold]
fn invariant_with_help_and_reason_label(
    reason: &'static str,
    help: String,
    span: Option<Span>,
) -> OxcDiagnostic {
    diagnostic(ErrorCategory::Invariant, reason)
        .with_help(help)
        .with_labels(span.map(|span| span.primary_label(reason)))
}

#[cold]
pub fn reserved_identifier(name: &str, span: Option<Span>) -> OxcDiagnostic {
    diagnostic(ErrorCategory::Syntax, "Expected a non-reserved identifier name")
        .with_help(format!(
            "`{name}` is a reserved word in JavaScript and cannot be used as an identifier name"
        ))
        .with_labels(span.map(|span| span.primary_label(format!("`{name}` is reserved"))))
}

#[cold]
pub fn local_fbt_variable(span: Option<Span>) -> OxcDiagnostic {
    diagnostic(ErrorCategory::Todo, "Support local variables named `fbt`")
        .with_help(
            "Local variables named `fbt` may conflict with the fbt plugin and are not yet supported",
        )
        .with_labels(span.map(|span| span.primary_label("Local variables named `fbt` are not supported")))
}

#[cold]
pub fn blocklisted_import(module: &str, span: Span) -> OxcDiagnostic {
    diagnostic(ErrorCategory::Todo, "Import from a blocklisted module")
        .with_help(format!("Remove the import from blocklisted module `{module}`"))
        .with_label(span.primary_label(format!("`{module}` is blocklisted")))
}

#[cold]
pub fn invalid_gating_directive(directive: &str, span: Span) -> OxcDiagnostic {
    diagnostic(
        ErrorCategory::Gating,
        "Dynamic gating directive is not a valid JavaScript identifier",
    )
    .with_help(format!("Found '{directive}'"))
    .with_label(span.primary_label("Invalid gating condition"))
}

#[cold]
pub fn multiple_gating_directives<I>(names: &[&str], spans: I) -> OxcDiagnostic
where
    I: IntoIterator<Item = Span>,
{
    diagnostic(ErrorCategory::Gating, "Multiple dynamic gating directives found")
        .with_help(format!("Expected a single directive but found [{}]", names.join(", ")))
        .with_labels(spans.into_iter().enumerate().map(|(index, span)| {
            if index == 0 {
                span.primary_label("First gating directive")
            } else {
                span.label("Additional gating directive")
            }
        }))
}

#[cold]
pub fn missing_phi_predecessor_mapping(
    predecessor: impl Display,
    block: impl Display,
) -> OxcDiagnostic {
    diagnostic(
        ErrorCategory::Invariant,
        "Expected non-existing phi operand's predecessor to have been mapped to a new terminal",
    )
    .with_help(format!("Could not find mapping for predecessor bb{predecessor} in block bb{block}"))
}

#[cold]
pub fn instruction_in_completed_scope(id: impl Debug, scope_id: impl Debug) -> OxcDiagnostic {
    diagnostic(
        ErrorCategory::Invariant,
        "Encountered an instruction that should be part of a scope, but where that scope has already completed",
    )
    .with_help(format!(
        "Instruction [{id:?}] is part of scope @{scope_id:?}, but that scope has already completed"
    ))
}

#[cold]
pub fn non_const_declaration_hoisting(name: &str, kind: impl Debug) -> OxcDiagnostic {
    diagnostic(ErrorCategory::Todo, "Handle non-const declarations for hoisting")
        .with_help(format!("variable \"{name}\" declared with {kind:?}"))
}

#[cold]
pub fn unsupported_declaration_hoisting(name: &str, kind: &str) -> OxcDiagnostic {
    diagnostic(ErrorCategory::Todo, "Unsupported declaration type for hoisting")
        .with_help(format!("variable \"{name}\" declared with {kind}"))
}

#[cold]
pub fn missing_parameter_binding(name: &str, span: Span) -> OxcDiagnostic {
    diagnostic(ErrorCategory::Invariant, "Could not find binding")
        .with_help(format!("[BuildHIR] Could not find binding for param `{name}`"))
        .with_label(span.primary_label("Could not find binding"))
}

#[cold]
pub fn unsupported_eval(span: Span) -> OxcDiagnostic {
    diagnostic(ErrorCategory::UnsupportedSyntax, "The 'eval' function is not supported")
        .with_help(
            "Eval is an anti-pattern in JavaScript, and the code executed cannot be evaluated by React Compiler",
        )
        .with_label(span.primary_label("`eval` cannot be analyzed by React Compiler"))
}

#[cold]
pub fn const_reassignment(
    name: &str,
    reassignment_span: Span,
    declaration_span: Option<Span>,
) -> OxcDiagnostic {
    let mut diagnostic = diagnostic(ErrorCategory::Syntax, "Cannot reassign a `const` variable")
        .with_help(format!("`{name}` is declared as const"))
        .with_label(reassignment_span.primary_label(format!("Cannot reassign `{name}`")));
    diagnostic
        .labels
        .extend(declaration_span.map(|span| span.label(format!("`{name}` is declared here"))));
    diagnostic
}

#[cold]
pub fn unsupported_with_statement(span: Span) -> OxcDiagnostic {
    diagnostic(ErrorCategory::UnsupportedSyntax, "JavaScript 'with' syntax is not supported")
        .with_help(
            "'with' syntax is considered deprecated and removed from JavaScript standards, consider alternatives",
        )
        .with_label(span.primary_label("`with` cannot be analyzed by React Compiler"))
}

#[cold]
pub fn unsupported_inline_class(span: Span) -> OxcDiagnostic {
    diagnostic(ErrorCategory::UnsupportedSyntax, "Inline `class` declarations are not supported")
        .with_help("Move class declarations outside of components/hooks")
        .with_label(span.primary_label("Move this class outside the component or hook"))
}

#[cold]
pub fn undefined_ssa_identifier(name: &str, span: Option<Span>) -> OxcDiagnostic {
    diagnostic(
        ErrorCategory::Todo,
        "[hoisting] EnterSSA: Expected identifier to be defined before being used",
    )
    .with_help(format!("Identifier {name} is undefined"))
    .with_labels(
        span.map(|span| span.primary_label(format!("`{name}` is used before it is defined"))),
    )
}

#[cold]
pub fn invalid_module_type(module: &str, expect_hook: bool, span: Option<Span>) -> OxcDiagnostic {
    diagnostic(ErrorCategory::Config, "Invalid type configuration for module")
        .with_help(format!(
            "Expected type for `import ... from '{module}'` {} based on the module name",
            if expect_hook { "to be a hook" } else { "not to be a hook" }
        ))
        .with_labels(
            span.map(|span| {
                span.primary_label(format!("Invalid type configuration for `{module}`"))
            }),
        )
}

#[cold]
pub fn expected_inline_memo_function(span: Option<Span>) -> OxcDiagnostic {
    const MESSAGE: &str = "Expected the first argument to be an inline function expression";
    diagnostic(ErrorCategory::UseMemo, MESSAGE)
        .with_help("Pass an inline function expression as the first argument")
        .with_labels(span.map(|span| span.primary_label(MESSAGE)))
}

#[cold]
pub fn expected_simple_memo_dependencies(span: Option<Span>) -> OxcDiagnostic {
    const MESSAGE: &str = "Expected the dependency list to be an array of simple expressions (e.g. `x`, `x.y.z`, `x?.y?.z`)";
    diagnostic(ErrorCategory::UseMemo, MESSAGE)
        .with_help("Use an array literal containing identifiers or property access expressions")
        .with_labels(span.map(|span| span.primary_label(MESSAGE)))
}

#[cold]
pub fn empty_goto(block: impl Display, span: Option<Span>) -> OxcDiagnostic {
    diagnostic(ErrorCategory::Invariant, "Unexpected empty block with `goto` terminal")
        .with_help(format!("Block bb{block} is empty"))
        .with_labels(
            span.map(|span| span.primary_label("Unexpected empty block with `goto` terminal")),
        )
}

#[cold]
pub fn duplicate_fbt_tags<I>(tag_name: &str, name: &str, locations: I) -> OxcDiagnostic
where
    I: IntoIterator<Item = Span>,
{
    diagnostic(ErrorCategory::Todo, "Support duplicate fbt tags")
        .with_help(format!(
            "Support `<{tag_name}>` tags with multiple `<{tag_name}:{name}>` values"
        ))
        .with_labels(
            locations
                .into_iter()
                .map(|span| span.label(format!("Multiple `<{tag_name}:{name}>` tags found"))),
        )
}

#[cold]
pub fn invalid_jsx_namespace(namespace: &str, name: &str, span: Option<Span>) -> OxcDiagnostic {
    diagnostic(
        ErrorCategory::Syntax,
        "Expected JSXNamespacedName to have no colons in the namespace or name",
    )
    .with_help(format!("Got `{namespace}` : `{name}`"))
    .with_labels(
        span.map(|span| span.primary_label("JSX namespace names cannot contain additional colons")),
    )
}

#[cold]
pub fn ssa_variable_already_defined(place: &str, span: Option<Span>) -> OxcDiagnostic {
    invariant_with_help_and_reason_label(
        "Expected variable not to be defined prior to declaration",
        format!("{place} was already defined"),
        span,
    )
}

#[cold]
pub fn ssa_inconsistent_unnamed_const(
    kind: &str,
    place: &str,
    span: Option<Span>,
) -> OxcDiagnostic {
    invariant_with_help_and_reason_label(
        "Expected consistent kind for destructuring",
        format!("other places were `{kind}` but '{place}' is const"),
        span,
    )
}

#[cold]
pub fn ssa_inconsistent_const(kind: &str, place: &str, span: Option<Span>) -> OxcDiagnostic {
    invariant_with_help_and_reason_label(
        "Expected consistent kind for destructuring",
        format!("Other places were `{kind}` but '{place}' is const"),
        span,
    )
}

#[cold]
pub fn ssa_inconsistent_reassignment(kind: &str, place: &str, span: Option<Span>) -> OxcDiagnostic {
    invariant_with_help_and_reason_label(
        "Expected consistent kind for destructuring",
        format!("Other places were `{kind}` but '{place}' is reassigned"),
        span,
    )
}

#[cold]
pub fn ssa_dce_reassignment(span: Option<Span>) -> OxcDiagnostic {
    const MESSAGE: &str = "TODO: Handle reassignment in a value block where the original declaration was removed by dead code elimination (DCE)";
    diagnostic(ErrorCategory::Invariant, MESSAGE)
        .with_labels(span.map(|span| span.primary_label(MESSAGE)))
}

#[cold]
pub fn ssa_expected_operand() -> OxcDiagnostic {
    diagnostic(ErrorCategory::Invariant, "Expected at least one operand")
}

#[cold]
pub fn ssa_missing_declaration(place: &str, span: Option<Span>) -> OxcDiagnostic {
    invariant_with_help_and_reason_label(
        "Expected variable to have been defined",
        format!("No declaration for {place}"),
        span,
    )
}

#[cold]
pub fn break_block_already_scheduled(block: impl Display) -> OxcDiagnostic {
    diagnostic(ErrorCategory::Invariant, format!("Break block is already scheduled: bb{block}"))
}

#[cold]
pub fn continue_block_already_scheduled(block: impl Display) -> OxcDiagnostic {
    diagnostic(ErrorCategory::Invariant, format!("Continue block is already scheduled: bb{block}"))
}

#[cold]
pub fn if_consequent_already_scheduled(block: impl Display) -> OxcDiagnostic {
    diagnostic(
        ErrorCategory::Invariant,
        format!("Unexpected 'if' where consequent is already scheduled (bb{block})"),
    )
}

#[cold]
pub fn if_alternate_already_scheduled(block: impl Display) -> OxcDiagnostic {
    diagnostic(
        ErrorCategory::Invariant,
        format!("Unexpected 'if' where the alternate is already scheduled (bb{block})"),
    )
}

#[cold]
pub fn unexpected_collection_instruction(context_name: &str, actual: impl Debug) -> OxcDiagnostic {
    diagnostic(
        ErrorCategory::Invariant,
        format!(
            "Expected a StoreLocal or Destructure in {context_name} collection, found {actual:?}"
        ),
    )
}

#[cold]
pub fn unexpected_collection_variable(kind: impl Debug, context_name: &str) -> OxcDiagnostic {
    diagnostic(
        ErrorCategory::Invariant,
        format!("Unexpected {kind:?} variable in {context_name} collection"),
    )
}

#[cold]
pub fn unpruned_hoisted_instruction(kind: impl Debug) -> OxcDiagnostic {
    diagnostic(
        ErrorCategory::Invariant,
        format!("Expected {kind:?} to have been pruned in PruneHoistedContexts"),
    )
}

#[cold]
pub fn invalid_method_call_property(actual: &str, span: Option<Span>) -> OxcDiagnostic {
    diagnostic(
        ErrorCategory::Invariant,
        "[Codegen] Internal error: MethodCall::property must be an unpromoted + unmemoized MemberExpression",
    )
    .with_labels(span.map(|span| span.primary_label(format!("Got: '{actual}'"))))
}

#[cold]
pub fn unexpected_codegen_instruction(actual: impl Debug) -> OxcDiagnostic {
    diagnostic(
        ErrorCategory::Invariant,
        format!("Unexpected {actual:?} in codegenInstructionValue"),
    )
}

#[cold]
pub fn missing_codegen_temporary(id: impl Display, span: Option<Span>) -> OxcDiagnostic {
    let reason = format!("[Codegen] No value found for temporary, identifier id={id}");
    diagnostic(ErrorCategory::Invariant, &reason)
        .with_labels(span.map(|span| span.primary_label(reason)))
}

#[cold]
pub fn expected_object_method_lvalue() -> OxcDiagnostic {
    diagnostic(ErrorCategory::Invariant, "Expected object methods to have a temp lvalue")
}

#[cold]
pub fn unexpected_store_local_codegen() -> OxcDiagnostic {
    diagnostic(ErrorCategory::Invariant, "Unexpected StoreLocal in codegenInstructionValue")
}

#[cold]
pub fn enter_ssa_cycle(block: impl Display) -> OxcDiagnostic {
    diagnostic(ErrorCategory::Invariant, format!("found a cycle! visiting bb{block} again"))
}

#[cold]
pub fn expected_scope_terminal(block: impl Display) -> OxcDiagnostic {
    diagnostic(
        ErrorCategory::Invariant,
        format!("Expected block bb{block} to end in a scope terminal"),
    )
}

#[cold]
pub fn unexpected_unknown_effect(span: impl Debug) -> OxcDiagnostic {
    diagnostic(ErrorCategory::Invariant, format!("Unexpected unknown effect at {span:?}"))
}

#[cold]
pub fn cannot_resolve_shape(shape_id: impl Display) -> OxcDiagnostic {
    diagnostic(
        ErrorCategory::Invariant,
        format!("[HIR] Forget internal error: cannot resolve shape {shape_id}"),
    )
}

#[cold]
pub fn invalid_mutable_range(
    scope_id: impl Display,
    start: impl Display,
    end: impl Display,
    max: impl Display,
) -> OxcDiagnostic {
    diagnostic(
        ErrorCategory::Invariant,
        format!(
            "Invalid mutable range for scope: Scope @{scope_id} has range [{start}:{end}] but the valid range is [1:{max}]"
        ),
    )
}

#[cold]
pub fn expected_memo_callback(
    kind_name: &str,
    is_callback: bool,
    span: Option<Span>,
) -> OxcDiagnostic {
    diagnostic(
        ErrorCategory::UseMemo,
        format!("Expected a callback function to be passed to {kind_name}"),
    )
    .with_help(if is_callback {
        "The first argument to useCallback() must be a function to cache"
    } else {
        "The first argument to useMemo() must be a function that calculates a result to cache"
    })
    .with_labels(span.map(|span| {
        span.primary_label(if is_callback {
            "Expected a callback function"
        } else {
            "Expected a memoization function"
        })
    }))
}

#[cold]
pub fn expected_memo_dependency_array(kind_name: &str, span: Option<Span>) -> OxcDiagnostic {
    let message = format!("Expected the dependency list for {kind_name} to be an array literal");
    diagnostic(ErrorCategory::UseMemo, &message)
        .with_help(format!("Pass an array literal as the dependency list for {kind_name}"))
        .with_labels(span.map(|span| span.primary_label(message)))
}

#[cold]
pub fn unexpected_optional_terminal(kind: impl Debug) -> OxcDiagnostic {
    diagnostic(ErrorCategory::Invariant, format!("Unexpected terminal kind in optional: {kind:?}"))
}

#[cold]
pub fn unvisited_dominator_predecessor(block: impl Debug) -> OxcDiagnostic {
    diagnostic(
        ErrorCategory::Invariant,
        format!("At least one predecessor must have been visited for block {block:?}"),
    )
}

#[cold]
pub fn suppression(reason: &str, description: String, span: Span) -> OxcDiagnostic {
    diagnostic(ErrorCategory::Suppression, "React rule suppression prevents optimization")
        .with_help("Remove the suppression and address the reported React rule violation")
        .with_note(format!("{reason}. {description}"))
        .with_label(span.primary_label("Found React rule suppression"))
}

#[cold]
pub fn capitalized_call(name: &str, span: Option<Span>) -> OxcDiagnostic {
    diagnostic(ErrorCategory::CapitalizedCalls, "Capitalized function called without JSX")
        .with_help(format!(
            "Render `{name}` with JSX if it is a component; otherwise rename it to start with a lowercase letter or allowlist it in the compiler configuration"
        ))
        .with_note(format!(
            "`{name}` is treated as a component because it begins with an uppercase letter; React Compiler skipped optimizing this component or hook"
        ))
        .with_labels(span.map(|span| span.primary_label(format!("`{name}` may be a component"))))
}

#[cold]
pub fn conditional_hook(span: Option<Span>) -> OxcDiagnostic {
    diagnostic(
        ErrorCategory::Hooks,
        "Hooks must always be called in a consistent order and may not be called conditionally",
    )
    .with_help("Call Hooks unconditionally at the top level of the component or custom Hook")
    .with_labels(span.map(|span| span.primary_label("This Hook is called conditionally")))
}

#[cold]
pub fn hook_used_as_value(span: Option<Span>) -> OxcDiagnostic {
    diagnostic(
        ErrorCategory::Hooks,
        "Hooks may not be referenced as normal values; they must be called",
    )
    .with_help("Call the Hook directly instead of passing or storing it as a value")
    .with_labels(span.map(|span| span.primary_label("This Hook is used as a value")))
}

#[cold]
pub fn dynamic_hook(span: Option<Span>, origin_span: Option<Span>) -> OxcDiagnostic {
    let mut diagnostic = diagnostic(
        ErrorCategory::Hooks,
        "Hooks must be the same function on every render, but this value may change over time",
    )
    .with_help("Call a statically known Hook instead of selecting a Hook dynamically")
    .with_labels(span.map(|span| span.primary_label("This Hook may change between renders")));
    diagnostic.labels.extend(
        origin_span
            .filter(|origin_span| Some(*origin_span) != span)
            .map(|span| span.label("This dynamic Hook value originates here")),
    );
    diagnostic
}

#[cold]
pub fn hook_in_function_expression(
    description: String,
    span: Option<Span>,
    function_span: Option<Span>,
) -> OxcDiagnostic {
    let mut diagnostic = diagnostic(
        ErrorCategory::Hooks,
        "Hooks must be called at the top level of a function component or custom Hook",
    )
    .with_help(description)
    .with_labels(
        span.map(|span| span.primary_label("This Hook is called inside a nested function")),
    );
    diagnostic.labels.extend(
        function_span
            .filter(|function_span| Some(*function_span) != span)
            .map(|span| span.label("This is the nested function")),
    );
    diagnostic
}

#[cold]
pub fn unknown_target_type(target_type: &str) -> OxcDiagnostic {
    diagnostic(ErrorCategory::Invariant, format!("Unknown target type: {target_type}"))
}

#[cold]
pub fn expected_break_target(block: impl Display) -> OxcDiagnostic {
    diagnostic(ErrorCategory::Invariant, format!("Expected a break target for bb{block}"))
}

#[cold]
pub fn block_already_emitted(block: impl Display) -> OxcDiagnostic {
    diagnostic(ErrorCategory::Invariant, format!("Block bb{block} was already emitted"))
}

#[cold]
pub fn unexpected_value_block_fallthrough(block: impl Display) -> OxcDiagnostic {
    diagnostic(
        ErrorCategory::Invariant,
        format!("Did not expect to reach the fallthrough of a value block (bb{block})"),
    )
}

#[cold]
pub fn expected_branch_terminal(terminal_kind: &str, actual: impl Debug) -> OxcDiagnostic {
    diagnostic(
        ErrorCategory::Invariant,
        format!("Expected a branch terminal for {terminal_kind} test block, got {actual:?}"),
    )
}

#[cold]
pub fn expected_continue_target(block: impl Display) -> OxcDiagnostic {
    diagnostic(
        ErrorCategory::Invariant,
        format!("Expected continue target to be scheduled for bb{block}"),
    )
}

#[cold]
pub fn unexpected_error_category(category: impl Debug) -> OxcDiagnostic {
    diagnostic(ErrorCategory::Invariant, format!("Unexpected error category: {category:?}"))
}

#[cold]
pub fn exhaustive_dependencies(
    category: ErrorCategory,
    reason: &str,
    description: String,
) -> OxcDiagnostic {
    diagnostic(category, reason).with_help(description)
}

#[cold]
pub fn unsupported_object_pattern_rest(kind: &str, span: Span) -> OxcDiagnostic {
    diagnostic(
        ErrorCategory::Todo,
        format!("(BuildHIR::lowerAssignment) Handle {kind} rest element in ObjectPattern"),
    )
    .with_label(span.primary_label(format!("Unsupported {kind} rest element")))
}

#[cold]
pub fn missing_function_declaration_binding(name: &str, span: Span) -> OxcDiagnostic {
    diagnostic(
        ErrorCategory::Invariant,
        format!("Could not find binding for function declaration `{name}`"),
    )
    .with_label(span.primary_label(format!("No binding was found for `{name}`")))
}

#[cold]
pub fn local_fbt_tag(tag_name: &str, span: Option<Span>) -> OxcDiagnostic {
    let reason = format!("<{tag_name}> tags should be module-level imports");
    diagnostic(ErrorCategory::Invariant, &reason)
        .with_labels(span.map(|span| span.primary_label(reason)))
}

#[cold]
pub fn unsupported_object_method(kind: &str, span: Span) -> OxcDiagnostic {
    diagnostic(
        ErrorCategory::Todo,
        format!("(BuildHIR::lowerExpression) Handle {kind} functions in ObjectExpression"),
    )
    .with_label(span.primary_label(format!("Unsupported {kind} function")))
}

#[cold]
pub fn unsafe_reorderable_expression(expression_type: &str, span: Span) -> OxcDiagnostic {
    diagnostic(
        ErrorCategory::Todo,
        format!(
            "(BuildHIR::node.lowerReorderableExpression) Expression type `{expression_type}` cannot be safely reordered"
        ),
    )
    .with_label(span.primary_label(format!("`{expression_type}` cannot be safely reordered")))
}

#[cold]
pub fn unexpected_for_in_of_declarations(count: usize, span: Span) -> OxcDiagnostic {
    diagnostic(
        ErrorCategory::Invariant,
        format!("Expected only one declaration in for-in/of init, got {count}"),
    )
    .with_label(span.primary_label(format!("Found {count} declarations here")))
}

#[cold]
pub fn unsupported_non_trivial_init(context_name: &str, span: Option<Span>) -> OxcDiagnostic {
    diagnostic(ErrorCategory::Todo, format!("Support non-trivial {context_name} inits"))
        .with_labels(
            span.map(|span| span.primary_label(format!("Non-trivial {context_name} initializer"))),
        )
}

#[cold]
pub fn static_component_during_render(
    component_span: Option<Span>,
    creation_span: Option<Span>,
) -> OxcDiagnostic {
    diagnostic(ErrorCategory::StaticComponents, "Cannot create components during render")
        .with_help(
            "Components created during render will reset their state each time they are created. Declare components outside of render",
        )
        .with_labels(
            component_span
                .map(|span| span.primary_label("This component is created during render")),
        )
        .and_labels(
            creation_span.map(|span| span.label("The component is created during render here")),
        )
}

#[cold]
pub fn known_mutable_function(
    variable_name: &str,
    function_span: Option<Span>,
    value_span: Option<Span>,
) -> OxcDiagnostic {
    let mut diagnostic = diagnostic(
        ErrorCategory::Immutability,
        "Cannot modify local variables after render completes",
    )
    .with_help(format!(
        "This argument is a function which may reassign or mutate {variable_name} after render, \
         which can cause inconsistent behavior on subsequent renders. \
         Consider using state instead",
    ));
    diagnostic.labels.extend(
        value_span.map(|span| span.primary_label(format!("This modifies {variable_name}"))),
    );
    // Do not add an enclosing callback as a second label: it can cover a large
    // function body without identifying anything beyond the mutation above.
    diagnostic.labels.extend(
        function_span
            .filter(|function_span| {
                value_span.is_none_or(|value_span| {
                    function_span.start > value_span.start || function_span.end < value_span.end
                })
            })
            .map(|span| {
                if value_span.is_some() {
                    span.label(format!(
                        "This function may (indirectly) reassign or modify {variable_name} after render"
                    ))
                } else {
                    span.primary_label(format!(
                        "This function may (indirectly) reassign or modify {variable_name} after render"
                    ))
                }
            }),
    );
    diagnostic
}

#[cold]
fn diagnostic_with_help_and_label(
    category: ErrorCategory,
    message: &'static str,
    help: &'static str,
    span: Option<Span>,
    label: &'static str,
) -> OxcDiagnostic {
    diagnostic(category, message)
        .with_help(help)
        .with_labels(span.map(|span| span.primary_label(label)))
}

#[cold]
pub fn set_state_in_use_memo(span: Option<Span>) -> OxcDiagnostic {
    diagnostic_with_help_and_label(
        ErrorCategory::RenderSetState,
        "Calling setState from useMemo may trigger an infinite loop",
        "Each time the memo callback is evaluated it will change state. This can cause a memoization dependency to change, running the memo function again and causing an infinite loop. Instead of setting state in useMemo(), prefer deriving the value during render",
        span,
        "Found setState() within useMemo()",
    )
}

#[cold]
pub fn set_state_in_render_with_keyed_state(span: Option<Span>) -> OxcDiagnostic {
    diagnostic_with_help_and_label(
        ErrorCategory::RenderSetState,
        "Cannot call setState during render",
        "Calling setState during render may trigger an infinite loop.\n\
         * To reset state when other state/props change, use `const [state, setState] = useKeyedState(initialState, key)` to reset `state` when `key` changes.\n\
         * To derive data from other state/props, compute the derived data during render without using state",
        span,
        "Found setState() in render",
    )
}

#[cold]
pub fn set_state_in_render(span: Option<Span>) -> OxcDiagnostic {
    diagnostic_with_help_and_label(
        ErrorCategory::RenderSetState,
        "Cannot call setState during render",
        "Calling setState during render may trigger an infinite loop.\n\
         * To reset state when other state/props change, store the previous value in state and update conditionally.\n\
         * To derive data from other state/props, compute the derived data during render without using state",
        span,
        "Found setState() in render",
    )
}

#[cold]
pub fn unused_use_memo(span: Span) -> OxcDiagnostic {
    diagnostic(ErrorCategory::VoidUseMemo, "useMemo() result is unused")
        .with_help(
            "This useMemo() value is unused. useMemo() is for computing and caching values, not for arbitrary side effects",
        )
        .with_label(span.primary_label("useMemo() result is unused"))
}

#[cold]
pub fn use_memo_callback_parameters(span: Option<Span>) -> OxcDiagnostic {
    diagnostic_with_help_and_label(
        ErrorCategory::UseMemo,
        "useMemo() callbacks may not accept parameters",
        "useMemo() callbacks are called by React to cache calculations across re-renders. They should not take parameters. Instead, directly reference the props, state, or local variables needed for the computation",
        span,
        "Callbacks with parameters are not supported",
    )
}

#[cold]
pub fn async_or_generator_use_memo(span: Option<Span>) -> OxcDiagnostic {
    diagnostic_with_help_and_label(
        ErrorCategory::UseMemo,
        "useMemo() callbacks may not be async or generator functions",
        "useMemo() callbacks are called once and must synchronously return a value",
        span,
        "Async and generator functions are not supported",
    )
}

#[cold]
pub fn use_memo_no_return(span: Option<Span>) -> OxcDiagnostic {
    diagnostic_with_help_and_label(
        ErrorCategory::VoidUseMemo,
        "useMemo() callbacks must return a value",
        "This useMemo() callback doesn't return a value. useMemo() is for computing and caching values, not for arbitrary side effects",
        span,
        "useMemo() callbacks must return a value",
    )
}

#[cold]
pub fn use_memo_reassigns_outer_variable(
    span: Option<Span>,
    origin_span: Option<Span>,
) -> OxcDiagnostic {
    let mut diagnostic = diagnostic_with_help_and_label(
        ErrorCategory::UseMemo,
        "useMemo() callbacks may not reassign variables declared outside of the callback",
        "useMemo() callbacks must be pure functions and cannot reassign variables defined outside of the callback function",
        span,
        "Cannot reassign variable",
    );
    diagnostic.labels.extend(
        origin_span
            .filter(|origin_span| Some(*origin_span) != span)
            .map(|span| span.label("This variable is captured from outside the callback")),
    );
    diagnostic
}

const REF_ACCESS_HELP: &str = "React refs are values that are not needed for rendering. \
    Refs should only be accessed outside of render, such as in event handlers or effects. \
    Accessing a ref value (the `current` property) during render can cause your component \
    not to update as expected";

#[cold]
fn ref_access(span: Option<Span>, label: &'static str) -> OxcDiagnostic {
    diagnostic(ErrorCategory::Refs, "Cannot access refs during render")
        .with_help(REF_ACCESS_HELP)
        .with_labels(span.map(|span| span.primary_label(label)))
}

#[cold]
pub fn ref_value_access(span: Option<Span>) -> OxcDiagnostic {
    ref_access(span, "Cannot access ref value during render")
}

#[cold]
pub fn ref_passed_to_function(span: Option<Span>) -> OxcDiagnostic {
    ref_access(span, "Passing a ref to a function may read its value during render")
}

#[cold]
pub fn ref_update(span: Option<Span>, ref_span: Option<Span>) -> OxcDiagnostic {
    let ref_value_span = match (span, ref_span) {
        (Some(span), Some(ref_span)) if span.start == ref_span.start && ref_span.end < span.end => {
            Some(Span::new(ref_span.end, span.end))
        }
        _ => span,
    };
    let mut diagnostic = ref_access(ref_value_span, "Cannot update ref value during render");
    diagnostic.labels.extend(
        ref_span
            .filter(|ref_span| {
                ref_value_span.is_none_or(|value_span| {
                    ref_span.end <= value_span.start || ref_span.start >= value_span.end
                })
            })
            .map(|span| span.label("This value is a ref")),
    );
    diagnostic
}

#[cold]
pub fn function_accesses_ref(span: Option<Span>, ref_access_span: Option<Span>) -> OxcDiagnostic {
    let mut diagnostic = ref_access(span, "This function accesses a ref value");
    diagnostic.labels.extend(
        ref_access_span
            .filter(|ref_access_span| Some(*ref_access_span) != span)
            .map(|span| span.label("The ref is accessed here")),
    );
    diagnostic
}

#[cold]
pub fn set_state_in_effect(
    span: Option<Span>,
    effect_span: Option<Span>,
    verbose: bool,
) -> OxcDiagnostic {
    let help = if verbose {
        "Effects are intended to synchronize state between React and external systems. \
         Calling setState synchronously causes cascading renders that hurt performance.\n\n\
         This pattern may indicate one of several issues:\n\n\
         **1. Non-local derived data**: If the value being set could be computed from props/state \
         but requires data from a parent component, consider restructuring state ownership so the \
         derivation can happen during render in the component that owns the relevant state.\n\n\
         **2. Derived event pattern**: If you're detecting when a prop changes (e.g., `isPlaying` \
         transitioning from false to true), this often indicates the parent should provide an event \
         callback (like `onPlay`) instead of just the current state. Request access to the original event.\n\n\
         **3. Force update / external sync**: If you're forcing a re-render to sync with an external \
         data source (mutable values outside React), use `useSyncExternalStore` to properly subscribe \
         to external state changes."
    } else {
        "Effects should synchronize React with external systems. Calling setState synchronously inside an effect starts another render and is usually unnecessary. \
         Derive the value during render, initialize state directly, or update it from the event that caused the change. Use an effect only when synchronizing with an external system."
    };
    let mut diagnostic = diagnostic_with_help_and_label(
        ErrorCategory::EffectSetState,
        "Calling setState synchronously within an effect can trigger cascading renders",
        help,
        span,
        "Avoid calling setState() directly within an effect",
    );
    diagnostic.labels.extend(
        effect_span
            .filter(|effect_span| Some(*effect_span) != span)
            .map(|span| span.label("This is the containing effect")),
    );
    diagnostic
}

#[cold]
pub fn preserve_memo_mutated_dependency(span: Option<Span>) -> OxcDiagnostic {
    diagnostic(ErrorCategory::PreserveManualMemo, "Existing memoization could not be preserved")
        .with_help(
            "React Compiler has skipped optimizing this component because the existing manual memoization could not be preserved. \
             This dependency may be mutated later, which could cause the value to change unexpectedly",
        )
        .with_labels(span.map(|span| span.primary_label("This dependency may be modified later")))
}

#[cold]
pub fn preserve_memo_unmemoized(
    span: Option<Span>,
    callback_start_span: Option<Span>,
) -> OxcDiagnostic {
    let mut diagnostic =
        diagnostic(ErrorCategory::PreserveManualMemo, "Existing memoization could not be preserved")
        .with_help(
            "React Compiler could not prove that this useMemo/useCallback remains memoized. Fix related React Compiler errors inside the callback first. If manual memoization is not required for semantics, remove it; otherwise restructure the callback to avoid values that invalidate memoization",
        )
        .with_labels(span.map(|span| span.primary_label("Manual memoization is not preserved here")));
    diagnostic.labels.extend(
        callback_start_span
            .filter(|callback_start_span| {
                span.is_none_or(|primary_span| {
                    callback_start_span.end <= primary_span.start
                        || callback_start_span.start >= primary_span.end
                })
            })
            .map(|span| span.label("Manual memoization callback starts here")),
    );
    diagnostic
}

#[cold]
pub fn preserve_memo_inferred_dependencies(
    description: String,
    dependency_list_span: Option<Span>,
    inferred_dependency_span: Option<Span>,
    fallback_span: Option<Span>,
) -> OxcDiagnostic {
    let mut diagnostic = diagnostic(
        ErrorCategory::PreserveManualMemo,
        "Existing memoization could not be preserved",
    )
    .with_help(description);
    let primary_span = dependency_list_span.or(inferred_dependency_span).or(fallback_span);
    diagnostic.labels.extend(primary_span.map(|span| {
        span.primary_label(if dependency_list_span.is_some() {
            "This dependency list does not match the dependencies inferred from the callback"
        } else {
            "Could not preserve existing manual memoization"
        })
    }));
    diagnostic.labels.extend(
        inferred_dependency_span
            .filter(|span| Some(*span) != primary_span)
            .map(|span| span.label("This dependency is inferred here")),
    );
    diagnostic
}

#[cold]
pub fn jsx_in_try(span: Option<Span>) -> OxcDiagnostic {
    diagnostic(ErrorCategory::ErrorBoundaries, "Avoid constructing JSX within try/catch")
        .with_help(
            "React does not immediately render components when JSX is constructed, so rendering errors will not be caught by the try/catch. Wrap the component in an error boundary instead",
        )
        .with_labels(span.map(|span| span.primary_label("Avoid constructing JSX within try/catch")))
}

#[cold]
pub fn inconsistent_context_variable(
    place: &str,
    kind: impl Display,
    previous_kind: impl Display,
    span: Option<Span>,
) -> OxcDiagnostic {
    diagnostic(
        ErrorCategory::Invariant,
        "Expected all references to a variable to be consistently local or context references",
    )
    .with_help(format!(
        "Identifier {place} is referenced as a {kind} variable, but was previously referenced as a {previous_kind} variable"
    ))
    .with_labels(span.map(|span| span.primary_label(format!("this is {previous_kind}"))))
}

#[cold]
pub fn reassigned_after_render(
    variable: &str,
    reassignment_span: Option<Span>,
    declaration_span: Option<Span>,
) -> OxcDiagnostic {
    let mut diagnostic =
        diagnostic(ErrorCategory::Immutability, "Cannot reassign variable after render completes")
            .with_help(format!(
                "Reassigning {variable} after render has completed can cause inconsistent \
             behavior on subsequent renders. Consider using state instead"
            ))
            .with_labels(reassignment_span.map(|span| {
                span.primary_label(format!("Cannot reassign {variable} after render completes"))
            }));
    diagnostic.labels.extend(
        declaration_span
            .filter(|span| Some(*span) != reassignment_span)
            .map(|span| span.label(format!("{variable} is declared here"))),
    );
    diagnostic
}

#[cold]
pub fn reassigned_in_async_function(
    variable: &str,
    reassignment_span: Option<Span>,
    declaration_span: Option<Span>,
) -> OxcDiagnostic {
    let mut diagnostic =
        diagnostic(ErrorCategory::Immutability, "Cannot reassign variable in async function")
            .with_help(
                "Reassigning a variable in an async function can cause \
             inconsistent behavior on subsequent renders. \
             Consider using state instead",
            )
            .with_labels(
                reassignment_span
                    .map(|span| span.primary_label(format!("Cannot reassign {variable}"))),
            );
    diagnostic.labels.extend(
        declaration_span
            .filter(|span| Some(*span) != reassignment_span)
            .map(|span| span.label(format!("{variable} is declared here"))),
    );
    diagnostic
}

#[cold]
pub fn derived_state_in_effect(description: String, span: Option<Span>) -> OxcDiagnostic {
    diagnostic(
        ErrorCategory::EffectDerivationsOfState,
        "You might not need an effect. Derive values in render, not effects.",
    )
    .with_help(description)
    .with_labels(
        span.map(|span| {
            span.primary_label("This should be computed during render, not in an effect")
        }),
    )
}

#[cold]
pub fn derived_state_in_effect_from_dependencies(
    description: String,
    span: Option<Span>,
    dependency_spans: impl IntoIterator<Item = Span>,
) -> OxcDiagnostic {
    let mut diagnostic = diagnostic(
        ErrorCategory::EffectDerivationsOfState,
        "Values derived from props and state should be calculated during render, not in an effect",
    )
    .with_help(description)
    .with_labels(span.map(|span| {
        span.primary_label("This state update stores a value that can be calculated during render")
    }));
    let mut dependency_spans = dependency_spans.into_iter().peekable();
    if let Some(span) = dependency_spans.next() {
        let label = if dependency_spans.peek().is_some() {
            "These reactive values contribute to the derived state"
        } else {
            "This reactive value contributes to the derived state"
        };
        diagnostic.labels.push(span.label(label));
        diagnostic.labels.extend(dependency_spans.map(Into::into));
    }
    diagnostic
}

#[cold]
pub fn uninitialized_value(description: String, span: Option<Span>) -> OxcDiagnostic {
    diagnostic(
        ErrorCategory::Invariant,
        "[InferMutationAliasingEffects] Expected value kind to be initialized",
    )
    .with_help(description)
    .with_labels(span.map(|span| span.primary_label("this is uninitialized")))
}

#[cold]
pub fn immutable_value(
    reason: impl AsRef<str>,
    variable: &str,
    mutation_span: Option<Span>,
    declaration_span: Option<Span>,
) -> OxcDiagnostic {
    let mut diagnostic = diagnostic(ErrorCategory::Immutability, "This value cannot be modified")
        .with_help(reason.as_ref().to_string())
        .with_labels(
            mutation_span.map(|span| span.primary_label(format!("{variable} cannot be modified"))),
        );
    diagnostic.labels.extend(
        declaration_span
            .filter(|span| Some(*span) != mutation_span)
            .map(|span| span.label(format!("{variable} originates here"))),
    );
    diagnostic
}

#[cold]
pub fn incompatible_library(reason: impl AsRef<str>, span: Option<Span>) -> OxcDiagnostic {
    diagnostic(ErrorCategory::IncompatibleLibrary, "Use of incompatible library")
        .with_help(
            "This API returns functions which cannot be memoized without leading to stale UI. \
             To prevent this, by default React Compiler will skip memoizing this component/hook. \
             However, you may see issues if values from this API are passed to other components/hooks that are \
             memoized",
        )
        .with_labels(span.map(|span| span.primary_label(reason.as_ref().to_string())))
}

#[cold]
pub fn variable_accessed_before_declaration(
    variable: Option<&str>,
    access_span: Option<Span>,
    declaration_span: Option<Span>,
) -> OxcDiagnostic {
    let help_name = variable.unwrap_or("This variable");
    let label_name = variable.unwrap_or("variable");
    let mut diagnostic = diagnostic(
        ErrorCategory::Immutability,
        "Cannot access variable while it is being initialized",
    )
    .with_help(format!(
        "{help_name} is read while its declaration is still being initialized. Move the access after initialization. For a recursive callback, use a named function expression or restructure the callback so it does not capture itself during initialization"
    ));
    diagnostic.labels.extend(access_span.map(|span| {
        span.primary_label(format!("{label_name} is read during its own initialization"))
    }));
    diagnostic.labels.extend(
        declaration_span.map(|span| span.label(format!("{label_name} is initialized here"))),
    );
    diagnostic
}

#[cold]
pub fn global_reassignment(variable: &str, span: Option<Span>) -> OxcDiagnostic {
    diagnostic(
        ErrorCategory::Globals,
        "Cannot reassign variables declared outside of the component/hook",
    )
    .with_help(format!(
        "Variable {variable} is declared outside of the component/hook. Reassigning this value during render is a side effect which can cause unpredictable behavior. If this variable is used in rendering, use useState instead. Otherwise, update it in an effect"
    ))
    .with_labels(span.map(|span| span.primary_label(format!("{variable} cannot be reassigned"))))
}

#[cold]
pub fn impure_function(name: Option<&str>, span: Option<Span>) -> OxcDiagnostic {
    let prefix = name.map_or_else(String::new, |name| format!("`{name}` is an impure function. "));
    diagnostic(ErrorCategory::Purity, "Cannot call impure function during render")
        .with_help(format!(
            "{prefix}Calling an impure function can produce unstable results that update unpredictably when the component re-renders"
        ))
        .with_labels(span.map(|span| span.primary_label("Cannot call impure function")))
}

pub fn is_unexpected_error(diagnostic: &OxcDiagnostic) -> bool {
    ErrorCategory::Invariant.matches(diagnostic) && diagnostic.message == "unexpected error"
}

#[cold]
pub fn pipeline_error(span: Option<Span>) -> OxcDiagnostic {
    diagnostic(ErrorCategory::Invariant, "Unexpected pipeline error")
        .with_help(
            "Please report this internal React Compiler error to Oxc with a minimal reproduction",
        )
        .with_note("The compiler pipeline stopped before this component or hook could be optimized")
        .with_labels(span.map(|span| span.primary_label("The compiler pipeline failed here")))
}

/// A lint finding paired with its [`ErrorCategory`].
///
/// The diagnostic's message contains only the user-facing reason; its category
/// is carried separately in the structured diagnostic code.
#[derive(Debug, Clone)]
pub struct LintDiagnostic {
    pub category: ErrorCategory,
    pub diagnostic: OxcDiagnostic,
}

/// Pair a compiler diagnostic with the category in its structured error code.
pub(crate) fn categorize(diagnostic: OxcDiagnostic) -> LintDiagnostic {
    let category = diagnostic
        .code
        .scope
        .as_deref()
        .filter(|scope| *scope == ErrorCategory::CODE_SCOPE)
        .and_then(|_| diagnostic.code.number.as_deref())
        .and_then(ErrorCategory::from_name);
    debug_assert!(category.is_some(), "missing React Compiler diagnostic category");
    LintDiagnostic { category: category.unwrap_or(ErrorCategory::Invariant), diagnostic }
}

/// Whether any diagnostic is an error at the TS compiler's *internal*
/// severity, which decides `panicThreshold: critical_errors`. Internal and
/// displayed severity agree except for `PreserveManualMemo`, which displays
/// as an error but is internally a warning (it must not trigger the panic
/// threshold).
pub fn has_critical_errors(diagnostics: &[OxcDiagnostic]) -> bool {
    diagnostics
        .iter()
        .any(|d| d.severity == Severity::Error && !ErrorCategory::PreserveManualMemo.matches(d))
}

/// Whether diagnostics should abort compilation for the configured panic threshold.
///
/// Config errors are always fatal, matching the upstream compiler.
pub fn should_panic(diagnostics: &[OxcDiagnostic], panic_threshold: PanicThreshold) -> bool {
    diagnostics.iter().any(|d| ErrorCategory::Config.matches(d))
        || match panic_threshold {
            PanicThreshold::AllErrors => true,
            PanicThreshold::CriticalErrors => has_critical_errors(diagnostics),
            PanicThreshold::None => false,
        }
}

/// Owned copy of a diagnostic for the log accumulator. Promote the first
/// existing location to primary, or use `fallback_span` when the diagnostic has
/// no source location of its own.
#[cold]
pub fn with_fallback_label(
    diagnostic: &OxcDiagnostic,
    fallback_span: Option<Span>,
) -> OxcDiagnostic {
    let mut diagnostic = diagnostic.clone();
    if diagnostic.labels.is_empty() {
        match fallback_span {
            Some(span) => {
                let label = diagnostic.message.to_string();
                diagnostic.with_label(span.primary_label(label))
            }
            None => diagnostic,
        }
    } else {
        if !diagnostic.labels.iter().any(LabeledSpan::primary) {
            diagnostic.labels[0] = primary_label(diagnostic.labels[0].clone());
        }
        diagnostic
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_fields_are_populated_for_every_category() {
        let categories = [
            ErrorCategory::Hooks,
            ErrorCategory::CapitalizedCalls,
            ErrorCategory::UseMemo,
            ErrorCategory::PreserveManualMemo,
            ErrorCategory::IncompatibleLibrary,
            ErrorCategory::Immutability,
            ErrorCategory::Globals,
            ErrorCategory::Refs,
            ErrorCategory::EffectSetState,
            ErrorCategory::EffectDerivationsOfState,
            ErrorCategory::ErrorBoundaries,
            ErrorCategory::Purity,
            ErrorCategory::RenderSetState,
            ErrorCategory::StaticComponents,
            ErrorCategory::Config,
            ErrorCategory::Gating,
            ErrorCategory::Todo,
            ErrorCategory::Syntax,
            ErrorCategory::UnsupportedSyntax,
            ErrorCategory::Suppression,
            ErrorCategory::VoidUseMemo,
            ErrorCategory::MemoDependencies,
            ErrorCategory::EffectExhaustiveDependencies,
            ErrorCategory::Invariant,
        ];

        for category in categories {
            let diagnostic = diagnostic(category, "Example diagnostic");
            assert_eq!(diagnostic.message, "Example diagnostic");
            assert!(category.matches(&diagnostic));
            assert!(diagnostic.help.is_some());
            assert!(diagnostic.note.is_some());
            assert!(diagnostic.url.is_some());
        }
    }

    #[test]
    fn hook_diagnostic_uses_a_title_and_primary_label() {
        let diagnostic = conditional_hook(Some(Span::new(4, 11)));

        assert_eq!(
            diagnostic.message,
            "Hooks must always be called in a consistent order and may not be called conditionally"
        );
        assert!(!diagnostic.message.contains("http"));
        assert!(!diagnostic.help.as_deref().unwrap().contains("http"));
        assert_eq!(diagnostic.labels.len(), 1);
        assert!(diagnostic.labels[0].primary());
        assert_eq!(diagnostic.labels[0].label(), Some("This Hook is called conditionally"));
    }

    #[test]
    fn related_location_is_a_secondary_label() {
        let diagnostic = variable_accessed_before_declaration(
            Some("value"),
            Some(Span::new(1, 6)),
            Some(Span::new(9, 14)),
        );

        assert!(diagnostic.labels[0].primary());
        assert!(!diagnostic.labels[1].primary());
        assert_eq!(diagnostic.labels[1].label(), Some("value is initialized here"));
    }

    #[test]
    fn fallback_location_is_labeled_and_primary() {
        let diagnostic = invariant_expected_node_all_scopes();
        let diagnostic = with_fallback_label(&diagnostic, Some(Span::new(2, 8)));

        assert_eq!(diagnostic.labels.len(), 1);
        assert!(diagnostic.labels[0].primary());
        assert_eq!(diagnostic.labels[0].label(), Some(diagnostic.message.as_ref()));
    }

    #[test]
    fn existing_location_is_promoted_to_primary() {
        let diagnostic = todo_support_destructuring_context_variables([Span::new(4, 9)]);
        assert!(!diagnostic.labels[0].primary());

        let diagnostic = with_fallback_label(&diagnostic, Some(Span::new(20, 30)));
        assert_eq!(diagnostic.labels.len(), 1);
        assert!(diagnostic.labels[0].primary());
        assert_eq!(diagnostic.labels[0].span(), Span::new(4, 9));
    }

    #[test]
    fn memo_dependency_mismatch_labels_both_locations() {
        let diagnostic = preserve_memo_inferred_dependencies(
            "dependency mismatch".to_string(),
            Some(Span::new(20, 25)),
            Some(Span::new(4, 9)),
            Some(Span::new(1, 30)),
        );

        assert!(diagnostic.labels[0].primary());
        assert_eq!(diagnostic.labels[0].span(), Span::new(20, 25));
        assert!(!diagnostic.labels[1].primary());
        assert_eq!(diagnostic.labels[1].span(), Span::new(4, 9));
    }

    #[test]
    fn unexpected_error_uses_structured_category() {
        assert!(is_unexpected_error(&diagnostic(ErrorCategory::Invariant, "unexpected error")));
        assert!(!is_unexpected_error(&diagnostic(ErrorCategory::Syntax, "unexpected error")));
    }
}
