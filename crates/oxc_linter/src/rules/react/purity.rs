use oxc_react_compiler::ErrorCategory;

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
    utils::{run_react_compiler_rule, should_run_react_compiler},
};

#[derive(Debug, Default, Clone)]
pub struct Purity;

declare_react_compiler_lint!(
    /// ### What it does
    ///
    /// Validates that components and hooks are pure by checking that they do
    /// not call known-impure functions such as `Math.random()`, `Date.now()`,
    /// or `performance.now()` during render.
    ///
    /// Async function components are exempt, including impure calls in nested
    /// callbacks. An async component is a Server Component: it renders once per
    /// request and does not re-render. Async hooks are still checked.
    ///
    upstream = "purity",
    ///
    /// ### Why is this bad?
    ///
    /// Impure renders return different output for the same props and state,
    /// breaking memoization, concurrent rendering, and replayability.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// function Component() {
    ///   const rand = Math.random();
    ///   return <div>{rand}</div>;
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// import { useState } from 'react';
    /// function Component() {
    ///   const [rand, setRand] = useState(0);
    ///   return <button onClick={() => setRand(Math.random())}>{rand}</button>;
    /// }
    ///
    /// async function Posts({ posts }) {
    ///   return <ul>{posts.map(post => <li key={post.id}>{Date.now()}</li>)}</ul>;
    /// }
    /// ```
    Purity,
    react,
    correctness,
    recommended,
    version = "1.79.0",
    short_description = "Validates that components and hooks do not call known-impure functions.",
);

impl Rule for Purity {
    fn run_once(&self, ctx: &LintContext) {
        run_react_compiler_rule(ctx, ErrorCategory::Purity);
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        should_run_react_compiler(ctx)
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // ---- ImpureFunctionCallsRule-test.ts ----
        // Date constructed from a timestamp is not flagged
        "
function Component({ timestamp }) {
  const date = new Date(timestamp);
  return <Foo date={date} />;
}
",
        // ---- RustBackend-test.ts ----
        // Basic component compiles without errors
        "
function Component(props) {
  return <div>{props.text}</div>;
}
",
        // https://github.com/oxc-project/oxc/issues/27974
        "
export default async function Posts() {
  const posts = await getPosts();
  return posts.map(p => <p key={p.id}>{Date.now() - p.createdAt < 86_400_000 ? 'new' : ''}</p>);
}
",
        "
async function Posts() {
  const date = Date.now();
  const now = new Date();
  const rand = Math.random();
  return <Foo date={date} now={now} rand={rand} />;
}
",
        // Async arrow components
        "
const Posts = async () => {
  return <p>{Date.now()}</p>;
};
",
        "
const Posts = async function () {
  return <p>{new Date().getTime()}</p>;
};
",
        // Nested callbacks run during that same server render.
        "
async function Posts({ posts }) {
  return <ul>{posts.map(p => <li key={p.id}>{Date.now() - p.createdAt}</li>)}</ul>;
}
",
        "
async function Posts({ posts }) {
  const label = (post) => (Date.now() - post.createdAt < 86400000 ? 'new' : '');
  return <ul>{posts.map(p => <li key={p.id}>{label(p)}</li>)}</ul>;
}
",
        // A nested component also renders as part of that server render.
        "
async function Posts() {
  function Badge() {
    return <span>{Date.now()}</span>;
  }
  return <Badge />;
}
",
    ];

    let fail = vec![
        // ---- ImpureFunctionCallsRule-test.ts ----
        // Known impure function calls are caught
        "
function Component() {
  const date = Date.now();
  const now = performance.now();
  const rand = Math.random();
  return <Foo date={date} now={now} rand={rand} />;
}
",
        // Zero-argument Date constructor is impure
        "
function Component() {
  const date = new Date();
  const time = new Date().getTime();
  const year = new Date().getFullYear();
  return <Foo date={date} time={time} year={year} />;
}
",
        // Non-async components still report, including beside an async component.
        "
function Client() {
  return <p>{Date.now()}</p>;
}
async function Server() {
  return <p>{Date.now()}</p>;
}
",
        "
const Client = () => {
  return <p>{Math.random()}</p>;
};
",
        // Nested callbacks of a non-async component still run on every render.
        "
function Posts({ posts }) {
  return <ul>{posts.map(p => <li key={p.id}>{Date.now()}</li>)}</ul>;
}
",
        // Hooks stay checked, including async hooks.
        "
function useNow() {
  return <span>{Date.now()}</span>;
}
",
        "
async function useNow() {
  return <span>{new Date().getTime()}</span>;
}
",
    ];

    Tester::new(Purity::NAME, Purity::PLUGIN, pass, fail).test_and_snapshot();
}

#[test]
fn skips_node_modules() {
    use std::path::PathBuf;

    use crate::tester::Tester;

    let pass = vec![(
        "function Component() { return <div>{Date.now()}</div>; }",
        None,
        None,
        Some(PathBuf::from("node_modules/package/Component.tsx")),
    )];

    Tester::new(Purity::NAME, Purity::PLUGIN, pass, vec![]).test();
}
