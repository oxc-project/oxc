use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    AstNode,
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    rule::{DefaultRuleConfig, Rule},
};

fn no_multi_comp_diagnostic(span: Span) -> OxcDiagnostic {
    // See <https://oxc.rs/docs/contribute/linter/adding-rules.html#diagnostics> for details
    OxcDiagnostic::warn("Should be an imperative statement about what is wrong.")
        .with_help("Should be a command-like statement that tells the user how to fix the issue.")
        .with_label(span)
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename_all = "camelCase")]
struct ConfigElement0 {
    ignore_stateless: bool,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
pub struct NoMultiComp(ConfigElement0);

// See <https://github.com/oxc-project/oxc/issues/6050> for documentation details.
declare_oxc_lint!(
    /// ### What it does
    ///
    /// Briefly describe the rule's purpose.
    ///
    /// ### Why is this bad?
    ///
    /// Explain why violating this rule is problematic.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// FIXME: Tests will fail if examples are missing or syntactically incorrect.
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// FIXME: Tests will fail if examples are missing or syntactically incorrect.
    /// ```
    NoMultiComp,
    react,
    nursery, // TODO: change category to `correctness`, `suspicious`, `pedantic`, `perf`, `restriction`, or `style`
             // See <https://oxc.rs/docs/contribute/linter.html#rule-category> for details
    pending, // TODO: describe fix capabilities. Remove if no fix can be done,
             // keep at 'pending' if you think one could be added but don't know how.
             // Options are 'fix', 'fix_dangerous', 'suggestion', and 'conditional_fix_suggestion'
    config = NoMultiComp,
);

impl Rule for NoMultiComp {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        Ok(serde_json::from_value::<DefaultRuleConfig<Self>>(value)
            .unwrap_or_default()
            .into_inner())
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {}
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (r#"
			        var Hello = require('./components/Hello');
			        var HelloJohn = createReactClass({
			          render: function() {
			            return <Hello name="John" />;
			          }
			        });
			      "#, None, None),
("
			        class Hello extends React.Component {
			          render() {
			            return <div>Hello {this.props.name}</div>;
			          }
			        }
			      ", None, None),
("
			        var Heading = createReactClass({
			          render: function() {
			            return (
			              <div>
			                {this.props.buttons.map(function(button, index) {
			                  return <Button {...button} key={index}/>;
			                })}
			              </div>
			            );
			          }
			        });
			      ", None, None),
("
			        function Hello(props) {
			          return <div>Hello {props.name}</div>;
			        }
			        function HelloAgain(props) {
			          return <div>Hello again {props.name}</div>;
			        }
			      ", Some(serde_json::json!([{ "ignoreStateless": true }])), None),
(r#"
			        function Hello(props) {
			          return <div>Hello {props.name}</div>;
			        }
			        class HelloJohn extends React.Component {
			          render() {
			            return <Hello name="John" />;
			          }
			        }
			      "#, Some(serde_json::json!([{ "ignoreStateless": true }])), None),
(r#"
			        import React, { createElement } from "react"
			        const helperFoo = () => {
			          return true;
			        };
			        function helperBar() {
			          return false;
			        };
			        function RealComponent() {
			          return createElement("img");
			        };
			      "#, None, None),
(r#"
			        const Hello = React.memo(function(props) {
			          return <div>Hello {props.name}</div>;
			        });
			        class HelloJohn extends React.Component {
			          render() {
			            return <Hello name="John" />;
			          }
			        }
			      "#, Some(serde_json::json!([{ "ignoreStateless": true }])), None),
("
			        class StoreListItem extends React.PureComponent {
			          // A bunch of stuff here
			        }
			        export default React.forwardRef((props, ref) => <StoreListItem {...props} forwardRef={ref} />);
			      ", Some(serde_json::json!([{ "ignoreStateless": false }])), None),
("
			        class StoreListItem extends React.PureComponent {
			          // A bunch of stuff here
			        }
			        export default React.forwardRef((props, ref) => {
			          return <StoreListItem {...props} forwardRef={ref} />
			        });
			      ", Some(serde_json::json!([{ "ignoreStateless": false }])), None),
("
			        const HelloComponent = (props) => {
			          return <div></div>;
			        }
			        export default React.forwardRef((props, ref) => <HelloComponent {...props} forwardRef={ref} />);
			      ", Some(serde_json::json!([{ "ignoreStateless": false }])), None),
("
			        class StoreListItem extends React.PureComponent {
			          // A bunch of stuff here
			        }
			        export default React.forwardRef(
			          function myFunction(props, ref) {
			            return <StoreListItem {...props} forwardedRef={ref} />;
			          }
			        );
			      ", Some(serde_json::json!([{ "ignoreStateless": true }])), None),
("
			        const HelloComponent = (props) => {
			          return <div></div>;
			        }
			        class StoreListItem extends React.PureComponent {
			          // A bunch of stuff here
			        }
			        export default React.forwardRef(
			          function myFunction(props, ref) {
			            return <StoreListItem {...props} forwardedRef={ref} />;
			          }
			        );
			      ", Some(serde_json::json!([{ "ignoreStateless": true }])), None),
("
			        const HelloComponent = (props) => {
			          return <div></div>;
			        }
			        export default React.memo((props, ref) => <HelloComponent {...props} />);
			      ", Some(serde_json::json!([{ "ignoreStateless": true }])), None),
(r#"
			        import React from 'react';
			        function memo() {
			          var outOfScope = "hello"
			          return null;
			        }
			        class ComponentY extends React.Component {
			          memoCities = memo((cities) => cities.map((v) => ({ label: v })));
			          render() {
			            return (
			              <div>
			                <div>Counter</div>
			              </div>
			            );
			          }
			        }
			      "#, None, None),
(r#"
			        const MenuList = forwardRef(({onClose, ...props}, ref) => {
			          const {t} = useTranslation();
			          const handleLogout = useLogoutHandler();

			          const onLogout = useCallback(() => {
			            onClose();
			            handleLogout();
			          }, [onClose, handleLogout]);

			          return (
			            <MuiMenuList ref={ref} {...props}>
			              <MuiMenuItem key="logout" onClick={onLogout}>
			                {t('global-logout')}
			              </MuiMenuItem>
			            </MuiMenuList>
			          );
			        });

			        MenuList.displayName = 'MenuList';

			        MenuList.propTypes = {
			          onClose: PropTypes.func,
			        };

			        MenuList.defaultProps = {
			          onClose: () => null,
			        };

			        export default MenuList;
			      "#, None, None),
(r#"
			        const MenuList = forwardRef(({ onClose, ...props }, ref) => {
			          const onLogout = useCallback(() => {
			            onClose()
			          }, [onClose])

			          return (
			            <BlnMenuList ref={ref} {...props}>
			              <BlnMenuItem key="logout" onClick={onLogout}>
			                Logout
			              </BlnMenuItem>
			            </BlnMenuList>
			          )
			        })

			        MenuList.displayName = 'MenuList'

			        MenuList.propTypes = {
			          onClose: PropTypes.func
			        }

			        MenuList.defaultProps = {
			          onClose: () => null
			        }

			        export default MenuList
			      "#, None, None)
    ];

    let fail = vec![
        ("
			        function Hello(props) {
			          return <div>Hello {props.name}</div>;
			        }
			        function HelloAgain(props) {
			          return <div>Hello again {props.name}</div>;
			        }
			      ", None, None),
(r#"
			        function Hello(props) {
			          return <div>Hello {props.name}</div>;
			        }
			        class HelloJohn extends React.Component {
			          render() {
			            return <Hello name="John" />;
			          }
			        }
			      "#, None, None),
("
			        export default {
			          RenderHello(props) {
			            let {name} = props;
			            return <div>{name}</div>;
			          },
			          RenderHello2(props) {
			            let {name} = props;
			            return <div>{name}</div>;
			          }
			        };
			      ", None, None),
("
			        exports.Foo = function Foo() {
			          return <></>
			        }

			        exports.createSomeComponent = function createSomeComponent(opts) {
			          return function Foo() {
			            return <>{opts.a}</>
			          }
			        }
			      ", None, None),
("
			        class StoreListItem extends React.PureComponent {
			          // A bunch of stuff here
			        }
			        export default React.forwardRef((props, ref) => <div><StoreListItem {...props} forwardRef={ref} /></div>);
			      ", Some(serde_json::json!([{ "ignoreStateless": false }])), None),
("
			        const HelloComponent = (props) => {
			          return <div></div>;
			        }
			        const HelloComponent2 = React.forwardRef((props, ref) => <div></div>);
			      ", Some(serde_json::json!([{ "ignoreStateless": false }])), None),
("
			        const HelloComponent = (0, (props) => {
			          return <div></div>;
			        });
			        const HelloComponent2 = React.forwardRef((props, ref) => <><HelloComponent></HelloComponent></>);
			      ", Some(serde_json::json!([{ "ignoreStateless": false }])), None),
("
			        const forwardRef = React.forwardRef;
			        const HelloComponent = (0, (props) => {
			          return <div></div>;
			        });
			        const HelloComponent2 = forwardRef((props, ref) => <HelloComponent></HelloComponent>);
			      ", Some(serde_json::json!([{ "ignoreStateless": false }])), None),
("
			        const memo = React.memo;
			        const HelloComponent = (props) => {
			          return <div></div>;
			        };
			        const HelloComponent2 = memo((props) => <HelloComponent></HelloComponent>);
			      ", Some(serde_json::json!([{ "ignoreStateless": false }])), None),
("
			        const {forwardRef} = React;
			        const HelloComponent = (0, (props) => {
			          return <div></div>;
			        });
			        const HelloComponent2 = forwardRef((props, ref) => <HelloComponent></HelloComponent>);
			      ", Some(serde_json::json!([{ "ignoreStateless": false }])), None),
("
			        const {memo} = React;
			        const HelloComponent = (0, (props) => {
			          return <div></div>;
			        });
			        const HelloComponent2 = memo((props) => <HelloComponent></HelloComponent>);
			      ", Some(serde_json::json!([{ "ignoreStateless": false }])), None),
("
			        import React, { memo } from 'react';
			        const HelloComponent = (0, (props) => {
			          return <div></div>;
			        });
			        const HelloComponent2 = memo((props) => <HelloComponent></HelloComponent>);
			      ", Some(serde_json::json!([{ "ignoreStateless": false }])), None),
("
			        import {forwardRef} from 'react';
			        const HelloComponent = (0, (props) => {
			          return <div></div>;
			        });
			        const HelloComponent2 = forwardRef((props, ref) => <HelloComponent></HelloComponent>);
			      ", Some(serde_json::json!([{ "ignoreStateless": false }])), None),
("
			        const memo = require('react').memo;
			        const HelloComponent = (0, (props) => {
			          return <div></div>;
			        });
			        const HelloComponent2 = memo((props) => <HelloComponent></HelloComponent>);
			      ", Some(serde_json::json!([{ "ignoreStateless": false }])), None),
("
			        import Foo, { memo, forwardRef } from 'foo';
			        const Text = forwardRef(({ text }, ref) => {
			          return <div ref={ref}>{text}</div>;
			        })
			        const Label = memo(() => <Text />);
			      ", None, Some(serde_json::json!({ "settings": {  "react": {  "pragma": "Foo",  },  } })))
    ];

    Tester::new(NoMultiComp::NAME, NoMultiComp::PLUGIN, pass, fail).test_and_snapshot();
}
