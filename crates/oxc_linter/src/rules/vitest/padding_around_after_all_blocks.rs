use oxc_macros::declare_oxc_lint;

use crate::{
    context::LintContext,
    rule::Rule,
    rules::shared::padding_around_after_all_blocks::{DOCUMENTATION, run},
    utils::PossibleJestNode,
};

#[derive(Debug, Default, Clone)]
pub struct PaddingAroundAfterAllBlocks;

declare_oxc_lint!(
    PaddingAroundAfterAllBlocks,
    vitest,
    style,
    fix,
    docs = DOCUMENTATION,
    version = "next",
);

impl Rule for PaddingAroundAfterAllBlocks {
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
        "afterAll(() => {});",
        "const thing = 123;\n\nafterAll(() => {});",
        "describe('foo', () => {\nafterAll(() => {});\n});",
        "// This is a comment\nafterAll(() => {});",
        "import { afterAll } from 'vitest';\n\nafterAll(() => {});",
        "import { afterAll, describe } from 'vitest';\nimport { helper } from './helper';\n\nafterAll(() => {});",
        "import './setup';\n\nafterAll(() => {});",
        "/* leading block comment */\nafterAll(() => {});",
        "/**\n * JSDoc-style comment\n */\nafterAll(() => {});",
        "const thing = 123;\n\n/* attached to afterAll */\nafterAll(() => {});",
        "const thing = 123;\n\n/**\n * JSDoc attached to afterAll\n */\nafterAll(() => {});",
        "const thing = 123; /* trailing on prev */\n\nafterAll(() => {});",
        "describe('foo', () => {\nafterAll(() => {});\n\nafterAll(() => {});\n});",
    ];

    let fail = vec![
        "const thing = 123;\nafterAll(() => {});",
        "const thing = 123;\n//My comment\nafterAll(() => {});",
        "import { afterAll } from 'vitest';\nafterAll(() => {});",
        "import { afterAll } from 'vitest';\nimport { helper } from './helper';\nafterAll(() => {});",
        "import './setup';\nafterAll(() => {});",
        "const thing = 123;\n/* block comment */\nafterAll(() => {});",
        "const thing = 123;\n/**\n * JSDoc comment\n */\nafterAll(() => {});",
        "describe('foo', () => {\nafterAll(() => {});\nafterAll(() => {});\n});",
        "import { afterAll } from 'vitest';\n/* setup notes */\nafterAll(() => {});",
    ];

    let fix = vec![
        ("const thing = 123;\nafterAll(() => {});", "const thing = 123;\n\nafterAll(() => {});"),
        (
            "const thing = 123;\n// This is a comment\nafterAll(() => {});",
            "const thing = 123;\n\n// This is a comment\nafterAll(() => {});",
        ),
        (
            "import { afterAll } from 'vitest';\nafterAll(() => {});",
            "import { afterAll } from 'vitest';\n\nafterAll(() => {});",
        ),
        ("import './setup';\nafterAll(() => {});", "import './setup';\n\nafterAll(() => {});"),
        (
            "const thing = 123;\n/* block comment */\nafterAll(() => {});",
            "const thing = 123;\n\n/* block comment */\nafterAll(() => {});",
        ),
        (
            "const thing = 123;\n/**\n * JSDoc comment\n */\nafterAll(() => {});",
            "const thing = 123;\n\n/**\n * JSDoc comment\n */\nafterAll(() => {});",
        ),
        (
            "describe('foo', () => {\nafterAll(() => {});\nafterAll(() => {});\n});",
            "describe('foo', () => {\nafterAll(() => {});\n\nafterAll(() => {});\n});",
        ),
    ];

    Tester::new(PaddingAroundAfterAllBlocks::NAME, PaddingAroundAfterAllBlocks::PLUGIN, pass, fail)
        .with_vitest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
