use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    context::LintContext,
    fixer::RuleFixer,
    rule::Rule,
    utils::{
        JestFnKind, JestGeneralFnKind, KnownMemberExpressionProperty, PossibleJestNode,
        parse_general_jest_fn_call,
    },
};

fn hoisted_apis_on_top_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Hoisted API cannot be used in a runtime location in this file.")
        .with_help("Move this hoisted API to the top of the file to better reflect its behavior.\nIf possible, replace `vi.mock()` with `vi.doMock`, which is not hoisted.")
        .with_label(span)
}

const HOISTED_APIS: [&str; 3] = ["mock", "hoisted", "unmock"];

#[derive(Debug, Default, Clone)]
pub struct HoistedApisOnTop;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce hoisted APIs to be on top of the file.
    ///
    /// ### Why is this bad?
    ///
    /// Some Vitest APIs are hoisted automatically during the transform process. Using this APIs
    /// in look like runtime code can lead to unexpected results running tests.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// if (condition) {
    ///   vi.mock('some-module', () => {})
    /// }
    /// ```
    ///
    /// ```js
    /// if (condition) {
    ///   vi.unmock('some-module', () => {})
    /// }
    /// ```
    ///
    /// ```js
    /// if (condition) {
    ///   vi.hoisted(() => {})
    /// }
    /// ```
    ///
    /// ```js
    /// describe('suite', () => {
    ///   it('test', async () => {
    ///     vi.mock('some-module', () => {})
    ///
    ///     const sm = await import('some-module')
    ///
    ///   })
    /// })
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    ///
    /// ```js
    /// if (condition) {
    ///   vi.doMock('some-module', () => {})
    /// }
    /// ```
    ///
    /// ```js
    /// vi.mock('some-module', () => {})
    /// if (condition) {}
    /// ```
    ///
    /// ```js
    /// vi.unmock('some-module', () => {})
    /// if (condition) {}
    /// ```
    ///
    /// ```js
    /// vi.hoisted(() => {})
    /// if (condition) {}
    /// ```
    ///
    /// ```js
    /// vi.mock('some-module', () => {})
    ///
    /// describe('suite', () => {
    ///   it('test', async () => {
    ///     const sm = await import('some-module')
    ///   })
    /// })
    /// ```
    HoistedApisOnTop,
    vitest,
    correctness,
    suggestion,
);

impl Rule for HoistedApisOnTop {
    fn run_on_jest_node<'a, 'c>(
        &self,
        jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        Self::run(jest_node, ctx);
    }
}

impl HoistedApisOnTop {
    fn run<'a>(possible_jest_node: &PossibleJestNode<'a, '_>, ctx: &LintContext<'a>) {
        let node = possible_jest_node.node;

        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        let Some(vitest_fn) = parse_general_jest_fn_call(call_expr, possible_jest_node, ctx) else {
            return;
        };

        if vitest_fn.kind != JestFnKind::General(JestGeneralFnKind::Vitest) {
            return;
        }

        if !vitest_fn.members.iter().any(is_hoisted_api) {
            return;
        }

        let Some(member_name) =
            vitest_fn.members.first().and_then(KnownMemberExpressionProperty::name)
        else {
            return;
        };

        if member_name.as_ref() == "hoisted" {
            let parent_node = {
                let mut tmp_parent_node = ctx.nodes().parent_node(node.id());

                if matches!(tmp_parent_node.kind(), AstKind::AwaitExpression(_)) {
                    tmp_parent_node = ctx.nodes().parent_node(tmp_parent_node.id());
                }

                if matches!(tmp_parent_node.kind(), AstKind::VariableDeclarator(_)) {
                    tmp_parent_node = ctx.nodes().parent_node(tmp_parent_node.id());
                }

                tmp_parent_node
            };

            let grandparent_node = ctx.nodes().parent_node(parent_node.id());

            if matches!(
                parent_node.kind(),
                AstKind::ExpressionStatement(_) | AstKind::VariableDeclaration(_)
            ) && matches!(grandparent_node.kind(), AstKind::Program(_))
            {
                return;
            }
        } else {
            let parent_node = ctx.nodes().parent_node(node.id());
            let grandparent_node = ctx.nodes().parent_node(parent_node.id());
            if matches!(parent_node.kind(), AstKind::ExpressionStatement(_))
                && matches!(grandparent_node.kind(), AstKind::Program(_))
            {
                return;
            }
        }

        let fixer = RuleFixer::new(FixKind::Suggestion, ctx);

        let suggestion_move_node = {
            let multi_fixer = fixer.for_multifix();
            let mut rule_fixes = multi_fixer.new_fix_with_capacity(2);

            if matches!(ctx.nodes().parent_node(node.id()).kind(), AstKind::ExpressionStatement(_))
            {
                rule_fixes.push(fixer.delete(node));
            } else {
                rule_fixes.push(fixer.replace(GetSpan::span(node), "undefined"));
            }

            if let Some(last_import) = ctx.module_record().import_entries.last() {
                let new_code = format!("\n{};\n", ctx.source_range(GetSpan::span(node)));

                rule_fixes.push(
                    fixer.insert_text_after(&Span::empty(last_import.statement_span.end), new_code),
                );
            } else {
                let new_code = format!("{};\n", ctx.source_range(GetSpan::span(node)));

                rule_fixes.push(fixer.insert_text_after(&Span::empty(0), new_code));
            }

            rule_fixes.with_message("Moving hoisted methods to the top of the file")
        };

        let suggestion_do_mock = {
            if member_name == "mock" {
                let mock_member = vitest_fn.members.first().unwrap();
                fixer.replace(mock_member.span, "doMock")
            } else {
                fixer.noop()
            }
        };

        ctx.diagnostic_with_suggestions(
            hoisted_apis_on_top_diagnostic(call_expr.span),
            [suggestion_move_node, suggestion_do_mock],
        );
    }
}

