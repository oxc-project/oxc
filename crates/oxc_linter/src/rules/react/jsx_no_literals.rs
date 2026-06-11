use std::default::Default;

use rustc_hash::FxHashMap;
use schemars::JsonSchema;
use serde::Deserialize;

use oxc_ast::{
    AstKind,
    ast::{
        BindingPattern, Expression, IdentifierReference, JSXAttribute, JSXAttributeItem,
        JSXAttributeName, JSXAttributeValue, JSXChild, JSXElement, JSXElementName, JSXExpression,
        JSXFragment, JSXMemberExpression, JSXMemberExpressionObject, ModuleExportName, PropertyKey,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::SymbolId;
use oxc_span::Span;
use oxc_str::CompactStr;

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
    utils::is_react_component_name,
};

fn literal_text_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Disallow literal text as JSX children")
        .with_help("Wrap this text in a JSX expression container, such as a call to a translation function.")
        .with_label(span)
}

fn literal_attribute_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Disallow string literals in JSX attributes")
        .with_help("Replace this string literal with a non-literal expression, such as a call to a translation function.")
        .with_label(span)
}

fn restricted_attribute_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Disallow string literals on restricted JSX attributes")
        .with_help("This attribute is listed in `restrictedAttributes`; replace its string literal value with a non-literal expression.")
        .with_label(span)
}

/// The options shared between the top-level config and each `elementOverrides` entry.
#[derive(Debug, Default, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct JsxNoLiteralsOptions {
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

/// One entry in `elementOverrides`: the base options plus override-only fields.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct ElementOverrideOptions {
    #[serde(flatten)]
    options: JsxNoLiteralsOptions,

    /// (default: false) - When true the rule will allow the specified element to have string literals as children, wrapped or unwrapped without warning.
    allow_element: bool,

    /// (default: true) - When false the rule will not apply the current options set to nested elements. This is useful when you want to apply the rule to a specific element, but not to its children.
    apply_to_nested_elements: bool,
}

impl Default for ElementOverrideOptions {
    fn default() -> Self {
        Self {
            options: JsxNoLiteralsOptions::default(),
            allow_element: false,
            apply_to_nested_elements: true,
        }
    }
}

// Boxed so the struct stays pointer-sized: every rule is a `RuleEnum` variant
// that runs in a tight loop, and the config (two `Vec`s plus a `FxHashMap`) is
// otherwise large enough to bloat the whole enum.
#[derive(Debug, Default, Clone)]
pub struct JsxNoLiterals(Box<JsxNoLiteralsConfig>);

#[derive(Debug, Default, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct JsxNoLiteralsConfig {
    #[serde(flatten)]
    options: JsxNoLiteralsOptions,

    /// An object where the keys are the element names and the values are objects with the same options as above. This allows you to specify different options for different elements.
    element_overrides: FxHashMap<CompactStr, ElementOverrideOptions>,
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
    config = JsxNoLiteralsConfig,
    version = "next",
);

impl Rule for JsxNoLiterals {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<JsxNoLiteralsConfig>>(value)
            .map(|config| Self(Box::new(config.into_inner())))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        // Deliberately chose to structure the code like this so it can be optimized by linter codegen
        match node.kind() {
            AstKind::JSXElement(jsx_el) => {
                if Self::has_parent_jsx_element(node, ctx) {
                    return;
                }
                self.check_element(jsx_el, None, ctx);
            }
            AstKind::JSXFragment(fragment) => {
                if Self::has_parent_jsx_element(node, ctx) {
                    return;
                }
                self.check_fragment(fragment, None, ctx);
            }
            _ => {}
        }
    }
}

impl JsxNoLiterals {
    fn is_allowed_string(str_literal: &str, cfg: &JsxNoLiteralsOptions) -> bool {
        cfg.allowed_strings.iter().any(|allowed| allowed.as_str().trim() == str_literal.trim())
    }

    fn has_parent_jsx_element(node: &AstNode, ctx: &LintContext) -> bool {
        ctx.nodes().ancestors(node.id()).any(|ancestor| {
            matches!(ancestor.kind(), AstKind::JSXElement(_) | AstKind::JSXFragment(_))
        })
    }

