#![allow(clippy::self_named_module_files)]

pub use oxc_linter_core::ModuleRecord;
pub use oxc_linter_core::{
    AllowWarnDeny, AstNode, ExternalPluginStore, FixKind, Fixer, LintOptions, LintPlugins,
    LintServiceOptions, Oxlintrc, RuleCategory, RuleFixMeta, RuleMeta, RuleRunFunctionsImplemented,
    RuleRunner, RuntimeFileSystem, ast_util, config, context, disable_directives,
    external_plugin_store, fixer, frameworks, globals, loader, module_graph_visitor, module_record,
    options, read_to_arena_str, rule, service, timing, utils,
};

pub mod rules;
pub mod table;

mod generated {
    mod rule_runner_impls;
    pub mod rules_enum;
}

#[cfg(test)]
mod tester;

pub use generated::rules_enum::*;
pub use table::{RuleTable, RuleTableRow, RuleTableSection};

pub type Config = oxc_linter_core::Config<RuleEnum>;
pub type ConfigStore = oxc_linter_core::ConfigStore<RuleEnum>;
pub type ConfigStoreBuilder = oxc_linter_core::ConfigStoreBuilder<RuleEnum>;
pub type Linter = oxc_linter_core::Linter<RuleEnum>;
pub type LintService = oxc_linter_core::LintService<RuleEnum>;
pub type LintRunner = oxc_linter_core::LintRunner<RuleEnum>;
pub type LintRunnerBuilder = oxc_linter_core::LintRunnerBuilder<RuleEnum>;
pub type ResolvedLinterState = oxc_linter_core::ResolvedLinterState<RuleEnum>;
pub type TsGoLintState = oxc_linter_core::TsGoLintState<RuleEnum>;

use oxc_linter_core::{
    context::{ContextHost, LintContext},
    rule::BuiltinRule,
    timing::RuleTimingStat,
    utils::PossibleJestNode,
};

impl BuiltinRule for RuleEnum {
    fn all_rules() -> &'static [Self] {
        RULES.as_slice()
    }

    fn id(&self) -> usize {
        Self::id(self)
    }

    fn name(&self) -> &'static str {
        Self::name(self)
    }

    fn plugin_name(&self) -> &'static str {
        Self::plugin_name(self)
    }

    fn category(&self) -> RuleCategory {
        Self::category(self)
    }

    fn fix(&self) -> RuleFixMeta {
        Self::fix(self)
    }

    fn has_config(&self) -> bool {
        Self::has_config(self)
    }

    fn is_tsgolint_rule(&self) -> bool {
        Self::is_tsgolint_rule(self)
    }

    fn from_configuration(
        &self,
        value: serde_json::Value,
    ) -> Result<Self, serde_json::error::Error> {
        Self::from_configuration(self, value)
    }

    fn to_configuration(&self) -> Option<Result<serde_json::Value, serde_json::Error>> {
        Self::to_configuration(self)
    }

    fn types_info(&self) -> Option<&'static oxc_semantic::AstTypesBitset> {
        Self::types_info(self)
    }

    fn run_info(&self) -> RuleRunFunctionsImplemented {
        Self::run_info(self)
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        Self::should_run(self, ctx)
    }

    fn run<'a, const TIMINGS: bool>(
        &self,
        node: &AstNode<'a>,
        ctx: &LintContext<'a>,
        timing_stat: Option<&mut RuleTimingStat>,
    ) {
        Self::run::<TIMINGS>(self, node, ctx, timing_stat);
    }

    fn run_once<const TIMINGS: bool>(
        &self,
        ctx: &LintContext<'_>,
        timing_stat: Option<&mut RuleTimingStat>,
    ) {
        Self::run_once::<TIMINGS>(self, ctx, timing_stat);
    }

    fn run_on_jest_node<'a, 'c, const TIMINGS: bool>(
        &self,
        jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
        timing_stat: Option<&mut RuleTimingStat>,
    ) {
        Self::run_on_jest_node::<TIMINGS>(self, jest_node, ctx, timing_stat);
    }

    #[cfg(feature = "ruledocs")]
    fn version(&self) -> &'static str {
        Self::version(self)
    }

    #[cfg(feature = "ruledocs")]
    fn documentation(&self) -> Option<&'static str> {
        Self::documentation(self)
    }

    #[cfg(feature = "ruledocs")]
    fn schema(
        &self,
        generator: &mut schemars::SchemaGenerator,
    ) -> Option<schemars::schema::Schema> {
        Self::schema(self, generator)
    }
}

#[cfg(target_pointer_width = "64")]
#[test]
fn size_asserts() {
    assert_eq!(std::mem::size_of::<RuleEnum>(), 16);
}
