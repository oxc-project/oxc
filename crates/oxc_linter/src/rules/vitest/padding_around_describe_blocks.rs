use oxc_macros::declare_oxc_lint;

use crate::{
    context::LintContext,
    rule::Rule,
    rules::shared::padding_around_describe_blocks::{DOCUMENTATION, run},
    utils::PossibleJestNode,
};

#[derive(Debug, Default, Clone)]
pub struct PaddingAroundDescribeBlocks;

declare_oxc_lint!(
    PaddingAroundDescribeBlocks,
    vitest,
    style,
    fix,
    docs = DOCUMENTATION,
    // TODO: confirm this matches the actual next-unreleased crate version at build time
    // (crates/oxc_linter/Cargo.toml was at 1.79.0 in this checkout).
    version = "1.80.0",
);

impl Rule for PaddingAroundDescribeBlocks {
    fn run_on_jest_node<'a, 'c>(
        &self,
        jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        run(jest_node, ctx);
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "describe('foo', () => {});",
        "const thing = 123;\n\ndescribe('foo', () => {});",
        "describe('foo', () => {\ndescribe('bar', () => {});\n});",
        "// This is a comment\ndescribe('foo', () => {});",
        "import { describe } from 'vitest';\n\ndescribe('foo', () => {});",
        "import { describe } from 'vitest';\nimport { helper } from './helper';\n\ndescribe('foo', () => {});",
        "import './setup';\n\ndescribe('foo', () => {});",
        "/* leading block comment */\ndescribe('foo', () => {});",
        "/**\n * JSDoc-style comment\n */\ndescribe('foo', () => {});",
        "const thing = 123;\n\n/* attached to describe */\ndescribe('foo', () => {});",
        "const thing = 123;\n\n/**\n * JSDoc attached to describe\n */\ndescribe('foo', () => {});",
        "const thing = 123; /* trailing on prev */\n\ndescribe('foo', () => {});",
        "describe('foo', () => {\ndescribe('bar', () => {});\n\ndescribe('baz', () => {});\n});",
        // `fdescribe`/`xdescribe` are Jest-only globals, not part of Vitest's API,
        // so Vitest-mode parsing doesn't recognize them as describe blocks at all
        // (see the jest wrapper's test for coverage of those aliases).
        "const thing = 123;\nfdescribe('foo', () => {});",
        "const thing = 123;\nxdescribe('foo', () => {});",
        "const thing = 123;\n\ndescribe.each([1, 2])('foo %i', () => {});",
        "const thing = 123;\n\ndescribe.only('foo', () => {});",
        "const thing = 123;\n\ndescribe.skip('foo', () => {});",
        "const thing = 123;\n\nsuite('foo', () => {});",
        "const thing = 123;\n\nsuite.each([1, 2])('foo %i', () => {});",
        "describe('foo', () => {});\n\ndescribe('bar', () => {});",
    ];

    let fail = vec![
        "const thing = 123;\ndescribe('foo', () => {});",
        "const thing = 123;\n//My comment\ndescribe('foo', () => {});",
        "import { describe } from 'vitest';\ndescribe('foo', () => {});",
        "import { describe } from 'vitest';\nimport { helper } from './helper';\ndescribe('foo', () => {});",
        "import './setup';\ndescribe('foo', () => {});",
        "const thing = 123;\n/* block comment */\ndescribe('foo', () => {});",
        "const thing = 123;\n/**\n * JSDoc comment\n */\ndescribe('foo', () => {});",
        "describe('foo', () => {\ndescribe('bar', () => {});\ndescribe('baz', () => {});\n});",
        "import { describe } from 'vitest';\n/* setup notes */\ndescribe('foo', () => {});",
        "const thing = 123;\ndescribe.each([1, 2])('foo %i', () => {});",
        "const thing = 123;\ndescribe.only('foo', () => {});",
        "const thing = 123;\ndescribe.skip('foo', () => {});",
        "const thing = 123;\nsuite('foo', () => {});",
        "const thing = 123;\nsuite.each([1, 2])('foo %i', () => {});",
        "describe('foo', () => {});\ndescribe('bar', () => {});",
    ];

    let fix = vec![
        (
            "const thing = 123;\ndescribe('foo', () => {});",
            "const thing = 123;\n\ndescribe('foo', () => {});",
        ),
        (
            "const thing = 123;\n// This is a comment\ndescribe('foo', () => {});",
            "const thing = 123;\n\n// This is a comment\ndescribe('foo', () => {});",
        ),
        (
            "import { describe } from 'vitest';\ndescribe('foo', () => {});",
            "import { describe } from 'vitest';\n\ndescribe('foo', () => {});",
        ),
        (
            "import './setup';\ndescribe('foo', () => {});",
            "import './setup';\n\ndescribe('foo', () => {});",
        ),
        (
            "const thing = 123;\n/* block comment */\ndescribe('foo', () => {});",
            "const thing = 123;\n\n/* block comment */\ndescribe('foo', () => {});",
        ),
        (
            "const thing = 123;\n/**\n * JSDoc comment\n */\ndescribe('foo', () => {});",
            "const thing = 123;\n\n/**\n * JSDoc comment\n */\ndescribe('foo', () => {});",
        ),
        (
            "describe('foo', () => {\ndescribe('bar', () => {});\ndescribe('baz', () => {});\n});",
            "describe('foo', () => {\ndescribe('bar', () => {});\n\ndescribe('baz', () => {});\n});",
        ),
        (
            "const thing = 123;\nsuite('foo', () => {});",
            "const thing = 123;\n\nsuite('foo', () => {});",
        ),
    ];

    Tester::new(PaddingAroundDescribeBlocks::NAME, PaddingAroundDescribeBlocks::PLUGIN, pass, fail)
        .with_vitest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
