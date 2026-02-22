/// Compiler error and diagnostic types.
///
/// Port of `CompilerError.ts` from the React Compiler.
///
/// Provides structured error types for the compiler pipeline, including
/// error categories, severity levels, diagnostics, and suggestions.
use std::fmt;

use oxc_span::Span;

/// Sentinel value indicating a generated source location (no corresponding source).
///
/// In the original TS, this is `Symbol()` / `GeneratedSource`.
/// In Rust we use `Span::default()` (an empty span) to represent generated source.
pub const GENERATED_SOURCE: SourceLocation = SourceLocation::Generated;

/// A source location, either from source code or generated.
///
/// Replaces `t.SourceLocation | typeof GeneratedSource` from the TS version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SourceLocation {
    /// A real source location with a span.
    Source(Span),
    /// A generated/synthetic location with no corresponding source.
    #[default]
    Generated,
}

impl From<Span> for SourceLocation {
    fn from(span: Span) -> Self {
        SourceLocation::Source(span)
    }
}

/// Severity of a compiler error/diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorSeverity {
    /// An actionable error that the developer can fix. For example, product code errors
    /// should be reported as such.
    Error,
    /// An error that the developer may not necessarily be able to fix. For example, syntax
    /// not supported by the compiler does not indicate any fault in the product code.
    Warning,
    /// Not an error. These will not be surfaced in ESLint, but may be surfaced in other ways
    /// (eg Forgive) where informational hints can be shown.
    Hint,
    /// These errors will not be reported anywhere. Useful for work in progress validations.
    Off,
}

impl fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorSeverity::Error => write!(f, "Error"),
            ErrorSeverity::Warning => write!(f, "Warning"),
            ErrorSeverity::Hint => write!(f, "Hint"),
            ErrorSeverity::Off => write!(f, "Off"),
        }
    }
}

/// Category of a compiler error. Each category maps to an ESLint rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCategory {
    /// Checking for valid hooks usage (non conditional, non-first class, non reactive, etc)
    Hooks,
    /// Checking for no capitalized calls (not definitively an error, hence separating)
    CapitalizedCalls,
    /// Checking for static components
    StaticComponents,
    /// Checking for valid usage of manual memoization
    UseMemo,
    /// Checking that useMemos always return a value
    VoidUseMemo,
    /// Checks that manual memoization is preserved
    PreserveManualMemo,
    /// Checks for exhaustive useMemo/useCallback dependencies without extraneous values
    MemoDependencies,
    /// Checks for known incompatible libraries
    IncompatibleLibrary,
    /// Checking for no mutations of props, hook arguments, hook return values
    Immutability,
    /// Checking for assignments to globals
    Globals,
    /// Checking for valid usage of refs, ie no access during render
    Refs,
    /// Checks for memoized effect deps
    EffectDependencies,
    /// Checks for exhaustive and extraneous effect dependencies
    EffectExhaustiveDependencies,
    /// Checks for no setState in effect bodies
    EffectSetState,
    /// Effect derivations of state
    EffectDerivationsOfState,
    /// Validates against try/catch in place of error boundaries
    ErrorBoundaries,
    /// Checking for pure functions
    Purity,
    /// Validates against setState in render
    RenderSetState,
    /// Internal invariants
    Invariant,
    /// Todos
    Todo,
    /// Syntax errors
    Syntax,
    /// Checks for use of unsupported syntax
    UnsupportedSyntax,
    /// Config errors
    Config,
    /// Gating error
    Gating,
    /// Suppressions
    Suppression,
    /// fbt-specific issues
    Fbt,
}

impl fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

/// Operation type for a compiler suggestion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompilerSuggestionOperation {
    InsertBefore,
    InsertAfter,
    Remove,
    Replace,
}

/// A suggestion for fixing a compiler error.
#[derive(Debug, Clone)]
pub enum CompilerSuggestion {
    /// Insert, replace, or similar operations that include replacement text.
    TextEdit {
        op: CompilerSuggestionOperation,
        range: (u32, u32),
        description: String,
        text: String,
    },
    /// Remove operation (no replacement text needed).
    Remove { range: (u32, u32), description: String },
}