    fn resolve_from_object_pattern(
        binding: &BindingPattern,
        symbol_id: SymbolId,
    ) -> Option<CompactStr> {
        let BindingPattern::ObjectPattern(obj) = binding else { return None };

        for prop in &obj.properties {
            if let BindingPattern::BindingIdentifier(local) = &prop.value
                && local.symbol_id.get() == Some(symbol_id)
                && let PropertyKey::StaticIdentifier(key) = &prop.key
            {
                return Some(key.name.to_compact_str());
            }
        }
        None
    }

    fn resolve_element_name(id_ref: &IdentifierReference, ctx: &LintContext) -> CompactStr {
        let local_name = id_ref.name.to_compact_str();

        let Some(reference_id) = id_ref.reference_id.get() else {
            return local_name;
        };
        let Some(symbol_id) = ctx.scoping().get_reference(reference_id).symbol_id() else {
            return local_name;
        };

        let decl = ctx.semantic().symbol_declaration(symbol_id);

        match decl.kind() {
            AstKind::ImportSpecifier(specifier) => {
                if let ModuleExportName::IdentifierName(imported) = &specifier.imported {
                    return imported.name.to_compact_str();
                }
            }
            AstKind::VariableDeclarator(declarator) => {
                if let Some(name) = Self::resolve_from_object_pattern(&declarator.id, symbol_id) {
                    return name;
                }
            }
            _ => {}
        }

        local_name
    }

    fn resolve_member_expression_name(
        member_expr: &JSXMemberExpression,
        ctx: &LintContext,
    ) -> CompactStr {
        let object = match &member_expr.object {
            JSXMemberExpressionObject::IdentifierReference(id_ref) => {
                Self::resolve_element_name(id_ref, ctx)
            }
            JSXMemberExpressionObject::MemberExpression(inner) => {
                Self::resolve_member_expression_name(inner, ctx)
            }
            JSXMemberExpressionObject::ThisExpression(_) => CompactStr::from("this"),
        };

        CompactStr::from(format!("{object}.{}", member_expr.property.name).as_str())
    }

