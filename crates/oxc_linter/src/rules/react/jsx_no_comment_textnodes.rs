use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
    AstNode,
};

fn jsx_no_comment_textnodes_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Comments inside children section of tag should be placed inside braces")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct JsxNoCommentTextnodes;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule prevents comment strings (e.g. beginning with `//` or `/*`) from being accidentally injected as a text node in JSX statements.
    ///
    /// ### Why is this bad?
    ///
    /// In JSX, any text node that is not wrapped in curly braces is considered a literal string to be rendered. This can lead to unexpected behavior when the text contains a comment.
    ///
    /// ### Example
    /// ```jsx
    /// // Incorrect:
    ///
    /// const Hello = () => {
    ///     return <div>// empty div</div>;
    /// }
    ///
    /// const Hello = () => {
    ///     return <div>/* empty div */</div>;
    /// }
    ///
    /// // Correct:
    ///
    /// const Hello = () => {
    ///     return <div>// empty div</div>;
    /// }
    ///
    /// const Hello = () => {
    ///     return <div>{/* empty div */}</div>;
    /// }
    /// ```
    JsxNoCommentTextnodes,
    suspicious
);

impl Rule for JsxNoCommentTextnodes {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXText(jsx_text) = node.kind() else {
            return;
        };

        if has_comment_pattern(&jsx_text.value) {
            ctx.diagnostic(jsx_no_comment_textnodes_diagnostic(jsx_text.span));
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_jsx()
    }
}

/// Returns true if the given text contains a comment pattern such as `//` or `/*`.
fn has_comment_pattern(text: &str) -> bool {
    text.lines().any(|line| {
        let line = line.trim();
        line.starts_with("//") || line.starts_with("/*")
    })
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            "
			        class Comp1 extends Component {
			          render() {
			            return (
			              <div>
			                {/* valid */}
			              </div>
			            );
			          }
			        }
			      ",
            None,
        ),
        (
            "
			        class Comp1 extends Component {
			          render() {
			            return (
			              <>
			                {/* valid */}
			              </>
			            );
			          }
			        }
			      ",
            None,
        ),
        (
            "
			        class Comp1 extends Component {
			          render() {
			            return (<div>{/* valid */}</div>);
			          }
			        }
			      ",
            None,
        ),
        (
            "
			        class Comp1 extends Component {
			          render() {
			            const bar = (<div>{/* valid */}</div>);
			            return bar;
			          }
			        }
			      ",
            None,
        ),
        (
            "
			        var Hello = createReactClass({
			          foo: (<div>{/* valid */}</div>),
			          render() {
			            return this.foo;
			          },
			        });
			      ",
            None,
        ),
        (
            "
			        class Comp1 extends Component {
			          render() {
			            return (
			              <div>
			                {/* valid */}
			                {/* valid 2 */}
			                {/* valid 3 */}
			              </div>
			            );
			          }
			        }
			      ",
            None,
        ),
        (
            "
			        class Comp1 extends Component {
			          render() {
			            return (
			              <div>
			              </div>
			            );
			          }
			        }
			      ",
            None,
        ),
        (
            "
			        var foo = require('foo');
			      ",
            None,
        ),
        (
            "
			        <Foo bar='test'>
			          {/* valid */}
			        </Foo>
			      ",
            None,
        ),
        (
            "
			        <strong>
			          &nbsp;https://www.example.com/attachment/download/1
			        </strong>
			      ",
            None,
        ),
        (
            "
			        <Foo /* valid */ placeholder={'foo'}/>
			      ",
            None,
        ),
        (
            "
			        </* valid */></>
			      ",
            None,
        ),
        (
            "
			        <></* valid *//>
			      ",
            None,
        ),
        (
            "
			        <Foo title={'foo' /* valid */}/>
			      ",
            None,
        ),
        ("<pre>&#x2F;&#x2F; TODO: Write perfect code</pre>", None),
        ("<pre>&#x2F;&#42; TODO: Write perfect code &#42;&#x2F;</pre>", None),
        (
            "
			        <div>
			          <span className=\"pl-c\"><span className=\"pl-c\">&#47;&#47;</span> ...</span><br />
			        </div>
			      ",
            None,
        ),
    ];

    let fail = vec![
        (
            "
			        class Comp1 extends Component {
			          render() {
			            return (<div>// invalid</div>);
			          }
			        }
			      ",
            None,
        ),
        (
            "
			        class Comp1 extends Component {
			          render() {
			            return (<>// invalid</>);
			          }
			        }
			      ",
            None,
        ),
        (
            "
			        class Comp1 extends Component {
			          render() {
			            return (<div>/* invalid */</div>);
			          }
			        }
			      ",
            None,
        ),
        (
            "
			        class Comp1 extends Component {
			          render() {
			            return (
			              <div>
			                // invalid
			              </div>
			            );
			          }
			        }
			      ",
            None,
        ),
        (
            "
			        class Comp1 extends Component {
			          render() {
			            return (
			              <div>
			                asdjfl
			                /* invalid */
			                foo
			              </div>
			            );
			          }
			        }
			      ",
            None,
        ),
        (
            "
			        class Comp1 extends Component {
			          render() {
			            return (
			              <div>
			                {'asdjfl'}
			                // invalid
			                {'foo'}
			              </div>
			            );
			          }
			        }
			      ",
            None,
        ),
        (
            "
			        const Component2 = () => {
			          return <span>/*</span>;
			        };
			      ",
            None,
        ),
    ];

    Tester::new(JsxNoCommentTextnodes::NAME, pass, fail).test_and_snapshot();
}