/// Detail of a compiler diagnostic — either an error source or a hint.
#[derive(Debug, Clone)]
pub enum CompilerDiagnosticDetail {
    /// A/the source of the error
    Error { loc: Option<SourceLocation>, message: Option<String> },
    /// A hint to help fix the error
    Hint { message: String },
}

/// Options for creating a `CompilerDiagnostic`.
#[derive(Debug, Clone)]
pub struct CompilerDiagnosticOptions {
    pub category: ErrorCategory,
    pub reason: String,
    pub description: Option<String>,
    pub details: Vec<CompilerDiagnosticDetail>,
    pub suggestions: Option<Vec<CompilerSuggestion>>,
}

/// A structured compiler diagnostic with category, reason, details, and optional suggestions.
#[derive(Debug, Clone)]
pub struct CompilerDiagnostic {
    pub options: CompilerDiagnosticOptions,
}

impl CompilerDiagnostic {
    /// Create a new diagnostic without details (details can be added with `with_details`).
    pub fn create(
        category: ErrorCategory,
        reason: String,
        description: Option<String>,
        suggestions: Option<Vec<CompilerSuggestion>>,
    ) -> Self {
        Self {
            options: CompilerDiagnosticOptions {
                category,
                reason,
                description,
                details: Vec::new(),
                suggestions,
            },
        }
    }

    pub fn reason(&self) -> &str {
        &self.options.reason
    }

    pub fn description(&self) -> Option<&str> {
        self.options.description.as_deref()
    }

    pub fn severity(&self) -> ErrorSeverity {
        get_rule_for_category(self.category()).severity
    }

    pub fn suggestions(&self) -> Option<&[CompilerSuggestion]> {
        self.options.suggestions.as_deref()
    }

    pub fn category(&self) -> ErrorCategory {
        self.options.category
    }

    /// Add details to this diagnostic.
    #[must_use]
    pub fn with_details(mut self, details: Vec<CompilerDiagnosticDetail>) -> Self {
        self.options.details.extend(details);
        self
    }

    /// Add a single detail to this diagnostic.
    #[must_use]
    pub fn with_detail(mut self, detail: CompilerDiagnosticDetail) -> Self {
        self.options.details.push(detail);
        self
    }

    /// Returns the primary source location from the first error detail.
    pub fn primary_location(&self) -> Option<SourceLocation> {
        self.options.details.iter().find_map(|d| match d {
            CompilerDiagnosticDetail::Error { loc, .. } => loc.as_ref().copied(),
            CompilerDiagnosticDetail::Hint { .. } => None,
        })
    }
}

impl fmt::Display for CompilerDiagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", print_error_summary(self.category(), self.reason()))?;
        if let Some(desc) = self.description() {
            write!(f, ". {desc}.")?;
        }
        Ok(())
    }
}

/// Options for creating a `CompilerErrorDetail` (deprecated, use `CompilerDiagnostic`).
#[derive(Debug, Clone)]
pub struct CompilerErrorDetailOptions {
    pub category: ErrorCategory,
    pub reason: String,
    pub description: Option<String>,
    pub loc: Option<SourceLocation>,
    pub suggestions: Option<Vec<CompilerSuggestion>>,
}

/// A single error detail (deprecated — use `CompilerDiagnostic` instead).
///
/// Each bailout or invariant in HIR lowering creates a `CompilerErrorDetail`,
/// which is then aggregated into a single `CompilerError` later.
#[derive(Debug, Clone)]
pub struct CompilerErrorDetail {
    pub options: CompilerErrorDetailOptions,
}

impl CompilerErrorDetail {
    pub fn new(options: CompilerErrorDetailOptions) -> Self {
        Self { options }
    }

    pub fn reason(&self) -> &str {
        &self.options.reason
    }

    pub fn description(&self) -> Option<&str> {
        self.options.description.as_deref()
    }

    pub fn severity(&self) -> ErrorSeverity {
        get_rule_for_category(self.category()).severity
    }

    pub fn loc(&self) -> Option<SourceLocation> {
        self.options.loc
    }

    pub fn suggestions(&self) -> Option<&[CompilerSuggestion]> {
        self.options.suggestions.as_deref()
    }

    pub fn category(&self) -> ErrorCategory {
        self.options.category
    }

    pub fn primary_location(&self) -> Option<SourceLocation> {
        self.options.loc
    }
}

