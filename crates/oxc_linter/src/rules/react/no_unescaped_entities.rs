use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    rule::Rule,
};

static ESCAPED_DOUBLE_QUOTE: &str = "&quot; or &ldquo; or &#34; or &rdquo;";
static ESCAPED_SINGLE_QUOTE: &str = "&apos; or &lsquo; or &#39; or &rsquo;";

fn no_unescaped_entities_diagnostic(span: Span, unescaped: char) -> OxcDiagnostic {
    let escaped = if unescaped == '"' { ESCAPED_DOUBLE_QUOTE } else { ESCAPED_SINGLE_QUOTE };
    OxcDiagnostic::warn(format!("`{unescaped}` can be escaped with {escaped}")).with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoUnescapedEntities;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule prevents characters that you may have meant as JSX escape characters from being accidentally injected as a text node in JSX statements.
    ///
    /// ### Why is this bad?
    ///
    /// JSX escape characters are used to inject characters into JSX statements that would otherwise be interpreted as code.
    ///
    /// ### Example
    /// Incorrect
    ///
    /// ```jsx
    /// <div> > </div>
    /// ```
    ///
    /// Correct
    ///
    /// ```jsx
    /// <div> &gt; </div>
    /// ```
    ///
    /// ```jsx
    /// <div> {'>'} </div>
    /// ```
    NoUnescapedEntities,
    react,
    pedantic
);

impl Rule for NoUnescapedEntities {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::JSXText(jsx_text) = node.kind() {
            let source = jsx_text.raw.unwrap().as_str();
            for (i, &byte) in source.as_bytes().iter().enumerate() {
                if matches!(byte, b'\'' | b'\"') {
                    #[expect(clippy::cast_possible_truncation)]
                    let start = jsx_text.span.start + i as u32;
                    ctx.diagnostic(no_unescaped_entities_diagnostic(
                        Span::sized(start, 1),
                        byte as char,
                    ));
                }
            }
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_jsx()
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "
        var Hello = createReactClass({
          render: function() {
            return (
              <div/>
            );
          }
        });
        ",
        "
        var Hello = createReactClass({
          render: function() {
            return <div>Here is some text!</div>;
          }
        });
        ",
        "
        var Hello = createReactClass({
          render: function() {
            return <div>I&rsquo;ve escaped some entities: &gt; &lt; &amp;</div>;
          }
        });
        ",
        "
        var Hello = createReactClass({
          render: function() {
            return <div>first line is ok
            so is second
            and here are some escaped entities: &gt; &lt; &amp;</div>;
          }
        });
        ",
        "
        var Hello = createReactClass({
          render: function() {
            return <div>{\">\" + \"<\" + \"&\" + '\"'}</div>;
          },
        });
        ",
        "
        var Hello = createReactClass({
          render: function() {
            return <>Here is some text!</>;
          }
        });
        ",
        "
        var Hello = createReactClass({
          render: function() {
            return <>I&rsquo;ve escaped some entities: &gt; &lt; &amp;</>;
          }
        });
        ",
        "
        var Hello = createReactClass({
          render: function() {
            return <>{\">\" + \"<\" + \"&\" + '\"'}</>;
          },
        });
        ",
    ];

    let fail = vec![
        "var Hello = createReactClass({
            render: function() {
              return <>> babel-eslint</>;
            }
          });",
        "var Hello = createReactClass({
            render: function() {
              return <>first line is ok
              so is second
              and here are some bad entities: ></>
            }
          });",
        "
        var Hello = createReactClass({
            render: function() {
              return <div>'</div>;
            }
        });
        ",
        r#"
        var Hello = createReactClass({
            render: function() {
              return <>{"Unbalanced braces - babel-eslint"}}</>;
            }
          });
        "#,
        // "var Hello = createReactClass({
        //     render: function() {
        //       return <>foo & bar</>;
        //     }
        //   });",
        // "        var Hello = createReactClass({
        //     render: function() {
        //       return <span>foo & bar</span>;
        //     }
        //   });
        // ",
        r#"<script>window.foo = "bar"</script>"#,
        r#"<script>测试 " 测试</script>"#,
    ];

    Tester::new(NoUnescapedEntities::NAME, NoUnescapedEntities::PLUGIN, pass, fail)
        .test_and_snapshot();
}
