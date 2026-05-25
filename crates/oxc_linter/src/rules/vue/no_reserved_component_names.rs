use phf::{Set, phf_set};

use oxc_ast::{
    AstKind,
    ast::{CallExpression, Expression, ObjectExpression, ObjectPropertyKind, TemplateLiteral},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
    utils::{find_property, is_vue_component_options_object_excluding_instance},
};

static HTML_ELEMENTS: Set<&'static str> = phf_set! {
    "a", "abbr", "address", "area", "article", "aside", "audio", "b", "base", "bdi", "bdo",
    "blockquote", "body", "br", "button", "canvas", "caption", "cite", "code", "col", "colgroup",
    "data", "datalist", "dd", "del", "details", "dfn", "dialog", "div", "dl", "dt", "em", "embed",
    "fencedframe", "fieldset", "figcaption", "figure", "footer", "form", "h1", "h2", "h3", "h4",
    "h5", "h6", "head", "header", "hgroup", "hr", "html", "i", "iframe", "img", "input", "ins",
    "kbd", "label", "legend", "li", "link", "main", "map", "mark", "menu", "meta", "meter", "nav",
    "noscript", "object", "ol", "optgroup", "option", "output", "p", "picture", "pre", "progress",
    "q", "rp", "rt", "ruby", "s", "samp", "script", "search", "section", "select",
    "selectedcontent", "slot", "small", "source", "span", "strong", "style", "sub", "summary",
    "sup", "table", "tbody", "td", "template", "textarea", "tfoot", "th", "thead", "time",
    "title", "tr", "track", "u", "ul", "var", "video", "wbr",
};

static DEPRECATED_HTML_ELEMENTS: Set<&'static str> = phf_set! {
    "acronym", "applet", "basefont", "bgsound", "big", "blink", "center", "dir", "font", "frame",
    "frameset", "isindex", "keygen", "listing", "marquee", "menuitem", "multicol", "nextid",
    "nobr", "noembed", "noframes", "param", "plaintext", "rb", "rtc", "spacer", "strike", "tt",
    "xmp",
};

static SVG_ELEMENTS: Set<&'static str> = phf_set! {
    "a", "animate", "animateMotion", "animateTransform", "circle", "clipPath", "defs", "desc",
    "ellipse", "feBlend", "feColorMatrix", "feComponentTransfer", "feComposite",
    "feConvolveMatrix", "feDiffuseLighting", "feDisplacementMap", "feDistantLight", "feDropShadow",
    "feFlood", "feFuncA", "feFuncB", "feFuncG", "feFuncR", "feGaussianBlur", "feImage", "feMerge",
    "feMergeNode", "feMorphology", "feOffset", "fePointLight", "feSpecularLighting", "feSpotLight",
    "feTile", "feTurbulence", "filter", "foreignObject", "g", "image", "line", "linearGradient",
    "marker", "mask", "metadata", "mpath", "path", "pattern", "polygon", "polyline",
    "radialGradient", "rect", "script", "set", "stop", "style", "svg", "switch", "symbol", "text",
    "textPath", "title", "tspan", "use", "view",
};

static KEBAB_CASE_ELEMENTS: Set<&'static str> = phf_set! {
    "annotation-xml", "color-profile", "font-face", "font-face-src", "font-face-uri",
    "font-face-format", "font-face-name", "missing-glyph",
};

static VUE2_BUILTIN: Set<&'static str> = phf_set! {
    "template", "slot", "component", "Component", "transition", "Transition", "transition-group",
    "TransitionGroup", "keep-alive", "KeepAlive",
};

static VUE3_BUILTIN_EXTRA: Set<&'static str> = phf_set! {
    "teleport", "Teleport", "suspense", "Suspense",
};

fn reserved_diagnostic(name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Name \"{name}\" is reserved.")).with_label(span)
}

fn reserved_in_html_diagnostic(name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Name \"{name}\" is reserved in HTML.")).with_label(span)
}

fn reserved_in_vue_diagnostic(name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Name \"{name}\" is reserved in Vue.js.")).with_label(span)
}

fn reserved_in_vue3_diagnostic(name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Name \"{name}\" is reserved in Vue.js 3.x.")).with_label(span)
}

