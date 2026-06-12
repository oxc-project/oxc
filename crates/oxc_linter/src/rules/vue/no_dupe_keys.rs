use std::borrow::Cow;

use rustc_hash::FxHashSet;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use oxc_ast::{
    AstKind,
    ast::{
        BindingPattern, CallExpression, Expression, ObjectExpression, ObjectPropertyKind,
        PropertyKey, PropertyKind, Statement, TSSignature,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::SymbolId;
use oxc_span::{GetSpan, Span};
use oxc_syntax::number::ToJsString;

use crate::{
    AstNode,
    context::LintContext,
    frameworks::FrameworkOptions,
    rule::{DefaultRuleConfig, Rule},
    utils::{for_each_define_props_type_signature, is_vue_component_options_object},
};

fn duplicate_key_diagnostic(span: Span, name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Duplicate key '{name}'. May cause name collision in script or template tag."
    ))
    .with_label(span)
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct NoDupeKeysConfig {
    /// Additional group names to search for duplicate keys in, on top of the
    /// built-in `props`, `computed`, `data`, `methods` and `setup` groups.
    groups: Vec<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
pub struct NoDupeKeys(Box<NoDupeKeysConfig>);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow duplication of field names.
    ///
    /// ### Why is this bad?
    ///
    /// Duplicate keys in Vue component options (props, data, computed, methods, setup)
    /// can cause unexpected behavior because they may overwrite each other at runtime,
    /// and they cause name collisions in the template.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```vue
    /// <script>
    /// export default {
    ///   props: ['foo'],
    ///   computed: {
    ///     foo() {}
    ///   }
    /// }
    /// </script>
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```vue
    /// <script>
    /// export default {
    ///   props: ['foo'],
    ///   computed: {
    ///     bar() {}
    ///   }
    /// }
    /// </script>
    /// ```
    NoDupeKeys,
    vue,
    correctness,
    version = "next",
    config = NoDupeKeys,
);

impl Rule for NoDupeKeys {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::ObjectExpression(obj) => self.check_component_options(node, obj, ctx),
            AstKind::CallExpression(call) => check_define_props(node, call, ctx),
            _ => {}
        }
    }
}

impl NoDupeKeys {
    fn check_component_options<'a>(
        &self,
        node: &AstNode<'a>,
        obj: &'a ObjectExpression<'a>,
        ctx: &LintContext<'a>,
    ) {
        if !is_vue_component_options_object(node, ctx) {
            return;
        }
        let extra_groups = &self.0.groups;
        // dedup: user-supplied group names may overlap with built-in names
        let groups: FxHashSet<&str> =
            GROUP_NAMES.iter().copied().chain(extra_groups.iter().map(String::as_str)).collect();
        let mut seen: FxHashSet<Cow<'a, str>> = FxHashSet::default();
        // Walk all properties in source order so duplicate group names are both visited
        for prop_kind in &obj.properties {
            let ObjectPropertyKind::ObjectProperty(prop) = prop_kind else { continue };
            let Some(group_name) = static_key_name(&prop.key) else { continue };
            if !groups.contains(group_name.as_ref()) {
                continue;
            }
            collect_group_keys(&prop.value, &mut seen, ctx);
        }
    }
}