fn is_hoisted_api(member: &KnownMemberExpressionProperty) -> bool {
    let Some(name) = member.name() else {
        return false;
    };

    HOISTED_APIS.contains(&name.as_ref())
}

#[test]
fn test() {
    use crate::tester::Tester;
    let pass = vec![
        "vi.mock()",
        "
			vi.hoisted();
			import foo from 'bar';
			    ",
        "
			import foo from 'bar';
			vi.unmock(baz);
			    ",
        "import 'vi';\nconst foo = await vi.hoisted(async () => {});",
    ];

    let fail = vec![
        "
			if (foo) {
			  vi.mock('foo', () => {});
			}
			      ",
        "
			import foo from 'bar';

			if (foo) {
			  vi.hoisted();
			}
			    ",
        "
			import foo from 'bar';

			if (foo) {
			  vi.unmock();
			}
			    ",
        "
			import foo from 'bar';

			if (foo) {
			  vi.mock();
			}
			    ",
        "
			if (shouldMock) {
			  vi.mock(import('something'), () => bar);
			}

			import something from 'something';
			      ",
    ];

    let fix = vec![
        (
            "
			if (foo) {
			  vi.mock('foo', () => {});
			}
			      ",
            (
                "vi.mock('foo', () => {});\n
			if (foo) {
			  ;
			}
			      ",
                "
			if (foo) {
			  vi.doMock('foo', () => {});
			}
			      ",
            ),
        ),
        (
            "
			import foo from 'bar';

			if (foo) {
			  vi.hoisted();
			}
			    ",
            (
                "
			import foo from 'bar';
vi.hoisted();


			if (foo) {
			  ;
			}
			    ",
                "
			import foo from 'bar';

			if (foo) {
			  vi.hoisted();
			}
			    ",
            ),
        ),
        (
            "
			import foo from 'bar';

			if (foo) {
			  vi.unmock();
			}
			    ",
            (
                "
			import foo from 'bar';
vi.unmock();


			if (foo) {
			  ;
			}
			    ",
                "
			import foo from 'bar';

			if (foo) {
			  vi.unmock();
			}
			    ",
            ),
        ),
        (
            "
			import foo from 'bar';

			if (foo) {
			  vi.mock();
			}
			    ",
            (
                "
			import foo from 'bar';
vi.mock();


			if (foo) {
			  ;
			}
			    ",
                "
			import foo from 'bar';

			if (foo) {
			  vi.doMock();
			}
			    ",
            ),
        ),
        (
            "
			if (shouldMock) {
			  vi.mock(import('something'), () => bar);
			}

			import something from 'something';
			      ",
            (
                "
			if (shouldMock) {
			  ;
			}

			import something from 'something';
vi.mock(import('something'), () => bar);

			      ",
                "
			if (shouldMock) {
			  vi.doMock(import('something'), () => bar);
			}

			import something from 'something';
			      ",
            ),
        ),
    ];

    Tester::new(HoistedApisOnTop::NAME, HoistedApisOnTop::PLUGIN, pass, fail)
        .change_rule_path_extension("mjs")
        .expect_fix(fix)
        .with_vitest_plugin(true)
        .test_and_snapshot();
}