#[derive(Debug, Default, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct NoReservedComponentNames {
    /// Disallow Vue 2 built-in component names (e.g. `Transition`, `KeepAlive`).
    disallow_vue_built_in_components: bool,
    /// Disallow Vue 3 built-in component names (e.g. `Teleport`, `Suspense`).
    /// Note: this also catches Vue 2 built-ins because Vue 3's set includes them.
    disallow_vue3_built_in_components: bool,
    /// Match HTML / SVG element names case-sensitively. When `false` (default),
    /// the capitalized form of an HTML element (e.g. `Div`) is also reported.
    html_element_case_sensitive: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow Vue component names that collide with HTML / SVG element names
    /// (and optionally Vue built-in component names).
    ///
    /// ### Why is this bad?
    ///
    /// Using a reserved name silently shadows the standard element / built-in
    /// component, producing confusing behavior at runtime.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```vue
    /// <script>
    /// export default {
    ///   name: 'div',
    /// }
    /// </script>
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```vue
    /// <script>
    /// export default {
    ///   name: 'MyComponent',
    /// }
    /// </script>
    /// ```
    NoReservedComponentNames,
    vue,
    correctness,
    config = NoReservedComponentNames,
    version = "next",
);

impl Rule for NoReservedComponentNames {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::ObjectExpression(obj) => {
                if !is_vue_component_options_object_excluding_instance(node, ctx) {
                    return;
                }
                self.check_options_object(obj, ctx);
            }
            AstKind::CallExpression(call) => {
                if is_x_dot_component_call(call) && call.arguments.len() == 2 {
                    if let Some(arg) = call.arguments.first().and_then(|a| a.as_expression()) {
                        self.check_name_expression(arg, ctx);
                    }
                } else if is_define_options_call(call) {
                    self.check_define_options(call, ctx);
                }
            }
            _ => {}
        }
    }

    fn should_run(&self, ctx: &crate::context::ContextHost) -> bool {
        ctx.file_extension().is_some_and(|ext| ext == "vue")
    }
}

impl NoReservedComponentNames {
    fn check_options_object<'a>(&self, obj: &ObjectExpression<'a>, ctx: &LintContext<'a>) {
        if let Some(name_prop) = find_property(obj, "name") {
            self.check_name_expression(name_prop.value.get_inner_expression(), ctx);
        }
        let Some(components_prop) = find_property(obj, "components") else { return };
        let Expression::ObjectExpression(components_obj) =
            components_prop.value.get_inner_expression()
        else {
            return;
        };
        for entry in &components_obj.properties {
            let ObjectPropertyKind::ObjectProperty(prop) = entry else { continue };
            let Some(name) = prop.key.static_name() else { continue };
            self.report_if_reserved(name.as_ref(), prop.key.span(), ctx);
        }
    }

    fn check_define_options<'a>(&self, call: &CallExpression<'a>, ctx: &LintContext<'a>) {
        let Some(arg) = call.arguments.first().and_then(|a| a.as_expression()) else { return };
        let Expression::ObjectExpression(obj) = arg.get_inner_expression() else { return };
        let Some(name_prop) = find_property(obj, "name") else { return };
        self.check_name_expression(name_prop.value.get_inner_expression(), ctx);
    }

    fn check_name_expression<'a>(&self, expr: &Expression<'a>, ctx: &LintContext<'a>) {
        match expr {
            Expression::StringLiteral(lit) => {
                self.report_if_reserved(&lit.value, lit.span, ctx);
            }
            Expression::TemplateLiteral(tpl) => {
                if let Some(value) = single_quasi_value(tpl) {
                    self.report_if_reserved(value, tpl.span, ctx);
                }
            }
            _ => {}
        }
    }

    fn report_if_reserved(&self, name: &str, span: Span, ctx: &LintContext<'_>) {
        if self.is_reserved_html(name) {
            ctx.diagnostic(reserved_in_html_diagnostic(name, span));
        } else if self.disallow_vue_built_in_components && VUE2_BUILTIN.contains(name) {
            ctx.diagnostic(reserved_in_vue_diagnostic(name, span));
        } else if self.disallow_vue3_built_in_components {
            if VUE2_BUILTIN.contains(name) {
                ctx.diagnostic(reserved_in_vue_diagnostic(name, span));
            } else if VUE3_BUILTIN_EXTRA.contains(name) {
                ctx.diagnostic(reserved_in_vue3_diagnostic(name, span));
            } else if self.is_reserved_other(name) {
                ctx.diagnostic(reserved_diagnostic(name, span));
            }
        } else if self.is_reserved_other(name) {
            ctx.diagnostic(reserved_diagnostic(name, span));
        }
    }

    fn is_reserved_html(&self, name: &str) -> bool {
        if HTML_ELEMENTS.contains(name) {
            return true;
        }
        if !self.html_element_case_sensitive
            && let Some(lowered) = lower_first_char(name)
            && HTML_ELEMENTS.contains(lowered.as_str())
        {
            return true;
        }
        false
    }

    fn is_reserved_other(&self, name: &str) -> bool {
        if DEPRECATED_HTML_ELEMENTS.contains(name)
            || KEBAB_CASE_ELEMENTS.contains(name)
            || SVG_ELEMENTS.contains(name)
        {
            return true;
        }
        if !self.html_element_case_sensitive {
            if let Some(lowered) = lower_first_char(name) {
                if DEPRECATED_HTML_ELEMENTS.contains(lowered.as_str()) {
                    return true;
                }
                if SVG_ELEMENTS.contains(lowered.as_str()) && is_all_ascii_lowercase(&lowered) {
                    return true;
                }
            }
            if let Some(kebab) = pascal_to_kebab(name)
                && KEBAB_CASE_ELEMENTS.contains(kebab.as_str())
            {
                return true;
            }
        }
        false
    }
}

