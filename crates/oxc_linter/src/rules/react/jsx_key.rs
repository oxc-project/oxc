use std::ops::Deref;

use cow_utils::CowUtils;
use rustc_hash::FxHashSet;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use oxc_ast::{
    AstKind,
    ast::{
        ArrayExpression, ArrayExpressionElement, CallExpression, Expression, IdentifierReference,
        JSXAttributeItem, JSXAttributeName, JSXAttributeValue, JSXChild, JSXElement, JSXExpression,
        JSXFragment, Statement,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    ast_util::is_node_within_call_argument,
    context::{ContextHost, LintContext},
    rule::{DefaultRuleConfig, Rule},
    utils::default_true,
};

const TARGET_METHODS: [&str; 3] = ["flatMap", "from", "map"];

fn missing_key_prop_for_element_in_array(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(r#"Missing "key" prop for element in array."#).with_label(span)
}

fn missing_key_prop_for_element_in_iterator(iter_span: Span, el_span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(r#"Missing "key" prop for element in iterator."#)
        .with_help(r#"Add a "key" prop to the element in the iterator (https://react.dev/learn/rendering-lists#keeping-list-items-in-order-with-key)."#)
        .with_labels([
            iter_span.label("Iterator starts here."),
            el_span.label("Element generated here."),
        ])
}

fn key_prop_must_be_placed_before_spread(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(r#""key" prop must be placed before any `{...spread}`"#)
        .with_help("To avoid conflicting with React's new JSX transform: https://reactjs.org/blog/2020/09/22/introducing-the-new-jsx-transform.html")
        .with_label(span)
}

fn duplicate_key_prop(key_value: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Duplicate key '{key_value}' found in JSX elements"))
        .with_help("Each child in a list should have a unique 'key' prop")
        .with_label(span)
}

#[derive(Debug, Default, Clone, JsonSchema, Deserialize)]
#[schemars(transparent)]
pub struct JsxKey(Box<JsxKeyConfig>);

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct JsxKeyConfig {
    /// When true, require key prop to be placed before any spread props
    #[serde(default = "default_true")]
    pub check_key_must_before_spread: bool,
    /// When true, warn on duplicate key values
    #[serde(default = "default_true")]
    pub warn_on_duplicates: bool,
    /// When true, check fragment shorthand `<>` for keys
    #[serde(default = "default_true")]
    pub check_fragment_shorthand: bool,
}

impl Deref for JsxKey {
    type Target = JsxKeyConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce `key` prop for elements in array
    ///
    /// ### Why is this bad?
    ///
    /// React requires a `key` prop for elements in an array to help identify which items have changed, are added, or are removed.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// [1, 2, 3].map(x => <App />);
    /// [1, 2, 3]?.map(x => <BabelEslintApp />)
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// [1, 2, 3].map(x => <App key={x} />);
    /// [1, 2, 3]?.map(x => <BabelEslintApp key={x} />)
    /// ```
    JsxKey,
    react,
    correctness,
    config = JsxKey,
);

impl Rule for JsxKey {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::JSXElement(jsx_elem) => {
                check_jsx_element(node, jsx_elem, ctx);
                if self.check_key_must_before_spread {
                    check_jsx_element_is_key_before_spread(jsx_elem, ctx);
                }
                if self.warn_on_duplicates {
                    check_duplicate_keys_in_children(jsx_elem, ctx);
                }
            }
            AstKind::JSXFragment(jsx_frag) => {
                if self.check_fragment_shorthand {
                    check_jsx_fragment(node, jsx_frag, ctx);
                }
            }
            AstKind::ArrayExpression(array_expr) => {
                if self.warn_on_duplicates {
                    check_duplicate_keys_in_array(array_expr, ctx);
                }
            }

            _ => {}
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_jsx()
    }
}

pub fn is_to_array(call: &CallExpression<'_>) -> bool {
    call.callee_name().is_some_and(|subject| subject == "toArray")
}

pub fn import_matcher<'a>(
    ctx: &LintContext<'a>,
    actual_local_name: &'a str,
    expected_module_name: &'a str,
) -> bool {
    let expected_module_name = expected_module_name.cow_to_ascii_lowercase();
    ctx.module_record().import_entries.iter().any(|import| {
        import.module_request.name() == expected_module_name
            && import.local_name.name() == actual_local_name
    })
}

pub fn is_import<'a>(
    ctx: &LintContext<'a>,
    actual_local_name: &'a str,
    expected_local_name: &'a str,
    expected_module_name: &'a str,
) -> bool {
    if ctx.module_record().requested_modules.is_empty()
        && ctx.scoping().get_bindings(ctx.scoping().root_scope_id()).is_empty()
    {
        return actual_local_name == expected_local_name;
    }

    import_matcher(ctx, actual_local_name, expected_module_name)
}

fn is_children_from_react<'a>(ident: &IdentifierReference<'a>, ctx: &LintContext<'a>) -> bool {
    const REACT_MODULE: &str = "react";
    const CHILDREN: &str = "Children";

    let name = ident.name.as_str();

    // Check if directly imported: import { Children } from 'react'
    if import_matcher(ctx, name, REACT_MODULE) {
        return true;
    }

    // Check if it's a local variable that might be destructured from React
    // e.g., const { Children } = React; or const { Children } = Act;
    if name == CHILDREN {
        // Get the symbol ID from the reference
        if let Some(reference_id) = ident.reference_id.get() {
            let reference = ctx.scoping().get_reference(reference_id);
            if let Some(symbol_id) = reference.symbol_id() {
                // Get the declaration node
                let decl_id = ctx.scoping().symbol_declaration(symbol_id);
                let decl_node = ctx.nodes().get_node(decl_id);

                // Check if this is a VariableDeclarator with ObjectPattern
                if let AstKind::VariableDeclarator(var_decl) = decl_node.kind() {
                    // Check if init is an identifier imported from React
                    if let Some(Expression::Identifier(init_ident)) = var_decl.init.as_ref() {
                        // Check if the init identifier is imported from 'react' module
                        return import_matcher(ctx, init_ident.name.as_str(), REACT_MODULE);
                    }
                }
            }
        }
    }

    false
}

pub fn is_children<'a, 'b>(call: &'b CallExpression<'a>, ctx: &'b LintContext<'a>) -> bool {
    const REACT: &str = "React";
    const CHILDREN: &str = "Children";

    let Some(member) = call.callee.as_member_expression() else { return false };

    if let Expression::Identifier(ident) = member.object() {
        return is_children_from_react(ident, ctx);
    }

    let Some(inner_member) = member.object().get_inner_expression().as_member_expression() else {
        return false;
    };

    let Some(ident) = inner_member.object().get_identifier_reference() else { return false };

    let Some(local_name) = inner_member.static_property_name() else { return false };

    is_import(ctx, ident.name.as_str(), REACT, REACT) && local_name == CHILDREN
}
fn is_within_children_to_array<'a, 'b>(node: &'b AstNode<'a>, ctx: &'b LintContext<'a>) -> bool {
    let parents_iter = ctx.nodes().ancestors(node.id()).skip(1);
    parents_iter
        .filter_map(|parent_node| parent_node.kind().as_call_expression())
        .any(|parent_call| is_children(parent_call, ctx) && is_to_array(parent_call))
}

enum InsideArrayOrIterator {
    Array,
    Iterator(Span),
}

#[expect(clippy::bool_to_int_with_if)]
fn is_in_array_or_iter<'a, 'b>(
    node: &'b AstNode<'a>,
    ctx: &'b LintContext<'a>,
) -> Option<InsideArrayOrIterator> {
    let jsx_node = node;
    let mut node = node;

    let mut is_outside_containing_function = false;
    let mut is_explicit_return = false;

    while !matches!(node.kind(), AstKind::Program(_)) {
        let parent = ctx.nodes().parent_node(node.id());
        match parent.kind() {
            AstKind::ArrowFunctionExpression(arrow_expr) => {
                let is_arrow_expr_statement = matches!(
                    arrow_expr.body.statements.first(),
                    Some(Statement::ExpressionStatement(_))
                );
                if !is_explicit_return && !is_arrow_expr_statement {
                    return None;
                }

                if let AstKind::ObjectProperty(_) = ctx.nodes().parent_kind(parent.id()) {
                    return None;
                }
                if is_outside_containing_function {
                    return None;
                }

                is_outside_containing_function = true;
            }
            AstKind::Function(_) => {
                if let AstKind::ObjectProperty(_) = ctx.nodes().parent_kind(parent.id()) {
                    return None;
                }
                if is_outside_containing_function {
                    return None;
                }

                is_outside_containing_function = true;
            }
            AstKind::ArrayExpression(_) => {
                if is_outside_containing_function {
                    return None;
                }

                return Some(InsideArrayOrIterator::Array);
            }
            AstKind::CallExpression(v) => {
                let callee = &v.callee.without_parentheses();

                if let Some(member_expr) = callee.as_member_expression()
                    && let Some((span, ident)) = member_expr.static_property_info()
                    && TARGET_METHODS.contains(&ident)
                {
                    // Early exit if no arguments to check
                    if v.arguments.is_empty() {
                        return None;
                    }

                    // Array.from uses 2nd argument (index 1), others use 1st argument (index 0)
                    let target_arg_index = if ident == "from" { 1 } else { 0 };
                    if is_node_within_call_argument(jsx_node, v, target_arg_index) {
                        return Some(InsideArrayOrIterator::Iterator(span));
                    }
                }

                return None;
            }
            AstKind::JSXElement(_)
            | AstKind::JSXOpeningElement(_)
            | AstKind::ObjectProperty(_)
            | AstKind::JSXFragment(_) => return None,
            AstKind::ReturnStatement(_) => {
                is_explicit_return = true;
            }
            _ => {}
        }
        node = parent;
    }

    None
}

fn check_jsx_element<'a>(node: &AstNode<'a>, jsx_elem: &JSXElement<'a>, ctx: &LintContext<'a>) {
    if let Some(outer) = is_in_array_or_iter(node, ctx) {
        if is_within_children_to_array(node, ctx) {
            return;
        }
        if !jsx_elem.opening_element.attributes.iter().any(|attr| {
            let JSXAttributeItem::Attribute(attr) = attr else {
                return false;
            };

            let JSXAttributeName::Identifier(attr_ident) = &attr.name else {
                return false;
            };
            attr_ident.name == "key"
        }) {
            ctx.diagnostic(gen_diagnostic(jsx_elem.opening_element.name.span(), &outer));
        }
    }
}

