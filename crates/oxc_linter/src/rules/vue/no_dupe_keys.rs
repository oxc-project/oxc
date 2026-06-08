use rustc_hash::FxHashSet;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use oxc_ast::{
    AstKind,
    ast::{
        ArrayExpressionElement, Expression, ImportDeclarationSpecifier, ObjectExpression,
        ObjectPropertyKind, PropertyKey, PropertyKind, Statement,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    context::LintContext,
    frameworks::FrameworkOptions,
    rule::Rule,
    utils::{find_property, is_vue_component_options_object},
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
pub struct NoDupeKeys(NoDupeKeysConfig);

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
        Ok(Self(config))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ObjectExpression(obj) = node.kind() else { return };
        if !is_vue_component_options_object(node, ctx) {
            return;
        }
        let extra_groups = &self.0.groups;
        let mut seen: Vec<(&str, Span)> = Vec::new();
        for group in GROUP_NAMES.iter().copied().chain(extra_groups.iter().map(String::as_str)) {
            let Some(group_prop) = find_property(obj, group) else { continue };
            collect_group_keys(&group_prop.value, &mut seen, ctx);
        }
    }

    fn run_once(&self, ctx: &LintContext) {
        if ctx.frameworks_options() != FrameworkOptions::VueSetup {
            return;
        }
        let Some(define_props_node) = find_define_props_call(ctx) else { return };
        let props = collect_prop_names_from_call(define_props_node, ctx);
        if props.is_empty() {
            return;
        }
        let renamed = collect_renamed_props(define_props_node, ctx);

        for node in ctx.nodes() {
            if !is_script_setup_top_level(node.id(), ctx) {
                continue;
            }
            match node.kind() {
                AstKind::VariableDeclarator(decl) => {
                    if decl
                        .init
                        .as_ref()
                        .is_some_and(|init| is_define_props_initializer(init, define_props_node))
                    {
                        continue;
                    }
                    if decl
                        .init
                        .as_ref()
                        .is_some_and(|init| is_props_access(init, define_props_node, ctx))
                    {
                        continue;
                    }
                    let Some(bound_name) = bound_name_of_declarator(decl) else { continue };
                    check_prop_duplicate(bound_name, decl.id.span(), &props, &renamed, ctx);
                }
                AstKind::Function(func)
                    if func.r#type == oxc_ast::ast::FunctionType::FunctionDeclaration =>
                {
                    let Some(id) = &func.id else { continue };
                    check_prop_duplicate(id.name.as_str(), id.span, &props, &renamed, ctx);
                }
                AstKind::ImportDeclaration(import) => {
                    let Some(specifiers) = &import.specifiers else { continue };
                    for specifier in specifiers {
                        let (name, span) = match specifier {
                            ImportDeclarationSpecifier::ImportDefaultSpecifier(s) => {
                                (s.local.name.as_str(), s.local.span)
                            }
                            ImportDeclarationSpecifier::ImportNamespaceSpecifier(s) => {
                                (s.local.name.as_str(), s.local.span)
                            }
                            ImportDeclarationSpecifier::ImportSpecifier(s) => {
                                (s.local.name.as_str(), s.local.span)
                            }
                        };
                        check_prop_duplicate(name, span, &props, &renamed, ctx);
                    }
                }
                _ => {}
            }
        }
    }
}

const GROUP_NAMES: &[&str] = &["props", "computed", "data", "methods", "setup"];