impl fmt::Display for CompilerErrorDetail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", print_error_summary(self.category(), self.reason()))?;
        if let Some(desc) = self.description() {
            write!(f, ". {desc}.")?;
        }
        Ok(())
    }
}

/// A single entry in a `CompilerError`, either a diagnostic or a legacy error detail.
#[derive(Debug, Clone)]
pub enum CompilerErrorEntry {
    Diagnostic(CompilerDiagnostic),
    Detail(CompilerErrorDetail),
}

impl CompilerErrorEntry {
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            CompilerErrorEntry::Diagnostic(d) => d.severity(),
            CompilerErrorEntry::Detail(d) => d.severity(),
        }
    }
}

impl fmt::Display for CompilerErrorEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompilerErrorEntry::Diagnostic(d) => write!(f, "{d}"),
            CompilerErrorEntry::Detail(d) => write!(f, "{d}"),
        }
    }
}

/// An aggregate of compiler diagnostics/errors.
///
/// This allows aggregating all issues found by the compiler into a single error
/// before propagation. Where possible, prefer to push diagnostics into the error
/// aggregate instead of returning immediately.
#[derive(Debug, Clone)]
pub struct CompilerError {
    pub details: Vec<CompilerErrorEntry>,
    pub disabled_details: Vec<CompilerErrorEntry>,
}

impl Default for CompilerError {
    fn default() -> Self {
        Self::new()
    }
}

impl CompilerError {
    pub fn new() -> Self {
        Self { details: Vec::new(), disabled_details: Vec::new() }
    }

    /// Check an invariant condition. Returns `Err(CompilerError)` if the condition is false.
    ///
    /// # Errors
    /// Returns a `CompilerError` with category `Invariant` if `condition` is false.
    pub fn invariant_result(
        condition: bool,
        reason: &str,
        description: Option<&str>,
        loc: SourceLocation,
    ) -> Result<(), CompilerError> {
        if condition {
            return Ok(());
        }
        let mut errors = CompilerError::new();
        errors.push_diagnostic(
            CompilerDiagnostic::create(
                ErrorCategory::Invariant,
                reason.to_string(),
                description.map(ToString::to_string),
                None,
            )
            .with_detail(CompilerDiagnosticDetail::Error {
                loc: Some(loc),
                message: Some(reason.to_string()),
            }),
        );
        Err(errors)
    }

    /// Create a CompilerError for a todo item.
    pub fn todo(reason: &str, description: Option<&str>, loc: SourceLocation) -> CompilerError {
        let mut errors = CompilerError::new();
        errors.push_error_detail(CompilerErrorDetail::new(CompilerErrorDetailOptions {
            category: ErrorCategory::Todo,
            reason: reason.to_string(),
            description: description.map(ToString::to_string),
            loc: Some(loc),
            suggestions: None,
        }));
        errors
    }

    /// Create a CompilerError for invalid JS.
    pub fn invalid_js(
        reason: &str,
        description: Option<&str>,
        loc: SourceLocation,
    ) -> CompilerError {
        let mut errors = CompilerError::new();
        errors.push_error_detail(CompilerErrorDetail::new(CompilerErrorDetailOptions {
            category: ErrorCategory::Syntax,
            reason: reason.to_string(),
            description: description.map(ToString::to_string),
            loc: Some(loc),
            suggestions: None,
        }));
        errors
    }

    /// Create a CompilerError for invalid config.
    pub fn invalid_config(
        reason: &str,
        description: Option<&str>,
        loc: Option<SourceLocation>,
    ) -> CompilerError {
        let mut errors = CompilerError::new();
        errors.push_error_detail(CompilerErrorDetail::new(CompilerErrorDetailOptions {
            category: ErrorCategory::Config,
            reason: reason.to_string(),
            description: description.map(ToString::to_string),
            loc,
            suggestions: None,
        }));
        errors
    }

    /// Create a CompilerError for an invariant violation.
    pub fn invariant(
        reason: &str,
        description: Option<&str>,
        loc: SourceLocation,
    ) -> CompilerError {
        let mut errors = CompilerError::new();
        errors.push_diagnostic(
            CompilerDiagnostic::create(
                ErrorCategory::Invariant,
                reason.to_string(),
                description.map(ToString::to_string),
                None,
            )
            .with_detail(CompilerDiagnosticDetail::Error {
                loc: Some(loc),
                message: Some(reason.to_string()),
            }),
        );
        errors
    }

