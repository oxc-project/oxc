//! Test cases from eslint-plugin-react (jsx/no-uses-vars)

use super::NoUnusedVars;
use crate::{tester::Tester, RuleMeta as _};

/// !!!! STOP !!!!
/// Are you fixing a bug in this rule and want to add a test case? Please put
/// it in `oxc.rs`. These are _only_ the test cases ported from the original
/// React rule.
#[test]
fn test() {
    let pass = vec![
        "
			var App;
			React.render(<App/>);
        ",
        "
			function foo() {
				var App;
				var bar = React.render(<App/>);
				return bar;
			};
			foo()
        ",
        "
			var a = 1;
			React.render(<img src={a} />);
        ",
        "
			var App;
			function f() {
				return <App />;
			}
			f();
        ",
        "
			var App;
			<App.Hello />
        ",
        "
			class HelloMessage {};
			<HelloMessage />
        ",
        "
			class HelloMessage {
				render() {
				var HelloMessage = <div>Hello</div>;
				return HelloMessage;
				}
			};
			<HelloMessage />
        ",
        "
			function foo() {
				var App = { Foo: { Bar: {} } };
				var bar = React.render(<App.Foo.Bar/>);
				return bar;
			};
			foo()
        ",
        "
			function foo() {
				var App = { Foo: { Bar: { Baz: {} } } };
				var bar = React.render(<App.Foo.Bar.Baz/>);
				return bar;
			};
			foo()
        ",
        "
			var object;
			React.render(<object.Tag />);
		",
        "
			var object;
			React.render(<object.tag />);
		",
    ];

    let fail = vec![
        "
			var App;
		",
        r#"
			var App;
          	var unused;
          	React.render(<App unused=""/>);
		"#,
        "
			var App;
          	var Hello;
          	React.render(<App:Hello/>);
		",
        r#"
			var Button;
			var Input;
			React.render(<Button.Input unused=""/>);
		"#,
        "
			class unused {}
		",
        "
			class HelloMessage {
				render() {
					var HelloMessage = <div>Hello</div>;
					return HelloMessage;
				}
			}
		",
        "
			import {Hello} from 'Hello';
			function Greetings() {
				const Hello = require('Hello').default;
				return <Hello />;
			}
			Greetings();
		",
        "
			var lowercase;
          	React.render(<lowercase />);
		",
        "
			function Greetings(div) {
				return <div />;
			}
			Greetings();
		",
    ];

    Tester::new(NoUnusedVars::NAME, NoUnusedVars::PLUGIN, pass, fail)
        .intentionally_allow_no_fix_tests()
        .with_snapshot_suffix("eslint-plugin-react")
        .test_and_snapshot();
}