fn collect_group_keys<'a>(
    value: &'a Expression<'a>,
    seen: &mut Vec<(&'a str, Span)>,
    ctx: &LintContext<'a>,
) {
    match value {
        Expression::ArrayExpression(arr) => {
            for el in &arr.elements {
                if let ArrayExpressionElement::StringLiteral(s) = el {
                    report_or_add(s.value.as_str(), s.span, seen, ctx);
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
                        && let Some(Expression::ObjectExpression(ret_obj)) = &ret.argument
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
                        && let Some(Expression::ObjectExpression(ret_obj)) = &ret.argument
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
    seen: &mut Vec<(&'a str, Span)>,
    ctx: &LintContext<'a>,
) {
    let getter_names: Vec<&str> = obj
        .properties
        .iter()
        .filter_map(|p| {
            let ObjectPropertyKind::ObjectProperty(prop) = p else { return None };
            if prop.kind == PropertyKind::Get { static_key_name(&prop.key) } else { None }
        })
        .collect();

    let mut used_getters: Vec<&str> = Vec::new();

    for prop_kind in &obj.properties {
        let ObjectPropertyKind::ObjectProperty(prop) = prop_kind else { continue };
        let Some(name) = static_key_name(&prop.key) else { continue };

        if prop.kind == PropertyKind::Set
            && let Some(&getter) = getter_names.iter().find(|&&g| g == name)
            && !used_getters.contains(&getter)
        {
            used_getters.push(getter);
            continue;
        }

        report_or_add(name, prop.key.span(), seen, ctx);
    }
}

fn report_or_add<'a>(
    name: &'a str,
    span: Span,
    seen: &mut Vec<(&'a str, Span)>,
    ctx: &LintContext,
) {
    if seen.iter().any(|(n, _)| *n == name) {
        ctx.diagnostic(duplicate_key_diagnostic(span, name));
    } else {
        seen.push((name, span));
    }
}

fn static_key_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(id) => Some(id.name.as_str()),
        PropertyKey::StringLiteral(s) => Some(s.value.as_str()),
        _ => None,
    }
}

// ---- script setup helpers ----

/// True when the node is at the top level of a `<script setup>` block
/// (i.e., not nested inside a function body).
fn is_script_setup_top_level(node_id: oxc_semantic::NodeId, ctx: &LintContext) -> bool {
    for ancestor in ctx.nodes().ancestors(node_id).skip(1) {
        match ancestor.kind() {
            AstKind::Program(_) => return true,
            AstKind::FunctionBody(_) => return false,
            _ => {}
        }
    }
    true
}

fn check_prop_duplicate(
    name: &str,
    span: Span,
    props: &[(String, Span)],
    renamed: &FxHashSet<String>,
    ctx: &LintContext,
) {
    if renamed.contains(name) {
        return;
    }
    if props.iter().any(|(p, _)| p.as_str() == name) {
        ctx.diagnostic(duplicate_key_diagnostic(span, name));
    }
}

fn find_define_props_call<'a>(
    ctx: &LintContext<'a>,
) -> Option<&'a oxc_ast::ast::CallExpression<'a>> {
    for node in ctx.nodes() {
        if let AstKind::CallExpression(call) = node.kind()
            && call.callee_name() == Some("defineProps")
        {
            return Some(call);
        }
    }
    None
}

/// Collects (prop_name, span) pairs from a `defineProps(...)` call.
/// Returns owned Strings to avoid lifetime issues with `static_name()` → `Cow`.
fn collect_prop_names_from_call(
    call: &oxc_ast::ast::CallExpression,
    ctx: &LintContext,
) -> Vec<(String, Span)> {
    let mut props: Vec<(String, Span)> = Vec::new();
    if let Some(arg) = call.arguments.first().and_then(|a| a.as_expression()) {
        match arg {
            Expression::ObjectExpression(obj) => {
                for prop_kind in &obj.properties {
                    if let ObjectPropertyKind::ObjectProperty(p) = prop_kind
                        && let Some(name) = static_key_name(&p.key)
                    {
                        props.push((name.to_string(), p.key.span()));
                    }
                }
            }
            Expression::ArrayExpression(arr) => {
                for el in &arr.elements {
                    if let ArrayExpressionElement::StringLiteral(s) = el {
                        props.push((s.value.to_string(), s.span));
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

fn collect_ts_type_prop_names(
    call: &oxc_ast::ast::CallExpression,
    ctx: &LintContext,
    out: &mut Vec<(String, Span)>,
) {
    let Some(type_params) = &call.type_arguments else { return };
    let Some(first_type) = type_params.params.first() else { return };
    collect_ts_type_keys(first_type, ctx, out);
}

fn collect_ts_type_keys(
    ts_type: &oxc_ast::ast::TSType,
    ctx: &LintContext,
    out: &mut Vec<(String, Span)>,
) {
    use oxc_ast::ast::{TSSignature, TSType, TSTypeName};
    match ts_type {
        TSType::TSTypeLiteral(lit) => {
            for member in &lit.members {
                let key = match member {
                    TSSignature::TSPropertySignature(s) => &s.key,
                    TSSignature::TSMethodSignature(s) => &s.key,
                    _ => continue,
                };
                if let Some(name) = key.static_name() {
                    out.push((name.into_owned(), key.span()));
                }
            }
        }
        TSType::TSTypeReference(r) => {
            let TSTypeName::IdentifierReference(id) = &r.type_name else { return };
            let Some(reference_id) = id.reference_id.get() else { return };
            let Some(symbol_id) = ctx.scoping().get_reference(reference_id).symbol_id() else {
                return;
            };
            let decl_node = ctx.nodes().get_node(ctx.scoping().symbol_declaration(symbol_id));
            match decl_node.kind() {
                AstKind::TSInterfaceDeclaration(iface) => {
                    for member in &iface.body.body {
                        let key = match member {
                            TSSignature::TSPropertySignature(s) => &s.key,
                            TSSignature::TSMethodSignature(s) => &s.key,
                            _ => continue,
                        };
                        if let Some(name) = key.static_name() {
                            out.push((name.into_owned(), key.span()));
                        }
                    }
                }
                AstKind::TSTypeAliasDeclaration(alias) => {
                    collect_ts_type_keys(&alias.type_annotation, ctx, out);
                }
                _ => {}
            }
        }
        TSType::TSIntersectionType(t) => {
            for ty in &t.types {
                collect_ts_type_keys(ty, ctx, out);
            }
        }
        TSType::TSUnionType(t) => {
            for ty in &t.types {
                collect_ts_type_keys(ty, ctx, out);
            }
        }
        _ => {}
    }
}

/// Prop names that are renamed in `const { foo: bar } = defineProps(...)` — `foo` is renamed.
fn collect_renamed_props(
    call: &oxc_ast::ast::CallExpression,
    ctx: &LintContext,
) -> FxHashSet<String> {
    let mut renamed = FxHashSet::default();
    for node in ctx.nodes() {
        let AstKind::VariableDeclarator(decl) = node.kind() else { continue };
        if !decl.init.as_ref().is_some_and(|init| is_exact_define_props(init, call)) {
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

fn bound_name_of_declarator<'a>(decl: &'a oxc_ast::ast::VariableDeclarator<'a>) -> Option<&'a str> {
    if let oxc_ast::ast::BindingPattern::BindingIdentifier(id) = &decl.id {
        Some(id.name.as_str())
    } else {
        None
    }
}

fn is_define_props_initializer<'a>(
    init: &'a Expression<'a>,
    call: &'a oxc_ast::ast::CallExpression<'a>,
) -> bool {
    match init {
        Expression::CallExpression(c) => {
            std::ptr::eq(c.as_ref(), call)
                || (c.callee_name() == Some("withDefaults")
                    && c.arguments
                        .first()
                        .and_then(|a| a.as_expression())
                        .is_some_and(|a| is_define_props_initializer(a, call)))
        }
        _ => false,
    }
}

fn is_exact_define_props<'a>(
    init: &'a Expression<'a>,
    call: &'a oxc_ast::ast::CallExpression<'a>,
) -> bool {
    matches!(init, Expression::CallExpression(c) if std::ptr::eq(c.as_ref(), call))
}

fn is_props_access<'a>(
    init: &'a Expression<'a>,
    define_props_call: &'a oxc_ast::ast::CallExpression<'a>,
    ctx: &LintContext<'a>,
) -> bool {
    let props_var_name = find_props_variable_name(define_props_call, ctx);
    match init {
        Expression::StaticMemberExpression(m) => {
            is_props_identifier(&m.object, props_var_name.as_deref())
        }
        Expression::ComputedMemberExpression(m) => {
            is_props_identifier(&m.object, props_var_name.as_deref())
        }
        Expression::CallExpression(c) => {
            matches!(c.callee_name(), Some("toRefs" | "toRef"))
                && c.arguments
                    .first()
                    .and_then(|a| a.as_expression())
                    .is_some_and(|a| is_props_identifier(a, props_var_name.as_deref()))
        }
        _ => false,
    }
}

fn find_props_variable_name(
    call: &oxc_ast::ast::CallExpression,
    ctx: &LintContext,
) -> Option<String> {
    for node in ctx.nodes() {
        let AstKind::VariableDeclarator(decl) = node.kind() else { continue };
        if !decl.init.as_ref().is_some_and(|i| is_define_props_initializer(i, call)) {
            continue;
        }
        if let Some(name) = bound_name_of_declarator(decl) {
            return Some(name.to_string());
        }
    }
    None
}

fn is_props_identifier(expr: &Expression, props_name: Option<&str>) -> bool {
    let Some(name) = props_name else { return false };
    matches!(expr, Expression::Identifier(id) if id.name.as_str() == name)
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
    ];

    Tester::new(NoDupeKeys::NAME, NoDupeKeys::PLUGIN, pass, fail).test_and_snapshot();
}