fn is_x_dot_component_call(call: &CallExpression<'_>) -> bool {
    let Some(member) = call.callee.get_member_expr() else { return false };
    member.static_property_name().is_some_and(|name| name == "component")
}

fn is_define_options_call(call: &CallExpression<'_>) -> bool {
    call.callee.get_identifier_reference().is_some_and(|ident| ident.name == "defineOptions")
}

fn single_quasi_value<'a>(tpl: &'a TemplateLiteral<'a>) -> Option<&'a str> {
    if !tpl.expressions.is_empty() || tpl.quasis.len() != 1 {
        return None;
    }
    tpl.quasis[0].value.cooked.as_deref()
}

fn lower_first_char(name: &str) -> Option<String> {
    let mut chars = name.chars();
    let first = chars.next()?;
    if !first.is_ascii_uppercase() {
        return None;
    }
    Some(first.to_ascii_lowercase().to_string() + chars.as_str())
}

fn is_all_ascii_lowercase(name: &str) -> bool {
    name.chars().all(|c| c.is_ascii_lowercase())
}

/// `AnnotationXml` → `annotation-xml`. Returns `None` if `name` does not look
/// like a PascalCase identifier (i.e. has no internal uppercase transition).
fn pascal_to_kebab(name: &str) -> Option<String> {
    let mut chars = name.chars();
    let first = chars.next()?;
    if !first.is_ascii_uppercase() {
        return None;
    }
    let mut out = String::with_capacity(name.len() + 2);
    out.push(first.to_ascii_lowercase());
    let mut had_uppercase_break = false;
    for c in chars {
        if c.is_ascii_uppercase() {
            out.push('-');
            out.push(c.to_ascii_lowercase());
            had_uppercase_break = true;
        } else {
            out.push(c);
        }
    }
    had_uppercase_break.then_some(out)
}

