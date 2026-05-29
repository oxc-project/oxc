use oxc_macros::declare_oxc_lint;

use crate::{
    context::LintContext,
    rule::Rule,
    rules::shared::padding_around_before_all_blocks::{DOCUMENTATION, run},
    utils::PossibleJestNode,
};

#[derive(Debug, Default, Clone)]
pub struct PaddingAroundBeforeAllBlocks;

declare_oxc_lint!(
    PaddingAroundBeforeAllBlocks,
    vitest,
    style,
    fix,
    docs = DOCUMENTATION,
    version = "1.67.0",
);

impl Rule for PaddingAroundBeforeAllBlocks {
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
        "beforeAll(() => {});",
        "const thing = 123;\n\nbeforeAll(() => {});",
        "describe('foo', () => {\nbeforeAll(() => {});\n});",
        "// This is a comment\nbeforeAll(() => {});",
        "import { beforeAll } from 'vitest';\n\nbeforeAll(() => {});",
        "import { beforeAll, describe } from 'vitest';\nimport { helper } from './helper';\n\nbeforeAll(() => {});",
        "import './setup';\n\nbeforeAll(() => {});",
        "/* leading block comment */\nbeforeAll(() => {});",
        "/**\n * JSDoc-style comment\n */\nbeforeAll(() => {});",
        "const thing = 123;\n\n/* attached to beforeAll */\nbeforeAll(() => {});",
        "const thing = 123;\n\n/**\n * JSDoc attached to beforeAll\n */\nbeforeAll(() => {});",
        "const thing = 123; /* trailing on prev */\n\nbeforeAll(() => {});",
        "describe('foo', () => {\nbeforeAll(() => {});\n\nbeforeAll(() => {});\n});",
    ];

    let fail = vec![
        "const thing = 123;\nbeforeAll(() => {});",
        "const thing = 123;\n//My comment\nbeforeAll(() => {});",
        "import { beforeAll } from 'vitest';\nbeforeAll(() => {});",
        "import { beforeAll } from 'vitest';\nimport { helper } from './helper';\nbeforeAll(() => {});",
        "import './setup';\nbeforeAll(() => {});",
        "const thing = 123;\n/* block comment */\nbeforeAll(() => {});",
        "const thing = 123;\n/**\n * JSDoc comment\n */\nbeforeAll(() => {});",
        "describe('foo', () => {\nbeforeAll(() => {});\nbeforeAll(() => {});\n});",
        "import { beforeAll } from 'vitest';\n/* setup notes */\nbeforeAll(() => {});",
    ];

    let fix = vec![
        ("const thing = 123;\nbeforeAll(() => {});", "const thing = 123;\n\nbeforeAll(() => {});"),
        (
            "const thing = 123;\n// This is a comment\nbeforeAll(() => {});",
            "const thing = 123;\n\n// This is a comment\nbeforeAll(() => {});",
        ),
        (
            "import { beforeAll } from 'vitest';\nbeforeAll(() => {});",
            "import { beforeAll } from 'vitest';\n\nbeforeAll(() => {});",
        ),
        ("import './setup';\nbeforeAll(() => {});", "import './setup';\n\nbeforeAll(() => {});"),
        (
            "const thing = 123;\n/* block comment */\nbeforeAll(() => {});",
            "const thing = 123;\n\n/* block comment */\nbeforeAll(() => {});",
        ),
        (
            "const thing = 123;\n/**\n * JSDoc comment\n */\nbeforeAll(() => {});",
            "const thing = 123;\n\n/**\n * JSDoc comment\n */\nbeforeAll(() => {});",
        ),
        (
            "describe('foo', () => {\nbeforeAll(() => {});\nbeforeAll(() => {});\n});",
            "describe('foo', () => {\nbeforeAll(() => {});\n\nbeforeAll(() => {});\n});",
        ),
    ];

    Tester::new(
        PaddingAroundBeforeAllBlocks::NAME,
        PaddingAroundBeforeAllBlocks::PLUGIN,
        pass,
        fail,
    )
    .with_vitest_plugin(true)
    .expect_fix(fix)
    .test_and_snapshot();
}
