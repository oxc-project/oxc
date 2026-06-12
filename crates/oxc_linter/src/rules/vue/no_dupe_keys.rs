use rustc_hash::FxHashSet;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use oxc_ast::{
    AstKind,
    ast::{
        ArrayExpressionElement, Expression, ObjectExpression, ObjectPropertyKind, PropertyKind,
        Statement, TSSignature,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::SymbolId;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    context::LintContext,
    frameworks::FrameworkOptions,
    rule::Rule,
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
        let config: NoDupeKeysConfig = if value.is_null() {
            NoDupeKeysConfig::default()
        } else if let Some(arr) = value.as_array() {
            if arr.is_empty() {
                NoDupeKeysConfig::default()
            } else {
                serde_json::from_value(arr[0].clone())?
            }
        } else {
            serde_json::from_value(value)?
        };
        Ok(Self(Box::new(config)))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ObjectExpression(obj) = node.kind() else { return };
        if !is_vue_component_options_object(node, ctx) {
            return;
        }
        let extra_groups = &self.0.groups;
        // dedup: user-supplied group names may overlap with built-in names
        let groups: FxHashSet<&str> =
            GROUP_NAMES.iter().copied().chain(extra_groups.iter().map(String::as_str)).collect();
        let mut seen: FxHashSet<String> = FxHashSet::default();
        // Walk all properties in source order so duplicate group names are both visited
        for prop_kind in &obj.properties {
            let ObjectPropertyKind::ObjectProperty(prop) = prop_kind else { continue };
            let Some(group_name) = prop.key.static_name() else { continue };
            if !groups.contains(group_name.as_ref()) {
                continue;
            }
            collect_group_keys(&prop.value, &mut seen, ctx);
        }
    }

    fn run_once(&self, ctx: &LintContext) {
        if ctx.frameworks_options() != FrameworkOptions::VueSetup {
            return;
        }
        let Some(define_props_call) = find_define_props_call(ctx) else { return };
        let props = collect_prop_names_from_call(define_props_call, ctx);
        if props.is_empty() {
            return;
        }
        let renamed = collect_renamed_props(define_props_call, ctx);
        // Collect all symbols bound by defineProps (simple binding + destructured) for span checks
        let props_symbol_ids = find_props_symbol_ids(define_props_call, ctx);
        let root_scope_id = ctx.scoping().root_scope_id();

        for prop_name in &props {
            if renamed.contains(prop_name.as_str()) {
                continue;
            }
            // Only look in the module (root) scope — nested function/block bindings are invisible
            let Some(symbol_id) =
                ctx.scoping().get_binding(root_scope_id, prop_name.as_str().into())
            else {
                continue;
            };
            let decl = ctx.semantic().symbol_declaration(symbol_id);
            let span = match decl.kind() {
                AstKind::VariableDeclarator(d) => {
                    if d.init.as_ref().is_some_and(|init| {
                        // Initialised directly from defineProps / withDefaults(defineProps(...))
                        is_define_props_initializer(init, define_props_call)
                            // Initialiser references the props object or any destructured binding
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
}

const GROUP_NAMES: &[&str] = &["props", "computed", "data", "methods", "setup"];

fn collect_group_keys<'a>(
    value: &'a Expression<'a>,
    seen: &mut FxHashSet<String>,
    ctx: &LintContext<'a>,
) {
    match value.get_inner_expression() {
        Expression::ArrayExpression(arr) => {
            for el in &arr.elements {
                match el {
                    ArrayExpressionElement::StringLiteral(s) => {
                        report_or_add(s.value.as_str(), s.span, seen, ctx);
                    }
                    ArrayExpressionElement::TemplateLiteral(t) if t.expressions.is_empty() => {
                        if let Some(cooked) = t.quasis.first().and_then(|q| q.value.cooked.as_ref())
                        {
                            report_or_add(cooked.as_str(), t.span, seen, ctx);
                        }
                    }
                    _ => {}
                }
            }
        }
        Expression::ObjectExpression(obj) => {
            collect_object_keys(obj, seen, ctx);
        }
        Expression::FunctionExpression(func) => {
            if let Some(body) = &func.body {
                for stmt in &body.statements {
                    if let Statement::ReturnStatement(ret) = stmt
                        && let Some(ret_expr) = &ret.argument
                        && let Expression::ObjectExpression(ret_obj) =
                            ret_expr.get_inner_expression()
                    {
                        collect_object_keys(ret_obj, seen, ctx);
                    }
                }
            }
        }
        Expression::ArrowFunctionExpression(arrow) => {
            if arrow.expression {
                if let Some(Statement::ExpressionStatement(es)) = arrow.body.statements.first()
                    && let Expression::ObjectExpression(obj) = es.expression.get_inner_expression()
                {
                    collect_object_keys(obj, seen, ctx);
                }
            } else {
                for stmt in &arrow.body.statements {
                    if let Statement::ReturnStatement(ret) = stmt
                        && let Some(ret_expr) = &ret.argument
                        && let Expression::ObjectExpression(ret_obj) =
                            ret_expr.get_inner_expression()
                    {
                        collect_object_keys(ret_obj, seen, ctx);
                    }
                }
            }
        }
        _ => {}
    }
}

fn collect_object_keys<'a>(
    obj: &'a ObjectExpression<'a>,
    seen: &mut FxHashSet<String>,
    ctx: &LintContext<'a>,
) {
    let getter_names: FxHashSet<String> = obj
        .properties
        .iter()
        .filter_map(|p| {
            let ObjectPropertyKind::ObjectProperty(prop) = p else { return None };
            if prop.kind == PropertyKind::Get {
                prop.key.static_name().map(std::borrow::Cow::into_owned)
            } else {
                None
            }
        })
        .collect();

    let mut used_getters: FxHashSet<String> = FxHashSet::default();

    for prop_kind in &obj.properties {
        let ObjectPropertyKind::ObjectProperty(prop) = prop_kind else { continue };
        let Some(name) = prop.key.static_name() else { continue };
        let name_str = name.as_ref();

        if prop.kind == PropertyKind::Set
            && getter_names.contains(name_str)
            && !used_getters.contains(name_str)
        {
            used_getters.insert(name_str.to_string());
            continue;
        }

        report_or_add(name_str, prop.key.span(), seen, ctx);
    }
}

fn report_or_add(name: &str, span: Span, seen: &mut FxHashSet<String>, ctx: &LintContext) {
    if !seen.insert(name.to_string()) {
        ctx.diagnostic(duplicate_key_diagnostic(span, name));
    }
}

// ---- script setup helpers ----

fn find_define_props_call<'a>(
    ctx: &LintContext<'a>,
) -> Option<&'a oxc_ast::ast::CallExpression<'a>> {
    for node in ctx.nodes() {
        if let AstKind::CallExpression(call) = node.kind()
            && matches!(call.callee, Expression::Identifier(_))
            && call.callee_name() == Some("defineProps")
        {
            return Some(call);
        }
    }
    None
}

/// Collect all `SymbolId`s bound by a defineProps assignment, including destructured bindings.
/// Handles `const props = defineProps(...)`, `const { foo } = defineProps(...)`, etc.
fn find_props_symbol_ids(call: &oxc_ast::ast::CallExpression, ctx: &LintContext) -> Vec<SymbolId> {
    let mut ids = Vec::new();
    for node in ctx.nodes() {
        let AstKind::VariableDeclarator(decl) = node.kind() else { continue };
        if !decl.init.as_ref().is_some_and(|init| is_define_props_initializer(init, call)) {
            continue;
        }
        collect_binding_symbol_ids(&decl.id, &mut ids);
    }
    ids
}

fn collect_binding_symbol_ids(pattern: &oxc_ast::ast::BindingPattern, ids: &mut Vec<SymbolId>) {
    use oxc_ast::ast::BindingPattern;
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
        BindingPattern::ArrayPattern(arr) => {
            for el in arr.elements.iter().flatten() {
                collect_binding_symbol_ids(el, ids);
            }
            if let Some(rest) = &arr.rest {
                collect_binding_symbol_ids(&rest.argument, ids);
            }
        }
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
    call: &'a oxc_ast::ast::CallExpression<'a>,
    ctx: &LintContext<'a>,
) -> Vec<String> {
    let mut props: Vec<String> = Vec::new();
    if let Some(arg) = call.arguments.first().and_then(|a| a.as_expression()) {
        match arg {
            Expression::ObjectExpression(obj) => {
                for prop_kind in &obj.properties {
                    if let ObjectPropertyKind::ObjectProperty(p) = prop_kind
                        && let Some(name) = p.key.static_name()
                    {
                        props.push(name.into_owned());
                    }
                }
            }
            Expression::ArrayExpression(arr) => {
                for el in &arr.elements {
                    match el {
                        ArrayExpressionElement::StringLiteral(s) => {
                            props.push(s.value.to_string());
                        }
                        ArrayExpressionElement::TemplateLiteral(t) if t.expressions.is_empty() => {
                            if let Some(cooked) =
                                t.quasis.first().and_then(|q| q.value.cooked.as_ref())
                            {
                                props.push(cooked.to_string());
                            }
                        }
                        _ => {}
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
    call: &'a oxc_ast::ast::CallExpression<'a>,
    ctx: &LintContext<'a>,
    out: &mut Vec<String>,
) {
    let Some(type_params) = &call.type_arguments else { return };
    let Some(first_type) = type_params.params.first() else { return };
    for_each_define_props_type_signature(first_type, ctx, &mut |sig| {
        let key = match sig {
            TSSignature::TSPropertySignature(s) => &s.key,
            TSSignature::TSMethodSignature(s) => &s.key,
            _ => return,
        };
        if let Some(name) = key.static_name() {
            out.push(name.into_owned());
        }
    });
}

/// Prop names that are renamed in `const { foo: bar } = defineProps(...)` — `foo` is renamed.
/// withDefaults wrapping is also handled so `const { foo: bar } = withDefaults(defineProps<...>(), ...)` works.
fn collect_renamed_props(
    call: &oxc_ast::ast::CallExpression,
    ctx: &LintContext,
) -> FxHashSet<String> {
    let mut renamed = FxHashSet::default();
    for node in ctx.nodes() {
        let AstKind::VariableDeclarator(decl) = node.kind() else { continue };
        if !decl.init.as_ref().is_some_and(|init| is_define_props_initializer(init, call)) {
            continue;
        }
        if let oxc_ast::ast::BindingPattern::ObjectPattern(pat) = &decl.id {
            for prop in &pat.properties {
                if let Some(key_name) = prop.key.static_name()
                    && let oxc_ast::ast::BindingPattern::BindingIdentifier(val) = &prop.value
                    && key_name.as_ref() != val.name.as_str()
                {
                    renamed.insert(key_name.into_owned());
                }
            }
        }
    }
    renamed
}

fn is_define_props_initializer<'a>(
    init: &'a Expression<'a>,
    call: &'a oxc_ast::ast::CallExpression<'a>,
) -> bool {
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
    ];

    Tester::new(NoDupeKeys::NAME, NoDupeKeys::PLUGIN, pass, fail).test_and_snapshot();
}