fn check_jsx_element_is_key_before_spread<'a>(jsx_elem: &JSXElement<'a>, ctx: &LintContext<'a>) {
    let mut key_idx_span: Option<(usize, Span)> = None;
    let mut spread_idx: Option<usize> = None;

    for (i, attr) in jsx_elem.opening_element.attributes.iter().enumerate() {
        match attr {
            JSXAttributeItem::Attribute(attr) => {
                let JSXAttributeName::Identifier(ident) = &attr.name else {
                    continue;
                };
                if ident.name == "key" {
                    key_idx_span = Some((i, attr.name.span()));
                }
            }
            JSXAttributeItem::SpreadAttribute(_) => spread_idx = Some(i),
        }
        if key_idx_span.map(|x| x.0).is_some() && spread_idx.is_some() {
            break;
        }
    }

    if let (Some((key_idx, key_span)), Some(spread_idx)) = (key_idx_span, spread_idx)
        && key_idx > spread_idx
    {
        ctx.diagnostic(key_prop_must_be_placed_before_spread(key_span));
    }
}

fn check_jsx_fragment<'a>(node: &AstNode<'a>, fragment: &JSXFragment<'a>, ctx: &LintContext<'a>) {
    if let Some(outer) = is_in_array_or_iter(node, ctx) {
        if is_within_children_to_array(node, ctx) {
            return;
        }
        ctx.diagnostic(gen_diagnostic(fragment.opening_fragment.span, &outer));
    }
}

