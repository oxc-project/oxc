use std::borrow::Cow;

use cow_utils::CowUtils;

use oxc_ast::{
    AstKind,
    ast::{
        ArrowFunctionExpression, BindingPattern, CallExpression, ChainElement, Expression,
        Function, FunctionBody, ObjectExpression, ObjectPattern, ObjectPropertyKind, PropertyKey,
        ReturnStatement, TSLiteral, TSSignature, TSType,
    },
};
use oxc_ast_visit::{Visit, walk};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::scope::ScopeFlags;

use crate::{
    AstNode,
    context::LintContext,
    frameworks::FrameworkOptions,
    rule::Rule,
    utils::{find_property, for_each_define_props_type_signature, is_vue_component_options_object},
};

/// Constructor names that Vue recognizes as prop types.
const NATIVE_TYPES: [&str; 8] =
    ["String", "Number", "Boolean", "Function", "Object", "Array", "Symbol", "BigInt"];

/// Types whose default must be produced by a factory function.
const FUNCTION_VALUE_TYPES: [&str; 3] = ["Function", "Object", "Array"];

fn invalid_type_diagnostic(span: Span, name: &str, types: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Type of the default value for '{name}' prop must be a {types}."))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct RequireValidDefaultProp;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces that the default value of a prop matches its declared type.
    ///
    /// ### Why is this bad?
    ///
    /// A default value whose type does not match the prop's declared `type`
    /// silently violates the component's own contract. `Object` and `Array`
    /// defaults must additionally be returned from a factory function, because
    /// a literal would be shared across every component instance.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```vue
    /// <script>
    /// export default {
    ///   props: {
    ///     foo: {
    ///       type: Number,
    ///       default: 'abc',
    ///     },
    ///     bar: {
    ///       type: Object,
    ///       default: {},
    ///     },
    ///   }
    /// }
    /// </script>
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```vue
    /// <script>
    /// export default {
    ///   props: {
    ///     foo: {
    ///       type: Number,
    ///       default: 100,
    ///     },
    ///     bar: {
    ///       type: Object,
    ///       default: () => ({}),
    ///     },
    ///   }
    /// }
    /// </script>
    /// ```
    RequireValidDefaultProp,
    vue,
    correctness,
    version = "next",
);

