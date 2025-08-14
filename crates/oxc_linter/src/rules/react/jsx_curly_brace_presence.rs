use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    rule::Rule,
};
use lazy_regex::{Lazy, Regex, lazy_regex};
use oxc_allocator::{Allocator, Vec};

use oxc_ast::{
    AstBuilder, AstKind,
    ast::{
        Expression, JSXAttributeItem, JSXAttributeValue, JSXChild, JSXElementName, JSXExpression,
        JSXExpressionContainer,
    },
};
use oxc_codegen::CodegenOptions;
use oxc_diagnostics::{Error, LabeledSpan, OxcDiagnostic};
use oxc_macros::declare_oxc_lint;
use oxc_semantic::NodeId;
use oxc_span::{GetSpan as _, Span};
use serde_json::Value;

fn jsx_curly_brace_presence_unnecessary_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Curly braces are unnecessary here.").with_label(span)
}
fn jsx_curly_brace_presence_necessary_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Curly braces are required here.")
        .with_help("Wrap this value in curly braces")
        .with_labels([LabeledSpan::new_primary_with_span(
            Some("Wrap this value in curly braces".into()),
            span,
        )])
}

#[derive(Debug, Default, Clone, Copy)]
enum Allowed {
    Always,
    Never,
    #[default]
    Ignore,
}

impl TryFrom<&str> for Allowed {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "always" => Ok(Self::Always),
            "never" => Ok(Self::Never),
            "ignore" => Ok(Self::Ignore),
            _ => Err(()),
        }
    }
}

impl Allowed {
    pub fn is_never(self) -> bool {
        matches!(self, Self::Never)
    }

    #[inline]
    pub fn is_always(self) -> bool {
        matches!(self, Self::Always)
    }
}

#[derive(Debug, Clone)]
pub struct JsxCurlyBracePresence {
    props: Allowed,
    children: Allowed,
    prop_element_values: Allowed,
}

impl Default for JsxCurlyBracePresence {
    fn default() -> Self {
        Self {
            props: Allowed::Never,
            children: Allowed::Never,
            prop_element_values: Allowed::Ignore,
        }
    }
}

declare_oxc_lint!(
    /// # Disallow unnecessary JSX expressions when literals alone are
    /// sufficient or enforce JSX expressions on literals in JSX children or
    /// attributes (`react/jsx-curly-brace-presence`)
    ///
    /// 🔧 This rule is automatically fixable by the [`--fix` CLI option](https://oxc-project.github.io/docs/guide/usage/linter/cli.html#fix-problems).
    ///
    /// This rule allows you to enforce curly braces or disallow unnecessary
    /// curly braces in JSX props and/or children.
    ///
    /// For situations where JSX expressions are unnecessary, please refer to
    /// [the React doc](https://facebook.github.io/react/docs/jsx-in-depth.html)
    /// and [this page about JSX
    /// gotchas](https://github.com/facebook/react/blob/v15.4.0-rc.3/docs/docs/02.3-jsx-gotchas.md#html-entities).
    ///
    /// ## Rule Details
    ///
    /// By default, this rule will check for and warn about unnecessary curly
    /// braces in both JSX props and children. For the sake of backwards
    /// compatibility, prop values that are JSX elements are not considered by
    /// default.
    ///
    /// You can pass in options to enforce the presence of curly braces on JSX
    /// props, children, JSX prop values that are JSX elements, or any
    /// combination of the three. The same options are available for not
    /// allowing unnecessary curly braces as well as ignoring the check.
    ///
    /// **Note**: it is _highly recommended_ that you configure this rule with
    /// an object, and that you set "propElementValues" to "always". The ability
    /// to omit curly braces around prop values that are JSX elements is
    /// obscure, and intentionally undocumented, and should not be relied upon.
    ///
    /// ## Rule Options
    ///
    /// ```js
    /// ...
    /// "react/jsx-curly-brace-presence": [<enabled>, { "props": <string>, "children": <string>, "propElementValues": <string> }]
    /// ...
    /// ```
    ///
    /// or alternatively
    ///
    /// ```js
    /// ...
    /// "react/jsx-curly-brace-presence": [<enabled>, <string>]
    /// ...
    /// ```
    ///
    /// ### Valid options for `<string>`
    ///
    /// They are `always`, `never` and `ignore` for checking on JSX props and
    /// children.
    ///
    /// - `always`: always enforce curly braces inside JSX props, children, and/or JSX prop values that are JSX Elements
    /// - `never`: never allow unnecessary curly braces inside JSX props, children, and/or JSX prop values that are JSX Elements
    /// - `ignore`: ignore the rule for JSX props, children, and/or JSX prop values that are JSX Elements
    ///
    /// If passed in the option to fix, this is how a style violation will get fixed
    ///
    /// - `always`: wrap a JSX attribute in curly braces/JSX expression and/or a JSX child the same way but also with double quotes
    /// - `never`: get rid of curly braces from a JSX attribute and/or a JSX child
    ///
    /// - All fixing operations use double quotes.
    ///
    /// For examples:
    ///
    /// Examples of **incorrect** code for this rule, when configured with `{ props: "always", children: "always" }`:
    ///
    /// ```jsx
    /// <App>Hello world</App>;
    /// <App prop='Hello world'>{'Hello world'}</App>;
    /// ```
    ///
    /// They can be fixed to:
    ///
    /// ```jsx
    /// <App>{"Hello world"}</App>;
    /// <App prop={"Hello world"}>{'Hello world'}</App>;
    /// ```
    ///
    /// Examples of **incorrect** code for this rule, when configured with `{ props: "never", children: "never" }`:
    ///
    /// ```jsx
    /// <App>{'Hello world'}</App>;
    /// <App prop={'Hello world'} attr={"foo"} />;
    /// ```
    ///
    /// They can be fixed to:
    ///
    /// ```jsx
    /// <App>Hello world</App>;
    /// <App prop="Hello world" attr="foo" />;
    /// ```
    ///
    /// Examples of **incorrect** code for this rule, when configured with `{ props: "always", children: "always", "propElementValues": "always" }`:
    ///
    /// ```jsx
    /// <App prop=<div /> />;
    /// ```
    ///
    /// They can be fixed to:
    ///
    /// ```jsx
    /// <App prop={<div />} />;
    /// ```
    ///
    /// Examples of **incorrect** code for this rule, when configured with `{ props: "never", children: "never", "propElementValues": "never" }`:
    ///
    /// ```jsx
    /// <App prop={<div />} />;
    /// ```
    ///
    /// They can be fixed to:
    ///
    /// ```jsx
    /// <App prop=<div /> />;
    /// ```
    ///
    /// ### Alternative syntax
    ///
    /// The options are also `always`, `never`, and `ignore` for the same meanings.
    ///
    /// In this syntax, only a string is provided and the default will be set to
    /// that option for checking on both JSX props and children.
    ///
    /// For examples:
    ///
    /// Examples of **incorrect** code for this rule, when configured with `"always"`:
    ///
    /// ```jsx
    /// <App>Hello world</App>;
    /// <App prop='Hello world' attr="foo">Hello world</App>;
    /// ```
    ///
    /// They can be fixed to:
    ///
    /// ```jsx
    /// <App>{"Hello world"}</App>;
    /// <App prop={"Hello world"} attr={"foo"}>{"Hello world"}</App>;
    /// ```
    ///
    /// Examples of **incorrect** code for this rule, when configured with `"never"`:
    ///
    /// ```jsx
    /// <App prop={'foo'} attr={"bar"}>{'Hello world'}</App>;
    /// ```
    ///
    /// It can fixed to:
    ///
    /// ```jsx
    /// <App prop="foo" attr="bar">Hello world</App>;
    /// ```
    ///
    /// ## Edge cases
    ///
    /// The fix also deals with template literals, strings with quotes, and
    /// strings with escapes characters.
    ///
    /// - If the rule is set to get rid of unnecessary curly braces and the
    ///   template literal inside a JSX expression has no expression, it will
    ///   throw a warning and be fixed with double quotes. For example:
    ///
    /// ```jsx
    /// <App prop={`Hello world`}>{`Hello world`}</App>;
    /// ```
    ///
    /// will be warned and fixed to:
    ///
    /// ```jsx
    /// <App prop="Hello world">Hello world</App>;
    /// ```
    ///
    /// - If the rule is set to enforce curly braces and the strings have
    ///   quotes, it will be fixed with double quotes for JSX children and the
    ///   normal way for JSX attributes. Also, double quotes will be escaped in
    ///   the fix.
    ///
    /// For example:
    ///
    /// ```jsx
    /// <App prop='Hello "foo" world'>Hello 'foo' "bar" world</App>;
    /// ```
    ///
    /// will warned and fixed to:
    ///
    /// ```jsx
    /// <App prop={"Hello \"foo\" world"}>{"Hello 'foo' \"bar\" world"}</App>;
    /// ```
    ///
    /// - If the rule is set to get rid of unnecessary curly braces(JSX
    ///   expression) and there are characters that need to be escaped in its JSX
    ///   form, such as quote characters, [forbidden JSX text
    ///   characters](https://facebook.github.io/jsx/), escaped characters and
    ///   anything that looks like HTML entity names, the code will not be warned
    ///   because the fix may make the code less readable.
    ///
    /// Examples of **correct** code for this rule, even when configured with `"never"`:
    ///
    /// ```jsx
    /// <Color text={"\u00a0"} />
    /// <App>{"Hello \u00b7 world"}</App>;
    /// <style type="text/css">{'.main { margin-top: 0; }'}</style>;
    /// /**
    ///  * there's no way to inject a whitespace into jsx without a container so this
    ///  * will always be allowed.
    ///  */
    /// <App>{' '}</App>
    /// <App>{'     '}</App>
    /// <App>{/* comment */ <Bpp />}</App> // the comment makes the container necessary
    /// ```
    ///
    /// ## When Not To Use It
    ///
    /// You should turn this rule off if you are not concerned about maintaining
    /// consistency regarding the use of curly braces in JSX props and/or
    /// children as well as the use of unnecessary JSX expressions.
    JsxCurlyBracePresence,
    react,
    style,
    fix
);

