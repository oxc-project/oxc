use crate::AllowWarnDeny;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Options for the linter.
#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct OxlintOptions {
    /// Enable rules that require type information.
    ///
    /// Equivalent to passing `--type-aware` on the CLI.
    ///
    /// Note that this requires the `oxlint-tsgolint` package to be installed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_aware: Option<bool>,
    /// Enable experimental type checking (includes TypeScript compiler diagnostics).
    ///
    /// Equivalent to passing `--type-check` on the CLI.
    ///
    /// Note that this requires the `oxlint-tsgolint` package to be installed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_check: Option<bool>,
    /// Ensure warnings produce a non-zero exit code.
    ///
    /// Equivalent to passing `--deny-warnings` on the CLI.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deny_warnings: Option<bool>,
    /// Specify a warning threshold. Exits with an error status if warnings exceed this value.
    ///
    /// Equivalent to passing `--max-warnings` on the CLI.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_warnings: Option<usize>,
    /// Report unused disable directives (e.g. `// oxlint-disable-line` or `// eslint-disable-line`).
    ///
    /// Equivalent to passing `--report-unused-disable-directives-severity` on the CLI.
    /// CLI flags take precedence over this value when both are set.
    /// Only supported in the root configuration file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub report_unused_disable_directives: Option<AllowWarnDeny>,
    /// Whether oxlint should respect `eslint-disable*` and `eslint-enable*`
    /// directives in addition to its native `oxlint-*` directives.
    ///
    /// Defaults to `true`.
    /// Only supported in the root configuration file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub respect_eslint_disable_directives: Option<bool>,
}

impl OxlintOptions {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.type_aware.is_none()
            && self.type_check.is_none()
            && self.deny_warnings.is_none()
            && self.max_warnings.is_none()
            && self.report_unused_disable_directives.is_none()
            && self.respect_eslint_disable_directives.is_none()
    }

    #[must_use]
    pub fn merge(&self, other: &OxlintOptions) -> Self {
        Self {
            type_aware: self.type_aware.or(other.type_aware),
            type_check: self.type_check.or(other.type_check),
            deny_warnings: self.deny_warnings.or(other.deny_warnings),
            max_warnings: self.max_warnings.or(other.max_warnings),
            report_unused_disable_directives: self
                .report_unused_disable_directives
                .or(other.report_unused_disable_directives),
            respect_eslint_disable_directives: self
                .respect_eslint_disable_directives
                .or(other.respect_eslint_disable_directives),
        }
    }
}
