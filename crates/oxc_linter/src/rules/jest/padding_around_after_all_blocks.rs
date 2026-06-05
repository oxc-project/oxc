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
    jest,
    style,
    fix,
    docs = DOCUMENTATION,
    version = "1.59.0",
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
        "import { afterAll } from '@jest/globals';\n\nafterAll(() => {});",
        "import { afterAll, describe } from '@jest/globals';\nimport { helper } from './helper';\n\nafterAll(() => {});",
        "import './setup';\n\nafterAll(() => {});",
        "/* leading block comment */\nafterAll(() => {});",
        "/**\n * JSDoc-style comment\n */\nafterAll(() => {});",
        "const thing = 123;\n\n/* attached to afterAll */\nafterAll(() => {});",
        "const thing = 123;\n\n/**\n * JSDoc attached to afterAll\n */\nafterAll(() => {});",
        "const thing = 123; /* trailing on prev */\n\nafterAll(() => {});",
        "describe('foo', () => {\nafterAll(() => {});\n\nafterAll(() => {});\n});",
        "const thing = 123;\n\n/* one */\n/* two */\nafterAll(() => {});",
        "const thing = 123;\n\nafterAll(() => {});\n\nconst otherThing = 123;",
    ];

    let fail = vec![
        "const thing = 123;\nafterAll(() => {});",
        "const thing = 123;\n//My comment\nafterAll(() => {});",
        "import { afterAll } from '@jest/globals';\nafterAll(() => {});",
        "import { afterAll } from '@jest/globals';\nimport { helper } from './helper';\nafterAll(() => {});",
        "import './setup';\nafterAll(() => {});",
        "const thing = 123;\n/* block comment */\nafterAll(() => {});",
        "const thing = 123;\n/**\n * JSDoc comment\n */\nafterAll(() => {});",
        "describe('foo', () => {\nafterAll(() => {});\nafterAll(() => {});\n});",
        "import { afterAll } from '@jest/globals';\n/* setup notes */\nafterAll(() => {});",
        "const thing = 123;\n\nafterAll(() => {});\nconst otherThing = 123;",
        "const thing = 123;\nafterAll(() => {});\n\nconst otherThing = 123;",
    ];

    let fix = vec![
        ("const thing = 123;\nafterAll(() => {});", "const thing = 123;\n\nafterAll(() => {});"),
        (
            "const thing = 123;\n// This is a comment\nafterAll(() => {});",
            "const thing = 123;\n\n// This is a comment\nafterAll(() => {});",
        ),
        (
            "import { afterAll } from '@jest/globals';\nafterAll(() => {});",
            "import { afterAll } from '@jest/globals';\n\nafterAll(() => {});",
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
        (
            "const thing = 123;\n\nafterAll(() => {});\nconst otherThing = 123;",
            "const thing = 123;\n\nafterAll(() => {});\n\nconst otherThing = 123;",
        ),
        (
            "const thing = 123;\nafterAll(() => {});\n\nconst otherThing = 123;",
            "const thing = 123;\n\nafterAll(() => {});\n\nconst otherThing = 123;",
        ),
    ];

    Tester::new(PaddingAroundAfterAllBlocks::NAME, PaddingAroundAfterAllBlocks::PLUGIN, pass, fail)
        .with_jest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