impl Rule for JsxCurlyBracePresence {
    fn from_configuration(value: Value) -> Self {
        let default = Self::default();
        let value = match value.as_array() {
            Some(arr) => &arr[0],
            _ => &value,
        };
        match value {
            Value::String(s) => {
                let allowed = Allowed::try_from(s.as_str())
				.map_err(|()| Error::msg(
					r#"Invalid string config for eslint-plugin-react/jsx-curly-brace-presence: only "always", "never", or "ignored" are allowed. "#
				)).unwrap();
                Self { props: allowed, children: allowed, prop_element_values: allowed }
            }
            Value::Object(obj) => {
                let props = obj
                    .get("props")
                    .and_then(Value::as_str)
                    .and_then(|props| Allowed::try_from(props).ok())
                    .unwrap_or(default.props);
                let children = obj
                    .get("children")
                    .and_then(Value::as_str)
                    .and_then(|children| Allowed::try_from(children).ok())
                    .unwrap_or(default.children);
                let prop_element_values = obj
                    .get("propElementValues")
                    .and_then(Value::as_str)
                    .and_then(|prop_element_values| Allowed::try_from(prop_element_values).ok())
                    .unwrap_or(default.prop_element_values);

                Self { props, children, prop_element_values }
            }
            _ => default,
        }
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::JSXElement(el) => {
                el.opening_element.attributes.iter().for_each(|attr| {
                    self.check_jsx_attribute(ctx, attr, node);
                });
                if self.children.is_never()
                    && matches!(&el.opening_element.name, JSXElementName::Identifier(ident) if ident.name == "script")
                {
                    return;
                }
                self.check_jsx_child(ctx, &el.children, node);
            }
            AstKind::JSXFragment(fragment) => {
                self.check_jsx_child(ctx, &fragment.children, node);
            }
            _ => {}
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_jsx()
    }
}