#[test]
fn test() {
    use crate::tester::Tester;
    use std::path::PathBuf;

    let pass = vec![
        ("<script>export default {}</script>", None, None, Some(PathBuf::from("test.vue"))),
        (
            "<script>export default { ...name }</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "<script>export default { name: 'FooBar' }</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "<script>export default { name: 'FooBar' }</script>",
            Some(
                serde_json::json!([ { "disallowVueBuiltInComponents": true, "disallowVue3BuiltInComponents": true } ]),
            ),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "<script>Vue.component('FooBar', {})</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "<script>app.component('FooBar', {})</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // .js file is not a Vue SFC so `new Vue({ name: 'foo!bar' })` should be ignored
        ("new Vue({ name: 'foo!bar' })", None, None, Some(PathBuf::from("test.js"))),
        (
            "<script>Vue.component(`fooBar${foo}`, component)</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "<script>export default { template: '<template><div /></template>' }</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "<script>export default { template: '<template><div><slot></slot></div></template>' }</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        ("fn1(component.data)", None, None, Some(PathBuf::from("test.js"))),
        ("<script setup>defineOptions({})</script>", None, None, Some(PathBuf::from("test.vue"))),
        (
            "<script setup>defineOptions({ ...name })</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "<script setup>defineOptions({ name: 'Foo' })</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
    ];

    let fail = vec![
        // pattern 1: export default { name: '<reserved>' }
        (
            "<script>export default { name: 'div' }</script>",
            None,
            None,
            Some(PathBuf::from("div.vue")),
        ),
        // pattern 2: Vue.component('<reserved>', component)
        (
            "<script>Vue.component('div', component)</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // pattern 3: app.component('<reserved>', component)
        (
            "<script>app.component('div', component)</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // pattern 4: Vue.component(`<reserved>`, {}) (TemplateLiteral)
        ("<script>Vue.component(`div`, {})</script>", None, None, Some(PathBuf::from("test.vue"))),
        // pattern 5: app.component(`<reserved>`, {})
        ("<script>app.component(`div`, {})</script>", None, None, Some(PathBuf::from("test.vue"))),
        // pattern 6: export default { components: { '<reserved>': {} } }
        (
            "<script>export default { components: { 'div': {} } }</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // pattern 7: <script setup> defineOptions({ name: '<reserved>' })
        (
            "<script setup>defineOptions({ name: 'div' })</script>",
            None,
            None,
            Some(PathBuf::from("div.vue")),
        ),
        // pattern 8: htmlElementCaseSensitive option
        (
            "<script setup>defineOptions({ name: 'div' })</script>",
            Some(serde_json::json!([ { "htmlElementCaseSensitive": true } ])),
            None,
            Some(PathBuf::from("div.vue")),
        ),
        // pattern 9: Vue 2 builtin + disallowVueBuiltInComponents
        (
            "<script>export default { name: 'Transition' }</script>",
            Some(serde_json::json!([ { "disallowVueBuiltInComponents": true } ])),
            None,
            Some(PathBuf::from("Transition.vue")),
        ),
        // pattern 10: Vue 2 builtin detected under disallowVue3BuiltInComponents
        // (Vue3 builtin list includes Vue2; messageId is `reservedInVue`)
        (
            "<script>export default { name: 'Transition' }</script>",
            Some(serde_json::json!([ { "disallowVue3BuiltInComponents": true } ])),
            None,
            Some(PathBuf::from("Transition.vue")),
        ),
        // pattern 11: Vue 3 builtin (Teleport) + disallowVue3BuiltInComponents
        (
            "<script>export default { name: 'Teleport' }</script>",
            Some(serde_json::json!([ { "disallowVue3BuiltInComponents": true } ])),
            None,
            Some(PathBuf::from("Teleport.vue")),
        ),
        // pattern 12: case-insensitive HTML (default) — `Div` should match `div`
        (
            "<script>export default { name: 'Div' }</script>",
            None,
            None,
            Some(PathBuf::from("Div.vue")),
        ),
        // pattern 13: kebab-case (annotation-xml)
        (
            "<script>export default { name: 'annotation-xml' }</script>",
            None,
            None,
            Some(PathBuf::from("annotation-xml.vue")),
        ),
        // pattern 14: PascalCase of kebab-case (case insensitive)
        (
            "<script>export default { name: 'AnnotationXml' }</script>",
            None,
            None,
            Some(PathBuf::from("AnnotationXml.vue")),
        ),
        // pattern 15: SVG element
        (
            "<script>export default { name: 'circle' }</script>",
            None,
            None,
            Some(PathBuf::from("circle.vue")),
        ),
        // pattern 16: deprecated HTML element
        (
            "<script>export default { name: 'marquee' }</script>",
            None,
            None,
            Some(PathBuf::from("marquee.vue")),
        ),
    ];

    Tester::new(NoReservedComponentNames::NAME, NoReservedComponentNames::PLUGIN, pass, fail)
        .test_and_snapshot();
}