    /// Merge another `CompilerError` into this one.
    pub fn merge(&mut self, other: CompilerError) {
        self.details.extend(other.details);
        self.disabled_details.extend(other.disabled_details);
    }

    /// Push a new diagnostic entry.
    pub fn push_diagnostic(&mut self, diagnostic: CompilerDiagnostic) {
        if diagnostic.severity() == ErrorSeverity::Off {
            self.disabled_details.push(CompilerErrorEntry::Diagnostic(diagnostic));
        } else {
            self.details.push(CompilerErrorEntry::Diagnostic(diagnostic));
        }
    }

    /// Push a legacy error detail.
    pub fn push_error_detail(&mut self, detail: CompilerErrorDetail) {
        if detail.severity() == ErrorSeverity::Off {
            self.disabled_details.push(CompilerErrorEntry::Detail(detail));
        } else {
            self.details.push(CompilerErrorEntry::Detail(detail));
        }
    }

    /// Returns `true` if there are any error details (active, not disabled).
    pub fn has_any_errors(&self) -> bool {
        !self.details.is_empty()
    }

    /// Convert to a `Result`. Returns `Err(self)` if there are errors, `Ok(())` otherwise.
    ///
    /// # Errors
    /// Returns `Err(self)` when there are any active errors in this aggregate.
    pub fn into_result(self) -> Result<(), CompilerError> {
        if self.has_any_errors() { Err(self) } else { Ok(()) }
    }

    /// Returns `true` if any of the error details are of severity `Error`.
    pub fn has_errors(&self) -> bool {
        self.details.iter().any(|d| d.severity() == ErrorSeverity::Error)
    }

    /// Returns `true` if there are no `Error`s and there is at least one `Warning`.
    pub fn has_warning(&self) -> bool {
        let mut has_warn = false;
        for detail in &self.details {
            if detail.severity() == ErrorSeverity::Error {
                return false;
            }
            if detail.severity() == ErrorSeverity::Warning {
                has_warn = true;
            }
        }
        has_warn
    }

    /// Returns `true` if there are only `Hint` severity details (no errors or warnings).
    pub fn has_hints(&self) -> bool {
        let mut has_hint = false;
        for detail in &self.details {
            match detail.severity() {
                ErrorSeverity::Error | ErrorSeverity::Warning => return false,
                ErrorSeverity::Hint => has_hint = true,
                ErrorSeverity::Off => {}
            }
        }
        has_hint
    }
}

impl fmt::Display for CompilerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.details.is_empty() {
            return write!(f, "ReactCompilerError");
        }
        let mut first = true;
        for detail in &self.details {
            if !first {
                write!(f, "\n\n")?;
            }
            write!(f, "{detail}")?;
            first = false;
        }
        Ok(())
    }
}

impl std::error::Error for CompilerError {}

/// Preset classification for lint rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LintRulePreset {
    /// Rules that are stable and included in the `recommended` preset.
    Recommended,
    /// Rules that are more experimental and only included in the `recommended-latest` preset.
    RecommendedLatest,
    /// Rules that are disabled.
    Off,
}

/// A lint rule definition derived from an error category.
#[derive(Debug, Clone)]
pub struct LintRule {
    pub category: ErrorCategory,
    pub severity: ErrorSeverity,
    pub name: &'static str,
    pub description: &'static str,
    pub preset: LintRulePreset,
}