impl JsxCurlyBracePresence {
    fn check_jsx_child<'a>(
        &self,
        ctx: &LintContext<'a>,
        children: &Vec<'a, JSXChild<'a>>,
        node: &AstNode<'a>,
    ) {
        for child in children {
            match child {
                JSXChild::ExpressionContainer(container) => {
                    self.check_expression_container(ctx, container, node, false);
                }
                JSXChild::Text(text) => {
                    let text_value = text.value.as_str();
                    if self.children.is_always()
                        && !contains_only_html_entities(text_value)
                        && !is_whitespace(text_value)
                    {
                        report_missing_curly_for_text_node(ctx, text.span, text_value);
                    }
                }
                _ => {}
            }
        }
    }

    fn check_jsx_attribute<'a>(
        &self,
        ctx: &LintContext<'a>,
        attr: &JSXAttributeItem<'a>,
        node: &AstNode<'a>,
    ) {
        let JSXAttributeItem::Attribute(attr) = attr else {
            return;
        };
        let Some(value) = attr.value.as_ref() else { return };

        match value {
            JSXAttributeValue::ExpressionContainer(container) => {
                self.check_expression_container(ctx, container, node, true);
            }
            JSXAttributeValue::Element(el) => {
                if self.prop_element_values.is_always() {
                    report_missing_curly_for_expression(ctx, el.span);
                }
            }
            JSXAttributeValue::Fragment(fragment) => {
                if self.prop_element_values.is_always() {
                    report_missing_curly_for_expression(ctx, fragment.span);
                }
            }
            JSXAttributeValue::StringLiteral(string) => {
                if self.props.is_always() {
                    report_missing_curly_for_string_attribute_value(
                        ctx,
                        string.span,
                        string.value.as_str(),
                    );
                }
            }
        }
    }

    fn check_expression_container<'a>(
        &self,
        ctx: &LintContext<'a>,
        container: &JSXExpressionContainer<'a>,
        node: &AstNode<'a>,
        // true for JSX props, false for JSX children
        parent_is_attribute: bool,
    ) {
        let Some(inner) = container.expression.as_expression() else { return };
        let allowed = if parent_is_attribute { self.props } else { self.children };
        match inner {
            Expression::JSXFragment(_) => {
                if !parent_is_attribute
                    && self.children.is_never()
                    && !has_adjacent_jsx_expression_containers(ctx, container, node.id())
                {
                    report_unnecessary_curly(ctx, container, inner.span());
                }
            }
            Expression::JSXElement(el) => {
                if parent_is_attribute {
                    if self.prop_element_values.is_never() && el.closing_element.is_none() {
                        report_unnecessary_curly_for_attribute_value(ctx, container, inner.span());
                    }
                } else if self.children.is_never()
                    && !has_adjacent_jsx_expression_containers(ctx, container, node.id())
                {
                    report_unnecessary_curly(ctx, container, inner.span());
                }
            }
            Expression::StringLiteral(string) => {
                if allowed.is_never() {
                    let raw = ctx.source_range(string.span().shrink_left(1).shrink_right(1));
                    if is_allowed_string_like_in_container(
                        ctx,
                        raw,
                        container,
                        node.id(),
                        parent_is_attribute,
                    ) {
                        return;
                    }
                    if parent_is_attribute {
                        report_unnecessary_curly_for_attribute_value(ctx, container, string.span);
                    } else {
                        report_unnecessary_curly(ctx, container, string.span);
                    }
                }
            }
            Expression::TemplateLiteral(template) => {
                if allowed.is_never() && template.is_no_substitution_template() {
                    let string = template.single_quasi().unwrap();
                    if !parent_is_attribute && contains_quote_characters(string.as_str())
                        || is_allowed_string_like_in_container(
                            ctx,
                            string.as_str(),
                            container,
                            node.id(),
                            parent_is_attribute,
                        )
                    {
                        return;
                    }
                    if parent_is_attribute {
                        report_unnecessary_curly_for_attribute_value(ctx, container, template.span);
                    } else {
                        report_unnecessary_curly(ctx, container, template.span);
                    }
                }
            }
            _ => {}
        }
    }
}

fn is_allowed_string_like_in_container<'a>(
    ctx: &LintContext<'a>,
    s: &'a str,
    container: &JSXExpressionContainer<'a>,
    node_id: NodeId,
    is_prop: bool,
) -> bool {
    is_whitespace(s)
        || contains_line_break_or_is_empty(s)
        || contains_html_entity(s)
        || !is_prop && contains_disallowed_jsx_text_chars(s)
        || !is_prop && s.trim() != s
        || contains_multiline_comment(s)
        || contains_line_break_literal(s)
        || contains_utf8_escape(s)
        || has_adjacent_jsx_expression_containers(ctx, container, node_id)
}

fn is_whitespace(s: &str) -> bool {
    s.chars().all(char::is_whitespace)
}

fn contains_line_break_or_is_empty(s: &str) -> bool {
    s.chars().any(|c| matches!(c, '\n' | '\r')) || s.trim().is_empty()
}

fn contains_line_break_literal(s: &str) -> bool {
    s.chars().zip(s.chars().skip(1)).any(|tuple| matches!(tuple, ('\\', 'n' | 'r')))
}

fn contains_disallowed_jsx_text_chars(s: &str) -> bool {
    s.chars().any(|c| matches!(c, '<' | '>' | '{' | '}' | '\\'))
}

fn contains_multiline_comment(s: &str) -> bool {
    s.contains("/*") || s.contains("*/")
}

fn contains_quote_characters(s: &str) -> bool {
    s.chars().any(|c| matches!(c, '"' | '\''))
}

fn contains_double_quote_characters(s: &str) -> bool {
    s.chars().any(|c| matches!(c, '"'))
}

fn contains_utf8_escape(s: &str) -> bool {
    s.chars().zip(s.chars().skip(1)).any(|tuple| matches!(tuple, ('\\', 'u')))
}

pub static HTML_ENTITY_REGEX: Lazy<Regex> = lazy_regex!(r#"(&[A-Za-z\d#]+;)"#);

fn contains_html_entity(s: &str) -> bool {
    HTML_ENTITY_REGEX.is_match(s)
}

fn contains_only_html_entities(s: &str) -> bool {
    HTML_ENTITY_REGEX.replace_all(s, "").trim().is_empty()
}

fn report_unnecessary_curly<'a>(
    ctx: &LintContext<'a>,
    container: &JSXExpressionContainer<'a>,
    inner_span: Span,
) {
    ctx.diagnostic_with_fix(jsx_curly_brace_presence_unnecessary_diagnostic(inner_span), |fixer| {
        match &container.expression {
            JSXExpression::TemplateLiteral(template_lit) => {
                let mut fix = fixer.codegen();
                fix.print_str(template_lit.single_quasi().unwrap().as_str());

                fixer.replace(container.span, fix)
            }
            JSXExpression::StringLiteral(string_literal) => {
                let mut fix = fixer.codegen();
                fix.print_str(string_literal.value.as_str());

                fixer.replace(container.span, fix)
            }
            _ => {
                let mut fix = fixer.new_fix_with_capacity(2);

                fix.push(fixer.delete_range(Span::sized(container.span.start, 1)));
                fix.push(fixer.delete_range(Span::sized(container.span.end - 1, 1)));

                fix.with_message("remove the curly braces")
            }
        }
    });
}