fn check_define_props<'a>(node: &AstNode<'a>, call: &'a CallExpression<'a>, ctx: &LintContext<'a>) {
    if ctx.frameworks_options() != FrameworkOptions::VueSetup {
        return;
    }
    if !matches!(call.callee, Expression::Identifier(_))
        || call.callee_name() != Some("defineProps")
    {
        return;
    }
    let props = collect_prop_names_from_call(call, ctx);
    if props.is_empty() {
        return;
    }
    let (props_symbol_ids, renamed) = collect_props_bindings(node, call, ctx);
    let root_scope_id = ctx.scoping().root_scope_id();

    for prop_name in &props {
        if renamed.contains(prop_name.as_ref()) {
            continue;
        }
        // Only look in the module (root) scope — nested function/block bindings are invisible
        let Some(symbol_id) = ctx.scoping().get_binding(root_scope_id, prop_name.as_ref().into())
        else {
            continue;
        };
        let decl = ctx.semantic().symbol_declaration(symbol_id);
        let span = match decl.kind() {
            AstKind::VariableDeclarator(d) => {
                if d.init.as_ref().is_some_and(|init| {
                    // The defineProps call itself counts as a props reference upstream,
                    // so any initializer containing it (e.g. `reactive(defineProps(...))`)
                    // is exempt, as is one referencing the props object or any
                    // destructured binding.
                    init.span().contains_inclusive(call.span)
                        || is_inside_props_reference(init, &props_symbol_ids, ctx)
                }) {
                    continue;
                }
                d.id.span()
            }
            _ => decl.kind().span(),
        };
        ctx.diagnostic(duplicate_key_diagnostic(span, prop_name.as_ref()));
    }
}

const GROUP_NAMES: &[&str] = &["props", "computed", "data", "methods", "setup"];

fn collect_group_keys<'a>(
    value: &'a Expression<'a>,
    seen: &mut FxHashSet<Cow<'a, str>>,
    ctx: &LintContext<'a>,
) {
    // Only unwrap parens: espree has no paren nodes, so upstream sees through them,
    // while a TS `as`-wrapped value is opaque to upstream and must stay opaque here.
    match value.without_parentheses() {
        Expression::ArrayExpression(arr) => {
            for el in &arr.elements {
                let Some(expr) = el.as_expression() else { continue };
                let expr = expr.without_parentheses();
                if let Some(name) = literal_element_name(expr)
                    && !name.is_empty()
                {
                    report_or_add(name, expr.span(), seen, ctx);
                }
            }
        }
        Expression::ObjectExpression(obj) => {
            collect_object_keys(obj, seen, ctx);
        }
        Expression::FunctionExpression(func) => {
            if let Some(body) = &func.body {
                collect_returned_object_keys(&body.statements, seen, ctx);
            }
        }
        Expression::ArrowFunctionExpression(arrow) => {
            if arrow.expression {
                if let Some(Statement::ExpressionStatement(es)) = arrow.body.statements.first()
                    && let Expression::ObjectExpression(obj) = es.expression.without_parentheses()
                {
                    collect_object_keys(obj, seen, ctx);
                }
            } else {
                collect_returned_object_keys(&arrow.body.statements, seen, ctx);
            }
        }
        _ => {}
    }
}

fn collect_returned_object_keys<'a>(
    statements: &'a [Statement<'a>],
    seen: &mut FxHashSet<Cow<'a, str>>,
    ctx: &LintContext<'a>,
) {
    for stmt in statements {
        if let Statement::ReturnStatement(ret) = stmt
            && let Some(ret_expr) = &ret.argument
            && let Expression::ObjectExpression(ret_obj) = ret_expr.without_parentheses()
        {
            collect_object_keys(ret_obj, seen, ctx);
        }
    }
}

fn collect_object_keys<'a>(
    obj: &'a ObjectExpression<'a>,
    seen: &mut FxHashSet<Cow<'a, str>>,
    ctx: &LintContext<'a>,
) {
    let getter_names: FxHashSet<Cow<'a, str>> = obj
        .properties
        .iter()
        .filter_map(|p| {
            let prop = p.as_property()?;
            if prop.kind == PropertyKind::Get { static_key_name(&prop.key) } else { None }
        })
        .collect();

    let mut used_getters: FxHashSet<Cow<'a, str>> = FxHashSet::default();

    for prop_kind in &obj.properties {
        let ObjectPropertyKind::ObjectProperty(prop) = prop_kind else { continue };
        let Some(name) = static_key_name(&prop.key) else { continue };
        // upstream skips empty names (`if (name)`)
        if name.is_empty() {
            continue;
        }

        if prop.kind == PropertyKind::Set
            && getter_names.contains(name.as_ref())
            && !used_getters.contains(name.as_ref())
        {
            used_getters.insert(name);
            continue;
        }

        report_or_add(name, prop.key.span(), seen, ctx);
    }
}

