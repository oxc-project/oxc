use oxc_ast::{
    ast::{Expression, JSXAttributeItem, JSXAttributeName, JSXElement, JSXFragment, Statement},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_span::{GetSpan, Span};

use oxc_macros::declare_oxc_lint;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
enum JsxKeyDiagnostic {
    #[error(r#"eslint-plugin-react(jsx-key): Missing "key" prop for element in array."#)]
    #[diagnostic(severity(warning))]
    MissingKeyPropForElementInArray(#[label] Span),

    #[error(r#"eslint-plugin-react(jsx-key): Missing "key" prop for element in iterator."#)]
    #[diagnostic(severity(warning), help(r#"Add a "key" prop to the element in the iterator (https://react.dev/learn/rendering-lists#keeping-list-items-in-order-with-key)."#))]
    MissingKeyPropForElementInIterator(
        #[label("Iterator starts here")] Span,
        #[label("Element generated here")] Span,
    ),

    #[error(
        r#"eslint-plugin-react(jsx-key): "key" prop must be placed before any `{{...spread}}`"#
    )]
    #[diagnostic(severity(warning), help("To avoid conflicting with React's new JSX transform: https://reactjs.org/blog/2020/09/22/introducing-the-new-jsx-transform.html"))]
    KeyPropMustBePlacedBeforeSpread(#[label] Span),
}

#[derive(Debug, Default, Clone)]
pub struct JsxKey;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce `key` prop for elements in array
    ///
    /// ### Example
    /// ```javascript
    /// // Bad
    /// [1, 2, 3].map(x => <App />);
    /// [1, 2, 3]?.map(x => <BabelEslintApp />)
    ///
    /// // Good
    /// [1, 2, 3].map(x => <App key={x} />);
    /// [1, 2, 3]?.map(x => <BabelEslintApp key={x} />)
    /// ```
    JsxKey,
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
    let mut is_arrow_expr_statement = false;

    loop {
        let Some(parent) = ctx.nodes().parent_node(node.id()) else {
            return None;
        };

        match parent.kind() {
            AstKind::ArrowExpression(arrow_expr) => {
                is_arrow_expr_statement = matches!(
                    arrow_expr.body.statements.first(),
                    Some(Statement::ExpressionStatement(_))
                );
                if !is_explicit_return && !is_arrow_expr_statement {
                    return None;
                }

                let Some(parent) = ctx.nodes().parent_node(parent.id()) else {
                    return None;
                };

                if let AstKind::ObjectProperty(_) = parent.kind() {
                    return None;
                }
                if is_outside_containing_function {
                    return None;
                }
                is_outside_containing_function = true;
            }
            AstKind::Function(_) => {
                let Some(parent) = ctx.nodes().parent_node(parent.id()) else {
                    return None;
                };

                if let AstKind::ObjectProperty(_) = parent.kind() {
                    return None;
                }
                if is_outside_containing_function {
                    return None;
                }
                is_outside_containing_function = true;
            }
            AstKind::ArrayExpression(_) => {
                if is_arrow_expr_statement {
                    return None;
                }

                return Some(InsideArrayOrIterator::Array);
            }
            AstKind::CallExpression(v) => {
                let callee = &v.callee.without_parenthesized();

                if let Expression::MemberExpression(v) = callee {
                    if let Some((span, ident)) = v.static_property_info() {
                        if TARGET_METHODS.contains(ident) {
                            return Some(InsideArrayOrIterator::Iterator(span));
                        }
                    }
                }

                return None;
            }
            AstKind::JSXElement(_) | AstKind::JSXOpeningElement(_) | AstKind::ObjectProperty(_) => {
                return None
            }
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
        if !jsx_elem.opening_element.attributes.iter().any(|attr| {
            let JSXAttributeItem::Attribute(attr) = attr else { return false };

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
                let JSXAttributeName::Identifier(ident) = &attr.name else { continue };
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
            ctx.diagnostic(JsxKeyDiagnostic::KeyPropMustBePlacedBeforeSpread(key_span));
        }
    }
}

fn check_jsx_fragment<'a>(node: &AstNode<'a>, fragment: &JSXFragment<'a>, ctx: &LintContext<'a>) {
    if let Some(outer) = is_in_array_or_iter(node, ctx) {
        ctx.diagnostic(gen_diagnostic(fragment.opening_fragment.span, &outer));
    }
}

fn gen_diagnostic(span: Span, outer: &InsideArrayOrIterator) -> JsxKeyDiagnostic {
    match outer {
        InsideArrayOrIterator::Array => JsxKeyDiagnostic::MissingKeyPropForElementInArray(span),
        InsideArrayOrIterator::Iterator(v) => {
            JsxKeyDiagnostic::MissingKeyPropForElementInIterator(*v, span)
        }
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
    ];

    Tester::new_without_config(JsxKey::NAME, pass, fail).test_and_snapshot();
}
