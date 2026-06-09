use oxc_ast::{
    AstKind,
    ast::{JSXAttributeItem, JSXAttributeValue, JSXChild, JSXElement, JSXExpression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_str::CompactStr;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

fn jsx_no_literals_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Disallow usage of unwrapped string literals in JSX")
        .with_help("Wrap this string in a JSX expression container, such as a call to a translation function.")
        .with_label(span)
}

#[derive(Debug, Default, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct JsxNoLiterals {
    /// (default: false) - Enforces no string literals used as children, wrapped or unwrapped.
    no_strings: bool,
    /// An array of unique string values that would otherwise warn, but will be ignored.
    allowed_strings: Vec<CompactStr>,
    /// (default: false) - When true the rule ignores literals used in props, wrapped or unwrapped.
    ignore_props: bool,
    /// (default: false) - Enforces no string literals used in attributes when set to true.
    no_attribute_strings: bool,
    /// An array of unique attribute names where string literals should be restricted. Only the specified attributes will be checked for string literals when this option is used. Note: When noAttributeStrings is true, this option is ignored at the root level.
    restricted_attributes: Vec<CompactStr>,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows usage of unwrapped string literals inside JSX, such as text
    /// children of a JSX element or string-valued props.
    ///
    /// ### Why is this bad?
    ///
    /// Hard-coded string literals in JSX make it difficult to support
    /// internationalization (i18n). By requiring literals to be wrapped in a
    /// JSX expression container (for example, a call to a translation
    /// function), this rule helps ensure all user-facing text flows through a
    /// single, auditable mechanism rather than being scattered as inline
    /// strings throughout the markup.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <div>Hello world</div>;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <div>{'Hello world'}</div>;
    /// ```
    JsxNoLiterals,
    react,
    restriction,
    none,
    config = JsxNoLiterals,
    version = "next",
);

impl Rule for JsxNoLiterals {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXElement(jsx_el) = node.kind() else {
            return;
        };

        self.check_in_attributes(jsx_el, ctx);
        self.check_in_children(jsx_el, ctx);
    }
}

impl JsxNoLiterals {
    fn is_allowed_string(&self, str_literal: &str) -> bool {
        self.allowed_strings.iter().any(|allowed| allowed.as_str().trim() == str_literal.trim())
    }

    fn check_in_attributes(&self, jsx_el: &JSXElement, ctx: &LintContext) {
        if self.ignore_props {
            return;
        }

        for attr in &jsx_el.opening_element.attributes {
            let JSXAttributeItem::Attribute(attr) = attr else {
                continue;
            };

            let Some(value) = &attr.value else {
                continue;
            };

            let JSXAttributeValue::StringLiteral(str_literal) = value else {
                continue;
            };

            if self.no_attribute_strings {
                ctx.diagnostic(jsx_no_literals_diagnostic(attr.span));
                continue;
            }

            if self.is_allowed_string(&str_literal.value.as_str()) {
                continue;
            }

            if self.no_strings != true {
                continue;
            }

            ctx.diagnostic(jsx_no_literals_diagnostic(attr.span));
        }
    }