fn report_or_add<'a>(
    name: Cow<'a, str>,
    span: Span,
    seen: &mut FxHashSet<Cow<'a, str>>,
    ctx: &LintContext<'a>,
) {
    if seen.contains(name.as_ref()) {
        ctx.diagnostic(duplicate_key_diagnostic(span, &name));
    } else {
        seen.insert(name);
    }
}

/// Mirrors upstream `getStringLiteralValue`: the prop-name string of a literal array element.
/// Non-string literals are stringified like JS `String(value)`; `null` has no name.
fn literal_element_name<'a>(expr: &Expression<'a>) -> Option<Cow<'a, str>> {
    match expr {
        Expression::StringLiteral(s) => Some(Cow::Borrowed(s.value.as_str())),
        Expression::TemplateLiteral(t) => t.single_quasi().map(Into::into),
        Expression::NumericLiteral(n) => Some(Cow::Owned(n.value.to_js_string())),
        Expression::BooleanLiteral(b) => {
            Some(Cow::Borrowed(if b.value { "true" } else { "false" }))
        }
        Expression::BigIntLiteral(b) => Some(Cow::Borrowed(b.value.as_str())),
        Expression::RegExpLiteral(r) => Some(Cow::Owned(r.regex.to_string())),
        _ => None,
    }
}

/// `PropertyKey::static_name` adjusted to upstream `getStaticPropertyName` semantics:
/// numeric keys are formatted like JS `String(n)` (`1e-7` → "1e-7", not "0.0000001"),
/// a computed `[true]` key is named "true", and a computed `[null]` key has no name
/// (upstream's `getStringLiteralValue` bails on `value == null`; a plain `null` key
/// is an identifier, not this variant).
fn static_key_name<'a>(key: &PropertyKey<'a>) -> Option<Cow<'a, str>> {
    match key {
        PropertyKey::NumericLiteral(n) => Some(Cow::Owned(n.value.to_js_string())),
        PropertyKey::BooleanLiteral(b) => {
            Some(Cow::Borrowed(if b.value { "true" } else { "false" }))
        }
        PropertyKey::NullLiteral(_) => None,
        _ => key.static_name(),
    }
}

// ---- script setup helpers ----

/// Resolve the declarator binding the defineProps result (directly or via `withDefaults`),
/// mirroring upstream `getPropsPattern`, and collect every bound `SymbolId` (for
/// initializer reference checks) plus the prop names renamed via
/// `const { foo: bar } = defineProps(...)`.
fn collect_props_bindings<'a>(
    node: &AstNode<'a>,
    call: &CallExpression<'a>,
    ctx: &LintContext<'a>,
) -> (Vec<SymbolId>, FxHashSet<Cow<'a, str>>) {
    let mut ids = Vec::new();
    let mut renamed = FxHashSet::default();
    let declarator = ctx.nodes().ancestors(node.id()).find_map(|ancestor| {
        if let AstKind::VariableDeclarator(decl) = ancestor.kind() { Some(decl) } else { None }
    });
    let Some(decl) = declarator else { return (ids, renamed) };
    if !decl.init.as_ref().is_some_and(|init| is_define_props_initializer(init, call)) {
        return (ids, renamed);
    }
    collect_binding_symbol_ids(&decl.id, &mut ids);
    if let BindingPattern::ObjectPattern(pat) = &decl.id {
        for prop in &pat.properties {
            if let Some(key_name) = static_key_name(&prop.key)
                && let BindingPattern::BindingIdentifier(val) = &prop.value
                && key_name.as_ref() != val.name.as_str()
            {
                renamed.insert(key_name);
            }
        }
    }
    (ids, renamed)
}

