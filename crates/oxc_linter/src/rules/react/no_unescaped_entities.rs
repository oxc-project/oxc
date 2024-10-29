use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use phf::{phf_map, Map};

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
    AstNode,
};

fn no_unescaped_entities_diagnostic(span: Span, unescaped: char, escaped: &str) -> OxcDiagnostic {
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
    pedantic
);

impl Rule for NoUnescapedEntities {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::JSXText(jsx_text) = node.kind() {
            let source = jsx_text.span.source_text(ctx.source_text());
            for (i, char) in source.char_indices() {
                if !CHARS.contains(&char) {
                    continue;
                }
                if let Some(escapes) = DEFAULTS.get(&char) {
                    #[allow(clippy::cast_possible_truncation)]
                    ctx.diagnostic(no_unescaped_entities_diagnostic(
                        Span::new(
                            jsx_text.span.start + i as u32,
                            jsx_text.span.start + i as u32 + 1,
                        ),
                        char,
                        &escapes.join(" or "),
                    ));
                }
            }
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_jsx()
    }
}

// NOTE: If we add substantially more characters, we should consider using a hash set instead.
pub const CHARS: [char; 4] = ['>', '"', '\'', '}'];

pub const DEFAULTS: Map<char, &'static [&'static str]> = phf_map! {
    '>' => &["&gt;"],
    '"' => &["&quot;", "&ldquo;", "&#34;", "&rdquo;"],
    '\'' => &["&apos;", "&lsquo;", "&#39;", "&rsquo;"],
    '}' => &["&#125;"],
};

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

    Tester::new(NoUnescapedEntities::NAME, pass, fail).test_and_snapshot();
}