fn report_unnecessary_curly_for_attribute_value<'a>(
    ctx: &LintContext<'a>,
    container: &JSXExpressionContainer<'a>,
    inner_span: Span,
) {
    ctx.diagnostic_with_fix(jsx_curly_brace_presence_unnecessary_diagnostic(inner_span), |fixer| {
        let alloc = Allocator::default();
        let ast_builder = AstBuilder::new(&alloc);

        let str = match &container.expression {
            JSXExpression::TemplateLiteral(template_lit) => template_lit.single_quasi().unwrap(),
            JSXExpression::StringLiteral(string_lit) => string_lit.value,
            JSXExpression::JSXElement(el) => {
                return fixer.replace(container.span, ctx.source_range(el.span));
            }
            _ => unreachable!(),
        };

        let mut fix = fixer.codegen();

        if !contains_double_quote_characters(str.as_str()) {
            fix = fix.with_options(CodegenOptions::default());
        }

        fix.print_expression(&ast_builder.expression_string_literal(
            Span::default(),
            str.as_str(),
            None,
        ));

        fixer.replace(container.span, fix)
    });
}

fn report_missing_curly_for_expression(ctx: &LintContext, span: Span) {
    ctx.diagnostic_with_fix(jsx_curly_brace_presence_necessary_diagnostic(span), |fixer| {
        let mut fix = fixer.new_fix_with_capacity(2);
        fix.push(fixer.insert_text_before(&span, "{"));
        fix.push(fixer.insert_text_after(&span, "}"));
        fix.with_message("add the missing curly braces")
    });
}

fn report_missing_curly_for_string_attribute_value(
    ctx: &LintContext,
    span: Span,
    string_value: &str,
) {
    ctx.diagnostic_with_fix(jsx_curly_brace_presence_necessary_diagnostic(span), |fixer| {
        let mut replace = fixer.codegen().with_options(CodegenOptions::default());

        let alloc = Allocator::default();
        let ast_builder = AstBuilder::new(&alloc);

        replace.print_expression(&ast_builder.expression_string_literal(
            Span::default(),
            string_value,
            None,
        ));

        let mut fix = fixer.new_fix_with_capacity(3);
        fix.push(fixer.insert_text_before(&span, "{"));
        fix.push(fixer.insert_text_after(&span, "}"));
        fix.push(fixer.replace(span, replace));
        fix.with_message("add the missing curly braces")
    });
}
fn report_missing_curly_for_text_node(ctx: &LintContext, span: Span, string_value: &str) {
    ctx.diagnostic_with_fix(jsx_curly_brace_presence_necessary_diagnostic(span), |fixer| {
        let fixer = fixer.for_multifix();
        let alloc = Allocator::default();
        let ast_builder = AstBuilder::new(&alloc);
        let line_matches =
            string_value.match_indices('\n').map(|(i, _)| i).collect::<std::vec::Vec<_>>();
        let fix_contexts = if line_matches.is_empty() {
            build_missing_curly_fix_context_for_part(span, string_value, 0)
                .iter()
                .copied()
                .collect::<std::vec::Vec<_>>()
        } else {
            string_value
                .split('\n')
                .enumerate()
                .flat_map(|(index, line)| {
                    let line_start = calculate_line_start(line_matches.as_slice(), index);
                    build_missing_curly_fix_context_for_line(span, line, line_start)
                })
                .collect::<std::vec::Vec<_>>()
        };
        if fix_contexts.is_empty() {
            return fixer.noop();
        }
        let mut fix = fixer.new_fix_with_capacity(fix_contexts.len() * 3);
        for (span_from_first_char, text) in fix_contexts {
            let mut replace = fixer.codegen().with_options(CodegenOptions::default());
            replace.print_expression(&ast_builder.expression_string_literal(
                Span::default(),
                text,
                None,
            ));
            fix.push(fixer.replace(span_from_first_char, replace));
            fix.push(fixer.insert_text_before(&span_from_first_char, "{"));
            fix.push(fixer.insert_text_after(&span_from_first_char, "}"));
        }
        fix.with_message("add the missing curly braces")
    });
}

fn build_missing_curly_fix_context_for_line(
    span: Span,
    line: &str,
    line_start: u32,
) -> std::vec::Vec<(Span, &str)> {
    let html_entities =
        HTML_ENTITY_REGEX.find_iter(line).map(|mat| mat.end()).collect::<std::vec::Vec<_>>();
    HTML_ENTITY_REGEX
        .split(line)
        .enumerate()
        .filter_map(|(index, part)| {
            let part_start = calculate_part_start(html_entities.as_slice(), index);
            build_missing_curly_fix_context_for_part(span, part, line_start + part_start)
        })
        .collect::<std::vec::Vec<_>>()
}

fn build_missing_curly_fix_context_for_part(
    span: Span,
    part: &str,
    part_start: u32,
) -> Option<(Span, &str)> {
    part.char_indices().find(|(_, ch)| !ch.is_whitespace()).map(|(first_char_index, _)| {
        let text = part.split_at(first_char_index).1;
        let new_start = span.start + part_start + u32::try_from(first_char_index).unwrap();
        let span_from_first_char =
            Span::new(new_start, new_start + u32::try_from(text.len()).unwrap());
        (span_from_first_char, text)
    })
}

fn calculate_line_start(line_matches: &[usize], index: usize) -> u32 {
    if index == 0 {
        0u32
    } else {
        u32::try_from(
            line_matches.get(index - 1).map_or(1usize, |new_line_index| *new_line_index + 1),
        )
        .unwrap()
    }
}

fn calculate_part_start(line_matches: &[usize], index: usize) -> u32 {
    if index == 0 {
        0u32
    } else {
        u32::try_from(
            line_matches.get(index - 1).copied().map_or(0usize, |new_line_index| new_line_index),
        )
        .unwrap()
    }
}

fn has_adjacent_jsx_expression_containers<'a>(
    ctx: &LintContext<'a>,
    container: &JSXExpressionContainer<'a>,
    node_id: NodeId,
    // element: &JSXElement<'a>,
) -> bool {
    let parent = ctx.nodes().parent_kind(node_id);
    let children = match parent {
        AstKind::JSXElement(el) => &el.children,
        AstKind::JSXFragment(fragment) => &fragment.children,
        AstKind::ExpressionStatement(expr) => match &expr.expression {
            Expression::JSXElement(el) => &el.children,
            Expression::JSXFragment(fragment) => &fragment.children,
            _ => {
                return false;
            }
        },
        _ => {
            return false;
        }
    };
    let Some(this_container_idx) = children.iter().position(|child| child.span() == container.span)
    else {
        return false;
    };

    [this_container_idx.checked_sub(1), this_container_idx.checked_add(1)]
        .into_iter()
        // [prev id, next id] -> [prev node, next node], removing out-of-bounds indices
        .filter_map(|idx| idx.and_then(|idx| children.get(idx)))
        .any(JSXChild::is_expression_container)
}