fn collect_binding_symbol_ids(pattern: &BindingPattern, ids: &mut Vec<SymbolId>) {
    match pattern {
        BindingPattern::BindingIdentifier(id) => {
            if let Some(sym) = id.symbol_id.get() {
                ids.push(sym);
            }
        }
        BindingPattern::ObjectPattern(obj) => {
            for prop in &obj.properties {
                collect_binding_symbol_ids(&prop.value, ids);
            }
            if let Some(rest) = &obj.rest {
                collect_binding_symbol_ids(&rest.argument, ids);
            }
        }
        // upstream's extractReferences does not descend into array patterns
        BindingPattern::ArrayPattern(_) => {}
        BindingPattern::AssignmentPattern(assign) => {
            collect_binding_symbol_ids(&assign.left, ids);
        }
    }
}

/// True if any reference to one of `symbol_ids` falls within `init`'s span.
fn is_inside_props_reference(
    init: &Expression,
    symbol_ids: &[SymbolId],
    ctx: &LintContext,
) -> bool {
    if symbol_ids.is_empty() {
        return false;
    }
    let init_span = init.span();
    symbol_ids.iter().any(|&symbol_id| {
        ctx.semantic().symbol_references(symbol_id).any(|reference| {
            let ref_span = ctx.nodes().get_node(reference.node_id()).kind().span();
            init_span.start <= ref_span.start && ref_span.end <= init_span.end
        })
    })
}

fn collect_prop_names_from_call<'a>(
    call: &'a CallExpression<'a>,
    ctx: &LintContext<'a>,
) -> Vec<Cow<'a, str>> {
    let mut props: Vec<Cow<'a, str>> = Vec::new();
    if let Some(arg) = call.arguments.first().and_then(|a| a.as_expression()) {
        match arg.without_parentheses() {
            Expression::ObjectExpression(obj) => {
                for prop_kind in &obj.properties {
                    if let ObjectPropertyKind::ObjectProperty(p) = prop_kind
                        && let Some(name) = static_key_name(&p.key)
                    {
                        props.push(name);
                    }
                }
            }
            Expression::ArrayExpression(arr) => {
                for el in &arr.elements {
                    let Some(expr) = el.as_expression() else { continue };
                    if let Some(name) = literal_element_name(expr.without_parentheses()) {
                        props.push(name);
                    }
                }
            }
            _ => {}
        }
    } else {
        collect_ts_type_prop_names(call, ctx, &mut props);
    }
    props
}

fn collect_ts_type_prop_names<'a>(
    call: &'a CallExpression<'a>,
    ctx: &LintContext<'a>,
    out: &mut Vec<Cow<'a, str>>,
) {
    let Some(type_params) = &call.type_arguments else { return };
    let Some(first_type) = type_params.params.first() else { return };
    for_each_define_props_type_signature(first_type, ctx, &mut |sig| {
        let key = match sig {
            TSSignature::TSPropertySignature(s) => &s.key,
            TSSignature::TSMethodSignature(s) => &s.key,
            _ => return,
        };
        if let Some(name) = static_key_name(key) {
            out.push(name);
        }
    });
}

fn is_define_props_initializer<'a>(init: &'a Expression<'a>, call: &'a CallExpression<'a>) -> bool {
    match init {
        Expression::CallExpression(c) => {
            std::ptr::eq(c.as_ref(), call)
                || (matches!(c.callee, Expression::Identifier(_))
                    && c.callee_name() == Some("withDefaults")
                    && c.arguments
                        .first()
                        .and_then(|a| a.as_expression())
                        .is_some_and(|a| is_define_props_initializer(a, call)))
        }
        _ => false,
    }
}

