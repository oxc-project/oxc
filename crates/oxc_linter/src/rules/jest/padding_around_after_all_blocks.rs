use oxc_ast::AstKind;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNode;

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
    rules::shared::padding_around_after_all_blocks::{DOCUMENTATION, run},
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
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(_) = node.kind() else { return };
        run(node, ctx);
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.frameworks().is_test()
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "afterAll(() => {});",
        "const thing = 123;\n\nafterAll(() => {});",
        "describe('foo', () => {\nafterAll(() => {});\n});",
        "const thing = 123;\n\n/* one */\n/* two */\nafterAll(() => {});",
        "const cleanUp = false; cleanUp && afterAll(() => {});",
        "const cleanUp = false;\nif (cleanUp)\n  afterAll(() => {});",
        "const cleanUp = false; if (cleanUp) afterAll(() => {});",
        "const cleanUp = false; if (cleanUp)\n  afterAll(() => {});",
        "const cleanUp = false; while (cleanUp) afterAll(() => {});",
        "const cleanUp = false; for (; cleanUp;) afterAll(() => {});",
        "const thing = 123; (afterAll(() => {}));",
        "const thing = 123; await afterAll(() => {}) && foo();",
    ];

    let fail = vec![
        "const thing = 123;\nafterAll(() => {});",
        "const thing = 123;\n/* one */\n/* two */\nafterAll(() => {});",
    ];

    let fix = vec![
        (
            "const thing = 123; afterAll(() => {}), foo();",
            "const thing = 123;\n\nafterAll(() => {}), foo();",
        ),
        (
            "const thing = 123; afterAll(() => {}) && foo();",
            "const thing = 123;\n\nafterAll(() => {}) && foo();",
        ),
        (
            "const thing = 123; await afterAll(() => {});",
            "const thing = 123;\n\nawait afterAll(() => {});",
        ),
        (
            "const thing = 123;\nawait afterAll(() => {});",
            "const thing = 123;\n\nawait afterAll(() => {});",
        ),
        (
            "const thing = 123; await /* hook */ afterAll(() => {});",
            "const thing = 123;\n\nawait /* hook */ afterAll(() => {});",
        ),
        ("const thing = 123;\nafterAll(() => {});", "const thing = 123;\n\nafterAll(() => {});"),
        (
            "const thing = 123;\n/* one */\n/* two */\nafterAll(() => {});",
            "const thing = 123;\n\n/* one */\n/* two */\nafterAll(() => {});",
        ),
        (
            "const thing = 123; await (afterAll(() => {}));",
            "const thing = 123;\n\nawait (afterAll(() => {}));",
        ),
        (
            "const thing = 123; await (afterAll(() => {}) && foo());",
            "const thing = 123;\n\nawait (afterAll(() => {}) && foo());",
        ),
    ];

    Tester::new(PaddingAroundAfterAllBlocks::NAME, PaddingAroundAfterAllBlocks::PLUGIN, pass, fail)
        .with_jest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