#[test]
fn test() {
    use serde_json::json;

    use crate::tester::Tester;

    let pass = vec![
        ("<App {...props}>foo</App>", None),
        ("<>foo</>", None),
        ("<App {...props}>foo</App>", Some(json!([{ "props": "never" }]))),
        ("<App>{' '}</App>", None),
        (
            "<App>{' '}
			</App>",
            None,
        ),
        ("<App>{'     '}</App>", None),
        (
            "<App>{'     '}
			</App>",
            None,
        ),
        ("<App>{' '}</App>", Some(json!([{ "children": "never" }]))),
        ("<App>{'    '}</App>", Some(json!([{ "children": "never" }]))),
        ("<App>{' '}</App>", Some(json!([{ "children": "always" }]))),
        ("<App>{'        '}</App>", Some(json!([{ "children": "always" }]))),
        ("<App {...props}>foo</App>", Some(json!([{ "props": "always" }]))),
        ("<App>{`Hello ${word} World`}</App>", Some(json!([{ "children": "never" }]))),
        (
            "
			        <React.Fragment>
			          foo{' '}
			          <span>bar</span>
			        </React.Fragment>
			      ",
            Some(json!([{ "children": "never" }])),
        ),
        (
            "
			        <>
			          foo{' '}
			          <span>bar</span>
			        </>
			      ",
            Some(json!([{ "children": "never" }])),
        ),
        ("<App>{`Hello \n World`}</App>", Some(json!([{ "children": "never" }]))),
        ("<App>{`Hello ${word} World`}{`foo`}</App>", Some(json!([{ "children": "never" }]))),
        ("<App prop={`foo ${word} bar`}>foo</App>", Some(json!([{ "props": "never" }]))),
        ("<App prop={`foo ${word} bar`} />", Some(json!([{ "props": "never" }]))),
        ("<App>{<myApp></myApp>}</App>", Some(json!([{ "children": "always" }]))),
        ("<App>{[]}</App>", None),
        ("<App>foo</App>", None),
        (r#"<App>{"foo"}{<Component>bar</Component>}</App>"#, None),
        ("<App prop='bar'>foo</App>", None),
        ("<App prop={true}>foo</App>", None),
        ("<App prop>foo</App>", None),
        (r"<App prop='bar'>{'foo \n bar'}</App>", None),
        ("<App prop={ ' ' }/>", None),
        ("<MyComponent prop='bar'>foo</MyComponent>", Some(json!([{ "props": "never" }]))),
        (r#"<MyComponent prop="bar">foo</MyComponent>"#, Some(json!([{ "props": "never" }]))),
        ("<MyComponent>foo</MyComponent>", Some(json!([{ "children": "never" }]))),
        (r#"<MyComponent>{<App/>}{"123"}</MyComponent>"#, Some(json!([{ "children": "never" }]))),
        (r#"<App>{"foo 'bar' \"foo\" bar"}</App>"#, Some(json!([{ "children": "never" }]))),
        ("<MyComponent prop={'bar'}>foo</MyComponent>", Some(json!([{ "props": "always" }]))),
        ("<MyComponent>{'foo'}</MyComponent>", Some(json!([{ "children": "always" }]))),
        (r#"<MyComponent prop={"bar"}>foo</MyComponent>"#, Some(json!([{ "props": "always" }]))),
        (r#"<MyComponent>{"foo"}</MyComponent>"#, Some(json!([{ "children": "always" }]))),
        ("<MyComponent>{'foo'}</MyComponent>", Some(json!([{ "children": "ignore" }]))),
        ("<MyComponent prop={'bar'}>foo</MyComponent>", Some(json!([{ "props": "ignore" }]))),
        ("<MyComponent>foo</MyComponent>", Some(json!([{ "children": "ignore" }]))),
        ("<MyComponent prop='bar'>foo</MyComponent>", Some(json!([{ "props": "ignore" }]))),
        (r#"<MyComponent prop="bar">foo</MyComponent>"#, Some(json!([{ "props": "ignore" }]))),
        (
            "<MyComponent prop='bar'>{'foo'}</MyComponent>",
            Some(json!([{ "children": "always", "props": "never" }])),
        ),
        (
            "<MyComponent prop={'bar'}>foo</MyComponent>",
            Some(json!([{ "children": "never", "props": "always" }])),
        ),
        ("<MyComponent prop={'bar'}>{'foo'}</MyComponent>", Some(json!(["always"]))),
        (r#"<MyComponent prop={"bar"}>{"foo"}</MyComponent>"#, Some(json!(["always"]))),
        (r#"<MyComponent prop={"bar"} attr={'foo'} />"#, Some(json!(["always"]))),
        (r#"<MyComponent prop="bar" attr='foo' />"#, Some(json!(["never"]))),
        ("<MyComponent prop='bar'>foo</MyComponent>", Some(json!(["never"]))),
        (
            "<MyComponent prop={`bar ${word} foo`}>{`foo ${word}`}</MyComponent>",
            Some(json!(["never"])),
        ),
        (r#"<MyComponent>{"div { margin-top: 0; }"}</MyComponent>"#, Some(json!(["never"]))),
        (r#"<MyComponent>{"<Foo />"}</MyComponent>"#, Some(json!(["never"]))),
        (r#"<MyComponent prop={"Hello \u1026 world"}>bar</MyComponent>"#, Some(json!(["never"]))),
        (r#"<MyComponent>{"Hello \u1026 world"}</MyComponent>"#, Some(json!(["never"]))),
        (r#"<MyComponent prop={"Hello &middot; world"}>bar</MyComponent>"#, Some(json!(["never"]))),
        (r#"<MyComponent>{"Hello &middot; world"}</MyComponent>"#, Some(json!(["never"]))),
        (r#"<MyComponent>{"Hello \n world"}</MyComponent>"#, Some(json!(["never"]))),
        (r#"<MyComponent>{"space after "}</MyComponent>"#, Some(json!(["never"]))),
        (r#"<MyComponent>{" space before"}</MyComponent>"#, Some(json!(["never"]))),
        ("<MyComponent>{`space after `}</MyComponent>", Some(json!(["never"]))),
        ("<MyComponent>{` space before`}</MyComponent>", Some(json!(["never"]))),
        (
            "
			        <App prop={`
			          a
			          b
			        `} />
			      ",
            Some(json!(["never"])),
        ),
        (
            "
			        <App prop={`
			          a
			          b
			        `} />
			      ",
            Some(json!(["always"])),
        ),
        (
            "
			        <App>
			          {`
			            a
			            b
			          `}
			        </App>
			      ",
            Some(json!(["never"])),
        ),
        (
            "
			        <App>{`
			          a
			          b
			        `}</App>
			      ",
            Some(json!(["always"])),
        ),
        (
            "
			        <MyComponent>
			          %
			        </MyComponent>
			      ",
            Some(json!([{ "children": "never" }])),
        ),
        (
            "
			        <MyComponent>
			          { 'space after ' }
			          <b>foo</b>
			          { ' space before' }
			        </MyComponent>
			      ",
            Some(json!([{ "children": "never" }])),
        ),
        (
            "
			        <MyComponent>
			          { `space after ` }
			          <b>foo</b>
			          { ` space before` }
			        </MyComponent>
			      ",
            Some(json!([{ "children": "never" }])),
        ),
        (
            "
			        <MyComponent>
			          foo
			          <div>bar</div>
			        </MyComponent>
			      ",
            Some(json!([{ "children": "never" }])),
        ),
        (
            "
			        <MyComponent p={<Foo>Bar</Foo>}>
			        </MyComponent>
			      ",
            None,
        ),
        (
            r#"
			        <MyComponent>
			          <div>
			            <p>
			              <span>
			                {"foo"}
			              </span>
			            </p>
			          </div>
			        </MyComponent>
			      "#,
            Some(json!([{ "children": "always" }])),
        ),
        (
            "
			        <App>
			          <Component />&nbsp;
			          &nbsp;
			        </App>
			      ",
            Some(json!([{ "children": "always" }])),
        ),
        (
            "
			        const Component2 = () => {
			          return <span>/*</span>;
			        };
			      ",
            None,
        ),
        (
            "
			        const Component2 = () => {
			          return <span>/*</span>;
			        };
			      ",
            Some(json!([{ "props": "never", "children": "never" }])),
        ),
        (
            r#"
			        import React from "react";

			        const Component = () => {
			          return <span>{"/*"}</span>;
			        };
			      "#,
            Some(json!([{ "props": "never", "children": "never" }])),
        ),
        ("<App>{/* comment */}</App>", None),
        ("<App horror=<div /> />", None),
        ("<App horror={<div />} />", None),
        ("<App horror=<div /> />", Some(json!([{ "propElementValues": "ignore" }]))),
        ("<App horror={<div />} />", Some(json!([{ "propElementValues": "ignore" }]))),
        (
            r#"
			        <script>{`window.foo = "bar"`}</script>
			      "#,
            None,
        ),
        (
            r#"
			        <CollapsibleTitle
			          extra={<span className="activity-type">{activity.type}</span>}
			        />
			      "#,
            Some(json!(["never"])),
        ),
        ("<App label={`${label}`} />", Some(json!(["never"]))),
        ("<App>{`${label}`}</App>", Some(json!(["never"]))),
        (r#"<div>{`Nobody's "here"`}</div>"#, None),
    ];

    let fail = vec![
        ("<App prop={`foo`} />", Some(json!([{ "props": "never" }]))),
        ("<App>{<myApp></myApp>}</App>", Some(json!([{ "children": "never" }]))),
        ("<App>{<myApp></myApp>}</App>", None),
        ("<App prop={`foo`}>foo</App>", Some(json!([{ "props": "never" }]))),
        ("<App>{`foo`}</App>", Some(json!([{ "children": "never" }]))),
        ("<>{`foo`}</>", Some(json!([{ "children": "never" }]))),
        ("<MyComponent>{'foo'}</MyComponent>", None),
        ("<MyComponent prop={'bar'}>foo</MyComponent>", None),
        ("<MyComponent>{'foo'}</MyComponent>", Some(json!([{ "children": "never" }]))),
        ("<MyComponent prop={'bar'}>foo</MyComponent>", Some(json!([{ "props": "never" }]))),
        (
            "
			        <MyComponent>
			          {'%'}
			        </MyComponent>
			      ",
            Some(json!([{ "children": "never" }])),
        ),
        (
            "
			        <MyComponent>
			          {'foo'}
			          <div>
			            {'bar'}
			          </div>
			          {'baz'}
			        </MyComponent>
			      ",
            Some(json!([{ "children": "never" }])),
        ),
        (
            "
			        <MyComponent>
			          {'foo'}
			          <div>
			            {'bar'}
			          </div>
			          {'baz'}
			          {'some-complicated-exp'}
			        </MyComponent>
			      ",
            Some(json!([{ "children": "never" }])),
        ),
        ("<MyComponent prop='bar'>foo</MyComponent>", Some(json!([{ "props": "always" }]))),
        (
            r#"<MyComponent prop="foo 'bar'">foo</MyComponent>"#,
            Some(json!([{ "props": "always" }])),
        ),
        (
            r#"<MyComponent prop='foo "bar"'>foo</MyComponent>"#,
            Some(json!([{ "props": "always" }])),
        ),
        (
            r#"<MyComponent prop="foo 'bar'">foo</MyComponent>"#,
            Some(json!([{ "props": "always" }])),
        ),
        ("<MyComponent>foo bar </MyComponent>", Some(json!([{ "children": "always" }]))),
        (
            r#"<MyComponent prop="foo 'bar' \n ">foo</MyComponent>"#,
            Some(json!([{ "props": "always" }])),
        ),
        ("<MyComponent>foo bar \r </MyComponent>", Some(json!([{ "children": "always" }]))),
        ("<MyComponent>foo bar 'foo'</MyComponent>", Some(json!([{ "children": "always" }]))),
        (r#"<MyComponent>foo bar "foo"</MyComponent>"#, Some(json!([{ "children": "always" }]))),
        ("<MyComponent>foo bar <App/></MyComponent>", Some(json!([{ "children": "always" }]))),
        ("<MyComponent>foo \n bar</MyComponent>", Some(json!([{ "children": "always" }]))),
        ("<MyComponent>foo \\u1234 bar</MyComponent>", Some(json!([{ "children": "always" }]))),
        ("<MyComponent prop='foo \\u1234 bar' />", Some(json!([{ "props": "always" }]))),
        ("<MyComponent prop={'bar'}>{'foo'}</MyComponent>", Some(json!(["never"]))),
        ("<MyComponent prop='bar'>foo</MyComponent>", Some(json!(["always"]))),
        (r#"<App prop={'foo'} attr={" foo "} />"#, Some(json!([{ "props": "never" }]))),
        (r#"<App prop='foo' attr="bar" />"#, Some(json!([{ "props": "always" }]))),
        (r#"<App prop='foo' attr={"bar"} />"#, Some(json!([{ "props": "always" }]))),
        ("<App prop={'foo'} attr='bar' />", Some(json!([{ "props": "always" }]))),
        ("<App prop='foo &middot; bar' />", Some(json!([{ "props": "always" }]))),
        ("<App>foo &middot; bar</App>", Some(json!([{ "children": "always" }]))),
        (r#"<App>{'foo "bar"'}</App>"#, Some(json!([{ "children": "never" }]))),
        (r#"<App>{"foo 'bar'"}</App>"#, Some(json!([{ "children": "never" }]))),
        (
            r#"
			        <App prop="" />"#,
            Some(json!(["always"])),
        ),
        (
            "
			        <App prop='' />",
            Some(json!(["always"])),
        ),
        (
            "
			        <App>
			          foo bar
			          <div>foo bar foo</div>
			          <span>
			            foo bar <i>foo bar</i>
			            <strong>
			              foo bar
			            </strong>
			          </span>
			        </App>
			      ",
            Some(json!([{ "children": "always" }])),
        ),
        (
            "
        	        <App>
        	          &lt;Component&gt;
        	          &nbsp;<Component />&nbsp;
        	          &nbsp;
        	        </App>
        	      ",
            Some(json!([{ "children": "always" }])),
        ),
        (
            "
			        <Box mb={'1rem'} />
			      ",
            Some(json!([{ "props": "never" }])),
        ),
        (
            "
			        <Box mb={'1rem {}'} />
			      ",
            Some(json!(["never"])),
        ),
        (r#"<MyComponent prop={"{ style: true }"}>bar</MyComponent>"#, Some(json!(["never"]))),
        (r#"<MyComponent prop={"< style: true >"}>foo</MyComponent>"#, Some(json!(["never"]))),
        (
            "<App horror=<div /> />",
            Some(
                json!([{ "props": "always", "children": "always", "propElementValues": "always" }]),
            ),
        ),
        (
            "<App horror={<div />} />",
            Some(json!([{ "props": "never", "children": "never", "propElementValues": "never" }])),
        ),
        (
            r#"<Foo bar={"'"} />"#,
            Some(json!([{ "props": "never", "children": "never", "propElementValues": "never" }])),
        ),
        (
            r#"
        	        <Foo help={'The maximum time range for searches. (i.e. "P30D" for 30 days, "PT24H" for 24 hours)'} />
        	      "#,
            Some(json!(["never"])),
        ),
    ];

    let fix = vec![
        ("<App prop={`foo`} />", r#"<App prop="foo" />"#, Some(json!([{ "props": "never" }]))),
        (
            "<App>{<myApp></myApp>}</App>",
            "<App><myApp></myApp></App>",
            Some(json!([{ "children": "never" }])),
        ),
        ("<App>{<myApp></myApp>}</App>", "<App><myApp></myApp></App>", None),
        (
            "<App prop={`foo`}>foo</App>",
            r#"<App prop="foo">foo</App>"#,
            Some(json!([{ "props": "never" }])),
        ),
        ("<App>{`foo`}</App>", "<App>foo</App>", Some(json!([{ "children": "never" }]))),
        ("<>{`foo`}</>", "<>foo</>", Some(json!([{ "children": "never" }]))),
        ("<MyComponent>{'foo'}</MyComponent>", "<MyComponent>foo</MyComponent>", None),
        (
            "<MyComponent prop={'bar'}>foo</MyComponent>",
            r#"<MyComponent prop="bar">foo</MyComponent>"#,
            None,
        ),
        (
            "<MyComponent>{'foo'}</MyComponent>",
            "<MyComponent>foo</MyComponent>",
            Some(json!([{ "children": "never" }])),
        ),
        (
            "<MyComponent prop={'bar'}>foo</MyComponent>",
            r#"<MyComponent prop="bar">foo</MyComponent>"#,
            Some(json!([{ "props": "never" }])),
        ),
        (
            "
			        <MyComponent>
			          {'%'}
			        </MyComponent>
			      ",
            "
			        <MyComponent>
			          %
			        </MyComponent>
			      ",
            Some(json!([{ "children": "never" }])),
        ),
        (
            "
			        <MyComponent>
			          {'foo'}
			          <div>
			            {'bar'}
			          </div>
			          {'baz'}
			        </MyComponent>
			      ",
            "
			        <MyComponent>
			          foo
			          <div>
			            bar
			          </div>
			          baz
			        </MyComponent>
			      ",
            Some(json!([{ "children": "never" }])),
        ),
        (
            "
			        <MyComponent>
			          {'foo'}
			          <div>
			            {'bar'}
			          </div>
			          {'baz'}
			          {'some-complicated-exp'}
			        </MyComponent>
			      ",
            "
			        <MyComponent>
			          foo
			          <div>
			            bar
			          </div>
			          baz
			          some-complicated-exp
			        </MyComponent>
			      ",
            Some(json!([{ "children": "never" }])),
        ),
        (
            "<MyComponent prop='bar'>foo</MyComponent>",
            r#"<MyComponent prop={"bar"}>foo</MyComponent>"#,
            Some(json!([{ "props": "always" }])),
        ),
        (
            r#"<MyComponent prop="foo 'bar'">foo</MyComponent>"#,
            r#"<MyComponent prop={"foo 'bar'"}>foo</MyComponent>"#,
            Some(json!([{ "props": "always" }])),
        ),
        (
            r#"<MyComponent prop='foo "bar"'>foo</MyComponent>"#,
            r#"<MyComponent prop={"foo \"bar\""}>foo</MyComponent>"#,
            Some(json!([{ "props": "always" }])),
        ),
        (
            "<MyComponent>foo bar </MyComponent>",
            r#"<MyComponent>{"foo bar "}</MyComponent>"#,
            Some(json!([{ "children": "always" }])),
        ),
        (
            r#"<MyComponent prop="foo 'bar' \n ">foo</MyComponent>"#,
            r#"<MyComponent prop={"foo 'bar' \\n "}>foo</MyComponent>"#,
            Some(json!([{ "props": "always" }])),
        ),
        (
            r"<MyComponent>foo bar \r </MyComponent>",
            r#"<MyComponent>{"foo bar \\r "}</MyComponent>"#,
            Some(json!([{ "children": "always" }])),
        ),
        (
            "<MyComponent>foo bar 'foo'</MyComponent>",
            r#"<MyComponent>{"foo bar 'foo'"}</MyComponent>"#,
            Some(json!([{ "children": "always" }])),
        ),
        (
            r#"<MyComponent>foo bar "foo"</MyComponent>"#,
            r#"<MyComponent>{"foo bar \"foo\""}</MyComponent>"#,
            Some(json!([{ "children": "always" }])),
        ),
        (
            "<MyComponent>foo bar <App/></MyComponent>",
            r#"<MyComponent>{"foo bar "}<App/></MyComponent>"#,
            Some(json!([{ "children": "always" }])),
        ),
        (
            //Note: this case might not fully translate from eslint to oxlint
            "<MyComponent>foo \\n bar</MyComponent>",
            r#"<MyComponent>{"foo \\n bar"}</MyComponent>"#,
            Some(json!([{ "children": "always" }])),
        ),
        (
            "<MyComponent>😄</MyComponent>",
            r#"<MyComponent>{"😄"}</MyComponent>"#,
            Some(json!([{ "children": "always" }])),
        ),
        (
            r"<MyComponent>foo \u1234 bar</MyComponent>",
            r#"<MyComponent>{"foo \\u1234 bar"}</MyComponent>"#,
            Some(json!([{ "children": "always" }])),
        ),
        (
            r"<MyComponent prop='foo \u1234 bar' />",
            r#"<MyComponent prop={"foo \\u1234 bar"} />"#,
            Some(json!([{ "props": "always" }])),
        ),
        (
            "<MyComponent prop={'bar'}>{'foo'}</MyComponent>",
            r#"<MyComponent prop="bar">foo</MyComponent>"#,
            Some(json!(["never"])),
        ),
        (
            "<MyComponent prop='bar'>foo</MyComponent>",
            r#"<MyComponent prop={"bar"}>{"foo"}</MyComponent>"#,
            Some(json!(["always"])),
        ),
        (
            r#"<App prop={'foo'} attr={" foo "} />"#,
            r#"<App prop="foo" attr=" foo " />"#,
            Some(json!([{ "props": "never" }])),
        ),
        (
            r#"<App prop='foo' attr="bar" />"#,
            r#"<App prop={"foo"} attr={"bar"} />"#,
            Some(json!([{ "props": "always" }])),
        ),
        (
            r#"<App prop='foo' attr={"bar"} />"#,
            r#"<App prop={"foo"} attr={"bar"} />"#,
            Some(json!([{ "props": "always" }])),
        ),
        (
            "<App prop={'foo'} attr='bar' />",
            r#"<App prop={'foo'} attr={"bar"} />"#,
            Some(json!([{ "props": "always" }])),
        ),
        (
            "<App prop='foo &middot; bar' />",
            r#"<App prop={"foo &middot; bar"} />"#,
            Some(json!([{ "props": "always" }])),
        ),
        (
            "<App>foo &middot; bar</App>",
            r#"<App>{"foo &middot; bar"}</App>"#,
            Some(json!([{ "children": "always" }])),
        ),
        (
            r#"<App>{'foo "bar"'}</App>"#,
            r#"<App>foo "bar"</App>"#,
            Some(json!([{ "children": "never" }])),
        ),
        (
            r#"<App>{"foo 'bar'"}</App>"#,
            "<App>foo 'bar'</App>",
            Some(json!([{ "children": "never" }])),
        ),
        (
            // Note: this case does not exist in the eslint tests cases
            r#"
			        <App prop="" />"#,
            r#"
			        <App prop={""} />"#,
            Some(json!(["always"])),
        ),
        (
            "
			        <App prop='",
            "
			        <App prop='",
            Some(json!(["always"])),
        ),
        (
            "
			        <App>
			          foo bar
			          <div>foo bar foo</div>
			          <span>
			            foo bar <i>foo bar</i>
			            <strong>
			              foo bar
			            </strong>
			          </span>
			        </App>
			      ",
            r#"
			        <App>
			          {"foo bar"}
			          <div>{"foo bar foo"}</div>
			          <span>
			            {"foo bar "}<i>{"foo bar"}</i>
			            <strong>
			              {"foo bar"}
			            </strong>
			          </span>
			        </App>
			      "#,
            Some(json!([{ "children": "always" }])),
        ),
        (
            "
			        <App>
			          &lt;Component&gt;
			          &nbsp;<Component />&nbsp;
			          &nbsp;
			        </App>
			      ",
            r#"
			        <App>
			          &lt;{"Component"}&gt;
			          &nbsp;<Component />&nbsp;
			          &nbsp;
			        </App>
			      "#,
            Some(json!([{ "children": "always" }])),
        ),
        (
            "
			        <Box mb={'1rem'} />
			      ",
            r#"
			        <Box mb="1rem" />
			      "#,
            Some(json!([{ "props": "never" }])),
        ),
        (
            "
			        <Box mb={'1rem {}'} />
			      ",
            r#"
			        <Box mb="1rem {}" />
			      "#,
            Some(json!(["never"])),
        ),
        (
            r#"<MyComponent prop={"{ style: true }"}>bar</MyComponent>"#,
            r#"<MyComponent prop="{ style: true }">bar</MyComponent>"#,
            Some(json!(["never"])),
        ),
        (
            r#"<MyComponent prop={"< style: true >"}>foo</MyComponent>"#,
            r#"<MyComponent prop="< style: true >">foo</MyComponent>"#,
            Some(json!(["never"])),
        ),
        (
            "<App horror=<div /> />",
            "<App horror={<div />} />",
            Some(
                json!([{ "props": "always", "children": "always", "propElementValues": "always" }]),
            ),
        ),
        (
            "<App horror={<div />} />",
            "<App horror=<div /> />",
            Some(json!([{ "props": "never", "children": "never", "propElementValues": "never" }])),
        ),
        (
            r#"<Foo bar={"'"} />"#,
            r#"<Foo bar="'" />"#,
            Some(json!([{ "props": "never", "children": "never", "propElementValues": "never" }])),
        ),
        (
            r#"
			        <Foo help={'The maximum time range for searches. (i.e. "P30D" for 30 days, "PT24H" for 24 hours)'} />
			      "#,
            r#"
			        <Foo help='The maximum time range for searches. (i.e. "P30D" for 30 days, "PT24H" for 24 hours)' />
			      "#,
            Some(json!(["never"])),
        ),
        (
            "
            <App>
                &lt;Component&gt;foo bar &lt;Component /&gt;
                &nbsp;
            </App>
            ",
            r#"
            <App>
                &lt;{"Component"}&gt;{"foo bar "}&lt;{"Component /"}&gt;
                &nbsp;
            </App>
            "#,
            Some(json!([{ "children": "always" }])),
        ),
        (r#"<Foo bar={'"'} />"#, r#"<Foo bar='"' />"#, Some(json!(["never"]))),
    ];
    Tester::new(JsxCurlyBracePresence::NAME, JsxCurlyBracePresence::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