/// Get the lint rule definition for a given error category.
pub fn get_rule_for_category(category: ErrorCategory) -> LintRule {
    match category {
        ErrorCategory::CapitalizedCalls => LintRule {
            category,
            severity: ErrorSeverity::Error,
            name: "capitalized-calls",
            description: "Validates against calling capitalized functions/methods instead of using JSX",
            preset: LintRulePreset::Off,
        },
        ErrorCategory::Config => LintRule {
            category,
            severity: ErrorSeverity::Error,
            name: "config",
            description: "Validates the compiler configuration options",
            preset: LintRulePreset::Recommended,
        },
        ErrorCategory::EffectDependencies => LintRule {
            category,
            severity: ErrorSeverity::Error,
            name: "memoized-effect-dependencies",
            description: "Validates that effect dependencies are memoized",
            preset: LintRulePreset::Off,
        },
        ErrorCategory::EffectExhaustiveDependencies => LintRule {
            category,
            severity: ErrorSeverity::Error,
            name: "exhaustive-effect-dependencies",
            description: "Validates that effect dependencies are exhaustive and without extraneous values",
            preset: LintRulePreset::Off,
        },
        ErrorCategory::EffectDerivationsOfState => LintRule {
            category,
            severity: ErrorSeverity::Error,
            name: "no-deriving-state-in-effects",
            description: "Validates against deriving values from state in an effect",
            preset: LintRulePreset::Off,
        },
        ErrorCategory::EffectSetState => LintRule {
            category,
            severity: ErrorSeverity::Error,
            name: "set-state-in-effect",
            description: "Validates against calling setState synchronously in an effect",
            preset: LintRulePreset::Recommended,
        },
        ErrorCategory::ErrorBoundaries => LintRule {
            category,
            severity: ErrorSeverity::Error,
            name: "error-boundaries",
            description: "Validates usage of error boundaries instead of try/catch for errors in child components",
            preset: LintRulePreset::Recommended,
        },
        ErrorCategory::Fbt => LintRule {
            category,
            severity: ErrorSeverity::Error,
            name: "fbt",
            description: "Validates usage of fbt",
            preset: LintRulePreset::Off,
        },
        ErrorCategory::Gating => LintRule {
            category,
            severity: ErrorSeverity::Error,
            name: "gating",
            description: "Validates configuration of gating mode",
            preset: LintRulePreset::Recommended,
        },
        ErrorCategory::Globals => LintRule {
            category,
            severity: ErrorSeverity::Error,
            name: "globals",
            description: "Validates against assignment/mutation of globals during render",
            preset: LintRulePreset::Recommended,
        },
        ErrorCategory::Hooks => LintRule {
            category,
            severity: ErrorSeverity::Error,
            name: "hooks",
            description: "Validates the rules of hooks",
            preset: LintRulePreset::Off,
        },
        ErrorCategory::Immutability => LintRule {
            category,
            severity: ErrorSeverity::Error,
            name: "immutability",
            description: "Validates against mutating props, state, and other values that are immutable",
            preset: LintRulePreset::Recommended,
        },
        ErrorCategory::Invariant => LintRule {
            category,
            severity: ErrorSeverity::Error,
            name: "invariant",
            description: "Internal invariants",
            preset: LintRulePreset::Off,
        },
        ErrorCategory::PreserveManualMemo => LintRule {
            category,
            severity: ErrorSeverity::Error,
            name: "preserve-manual-memoization",
            description: "Validates that existing manual memoization is preserved by the compiler",
            preset: LintRulePreset::Recommended,
        },
        ErrorCategory::Purity => LintRule {
            category,
            severity: ErrorSeverity::Error,
            name: "purity",
            description: "Validates that components/hooks are pure by checking that they do not call known-impure functions",
            preset: LintRulePreset::Recommended,
        },
        ErrorCategory::Refs => LintRule {
            category,
            severity: ErrorSeverity::Error,
            name: "refs",
            description: "Validates correct usage of refs, not reading/writing during render",
            preset: LintRulePreset::Recommended,
        },
        ErrorCategory::RenderSetState => LintRule {
            category,
            severity: ErrorSeverity::Error,
            name: "set-state-in-render",
            description: "Validates against setting state during render",
            preset: LintRulePreset::Recommended,
        },
        ErrorCategory::StaticComponents => LintRule {
            category,
            severity: ErrorSeverity::Error,
            name: "static-components",
            description: "Validates that components are static, not recreated every render",
            preset: LintRulePreset::Recommended,
        },
        ErrorCategory::Suppression => LintRule {
            category,
            severity: ErrorSeverity::Error,
            name: "rule-suppression",
            description: "Validates against suppression of other rules",
            preset: LintRulePreset::Off,
        },
        ErrorCategory::Syntax => LintRule {
            category,
            severity: ErrorSeverity::Error,
            name: "syntax",
            description: "Validates against invalid syntax",
            preset: LintRulePreset::Off,
        },
        ErrorCategory::Todo => LintRule {
            category,
            severity: ErrorSeverity::Hint,
            name: "todo",
            description: "Unimplemented features",
            preset: LintRulePreset::Off,
        },
        ErrorCategory::UnsupportedSyntax => LintRule {
            category,
            severity: ErrorSeverity::Warning,
            name: "unsupported-syntax",
            description: "Validates against syntax that we do not plan to support in React Compiler",
            preset: LintRulePreset::Recommended,
        },
        ErrorCategory::UseMemo => LintRule {
            category,
            severity: ErrorSeverity::Error,
            name: "use-memo",
            description: "Validates usage of the useMemo() hook against common mistakes",
            preset: LintRulePreset::Recommended,
        },
        ErrorCategory::VoidUseMemo => LintRule {
            category,
            severity: ErrorSeverity::Error,
            name: "void-use-memo",
            description: "Validates that useMemos always return a value and that the result is used",
            preset: LintRulePreset::RecommendedLatest,
        },
        ErrorCategory::MemoDependencies => LintRule {
            category,
            severity: ErrorSeverity::Error,
            name: "memo-dependencies",
            description: "Validates that useMemo() and useCallback() specify comprehensive dependencies",
            preset: LintRulePreset::Off,
        },
        ErrorCategory::IncompatibleLibrary => LintRule {
            category,
            severity: ErrorSeverity::Warning,
            name: "incompatible-library",
            description: "Validates against usage of libraries which are incompatible with memoization",
            preset: LintRulePreset::Recommended,
        },
    }
}