    fn check_in_children(&self, jsx_el: &JSXElement, ctx: &LintContext) {
        for child in &jsx_el.children {
            match child {
                JSXChild::Element(element) => {
                    self.check_in_attributes(element, ctx);
                }
                JSXChild::Text(text) => {
                    let value = text.value.as_str();

                    if self.is_allowed_string(value) {
                        continue;
                    }

                    if !value.trim().is_empty() {
                        ctx.diagnostic(jsx_no_literals_diagnostic(text.span));
                    }
                }
                JSXChild::ExpressionContainer(container) => {
                    if self.no_strings {
                        match &container.expression {
                            JSXExpression::StringLiteral(literal) => {
                                ctx.diagnostic(jsx_no_literals_diagnostic(literal.span))
                            }
                            JSXExpression::TemplateLiteral(literal) => {
                                ctx.diagnostic(jsx_no_literals_diagnostic(literal.span))
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            r#"
                    class Comp1 extends Component {
                      render() {
                        return (
                          <div>
                            <button type="button"></button>
                          </div>
                        );
                      }
                    }
                  "#,
            Some(
                serde_json::json!([ { "noStrings": true, "allowedStrings": ["button", "submit"], }, ]),
            ),
        ),
        (
            "
                    class Comp2 extends Component {
                      render() {
                        return (
                          <div>
                            {'asdjfl'}
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
                            {'asdjfl'}
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
                        return (<div>{'test'}</div>);
                      }
                    }
                  ",
            None,
        ),
        (
            "
                    class Comp1 extends Component {
                      render() {
                        const bar = (<div>{'hello'}</div>);
                        return bar;
                      }
                    }
                  ",
            None,
        ),
        (
            "
                    var Hello = createReactClass({
                      foo: (<div>{'hello'}</div>),
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
                            {'asdjfl'}
                            {'test'}
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
                      {'blarg'}
                    </Foo>
                  ",
            None,
        ),
        (
            r#"
                    <Foo bar="test">
                      {intl.formatText(message)}
                    </Foo>
                  "#,
            Some(serde_json::json!([{ "noStrings": true, "ignoreProps": true }])),
        ),
        (
            r#"
                    <Foo bar="test">
                      {translate('my.translate.key')}
                    </Foo>
                  "#,
            Some(serde_json::json!([{ "noStrings": true, "ignoreProps": true }])),
        ),
        ("<Foo bar={true} />", Some(serde_json::json!([{ "noStrings": true }]))),
        ("<Foo bar={false} />", Some(serde_json::json!([{ "noStrings": true }]))),
        ("<Foo bar={100} />", Some(serde_json::json!([{ "noStrings": true }]))),
        ("<Foo bar={null} />", Some(serde_json::json!([{ "noStrings": true }]))),
        ("<Foo bar={{}} />", Some(serde_json::json!([{ "noStrings": true }]))),
        (
            "
                    class Comp1 extends Component {
                      asdf() {}
                      render() {
                        return <Foo bar={this.asdf} class='xx' />;
                      }
                    }
                  ",
            Some(serde_json::json!([{ "noStrings": true, "ignoreProps": true }])),
        ),
        (
            "
                    class Comp1 extends Component {
                      render() {
                        let foo = `bar`;
                        return <div />;
                      }
                    }
                  ",
            Some(serde_json::json!([{ "noStrings": true }])),
        ),
        (
            "
                    class Comp1 extends Component {
                      render() {
                        return <div>asdf</div>
                      }
                    }
                  ",
            Some(serde_json::json!([{ "allowedStrings": ["asdf"] }])),
        ),
        (
            "
                    class Comp1 extends Component {
                      render() {
                        return <div>asdf</div>
                      }
                    }
                  ",
            Some(serde_json::json!([{ "noStrings": false, "allowedStrings": ["asdf"] }])),
        ),
        (
            "
                    class Comp1 extends Component {
                      render() {
                        return <div>&nbsp;</div>
                      }
                    }
                  ",
            Some(serde_json::json!([{ "noStrings": true, "allowedStrings": ["&nbsp;"] }])),
        ),
        (
            "
                    class Comp1 extends Component {
                      render() {
                        return (
                          <div>
                            &nbsp;
                          </div>
                        );
                      }
                    }
                  ",
            Some(serde_json::json!([{ "noStrings": true, "allowedStrings": ["&nbsp;"] }])),
        ),
        (
            "
                    class Comp1 extends Component {
                      render() {
                        return <div>foo: {bar}*</div>
                      }
                    }
                  ",
            Some(serde_json::json!([{ "noStrings": true, "allowedStrings": ["foo: ", "*"] }])),
        ),
        (
            "
                    class Comp1 extends Component {
                      render() {
                        return <div>foo</div>
                      }
                    }
                  ",
            Some(serde_json::json!([{ "noStrings": true, "allowedStrings": [" foo "] }])),
        ),
        (
            "
                    class Comp1 extends Component {
                      asdf() {}
                      render() {
                        const xx = 'xx';

                        return <Foo bar={this.asdf} class={xx} />;
                      }
                    }
                  ",
            Some(serde_json::json!([{ "noStrings": true, "ignoreProps": false }])),
        ),
        (
            "
                    <img alt='blank image'></img>
                  ",
            None,
        ),
        (
            "
                    <div>&mdash;</div>
                  ",
            Some(serde_json::json!([{ "noStrings": true, "allowedStrings": ["&mdash;", "—"] }])),
        ),
        (
            "
                    <div>—</div>
                  ",
            Some(serde_json::json!([{ "noStrings": true, "allowedStrings": ["&mdash;", "—"] }])),
        ),
        (
            r#"
                    <img src="image.jpg" alt="text" />
                  "#,
            Some(serde_json::json!([{ "restrictedAttributes": ["className", "id"] }])),
        ),
        (
            r#"
                    <div className="allowed" />
                  "#,
            Some(
                serde_json::json!([{ "restrictedAttributes": ["className"], "allowedStrings": ["allowed"] }]),
            ),
        ),
        (
            r#"
                    <div className="test" title="hello" />
                  "#,
            Some(
                serde_json::json!([{ "noStrings": true, "ignoreProps": true, "restrictedAttributes": ["className"], "allowedStrings": ["test"], }]),
            ),
        ),
        (
            r#"
                    <div className="test" id="foo" />
                  "#,
            Some(serde_json::json!([{ "restrictedAttributes": [] }])),
        ),
        (
            "
                    <T>foo</T>
                  ",
            Some(serde_json::json!([{ "elementOverrides": { "T": { "allowElement": true } } }])),
        ),
        (
            "
                    <T>foo <div>bar</div></T>
                  ",
            Some(serde_json::json!([{ "elementOverrides": { "T": { "allowElement": true } } }])),
        ),
        (
            "
                    <T>foo <div>{'bar'}</div></T>
                  ",
            Some(
                serde_json::json!([{ "elementOverrides": { "T": { "allowElement": true, "applyToNestedElements": false } } }]),
            ),
        ),
        (
            "
                    <div>
                      <div>{'foo'}</div>
                      <T>{2}</T>
                    </div>
                  ",
            Some(serde_json::json!([{ "elementOverrides": { "T": { "noStrings": true } } }])),
        ),
        (
            "
                    <T>{2}<div>{2}</div></T>
                  ",
            Some(serde_json::json!([{ "elementOverrides": { "T": { "noStrings": true } } }])),
        ),
        (
            "
                    <T>{2}<div>{'foo'}</div></T>
                  ",
            Some(
                serde_json::json!([{ "elementOverrides": { "T": { "noStrings": true, "applyToNestedElements": false } } }]),
            ),
        ),
        (
            "
                    <div>
                      <div>{'foo'}</div>
                      <T>foo</T>
                    </div>
                  ",
            Some(
                serde_json::json!([{ "elementOverrides": { "T": { "allowedStrings": ["foo"] } } }]),
            ),
        ),
        (
            "
                    <T>foo<div>foo</div></T>
                  ",
            Some(
                serde_json::json!([{ "elementOverrides": { "T": { "allowedStrings": ["foo"] } } }]),
            ),
        ),
        (
            "
                    <T>foo<div>{'foo'}</div></T>
                  ",
            Some(
                serde_json::json!([{ "elementOverrides": { "T": { "allowedStrings": ["foo"], "applyToNestedElements": false } } }]),
            ),
        ),
        (
            r#"
                    <div>
                      <div foo={2} />
                      <T foo="bar" />
                    </div>
                  "#,
            Some(
                serde_json::json!([{ "noStrings": true, "elementOverrides": { "T": { "noStrings": true, "ignoreProps": true } } }]),
            ),
        ),
        (
            r#"
                    <T foo="bar"><div foo="bar" /></T>
                  "#,
            Some(
                serde_json::json!([{ "noStrings": true, "elementOverrides": { "T": { "noStrings": true, "ignoreProps": true } } }]),
            ),
        ),
        (
            r#"
                    <T foo="bar"><div foo={2} /></T>
                  "#,
            Some(
                serde_json::json!([{ "noStrings": true, "elementOverrides": { "T": { "noStrings": true, "ignoreProps": true, "applyToNestedElements": false } } }]),
            ),
        ),
        (
            r#"
                    <div>
                      <div foo="foo" />
                      <T foo={2} />
                    </div>
                  "#,
            Some(
                serde_json::json!([{ "elementOverrides": { "T": { "noAttributeStrings": true } } }]),
            ),
        ),
        (
            "
                    <T foo={2}><div foo={2} /></T>
                  ",
            Some(
                serde_json::json!([{ "elementOverrides": { "T": { "noAttributeStrings": true } } }]),
            ),
        ),
        (
            r#"
                    <T foo={2}><div foo="foo" /></T>
                  "#,
            Some(
                serde_json::json!([{ "elementOverrides": { "T": { "noAttributeStrings": true, "applyToNestedElements": false } } }]),
            ),
        ),
        (
            "
                    <T>foo<U>foo</U></T>
                  ",
            Some(
                serde_json::json!([{ "elementOverrides": { "T": { "allowedStrings": ["foo"] }, "U": { "allowedStrings": ["foo"] } } }]),
            ),
        ),
        (
            "
                    import { T } from 'foo';
                    <T>{'foo'}</T>
                  ",
            None,
        ),
        (
            "
                    import { T as U } from 'foo';
                    <U>foo</U>
                  ",
            Some(serde_json::json!([{ "elementOverrides": { "T": { "allowElement": true } } }])),
        ),
        (
            "
                    const { T: U } = require('foo');
                    <U>foo</U>
                  ",
            Some(serde_json::json!([{ "elementOverrides": { "T": { "allowElement": true } } }])),
        ),
        (
            "
                    const { T: U } = require('foo').Foo;
                    <U>foo</U>
                  ",
            Some(serde_json::json!([{ "elementOverrides": { "T": { "allowElement": true } } }])),
        ),
        (
            "
                    const { T: U } = require('foo').Foo.Foo;
                    <U>foo</U>
                  ",
            Some(serde_json::json!([{ "elementOverrides": { "T": { "allowElement": true } } }])),
        ),
        (
            "
                    const foo = 2;
                    <T>foo</T>
                  ",
            Some(serde_json::json!([{ "elementOverrides": { "T": { "allowElement": true } } }])),
        ),
        (
            "
                    <T.U>foo</T.U>
                  ",
            Some(serde_json::json!([{ "elementOverrides": { "T.U": { "allowElement": true } } }])),
        ),
        (
            "
                    import { T as U } from 'foo';
                    <U.U>foo</U.U>
                  ",
            Some(serde_json::json!([{ "elementOverrides": { "T.U": { "allowElement": true } } }])),
        ),
        (
            "
                    <React.Fragment>foo</React.Fragment>
                  ",
            Some(
                serde_json::json!([{ "elementOverrides": { "Fragment": { "allowElement": true } } }]),
            ),
        ),
        (
            "
                    <React.Fragment>foo</React.Fragment>
                  ",
            Some(
                serde_json::json!([{ "elementOverrides": { "React.Fragment": { "allowElement": true } } }]),
            ),
        ),
        (
            "
                    <div>{'foo'}</div>
                  ",
            Some(serde_json::json!([{ "elementOverrides": { "div": { "allowElement": true } } }])),
        ),
        (
            r#"
                    <div>
                      <Input type="text" />
                      <Button className="primary" />
                      <Image src="photo.jpg" />
                    </div>
                  "#,
            Some(
                serde_json::json!([{ "elementOverrides": { "Input": { "restrictedAttributes": ["placeholder"] }, "Button": { "restrictedAttributes": ["type"] }, }, }]),
            ),
        ),
        (
            r#"
                    <div title="container">
                      <Button className="btn" />
                    </div>
                  "#,
            Some(
                serde_json::json!([{ "restrictedAttributes": ["className"], "elementOverrides": { "Button": { "restrictedAttributes": ["disabled"] }, }, }]),
            ),
        ),
        (
            r#"
                    <Button className="btn" />
                  "#,
            Some(
                serde_json::json!([{ "noAttributeStrings": true, "elementOverrides": { "Button": { "restrictedAttributes": ["type"] }, }, }]),
            ),
        ),
    ];

    let fail = vec![
        (
            "
                    class Comp1 extends Component {
                      render() {
                        return (<div>test</div>);
                      }
                    }
                  ",
            None,
        ),
        (
            "
                    class Comp1 extends Component {
                      render() {
                        return (<>test</>);
                      }
                    }
                  ",
            None,
        ),
        (
            "
                    class Comp1 extends Component {
                      render() {
                        const foo = (<div>test</div>);
                        return foo;
                      }
                    }
                  ",
            None,
        ),
        (
            "
                    class Comp1 extends Component {
                      render() {
                        const varObjectTest = { testKey : (<div>test</div>) };
                        return varObjectTest.testKey;
                      }
                    }
                  ",
            None,
        ),
        (
            "
                    var Hello = createReactClass({
                      foo: (<div>hello</div>),
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
                            asdjfl
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
                            test
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
                            test
                            {'foo'}
                          </div>
                        );
                      }
                    }
                  ",
            None,
        ),
        (
            r#"
                    <Foo bar="test">
                      {'Test'}
                    </Foo>
                  "#,
            Some(serde_json::json!([{ "noStrings": true, "ignoreProps": false }])),
        ),
        (
            r#"
                    <Foo bar="test">
                      {'Test' + name}
                    </Foo>
                  "#,
            Some(serde_json::json!([{ "noStrings": true, "ignoreProps": false }])),
        ),
        (
            r#"
                    <Foo bar="test">
                      Test
                    </Foo>
                  "#,
            Some(serde_json::json!([{ "noStrings": true, "ignoreProps": false }])),
        ),
        (
            "
                    <Foo>
                      {`Test`}
                    </Foo>
                  ",
            Some(serde_json::json!([{ "noStrings": true }])),
        ),
        (
            "<Foo bar={`Test`} />",
            Some(serde_json::json!([{ "noStrings": true, "ignoreProps": false }])),
        ),
        (
            "<Foo bar={`${baz}`} />",
            Some(serde_json::json!([{ "noStrings": true, "ignoreProps": false }])),
        ),
        (
            "<Foo bar={`Test ${baz}`} />",
            Some(serde_json::json!([{ "noStrings": true, "ignoreProps": false }])),
        ),
        (
            "<Foo bar={`foo` + 'bar'} />",
            Some(serde_json::json!([{ "noStrings": true, "ignoreProps": false }])),
        ),
        (
            "<Foo bar={`foo` + `bar`} />",
            Some(serde_json::json!([{ "noStrings": true, "ignoreProps": false }])),
        ),
        (
            "<Foo bar={'foo' + `bar`} />",
            Some(serde_json::json!([{ "noStrings": true, "ignoreProps": false }])),
        ),
        (
            "
                    class Comp1 extends Component {
                      render() {
                        return <div bar={'foo'}>asdf</div>
                      }
                    }
                  ",
            Some(
                serde_json::json!([{ "noStrings": true, "allowedStrings": ["asd"], "ignoreProps": false }]),
            ),
        ),
        (
            "<Foo bar={'bar'} />",
            Some(serde_json::json!([{ "noStrings": true, "ignoreProps": false }])),
        ),
        (
            "
                    <img alt='blank image'></img>
                  ",
            Some(serde_json::json!([{ "noAttributeStrings": true }])),
        ),
        (
            "export const WithChildren = ({}) => <div>baz bob</div>;",
            Some(serde_json::json!([{ "noAttributeStrings": true }])),
        ),
        (
            r#"export const WithAttributes = ({}) => <div title="foo bar" />;"#,
            Some(serde_json::json!([{ "noAttributeStrings": true }])),
        ),
        (
            r#"
                    export const WithAttributesAndChildren = ({}) => (
                      <div title="foo bar">baz bob</div>
                    );
                  "#,
            Some(serde_json::json!([{ "noAttributeStrings": true }])),
        ),
        (
            r#"
                    <div className="test" />
                  "#,
            Some(serde_json::json!([{ "restrictedAttributes": ["className"] }])),
        ),
        (
            r#"
                    <div className="test" id="foo" title="bar" />
                  "#,
            Some(serde_json::json!([{ "restrictedAttributes": ["className", "id"] }])),
        ),
        (
            r#"
                    <div src="image.jpg" />
                  "#,
            Some(
                serde_json::json!([{ "noAttributeStrings": true, "restrictedAttributes": ["className"], }]),
            ),
        ),
        (
            r#"
                    <div title="text">test</div>
                  "#,
            Some(serde_json::json!([{ "restrictedAttributes": ["title"], "noStrings": true, }])),
        ),
        (
            r#"
                    <div className="test" title="hello" />
                  "#,
            Some(
                serde_json::json!([{ "noStrings": true, "ignoreProps": false, "restrictedAttributes": ["className"] }]),
            ),
        ),
        (
            r#"
                    <div className="test" title="hello" />
                  "#,
            Some(
                serde_json::json!([{ "noStrings": true, "ignoreProps": true, "restrictedAttributes": ["className"] }]),
            ),
        ),
        (
            "
                    <div>
                      <div>foo</div>
                      <T>bar</T>
                    </div>
                  ",
            Some(serde_json::json!([{ "elementOverrides": { "T": {} } }])),
        ),
        (
            "
                    <div>
                      <div>foo</div>
                      <T>bar</T>
                    </div>
                  ",
            Some(serde_json::json!([{ "elementOverrides": { "T": { "allowElement": true } } }])),
        ),
        (
            "
                    <T>foo <div>bar</div></T>
                  ",
            Some(
                serde_json::json!([{ "elementOverrides": { "T": { "allowElement": true, "applyToNestedElements": false } } }]),
            ),
        ),
        (
            "
                    <div>
                      <div>foo</div>
                      <T>{'bar'}</T>
                    </div>
                  ",
            Some(serde_json::json!([{ "elementOverrides": { "T": { "noStrings": true } } }])),
        ),
        (
            "
                    <div>
                      <div>foo</div>
                      <T>{'bar'}<div>{'baz'}</div></T>
                    </div>
                  ",
            Some(serde_json::json!([{ "elementOverrides": { "T": { "noStrings": true } } }])),
        ),
        (
            "
                    <div>
                      <div>foo</div>
                      <T>{'bar'}<div>{'baz'}</div></T>
                    </div>
                  ",
            Some(
                serde_json::json!([{ "elementOverrides": { "T": { "noStrings": true, "applyToNestedElements": false } } }]),
            ),
        ),
        (
            "
                    <div>
                      <div>{'foo'}</div>
                      <T>{'foo'}</T>
                    </div>
                  ",
            Some(
                serde_json::json!([{ "noStrings": true, "elementOverrides": { "T": { "noStrings": true, "allowedStrings": ["foo"] } } }]),
            ),
        ),
        (
            "
                    <div>
                      <div>{'foo'}</div>
                      <T>{'foo'}<div>{'foo'}</div></T>
                    </div>
                  ",
            Some(
                serde_json::json!([{ "noStrings": true, "elementOverrides": { "T": { "noStrings": true, "allowedStrings": ["foo"] } } }]),
            ),
        ),
        (
            "
                    <div>
                      <div>{'foo'}</div>
                      <T>{'foo'}<div>{'foo'}</div></T>
                    </div>
                  ",
            Some(
                serde_json::json!([{ "noStrings": true, "elementOverrides": { "T": { "noStrings": true, "allowedStrings": ["foo"], "applyToNestedElements": false } } }]),
            ),
        ),
        (
            r#"
                    <div>
                      <div foo1="bar" />
                      <T foo2="bar" />
                    </div>
                  "#,
            Some(
                serde_json::json!([{ "noStrings": true, "elementOverrides": { "T": { "noStrings": true, "ignoreProps": true } } }]),
            ),
        ),
        (
            r#"
                    <div>
                      <div foo1="bar" />
                      <T foo2="bar"><div foo3="bar" /></T>
                    </div>
                  "#,
            Some(
                serde_json::json!([{ "noStrings": true, "elementOverrides": { "T": { "noStrings": true, "ignoreProps": true } } }]),
            ),
        ),
        (
            r#"
                    <div>
                      <div foo1="bar" />
                      <T foo2="bar"><div foo3="bar" /></T>
                    </div>
                  "#,
            Some(
                serde_json::json!([{ "noStrings": true, "elementOverrides": { "T": { "noStrings": true, "ignoreProps": true, "applyToNestedElements": false } } }]),
            ),
        ),
        (
            r#"
                    <div>
                      <div foo1="bar1" />
                      <T foo2="bar2" />
                    </div>
                  "#,
            Some(
                serde_json::json!([{ "elementOverrides": { "T": { "noAttributeStrings": true } } }]),
            ),
        ),
        (
            r#"
                    <div>
                      <div foo1="bar1" />
                      <T foo2="bar2"><div foo3="bar3" /></T>
                    </div>
                  "#,
            Some(
                serde_json::json!([{ "elementOverrides": { "T": { "noAttributeStrings": true } } }]),
            ),
        ),
        (
            r#"
                    <div>
                      <div foo1="bar1" />
                      <T foo2="bar2"><div foo3="bar3" /></T>
                    </div>
                  "#,
            Some(
                serde_json::json!([{ "elementOverrides": { "T": { "noAttributeStrings": true, "applyToNestedElements": false } } }]),
            ),
        ),
        (
            "
                    <div>
                      <div>{'foo'}</div>
                      <T>{'bar'}</T>
                    </div>
                  ",
            Some(serde_json::json!([{ "noStrings": true, "elementOverrides": { "T": {} } }])),
        ),
        (
            "
                    <div>
                      <div>foo</div>
                      <T>foo</T>
                    </div>
                  ",
            Some(
                serde_json::json!([{ "allowedStrings": ["foo"], "elementOverrides": { "T": {} } }]),
            ),
        ),
        (
            "
                    <div>
                      <div>foo</div>
                      <T>foo</T>
                      <T>bar</T>
                      <T>baz</T>
                    </div>
                  ",
            Some(
                serde_json::json!([{ "allowedStrings": ["foo"], "elementOverrides": { "T": { "allowedStrings": ["bar"] } } }]),
            ),
        ),
        (
            r#"
                    <div>
                      <div foo1="bar1" />
                      <T foo2="bar2" />
                    </div>
                  "#,
            Some(
                serde_json::json!([{ "noStrings": true, "ignoreProps": true, "elementOverrides": { "T": { "noStrings": true } } }]),
            ),
        ),
        (
            r#"
                    <div>
                      <div foo1="bar1" />
                      <T foo2="bar2" />
                    </div>
                  "#,
            Some(
                serde_json::json!([{ "noAttributeStrings": true, "elementOverrides": { "T": {} } }]),
            ),
        ),
        (
            "
                    <div>
                      <T>foo</T>
                      <U>bar</U>
                    </div>
                  ",
            Some(serde_json::json!([{ "elementOverrides": { "T": {}, "U": {} } }])),
        ),
        (
            "
                    <div>
                      <T>foo</T>
                      <U>bar</U>
                    </div>
                  ",
            Some(
                serde_json::json!([{ "elementOverrides": { "T": {}, "U": { "allowElement": true } } }]),
            ),
        ),
        (
            "
                    <T>foo <U>bar</U></T>
                  ",
            Some(
                serde_json::json!([{ "elementOverrides": { "T": {}, "U": { "allowElement": true } } }]),
            ),
        ),
        (
            "
                    <T>{'foo'}<U>{'bar'}</U></T>
                  ",
            Some(
                serde_json::json!([{ "elementOverrides": { "T": { "noStrings": true }, "U": {} } }]),
            ),
        ),
        (
            "
                    <T>foo<U>foo</U></T>
                  ",
            Some(
                serde_json::json!([{ "elementOverrides": { "T": { "allowedStrings": ["foo"] }, "U": {} } }]),
            ),
        ),
        (
            "
                    <T>foo<U>foo</U></T>
                  ",
            Some(
                serde_json::json!([{ "elementOverrides": { "T": {}, "U": { "allowedStrings": ["foo"] } } }]),
            ),
        ),
        (
            "
                    <div>
                      <Fragment>foo</Fragment>
                      <React.Fragment>foo</React.Fragment>
                    </div>
                  ",
            Some(
                serde_json::json!([{ "elementOverrides": { "React.Fragment": { "allowElement": true } } }]),
            ),
        ),
        (
            "
                    <div>foo</div>
                  ",
            Some(serde_json::json!([{ "elementOverrides": { "div": { "allowElement": true } } }])),
        ),
        (
            r#"
                    <div>
                      <div type="text" />
                      <Button type="submit" />
                    </div>
                  "#,
            Some(
                serde_json::json!([{ "elementOverrides": { "Button": { "restrictedAttributes": ["type"] }, }, }]),
            ),
        ),
        (
            r#"
                    <div>
                      <Input placeholder="Enter text" type="password" />
                      <Button type="submit" disabled="true" />
                    </div>
                  "#,
            Some(
                serde_json::json!([{ "elementOverrides": { "Input": { "restrictedAttributes": ["placeholder"] }, "Button": { "restrictedAttributes": ["disabled"] }, }, }]),
            ),
        ),
        (
            r#"
                    <div>
                      <div className="wrapper" id="main" />
                      <Button className="btn" id="submit-btn" />
                    </div>
                  "#,
            Some(
                serde_json::json!([{ "restrictedAttributes": ["className"], "elementOverrides": { "Button": { "restrictedAttributes": ["id"] }, }, }]),
            ),
        ),
        (
            r#"
                    <div>
                      <div foo1="bar1" />
                      <T foo2="bar2" />
                    </div>
                  "#,
            Some(
                serde_json::json!([{ "noAttributeStrings": true, "elementOverrides": { "T": { "restrictedAttributes": ["foo2"] }, }, }]),
            ),
        ),
    ];

    Tester::new(JsxNoLiterals::NAME, JsxNoLiterals::PLUGIN, pass, fail).test_and_snapshot();
}
