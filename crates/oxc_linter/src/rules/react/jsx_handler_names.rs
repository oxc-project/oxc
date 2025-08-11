use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    rule::Rule,
};

fn jsx_handler_names_diagnostic(span: Span) -> OxcDiagnostic {
    // See <https://oxc.rs/docs/contribute/linter/adding-rules.html#diagnostics> for details
    OxcDiagnostic::warn("Should be an imperative statement about what is wrong")
        .with_help("Should be a command-like statement that tells the user how to fix the issue")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct JsxHandlerNames;

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
    JsxHandlerNames,
    react,
    nursery, // TODO: change category to `correctness`, `suspicious`, `pedantic`, `perf`, `restriction`, or `style`
             // See <https://oxc.rs/docs/contribute/linter.html#rule-category> for details
    pending  // TODO: describe fix capabilities. Remove if no fix can be done,
             // keep at 'pending' if you think one could be added but don't know how.
             // Options are 'fix', 'fix_dangerous', 'suggestion', and 'conditional_fix_suggestion'
);

impl Rule for JsxHandlerNames {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {}
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("<TestComponent onChange={this.handleChange} />", None),
        ("<TestComponent onChange={this.handle123Change} />", None),
        ("<TestComponent onChange={this.props.onChange} />", None),
        (
            "
			        <TestComponent
			          onChange={
			            this
			              .handleChange
			          } />
			      ",
            None,
        ),
        (
            "
			        <TestComponent
			          onChange={
			            this
			              .props
			              .handleChange
			          } />
			      ",
            None,
        ),
        (
            "<TestComponent onChange={handleChange} />",
            Some(serde_json::json!([{ "checkLocalVariables": true }])),
        ),
        (
            "<TestComponent onChange={takeCareOfChange} />",
            Some(serde_json::json!([{ "checkLocalVariables": false }])),
        ),
        (
            "<TestComponent onChange={event => window.alert(event.target.value)} />",
            Some(serde_json::json!([{ "checkInlineFunction": false }])),
        ),
        (
            "<TestComponent onChange={() => handleChange()} />",
            Some(
                serde_json::json!([        {          "checkInlineFunction": true,          "checkLocalVariables": true,        },      ]),
            ),
        ),
        (
            "<TestComponent onChange={() => this.handleChange()} />",
            Some(serde_json::json!([{ "checkInlineFunction": true }])),
        ),
        ("<TestComponent onChange={() => 42} />", None),
        ("<TestComponent onChange={this.props.onFoo} />", None),
        ("<TestComponent isSelected={this.props.isSelected} />", None),
        ("<TestComponent shouldDisplay={this.state.shouldDisplay} />", None),
        ("<TestComponent shouldDisplay={arr[0].prop} />", None),
        ("<TestComponent onChange={props.onChange} />", None),
        ("<TestComponent ref={this.handleRef} />", None),
        ("<TestComponent ref={this.somethingRef} />", None),
        (
            "<TestComponent test={this.props.content} />",
            Some(
                serde_json::json!([        {          "eventHandlerPrefix": "on",          "eventHandlerPropPrefix": "on",        },      ]),
            ),
        ),
        ("<TestComponent onChange={props::handleChange} />", None),
        ("<TestComponent onChange={::props.onChange} />", None),
        ("<TestComponent onChange={props.foo::handleChange} />", None),
        (
            "<TestComponent onChange={() => props::handleChange()} />",
            Some(serde_json::json!([{ "checkInlineFunction": true }])),
        ),
        (
            "<TestComponent onChange={() => ::props.onChange()} />",
            Some(serde_json::json!([{ "checkInlineFunction": true }])),
        ),
        (
            "<TestComponent onChange={() => props.foo::handleChange()} />",
            Some(serde_json::json!([{ "checkInlineFunction": true }])),
        ),
        ("<TestComponent only={this.only} />", None),
        (
            "<TestComponent onChange={this.someChange} />",
            Some(
                serde_json::json!([        {          "eventHandlerPrefix": false,          "eventHandlerPropPrefix": "on",        },      ]),
            ),
        ),
        (
            "<TestComponent somePrefixChange={this.someChange} />",
            Some(
                serde_json::json!([        {          "eventHandlerPrefix": false,          "eventHandlerPropPrefix": "somePrefix",        },      ]),
            ),
        ),
        (
            "<TestComponent someProp={this.handleChange} />",
            Some(serde_json::json!([{ "eventHandlerPropPrefix": false }])),
        ),
        (
            "<TestComponent someProp={this.somePrefixChange} />",
            Some(
                serde_json::json!([        {          "eventHandlerPrefix": "somePrefix",          "eventHandlerPropPrefix": false,        },      ]),
            ),
        ),
        (
            "<TestComponent someProp={props.onChange} />",
            Some(serde_json::json!([{ "eventHandlerPropPrefix": false }])),
        ),
        (
            "<ComponentFromOtherLibraryBar customPropNameBar={handleSomething} />;",
            Some(
                serde_json::json!([{ "checkLocalVariables": true, "ignoreComponentNames": ["ComponentFromOtherLibraryBar"] }]),
            ),
        ),
        (
            "
			        function App() {
			          return (
			            <div>
			              <MyLibInput customPropNameBar={handleSomething} />;
			              <MyLibCheckbox customPropNameBar={handleSomething} />;
			              <MyLibButtom customPropNameBar={handleSomething} />;
			            </div>
			          )
			        }
			      ",
            Some(
                serde_json::json!([{ "checkLocalVariables": true, "ignoreComponentNames": ["MyLib*"] }]),
            ),
        ),
    ];

    let fail = vec![
        ("<TestComponent onChange={this.doSomethingOnChange} />", None),
        ("<TestComponent onChange={this.handlerChange} />", None),
        ("<TestComponent onChange={this.handle} />", None),
        ("<TestComponent onChange={this.handle2} />", None),
        ("<TestComponent onChange={this.handl3Change} />", None),
        ("<TestComponent onChange={this.handle4change} />", None),
        (
            "<TestComponent onChange={takeCareOfChange} />",
            Some(serde_json::json!([{ "checkLocalVariables": true }])),
        ),
        (
            "<TestComponent onChange={() => this.takeCareOfChange()} />",
            Some(serde_json::json!([{ "checkInlineFunction": true }])),
        ),
        ("<TestComponent only={this.handleChange} />", None),
        ("<TestComponent2 only={this.handleChange} />", None),
        ("<TestComponent handleChange={this.handleChange} />", None),
        (
            "<TestComponent whenChange={handleChange} />",
            Some(serde_json::json!([{ "checkLocalVariables": true }])),
        ),
        (
            "<TestComponent whenChange={() => handleChange()} />",
            Some(
                serde_json::json!([        {          "checkInlineFunction": true,          "checkLocalVariables": true,        },      ]),
            ),
        ),
        (
            "<TestComponent onChange={handleChange} />",
            Some(
                serde_json::json!([        {          "checkLocalVariables": true,          "eventHandlerPrefix": "handle",          "eventHandlerPropPrefix": "when",        },      ]),
            ),
        ),
        (
            "<TestComponent onChange={() => handleChange()} />",
            Some(
                serde_json::json!([        {          "checkInlineFunction": true,          "checkLocalVariables": true,          "eventHandlerPrefix": "handle",          "eventHandlerPropPrefix": "when",        },      ]),
            ),
        ),
        ("<TestComponent onChange={this.onChange} />", None),
        ("<TestComponent onChange={props::onChange} />", None),
        ("<TestComponent onChange={props.foo::onChange} />", None),
        (
            "
			        function App() {
			          return (
			            <div>
			              <MyLibInput customPropNameBar={handleInput} />;
			              <MyLibCheckbox customPropNameBar={handleCheckbox} />;
			              <MyLibButtom customPropNameBar={handleButton} />;
			            </div>
			          )
			        }
			      ",
            Some(
                serde_json::json!([{ "checkLocalVariables": true, "ignoreComponentNames": ["MyLibrary*"] }]),
            ),
        ),
    ];

    Tester::new(JsxHandlerNames::NAME, JsxHandlerNames::PLUGIN, pass, fail).test_and_snapshot();
}