#[test]
fn test() {
    use std::path::PathBuf;

    use crate::tester::Tester;

    let pass = vec![
        (
            r"
<script>
        export default {
          props: ['foo'],
          computed: {
            bar () {
            }
          },
          data () {
            return {
              dat: null
            }
          },
          data () {
            return
          },
          methods: {
            _foo () {},
            test () {
            }
          }
        }
</script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r"
<script>
        export default {
          props: ['foo'],
          data () {
            return {
              dat: null
            }
          },
          data () {
            return
          },
          methods: {
            _foo () {},
            test () {
            }
          },
          setup () {
            const _foo = () => {}
            const dat = ref(null)
            const bar = computed(() => 'bar')

            return {
              bar
            }
          }
        }
</script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r"
<script>
        export default {
          props: ['foo'],
          computed: {
            bar () {
            }
          },
          data: () => {
            return {
              dat: null
            }
          },
          data: () => {
            return
          },
          methods: {
            _foo () {},
            test () {
            }
          }
        }
</script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r"
<script>
        export default {
          props: ['foo'],
          computed: {
            bar () {
            }
          },
          data: () => ({
            dat: null
          }),
          data: () => {
            return
          },
          methods: {
            _foo () {},
            test () {
            }
          }
        }
</script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r"
<script>
        export default {
          ...foo(),
          props: {
            ...foo(),
            foo: String
          },
          computed: {
            ...mapGetters({
              test: 'getTest'
            }),
            bar: {
              get () {
              }
            }
          },
          data: {
            ...foo(),
            dat: null
          },
          methods: {
            ...foo(),
            test () {
            }
          },
          data () {
            return {
              ...dat
            }
          },
        }
</script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r"
<script>
        export default {
          ...foo(),
          props: {
            ...foo(),
            foo: String
          },
          computed: {
            ...mapGetters({
              test: 'getTest'
            }),
            bar: {
              get () {
              }
            }
          },
          data: {
            ...foo(),
            dat: null
          },
          methods: {
            ...foo(),
            test () {
            }
          },
          data () {
            return {
              ...dat
            }
          },
          setup () {
            const com = computed(() => 1)

            return {
              ...foo(),
              com
            }
          }
        }
</script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r"
<script>
        export default {
          ...foo(),
          props: {
            ...foo(),
            foo: String
          },
          computed: {
            ...mapGetters({
              test: 'getTest'
            }),
            bar: {
              get () {
              }
            }
          },
          data: {
            ...foo(),
            dat: null
          },
          methods: {
            ...foo(),
            test () {
            }
          },
          data: () => {
            return {
              ...dat
            }
          },
        }
</script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r"
<script>
        export default {
          ...foo(),
          props: {
            ...foo(),
            foo: String
          },
          computed: {
            ...mapGetters({
              test: 'getTest'
            }),
            bar: {
              get () {
              }
            }
          },
          data: {
            ...foo(),
            dat: null
          },
          methods: {
            ...foo(),
            test () {
            }
          },
          data: () => ({
            ...dat
          }),
        }
</script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r"
        // @vue/component
        export const compA = {
          props: {
            propA: String
          }
        }

        // @vue/component
        export const compB = {
          props: {
            propA: String
          }
        }
      ",
            None,
            None,
            Some(PathBuf::from("test.js")),
        ),
        (
            r"
<script>
        // @vue/component
        export const compA = {
          props: {
            propA: String
          },
          setup (props) {
            const com = computed(() => props.propA)

            return {
              com
            }
          }
        }

        // @vue/component
        export const compB = {
          props: {
            propA: String
          },
          setup (props) {
            const com = computed(() => props.propA)

            return {
              com
            }
          }
        }
</script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r"
<script>
      export default {
        data () {
          return {
            get foo() {
              return foo
            },
            set foo(v) {
              foo = v
            }
          }
        }
      }
</script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r"
<script>
      export default {
        data () {
          return {
            set foo(v) {
              foo = v
            },
            get foo() {
              return foo
            }
          }
        }
      }
</script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r"
<script>
      export default {
        data () {
          return {
            get foo() {
              return foo
            },
            bar,
            set foo(v) {
              foo = v
            }
          }
        }
      }
</script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r"
<script>
        export default {
          asyncData() {
            return {
              foo: 1
            }
          },
          data() {
            return {
              foo: 2
            }
          },
        }
</script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r"
<script setup>
  defineProps({
    foo: String,
  })
  const bar = 0
</script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r#"
<script setup lang="ts">
  defineProps<{
    foo: string;
  }>();

  const bar = 0
</script>
"#,
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r"
<script setup>
const props = defineProps(['foo', 'bar'])
const { foo, bar } = props
</script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r"
<script setup>
const props = defineProps(['foo', 'bar'])
const foo = props.foo
const bar = props.bar
</script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r"
<script setup>
import {toRefs} from 'vue'
const props = defineProps(['foo', 'bar'])
const { foo, bar } = toRefs(props)
</script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r"
<script setup>
import {toRef} from 'vue'
const props = defineProps(['foo', 'bar'])
const foo = toRef(props, 'foo')
const bar = toRef(props, 'bar')
</script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r"
<script setup>
const {foo,bar} = defineProps(['foo', 'bar'])
</script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r"
<script setup>
const {foo=42,bar='abc'} = defineProps(['foo', 'bar'])
</script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r#"
<script setup lang="ts">
const props = withDefaults(
  defineProps<{
    foo?: string | number
  }>(),
  {
    foo: "Foo",
  }
);
const foo = props.foo
</script>
"#,
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r"
<script setup>
const { foo: renamedFoo, bar: renamedBar } = defineProps(['foo', 'bar'])
const foo = 42
const bar = 'hello'
</script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // External import types (import { Props } from './x') require cross-file TS type
        // resolution which is not supported; skipping this case.

        // fix-2a: nested function's binding is not in root scope — no false positive
        (
            r"
<script setup>
defineProps({foo: String})
function outer() {
  function foo() {}
}
</script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // fix-2b: block-scoped const is not in root scope — no false positive
        (
            r"
<script setup>
defineProps(['foo'])
{
  const foo = 1
}
</script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // fix-1: built-in group name in groups option must not double-report
        (
            r"
<script>
export default {
  props: ['foo', 'bar']
}
</script>
",
            Some(serde_json::json!([{ "groups": ["props"] }])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // fix-3: initializer that references the props variable should be skipped
        (
            r"
<script setup>
const props = defineProps(['foo'])
const foo = computed(() => props.foo)
</script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // fix-4: withDefaults wrapping defineProps — rename collected correctly
        (
            r#"
<script setup lang="ts">
const { foo: renamedFoo } = withDefaults(defineProps<{foo?: string}>(), {foo: 'x'})
const foo = 1
</script>
"#,
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // fix-5: member-expression callee (utils.defineProps) must not be treated as defineProps
        (
            r"
<script setup>
const p = utils.defineProps({ foo: String })
const foo = 1
</script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // new-1: destructured prop binding referenced in initializer must be exempted
        (
            r"
<script setup>
const { foo } = defineProps(['foo', 'bar'])
const bar = computed(() => foo + 1)
</script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // empty template literal names are ignored (upstream `if (name)` guard)
        (
            r"
<script>
export default { props: [``, ``] }
</script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // empty string keys are ignored (upstream `if (name)` guard)
        (
            r"
<script>
export default { data () { return { '': 1, '': 2 } } }
</script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // TS as-expression group value is skipped — only parentheses are transparent
        (
            r#"
<script lang="ts">
export default { props: ['foo'], data: ({ foo: null } as any) }
</script>
"#,
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // TS as-expression return argument is skipped
        (
            r#"
<script lang="ts">
export default { props: ['foo'], data () { return { foo: 1 } as any } }
</script>
"#,
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // numeric keys compare using JS number formatting ('1e-7', not '0.0000001')
        (
            r"
<script>
export default { props: { '0.0000001': String }, data () { return { 1e-7: 1 } } }
</script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // an initializer containing the defineProps call itself is exempt
        // (upstream includes the call node in propReferences)
        (
            r"
<script setup>
const foo = reactive(defineProps(['foo']))
</script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // a computed `[null]` key has no static name upstream
        (
            r"
<script>
export default { props: { [null]: String }, data () { return { 'null': 1 } } }
</script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
    ];

    let fail = vec![
        (
            r"
<script>
        export default {
          props: ['foo'],
          computed: {
            foo () {
            }
          },
          data () {
            return {
              foo: null
            }
          },
          methods: {
            foo () {
            }
          },
          setup () {
            const foo = ref(1)

            return {
              foo
            }
          }
        }
</script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r"
<script>
        export default {
          props: ['foo'],
          computed: {
            foo () {
            }
          },
          data: () => {
            return {
              foo: null
            }
          },
          methods: {
            foo () {
            }
          },
          setup: () => {
            const foo = computed(() => 0)

            return {
              foo
            }
          }
        }
</script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r"
<script>
        export default {
          props: ['foo'],
          computed: {
            foo () {
            }
          },
          data: () => ({
            foo: null
          }),
          methods: {
            foo () {
            }
          }
        }
</script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r"
<script>
        export default {
          props: {
            foo: String
          },
          computed: {
            foo: {
              get () {
              }
            }
          },
          data: {
            foo: null
          },
          methods: {
            foo () {
            }
          },
          setup (props) {
            const foo = computed(() => props.foo)

            return {
              foo
            }
          }
        }
</script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r"
        new Vue({
          foo: {
            bar: String
          },
          data: {
            bar: null
          },
        })
      ",
            Some(serde_json::json!([{ "groups": ["foo"] }])),
            None,
            Some(PathBuf::from("test.js")),
        ),
        (
            r"
<script>
        export default {
          methods: {
            foo () {
              return 0
            }
          },
          setup () {
            const foo = () => 0

            return {
              foo
            }
          }
        }
</script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r"
<script>
        export default {
          methods: {
            foo () {
              return 0
            }
          },
          setup () {
            return {
              foo: () => 0
            }
          }
        }
</script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r"
<script>
        export default {
          methods: {
            foo () {
              return 0
            }
          },
          setup: () => ({
            foo: () => 0
          })
        }
</script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r"
<script>
        export default {
          computed: {
            foo () {
              return 0
            }
          },
          setup: () => ({
            foo: computed(() => 0)
          })
        }
</script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r"
<script>
        export default {
          data() {
            return {
              foo: 0
            }
          },
          setup: () => ({
            foo: ref(0)
          })
        }
</script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r"
<script>
        export default {
          data() {
            return {
              foo: 0
            }
          },
          setup: () => ({
            foo: 0
          })
        }
</script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r"
        defineComponent({
          foo: {
            bar: String
          },
          data: {
            bar: null
          },
        })
      ",
            Some(serde_json::json!([{ "groups": ["foo"] }])),
            None,
            Some(PathBuf::from("test.js")),
        ),
        (
            r"
        export default defineComponent({
          foo: {
            bar: String
          },
          data: {
            bar: null
          },
        })
      ",
            Some(serde_json::json!([{ "groups": ["foo"] }])),
            None,
            Some(PathBuf::from("test.js")),
        ),
        (
            r"
<script>
      export default {
        props: ['foo'],
        data () {
          return {
            get foo() {
              return foo
            },
            set foo(v) {
              foo = v
            }
          }
        }
      }
</script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r"
<script>
      export default {
        props: ['foo'],
        data () {
          return {
            set foo(v) {},
            get foo() {}
          }
        }
      }
</script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r"
<script>
      export default {
        props: ['foo'],
        data () {
          return {
            set foo(v) {}
          }
        }
      }
</script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r"
<script>
      export default {
        data () {
          return {
            get foo() {},
            set foo(v) {},
            get foo() {},
          }
        }
      }
</script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r"
<script>
      export default {
        data () {
          return {
            get foo() {},
            set foo(v) {},
            set foo(v) {},
          }
        }
      }
</script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r"
<script setup>
  defineProps({
    foo: String,
  })
  const foo = 0
</script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r"
<script setup>
  import { Foo } from './Foo.vue';
  import baz from './baz';

  defineProps({
    foo: String,
    bar: String,
    baz: String,
  });

  function foo() {
    const baz = 'baz';
  }
  const bar = () => 'bar';
</script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r#"
<script setup lang="ts">
defineProps<{
  foo: string;
  bar: string;
}>();

const foo = 'foo';
const bar = 'bar';
</script>
"#,
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r"
<script setup>
const props = defineProps(['foo', 'bar'])
const { foo } = props
const bar = 42
</script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r"
<script setup>
const { foo: renamedFoo } = defineProps(['foo', 'bar'])
const foo = 'foo'
const bar = 'bar'
</script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // External import type case (import { Props1 as Props } from './test01') requires
        // cross-file TS type resolution which is not supported; skipping this case.

        // fix-7a: class declaration at root scope conflicts with prop name
        (
            r"
<script setup>
defineProps({ Foo: String })
class Foo {}
</script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // fix-7b: destructuring binding at root scope conflicts with prop name
        (
            r"
<script setup>
defineProps({ foo: String })
const { foo } = useSomething()
</script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // fix-6: duplicate group name (second data) — second occurrence must be reported
        (
            r"
<script>
export default {
  props: ['foo'],
  data () { return {} },
  data () { return { foo: 1 } }
}
</script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // fix-8a: return value wrapped in parens should still be detected
        (
            r"
<script>
export default {
  props: ['foo'],
  data () { return ({ foo: null }) }
}
</script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // fix-8b: numeric property key
        (
            r"
<script>
export default {
  props: { 1: String },
  data () { return { 1: 'x' } }
}
</script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // fix-8c: group value itself wrapped in parens
        (
            r"
<script>
export default { props: ['foo'], data: ({ foo: null }) }
</script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // new-2a: template literal in array prop list (Options API)
        (
            r"
<script>
export default { props: [`foo`], data() { return { foo: 1 } } }
</script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // new-2b: template literal in defineProps array (script setup)
        (
            r"
<script setup>
defineProps([`foo`])
const foo = 1
</script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // array-destructured defineProps binding does not exempt initializers
        // (upstream extractReferences ignores array patterns)
        (
            r"
<script setup>
const [a] = defineProps(['foo', 'bar'])
const foo = computed(() => a)
</script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // parenthesized defineProps argument
        (
            r"
<script setup>
defineProps((['foo']))
const foo = 1
</script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // parenthesized array element in a group
        (
            r"
<script>
export default { props: [('foo')], data () { return { foo: 1 } } }
</script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // parenthesized element in defineProps array
        (
            r"
<script setup>
defineProps([('foo')])
const foo = 1
</script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // non-string literal array elements are stringified like JS String(value)
        (
            r"
<script>
export default { props: [1, true], data () { return { 1: 'x', true: 'y' } } }
</script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // numeric keys use JS exponent formatting ('1e-7', not '0.0000001')
        (
            r"
<script>
export default { props: { 1e-7: String }, data () { return { '1e-7': 1 } } }
</script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // a plain `null` key is an identifier and is a real key name
        (
            r"
<script>
export default { data () { return { null: 1, null: 2 } } }
</script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // a computed `[true]` key is stringified like JS String(value)
        (
            r"
<script>
export default { props: { [true]: String }, data () { return { 'true': 1 } } }
</script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
    ];

    Tester::new(NoDupeKeys::NAME, NoDupeKeys::PLUGIN, pass, fail).test_and_snapshot();
}
