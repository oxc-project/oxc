use cow_utils::CowUtils;
use oxc_ast::{
    ast::{Expression, JSXAttributeItem, JSXAttributeName, JSXElement, JSXFragment, Statement},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
    AstNode,
};

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

#[derive(Debug, Default, Clone)]
pub struct JsxKey;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce `key` prop for elements in array
    ///
    /// ### Why is this bad?
    ///
    /// React requires a `key` prop for elements in an array to help identify which items have changed, are added, or are removed.
    ///
    /// ### Example
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
    correctness
);

impl Rule for JsxKey {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::JSXElement(jsx_elem) => {
                check_jsx_element(node, jsx_elem, ctx);
                check_jsx_element_is_key_before_spread(jsx_elem, ctx);
            }
            AstKind::JSXFragment(jsx_frag) => {
                check_jsx_fragment(node, jsx_frag, ctx);
            }

            _ => {}
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_jsx()
    }
}

pub fn is_to_array(node: &AstNode<'_>) -> bool {
    const TOARRAY: &str = "toArray";

    let AstKind::CallExpression(call) = node.kind() else { return false };

    let Some(subject) = call.callee_name() else { return false };

    if subject != TOARRAY {
        return false;
    }

    true
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
        && ctx.scopes().get_bindings(ctx.scopes().root_scope_id()).is_empty()
    {
        return actual_local_name == expected_local_name;
    }

    import_matcher(ctx, actual_local_name, expected_module_name)
}

pub fn is_children<'a, 'b>(node: &'b AstNode<'a>, ctx: &'b LintContext<'a>) -> bool {
    const REACT: &str = "React";
    const CHILDREN: &str = "Children";

    let AstKind::CallExpression(call) = node.kind() else { return false };

    let Some(member) = call.callee.as_member_expression() else { return false };

    if let Expression::Identifier(ident) = member.object() {
        return is_import(ctx, ident.name.as_str(), CHILDREN, REACT);
    }

    let Some(inner_member) = member.object().get_inner_expression().as_member_expression() else {
        return false;
    };

    let Some(ident) = inner_member.object().get_identifier_reference() else { return false };

    let Some(local_name) = inner_member.static_property_name() else { return false };

    is_import(ctx, ident.name.as_str(), REACT, REACT) && local_name == CHILDREN
}
fn is_within_children_to_array<'a, 'b>(node: &'b AstNode<'a>, ctx: &'b LintContext<'a>) -> bool {
    let parents_iter = ctx.nodes().ancestors(node.id()).skip(2);
    parents_iter
        .filter(|parent_node| matches!(parent_node.kind(), AstKind::CallExpression(_)))
        .any(|parent_node| is_children(parent_node, ctx) && is_to_array(parent_node))
}

enum InsideArrayOrIterator {
    Array,
    Iterator(Span),
}