impl Rule for RequireValidDefaultProp {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            // Options API: `export default { props: { foo: { type, default } } }`,
            // `Vue.component(...)`, `Vue.extend(...)`, `defineComponent(...)`, etc.
            AstKind::ObjectExpression(obj) => {
                if !is_vue_component_options_object(node, ctx) {
                    return;
                }
                let Some(props_prop) = find_property(obj, "props") else { return };
                let Expression::ObjectExpression(props_obj) =
                    props_prop.value.get_inner_expression()
                else {
                    return;
                };
                for prop in &props_obj.properties {
                    let ObjectPropertyKind::ObjectProperty(prop) = prop else { continue };
                    process_prop(ctx, &prop.key, &prop.value, &DefaultSources::default());
                }
            }
            // `<script setup>`: `defineProps(...)` / `withDefaults(defineProps(...), {...})`.
            AstKind::CallExpression(call) => {
                if ctx.frameworks_options() != FrameworkOptions::VueSetup {
                    return;
                }
                let Some(ident) = call.callee.get_identifier_reference() else { return };
                match ident.name.as_str() {
                    // A `defineProps` wrapped by `withDefaults` is handled by the
                    // `withDefaults` branch, which also carries the default values.
                    "defineProps" if !is_wrapped_by_with_defaults(node, ctx) => {
                        let sources = DefaultSources {
                            destructure: enclosing_destructure(node, ctx),
                            with_defaults: None,
                        };
                        handle_define_props(call, ctx, &sources);
                    }
                    "withDefaults" if call.arguments.len() == 2 => {
                        handle_with_defaults(node, call, ctx);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

/// Where a default value is declared, mirroring upstream's `DefaultDefine.src`.
#[derive(PartialEq, Eq, Clone, Copy)]
enum DefaultSrc {
    /// `default:` property inside a prop options object.
    DefaultProperty,
    /// A `withDefaults(..., { foo })` entry.
    WithDefaults,
    /// A `const { foo = 1 } = defineProps(...)` destructure default. Factory
    /// functions are not allowed here.
    Assignment,
}

/// Default values supplied outside the prop options object, in `<script setup>`.
#[derive(Default)]
struct DefaultSources<'a> {
    destructure: Option<&'a ObjectPattern<'a>>,
    with_defaults: Option<&'a ObjectExpression<'a>>,
}

impl<'a> DefaultSources<'a> {
    fn with_defaults_expr(&self, name: &str) -> Option<&'a Expression<'a>> {
        let obj = self.with_defaults?;
        obj.properties.iter().find_map(|prop| {
            let ObjectPropertyKind::ObjectProperty(p) = prop else { return None };
            (p.key.static_name().as_deref() == Some(name)).then_some(&p.value)
        })
    }

    fn assignment_expr(&self, name: &str) -> Option<&'a Expression<'a>> {
        let pattern = self.destructure?;
        pattern.properties.iter().find_map(|prop| {
            if prop.key.static_name().as_deref() != Some(name) {
                return None;
            }
            match &prop.value {
                BindingPattern::AssignmentPattern(ap) => Some(&ap.right),
                _ => None,
            }
        })
    }
}

fn handle_with_defaults<'a>(
    node: &AstNode<'a>,
    call: &'a CallExpression<'a>,
    ctx: &LintContext<'a>,
) {
    let Some(first) = call.arguments.first().and_then(|a| a.as_expression()) else { return };
    let Some(second) = call.arguments.get(1).and_then(|a| a.as_expression()) else { return };
    let Expression::CallExpression(define_props) = first.get_inner_expression() else { return };
    if !define_props.callee.get_identifier_reference().is_some_and(|i| i.name == "defineProps") {
        return;
    }
    let with_defaults = match second.get_inner_expression() {
        Expression::ObjectExpression(obj) => Some(obj.as_ref()),
        _ => None,
    };
    let sources = DefaultSources { destructure: enclosing_destructure(node, ctx), with_defaults };
    handle_define_props(define_props, ctx, &sources);
}

fn handle_define_props<'a>(
    call: &'a CallExpression<'a>,
    ctx: &LintContext<'a>,
    sources: &DefaultSources<'a>,
) {
    if let Some(arg) = call.arguments.first().and_then(|a| a.as_expression()) {
        if let Expression::ObjectExpression(obj) = arg.get_inner_expression() {
            for prop in &obj.properties {
                let ObjectPropertyKind::ObjectProperty(p) = prop else { continue };
                process_prop(ctx, &p.key, &p.value, sources);
            }
        }
        return;
    }
    if let Some(type_args) = call.type_arguments.as_ref()
        && let Some(first) = type_args.params.first()
    {
        for_each_define_props_type_signature(first, ctx, &mut |sig| {
            process_type_signature(ctx, sig, sources);
        });
    }
}

fn is_wrapped_by_with_defaults(node: &AstNode, ctx: &LintContext) -> bool {
    matches!(
        ctx.nodes().parent_node(node.id()).kind(),
        AstKind::CallExpression(call)
            if call.callee.get_identifier_reference().is_some_and(|i| i.name == "withDefaults")
    )
}

fn enclosing_destructure<'a>(
    node: &AstNode<'a>,
    ctx: &LintContext<'a>,
) -> Option<&'a ObjectPattern<'a>> {
    ctx.nodes().ancestors(node.id()).find_map(|ancestor| {
        let AstKind::VariableDeclarator(decl) = ancestor.kind() else { return None };
        match &decl.id {
            BindingPattern::ObjectPattern(pattern) => Some(pattern.as_ref()),
            _ => None,
        }
    })
}