fn gen_diagnostic(span: Span, outer: &InsideArrayOrIterator) -> OxcDiagnostic {
    match outer {
        InsideArrayOrIterator::Array => missing_key_prop_for_element_in_array(span),
        InsideArrayOrIterator::Iterator(v) => missing_key_prop_for_element_in_iterator(*v, span),
    }
}

fn get_jsx_element_key_value(jsx_elem: &JSXElement) -> Option<(String, Span)> {
    for attr in &jsx_elem.opening_element.attributes {
        if let JSXAttributeItem::Attribute(attr) = attr
            && let JSXAttributeName::Identifier(ident) = &attr.name
            && ident.name == "key"
        {
            // Extract the key value
            if let Some(value) = &attr.value {
                match value {
                    JSXAttributeValue::StringLiteral(lit) => {
                        return Some((lit.value.to_string(), attr.span));
                    }
                    JSXAttributeValue::ExpressionContainer(container) => {
                        // JSXExpression inherits from Expression, so we match the Expression variants directly
                        match &container.expression {
                            JSXExpression::StringLiteral(lit) => {
                                return Some((lit.value.to_string(), attr.span));
                            }
                            JSXExpression::NumericLiteral(lit) => {
                                return Some((lit.value.to_string(), attr.span));
                            }
                            JSXExpression::TemplateLiteral(lit)
                                if lit.expressions.is_empty() && lit.quasis.len() == 1 =>
                            {
                                return Some((lit.quasis[0].value.raw.to_string(), attr.span));
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    None
}

fn check_duplicate_keys_in_array<'a>(array_expr: &ArrayExpression<'a>, ctx: &LintContext<'a>) {
    let mut seen_keys: FxHashSet<String> = FxHashSet::default();

    for element in &array_expr.elements {
        // ArrayExpressionElement also inherits from Expression
        if let ArrayExpressionElement::JSXElement(jsx_elem) = element
            && let Some((key_value, span)) = get_jsx_element_key_value(jsx_elem)
            && !seen_keys.insert(key_value.clone())
        {
            ctx.diagnostic(duplicate_key_prop(&key_value, span));
        }
    }
}

fn check_duplicate_keys_in_children<'a>(jsx_elem: &JSXElement<'a>, ctx: &LintContext<'a>) {
    let mut seen_keys: FxHashSet<String> = FxHashSet::default();

    for child in &jsx_elem.children {
        if let JSXChild::Element(child_elem) = child
            && let Some((key_value, span)) = get_jsx_element_key_value(child_elem)
            && !seen_keys.insert(key_value.clone())
        {
            ctx.diagnostic(duplicate_key_prop(&key_value, span));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("fn()", None, None),
        (r"[1, 2, 3].map(function () {})", None, None),
        (r"<App />;", None, None),
        (r"[<App key={0} />, <App key={1} />];", None, None),
        (r"[1, 2, 3].map(function(x) { return <App key={x} /> });", None, None),
        (r"[1, 2, 3].map(x => <App key={x} />);", None, None),
        (r"[1, 2 ,3].map(x => x && <App x={x} key={x} />);", None, None),
        (r#"[1, 2 ,3].map(x => x ? <App x={x} key="1" /> : <OtherApp x={x} key="2" />);"#, None, None),
        (r"[1, 2, 3].map(x => { return <App key={x} /> });", None, None),
        (r"Array.from([1, 2, 3], function(x) { return <App key={x} /> });", None, None),
        (r"Array.from([1, 2, 3], (x => <App key={x} />));", None, None),
        (r"Array.from([1, 2, 3], (x => {return <App key={x} />}));", None, None),
        (r"Array.from([1, 2, 3], someFn);", None, None),
        (r"Array.from([1, 2, 3]);", None, None),
        (r"[1, 2, 3].foo(x => <App />);", None, None),
        (r"var App = () => <div />;", None, None),
        (r"[1, 2, 3].map(function(x) { return; });", None, None),
        (r"foo(() => <div />);", None, None),
        (r"foo(() => <></>);", None, None),
        (r"<></>;", None, None),
        (r"<App {...{}} />;", None, None),
(r#"<App key="keyBeforeSpread" {...{}} />;"#, Some(serde_json::json!([{ "checkKeyMustBeforeSpread": true }])), None),
(r#"<div key="keyBeforeSpread" {...{}} />;"#, Some(serde_json::json!([{ "checkKeyMustBeforeSpread": true }])), None),
(r#"
			        const spans = [
			          <span key="notunique"/>,
			          <span key="notunique"/>,
			        ];
			      "#, None, None),
(r#"
			        function Component(props) {
			          return hasPayment ? (
			            <div className="stuff">
			              <BookingDetailSomething {...props} />
			              {props.modal && props.calculatedPrice && (
			                <SomeOtherThing items={props.something} discount={props.discount} />
			              )}
			            </div>
			          ) : null;
			        }
			      "#, None, None),
(r#"
			        import React, { FC, useRef, useState } from 'react';

			        import './ResourceVideo.sass';
			        import VimeoVideoPlayInModal from '../vimeoVideoPlayInModal/VimeoVideoPlayInModal';

			        type Props = {
			          videoUrl: string;
			          videoTitle: string;
			        };
			        const ResourceVideo: FC<Props> = ({
			          videoUrl,
			          videoTitle,
			        }: Props): JSX.Element => {
			          return (
			            <div className="resource-video">
			              <VimeoVideoPlayInModal videoUrl={videoUrl} />
			              <h3>{videoTitle}</h3>
			            </div>
			          );
			        };

			        export default ResourceVideo;
			      "#, None, None),
("
			        // testrule.jsx
			        const trackLink = () => {};
			        const getAnalyticsUiElement = () => {};

			        const onTextButtonClick = (e, item) => trackLink([, getAnalyticsUiElement(item), item.name], e);
			      ", None, None),
(r#"
			        function Component({ allRatings }) {
			          return (
			            <RatingDetailsStyles>
			              {Object.entries(allRatings)?.map(([key, value], index) => {
			                const rate = value?.split(/(?=[%, /])/);

			                if (!rate) return null;

			                return (
			                  <li key={`${entertainment.tmdbId}${index}`}>
			                    <img src={`/assets/rating/${key}.png`} />
			                    <span className="rating-details--rate">{rate?.[0]}</span>
			                    <span className="rating-details--rate-suffix">{rate?.[1]}</span>
			                  </li>
			                );
			              })}
			            </RatingDetailsStyles>
			          );
			        }
			      "#, None, None),
("
			        const baz = foo?.bar?.()?.[1] ?? 'qux';

			        qux()?.map()

			        const directiveRanges = comments?.map(tryParseTSDirective)
			      ", None, None),
(r#"
			        import { observable } from "mobx";

			        export interface ClusterFrameInfo {
			          frameId: number;
			          processId: number;
			        }

			        export const clusterFrameMap = observable.map<string, ClusterFrameInfo>();
			      "#, None, None),
("React.Children.toArray([1, 2 ,3].map(x => <App />));", None, None),
(r#"
			        import { Children } from "react";
			        Children.toArray([1, 2 ,3].map(x => <App />));
			      "#, None, None),
("
			        import Act from 'react';
			        import { Children as ReactChildren } from 'react';

			        const { Children } = Act;
			        const { toArray } = Children;

			        Act.Children.toArray([1, 2 ,3].map(x => <App />));
			        Act.Children.toArray(Array.from([1, 2 ,3], x => <App />));
			        Children.toArray([1, 2 ,3].map(x => <App />));
			        Children.toArray(Array.from([1, 2 ,3], x => <App />));
			        // ReactChildren.toArray([1, 2 ,3].map(x => <App />));
			        // ReactChildren.toArray(Array.from([1, 2 ,3], x => <App />));
			        // toArray([1, 2 ,3].map(x => <App />));
			        // toArray(Array.from([1, 2 ,3], x => <App />));
			      ", None, Some(serde_json::json!({ "settings": { "react": { "pragma": "Act", "fragment": "Frag" } }})))
    ];

    let fail = vec![
        ("[<App />];", None, None),
        ("[<App {...key} />];", None, None),
        ("[<App key={0}/>, <App />];", None, None),
        ("[1, 2 ,3].map(function(x) { return <App /> });", None, None),
        ("[1, 2 ,3].map(x => <App />);", None, None),
        ("[1, 2 ,3].map(x => x && <App x={x} />);", None, None),
        (r#"[1, 2 ,3].map(x => x ? <App x={x} key="1" /> : <OtherApp x={x} />);"#, None, None),
        (r#"[1, 2 ,3].map(x => x ? <App x={x} /> : <OtherApp x={x} key="2" />);"#, None, None),
        ("[1, 2 ,3].map(x => { return <App /> });", None, None),
        ("Array.from([1, 2 ,3], function(x) { return <App /> });", None, None),
        ("Array.from([1, 2 ,3], (x => { return <App /> }));", None, None),
        ("Array.from([1, 2 ,3], (x => <App />));", None, None),
        ("[1, 2, 3]?.map(x => <BabelEslintApp />)", None, None),
        ("[1, 2, 3]?.map(x => <TypescriptEslintApp />)", None, None),
        (
            "[1, 2, 3].map(x => <>{x}</>);",
            Some(serde_json::json!([{ "checkFragmentShorthand": true }])),
            Some(
                serde_json::json!({ "settings": { "react": { "pragma": "Act", "fragment": "Frag" } }}),
            ),
        ),
        (
            "[<></>];",
            Some(serde_json::json!([{ "checkFragmentShorthand": true }])),
            Some(
                serde_json::json!({ "settings": { "react": { "pragma": "Act", "fragment": "Frag" } }}),
            ),
        ),
        (
            r#"[<App {...obj} key="keyAfterSpread" />];"#,
            Some(serde_json::json!([{ "checkKeyMustBeforeSpread": true }])),
            Some(
                serde_json::json!({ "settings": { "react": { "pragma": "Act", "fragment": "Frag" } }}),
            ),
        ),
        (
            r#"[<div {...obj} key="keyAfterSpread" />];"#,
            Some(serde_json::json!([{ "checkKeyMustBeforeSpread": true }])),
            Some(
                serde_json::json!({ "settings": { "react": { "pragma": "Act", "fragment": "Frag" } }}),
            ),
        ),
        (
            r#"
			        const spans = [
			          <span key="notunique"/>,
			          <span key="notunique"/>,
			        ];
			      "#,
            Some(serde_json::json!([{ "warnOnDuplicates": true }])),
            None,
        ),
        (
            r#"
			        const div = (
			          <div>
			            <span key="notunique"/>
			            <span key="notunique"/>
			          </div>
			        );
			      "#,
            Some(serde_json::json!([{ "warnOnDuplicates": true }])),
            None,
        ),
        (
            "
			        const Test = () => {
			          const list = [1, 2, 3, 4, 5];

			          return (
			            <div>
			              {list.map(item => {
			                if (item < 2) {
			                  return <div>{item}</div>;
			                }

			                return <div />;
			              })}
			            </div>
			          );
			        };
			      ",
            None,
            None,
        ),
        (
            "
			        const TestO = () => {
			          const list = [1, 2, 3, 4, 5];

			          return (
			            <div>
			              {list.map(item => {
			                if (item < 2) {
			                  return <div>{item}</div>;
			                } else if (item < 5) {
			                  return <div></div>
			                }  else {
			                  return <div></div>
			                }

			                return <div />;
			              })}
			            </div>
			          );
			        };
			      ",
            None,
            None,
        ),
        (
            "
			        const TestCase = () => {
			          const list = [1, 2, 3, 4, 5];

			          return (
			            <div>
			              {list.map(item => {
			                if (item < 2) return <div>{item}</div>;
			                else if (item < 5) return <div />;
			                else return <div />;
			              })}
			            </div>
			          );
			        };
			      ",
            None,
            None,
        ),
        (
            "
			        const TestCase = () => {
			          const list = [1, 2, 3, 4, 5];

			          return (
			            <div>
			              {list.map(x => <div {...spread} key={x} />)}
			            </div>
			          );
			        };
			      ",
            Some(serde_json::json!([{ "checkKeyMustBeforeSpread": true }])),
            None,
        ),
    ];

    Tester::new(JsxKey::NAME, JsxKey::PLUGIN, pass, fail).test_and_snapshot();
}
