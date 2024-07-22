//! Test cases created by oxc maintainers

use super::NoUnusedVars;
use crate::{tester::Tester, RuleMeta as _};
use serde_json::json;

#[test]
fn test_simple_variables() {
    let pass = vec!["let a = 1; console.log(a)", "let a = 1; let b = a + 1; console.log(b)"];
    let fail = vec!["let a = 1", "let a = 1; a = 2"];

    Tester::new(NoUnusedVars::NAME, pass, fail)
        .test_and_snapshot_with_suffix("oxc_simple_variables");
}

#[test]
fn test_catch() {
    let pass = vec![
        // lb
        ("try {} catch (e) { throw e }", None),
        ("try {} catch (e) { }", Some(json!([{ "caughtErrors": "none" }]))),
        ("try {} catch { }", None),
    ];
    let fail = vec![
        // lb
        ("try {} catch (e) { }", Some(json!([{ "caughtErrors": "all" }]))),
    ];

    Tester::new(NoUnusedVars::NAME, pass, fail).test_and_snapshot_with_suffix("oxc_catch");
}

#[test]
fn test_imports() {
    let pass = vec![
        "import { a } from 'b'; console.log(a)",
        "import * as a from 'a'; console.log(a)",
        "import a from 'a'; console.log(a)",
        "import { default as a } from 'a'; console.log(a)",
    ];
    let fail = vec![
        "import { a } from 'a'",
        "import * as a from 'a'",
        "import { a as b } from 'a'; console.log(a)",
    ];

    Tester::new(NoUnusedVars::NAME, pass, fail).test_and_snapshot_with_suffix("oxc_imports");
}

#[test]
fn test_exports() {
    let pass = vec![
        "export const a = 1; console.log(a)",
        "export function foo() {}",
        "export default function foo() {}",
        "export class A {}",
        "export interface A {}",
        "export type A = string",
        "export enum E { }",
        // "export enum E { A, B }",
        "const a = 1; export { a }",
        "const a = 1; export default a",
        // re-exports
        "import { a } from 'a'; export { a }",
        "import { a as b } from 'a'; export { b }",
        "export * as a from 'a'",
        "export { a, b } from 'a'",
    ];
    let fail = vec!["import { a as b } from 'a'; export { a }"];

    // these are mostly pass[] cases, so do not snapshot
    Tester::new(NoUnusedVars::NAME, pass, fail).test();
}

#[test]
fn test_arguments() {
    let pass = vec![
        ("function foo(a) { return a } foo()", None),
        ("function foo(a, b) { return b } foo()", Some(json!([{ "args": "after-used" }]))),
    ];
    let fail = vec![
        ("function foo(a) {} foo()", None),
        ("function foo({ a }, b) { return b } foo()", Some(json!([{ "args": "after-used" }]))),
    ];

    Tester::new(NoUnusedVars::NAME, pass, fail).test_and_snapshot_with_suffix("oxc_arguments");
}
