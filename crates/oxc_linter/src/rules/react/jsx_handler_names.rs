use crate::{AstNode, context::LintContext, rule::Rule};
use lazy_regex::{Regex, RegexBuilder, regex};
use oxc_ast::{
    AstKind,
    ast::{
        ArrowFunctionExpression, Expression, JSXAttributeName, JSXAttributeValue, JSXElementName,
        JSXExpression, JSXMemberExpression, JSXMemberExpressionObject, Statement,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, Span};
use serde_json::Value;

fn bad_handler_name_diagnostic(span: Span, prop_name: &str, handler_name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Invalid handler name: {handler_name}"))
        .with_help(format!(
            "Handler function for {prop_name} prop key must be a camelCase name beginning with \'{handler_name}\' only"
        ))
        .with_label(span)
}

fn bad_handler_prop_name_diagnostic(
    span: Span,
    prop_name: &str,
    handler_name: &str,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Invalid handler prop name: {prop_name}"))
        .with_help(format!("Prop key for {handler_name} must begin with \'{prop_name}\'"))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct JsxHandlerNames(Box<JsxHandlerNamesConfig>);

#[derive(Debug, Clone)]
pub struct JsxHandlerNamesConfig {
    /// Whether to check for inline functions in JSX attributes.
    check_inline_functions: bool,
    /// Whether to check for local variables in JSX attributes.
    check_local_variables: bool,
    /// Event handler prop prefixes to check against.
    event_handler_prop_prefixes: CompactStr,
    /// Event handler prefixes to check against.
    event_handler_prefixes: CompactStr,
    /// Component names to ignore when checking for event handler prefixes.
    ignore_component_names: Vec<CompactStr>,
    /// Compiled regex for event handler prefixes.
    event_handler_regex: Option<Regex>,
    /// Compiled regex for event handler prop prefixes.
    event_handler_prop_regex: Option<Regex>,
}

impl std::ops::Deref for JsxHandlerNames {
    type Target = JsxHandlerNamesConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// See <https://github.com/oxc-project/oxc/issues/6050> for documentation details.
declare_oxc_lint!(
    /// ### What it does
    ///
    /// Ensures that any component or prop methods used to handle events are correctly prefixed.
    ///
    /// ### Why is this bad?
    ///
    /// Inconsistent naming of event handlers and props can reduce code readability and maintainability.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <MyComponent handleChange={this.handleChange} />
    /// <MyComponent onChange={this.componentChanged} />
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <MyComponent onChange={this.handleChange} />
    /// <MyComponent onChange={this.props.onFoo} />
    /// ```
    ///
    /// ### Options
    ///
    /// ```json
    /// {
    ///   "react/jsx-handler-names": [<enabled>, {
    ///     "eventHandlerPrefix": <eventHandlerPrefix>,
    ///     "eventHandlerPropPrefix": <eventHandlerPropPrefix>,
    ///     "checkLocalVariables": <boolean>,
    ///     "checkInlineFunction": <boolean>,
    ///     "ignoreComponentNames": Array<string>
    ///   }]
    /// }
    /// ```
    ///
    /// - `eventHandlerPrefix`: Prefix for component methods used as event handlers.
    ///   Defaults to `handle`
    /// - `eventHandlerPropPrefix`: Prefix for props that are used as event handlers
    ///   Defaults to `on`
    /// - `checkLocalVariables`: Determines whether event handlers stored as local variables
    ///   are checked. Defaults to `false`
    /// - `checkInlineFunction`: Determines whether event handlers set as inline functions are
    ///   checked. Defaults to `false`
    /// - `ignoreComponentNames`: Array of glob strings, when matched with component name,
    ///   ignores the rule on that component. Defaults to `[]`
    ///
    JsxHandlerNames,
    react,
    style,
);

fn build_event_handler_regex(prefixes: &str, prop_prefixes: &str) -> Option<Regex> {
    if prefixes.is_empty() {
        return None;
    }
    Some(
        RegexBuilder::new(
            format!(r"^((props\.{prop_prefixes})|((.*\.)?{prefixes}))[0-9]*[A-Z].*$").as_str(),
        )
        .build()
        .expect("Failed to compile regex for event handler prefixes"),
    )
}

fn build_event_handler_prop_regex(prop_prefix: &str) -> Option<Regex> {
    if prop_prefix.is_empty() {
        return None;
    }
    Some(
        RegexBuilder::new(format!(r"^({prop_prefix}[A-Z].*|ref)$").as_str())
            .build()
            .expect("Failed to compile regex for event handler prop prefixes"),
    )
}

impl Default for JsxHandlerNamesConfig {
    fn default() -> Self {
        let prefix = "handle";
        let prop_prefix = "on";
        JsxHandlerNamesConfig {
            check_inline_functions: false,
            check_local_variables: false,
            event_handler_prop_prefixes: CompactStr::from(prop_prefix),
            event_handler_prefixes: CompactStr::from(prefix),
            ignore_component_names: vec![],
            event_handler_regex: build_event_handler_regex(prefix, prop_prefix),
            event_handler_prop_regex: build_event_handler_prop_regex(prop_prefix),
        }
    }
}

impl Rule for JsxHandlerNames {
    fn from_configuration(value: serde_json::Value) -> Self {
        let mut check_inline_functions = false;
        let mut check_local_variables = false;
        let mut event_handler_prop_prefixes = "on";
        let mut event_handler_prefixes = "handle";
        let mut ignore_component_names = vec![];
        if let Some(options) = value.get(0).and_then(Value::as_object) {
            if let Some(prefixes) = options.get("eventHandlerPrefix") {
                if prefixes.as_bool() == Some(false) {
                    event_handler_prefixes = "";
                } else if let Some(s) = prefixes.as_str() {
                    event_handler_prefixes = s;
                }
            }
            if let Some(prefixes) = options.get("eventHandlerPropPrefix") {
                if prefixes.as_bool() == Some(false) {
                    event_handler_prop_prefixes = "";
                } else if let Some(s) = prefixes.as_str() {
                    event_handler_prop_prefixes = s;
                }
            }
            if let Some(raw) = options.get("checkInlineFunction") {
                if let Some(v) = raw.as_bool() {
                    check_inline_functions = v;
                }
            }
            if let Some(raw) = options.get("checkLocalVariables") {
                if let Some(v) = raw.as_bool() {
                    check_local_variables = v;
                }
            }
            if let Some(names) = options.get("ignoreComponentNames") {
                if let Some(arr) = names.as_array() {
                    for name in arr {
                        if let Some(s) = name.as_str() {
                            ignore_component_names.push(CompactStr::from(s));
                        }
                    }
                }
            }
        }

        let event_handler_regex =
            build_event_handler_regex(event_handler_prefixes, event_handler_prop_prefixes);
        let event_handler_prop_regex = build_event_handler_prop_regex(event_handler_prop_prefixes);

        Self(Box::new(JsxHandlerNamesConfig {
            check_inline_functions,
            check_local_variables,
            event_handler_prop_prefixes: CompactStr::from(event_handler_prop_prefixes),
            event_handler_prefixes: CompactStr::from(event_handler_prefixes),
            ignore_component_names,
            event_handler_regex,
            event_handler_prop_regex,
        }))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXAttribute(jsx_attribute) = node.kind() else {
            return;
        };

        if !self.ignore_component_names.is_empty() {
            let parent_node = ctx.nodes().parent_node(node.id());
            let AstKind::JSXOpeningElement(opening_element) = parent_node.kind() else {
                return;
            };
            let component_name = match &opening_element.name {
                JSXElementName::Identifier(ident) => ident.name.as_str(),
                JSXElementName::IdentifierReference(ident) => ident.name.as_str(),
                JSXElementName::MemberExpression(member_expr) => {
                    &get_member_expression_name(member_expr)
                }
                JSXElementName::NamespacedName(namespaced_name) => {
                    namespaced_name.name.name.as_str()
                }
                JSXElementName::ThisExpression(_) => "this",
            };
            for name in &self.ignore_component_names {
                if fast_glob::glob_match(name.as_ref(), component_name) {
                    return;
                }
            }
        }

        let Some(value) = &jsx_attribute.value else {
            return;
        };
        let JSXAttributeValue::ExpressionContainer(expression_container) = value else {
            return;
        };
        let value_expr = &expression_container.expression;
        let is_inline_handler = matches!(value_expr, JSXExpression::ArrowFunctionExpression(_));
        if !self.check_inline_functions && is_inline_handler {
            return;
        }
        if !self.check_local_variables {
            if let JSXExpression::ArrowFunctionExpression(arrow_function) = value_expr {
                if let Some(Statement::ExpressionStatement(stmt)) =
                    arrow_function.body.statements.first()
                {
                    if let Expression::CallExpression(callee_expr) = &stmt.expression {
                        if !callee_expr.callee.is_member_expression() {
                            // the inline-handler's body is not a method call
                            return;
                        }
                    } else {
                        // the inline-handler's body is not a function call
                        return;
                    }
                }
            } else if !value_expr.is_member_expression() {
                // the inline-handler's body is not an object's property access
                return;
            }
        }

        let prop_key = match &jsx_attribute.name {
            JSXAttributeName::Identifier(ident) => ident.name.as_str(),
            JSXAttributeName::NamespacedName(namespaced_name) => namespaced_name.name.name.as_str(),
        };

        let prop_value = if self.check_inline_functions && is_inline_handler {
            match &expression_container.expression {
                JSXExpression::ArrowFunctionExpression(arrow_function) => {
                    extract_callee_name_from_arrow_function(arrow_function)
                        .map(normalize_handler_name)
                }
                _ => None,
            }
        } else {
            Some(normalize_handler_name(ctx.source_range(expression_container.span.shrink(1))))
        };

        if prop_key == "ref" {
            return;
        }

        let prop_is_event_handler =
            self.event_handler_prop_regex.as_ref().map(|r| r.is_match(prop_key));
        let is_handler_name_correct = prop_value
            .as_ref()
            .map_or(Some(false), |v| self.event_handler_regex.as_ref().map(|r| r.is_match(v)));

        match (prop_is_event_handler, is_handler_name_correct) {
            (Some(true), Some(false)) => {
                ctx.diagnostic(bad_handler_name_diagnostic(
                    expression_container.span,
                    prop_key,
                    &self.event_handler_prefixes,
                ));
            }
            (Some(false), Some(true)) => {
                ctx.diagnostic(bad_handler_prop_name_diagnostic(
                    expression_container.span,
                    prop_key,
                    &self.event_handler_prop_prefixes,
                ));
            }
            _ => {
                // ok
            }
        }
    }
}

fn get_member_expression_name(member_expr: &JSXMemberExpression) -> CompactStr {
    match &member_expr.object {
        JSXMemberExpressionObject::IdentifierReference(ident) => ident.name.as_str().into(),
        JSXMemberExpressionObject::ThisExpression(_) => "this".into(),
        JSXMemberExpressionObject::MemberExpression(next_expr) => format!(
            "{}.{}",
            get_member_expression_name(next_expr),
            member_expr.property.name.as_str()
        )
        .into(),
    }
}

fn normalize_handler_name(s: &str) -> CompactStr {
    let s1 = regex!(r"\s*").replace_all(s, "");
    regex!(r"^this\.|.*::").replace(s1.as_ref(), "").into()
}

// test of aaa
#[test]
fn test_normalize_handler_name() {
    assert_eq!(normalize_handler_name("this.handleChange"), "handleChange");
    assert_eq!(normalize_handler_name("handleChange"), "handleChange");
    assert_eq!(normalize_handler_name("this.props.handleChange"), "props.handleChange");
    assert_eq!(normalize_handler_name("this.props.onChange"), "props.onChange");
    assert_eq!(normalize_handler_name("this.props.handleChange()"), "props.handleChange()");
    assert_eq!(normalize_handler_name("this.props.handleChange(42)"), "props.handleChange(42)");
    assert_eq!(
        normalize_handler_name("this.props.handleChange(42, 'foo')"),
        "props.handleChange(42,'foo')"
    );
    assert_eq!(
        normalize_handler_name("this.props.handleChange(42, 'foo', true)"),
        "props.handleChange(42,'foo',true)"
    );
    assert_eq!(normalize_handler_name("props::handleChange"), "handleChange");
}

fn extract_callee_name_from_arrow_function<'a>(
    arrow_function: &'a ArrowFunctionExpression<'a>,
) -> Option<&'a str> {
    let Some(Statement::ExpressionStatement(stmt)) = arrow_function.body.statements.first() else {
        return None;
    };
    let Expression::CallExpression(call_expr) = &stmt.expression else {
        return None;
    };
    match &call_expr.callee {
        Expression::Identifier(ident) => Some(ident.name.as_str()),
        Expression::StaticMemberExpression(member_expr) => Some(member_expr.property.name.as_str()),
        _ => None,
    }
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
            Some(serde_json::json!([{ "checkInlineFunction": true, "checkLocalVariables": true }])),
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
                serde_json::json!([{ "eventHandlerPrefix": "on", "eventHandlerPropPrefix": "on" }]),
            ),
        ),
        // These test cases are commented out because "::" is not understood by the parser.
        // ("<TestComponent onChange={props::handleChange} />", None),
        // ("<TestComponent onChange={::props.onChange} />", None),
        // ("<TestComponent onChange={props.foo::handleChange} />", None),
        // (
        //     "<TestComponent onChange={() => props::handleChange()} />",
        //     Some(serde_json::json!([{ "checkInlineFunction": true }])),
        // ),
        // (
        //     "<TestComponent onChange={() => ::props.onChange()} />",
        //     Some(serde_json::json!([{ "checkInlineFunction": true }])),
        // ),
        // (
        //     "<TestComponent onChange={() => props.foo::handleChange()} />",
        //     Some(serde_json::json!([{ "checkInlineFunction": true }])),
        // ),
        ("<TestComponent only={this.only} />", None),
        (
            "<TestComponent onChange={this.someChange} />",
            Some(
                serde_json::json!([{ "eventHandlerPrefix": false, "eventHandlerPropPrefix": "on" }]),
            ),
        ),
        (
            "<TestComponent somePrefixChange={this.someChange} />",
            Some(
                serde_json::json!([{ "eventHandlerPrefix": false, "eventHandlerPropPrefix": "somePrefix" }]),
            ),
        ),
        (
            "<TestComponent someProp={this.handleChange} />",
            Some(serde_json::json!([{ "eventHandlerPropPrefix": false }])),
        ),
        (
            "<TestComponent someProp={this.somePrefixChange} />",
            Some(
                serde_json::json!([{ "eventHandlerPrefix": "somePrefix", "eventHandlerPropPrefix": false }]),
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
                  <MyLibButton customPropNameBar={handleSomething} />;
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
            Some(serde_json::json!([{ "checkInlineFunction": true, "checkLocalVariables": true }])),
        ),
        (
            "<TestComponent onChange={handleChange} />",
            Some(
                serde_json::json!([{ "checkLocalVariables": true, "eventHandlerPrefix": "handle", "eventHandlerPropPrefix": "when" }]),
            ),
        ),
        (
            "<TestComponent onChange={() => handleChange()} />",
            Some(
                serde_json::json!([{ "checkInlineFunction": true, "checkLocalVariables": true, "eventHandlerPrefix": "handle", "eventHandlerPropPrefix": "when" }]),
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
                  <MyLibButton customPropNameBar={handleButton} />;
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
