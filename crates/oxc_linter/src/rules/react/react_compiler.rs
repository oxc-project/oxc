use std::borrow::Cow;

use oxc_diagnostics::Severity;
use oxc_macros::declare_oxc_lint;
use oxc_react_compiler::{EnvironmentConfig, PluginOptions};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

/// The compiler options `eslint-plugin-react-compiler` lints with — `lint`
/// output mode plus validations that are off by default in the compiler.
/// Mirrors `COMPILER_OPTIONS` in the plugin's `src/shared/RunReactCompiler.ts`.
fn react_compiler_options() -> PluginOptions {
    PluginOptions {
        output_mode: Some("lint".to_string()),
        // Don't emit errors on Flow suppressions — Flow already gave a signal.
        // Suppressed lines are filtered in `run_once` instead (like the eslint
        // plugin), so other diagnostics in the same function still surface.
        flow_suppressions: false,
        environment: EnvironmentConfig {
            validate_ref_access_during_render: true,
            validate_no_set_state_in_render: true,
            validate_no_set_state_in_effects: true,
            validate_no_jsx_in_try_statements: true,
            validate_no_impure_functions_in_render: true,
            validate_static_components: true,
            validate_no_freezing_known_mutable_functions: true,
            validate_no_void_use_memo: true,
            validate_no_capitalized_calls: Some(vec![]),
            validate_hooks_usage: true,
            validate_no_derived_computations_in_effects: true,
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

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct ReactCompilerConfig {
    /// Also report compiler bail-outs — places where React Compiler skipped a
    /// component or hook (for example because of unsupported syntax) without
    /// finding a rule violation. These do not indicate incorrect code, only
    /// code that the compiler declined to optimize.
    report_all_bailouts: bool,
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
        let options = react_compiler_options();

        let result = oxc_react_compiler::lint(program, ctx.semantic(), ctx.allocator(), options);

        let diagnostics = result.diagnostics.into_vec();
        let diagnostics = if self.report_all_bailouts {
            diagnostics
        } else {
            diagnostics
                .into_iter()
                .filter(|diagnostic| diagnostic.severity == Severity::Error)
                .collect::<Vec<_>>()
        };

        let suppressed_lines = flow_suppression_lines(ctx);
        for mut diagnostic in diagnostics {
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
