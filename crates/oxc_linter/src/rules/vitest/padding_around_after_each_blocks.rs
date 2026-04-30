use oxc_macros::declare_oxc_lint;

use crate::{
    context::LintContext,
    rule::Rule,
    rules::shared::padding_around_after_each_blocks::{DOCUMENTATION, run},
    utils::PossibleJestNode,
};

#[derive(Debug, Default, Clone)]
pub struct PaddingAroundAfterEachBlocks;

declare_oxc_lint!(
    PaddingAroundAfterEachBlocks,
    vitest,
    style,
    fix,
    docs = DOCUMENTATION,
    version = "next",
);

impl Rule for PaddingAroundAfterEachBlocks {
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
        "afterEach(() => {});",
        "const thing = 123;\n\nafterEach(() => {});",
        "describe('foo', () => {\nafterEach(() => {});\n});",
        "// This is a comment\nafterEach(() => {});",
        "import { afterEach } from 'vitest';\n\nafterEach(() => {});",
        "import { afterEach, describe } from 'vitest';\nimport { helper } from './helper';\n\nafterEach(() => {});",
        "import './setup';\n\nafterEach(() => {});",
        "/* leading block comment */\nafterEach(() => {});",
        "/**\n * JSDoc-style comment\n */\nafterEach(() => {});",
        "const thing = 123;\n\n/* attached to afterEach */\nafterEach(() => {});",
        "const thing = 123;\n\n/**\n * JSDoc attached to afterEach\n */\nafterEach(() => {});",
        "const thing = 123; /* trailing on prev */\n\nafterEach(() => {});",
        "describe('foo', () => {\nafterEach(() => {});\n\nafterEach(() => {});\n});",
        "const thing = 123;\n\n/* one */\n/* two */\nafterEach(() => {});",
        "const thing = 123;\n\nafterEach(() => {});\n\nconst otherThing = 123;",
    ];

    let fail = vec![
        "const thing = 123;\nafterEach(() => {});",
        "const thing = 123;\n//My comment\nafterEach(() => {});",
        "import { afterEach } from 'vitest';\nafterEach(() => {});",
        "import { afterEach } from 'vitest';\nimport { helper } from './helper';\nafterEach(() => {});",
        "import './setup';\nafterEach(() => {});",
        "const thing = 123;\n/* block comment */\nafterEach(() => {});",
        "const thing = 123;\n/**\n * JSDoc comment\n */\nafterEach(() => {});",
        "describe('foo', () => {\nafterEach(() => {});\nafterEach(() => {});\n});",
        "import { afterEach } from 'vitest';\n/* setup notes */\nafterEach(() => {});",
        "const thing = 123;\n\nafterEach(() => {});\nconst otherThing = 123;",
        "const thing = 123;\nafterEach(() => {});\n\nconst otherThing = 123;",
    ];

    let fix = vec![
        ("const thing = 123;\nafterEach(() => {});", "const thing = 123;\n\nafterEach(() => {});"),
        (
            "const thing = 123;\n// This is a comment\nafterEach(() => {});",
            "const thing = 123;\n\n// This is a comment\nafterEach(() => {});",
        ),
        (
            "import { afterEach } from 'vitest';\nafterEach(() => {});",
            "import { afterEach } from 'vitest';\n\nafterEach(() => {});",
        ),
        ("import './setup';\nafterEach(() => {});", "import './setup';\n\nafterEach(() => {});"),
        (
            "const thing = 123;\n/* block comment */\nafterEach(() => {});",
            "const thing = 123;\n\n/* block comment */\nafterEach(() => {});",
        ),
        (
            "const thing = 123;\n/**\n * JSDoc comment\n */\nafterEach(() => {});",
            "const thing = 123;\n\n/**\n * JSDoc comment\n */\nafterEach(() => {});",
        ),
        (
            "describe('foo', () => {\nafterEach(() => {});\nafterEach(() => {});\n});",
            "describe('foo', () => {\nafterEach(() => {});\n\nafterEach(() => {});\n});",
        ),
        (
            "const thing = 123;\n\nafterEach(() => {});\nconst otherThing = 123;",
            "const thing = 123;\n\nafterEach(() => {});\n\nconst otherThing = 123;",
        ),
        (
            "const thing = 123;\nafterEach(() => {});\n\nconst otherThing = 123;",
            "const thing = 123;\n\nafterEach(() => {});\n\nconst otherThing = 123;",
        ),
    ];

    Tester::new(
        PaddingAroundAfterEachBlocks::NAME,
        PaddingAroundAfterEachBlocks::PLUGIN,
        pass,
        fail,
    )
    .with_jest_plugin(true)
    .expect_fix(fix)
    .test_and_snapshot();
}