    fn inspect_element_literals(
        children: &[JSXChild],
        options: &JsxNoLiteralsOptions,
        ctx: &LintContext,
    ) {
        for child in children {
            match child {
                JSXChild::Text(text) => {
                    let value = text.value.as_str();

                    if Self::is_allowed_string(value, options) {
                        continue;
                    }

                    if !value.trim().is_empty() {
                        ctx.diagnostic(literal_text_diagnostic(text.span));
                    }
                }
                JSXChild::ExpressionContainer(container) if options.no_strings => {
                    match &container.expression {
                        JSXExpression::StringLiteral(literal) => {
                            ctx.diagnostic(literal_text_diagnostic(literal.span));
                        }
                        JSXExpression::TemplateLiteral(literal) => {
                            ctx.diagnostic(literal_text_diagnostic(literal.span));
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }

    fn inspect_jsx_expression(expr: &Expression, attr: &JSXAttribute, ctx: &LintContext) {
        match &expr {
            Expression::StringLiteral(_) | Expression::TemplateLiteral(_) => {
                ctx.diagnostic(literal_attribute_diagnostic(attr.span));
            }
            Expression::BinaryExpression(expression) => {
                Self::inspect_jsx_expression(&expression.left, attr, ctx);
                Self::inspect_jsx_expression(&expression.right, attr, ctx);
            }
            _ => {}
        }
    }

    fn inspect_element_attributes(
        jsx_el: &JSXElement,
        options: &JsxNoLiteralsOptions,
        ctx: &LintContext,
    ) {
        for attr in &jsx_el.opening_element.attributes {
            let JSXAttributeItem::Attribute(attr) = attr else {
                continue;
            };

            let Some(value) = &attr.value else {
                continue;
            };

            match value {
                JSXAttributeValue::StringLiteral(str_literal) => {
                    if Self::is_allowed_string(str_literal.value.as_str(), options) {
                        continue;
                    }

                    if !options.restricted_attributes.is_empty() {
                        let attr_name = match &attr.name {
                            JSXAttributeName::Identifier(ident) => ident.name.as_str(),
                            JSXAttributeName::NamespacedName(namespaced_name) => {
                                namespaced_name.name.name.as_str()
                            }
                        };

                        if options
                            .restricted_attributes
                            .iter()
                            .any(|restricted| restricted.as_str() == attr_name)
                        {
                            ctx.diagnostic(restricted_attribute_diagnostic(attr.span));
                            continue;
                        }
                    }

                    if options.ignore_props {
                        continue;
                    }

                    if options.no_attribute_strings || options.no_strings {
                        ctx.diagnostic(literal_attribute_diagnostic(attr.span));
                    }
                }
                JSXAttributeValue::ExpressionContainer(container) => {
                    if options.no_strings
                        && let Some(expr) = container.expression.as_expression()
                    {
                        Self::inspect_jsx_expression(expr, attr, ctx);
                    }
                }
                _ => {}
            }
        }
    }

    fn get_element_override_opts(
        &self,
        jsx_el: &JSXElement,
        ctx: &LintContext,
    ) -> Option<&ElementOverrideOptions> {
        let resolved = match &jsx_el.opening_element.name {
            JSXElementName::IdentifierReference(id_ref) => Self::resolve_element_name(id_ref, ctx),
            JSXElementName::MemberExpression(member_expr) => {
                Self::resolve_member_expression_name(member_expr, ctx)
            }
            JSXElementName::Identifier(ident) => ident.name.to_compact_str(),
            JSXElementName::NamespacedName(namespaced) => CompactStr::from(
                format!("{}:{}", namespaced.namespace.name, namespaced.name.name).as_str(),
            ),
            JSXElementName::ThisExpression(_) => {
                return None;
            }
        };

        if !is_react_component_name(resolved.as_str()) {
            return None;
        }

        if let Some(opts) = self.0.element_overrides.get(&resolved) {
            return Some(opts);
        }

        // For member expressions (e.g. `React.Fragment`), the full dotted name is
        // tried first, then the bare last-property name (`Fragment`) as a fallback.
        if let JSXElementName::MemberExpression(member_expr) = &jsx_el.opening_element.name {
            return self.0.element_overrides.get(&member_expr.property.name.to_compact_str());
        }

        None
    }

    fn descend_child_elements(
        &self,
        children: &[JSXChild],
        options: Option<&ElementOverrideOptions>,
        ctx: &LintContext,
    ) {
        for child in children {
            match child {
                JSXChild::Element(jsx_el) => {
                    self.check_element(jsx_el, options, ctx);
                }
                JSXChild::Fragment(fragment) => {
                    self.check_fragment(fragment, options, ctx);
                }
                _ => {}
            }
        }
    }

    fn check_element(
        &self,
        jsx_el: &JSXElement,
        inherited_opts: Option<&ElementOverrideOptions>,
        ctx: &LintContext,
    ) {
        let element_override_opts = self.get_element_override_opts(jsx_el, ctx).or(inherited_opts);

        let (options, allow_element, apply_to_nested_elements) =
            if let Some(element_override_opts) = element_override_opts {
                (
                    &element_override_opts.options,
                    element_override_opts.allow_element,
                    element_override_opts.apply_to_nested_elements,
                )
            } else {
                (&self.0.options, false, true)
            };

        if !allow_element {
            Self::inspect_element_literals(&jsx_el.children, options, ctx);
            Self::inspect_element_attributes(jsx_el, options, ctx);
        }

        self.descend_child_elements(
            &jsx_el.children,
            if apply_to_nested_elements { element_override_opts } else { None },
            ctx,
        );
    }

    fn check_fragment(
        &self,
        fragment: &JSXFragment,
        inherited_opts: Option<&ElementOverrideOptions>,
        ctx: &LintContext,
    ) {
        let options = inherited_opts.map_or(&self.0.options, |opts| &opts.options);

        Self::inspect_element_literals(&fragment.children, options, ctx);
        self.descend_child_elements(&fragment.children, inherited_opts, ctx);
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
