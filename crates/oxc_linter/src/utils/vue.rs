use phf::{Set, phf_set};

use oxc_ast::{
    AstKind,
    ast::{
        CallExpression, ExportDefaultDeclarationKind, Expression, IdentifierReference,
        ObjectExpression, ObjectProperty, ObjectPropertyKind,
    },
};
use oxc_span::GetSpan;

use crate::{
    AstNode, ContextSubHost, LintContext, frameworks::FrameworkOptions,
    module_record::ImportImportName,
};

// These sets mirror eslint-plugin-vue's `vue/no-reserved-component-names`.
// `globals::HTML_TAG` has a broader DOM/ARIA scope and different message
// grouping, so it is not a drop-in replacement for this Vue rule.
pub const VUE_RESERVED_HTML_ELEMENTS: Set<&'static str> = phf_set! {
    "a", "abbr", "address", "area", "article", "aside", "audio", "b", "base", "bdi", "bdo",
    "blockquote", "body", "br", "button", "canvas", "caption", "cite", "code", "col", "colgroup",
    "data", "datalist", "dd", "del", "details", "dfn", "dialog", "div", "dl", "dt", "em", "embed",
    "fencedframe", "fieldset", "figcaption", "figure", "footer", "form", "geolocation", "h1",
    "h2", "h3", "h4", "h5", "h6", "head", "header", "hgroup", "hr", "html", "i", "iframe",
    "img", "input", "ins", "kbd", "label", "legend", "li", "link", "main", "map", "mark",
    "menu", "meta", "meter", "nav", "noscript", "object", "ol", "optgroup", "option", "output",
    "p", "picture", "pre", "progress", "q", "rp", "rt", "ruby", "s", "samp", "script", "search",
    "section", "select", "selectedcontent", "slot", "small", "source", "span", "strong", "style",
    "sub", "summary", "sup", "table", "tbody", "td", "template", "textarea", "tfoot", "th",
    "thead", "time", "title", "tr", "track", "u", "ul", "var", "video", "wbr",
};

pub const VUE_RESERVED_DEPRECATED_HTML_ELEMENTS: Set<&'static str> = phf_set! {
    "acronym", "applet", "basefont", "bgsound", "big", "blink", "center", "dir", "font", "frame",
    "frameset", "isindex", "keygen", "listing", "marquee", "menuitem", "multicol", "nextid",
    "nobr", "noembed", "noframes", "param", "plaintext", "rb", "rtc", "spacer", "strike", "tt",
    "xmp",
};

pub const VUE_RESERVED_SVG_ELEMENTS: Set<&'static str> = phf_set! {
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

pub const VUE_RESERVED_KEBAB_CASE_ELEMENTS: Set<&'static str> = phf_set! {
    "annotation-xml", "color-profile", "font-face", "font-face-src", "font-face-uri",
    "font-face-format", "font-face-name", "missing-glyph",
};

pub const VUE2_BUILTIN_COMPONENT_NAMES: Set<&'static str> = phf_set! {
    "template", "slot", "component", "Component", "transition", "Transition", "transition-group",
    "TransitionGroup", "keep-alive", "KeepAlive",
};

pub const VUE3_BUILTIN_COMPONENT_NAMES_EXTRA: Set<&'static str> = phf_set! {
    "teleport", "Teleport", "suspense", "Suspense",
};

/// Check if any of the other contexts has a default export with the `name` property.
///
/// # Example
///
/// ```vue
/// <script>
/// export default {
///  emits: []
/// }
/// </script>
/// ```
///
/// Check if it has `emits` property with `has_default_exports_property(others, "emits")`
pub fn has_default_exports_property(others: &Vec<&ContextSubHost<'_>>, check_name: &str) -> bool {
    for host in others {
        for other_node in host.semantic().nodes() {
            let AstKind::ExportDefaultDeclaration(export) = other_node.kind() else {
                continue;
            };

            let ExportDefaultDeclarationKind::ObjectExpression(export_obj) = &export.declaration
            else {
                continue;
            };

            let has_emits_exports = export_obj.properties.iter().any(|property| {
                let ObjectPropertyKind::ObjectProperty(property) = property else {
                    return false;
                };

                property.key.name().is_some_and(|name| name == check_name)
            });

            if has_emits_exports {
                return true;
            }
        }
    }

    false
}

pub enum DefineMacroProblem {
    DefineInBoth,
    HasTypeAndArguments,
    EventsNotDefined,
    ReferencingLocally,
}