fn is_in_array_or_iter<'a, 'b>(
    node: &'b AstNode<'a>,
    ctx: &'b LintContext<'a>,
) -> Option<InsideArrayOrIterator> {
    let mut node = node;

    let mut is_outside_containing_function = false;
    let mut is_explicit_return = false;

    loop {
        let parent = ctx.nodes().parent_node(node.id())?;
        match parent.kind() {
            AstKind::ArrowFunctionExpression(arrow_expr) => {
                let is_arrow_expr_statement = matches!(
                    arrow_expr.body.statements.first(),
                    Some(Statement::ExpressionStatement(_))
                );
                if !is_explicit_return && !is_arrow_expr_statement {
                    return None;
                }

                let parent = ctx.nodes().parent_node(parent.id())?;

                if let AstKind::ObjectProperty(_) = parent.kind() {
                    return None;
                }
                if is_outside_containing_function {
                    return None;
                }
                is_outside_containing_function = true;
            }
            AstKind::Function(_) => {
                let parent = ctx.nodes().parent_node(parent.id())?;

                if let AstKind::ObjectProperty(_) = parent.kind() {
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

                if let Some(v) = callee.as_member_expression() {
                    if let Some((span, ident)) = v.static_property_info() {
                        if TARGET_METHODS.contains(ident) {
                            return Some(InsideArrayOrIterator::Iterator(span));
                        }
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

    if let (Some((key_idx, key_span)), Some(spread_idx)) = (key_idx_span, spread_idx) {
        if key_idx > spread_idx {
            ctx.diagnostic(key_prop_must_be_placed_before_spread(key_span));
        }
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

const TARGET_METHODS: phf::Set<&'static str> = phf::phf_set! {
    // <array>.map(() => <jsx />)
    "map",
    // <array>.map(() => <jsx />)
    "flatMap",
    // Array.from(<array>, () => <jsx />)
    "from"
};

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"fn()",
        r"[1, 2, 3].map(function () {})",
        r"<App />;",
        r"[<App key={0} />, <App key={1} />];",
        r"[1, 2, 3].map(function(x) { return <App key={x} /> });",
        r"[1, 2, 3].map(x => <App key={x} />);",
        r"[1, 2 ,3].map(x => x && <App x={x} key={x} />);",
        r#"[1, 2 ,3].map(x => x ? <App x={x} key="1" /> : <OtherApp x={x} key="2" />);"#,
        r"[1, 2, 3].map(x => { return <App key={x} /> });",
        r"Array.from([1, 2, 3], function(x) { return <App key={x} /> });",
        r"Array.from([1, 2, 3], (x => <App key={x} />));",
        r"Array.from([1, 2, 3], (x => {return <App key={x} />}));",
        r"Array.from([1, 2, 3], someFn);",
        r"Array.from([1, 2, 3]);",
        r"[1, 2, 3].foo(x => <App />);",
        r"var App = () => <div />;",
        r"[1, 2, 3].map(function(x) { return; });",
        r"foo(() => <div />);",
        r"foo(() => <></>);",
        r"<></>;",
        r"<App {...{}} />;",
        r#"<App key="keyBeforeSpread" {...{}} />;"#,
        r#"<div key="keyBeforeSpread" {...{}} />;"#,
        r#"const spans = [<span key="notunique"/>,<span key="notunique"/>];"#,
        r#"
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
            "#,
        r#"
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
            "#,
        r"
            // testrule.jsx
            const trackLink = () => {};
            const getAnalyticsUiElement = () => {};

            const onTextButtonClick = (e, item) => trackLink([, getAnalyticsUiElement(item), item.name], e);
            ",
        r#"
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
              "#,
        r"
            const baz = foo?.bar?.()?.[1] ?? 'qux';

            qux()?.map()

            const directiveRanges = comments?.map(tryParseTSDirective)
            ",
        r#"
          const foo: (JSX.Element | string)[] = [
            "text",
            <Fragment key={1}>hello world<sup>superscript</sup></Fragment>,
          ];
        "#,
        r#"
            import { observable } from "mobx";

            export interface ClusterFrameInfo {
              frameId: number;
              processId: number;
            }

            export const clusterFrameMap = observable.map<string, ClusterFrameInfo>();
          "#,
        r#"
            const columns: ColumnDef<User>[] = [{
              accessorKey: 'lastName',
              header: ({ column }) => <DataTableColumnHeader column={column} title="Last Name" />,
              cell: ({ row }) => <div>{row.getValue('lastName')}</div>,
              enableSorting: true,
              enableHiding: false,
            }]
        "#,
        r#"
            const columns: ColumnDef<User>[] = [{
              accessorKey: 'lastName',
              header: function ({ column }) { return <DataTableColumnHeader column={column} title="Last Name" /> },
              cell: ({ row }) => <div>{row.getValue('lastName')}</div>,
              enableSorting: true,
              enableHiding: false,
            }]
        "#,
        r#"
            const router = createBrowserRouter([
              {
                path: "/",
                element: <Root />,
                children: [
                  {
                    path: "team",
                    element: <Team />,
                  },
                ],
              },
            ]);
        "#,
        r#"
        function App() {
          return (
            <div className="App">
              {[1, 2, 3, 4, 5].map((val) => {
                const text = () => <strong>{val}</strong>;
                return null
              })}
            </div>
          );
        }"#,
        r#"
        function App() {
          return (
            <div className="App">
              {[1, 2, 3, 4, 5].map((val) => {
                const text = <strong>{val}</strong>;
                return <button key={val}>{text}</button>;
              })}
            </div>
          );
        }"#,
        r"
        MyStory.decorators = [
          (Component) => <div><Component /></div>
        ];
        ",
        r"
        MyStory.decorators = [
          (Component) => {
            const store = useMyStore();
            return <Provider store={store}><Component /></Provider>;
          }
        ];
        ",
        r"{React.Children.toArray(items.map((item) => {
            return (
              <div>
             {item}
             </div>
              );}))}
        ",
        r#"import { Children } from "react";
        Children.toArray([1, 2 ,3].map(x => <App />));
        "#,
        r#"import React from "react";
        React.Children.toArray([1, 2 ,3].map(x => <App />));
        "#,
        r"React.Children.toArray([1, 2 ,3].map(x => <App />));",
        r"{React.Children.toArray(items.map((item) => {
           return (
             <>
              {item}
             </>
            );
           }))}
        ",
    ];

    let fail = vec![
        r"[<App />];",
        r"[<App {...key} />];",
        r"[<App key={0}/>, <App />];",
        r"[1, 2 ,3].map(function(x) { return <App /> });",
        r"[1, 2 ,3].map(x => <App />);",
        r"[1, 2 ,3].map(x => x && <App x={x} />);",
        r#"[1, 2 ,3].map(x => x ? <App x={x} key="1" /> : <OtherApp x={x} />);"#,
        r#"[1, 2 ,3].map(x => x ? <App x={x} /> : <OtherApp x={x} key="2" />);"#,
        r"[1, 2 ,3].map(x => { return <App /> });",
        r"Array.from([1, 2 ,3], function(x) { return <App /> });",
        r"Array.from([1, 2 ,3], (x => { return <App /> }));",
        r"Array.from([1, 2 ,3], (x => <App />));",
        r"[1, 2, 3]?.map(x => <BabelEslintApp />)",
        r"[1, 2, 3]?.map(x => <TypescriptEslintApp />)",
        r"[1, 2, 3]?.map(x => <><OxcCompilerHello /></>)",
        "[1, 2, 3].map(x => <>{x}</>);",
        "[<></>];",
        r#"[<App {...obj} key="keyAfterSpread" />];"#,
        r#"[<div {...obj} key="keyAfterSpread" />];"#,
        r"
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
        r"
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
        r"
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
        r"
                const TestCase = () => {
                  const list = [1, 2, 3, 4, 5];

                  return (
                    <div>
                      {list.map(item => <Text foo bar baz qux onClick={() => onClickHandler()} onPointerDown={() => onPointerDownHandler()} onMouseDown={() => onMouseDownHandler()} />)}
                    </div>
                  );
                };
          ",
        r"
                const TestCase = () => {
                  const list = [1, 2, 3, 4, 5];

                  return (
                    <div>
                      {list.map(item => (<div>
                        <Text foo bar baz qux onClick={() => onClickHandler()} onPointerDown={() => onPointerDownHandler()} onMouseDown={() => onMouseDownHandler()} />
                        </div>)
                      )}
                    </div>
                  );
                };
          ",
        r"foo.Children.toArray([1, 2 ,3].map(x => <App />));",
        r"
        import Act from 'react';
        import { Children as ReactChildren } from 'react';

        const { Children } = Act;
        const { toArray } = Children;

        Act.Children.toArray([1, 2 ,3].map(x => <App />));
        Act.Children.toArray(Array.from([1, 2 ,3], x => <App />));
        Children.toArray([1, 2 ,3].map(x => <App />));
        Children.toArray(Array.from([1, 2 ,3], x => <App />));
        ",
    ];

    Tester::new(JsxKey::NAME, JsxKey::PLUGIN, pass, fail).test_and_snapshot();
}