/// Validate one prop given its key and declared value (the `{ type, default }`
/// object or a bare type like `Number` / `[Number]`).
fn process_prop<'a>(
    ctx: &LintContext<'a>,
    key: &PropertyKey<'a>,
    value: &'a Expression<'a>,
    sources: &DefaultSources<'a>,
) {
    let (type_names, default_property) =
        if let Expression::ObjectExpression(prop_def) = value.get_inner_expression() {
            let Some(type_prop) = find_property(prop_def, "type") else { return };
            (
                collect_native_types(&type_prop.value),
                find_property(prop_def, "default").map(|p| &p.value),
            )
        } else {
            (collect_native_types(value), None)
        };

    let name = key.static_name();
    let mut defaults: Vec<(DefaultSrc, &Expression)> = Vec::new();
    if let Some(def) = default_property {
        defaults.push((DefaultSrc::DefaultProperty, def));
    }
    if let Some(name) = name.as_deref() {
        if let Some(expr) = sources.with_defaults_expr(name) {
            defaults.push((DefaultSrc::WithDefaults, expr));
        }
        if let Some(expr) = sources.assignment_expr(name) {
            defaults.push((DefaultSrc::Assignment, expr));
        }
    }
    if defaults.is_empty() || type_names.is_empty() {
        return;
    }

    let display = match &name {
        Some(name) => Cow::Borrowed(name.as_ref()),
        None => Cow::Owned(format!("[{}]", key.span().source_text(ctx.source_text()))),
    };
    for (src, expr) in defaults {
        check_default(ctx, &display, &type_names, src, expr);
    }
}

/// Validate one `defineProps<T>()` type-declared prop.
fn process_type_signature<'a>(
    ctx: &LintContext<'a>,
    signature: &TSSignature<'a>,
    sources: &DefaultSources<'a>,
) {
    let TSSignature::TSPropertySignature(sig) = signature else { return };
    let Some(name) = sig.key.static_name() else { return };
    let Some(annotation) = sig.type_annotation.as_ref() else { return };
    let type_names = collect_native_types_from_ts(&annotation.type_annotation);
    if type_names.is_empty() {
        return;
    }

    let mut defaults: Vec<(DefaultSrc, &Expression)> = Vec::new();
    if let Some(expr) = sources.with_defaults_expr(&name) {
        defaults.push((DefaultSrc::WithDefaults, expr));
    }
    if let Some(expr) = sources.assignment_expr(&name) {
        defaults.push((DefaultSrc::Assignment, expr));
    }
    if defaults.is_empty() {
        return;
    }
    for (src, expr) in defaults {
        check_default(ctx, &name, &type_names, src, expr);
    }
}

fn check_default<'a>(
    ctx: &LintContext<'a>,
    display: &str,
    type_names: &[&'static str],
    src: DefaultSrc,
    expr: &'a Expression<'a>,
) {
    let Some(def_type) = get_value_type(expr) else { return };

    if def_type.is_function() {
        if type_names.contains(&"Function") {
            return;
        }
        // Factory functions cannot be used in destructure-default assignments.
        if src == DefaultSrc::Assignment {
            report(ctx, expr.span(), display, type_names);
            return;
        }
        match def_type {
            ValueType::ArrowExpr { body, return_type } => match return_type {
                None => {}
                Some(rt) if type_names.contains(&rt) => {}
                Some(_) => report(ctx, body.span(), display, type_names),
            },
            ValueType::BlockFn(body) => {
                for (return_type, span) in collect_return_types(body) {
                    if type_names.contains(&return_type) {
                        continue;
                    }
                    report(ctx, span, display, type_names);
                }
            }
            ValueType::Plain(_) => {}
        }
        return;
    }

    let value_type = def_type.type_name();
    // An assignment default may carry the value directly. For everything but
    // `Object`/`Array` so may a `default:` property; those two still require a
    // factory function, so they fall through to a report.
    if type_names.contains(&value_type)
        && (src == DefaultSrc::Assignment || !FUNCTION_VALUE_TYPES.contains(&value_type))
    {
        return;
    }

    // Outside assignments, `Object`/`Array` are reported as needing a `Function`.
    let mapped: Vec<&'static str>;
    let report_types: &[&'static str] = if src == DefaultSrc::Assignment {
        type_names
    } else {
        mapped = type_names
            .iter()
            .map(|t| if FUNCTION_VALUE_TYPES.contains(t) { "Function" } else { t })
            .collect();
        &mapped
    };
    report(ctx, expr.span(), display, report_types);
}

fn report(ctx: &LintContext, span: Span, name: &str, types: &[&str]) {
    let types = types.join(" or ");
    ctx.diagnostic(invalid_type_diagnostic(span, name, &types.cow_to_ascii_lowercase()));
}