pub fn check_define_macro_call_expression(
    call_expr: &CallExpression,
    ctx: &LintContext,
    has_export_default_equivalent: bool,
) -> Option<DefineMacroProblem> {
    let has_type_args = call_expr.type_arguments.is_some();

    if has_type_args && has_export_default_equivalent {
        return Some(DefineMacroProblem::DefineInBoth);
    }

    // `defineEmits` has type arguments and js arguments. Vue Compiler allows only one of them.
    if has_type_args && !call_expr.arguments.is_empty() {
        return Some(DefineMacroProblem::HasTypeAndArguments);
    }

    if has_type_args {
        // If there are type arguments, we don't need to check the arguments.
        return None;
    }

    let Some(expression) = call_expr.arguments.first().and_then(|first| first.as_expression())
    else {
        // `defineEmits();` is valid when `export default { emits: [] }` is defined
        if !has_export_default_equivalent {
            return Some(DefineMacroProblem::EventsNotDefined);
        }
        return None;
    };

    if has_export_default_equivalent {
        return Some(DefineMacroProblem::DefineInBoth);
    }

    match expression {
        Expression::ArrayExpression(_) | Expression::ObjectExpression(_) => None,
        Expression::Identifier(identifier) => {
            if !is_non_local_reference(identifier, ctx) {
                return Some(DefineMacroProblem::ReferencingLocally);
            }
            None
        }
        _ => Some(DefineMacroProblem::EventsNotDefined),
    }
}

fn is_non_local_reference(identifier: &IdentifierReference, ctx: &LintContext<'_>) -> bool {
    if let Some(symbol_id) = ctx.semantic().scoping().get_root_binding(identifier.name) {
        return matches!(
            ctx.semantic().symbol_declaration(symbol_id).kind(),
            AstKind::ImportSpecifier(_)
        );
    }

    // variables outside the current `<script>` block are valid.
    // This is the same for unresolved variables.
    true
}

/// Check if the given node is inside a Vue component instance method.
///
/// Vue component instance methods are `function` properties directly on a
/// component options object (e.g. `mounted() {}`) or under `methods` /
/// `computed` / `watch`.
pub fn is_in_vue_component_instance_method(node: &AstNode<'_>, ctx: &LintContext<'_>) -> bool {
    let Some(function_node) = ctx
        .nodes()
        .ancestors(node.id())
        .find(|ancestor| matches!(ancestor.kind(), AstKind::Function(_)))
    else {
        return false;
    };

    let property_node = ctx.nodes().parent_node(function_node.id());
    let AstKind::ObjectProperty(_) = property_node.kind() else {
        return false;
    };

    let object_node = ctx.nodes().parent_node(property_node.id());
    if is_vue_component_options_object(object_node, ctx) {
        return true;
    }

    let container_property_node = ctx.nodes().parent_node(object_node.id());
    if !matches!(container_property_node.kind(), AstKind::ObjectProperty(_)) {
        return false;
    }

    let Some(container_name) = container_property_node
        .kind()
        .as_object_property()
        .and_then(|prop| if prop.computed { None } else { prop.key.static_name() })
    else {
        return false;
    };

    matches!(container_name.as_ref(), "computed" | "methods" | "watch")
        && is_vue_component_options_object(
            ctx.nodes().parent_node(container_property_node.id()),
            ctx,
        )
}

/// What kind of Vue component options object an `ObjectExpression` is.
///
/// Mirrors upstream `getVueObjectType` return values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VueComponentObjectKind {
    /// `export default {...}` in a `.vue` file (outside `<script setup>`).
    Export,
    /// `createApp / defineComponent / Vue.component / app.mixin / ...` argument.
    Definition,
    /// `new Vue({...})` — a runtime instance, not a reusable component.
    Instance,
}