fn print_error_summary(category: ErrorCategory, message: &str) -> String {
    let heading = match category {
        ErrorCategory::CapitalizedCalls
        | ErrorCategory::Config
        | ErrorCategory::EffectDerivationsOfState
        | ErrorCategory::EffectSetState
        | ErrorCategory::ErrorBoundaries
        | ErrorCategory::Fbt
        | ErrorCategory::Gating
        | ErrorCategory::Globals
        | ErrorCategory::Hooks
        | ErrorCategory::Immutability
        | ErrorCategory::Purity
        | ErrorCategory::Refs
        | ErrorCategory::RenderSetState
        | ErrorCategory::StaticComponents
        | ErrorCategory::Suppression
        | ErrorCategory::Syntax
        | ErrorCategory::UseMemo
        | ErrorCategory::VoidUseMemo
        | ErrorCategory::MemoDependencies
        | ErrorCategory::EffectExhaustiveDependencies => "Error",
        ErrorCategory::EffectDependencies
        | ErrorCategory::IncompatibleLibrary
        | ErrorCategory::PreserveManualMemo
        | ErrorCategory::UnsupportedSyntax => "Compilation Skipped",
        ErrorCategory::Invariant => "Invariant",
        ErrorCategory::Todo => "Todo",
    };
    format!("{heading}: {message}")
}

/// Get all lint rules for all error categories.
pub fn all_lint_rules() -> Vec<LintRule> {
    let categories = [
        ErrorCategory::Hooks,
        ErrorCategory::CapitalizedCalls,
        ErrorCategory::StaticComponents,
        ErrorCategory::UseMemo,
        ErrorCategory::VoidUseMemo,
        ErrorCategory::PreserveManualMemo,
        ErrorCategory::MemoDependencies,
        ErrorCategory::IncompatibleLibrary,
        ErrorCategory::Immutability,
        ErrorCategory::Globals,
        ErrorCategory::Refs,
        ErrorCategory::EffectDependencies,
        ErrorCategory::EffectExhaustiveDependencies,
        ErrorCategory::EffectSetState,
        ErrorCategory::EffectDerivationsOfState,
        ErrorCategory::ErrorBoundaries,
        ErrorCategory::Purity,
        ErrorCategory::RenderSetState,
        ErrorCategory::Invariant,
        ErrorCategory::Todo,
        ErrorCategory::Syntax,
        ErrorCategory::UnsupportedSyntax,
        ErrorCategory::Config,
        ErrorCategory::Gating,
        ErrorCategory::Suppression,
        ErrorCategory::Fbt,
    ];
    categories.into_iter().map(get_rule_for_category).collect()
}