/// The resolved kind of a default value expression, mirroring upstream's
/// `getValueType`.
enum ValueType<'a> {
    /// A non-function value carrying its native type name.
    Plain(&'static str),
    /// `() => expr`, carrying the body expression and its return type (if any).
    ArrowExpr { body: &'a Expression<'a>, return_type: Option<&'static str> },
    /// A block-bodied factory function whose returns are checked individually.
    BlockFn(&'a FunctionBody<'a>),
}

impl ValueType<'_> {
    fn is_function(&self) -> bool {
        !matches!(self, ValueType::Plain(_))
    }

    fn type_name(&self) -> &'static str {
        match self {
            ValueType::Plain(name) => name,
            _ => "Function",
        }
    }
}

fn get_value_type<'a>(expr: &'a Expression<'a>) -> Option<ValueType<'a>> {
    // skipChainExpression: `Number?.()` is the `Number()` call.
    if let Expression::ChainExpression(chain) = expr {
        return match &chain.expression {
            ChainElement::CallExpression(call) => native_call_type(call),
            _ => None,
        };
    }
    match expr.get_inner_expression() {
        Expression::CallExpression(call) => native_call_type(call),
        Expression::TemplateLiteral(_) | Expression::StringLiteral(_) => {
            Some(ValueType::Plain("String"))
        }
        Expression::NumericLiteral(_) => Some(ValueType::Plain("Number")),
        Expression::BooleanLiteral(_) => Some(ValueType::Plain("Boolean")),
        Expression::BigIntLiteral(_) => Some(ValueType::Plain("BigInt")),
        Expression::ArrayExpression(_) => Some(ValueType::Plain("Array")),
        // Mirrors upstream `capitalize(typeof /x/)`: a regex literal is an `Object`.
        Expression::ObjectExpression(_) | Expression::RegExpLiteral(_) => {
            Some(ValueType::Plain("Object"))
        }
        Expression::FunctionExpression(func) => func.body.as_deref().map(ValueType::BlockFn),
        Expression::ArrowFunctionExpression(arrow) => Some(arrow_value_type(arrow)),
        _ => None,
    }
}

fn native_call_type<'a>(call: &CallExpression<'a>) -> Option<ValueType<'a>> {
    let Expression::Identifier(ident) = &call.callee else { return None };
    as_native_type(&ident.name).map(ValueType::Plain)
}

fn arrow_value_type<'a>(arrow: &'a ArrowFunctionExpression<'a>) -> ValueType<'a> {
    if arrow.expression
        && let Some(body) = arrow.get_expression()
    {
        let return_type = get_value_type(body).map(|vt| vt.type_name());
        return ValueType::ArrowExpr { body, return_type };
    }
    ValueType::BlockFn(&arrow.body)
}

fn as_native_type(name: &str) -> Option<&'static str> {
    NATIVE_TYPES.iter().copied().find(|native| *native == name)
}

/// Collect the native type names declared by a prop's `type` value:
/// `Number` / `[Number, String]` (`as` casts are unwrapped).
fn collect_native_types(value: &Expression) -> Vec<&'static str> {
    let mut out: Vec<&'static str> = Vec::new();
    let mut push = |t: &'static str| {
        if !out.contains(&t) {
            out.push(t);
        }
    };
    match value.get_inner_expression() {
        Expression::Identifier(ident) => {
            if let Some(t) = as_native_type(&ident.name) {
                push(t);
            }
        }
        Expression::ArrayExpression(array) => {
            for element in &array.elements {
                if let Some(Expression::Identifier(ident)) = element.as_expression()
                    && let Some(t) = as_native_type(&ident.name)
                {
                    push(t);
                }
            }
        }
        _ => {}
    }
    out
}

/// Collect the native type names a `defineProps<T>()` member maps to.
fn collect_native_types_from_ts(ts_type: &TSType) -> Vec<&'static str> {
    let mut out: Vec<&'static str> = Vec::new();
    collect_ts(ts_type, &mut out);
    out
}

