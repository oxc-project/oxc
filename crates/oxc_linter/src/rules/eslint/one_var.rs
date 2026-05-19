use oxc_ast::{
    AstKind, AstType,
    ast::{
        Expression, ForStatementInit, ForStatementLeft, VariableDeclaration,
        VariableDeclarationKind, VariableDeclarator,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::{node::NodeId, scope::ScopeId};
use rustc_hash::FxHashMap;
use schemars::{
    JsonSchema,
    r#gen::SchemaGenerator,
    schema::{InstanceType, ObjectValidation, Schema, SchemaObject},
};
use serde::{
    Deserialize,
    de::{self, Error},
};
use serde_json::Value;
use smallvec::SmallVec;
use std::{borrow::Cow, ptr};

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    rule::Rule,
};

fn one_var_diagnostic(
    span: Span,
    kind: DiagnosticKind,
    declaration_type: &'static str,
) -> OxcDiagnostic {
    let message = match kind {
        DiagnosticKind::Combine => {
            format!("Combine this with the previous '{declaration_type}' statement.")
        }
        DiagnosticKind::CombineInitialized => format!(
            "Combine this with the previous '{declaration_type}' statement with initialized variables."
        ),
        DiagnosticKind::CombineUninitialized => format!(
            "Combine this with the previous '{declaration_type}' statement with uninitialized variables."
        ),
        DiagnosticKind::Split => {
            format!("Split '{declaration_type}' declarations into multiple statements.")
        }
        DiagnosticKind::SplitInitialized => {
            format!("Split initialized '{declaration_type}' declarations into multiple statements.")
        }
        DiagnosticKind::SplitUninitialized => format!(
            "Split uninitialized '{declaration_type}' declarations into multiple statements."
        ),
        DiagnosticKind::SplitRequires => {
            "Split requires to be separated into a single block.".to_string()
        }
    };

    OxcDiagnostic::warn(message).with_label(span)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
enum RawMode {
    Always,
    Never,
    Consecutive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum RuntimeMode {
    #[default]
    Off,
    Always,
    Never,
    Consecutive,
}

impl From<RawMode> for RuntimeMode {
    fn from(mode: RawMode) -> Self {
        match mode {
            RawMode::Always => Self::Always,
            RawMode::Never => Self::Never,
            RawMode::Consecutive => Self::Consecutive,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct KindPolicy {
    initialized: RuntimeMode,
    uninitialized: RuntimeMode,
}

#[derive(Debug, Clone)]
struct OneVarRuntimeConfig {
    separate_requires: bool,
    var: KindPolicy,
    let_: KindPolicy,
    const_: KindPolicy,
    using: KindPolicy,
    await_using: KindPolicy,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
struct OneVarKindConfig {
    separate_requires: Option<bool>,
    var: Option<RawMode>,
    #[serde(rename = "let")]
    let_mode: Option<RawMode>,
    #[serde(rename = "const")]
    const_mode: Option<RawMode>,
    using: Option<RawMode>,
    await_using: Option<RawMode>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
struct OneVarInitConfig {
    initialized: Option<RawMode>,
    uninitialized: Option<RawMode>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(untagged)]
enum OneVarOption {
    Mode(RawMode),
    PerKind(OneVarKindConfig),
    InitUninit(OneVarInitConfig),
}

#[derive(Debug, Clone)]
struct StrictOneVarConfig(OneVarRuntimeConfig);

impl Default for OneVarRuntimeConfig {
    fn default() -> Self {
        Self::all(RuntimeMode::Always)
    }
}

impl OneVarRuntimeConfig {
    fn all(mode: RuntimeMode) -> Self {
        let policy = KindPolicy { initialized: mode, uninitialized: mode };
        Self {
            separate_requires: false,
            var: policy,
            let_: policy,
            const_: policy,
            using: policy,
            await_using: policy,
        }
    }

    fn policy(&self, kind: VariableDeclarationKind) -> KindPolicy {
        match kind {
            VariableDeclarationKind::Var => self.var,
            VariableDeclarationKind::Let => self.let_,
            VariableDeclarationKind::Const => self.const_,
            VariableDeclarationKind::Using => self.using,
            VariableDeclarationKind::AwaitUsing => self.await_using,
        }
    }

    fn policies(&self) -> [KindPolicy; 5] {
        [self.var, self.let_, self.const_, self.using, self.await_using]
    }

    fn needs_consecutive(&self) -> bool {
        self.policies().into_iter().any(|policy| {
            policy.initialized == RuntimeMode::Consecutive
                || policy.uninitialized == RuntimeMode::Consecutive
        })
    }

    fn needs_always(&self) -> bool {
        self.policies().into_iter().any(|policy| {
            policy.initialized == RuntimeMode::Always || policy.uninitialized == RuntimeMode::Always
        })
    }

    fn needs_never(&self) -> bool {
        self.policies().into_iter().any(|policy| {
            policy.initialized == RuntimeMode::Never || policy.uninitialized == RuntimeMode::Never
        })
    }
}

fn pair_mode(mode: Option<RawMode>) -> KindPolicy {
    let mode = mode.map_or(RuntimeMode::Off, RuntimeMode::from);
    KindPolicy { initialized: mode, uninitialized: mode }
}

impl Default for OneVarOption {
    fn default() -> Self {
        Self::Mode(RawMode::Always)
    }
}

impl TryFrom<OneVarOption> for OneVarRuntimeConfig {
    type Error = &'static str;

    fn try_from(option: OneVarOption) -> Result<Self, Self::Error> {
        match option {
            OneVarOption::Mode(mode) => Ok(Self::all(mode.into())),
            OneVarOption::PerKind(config) => {
                let has_any = config.separate_requires.is_some()
                    || config.var.is_some()
                    || config.let_mode.is_some()
                    || config.const_mode.is_some()
                    || config.using.is_some()
                    || config.await_using.is_some();

                if !has_any {
                    return Err("Empty configuration object is not valid for `one-var`");
                }

                Ok(Self {
                    separate_requires: config.separate_requires.unwrap_or(false),
                    var: pair_mode(config.var),
                    let_: pair_mode(config.let_mode),
                    const_: pair_mode(config.const_mode),
                    using: pair_mode(config.using),
                    await_using: pair_mode(config.await_using),
                })
            }
            OneVarOption::InitUninit(config) => {
                if config.initialized.is_none() && config.uninitialized.is_none() {
                    return Err("Empty configuration object is not valid for `one-var`");
                }

                let policy = KindPolicy {
                    initialized: config.initialized.map_or(RuntimeMode::Off, RuntimeMode::from),
                    uninitialized: config.uninitialized.map_or(RuntimeMode::Off, RuntimeMode::from),
                };

                Ok(Self {
                    separate_requires: false,
                    var: policy,
                    let_: policy,
                    const_: policy,
                    using: policy,
                    await_using: policy,
                })
            }
        }
    }
}

impl<'de> Deserialize<'de> for StrictOneVarConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Option::<Value>::deserialize(deserializer)?;
        let values = match value {
            None => return Ok(Self(OneVarRuntimeConfig::default())),
            Some(Value::Array(values)) => values,
            Some(_) => return Err(de::Error::custom("Expected array for rule configuration")),
        };

        let option = match values.as_slice() {
            [] => OneVarOption::default(),
            [option @ (Value::String(_) | Value::Object(_))] => {
                OneVarOption::deserialize(option).map_err(D::Error::custom)?
            }
            [_] => {
                return Err(de::Error::custom(
                    "Expected a string, per-kind object, or initialized/uninitialized object",
                ));
            }
            _ => return Err(de::Error::custom("Expected at most one option for `one-var`")),
        };

        OneVarRuntimeConfig::try_from(option).map(Self).map_err(D::Error::custom)
    }
}

impl JsonSchema for OneVarKindConfig {
    fn schema_name() -> String {
        "OneVarKindConfig".to_string()
    }

    fn schema_id() -> Cow<'static, str> {
        "OneVarKindConfig".into()
    }

    fn json_schema(generator: &mut SchemaGenerator) -> Schema {
        SchemaObject {
            instance_type: Some(InstanceType::Object.into()),
            object: Some(Box::new(ObjectValidation {
                min_properties: Some(1),
                properties: [
                    ("separateRequires".to_string(), generator.subschema_for::<bool>()),
                    ("var".to_string(), generator.subschema_for::<RawMode>()),
                    ("let".to_string(), generator.subschema_for::<RawMode>()),
                    ("const".to_string(), generator.subschema_for::<RawMode>()),
                    ("using".to_string(), generator.subschema_for::<RawMode>()),
                    ("awaitUsing".to_string(), generator.subschema_for::<RawMode>()),
                ]
                .into_iter()
                .collect(),
                additional_properties: Some(Box::new(false.into())),
                ..Default::default()
            })),
            ..Default::default()
        }
        .into()
    }
}

impl JsonSchema for OneVarInitConfig {
    fn schema_name() -> String {
        "OneVarInitConfig".to_string()
    }

    fn schema_id() -> Cow<'static, str> {
        "OneVarInitConfig".into()
    }

    fn json_schema(generator: &mut SchemaGenerator) -> Schema {
        SchemaObject {
            instance_type: Some(InstanceType::Object.into()),
            object: Some(Box::new(ObjectValidation {
                min_properties: Some(1),
                properties: [
                    ("initialized".to_string(), generator.subschema_for::<RawMode>()),
                    ("uninitialized".to_string(), generator.subschema_for::<RawMode>()),
                ]
                .into_iter()
                .collect(),
                additional_properties: Some(Box::new(false.into())),
                ..Default::default()
            })),
            ..Default::default()
        }
        .into()
    }
}

#[derive(Debug, Default, Clone)]
pub struct OneVar(OneVarRuntimeConfig);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces variables to be declared either together or separately.
    ///
    /// ### Why is this bad?
    ///
    /// Mixing declaration styles within the same scope makes code harder to scan
    /// and reason about. This rule enforces a consistent policy for `var`,
    /// `let`, `const`, `using`, and `await using` declarations.
    ///
    /// ### Intentional differences from ESLint
    ///
    /// - `require(...)` is treated specially only when `separateRequires` is enabled.
    ///   Without that option, `require(...)` is treated like any other initialized declarator.
    /// - With `separateRequires`, Oxlint may also report the ordinary `combine` diagnostics
    ///   that become visible after conceptually splitting a mixed declaration such as
    ///   `var a = require("a"), b = 1;`, including on later declarations in the same scope.
    /// - TypeScript ambient declarations such as `declare global` and `declare module`
    ///   are still checked for declaration grouping. This is intentional because their
    ///   declaration style is still user-authored code.
    /// - `consecutive` also applies inside `SwitchCase.consequent`, which ESLint skips because
    ///   it only checks `parent.body`.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule with the default `"always"` option:
    /// ```js
    /// var a = 1;
    /// var b = 2;
    /// ```
    ///
    /// Examples of **incorrect** code for the `"never"` option:
    /// ```js
    /// /* one-var: ["error", "never"] */
    /// let a = 1, b = 2;
    /// ```
    ///
    /// Examples of **correct** code for this rule with the default `"always"` option:
    /// ```js
    /// var a = 1, b = 2;
    /// ```
    ///
    /// Examples of **correct** code for the `"never"` option:
    /// ```js
    /// /* one-var: ["error", "never"] */
    /// let a = 1;
    /// let b = 2;
    /// ```
    OneVar,
    eslint,
    style,
    pending,
    config = OneVarOption,
    version = "next",
);

const VARIABLE_DECLARATION_NODE_TYPES: &AstTypesBitset =
    &AstTypesBitset::from_types(&[AstType::VariableDeclaration]);

impl Rule for OneVar {
    fn from_configuration(value: Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value::<StrictOneVarConfig>(value).map(|config| Self(config.0))
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        // Preserve the cheap file-level skip for files without variable declarations.
        ctx.semantic().nodes().contains_any(VARIABLE_DECLARATION_NODE_TYPES)
    }

    fn run_once(&self, ctx: &LintContext<'_>) {
        let needs_always = self.0.needs_always();
        let needs_never = self.0.needs_never();
        let needs_consecutive = self.0.needs_consecutive();

        if !needs_always && !needs_never && !needs_consecutive {
            return;
        }

        let mut previous_by_list: FxHashMap<NodeId, PreviousDeclaration> = FxHashMap::default();
        let mut always_state: FxHashMap<AlwaysStateKey, SeenState> = FxHashMap::default();
        let mut var_scope_cache: FxHashMap<ScopeId, ScopeId> = FxHashMap::default();

        // Process the semantic node table once instead of running a recursive AST visitor.
        // `previous_by_list` relies on source/preorder node order for statement-list adjacency.
        // Scope-wide checks keep per effective-scope state; `var` scope is resolved lazily.
        for node in ctx.semantic().nodes() {
            let node_id = node.id();
            let statement_list_id =
                needs_consecutive.then(|| statement_list_parent_id(ctx, node_id)).flatten();

            let AstKind::VariableDeclaration(decl) = node.kind() else {
                if let Some(statement_list_id) = statement_list_id {
                    previous_by_list.remove(&statement_list_id);
                }
                continue;
            };

            let policy = self.0.policy(decl.kind);
            let policy_needs_always = policy.initialized == RuntimeMode::Always
                || policy.uninitialized == RuntimeMode::Always;
            let policy_needs_never = policy.initialized == RuntimeMode::Never
                || policy.uninitialized == RuntimeMode::Never;
            let policy_needs_consecutive = policy.initialized == RuntimeMode::Consecutive
                || policy.uninitialized == RuntimeMode::Consecutive;
            let needs_declaration_checks =
                policy_needs_always || policy_needs_never || policy_needs_consecutive;
            let summary = if needs_declaration_checks {
                summarize_declaration(decl, self.0.separate_requires)
            } else {
                DeclSummary::default()
            };

            if needs_declaration_checks {
                let context = if policy_needs_always || policy_needs_never {
                    classify_var_decl_context(ctx, node.id(), decl)
                } else {
                    VarDeclContext::Other
                };
                let split_requires_reported = self.0.separate_requires
                    && policy.initialized == RuntimeMode::Always
                    && summary.has_mixed_require_groups();

                if split_requires_reported {
                    ctx.diagnostic(one_var_diagnostic(
                        decl.span(),
                        DiagnosticKind::SplitRequires,
                        declaration_type(decl.kind),
                    ));
                }

                if let Some(statement_list_id) = statement_list_id
                    && let Some(previous) = previous_by_list.get(&statement_list_id).copied()
                    && previous.kind == decl.kind
                    && policy_needs_consecutive
                {
                    self.check_consecutive(decl, summary, previous.summary, policy, ctx);
                }

                if policy_needs_always {
                    let scope_id = effective_scope_id(ctx, node, decl.kind, &mut var_scope_cache);
                    self.check_always(
                        decl,
                        summary,
                        policy,
                        context,
                        split_requires_reported,
                        scope_id,
                        &mut always_state,
                        ctx,
                    );
                }

                if policy_needs_never {
                    Self::check_never(decl, summary, policy, context, ctx);
                }
            }

            if let Some(statement_list_id) = statement_list_id {
                previous_by_list
                    .insert(statement_list_id, PreviousDeclaration { kind: decl.kind, summary });
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DiagnosticKind {
    Combine,
    CombineInitialized,
    CombineUninitialized,
    Split,
    SplitInitialized,
    SplitUninitialized,
    SplitRequires,
}

#[derive(Debug, Clone, Copy, Default)]
struct SeenState {
    initialized: bool,
    uninitialized: bool,
    required: bool, // meaningful only when `separateRequires == true`
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct AlwaysStateKey {
    scope_id: ScopeId,
    kind: u8,
}

impl AlwaysStateKey {
    fn new(scope_id: ScopeId, kind: VariableDeclarationKind) -> Self {
        Self { scope_id, kind: kind as u8 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VarDeclContext {
    StatementList,
    ExportNamedDeclaration,
    ForInit,
    ForInLeft,
    ForOfLeft,
    Other,
}

#[derive(Debug, Clone, Copy, Default)]
struct DeclSummary {
    initialized: usize,
    uninitialized: usize,
    total: usize,
    requires: usize,
}

impl DeclSummary {
    fn has_require(self) -> bool {
        self.requires > 0
    }

    fn has_mixed_require_groups(self) -> bool {
        self.requires > 0 && self.requires < self.total
    }

    fn non_require_initialized_count(self) -> usize {
        self.initialized.saturating_sub(self.requires)
    }

    fn has_non_require_declarator(self) -> bool {
        self.requires < self.total
    }

    fn has_non_require_group(self) -> bool {
        self.uninitialized > 0 || self.non_require_initialized_count() > 0
    }
}

#[derive(Debug, Clone, Copy)]
struct PreviousDeclaration {
    kind: VariableDeclarationKind,
    summary: DeclSummary,
}

impl OneVar {
    fn check_consecutive(
        &self,
        decl: &VariableDeclaration<'_>,
        summary: DeclSummary,
        previous: DeclSummary,
        policy: KindPolicy,
        ctx: &LintContext<'_>,
    ) {
        if policy.initialized != RuntimeMode::Consecutive
            && policy.uninitialized != RuntimeMode::Consecutive
        {
            return;
        }

        // Intentional oxlint divergence: `require(...)` is only special when
        // `separateRequires` is enabled.
        if self.0.separate_requires {
            let has_require = summary.has_require() || previous.has_require();
            let has_non_require =
                summary.has_non_require_declarator() || previous.has_non_require_declarator();
            if has_require && has_non_require {
                return;
            }
        }

        let diagnostic = if policy.initialized == RuntimeMode::Consecutive
            && policy.uninitialized == RuntimeMode::Consecutive
        {
            Some(DiagnosticKind::Combine)
        } else if policy.initialized == RuntimeMode::Consecutive
            && summary.initialized > 0
            && previous.initialized > 0
        {
            Some(DiagnosticKind::CombineInitialized)
        } else if policy.uninitialized == RuntimeMode::Consecutive
            && summary.uninitialized > 0
            && previous.uninitialized > 0
        {
            Some(DiagnosticKind::CombineUninitialized)
        } else {
            None
        };

        if let Some(diagnostic) = diagnostic {
            ctx.diagnostic(one_var_diagnostic(
                decl.span(),
                diagnostic,
                declaration_type(decl.kind),
            ));
        }
    }

    fn check_always(
        &self,
        decl: &VariableDeclaration<'_>,
        summary: DeclSummary,
        policy: KindPolicy,
        context: VarDeclContext,
        split_requires_reported: bool,
        scope_id: ScopeId,
        always_state: &mut FxHashMap<AlwaysStateKey, SeenState>,
        ctx: &LintContext<'_>,
    ) {
        let state_key = AlwaysStateKey::new(scope_id, decl.kind);
        let scope = always_state.get(&state_key).copied().unwrap_or_default();
        let mut reported = false;

        if policy.initialized == RuntimeMode::Always && policy.uninitialized == RuntimeMode::Always
        {
            let should_report = if self.0.separate_requires {
                (summary.requires > 0 && scope.required)
                    || (summary.has_non_require_group()
                        && (scope.initialized || scope.uninitialized))
            } else {
                (scope.initialized || scope.uninitialized) && summary.total > 0
            };

            if should_report {
                reported = true;
                ctx.diagnostic(one_var_diagnostic(
                    decl.span(),
                    DiagnosticKind::Combine,
                    declaration_type(decl.kind),
                ));
            }
        } else {
            if policy.initialized == RuntimeMode::Always && summary.initialized > 0 {
                let should_report = if self.0.separate_requires {
                    (summary.non_require_initialized_count() > 0 && scope.initialized)
                        || (summary.requires > 0 && scope.required)
                } else {
                    scope.initialized
                };

                if should_report {
                    reported = true;
                    ctx.diagnostic(one_var_diagnostic(
                        decl.span(),
                        DiagnosticKind::CombineInitialized,
                        declaration_type(decl.kind),
                    ));
                }
            }

            if policy.uninitialized == RuntimeMode::Always
                && summary.uninitialized > 0
                && scope.uninitialized
                && !matches!(context, VarDeclContext::ForInLeft | VarDeclContext::ForOfLeft)
            {
                reported = true;
                ctx.diagnostic(one_var_diagnostic(
                    decl.span(),
                    DiagnosticKind::CombineUninitialized,
                    declaration_type(decl.kind),
                ));
            }
        }

        // After conceptually splitting a mixed `separateRequires` declaration,
        // later `always` checks should still observe every bucket seeded by
        // that split, even if the original declaration already emitted a
        // visible `combine`.
        if !reported || split_requires_reported {
            self.record_always_buckets(summary, policy, always_state.entry(state_key).or_default());
        }
    }

    fn record_always_buckets(
        &self,
        summary: DeclSummary,
        policy: KindPolicy,
        current_scope: &mut SeenState,
    ) {
        let separate_requires = self.0.separate_requires;

        if policy.initialized == RuntimeMode::Always {
            if separate_requires {
                // Mixed declarations still seed both buckets so later `always`
                // checks can report the same follow-up combines as ESLint after
                // a conceptual split.
                if summary.has_require() {
                    current_scope.required = true;
                }
                if summary.non_require_initialized_count() > 0 {
                    current_scope.initialized = true;
                }
            } else if summary.initialized > 0 {
                current_scope.initialized = true;
            }
        }

        if policy.uninitialized == RuntimeMode::Always && summary.uninitialized > 0 {
            current_scope.uninitialized = true;
        }
    }

    fn check_never(
        decl: &VariableDeclaration<'_>,
        summary: DeclSummary,
        policy: KindPolicy,
        context: VarDeclContext,
        ctx: &LintContext<'_>,
    ) {
        if summary.total <= 1 || context == VarDeclContext::ForInit {
            return;
        }

        let diagnostic = if policy.initialized == RuntimeMode::Never
            && policy.uninitialized == RuntimeMode::Never
        {
            Some(DiagnosticKind::Split)
        } else if policy.initialized == RuntimeMode::Never && summary.initialized > 0 {
            Some(DiagnosticKind::SplitInitialized)
        } else if policy.uninitialized == RuntimeMode::Never && summary.uninitialized > 0 {
            Some(DiagnosticKind::SplitUninitialized)
        } else {
            None
        };

        if let Some(diagnostic) = diagnostic {
            ctx.diagnostic(one_var_diagnostic(
                decl.span(),
                diagnostic,
                declaration_type(decl.kind),
            ));
        }
    }
}

fn declaration_type(kind: VariableDeclarationKind) -> &'static str {
    match kind {
        VariableDeclarationKind::Var => "var",
        VariableDeclarationKind::Let => "let",
        VariableDeclarationKind::Const => "const",
        VariableDeclarationKind::Using => "using",
        VariableDeclarationKind::AwaitUsing => "await using",
    }
}

fn effective_scope_id(
    ctx: &LintContext<'_>,
    node: &AstNode<'_>,
    kind: VariableDeclarationKind,
    var_scope_cache: &mut FxHashMap<ScopeId, ScopeId>,
) -> ScopeId {
    let node_scope_id = node.scope_id();

    if kind != VariableDeclarationKind::Var {
        return node_scope_id;
    }

    if let Some(scope_id) = var_scope_cache.get(&node_scope_id).copied() {
        return scope_id;
    }

    let scoping = ctx.semantic().scoping();
    let mut current_scope_id = node_scope_id;
    let mut visited = SmallVec::<[ScopeId; 8]>::new();

    let var_scope_id = loop {
        if let Some(scope_id) = var_scope_cache.get(&current_scope_id).copied() {
            break scope_id;
        }

        if scoping.scope_flags(current_scope_id).is_var() {
            break current_scope_id;
        }

        visited.push(current_scope_id);
        let Some(parent_scope_id) = scoping.scope_parent_id(current_scope_id) else {
            break node_scope_id;
        };
        current_scope_id = parent_scope_id;
    };

    for scope_id in visited {
        var_scope_cache.insert(scope_id, var_scope_id);
    }
    var_scope_cache.insert(node_scope_id, var_scope_id);
    var_scope_id
}

// Only direct statement-list children participate in `consecutive`.
// Non-variable direct children clear the previous declaration for that list.
fn statement_list_parent_id(ctx: &LintContext<'_>, node_id: NodeId) -> Option<NodeId> {
    let nodes = ctx.semantic().nodes();
    if matches!(nodes.kind(node_id), AstKind::Program(_)) {
        return None;
    }

    let parent_id = nodes.parent_id(node_id);
    match nodes.kind(parent_id) {
        AstKind::Program(_)
        | AstKind::FunctionBody(_)
        | AstKind::BlockStatement(_)
        | AstKind::StaticBlock(_)
        | AstKind::TSModuleBlock(_)
        | AstKind::SwitchCase(_) => Some(parent_id),
        _ => None,
    }
}

fn classify_var_decl_context<'a>(
    ctx: &LintContext<'a>,
    node_id: NodeId,
    decl: &VariableDeclaration<'a>,
) -> VarDeclContext {
    match ctx.semantic().nodes().parent_kind(node_id) {
        AstKind::Program(_)
        | AstKind::FunctionBody(_)
        | AstKind::BlockStatement(_)
        | AstKind::StaticBlock(_)
        | AstKind::TSModuleBlock(_)
        | AstKind::SwitchCase(_) => VarDeclContext::StatementList,
        AstKind::ExportNamedDeclaration(_) => VarDeclContext::ExportNamedDeclaration,
        AstKind::ForStatement(statement) => {
            if let Some(ForStatementInit::VariableDeclaration(init)) = &statement.init
                && ptr::eq(decl, init.as_ref())
            {
                VarDeclContext::ForInit
            } else {
                VarDeclContext::Other
            }
        }
        AstKind::ForInStatement(statement) => {
            if let ForStatementLeft::VariableDeclaration(left) = &statement.left
                && ptr::eq(decl, left.as_ref())
            {
                VarDeclContext::ForInLeft
            } else {
                VarDeclContext::Other
            }
        }
        AstKind::ForOfStatement(statement) => {
            if let ForStatementLeft::VariableDeclaration(left) = &statement.left
                && ptr::eq(decl, left.as_ref())
            {
                VarDeclContext::ForOfLeft
            } else {
                VarDeclContext::Other
            }
        }
        _ => VarDeclContext::Other,
    }
}

fn summarize_declaration(decl: &VariableDeclaration<'_>, track_requires: bool) -> DeclSummary {
    let mut initialized_count = 0;
    let mut uninitialized_count = 0;
    let mut require_count = 0;

    for declarator in &decl.declarations {
        if declarator.init.is_some() {
            initialized_count += 1;
            if track_requires && is_require_declarator(declarator) {
                require_count += 1;
            }
        } else {
            uninitialized_count += 1;
        }
    }

    DeclSummary {
        initialized: initialized_count,
        uninitialized: uninitialized_count,
        total: decl.declarations.len(),
        requires: require_count,
    }
}

fn is_require_declarator(declarator: &VariableDeclarator<'_>) -> bool {
    let Some(Expression::CallExpression(call)) = &declarator.init else {
        return false;
    };
    matches!(
        &call.callee,
        Expression::Identifier(identifier) if identifier.name == "require"
    )
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("function foo() { var bar = true; }", None),
        ("function foo() { var bar = true, baz = 1; if (qux) { bar = false; } }", None),
        ("var foo = function() { var bar = true; baz(); }", None),
        ("function foo() { var bar = true, baz = false; }", Some(serde_json::json!(["always"]))),
        ("function foo() { var bar = true; var baz = false; }", Some(serde_json::json!(["never"]))),
        ("for (var i = 0, len = arr.length; i < len; i++) {}", Some(serde_json::json!(["never"]))),
        ("var bar = true; var baz = false;", Some(serde_json::json!([{ "initialized": "never" }]))),
        ("var bar = true, baz = false;", Some(serde_json::json!([{ "initialized": "always" }]))),
        ("var bar, baz;", Some(serde_json::json!([{ "initialized": "never" }]))),
        ("var bar; var baz;", Some(serde_json::json!([{ "uninitialized": "never" }]))),
        ("var bar, baz;", Some(serde_json::json!([{ "uninitialized": "always" }]))),
        ("var bar = true, baz = false;", Some(serde_json::json!([{ "uninitialized": "never" }]))),
        (
            "var bar = true, baz = false, a, b;",
            Some(serde_json::json!([{ "uninitialized": "always", "initialized": "always" }])),
        ),
        (
            "var bar = true; var baz = false; var a; var b;",
            Some(serde_json::json!([{ "uninitialized": "never", "initialized": "never" }])),
        ),
        (
            "function foo() { var a = [1, 2, 3]; var [b, c, d] = a; }",
            Some(serde_json::json!(["never"])),
        ),
        (
            "function foo() { let a = 1; var c = true; if (a) {let c = true; } }",
            Some(serde_json::json!(["always"])),
        ),
        ("function f() { let a; { let {} = obj; } }", Some(serde_json::json!(["always"]))),
        (
            "function foo() { const a = 1; var c = true; if (a) {const c = true; } }",
            Some(serde_json::json!(["always"])),
        ),
        ("function foo() { let a = 1; let b = true; }", Some(serde_json::json!(["never"]))),
        ("function foo() { const a = 1; const b = true; }", Some(serde_json::json!(["never"]))),
        (
            "let foo = true; for (let i = 0; i < 1; i++) { let foo = false; }",
            Some(serde_json::json!([{ "var": "always", "let": "always", "const": "never" }])),
        ),
        ("let foo = true, bar = false;", Some(serde_json::json!([{ "var": "never" }]))),
        (
            "for (let x of foo) {}; for (let y of foo) {}",
            Some(serde_json::json!([{ "uninitialized": "always" }])),
        ),
        (
            "for (let x in foo) {}; for (let y in foo) {}",
            Some(serde_json::json!([{ "uninitialized": "always" }])),
        ),
        (
            "var x; for (var y in foo) {}",
            Some(serde_json::json!([{ "initialized": "never", "uninitialized": "always" }])),
        ),
        (
            "var x; for (var y of foo) {}",
            Some(serde_json::json!([{ "initialized": "never", "uninitialized": "always" }])),
        ),
        (
            "var bar, baz; var a = true; var b = false;",
            Some(serde_json::json!([{ "uninitialized": "always", "initialized": "never" }])),
        ),
        (
            "var foo = require('foo'), bar;",
            Some(serde_json::json!([{ "separateRequires": false, "var": "always" }])),
        ),
        (
            "var foo = require('foo'), bar = require('bar');",
            Some(serde_json::json!([{ "separateRequires": true, "var": "always" }])),
        ),
        (
            "var bar = 'bar'; var foo = require('foo');",
            Some(serde_json::json!([{ "separateRequires": true, "var": "always" }])),
        ),
        (
            "var foo = require('foo'); var bar = 'bar';",
            Some(serde_json::json!([{ "separateRequires": true, "var": "always" }])),
        ),
        (
            "var foo = require('foo'); var bar;",
            Some(serde_json::json!([{ "separateRequires": true, "var": "consecutive" }])),
        ),
        (
            "var bar; var foo = require('foo');",
            Some(serde_json::json!([{ "separateRequires": true, "var": "consecutive" }])),
        ),
        ("var a = 0, b, c;", Some(serde_json::json!(["consecutive"]))),
        ("var a = 0, b = 1, c = 2;", Some(serde_json::json!(["consecutive"]))),
        ("var a = 0, b = 1; foo(); var c = 2;", Some(serde_json::json!(["consecutive"]))),
        ("let a = 0, b, c;", Some(serde_json::json!(["consecutive"]))),
        ("let a = 0, b = 1, c = 2;", Some(serde_json::json!(["consecutive"]))),
        ("let a = 0; foo; let b = 1;", Some(serde_json::json!([{ "let": "consecutive" }]))),
        ("const a = 0, b = 1;", Some(serde_json::json!([{ "const": "consecutive" }]))),
        ("const a = 0; foo; const b = 1;", Some(serde_json::json!([{ "const": "consecutive" }]))),
        ("var a; somethingElse(); var b;", Some(serde_json::json!([{ "var": "never" }]))),
        (
            "var foo = 1; let bar = function() { var x; }; var baz = 2;",
            Some(serde_json::json!([{ "var": "never" }])),
        ),
        ("class C { static { var a; let b; const c = 0; } }", Some(serde_json::json!(["always"]))),
        (
            "class C { static { const a = 0; if (foo) { const b = 0; } } }",
            Some(serde_json::json!(["always"])),
        ),
        ("class C { static { let a; if (foo) { let b; } } }", Some(serde_json::json!(["always"]))),
        ("class C { static { const a = 0; const b = 0; } }", Some(serde_json::json!(["never"]))),
        ("class C { static { let a; let b; } }", Some(serde_json::json!(["never"]))),
        ("class C { static { var a; var b; } }", Some(serde_json::json!(["never"]))),
        ("class C { static { let a; foo; let b; } }", Some(serde_json::json!(["consecutive"]))),
        ("class C { static { var a; foo; var b; } }", Some(serde_json::json!(["consecutive"]))),
        (
            "class C { static { let a; if (foo) { let b; } } }",
            Some(serde_json::json!(["consecutive"])),
        ),
        (
            "class C { static { if (foo) { let b; } let a; } }",
            Some(serde_json::json!(["consecutive"])),
        ),
        ("class C { static { if (foo) var b; var a; } }", Some(serde_json::json!(["consecutive"]))),
        ("var a; { var b; } var c;", Some(serde_json::json!(["consecutive"]))),
        ("using a = 0; let b = 1; const c = 2;", None),
        ("await using a = 0; let b = 1; const c = 2;", None),
        ("using a = 0, b = 1;", None),
        ("await using a = 0, b = 1;", None),
        ("using a = 0; using b = 1;", Some(serde_json::json!(["never"]))),
        ("await using a = 0; await using b = 1;", Some(serde_json::json!(["never"]))),
        ("using a = 0, b = 1;", Some(serde_json::json!(["consecutive"]))),
        ("await using a = 0, b = 1;", Some(serde_json::json!(["consecutive"]))),
        ("using a = 0, b = 1;", Some(serde_json::json!([{ "initialized": "always" }]))),
        ("await using a = 0, b = 1;", Some(serde_json::json!([{ "initialized": "always" }]))),
        ("using a = 0; using b = 1;", Some(serde_json::json!([{ "initialized": "never" }]))),
        (
            "await using a = 0; await using b = 1;",
            Some(serde_json::json!([{ "initialized": "never" }])),
        ),
        ("foo(() => { var a; var b; });", Some(serde_json::json!(["never"]))),
        ("const obj = { method() { var a; var b; } };", Some(serde_json::json!(["never"]))),
        ("class C { method() { var a; var b; } }", Some(serde_json::json!(["never"]))),
        ("const C = class { static { var a; var b; } };", Some(serde_json::json!(["never"]))),
        ("class C { field = function () { var a; var b; }; }", Some(serde_json::json!(["never"]))),
        ("export const a = 1; export const b = 2;", Some(serde_json::json!(["consecutive"]))),
        (
            "const foo = require('foo'); const bar = 'bar';",
            Some(serde_json::json!([{ "const": "consecutive", "separateRequires": true }])),
        ),
        (
            "var a = 0, b = 1; var c, d;",
            Some(serde_json::json!([{ "initialized": "consecutive", "uninitialized": "always" }])),
        ),
        (
            "var a = 0; var b, c; var d = 1;",
            Some(serde_json::json!([{ "initialized": "consecutive", "uninitialized": "always" }])),
        ),
        (
            "var a = 0, b = 1; var c; var d;",
            Some(serde_json::json!([{ "initialized": "consecutive", "uninitialized": "never" }])),
        ),
        (
            "var a = 0; var b; var c; var d = 1;",
            Some(serde_json::json!([{ "initialized": "consecutive", "uninitialized": "never" }])),
        ),
        (
            "var a, b; var c = 0, d = 1;",
            Some(serde_json::json!([{ "uninitialized": "consecutive", "initialized": "always" }])),
        ),
        (
            "var a; var b = 0, c = 1; var d;",
            Some(serde_json::json!([{ "uninitialized": "consecutive", "initialized": "always" }])),
        ),
        (
            "var a, b; var c = 0; var d = 1;",
            Some(serde_json::json!([{ "uninitialized": "consecutive", "initialized": "never" }])),
        ),
        (
            "var a; var b = 0; var c = 1; var d;",
            Some(serde_json::json!([{ "uninitialized": "consecutive", "initialized": "never" }])),
        ),
        (
            "let a, b; const c = 0, d = 1;",
            Some(serde_json::json!([{ "uninitialized": "consecutive", "initialized": "always" }])),
        ),
        (
            "let a; const b = 0, c = 1; let d;",
            Some(serde_json::json!([{ "uninitialized": "consecutive", "initialized": "always" }])),
        ),
        (
            "let a, b; const c = 0; const d = 1;",
            Some(serde_json::json!([{ "uninitialized": "consecutive", "initialized": "never" }])),
        ),
        (
            "let a; const b = 0; const c = 1; let d;",
            Some(serde_json::json!([{ "uninitialized": "consecutive", "initialized": "never" }])),
        ),
        (
            "let a, b; const c = 0, d = 1;",
            Some(serde_json::json!([{ "let": "consecutive", "const": "always" }])),
        ),
        (
            "let a; const b = 0, c = 1; let d;",
            Some(serde_json::json!([{ "let": "consecutive", "const": "always" }])),
        ),
        (
            "let a, b; const c = 0; const d = 1;",
            Some(serde_json::json!([{ "let": "consecutive", "const": "never" }])),
        ),
        (
            "const a = 0; let b, c; const d = 1;",
            Some(serde_json::json!([{ "const": "consecutive", "let": "always" }])),
        ),
        (
            "const a = 0; let b; let c; const d = 1;",
            Some(serde_json::json!([{ "const": "consecutive", "let": "never" }])),
        ),
        (
            "var a = 1, b = 2; foo(); var c = 3, d = 4;",
            Some(serde_json::json!([{ "initialized": "consecutive" }])),
        ),
        (
            "var bar = 1, baz = 2; qux(); var qux = 3, quux;",
            Some(serde_json::json!(["consecutive"])),
        ),
        (
            "let a, b; var c; var d; let e;",
            Some(
                serde_json::json!([{ "var": "never", "let": "consecutive", "const": "consecutive" }]),
            ),
        ),
        (
            "const a = 1, b = 2; var d; var e; const f = 3;",
            Some(
                serde_json::json!([{ "var": "never", "let": "consecutive", "const": "consecutive" }]),
            ),
        ),
        (
            "var a, b; const c = 1; const d = 2; let e; let f;",
            Some(serde_json::json!([{ "var": "consecutive" }])),
        ),
        ("const a = 0; class C { static { const b = 0; } }", Some(serde_json::json!(["always"]))),
        ("class C { static { const b = 0; } } const a = 0;", Some(serde_json::json!(["always"]))),
        ("var a; class C { static { var b; } }", Some(serde_json::json!(["always"]))),
        ("class C { static { var b; } } var a;", Some(serde_json::json!(["always"]))),
        (
            "class C { static { let x; const b = 0; let c; } }",
            Some(serde_json::json!(["consecutive"])),
        ),
        ("class C { static { var a; let b; var c; } }", Some(serde_json::json!(["consecutive"]))),
        ("function fn() { { using a = 0; } using b = 1; }", None),
        (
            "using a = 0, b = 1; foo(); using c = 2, d = 3;",
            Some(serde_json::json!([{ "initialized": "consecutive" }])),
        ),
        (
            "await using a = 0, b = 1; foo(); await using c = 2, d = 3;",
            Some(serde_json::json!([{ "initialized": "consecutive" }])),
        ),
    ];

    let fail = vec![
        ("var bar = true, baz = false;", Some(serde_json::json!(["never"]))),
        ("function foo() { var bar = true, baz = false; }", Some(serde_json::json!(["never"]))),
        ("if (foo) { var bar = true, baz = false; }", Some(serde_json::json!(["never"]))),
        (
            "switch (foo) { case bar: var baz = true, quux = false; }",
            Some(serde_json::json!(["never"])),
        ),
        (
            "switch (foo) { default: var baz = true, quux = false; }",
            Some(serde_json::json!(["never"])),
        ),
        (
            "function foo() { var bar = true; var baz = false; }",
            Some(serde_json::json!(["always"])),
        ),
        ("var a = 1; for (var b = 2;;) {}", Some(serde_json::json!(["always"]))),
        (
            "function foo() { var foo = true, bar = false; }",
            Some(serde_json::json!([{ "initialized": "never" }])),
        ),
        (
            "function foo() { var foo, bar; }",
            Some(serde_json::json!([{ "uninitialized": "never" }])),
        ),
        (
            "function foo() { var bar, baz; var a = true; var b = false; var c, d;}",
            Some(serde_json::json!([{ "uninitialized": "always", "initialized": "never" }])),
        ),
        (
            "function foo() { var bar = true, baz = false; var a; var b; var c = true, d = false; }",
            Some(serde_json::json!([{ "uninitialized": "never", "initialized": "always" }])),
        ),
        (
            "function foo() { var bar = true, baz = false; var a, b;}",
            Some(serde_json::json!([{ "uninitialized": "never", "initialized": "never" }])),
        ),
        (
            "function foo() { var bar = true; var baz = false; var a; var b;}",
            Some(serde_json::json!([{ "uninitialized": "always", "initialized": "always" }])),
        ),
        (
            "function foo() { var a = [1, 2, 3]; var [b, c, d] = a; }",
            Some(serde_json::json!(["always"])),
        ),
        ("function foo() { let a = 1; let b = 2; }", Some(serde_json::json!(["always"]))),
        ("function foo() { const a = 1; const b = 2; }", Some(serde_json::json!(["always"]))),
        ("function foo() { let a = 1, b = 2; }", Some(serde_json::json!([{ "let": "never" }]))),
        ("function foo() { let a, b; }", Some(serde_json::json!([{ "uninitialized": "never" }]))),
        ("function foo() { const a = 1, b = 2; }", Some(serde_json::json!([{ "const": "never" }]))),
        (
            "let foo = true; switch(foo) { case true: let bar = 2; break; case false: let baz = 3; break; }",
            Some(serde_json::json!([{ "var": "always", "let": "always", "const": "never" }])),
        ),
        ("var one = 1, two = 2; var three;", Some(serde_json::json!(["always"]))),
        ("var i = [0], j;", Some(serde_json::json!([{ "initialized": "never" }]))),
        ("var i = [0], j;", Some(serde_json::json!([{ "uninitialized": "never" }]))),
        ("var foo = function() { var bar = true; var baz = false; }", None),
        (
            "function foo() { var bar = true; if (qux) { var baz = false; } else { var quxx = 42; } }",
            None,
        ),
        ("var {} = obj; var a;", Some(serde_json::json!(["always"]))),
        ("function f() { var a; { var {} = obj; } }", Some(serde_json::json!(["always"]))),
        ("var {} = obj; var a;", Some(serde_json::json!(["consecutive"]))),
        (
            "function f() { {{{{{ var a; }}}}} {{{{{ var b; }}}}} }",
            Some(serde_json::json!(["always"])),
        ),
        ("var foo = () => { var bar = true; var baz = false; }", None),
        ("var foo = function() { var bar = true; if (qux) { var baz = false; } }", None),
        ("var foo; var bar;", None),
        ("for (var x of foo) {}; for (var y of foo) {}", Some(serde_json::json!(["always"]))),
        ("for (var x in foo) {}; for (var y in foo) {}", Some(serde_json::json!(["always"]))),
        (
            "var x = 1, y = 2; for (var z in foo) {}",
            Some(serde_json::json!([{ "initialized": "never", "uninitialized": "always" }])),
        ),
        (
            "var x = 1, y = 2; for (var z of foo) {}",
            Some(serde_json::json!([{ "initialized": "never", "uninitialized": "always" }])),
        ),
        (
            "var x; var y; for (var z in foo) {}",
            Some(serde_json::json!([{ "initialized": "never", "uninitialized": "always" }])),
        ),
        (
            "var x; var y; for (var z of foo) {}",
            Some(serde_json::json!([{ "initialized": "never", "uninitialized": "always" }])),
        ),
        ("var {foo} = 1, [bar] = 2;", Some(serde_json::json!([{ "initialized": "never" }]))),
        (
            "const foo = require('foo'); const bar = require('bar');",
            Some(serde_json::json!([{ "separateRequires": true, "const": "always" }])),
        ),
        (
            "var foo = require('foo'), bar;",
            Some(serde_json::json!([{ "separateRequires": true, "var": "always" }])),
        ),
        (
            "var foo, bar = require('bar');",
            Some(serde_json::json!([{ "separateRequires": true, "var": "always" }])),
        ),
        (
            "let foo, bar = require('bar');",
            Some(serde_json::json!([{ "separateRequires": true, "let": "always" }])),
        ),
        (
            "const foo = 0, bar = require('bar');",
            Some(serde_json::json!([{ "separateRequires": true, "const": "always" }])),
        ),
        (
            "var a = require('a'), b = 1; var c = require('c');",
            Some(serde_json::json!([{ "separateRequires": true, "var": "always" }])),
        ),
        (
            // Oxlint intentionally reports both the ordinary `combine` on the
            // mixed declaration and the follow-up `combine` that becomes
            // visible after a conceptual split.
            "var x = 1; var c = require('c'), d = 2; var e = require('e');",
            Some(serde_json::json!([{ "separateRequires": true, "var": "always" }])),
        ),
        (
            "var r = require('r'); var r2 = require('r2'), a; var b;",
            Some(serde_json::json!([{ "separateRequires": true, "var": "always" }])),
        ),
        (
            "var x = 1; var c = require('c'), d = 2;",
            Some(serde_json::json!([{ "separateRequires": true, "var": "always" }])),
        ),
        ("var a = 1, b; var c;", Some(serde_json::json!(["consecutive"]))),
        ("var a = 0, b = 1; var c = 2;", Some(serde_json::json!(["consecutive"]))),
        ("let a = 1, b; let c;", Some(serde_json::json!(["consecutive"]))),
        ("let a = 0, b = 1; let c = 2;", Some(serde_json::json!(["consecutive"]))),
        ("const a = 0, b = 1; const c = 2;", Some(serde_json::json!(["consecutive"]))),
        ("class C { static { let x, y; } }", Some(serde_json::json!(["never"]))),
        ("class C { static { var x, y; } }", Some(serde_json::json!(["never"]))),
        ("class C { static { let x; let y; } }", Some(serde_json::json!(["always"]))),
        ("class C { static { var x; var y; } }", Some(serde_json::json!(["always"]))),
        ("class C { static { let x; foo; let y; } }", Some(serde_json::json!(["always"]))),
        ("class C { static { var x; foo; var y; } }", Some(serde_json::json!(["always"]))),
        ("class C { static { var x; if (foo) { var y; } } }", Some(serde_json::json!(["always"]))),
        ("class C { static { let x; let y; } }", Some(serde_json::json!(["consecutive"]))),
        ("class C { static { var x; var y; } }", Some(serde_json::json!(["consecutive"]))),
        ("using a = 0; using b = 1;", Some(serde_json::json!(["always"]))),
        ("await using a = 0; await using b = 1;", Some(serde_json::json!(["always"]))),
        ("using a = 0, b = 1;", Some(serde_json::json!(["never"]))),
        ("await using a = 0, b = 1;", Some(serde_json::json!(["never"]))),
        ("using a = 0; using b = 1;", Some(serde_json::json!(["consecutive"]))),
        ("await using a = 0; await using b = 1;", Some(serde_json::json!(["consecutive"]))),
        ("function f() { var a; var b; }", Some(serde_json::json!(["consecutive"]))),
        ("const f = () => { var a; var b; };", Some(serde_json::json!(["consecutive"]))),
        ("var x = 1; var fs = require('fs');", Some(serde_json::json!(["always"]))),
        ("var x = 1; var fs = require('fs');", Some(serde_json::json!([{ "var": "consecutive" }]))),
        ("switch (x) { case 1: var a; var b; }", Some(serde_json::json!(["consecutive"]))),
        ("export const a = 1; export const b = 2;", Some(serde_json::json!(["always"]))),
        ("export const a = 1, b = 2;", Some(serde_json::json!(["never"]))),
        ("foo(() => { var a; var b; });", Some(serde_json::json!(["always"]))),
        ("const obj = { method() { var a; var b; } };", Some(serde_json::json!(["always"]))),
        ("class C { method() { var a; var b; } }", Some(serde_json::json!(["always"]))),
        ("const C = class { static { var a; var b; } };", Some(serde_json::json!(["always"]))),
        ("class C { field = function () { var a; var b; }; }", Some(serde_json::json!(["always"]))),
        ("if (foo) var x, y;", Some(serde_json::json!(["never"]))),
        ("if (foo) var x = 1, y = 1;", Some(serde_json::json!([{ "initialized": "never" }]))),
        ("if (foo) {} else var x, y;", Some(serde_json::json!(["never"]))),
        ("while (foo) var x, y;", Some(serde_json::json!(["never"]))),
        ("do var x, y; while (foo);", Some(serde_json::json!(["never"]))),
        ("do var x = f(), y = b(); while (x < y);", Some(serde_json::json!(["never"]))),
        ("for (;;) var x, y;", Some(serde_json::json!(["never"]))),
        ("for (foo in bar) var x, y;", Some(serde_json::json!(["never"]))),
        ("for (foo of bar) var x, y;", Some(serde_json::json!(["never"]))),
        ("with (foo) var x, y;", Some(serde_json::json!(["never"]))),
        ("label: var x, y;", Some(serde_json::json!(["never"]))),
        (
            "var a = 0; var b; var c; var d = 1",
            Some(serde_json::json!([{ "initialized": "consecutive", "uninitialized": "always" }])),
        ),
        (
            "var a = 0; var b = 1; var c; var d;",
            Some(serde_json::json!([{ "initialized": "consecutive", "uninitialized": "always" }])),
        ),
        (
            "const a = 0; var b = 1; var c = 2; const d = 3;",
            Some(serde_json::json!(["consecutive"])),
        ),
        (
            "let a = 0; const b = 1; const c = 1; var d = 2;",
            Some(serde_json::json!(["consecutive"])),
        ),
        (
            "class C { static { let a = 0; let b = 1; } }",
            Some(serde_json::json!([{ "initialized": "consecutive" }])),
        ),
        (
            "class C { static { var a = 0; var b = 1; } }",
            Some(serde_json::json!([{ "initialized": "consecutive" }])),
        ),
        ("using a = 0, b = 1;", Some(serde_json::json!([{ "initialized": "never" }]))),
        ("await using a = 0, b = 1;", Some(serde_json::json!([{ "initialized": "never" }]))),
        ("var a\nvar b", Some(serde_json::json!(["always"]))),
    ];

    Tester::new(OneVar::NAME, OneVar::PLUGIN, pass, fail).test_and_snapshot();
}

#[test]
fn test_typescript() {
    use crate::tester::Tester;

    let pass = vec![
        ("declare var a;", Some(serde_json::json!(["always"]))),
        ("declare var a, b;", Some(serde_json::json!(["always"]))),
        ("declare let a;", Some(serde_json::json!(["always"]))),
        ("let outer; declare module 'pkg' { let inner; }", Some(serde_json::json!(["always"]))),
        ("var outer; declare module 'pkg' { var inner; }", Some(serde_json::json!(["always"]))),
        ("var outer; namespace N { var inner; }", Some(serde_json::json!(["always"]))),
        ("var outer; declare global { var inner; }", Some(serde_json::json!(["always"]))),
    ];

    let fail = vec![
        ("declare var a, b;", Some(serde_json::json!(["never"]))),
        ("declare var a; declare var b;", Some(serde_json::json!(["always"]))),
        ("declare let a, b;", Some(serde_json::json!(["never"]))),
        ("declare module 'pkg' { var a; var b; }", Some(serde_json::json!(["always"]))),
        ("declare global { var a; var b; }", Some(serde_json::json!(["always"]))),
        ("declare global { let a; let b; }", Some(serde_json::json!(["consecutive"]))),
    ];

    Tester::new(OneVar::NAME, OneVar::PLUGIN, pass, fail)
        .change_rule_path_extension("ts")
        .with_snapshot_suffix("ts")
        .test_and_snapshot();
}

#[test]
fn invalid_configs_error_in_from_configuration() {
    for config in [
        serde_json::json!([null]),
        serde_json::json!([{}]),
        serde_json::json!(["always", "never"]),
        serde_json::json!([{ "var": "always", "initialized": "never" }]),
        serde_json::json!([{ "unknown": true }]),
        serde_json::json!([{ "separateRequires": true, "initialized": "always" }]),
        serde_json::json!([{ "separateRequires": "yes" }]),
        serde_json::json!([{ "var": "sometimes" }]),
    ] {
        assert!(OneVar::from_configuration(config).is_err());
    }

    for config in [
        serde_json::json!([]),
        serde_json::json!(["always"]),
        serde_json::json!([{ "separateRequires": true }]),
    ] {
        assert!(OneVar::from_configuration(config).is_ok());
    }
}
