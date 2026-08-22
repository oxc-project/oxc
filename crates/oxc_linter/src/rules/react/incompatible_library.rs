use oxc_react_compiler::ErrorCategory;

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
    utils::{run_react_compiler_rule, should_run_react_compiler},
};

#[derive(Debug, Default, Clone)]
pub struct IncompatibleLibrary;

declare_react_compiler_lint!(
    /// ### What it does
    ///
    /// Warns on usage of library APIs known to be incompatible with
    /// memoization (manual or automatic), such as `react-hook-form`'s
    /// `watch()`, TanStack Table's `useReactTable()`, and TanStack Virtual's
    /// `useVirtualizer()`.
    ///
    upstream = "incompatible-library",
    ///
    /// ### Why is this bad?
    ///
    /// These APIs rely on components re-rendering on every change;
    /// memoization — by the compiler or by hand — breaks their update model,
    /// so the UI stops reflecting new data.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// import { useReactTable } from '@tanstack/react-table';
    /// function Component({ columns, data }) {
    ///   const table = useReactTable({ columns, data });
    ///   return <div>{table.getRowModel().rows.length}</div>;
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// function Component({ rows }) {
    ///   return <div>{rows.length}</div>;
    /// }
    /// ```
    IncompatibleLibrary,
    react,
    correctness,
    version = "1.79.0",
    short_description = "Warns on usage of libraries that are incompatible with memoization.",
);

impl Rule for IncompatibleLibrary {
    fn run_once(&self, ctx: &LintContext) {
        run_react_compiler_rule(ctx, ErrorCategory::IncompatibleLibrary);
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        should_run_react_compiler(ctx)
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "
function Component(props) {
  return <div>{props.text}</div>;
}
",
    ];

    let fail = vec![
        // Derived from crates/oxc_react_compiler/fixtures cases.
        "
import {useReactTable} from '@tanstack/react-table';
function Component({columns, data}) {
  const table = useReactTable({columns, data});
  return <div>{table.getRowModel().rows.length}</div>;
}
",
    ];

    Tester::new(IncompatibleLibrary::NAME, IncompatibleLibrary::PLUGIN, pass, fail)
        .test_and_snapshot();
}