fn collect_ts(ts_type: &TSType, out: &mut Vec<&'static str>) {
    let mut push = |t: &'static str| {
        if !out.contains(&t) {
            out.push(t);
        }
    };
    match ts_type {
        TSType::TSStringKeyword(_) => push("String"),
        TSType::TSNumberKeyword(_) => push("Number"),
        TSType::TSBooleanKeyword(_) => push("Boolean"),
        TSType::TSLiteralType(literal) => match &literal.literal {
            TSLiteral::StringLiteral(_) | TSLiteral::TemplateLiteral(_) => push("String"),
            TSLiteral::NumericLiteral(_) => push("Number"),
            TSLiteral::BooleanLiteral(_) => push("Boolean"),
            TSLiteral::BigIntLiteral(_) => push("BigInt"),
            TSLiteral::UnaryExpression(_) => {}
        },
        TSType::TSUnionType(union) => {
            for member in &union.types {
                collect_ts(member, out);
            }
        }
        _ => {}
    }
}

/// The native type names of every top-level `return` in a factory function
/// body. Returns inside nested functions are ignored.
fn collect_return_types(body: &FunctionBody) -> Vec<(&'static str, Span)> {
    let mut collector = ReturnTypeCollector { nested_depth: 0, returns: Vec::new() };
    collector.visit_function_body(body);
    collector.returns
}

struct ReturnTypeCollector {
    nested_depth: u32,
    returns: Vec<(&'static str, Span)>,
}

impl<'a> Visit<'a> for ReturnTypeCollector {
    fn visit_function(&mut self, func: &Function<'a>, flags: ScopeFlags) {
        self.nested_depth += 1;
        walk::walk_function(self, func, flags);
        self.nested_depth -= 1;
    }

    fn visit_arrow_function_expression(&mut self, arrow: &ArrowFunctionExpression<'a>) {
        self.nested_depth += 1;
        walk::walk_arrow_function_expression(self, arrow);
        self.nested_depth -= 1;
    }