/// Classify the given `ObjectExpression` node as a Vue component options
/// object. Returns `None` if the node is not a Vue options object.
///
/// Recognized forms (see [`VueComponentObjectKind`] for the corresponding kind):
/// - `export default {...}` (`.vue` files only, skipped inside `<script setup>`)
/// - `createApp({...})` / `defineComponent({...})` / `defineNuxtComponent({...})`
/// - `Vue.component(name, {...})` / `Vue.mixin({...})` / `Vue.extend({...})` (Vue 2)
/// - `app.component(name, {...})` / `app.mixin({...})` (Vue 3 app instance)
/// - `component('x', {...})` (destructured)
/// - `new Vue({...})`
pub fn vue_component_options_kind(
    object_node: &AstNode<'_>,
    ctx: &LintContext<'_>,
) -> Option<VueComponentObjectKind> {
    let AstKind::ObjectExpression(object_expr) = object_node.kind() else {
        return None;
    };

    ctx.nodes().ancestors(object_node.id()).find_map(|ancestor| match ancestor.kind() {
        AstKind::ExportDefaultDeclaration(export_default_decl) => {
            if ctx.file_extension().is_none_or(|ext| ext != "vue") {
                return None;
            }
            if ctx.frameworks_options() == FrameworkOptions::VueSetup {
                return None;
            }
            (export_default_decl.declaration.span() == object_expr.span)
                .then_some(VueComponentObjectKind::Export)
        }
        AstKind::CallExpression(call_expr) => (is_last_argument_span(call_expr, object_expr.span)
            && is_vue_component_options_call(call_expr))
        .then_some(VueComponentObjectKind::Definition),
        AstKind::NewExpression(new_expr) => (new_expr
            .arguments
            .first()
            .and_then(|arg| arg.as_expression())
            .is_some_and(|expr| expr.span() == object_expr.span)
            && new_expr.callee.get_identifier_reference().is_some_and(|ident| ident.name == "Vue"))
        .then_some(VueComponentObjectKind::Instance),
        _ => None,
    })
}

/// Whether the given `ObjectExpression` is *any* Vue options object — the
/// counterpart of upstream `executeOnVue` / `defineVueVisitor`.
///
/// See [`vue_component_options_kind`] for the recognized forms.
pub fn is_vue_component_options_object(object_node: &AstNode<'_>, ctx: &LintContext<'_>) -> bool {
    vue_component_options_kind(object_node, ctx).is_some()
}

/// Whether the given `ObjectExpression` is a Vue *component* options object,
/// **excluding** `new Vue({...})` instances — the counterpart of upstream
/// `executeOnVueComponent`.
pub fn is_vue_component_options_object_excluding_instance(
    object_node: &AstNode<'_>,
    ctx: &LintContext<'_>,
) -> bool {
    matches!(
        vue_component_options_kind(object_node, ctx),
        Some(VueComponentObjectKind::Export | VueComponentObjectKind::Definition)
    )
}

fn is_last_argument_span(call_expr: &CallExpression<'_>, span: oxc_span::Span) -> bool {
    call_expr
        .arguments
        .last()
        .and_then(|arg| arg.as_expression())
        .is_some_and(|expr| expr.span() == span)
}

/// Check if the call expression is a Vue component / instance definition call.
///
/// Recognized forms:
/// - `createApp(...)` / `defineComponent(...)` / `defineNuxtComponent(...)` / `component(...)`
/// - `Vue.component(...)` / `Vue.mixin(...)` / `Vue.extend(...)` (Vue 2)
/// - `app.component(...)` / `app.mixin(...)` (Vue 3 app instance)
pub fn is_vue_component_options_call(call_expr: &CallExpression<'_>) -> bool {
    if let Some(ident) = call_expr.callee.get_identifier_reference() {
        return matches!(
            ident.name.as_str(),
            "createApp" | "defineComponent" | "defineNuxtComponent" | "component"
        );
    }

    let Some(member_expr) = call_expr.callee.get_member_expr() else {
        return false;
    };
    let Some(prop_name) = member_expr.static_property_name() else {
        return false;
    };

    if let Expression::Identifier(obj) = member_expr.object().get_inner_expression()
        && obj.name == "Vue"
    {
        return matches!(prop_name, "component" | "mixin" | "extend");
    }

    matches!(prop_name, "component" | "mixin")
}

/// Check whether the identifier is imported as `nextTick` or aliased from `'vue'`.
pub fn is_vue_next_tick_import(ident: &IdentifierReference, ctx: &LintContext<'_>) -> bool {
    let scoping = ctx.scoping();
    let Some(symbol_id) = scoping.get_reference(ident.reference_id()).symbol_id() else {
        return false;
    };
    for entry in &ctx.module_record().import_entries {
        if entry.module_request.name() != "vue" {
            continue;
        }
        let ImportImportName::Name(name_span) = &entry.import_name else {
            continue;
        };
        if name_span.name() != "nextTick" {
            continue;
        }
        if scoping.get_root_binding(entry.local_name.name().into()) == Some(symbol_id) {
            return true;
        }
    }
    false
}

/// Finds the first `ObjectProperty` whose static key matches `name` in the given object.
/// `SpreadElement` entries are skipped.
pub fn find_property<'a, 'b>(
    obj: &'b ObjectExpression<'a>,
    name: &str,
) -> Option<&'b ObjectProperty<'a>> {
    obj.properties.iter().find_map(|prop| {
        let ObjectPropertyKind::ObjectProperty(obj_prop) = prop else { return None };
        obj_prop.key.is_specific_static_name(name).then_some(obj_prop.as_ref())
    })
}
