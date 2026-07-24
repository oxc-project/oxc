use std::borrow::Cow;

use oxc_diagnostics::Severity;
use oxc_macros::declare_oxc_lint;
use oxc_react_compiler::{CompilerOutputMode, EnvironmentConfig, PluginOptions};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

/// The compiler options `eslint-plugin-react-compiler` lints with — `lint`
/// output mode plus the validations that are off by default in the compiler.
/// Mirrors `COMPILER_OPTIONS` in the plugin's `src/shared/RunReactCompiler.ts`.
///
/// Each validation is gated on the matching [`ReactCompilerConfig`] toggle, so
/// turning a sub-rule off in the config stops the compiler from emitting its
/// diagnostics at all (rather than reporting and then filtering them).
fn react_compiler_options(config: &ReactCompilerConfig) -> PluginOptions {
    PluginOptions {
        output_mode: Some(CompilerOutputMode::Lint),
        // Don't emit errors on Flow suppressions — Flow already gave a signal.
        // Suppressed lines are filtered in `run_once` instead (like the eslint
        // plugin), so other diagnostics in the same function still surface.
        flow_suppressions: false,
        // The `Suppression` category reports pre-existing `eslint-disable react-*`
        // comments. The compiler only scans for them when `hooks` or
        // `memoDependencies` is off (see `program.rs`: `validate_exhaustive &&
        // validate_hooks`), so without gating, an unrelated toggle would surface
        // a user's existing ESLint suppressions as new errors. Treat it as a
        // bail-out: an empty rule list silences the scan unless the user opts in
        // via `reportAllBailouts` (`None` lets the compiler use its defaults).
        eslint_suppression_rules: if config.report_all_bailouts { None } else { Some(vec![]) },
        environment: EnvironmentConfig {
            validate_ref_access_during_render: config.refs,
            validate_no_set_state_in_render: config.set_state_in_render,
            validate_no_set_state_in_effects: config.set_state_in_effect,
            validate_no_jsx_in_try_statements: config.error_boundaries,
            validate_no_impure_functions_in_render: config.purity,
            validate_static_components: config.static_components,
            validate_no_void_use_memo: config.void_use_memo,
            // `Some(vec![])` enables the check with the default allowlist; `None`
            // disables it entirely.
            validate_no_capitalized_calls: config.capitalized_calls.then(Vec::new),
            validate_hooks_usage: config.hooks,
            validate_no_derived_computations_in_effects: config.derived_computations_in_effect,
            validate_exhaustive_memoization_dependencies: config.memo_dependencies,
            // `PreserveManualMemo` fires when *either* flag is on, so both must be
            // cleared to silence it.
            enable_preserve_existing_memoization_guarantees: config.preserve_manual_memo,
            validate_preserve_existing_memoization_guarantees: config.preserve_manual_memo,
            ..EnvironmentConfig::default()
        },
        ..PluginOptions::default()
    }
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct ReactCompiler(Box<ReactCompilerConfig>);

impl std::ops::Deref for ReactCompiler {
    type Target = ReactCompilerConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct ReactCompilerConfig {
    /// Also report compiler bail-outs — places where React Compiler skipped a
    /// component or hook (for example because of unsupported syntax) without
    /// finding a rule violation. These do not indicate incorrect code, only
    /// code that the compiler declined to optimize.
    report_all_bailouts: bool,

    // ---- Sub-rule toggles ----
    //
    // Each flag enables one React Compiler validation. They default to `true`
    // (see the `Default` impl below); set one to `false` to disable that
    // category of diagnostic globally, e.g.:
    //
    // ```json
    // "react/react-compiler": ["error", { "refs": false }]
    // ```
    /// Hooks must be called unconditionally and in a consistent order
    /// (`validate_hooks_usage`).
    hooks: bool,
    /// Refs may not be read or written during render
    /// (`validate_ref_access_during_render`).
    refs: bool,
    /// `setState` may not be called during render
    /// (`validate_no_set_state_in_render`).
    set_state_in_render: bool,
    /// `setState` may not be called unconditionally inside an effect
    /// (`validate_no_set_state_in_effects`).
    set_state_in_effect: bool,
    /// JSX may not be constructed inside `try`/`catch`
    /// (`validate_no_jsx_in_try_statements`).
    error_boundaries: bool,
    /// Known-impure functions (e.g. `Math.random`, `Date.now`) may not be
    /// called during render (`validate_no_impure_functions_in_render`).
    purity: bool,
    /// Components must be statically defined (`validate_static_components`).
    static_components: bool,
    /// `useMemo` callbacks must return a value (`validate_no_void_use_memo`).
    void_use_memo: bool,
    /// Capitalized functions are reserved for components and must be invoked as
    /// JSX (`validate_no_capitalized_calls`).
    capitalized_calls: bool,
    /// Effects may not recompute values that could be derived during render
    /// (emitted as the `EffectDerivationsOfState` category;
    /// `validate_no_derived_computations_in_effects`).
    derived_computations_in_effect: bool,
    /// Memoization dependency arrays (`useMemo`/`useCallback`/`useEffect`) must
    /// be exhaustive — the `MemoDependencies` category, equivalent to ESLint's
    /// `react-hooks/exhaustive-deps` (`validate_exhaustive_memoization_dependencies`).
    memo_dependencies: bool,
    /// Manual `useMemo`/`useCallback` memoization must be preservable by the
    /// compiler — the `PreserveManualMemo` category. `eslint-plugin-react-compiler`
    /// ships this off by default; set to `false` to match
    /// (`enable_preserve_existing_memoization_guarantees` +
    /// `validate_preserve_existing_memoization_guarantees`).
    ///
    /// NOTE: the memoization validations are coupled. Disabling this alone
    /// *reclassifies* rather than removes — the compiler still bails on those
    /// sites and re-reports them under `MemoDependencies` (and a few under
    /// `Purity`). To silence the family, also set `memoDependencies: false`
    /// (and `purity: false` if those remain).
    preserve_manual_memo: bool,

    // ---- Rule-layer toggles ----
    //
    // These categories have no usable compiler flag, so the toggle is enforced
    // by dropping the diagnostic in the rule rather than in the compiler. They
    // still default to `true`.
    /// Values that are known to be mutable (props, state) may not be mutated —
    /// the `Immutability` category. Emitted by the core mutation/aliasing
    /// inference pass and by `validate_locals_not_reassigned_after_render`,
    /// neither of which the compiler exposes a flag for.
    immutability: bool,
    /// `useMemo`/`useCallback` callbacks must be well-formed — the `UseMemo`
    /// category (callbacks taking parameters, being async/generator, or
    /// reassigning outer variables). Distinct from `voidUseMemo`, which covers
    /// callbacks that return nothing. Emitted by `drop_manual_memoization` and
    /// the non-void part of `validate_use_memo`, neither gated by a flag.
    use_memo: bool,
}

impl Default for ReactCompilerConfig {
    fn default() -> Self {
        // Every validation is on by default — this mirrors the set of checks the
        // rule has always run. Only `report_all_bailouts` is opt-in.
        Self {
            report_all_bailouts: false,
            hooks: true,
            refs: true,
            set_state_in_render: true,
            set_state_in_effect: true,
            error_boundaries: true,
            purity: true,
            static_components: true,
            void_use_memo: true,
            capitalized_calls: true,
            derived_computations_in_effect: true,
            memo_dependencies: true,
            preserve_manual_memo: true,
            immutability: true,
            use_memo: true,
        }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Runs the React Compiler's analysis in lint-only mode and reports code
    /// that violates the Rules of React — for example calling hooks
    /// conditionally, calling setState during render, accessing refs during
    /// render, or mutating props and state.
    ///
    /// This rule surfaces the same diagnostics as `eslint-plugin-react-compiler`.
    ///
    /// ::: warning
    /// This rule is experimental, and will change to fit in better with Oxlint.
    /// :::
    ///
    /// ### Why is this bad?
    ///
    /// Code that breaks the Rules of React can behave unpredictably at runtime
    /// (stale UI, infinite re-render loops, lost state) and prevents React
    /// Compiler from optimizing the component. Following these rules keeps
    /// components correct and allows them to be automatically memoized.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// function Component(props) {
    ///   if (props.cond) {
    ///     useState(0); // hooks may not be called conditionally
    ///   }
    ///   return <div>{props.text}</div>;
    /// }
    /// ```
    ///
    /// ```jsx
    /// function Component() {
    ///   const ref = useRef(null);
    ///   return <div>{ref.current}</div>; // refs may not be read during render
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// function Component(props) {
    ///   const [state, setState] = useState(0);
    ///   return <button onClick={() => setState(state + 1)}>{props.text}</button>;
    /// }
    /// ```
    ReactCompiler,
    react,
    nursery,
    config = ReactCompilerConfig,
    version = "1.70.0",
);

impl Rule for ReactCompiler {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run_once(&self, ctx: &LintContext) {
        let program = ctx.nodes().program();
        let options =
            react_compiler_options(self);

        let result = oxc_react_compiler::lint(program, ctx.semantic(), ctx.allocator(), options);

        let suppressed_lines = flow_suppression_lines(ctx);
        for mut diagnostic in result.diagnostics.into_vec() {
            // Bail-outs surface as non-error severities; hide them unless asked.
            if !self.report_all_bailouts && diagnostic.severity != Severity::Error {
                continue;
            }

            let category = react_compiler_category(&diagnostic.message);

            // `Immutability` and `UseMemo` have no compiler flag, so the
            // sub-rules are enforced here by dropping their diagnostics when the
            // toggle is off.
            if !self.immutability && category == Some("Immutability") {
                continue;
            }
            if !self.use_memo && category == Some("UseMemo") {
                continue;
            }

            // Internal compiler errors are not Rules of React violations; hide
            // them unless the user opts into the full firehose.
            if !self.report_all_bailouts && category.is_some_and(is_internal_noise) {
                continue;
            }

            // If Flow already caught this error, we don't need to report it again.
            if is_flow_suppressed(&diagnostic, &suppressed_lines, ctx.source_text()) {
                continue;
            }
            // The rule code already identifies the source, so the prefix is noise here.
            if let Some(message) = diagnostic.message.strip_prefix("[ReactCompiler] ") {
                diagnostic.message = Cow::Owned(message.to_string());
            }
            ctx.diagnostic(diagnostic);
        }
    }
}

/// The leading category token of a React Compiler message — e.g. `Immutability`
/// from `"[ReactCompiler] Immutability: Cannot reassign ..."`. Returns `None`
/// when the message carries no category, as some fatal-error paths do. Reliable
/// because the rule runs with `panicThreshold: "none"`, so categorized
/// diagnostics always render with the `"{category}: {reason}"` shape.
fn react_compiler_category(message: &str) -> Option<&str> {
    message.strip_prefix("[ReactCompiler] ").and_then(|rest| rest.split_once(": ")).map(|(c, _)| c)
}

/// Categories that are internal compiler errors rather than Rules of React
/// violations. `Invariant` is a compiler-internal assertion; `Unexpected error`
/// and `Pipeline error` are thrown failures. They are hidden unless
/// `reportAllBailouts` is set.
fn is_internal_noise(category: &str) -> bool {
    matches!(category, "Invariant" | "Unexpected error" | "Pipeline error")
}

/// Flow suppression codes that silence a React Compiler diagnostic on the next
/// line, matching the eslint plugin's `hasFlowSuppression` check.
const FLOW_SUPPRESSIONS: [&str; 2] =
    ["$FlowFixMe[react-rule-hook]", "$FlowFixMe[react-rule-unsafe-ref]"];

/// Lines (0-based) on which a recognized Flow suppression comment ends.
fn flow_suppression_lines(ctx: &LintContext) -> Vec<u32> {
    let source_text = ctx.source_text();
    ctx.semantic()
        .comments()
        .iter()
        .filter(|comment| {
            let text = comment.span.source_text(source_text);
            FLOW_SUPPRESSIONS.iter().any(|suppression| text.contains(suppression))
        })
        .map(|comment| line_number(source_text, comment.span.end))
        .collect()
}

/// A diagnostic is suppressed when a Flow suppression comment sits on the line
/// directly above its primary location.
fn is_flow_suppressed(
    diagnostic: &oxc_diagnostics::OxcDiagnostic,
    suppressed_lines: &[u32],
    source_text: &str,
) -> bool {
    if suppressed_lines.is_empty() {
        return false;
    }
    let Some(label) = diagnostic.labels.first() else {
        return false;
    };
    let line = line_number(source_text, label.offset());
    line.checked_sub(1).is_some_and(|previous| suppressed_lines.contains(&previous))
}

fn line_number(source_text: &str, offset: u32) -> u32 {
    let end = (offset as usize).min(source_text.len());
    u32::try_from(memchr::memchr_iter(b'\n', &source_text.as_bytes()[..end]).count())
        .unwrap_or(u32::MAX)
}

// Test cases ported from `eslint-plugin-react-compiler` (MIT licensed):
// <https://github.com/facebook/react/tree/main/compiler/packages/eslint-plugin-react-compiler/__tests__>
// `RustBackendComparison-test.ts` only replicates the cases below to compare the
// TS and Rust backends, so it adds no new cases.
//
// The one case that cannot be ported is "Basic example with component syntax"
// (PluginTest-test.ts), which uses Flow's experimental `component` declaration
// syntax that oxc does not parse.
#[test]
fn test() {
    use serde_json::json;

    use crate::tester::Tester;

    let pass = vec![
        // ---- PluginTest-test.ts ----
        // [Invariant] Defined after use
        // (OK because invariants are only meant for the compiler team's consumption)
        (
            "
function Component(props) {
  let y = function () {
    m(x);
  };

  let x = { a };
  m(x);
  return y;
}
",
            None,
        ),
        // Classes don't throw
        (
            "
class Foo {
  #bar() {}
}
",
            None,
        ),
        // ---- InvalidHooksRule-test.ts ----
        // Basic example
        (
            "
function Component() {
  useHook();
  return <div>Hello world</div>;
}
",
            None,
        ),
        // Violation with Flow suppression
        (
            "
      // Valid since error already suppressed with flow.
      function useHook() {
        if (cond) {
          // $FlowFixMe[react-rule-hook]
          useConditionalHook();
        }
      }
    ",
            None,
        ),
        // ---- ReactCompilerRuleTypescript-test.ts ----
        // Basic example
        (
            "
function Button(props) {
  return null;
}
",
            None,
        ),
        // Repro for hooks as normal values
        (
            "
function Button(props) {
  const scrollview = React.useRef<ScrollView>(null);
  return <Button thing={scrollview} />;
}
",
            None,
        ),
        // ---- RustBackend-test.ts ----
        // Basic component compiles without errors
        (
            "
function Component(props) {
  return <div>{props.text}</div>;
}
",
            None,
        ),
        // Component with hooks compiles without errors
        (
            "
import {useState} from 'react';
function Component(props) {
  const [state, setState] = useState(0);
  return <div onClick={() => setState(state + 1)}>{state}</div>;
}
",
            None,
        ),
        // ---- oxlint-specific ----
        // A bail-out (local named `fbt`) is not a violation, only a skipped
        // optimization; it is not reported by default.
        (
            "function Component() {
                const fbt = 'span';
                return <fbt desc='label'>Hello</fbt>;
            }",
            None,
        ),
        // A sub-rule toggled off in the config does not report: this would
        // otherwise be a `Refs` violation (see the matching `fail` case).
        (
            "
function Component(props) {
  const ref = useRef(null);
  const value = ref.current;
  return value;
}
",
            Some(json!([{ "refs": false }])),
        ),
        // `immutability` is filtered in the rule (the compiler has no flag for
        // it): turning it off silences the `Immutability` diagnostic that the
        // matching `fail` case (mutating a `useState` value) otherwise reports.
        (
            "
import { useState } from 'react';
function Component(props) {
  const [state, setState] = useState({a: 0});
  state.a = 1;
  return <div>{props.foo}</div>;
}
",
            Some(json!([{ "immutability": false }])),
        ),
        // `useMemo` is also rule-layer filtered: turning it off silences the
        // `UseMemo` diagnostic for a malformed (async) `useMemo` callback.
        (
            "
import { useMemo } from 'react';
function Component(props) {
  const x = useMemo(async () => props.a + 1, [props.a]);
  return <div>{x}</div>;
}
",
            Some(json!([{ "useMemo": false }])),
        ),
    ];

    let fail = vec![
        // ---- PluginTest-test.ts ----
        // Multiple diagnostic kinds from the same function are surfaced
        (
            "
import Child from './Child';
function Component() {
  const result = cond ?? useConditionalHook();
  return <>
    {Child(result)}
  </>;
}
",
            None,
        ),
        // Multiple diagnostics within the same file are surfaced
        (
            "
function useConditional1() {
  'use memo';
  return cond ?? useConditionalHook();
}
function useConditional2(props) {
  'use memo';
  return props.cond && useConditionalHook();
}",
            None,
        ),
        // 'use no forget' does not disable eslint rule
        (
            "
let count = 0;
function Component() {
  'use no forget';
  return cond ?? useConditionalHook();

}
",
            None,
        ),
        // Multiple non-fatal useMemo diagnostics are surfaced
        (
            "
import {useMemo, useState} from 'react';

function Component({item, cond}) {
  const [prevItem, setPrevItem] = useState(item);
  const [state, setState] = useState(0);

  useMemo(() => {
    if (cond) {
      setPrevItem(item);
      setState(0);
    }
  }, [cond, item, init]);

  return <Child x={state} />;
  }",
            None,
        ),
        // ---- InvalidHooksRule-test.ts ----
        // Simple violation
        (
            "
function useConditional() {
  if (cond) {
    useConditionalHook();
  }
}
",
            None,
        ),
        // Multiple diagnostics within the same function are surfaced
        (
            "
function useConditional() {
  cond ?? useConditionalHook();
  props.cond && useConditionalHook();
  return <div>Hello world</div>;
}",
            None,
        ),
        // ---- ImpureFunctionCallsRule-test.ts ----
        // Known impure function calls are caught
        (
            "
function Component() {
  const date = Date.now();
  const now = performance.now();
  const rand = Math.random();
  return <Foo date={date} now={now} rand={rand} />;
}
",
            None,
        ),
        // ---- NoAmbiguousJsxRule-test.ts ----
        // JSX in try blocks are warned against
        (
            "
function Component(props) {
  let el;
  try {
    el = <Child />;
  } catch {
    return null;
  }
  return el;
}
",
            None,
        ),
        // ---- NoCapitalizedCallsRule-test.ts ----
        // Simple violation
        (
            "
import Child from './Child';
function Component() {
  return <>
    {Child()}
  </>;
}
",
            None,
        ),
        // Method call violation
        (
            "
import myModule from './MyModule';
function Component() {
  return <>
    {myModule.Child()}
  </>;
}
",
            None,
        ),
        // Multiple diagnostics within the same function are surfaced
        (
            "
import Child1 from './Child1';
import MyModule from './MyModule';
function Component() {
  return <>
    {Child1()}
    {MyModule.Child2()}
  </>;
}",
            None,
        ),
        // ---- NoRefAccessInRender-tests.ts ----
        // validate against simple ref access in render
        (
            "
function Component(props) {
  const ref = useRef(null);
  const value = ref.current;
  return value;
}
",
            None,
        ),
        // ---- ReactCompilerRuleTypescript-test.ts ----
        // Mutating useState value
        (
            "
        import { useState } from 'react';
        function Component(props) {
          // typescript syntax that hermes-parser doesn't understand yet
          const x: `foo${1}` = 'foo1';
          const [state, setState] = useState({a: 0});
          state.a = 1;
          return <div>{props.foo}</div>;
        }
      ",
            None,
        ),
        // ---- RustBackend-test.ts ----
        // Conditional hook call detected by Rust backend
        (
            "
function Component() {
  const result = cond ?? useConditionalHook();
  return <div>{result}</div>;
}
",
            None,
        ),
        // Multiple diagnostics detected by Rust backend
        (
            "
function useConditional1() {
  'use memo';
  return cond ?? useConditionalHook();
}
function useConditional2(props) {
  'use memo';
  return props.cond && useConditionalHook();
}
",
            None,
        ),
        // ---- oxlint-specific ----
        // Bail-outs are reported when `reportAllBailouts` is enabled.
        (
            "function Component() {
                const fbt = 'span';
                return <fbt desc='label'>Hello</fbt>;
            }",
            Some(json!([{ "reportAllBailouts": true }])),
        ),
    ];

    Tester::new(ReactCompiler::NAME, ReactCompiler::PLUGIN, pass, fail).test_and_snapshot();
}