    fn visit_return_statement(&mut self, stmt: &ReturnStatement<'a>) {
        if self.nested_depth == 0
            && let Some(arg) = &stmt.argument
            && let Some(value_type) = get_value_type(arg)
        {
            self.returns.push((value_type.type_name(), arg.span()));
        }
        walk::walk_return_statement(self, stmt);
    }
}

#[test]
#[expect(clippy::literal_string_with_formatting_args)]
fn test() {
    use std::path::PathBuf;

    use crate::tester::Tester;

    let pass = vec![
        (
            "<script>\nexport default {
                    ...foo,
                    props: { ...foo }
                  }\n</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "<script>\nexport default {
                    props: { foo: null }
                  }\n</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "<script>\nexport default {
                    props: ['foo']
                  }\n</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "<script>\nexport default {
                    props: {
                      foo: {
                        type: [Object, Number],
                        default: 10
                      }
                    }
                  }\n</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "<script>\nVue.component('example', {
                    props: {
                      foo: null,
                      foo: Number,
                      foo: [String, Number],
                      foo: { },
                      foo: { type: String },
                      foo: { type: Number, default: VAR_BAR },
                      foo: { type: Number, default: 100 },
                      foo: { type: Number, default: Number.MAX_VALUE },
                      foo: { type: Number, default: Foo.BAR },
                      foo: { type: {}, default: '' },
                      foo: { type: [String, Number], default: '' },
                      foo: { type: [String, Number], default: 0 },
                      foo: { type: String, default: '' },
                      foo: { type: String, default: `` },
                      foo: { type: Boolean, default: false },
                      foo: { type: Object, default: () => { } },
                      foo: { type: Array, default () { } },
                      foo: { type: String, default () { } },
                      foo: { type: Number, default () { } },
                      foo: { type: Boolean, default () { } },
                      foo: { type: Symbol, default () { } },
                      foo: { type: Array, default () { } },
                      foo: { type: Symbol, default: Symbol('a') },
                      foo: { type: String, default: `Foo` },
                      foo: { type: Foo, default: Foo('a') },
                      foo: { type: String, default: `Foo` },
                      foo: { type: BigInt, default: 1n },
                      foo: { type: String, default: null },
                      foo: { type: String, default () { return Foo } },
                      foo: { type: Number, default () { return Foo } },
                      foo: { type: Object, default () { return Foo } },
                      foo: { type: Object, default: null },
                    }
                  })\n</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "<script lang=\"ts\">\n
                    export default (Vue as VueConstructor<Vue>).extend({
                      props: {
                        foo: {
                          type: [Object, Number],
                          default: 10
                        } as PropOptions<object>
                      }
                    });
                  \n</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // { "parser": tsParser, "ecmaVersion": 6, "sourceType": "module" },
        (
            "<script>\nexport default {
                    props: {
                      foo: {
                        type: [Number],
                        default() {
                          return 10
                        }
                      }
                    }
                  }\n</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "<script>\nexport default {
                    props: {
                      foo: {
                        type: [Function, Number],
                        default() {
                          return 's'
                        }
                      }
                    }
                  }\n</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "<script>\nexport default {
                    props: {
                      foo: {
                        type: [Number],
                        default: () => 10
                      }
                    }
                  }\n</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "<script>\nexport default {
                    props: {
                      foo: {
                        type: [Function, Number],
                        default: () => 's'
                      }
                    }
                  }\n</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "<script>\nexport default {
                    props: {
                      foo: {
                        type: [,Object, Number],
                        default: 10
                      }
                    }
                  }\n</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "<script>\nexport default {
                    props: {
                      foo: {
                        type: Number,
                        default: Number?.()
                      }
                    }
                  }\n</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "<script lang=\"ts\">\nexport default Vue.extend({
                      props: {
                        foo: {
                          type: Array as PropType<string[]>,
                          default: () => []
                        }
                      }
                    });
                  \n</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // { "parser": tsParser, ...languageOptions },
        (
            "<script lang=\"ts\">\nexport default Vue.extend({
                      props: {
                        foo: {
                          type: Object as PropType<{ [key: number]: number }>,
                          default: () => {}
                        }
                      }
                    });
                  \n</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // { "parser": tsParser, "ecmaVersion": 6, "sourceType": "module" },
        (
            "<script lang=\"ts\">\nexport default Vue.extend({
                      props: {
                        foo: {
                          type: Function as PropType<() => number>,
                          default: () => 10
                        }
                      }
                    });
                  \n</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // { "parser": tsParser, "ecmaVersion": 6, "sourceType": "module" },
        (
            r#"<script setup lang="ts">
                  export interface SomePropInterface {
                    someProp?: false | string;
                    str?: 'foo' | 'bar';
                    num?: 1 | 2;
                  }

                  withDefaults(defineProps<SomePropInterface>(), {
                    someProp: false,
                    str: 'foo',
                    num: 1
                  });
                  </script>"#,
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // { "parser": vueEslintParser, "ecmaVersion": 6, "sourceType": "module", "parserOptions": { "parser": require.resolve("@typescript-eslint/parser") } },
        // NOTE: upstream uses getTypeScriptFixtureTestOptions() which needs
        // cross-file / generic TS type resolution (tsgolint territory); excluded.
        (
            "
                  <script setup>
                    const { foo = 'abc' } = defineProps({
                      foo: {
                        type: String,
                      }
                    })
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // { "parser": vueEslintParser },
        (
            r#"
                  <script setup lang="ts">
                    const { foo = [] } = defineProps({
                      foo: {
                        type: Array,
                      }
                    })
                  </script>
                  "#,
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // { "parser": vueEslintParser },
           // NOTE: upstream uses getTypeScriptFixtureTestOptions() which needs
           // cross-file / generic TS type resolution (tsgolint territory); excluded.
    ];

    let fail = vec![
        (
            "<script>\nexport default {
                    props: {
                      foo: {
                        type: [Number, String],
                        default: {}
                      }
                    }
                  }\n</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "<script>\nexport default {
                    props: {
                      foo: {
                        type: [Number, Object],
                        default: {}
                      }
                    }
                  }\n</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "<script>\nexport default {
                    props: {
                      foo: {
                        type: Number,
                        default: ''
                      }
                    }
                  }\n</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "<script>\nexport default {
                    props: {
                      foo: {
                        type: Number,
                        default: false
                      }
                    }
                  }\n</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "<script>\nexport default {
                    props: {
                      foo: {
                        type: Number,
                        default: {}
                      }
                    }
                  }\n</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "<script>\nexport default {
                    props: {
                      foo: {
                        type: Number,
                        default: []
                      }
                    }
                  }\n</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "<script>\nexport default {
                    props: {
                      foo: {
                        type: String,
                        default: 2
                      }
                    }
                  }\n</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "<script>\nexport default {
                    props: {
                      foo: {
                        type: String,
                        default: {}
                      }
                    }
                  }\n</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "<script>\nexport default {
                    props: {
                      foo: {
                        type: String,
                        default: []
                      }
                    }
                  }\n</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "<script>\nexport default {
                    props: {
                      foo: {
                        type: Boolean,
                        default: ''
                      }
                    }
                  }\n</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "<script>\nexport default {
                    props: {
                      foo: {
                        type: Boolean,
                        default: 5
                      }
                    }
                  }\n</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "<script>\nexport default {
                    props: {
                      foo: {
                        type: Boolean,
                        default: {}
                      }
                    }
                  }\n</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "<script>\nexport default {
                    props: {
                      foo: {
                        type: Boolean,
                        default: []
                      }
                    }
                  }\n</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "<script>\nexport default {
                    props: {
                      foo: {
                        type: Object,
                        default: ''
                      }
                    }
                  }\n</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "<script>\nexport default {
                    props: {
                      foo: {
                        type: Object,
                        default: 55
                      }
                    }
                  }\n</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "<script>\nexport default {
                    props: {
                      foo: {
                        type: Object,
                        default: false
                      }
                    }
                  }\n</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "<script>\nexport default {
                    props: {
                      foo: {
                        type: Object,
                        default: {}
                      }
                    }
                  }\n</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "<script>\nexport default {
                    props: {
                      foo: {
                        type: Object,
                        default: []
                      }
                    }
                  }\n</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "<script>\nexport default {
                    props: {
                      foo: {
                        type: Array,
                        default: ''
                      }
                    }
                  }\n</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "<script>\nexport default {
                    props: {
                      foo: {
                        type: Array,
                        default: 55
                      }
                    }
                  }\n</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "<script>\nexport default {
                    props: {
                      foo: {
                        type: Array,
                        default: false
                      }
                    }
                  }\n</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "<script>\nexport default {
                    props: {
                      foo: {
                        type: Array,
                        default: {}
                      }
                    }
                  }\n</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "<script>\nexport default {
                    props: {
                      foo: {
                        type: Array,
                        default: []
                      }
                    }
                  }\n</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "<script>\nexport default {
                    props: {
                      foo: {
                        type: [Object, Number],
                        default: {}
                      }
                    }
                  }\n</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "<script lang=\"ts\">\nexport default (Vue as VueConstructor<Vue>).extend({
                    props: {
                      foo: {
                        type: [Object, Number],
                        default: {}
                      } as PropOptions<object>
                    }
                  });\n</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // { "parser": tsParser, "ecmaVersion": 6, "sourceType": "module" },
        (
            "<script>\nexport default {
                    props: {
                      'foo': {
                        type: Object,
                        default: ''
                      },
                      ['bar']: {
                        type: Object,
                        default: ''
                      },
                      [baz]: {
                        type: Object,
                        default: ''
                      }
                    }
                  }\n</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "<script>\nexport default {
                    props: {
                      foo: {
                        type: String,
                        default: 1n
                      }
                    }
                  }\n</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "<script>\nexport default {
                    props: {
                      foo: {
                        type: Number,
                        default() {
                          return ''
                        }
                      }
                    }
                  }\n</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "<script>\nexport default {
                    props: {
                      foo: {
                        type: Object,
                        default() {
                          return ''
                        }
                      }
                    }
                  }\n</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "<script>\nexport default {
                    props: {
                      foo: {
                        type: String,
                        default() {
                          return 123
                        }
                      }
                    }
                  }\n</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "<script>\nexport default {
                    props: {
                      foo: {
                        type: Number,
                        default: () => {
                          return ''
                        }
                      }
                    }
                  }\n</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "<script>\nexport default {
                    props: {
                      foo: {
                        type: Object,
                        default: () => {
                          return ''
                        }
                      }
                    }
                  }\n</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "<script>\nexport default {
                    props: {
                      foo: {
                        type: String,
                        default: () => {
                          return 123
                        }
                      }
                    }
                  }\n</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "<script>\nexport default {
                    props: {
                      foo: {
                        type: Number,
                        default: () => ''
                      }
                    }
                  }\n</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "<script>\nexport default {
                    props: {
                      foo: {
                        type: Object,
                        default: () => ''
                      }
                    }
                  }\n</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "<script>\nexport default {
                    props: {
                      foo: {
                        type: String,
                        default: () => 123
                      }
                    }
                  }\n</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "<script>\nexport default {
                    props: {
                      foo: {
                        type: Function,
                        default: 1
                      }
                    }
                  }\n</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "<script>\nexport default {
                    props: {
                      foo: {
                        type: [String, Boolean],
                        default() {
                          switch (kind) {
                            case 1: return 1
                            case 2: return '' // OK
                            case 3: return {}
                            case 4: return Foo // ignore?
                            case 5: return () => {}
                            case 6: return false // OK
                          }

                          function foo () {
                            return 1 // ignore?
                          }
                        }
                      }
                    }
                  }\n</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "<script>\nexport default {
                    props: {
                      foo: {
                        type: String,
                        default: Number?.()
                      }
                    }
                  }\n</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "<script lang=\"ts\">\nexport default Vue.extend({
                      props: {
                        foo: {
                          type: Array as PropType<string[]>,
                          default: []
                        }
                      }
                    });
                  \n</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // { "parser": tsParser, "ecmaVersion": 6, "sourceType": "module" },
        (
            "<script lang=\"ts\">\nexport default Vue.extend({
                      props: {
                        foo: {
                          type: Object as PropType<{ [key: number]: number }>,
                          default: {}
                        }
                      }
                    });
                  \n</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // { "parser": tsParser, "ecmaVersion": 6, "sourceType": "module" },
        (
            "<script lang=\"ts\">\nexport default Vue.extend({
                      props: {
                        foo: {
                          type: Function as PropType<() => number>,
                          default: 10
                        }
                      }
                    });
                  \n</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // { "parser": tsParser, "ecmaVersion": 6, "sourceType": "module" },
        (
            "
                  <script setup>
                    defineProps({
                      foo: {
                        type: String,
                        default: () => 123
                      }
                    })
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // { "parser": vueEslintParser, "ecmaVersion": 6, "sourceType": "module" },
        (
            "
                  <script setup lang=\"ts\">
                    withDefaults(defineProps<{foo:string}>(),{
                      foo: () => 123
                    })
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // { "ecmaVersion": 6, "sourceType": "module", "parser": vueEslintParser, "parserOptions": { "parser": require.resolve("@typescript-eslint/parser") } },
        // NOTE: upstream uses getTypeScriptFixtureTestOptions() which needs
        // cross-file / generic TS type resolution (tsgolint territory); excluded.
        (
            "
                  <script setup>
                    const { foo = 123 } = defineProps({
                      foo: String
                    })
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // { "parser": vueEslintParser },
        (
            "
                  <script setup>
                    const { foo = 123 } = defineProps({
                      foo: {
                        type: String,
                        default: 123
                      }
                    })
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // { "parser": vueEslintParser },
        (
            "
                  <script setup>
                    const { foo = [] } = defineProps({
                      foo: {
                        type: Number,
                      }
                    })
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // { "parser": vueEslintParser },
        (
            "
                  <script setup>
                    const { foo = 42 } = defineProps({
                      foo: {
                        type: Array,
                      }
                    })
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // { "parser": vueEslintParser },
        (
            "
                  <script setup>
                    const { foo = [] } = defineProps({
                      foo: {
                        type: Array,
                        default: () => {
                          return 42
                        }
                      }
                    })
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // { "parser": vueEslintParser },
        (
            "
                  <script setup>
                    const { foo = (()=>[]) } = defineProps({
                      foo: {
                        type: Array,
                      }
                    })
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // { "parser": vueEslintParser },
        // NOTE: upstream uses getTypeScriptFixtureTestOptions() which needs
        // cross-file / generic TS type resolution (tsgolint territory); excluded.
        // oxc-specific: a regex literal default is an `Object` (mirrors upstream
        // `capitalize(typeof /x/)`), so it is invalid for a non-object type.
        (
            "<script>
                export default {
                  props: {
                    foo: {
                      type: String,
                      default: /x/
                    }
                  }
                }
              </script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
    ];

    Tester::new(RequireValidDefaultProp::NAME, RequireValidDefaultProp::PLUGIN, pass, fail)
        .test_and_snapshot();
}
