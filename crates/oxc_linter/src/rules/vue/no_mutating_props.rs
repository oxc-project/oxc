use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use oxc_ast::{
    AstKind,
    ast::{
        BindingIdentifier, BindingPattern, CallExpression, Expression, ObjectExpression,
        ObjectPropertyKind, UnaryOperator,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use rustc_hash::FxHashSet;

use crate::{
    AstNode,
    context::LintContext,
    frameworks::FrameworkOptions,
    rule::{DefaultRuleConfig, Rule},
    utils::{find_property, is_this_object, is_vue_component_options_object},
};

fn no_mutating_props_diagnostic(span: Span, name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Unexpected mutation of \"{name}\" prop."))
        .with_help(
            "Props are read-only. Emit an event and let the parent component perform the mutation.",
        )
        .with_label(span)
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct NoMutatingPropsConfig {
    /// When `true`, only mutations that write directly to a prop itself
    /// (e.g. `props.a = 1`, `props.a++`) are reported; mutations of nested
    /// values (e.g. `props.a.b = 1`, `props.a.push(1)`) are allowed.
    shallow_only: bool,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
pub struct NoMutatingProps(NoMutatingPropsConfig);

// Ported from <https://github.com/vuejs/eslint-plugin-vue/blob/master/lib/rules/no-mutating-props.js>
declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows mutating component props: assignments, update expressions,
    /// `delete`, mutating array methods (`push`, `splice`, ...) and
    /// `Object.assign` applied to props received via the `props` option,
    /// `setup(props)`, or `defineProps()`.
    ///
    /// ### Why is this bad?
    ///
    /// Vue props implement a one-way-down binding: when the parent component
    /// updates, the new value overwrites any local mutation, and Vue warns at
    /// runtime when a prop is reassigned. Mutating a prop makes the data flow
    /// hard to reason about and couples the child to the parent's internal
    /// state. The child should instead emit an event and let the owner of the
    /// data perform the mutation.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```vue
    /// <script setup>
    /// const props = defineProps(['todo', 'items'])
    /// props.todo = 'error'
    /// props.items.push('error')
    /// </script>
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```vue
    /// <script setup>
    /// import { computed } from 'vue'
    /// const props = defineProps(['todo'])
    /// const todoText = computed(() => props.todo.text)
    /// </script>
    /// ```
    ///
    /// ### Options
    ///
    /// ```json
    /// { "vue/no-mutating-props": ["error", { "shallowOnly": false }] }
    /// ```
    ///
    /// - `shallowOnly` (`boolean`, default `false`) — report only mutations
    ///   that reassign the prop itself, allowing mutations of nested values.
    NoMutatingProps,
    vue,
    correctness,
    config = NoMutatingPropsConfig,
    version = "next",
    short_description = "Disallow mutation of component props.",
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MutationKind {
    Assignment,
    Update,
    Call,
}

struct Mutation {
    kind: MutationKind,
    span: Span,
    /// Member-access path from the checked expression to the mutated value,
    /// e.g. for `props.a.b = 1` starting at `props` the path is `["a", "b"]`.
    path: Vec<String>,
}

const MUTATING_METHODS: [&str; 9] =
    ["push", "pop", "shift", "unshift", "reverse", "splice", "sort", "copyWithin", "fill"];

fn is_object_assign_call(call: &CallExpression, arg_span: Span) -> bool {
    if call.arguments.first().is_none_or(|arg| arg.span() != arg_span) {
        return false;
    }
    let Some(member) = call.callee.get_inner_expression().as_member_expression() else {
        return false;
    };
    member.static_property_name() == Some("assign")
        && matches!(
            member.object().get_inner_expression(),
            Expression::Identifier(ident) if ident.name == "Object"
        )
}

fn computed_key_text(expr: &Expression, ctx: &LintContext) -> String {
    match expr.get_inner_expression() {
        Expression::StringLiteral(lit) => lit.value.to_string(),
        Expression::NumericLiteral(lit) => lit.value.to_string(),
        _ => format!("[{}]", ctx.source_range(expr.span())),
    }
}

/// Walks up from `start` and returns the mutation applied to it, if any.
/// Port of `findMutating` from eslint-plugin-vue's `lib/utils/index.ts`.
fn find_mutating<'a>(start: &AstNode<'a>, ctx: &LintContext<'a>) -> Option<Mutation> {
    let mut path: Vec<String> = Vec::new();
    let mut current_id = start.id();
    let mut current_span = start.kind().span();
    loop {
        let parent = ctx.nodes().parent_node(current_id);
        match parent.kind() {
            AstKind::AssignmentExpression(assign) => {
                return (assign.left.span() == current_span).then_some(Mutation {
                    kind: MutationKind::Assignment,
                    span: assign.span,
                    path,
                });
            }
            AstKind::UpdateExpression(update) => {
                return Some(Mutation { kind: MutationKind::Update, span: update.span, path });
            }
            AstKind::UnaryExpression(unary) => {
                return (unary.operator == UnaryOperator::Delete).then_some(Mutation {
                    kind: MutationKind::Update,
                    span: unary.span,
                    path,
                });
            }
            AstKind::CallExpression(call) => {
                if call.callee.span() == current_span
                    && path.last().is_some_and(|name| MUTATING_METHODS.contains(&name.as_str()))
                {
                    path.pop();
                    return Some(Mutation { kind: MutationKind::Call, span: call.span, path });
                }
                if is_object_assign_call(call, current_span) {
                    return Some(Mutation { kind: MutationKind::Call, span: call.span, path });
                }
                return None;
            }
            AstKind::StaticMemberExpression(member) => {
                if member.object.span() != current_span {
                    return None;
                }
                path.push(member.property.name.to_string());
            }
            AstKind::ComputedMemberExpression(member) => {
                if member.object.span() != current_span {
                    return None;
                }
                path.push(computed_key_text(&member.expression, ctx));
            }
            AstKind::ChainExpression(_)
            | AstKind::ParenthesizedExpression(_)
            | AstKind::TSAsExpression(_)
            | AstKind::TSNonNullExpression(_)
            | AstKind::TSSatisfiesExpression(_)
            | AstKind::TSTypeAssertion(_) => {}
            _ => return None,
        }
        current_id = parent.id();
        current_span = parent.kind().span();
    }
}

impl NoMutatingProps {
    /// `shallowOnly` reports only direct writes to the prop itself.
    fn should_report(&self, mutation: &Mutation, is_root_props: bool) -> bool {
        if !self.0.shallow_only {
            return true;
        }
        mutation.path.len() == usize::from(is_root_props)
            && matches!(mutation.kind, MutationKind::Assignment | MutationKind::Update)
    }

    /// Checks all reads of a binding that holds the props object (empty
    /// `path`) or a value destructured out of it (`path` = keys from the
    /// props object down to the binding).
    fn verify_prop_binding<'a>(
        &self,
        binding: &BindingIdentifier<'a>,
        path: &[String],
        ctx: &LintContext<'a>,
    ) {
        let is_root_props = path.is_empty();
        for reference in ctx.scoping().get_resolved_references(binding.symbol_id()) {
            if !reference.is_read() {
                continue;
            }
            let ref_node = ctx.nodes().get_node(reference.node_id());
            let Some(mutation) = find_mutating(ref_node, ctx) else { continue };
            if !self.should_report(&mutation, is_root_props) {
                continue;
            }
            let name = if is_root_props {
                // Mutating the whole binding (`props = {}`) rebinds a local
                // variable and does not touch the props object.
                let Some(first) = mutation.path.first() else { continue };
                first.clone()
            } else {
                // For a destructured prop, mutating the binding itself is
                // only a prop mutation for calls (`arr.push(1)` where the
                // path is already consumed); plain reassignment rebinds.
                if mutation.path.is_empty() && mutation.kind != MutationKind::Call {
                    continue;
                }
                path[0].clone()
            };
            ctx.diagnostic(no_mutating_props_diagnostic(mutation.span, &name));
        }
    }

    /// Recursively visits a binding pattern, yielding every identifier with
    /// its member path relative to the pattern root.
    fn walk_pattern<'a>(
        &self,
        pattern: &BindingPattern<'a>,
        path: &[String],
        ctx: &LintContext<'a>,
    ) {
        match pattern {
            BindingPattern::BindingIdentifier(ident) => {
                self.verify_prop_binding(ident, path, ctx);
            }
            BindingPattern::AssignmentPattern(assignment) => {
                self.walk_pattern(&assignment.left, path, ctx);
            }
            BindingPattern::ObjectPattern(object) => {
                for property in &object.properties {
                    let key = property.key.static_name().map_or_else(
                        || format!("[{}]", ctx.source_range(property.key.span())),
                        |name| name.to_string(),
                    );
                    let mut child_path = path.to_vec();
                    child_path.push(key);
                    self.walk_pattern(&property.value, &child_path, ctx);
                }
                if let Some(rest) = &object.rest {
                    self.walk_pattern(&rest.argument, path, ctx);
                }
            }
            BindingPattern::ArrayPattern(array) => {
                for (index, element) in array.elements.iter().enumerate() {
                    if let Some(element) = element {
                        let mut child_path = path.to_vec();
                        child_path.push(index.to_string());
                        self.walk_pattern(element, &child_path, ctx);
                    }
                }
                if let Some(rest) = &array.rest {
                    let mut child_path = path.to_vec();
                    child_path.push(array.elements.len().to_string());
                    self.walk_pattern(&rest.argument, &child_path, ctx);
                }
            }
        }
    }

    /// Handles `const props = defineProps(...)` and
    /// `const props = withDefaults(defineProps(...), ...)`.
    fn check_define_props<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let mut target = node;
        let mut parent = ctx.nodes().parent_node(target.id());
        if let AstKind::CallExpression(call) = parent.kind()
            && matches!(
                call.callee.get_inner_expression(),
                Expression::Identifier(ident) if ident.name == "withDefaults"
            )
            && call.arguments.first().is_some_and(|arg| arg.span() == target.kind().span())
        {
            target = parent;
            parent = ctx.nodes().parent_node(target.id());
        }
        let AstKind::VariableDeclarator(declarator) = parent.kind() else { return };
        if declarator.init.as_ref().is_none_or(|init| init.span() != target.kind().span()) {
            return;
        }
        self.walk_pattern(&declarator.id, &[], ctx);
    }

    /// Reports `this.<prop>` mutations inside the component's options object.
    fn check_this_mutations<'a>(
        &self,
        options_node: &AstNode<'a>,
        prop_names: &FxHashSet<String>,
        ctx: &LintContext<'a>,
    ) {
        for node in ctx.nodes() {
            // Like upstream `isThis`, the object may be `this` itself or a
            // `const` alias of it (`const vm = this`).
            let name = match node.kind() {
                AstKind::StaticMemberExpression(member) if is_this_object(&member.object, ctx) => {
                    member.property.name.as_str()
                }
                AstKind::ComputedMemberExpression(member)
                    if is_this_object(&member.object, ctx) =>
                {
                    match member.expression.get_inner_expression() {
                        Expression::StringLiteral(lit) => lit.value.as_str(),
                        _ => continue,
                    }
                }
                _ => continue,
            };
            if !prop_names.contains(name) {
                continue;
            }
            if !ctx.nodes().ancestors(node.id()).any(|ancestor| ancestor.id() == options_node.id())
            {
                continue;
            }
            if let Some(mutation) = find_mutating(node, ctx)
                && self.should_report(&mutation, false)
            {
                ctx.diagnostic(no_mutating_props_diagnostic(mutation.span, name));
            }
        }
    }
}

fn collect_prop_names(obj: &ObjectExpression) -> FxHashSet<String> {
    let mut names = FxHashSet::default();
    let Some(props) = find_property(obj, "props") else { return names };
    match props.value.get_inner_expression() {
        Expression::ObjectExpression(object) => {
            for property in &object.properties {
                if let ObjectPropertyKind::ObjectProperty(property) = property
                    && let Some(name) = property.key.static_name()
                {
                    names.insert(name.to_string());
                }
            }
        }
        Expression::ArrayExpression(array) => {
            for element in &array.elements {
                if let Some(Expression::StringLiteral(lit)) =
                    element.as_expression().map(Expression::get_inner_expression)
                {
                    names.insert(lit.value.to_string());
                }
            }
        }
        _ => {}
    }
    names
}

fn setup_first_param<'a, 'b>(obj: &'b ObjectExpression<'a>) -> Option<&'b BindingPattern<'a>> {
    let setup = find_property(obj, "setup")?;
    let params = match setup.value.get_inner_expression() {
        Expression::FunctionExpression(function) => &function.params,
        Expression::ArrowFunctionExpression(function) => &function.params,
        _ => return None,
    };
    params.items.first().map(|param| &param.pattern)
}

impl Rule for NoMutatingProps {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        DefaultRuleConfig::<Self>::from_value(value).map(DefaultRuleConfig::into_inner)
    }

    fn run_once(&self, ctx: &LintContext) {
        if ctx.frameworks_options() == FrameworkOptions::VueSetup {
            for node in ctx.nodes() {
                if let AstKind::CallExpression(call) = node.kind()
                    && matches!(
                        call.callee.get_inner_expression(),
                        Expression::Identifier(ident) if ident.name == "defineProps"
                    )
                {
                    self.check_define_props(node, ctx);
                }
            }
        }

        for node in ctx.nodes() {
            let AstKind::ObjectExpression(obj) = node.kind() else { continue };
            if !is_vue_component_options_object(node, ctx) {
                continue;
            }
            let prop_names = collect_prop_names(obj);
            if !prop_names.is_empty() {
                self.check_this_mutations(node, &prop_names, ctx);
            }
            if let Some(param) = setup_first_param(obj)
                && !matches!(param, BindingPattern::ArrayPattern(_))
            {
                self.walk_pattern(param, &[], ctx);
            }
        }
    }

    fn should_run(&self, ctx: &crate::context::ContextHost) -> bool {
        ctx.file_extension().is_some_and(|ext| ext == "vue")
    }
}

// Test cases ported from
// <https://github.com/vuejs/eslint-plugin-vue/blob/master/tests/lib/rules/no-mutating-props.js>
#[test]
fn test() {
    use std::path::PathBuf;

    use crate::tester::Tester;

    let pass = vec![
        (r#"
                    <template>
                      <div>
                        <div v-if="foo"></div>
                        <div v-if="prop1 = [1, 2]"></div>
                        <div v-if="prop2++"></div>
                        <div v-text="prop3.shift()"></div>
                        <div v-text="prop4.slice(0).shift()"></div>
                        <div v-if="this.prop5 = [1, 2] && this.someProp"></div>
                        <div v-if="this.prop6++ && this.someProp < 10"></div>
                        <div v-text="this.prop7.shift()"></div>
                        <div v-text="this.prop8.slice(0).shift()"></div>
                      </div>
                    </template>
                    <script>
                      export default {
                        props: ['foo']
                      }
                    </script>
                  "#, None, None, Some(PathBuf::from("test.vue"))),
(r#"
                    <template>
                      <div>
                        <input v-model="prop1.text">
                        <input v-model="prop2">
                        <input v-model="this.prop3.text">
                        <input v-model="this.prop4">
                        <input :value="prop5.text" @input="$emit('input', $event.target.value)">
                        <div v-for="prop5 of data">
                          <input v-model="prop5">
                        </div>
                        <div v-for="(prop6, index) of data">
                          <input v-model="prop6">
                        </div>
                        <template v-for="(test, index) of data">
                          <template v-for="(prop6, index) of data">
                            <input v-model="prop6">
                            <div v-text="prop6.shift()"></div>
                          </template>
                        </template>
                      </div>
                    </template>
                    <script>
                      export default {
                        props: ['prop5', 'prop6', 'prop7', 'prop8']
                      }
                    </script>
                  "#, None, None, Some(PathBuf::from("test.vue"))),
(r#"
                    <template>
                      <div>
                        <input v-model="prop1.text">
                        <input v-model="prop2">
                        <input v-model="this.prop3.text">
                        <input v-model="this.prop4">
                      </div>
                    </template>
                    <script>
                      export default {
                        props: ['prop5', 'prop6', 'prop7', 'prop8']
                      }
                    </script>
                  "#, None, None, Some(PathBuf::from("test.vue"))),
(r#"
                    <template>
                      <div>
                        <input v-for="i in prop.slice()">
                        <input v-for="i in prop.foo.slice()">
                      </div>
                    </template>
                    <script>
                      export default {
                        props: ['prop']
                      }
                    </script>
                  "#, None, None, Some(PathBuf::from("test.vue"))),
("
                    <script>
                      export default {
                        props: {
                          todo: {
                            type: Object,
                            required: true
                          },
                          items: {
                            type: Array,
                            default: []
                          }
                        },
                        methods: {
                          openModal() {
                            this.$emit('someEvent', this.todo)
                            const a = this.items.slice(0).push('something') // no mutation because of `slice(0)`
                          }
                        }
                      }
                    </script>
                  ", None, None, Some(PathBuf::from("test.vue"))),
(r#"
                    <template>
                      <div v-if="prop">
                        <div v-for="prop in data">
                          <MyComp @click="prop.foo++"></MyComp>
                          <input v-model="prop">
                        </div>
                        <input v-model="prop()">
                        <input v-model="foo">
                        <input @click="prop().foo++">
                        <input v-model="foo[this]">
                        <input v-model="foo[this.prop]">
                        <input v-model="this">
                        <MyComp @click="bar = {prop: foo++}"></MyComp>
                      </div>
                    </template>
                    <script>
                      export default {
                        props: ['prop'],
                        methods: {
                          onKeydown() {
                            const vm = this
                            foo.prop = 1
                            vm()()()
                            vm.prop()()
                            prop++
                            prop = 1
                            const bar = {prop: foo}
                            prop[this] ++
                          }
                        }
                      }
                    </script>
                  "#, None, None, Some(PathBuf::from("test.vue"))),
(r#"
                    <template>
                      <div>
                        <button @click="foo++"></button>
                        <button @click="foo+=1"></button>
                        <button @click="foo.push($event)"></button>
                        <input v-model="foo">
                        <input v-model="this.foo">
                      </div>
                    </template>
                  "#, None, None, Some(PathBuf::from("test.vue"))),
(r#"
                    <template>
                      <div>
                        <input v-model="prop1.text">
                        <input v-model="this.prop2.text">
                        <button @click="prop3.text = '1'"></button>
                        <button @click="prop3.count++"></button>
                        <button @click="prop3.list.push(1)"></button>
                        <button @click="prop3.parent.text = '2'"></button>
                        <button @click="delete prop3.parent.text"></button>
                      </div>
                    </template>
                    <script>
                      export default {
                        props: ['prop1', 'prop2', 'prop3'],
                        methods: {
                            onKeydown() {
                              this.prop3.text = '2'
                              this.prop3.count ++
                              this.prop3.list.push(1)
                              this.prop3.parent.text = '2'
                              delete this.prop3.parent.text
                            }
                        }
                      }
                    </script>
                  "#, Some(serde_json::json!([{ "shallowOnly": true }])), None, Some(PathBuf::from("test.vue"))),
("
                    <script>
                      export default {
                        setup(props) {
                          props ++
                          props = 1
                          props.push(1)
                        }
                      }
                    </script>
                  ", None, None, Some(PathBuf::from("test.vue"))),
("
                    <script>
                      export default {
                        setup({a}) {
                          a ++
                          a = 1
                        }
                      }
                    </script>
                  ", None, None, Some(PathBuf::from("test.vue"))),
("
                    <script>
                      export default {
                        setup({...props}) {
                          props ++
                          props = 1
                          props.push(1)
                        }
                      }
                    </script>
                  ", None, None, Some(PathBuf::from("test.vue"))),
("
                    <script>
                      export default {
                        ssss(props) {
                          props.a ++
                        }
                      }
                    </script>
                  ", None, None, Some(PathBuf::from("test.vue"))),
("
                    <script>
                      export default {
                        setup(props) {
                          const a = props.a
                        }
                      }
                    </script>
                  ", None, None, Some(PathBuf::from("test.vue"))),
("
                    <script>
                      export default {
                        setup() {
                          props.a++
                        }
                      }
                    </script>
                  ", None, None, Some(PathBuf::from("test.vue"))),
("
                    <script>
                      export default {
                        setup(...props) {
                          props.a++
                        }
                      }
                    </script>
                  ", None, None, Some(PathBuf::from("test.vue"))),
("
                    <script>
                      export default {
                        setup([props]) {
                          props.a++
                        }
                      }
                    </script>
                  ", None, None, Some(PathBuf::from("test.vue"))),
("
                    <script>
                    // not <script setup>
                    const {value} = defineProps({
                      value: Object,
                    })
                    value.value++
                    </script>
                  ", None, None, Some(PathBuf::from("test.vue"))),
("
                    <script>
                    // not <script setup>
                    const {value} = defineProps({
                      value: Object,
                    })
                    value.value++
                    </script>
                    <script setup>
                    value.value++
                    </script>
                  ", None, None, Some(PathBuf::from("test.vue"))),
("
                  <script>
                    export default {
                      props: ['prop'],
                      setup(props) {
                        props.prop.sortAscending()
                      }
                    }
                  </script>", None, None, Some(PathBuf::from("test.vue"))),
("
                    <script>
                      export default {
                        setup(props) {
                            props.prop1.text = '2'
                            props.prop1.count ++
                            props.prop1.list.push(1)
                            props.prop1.parent.text = '2'
                        }
                      }
                    </script>
                  ", Some(serde_json::json!([{ "shallowOnly": true }])), None, Some(PathBuf::from("test.vue"))),
("
                    <script>
                      export default {
                        setup({a,b,c, d: [e, , f]}) {
                          a.foo ++
                          b.foo = 1
                          c.push(1)
            
                          c.x.push(1)
                          delete c.y
                          e.foo++
                          f.foo++
                        }
                      }
                    </script>
                  ", Some(serde_json::json!([{ "shallowOnly": true }])), None, Some(PathBuf::from("test.vue"))),
(r#"
                    <template>
                      <input v-model="foo">
                      <input v-model="bar">
                      <input v-model="Infinity">
                    </template>
                    <script setup>
                    import { ref } from 'vue'
                    import { bar } from './my-script'
                    defineProps({
                      foo: String,
                      bar: String,
                      Infinity: Number
                    })
                    const foo = ref('')
                    </script>
                  "#, None, None, Some(PathBuf::from("test.vue"))),
("
                    <script>
                      export default {
                        props: ['data'],
                        methods: {
                          update() {
                            return Object.assign({}, this.data, { extra: 'value' })
                          }
                        }
                      }
                    </script>
                  ", None, None, Some(PathBuf::from("test.vue"))),
        // additional case (not in upstream tests): a non-const alias is not a
        // safe `this` alias, matching upstream `isThis`
        (
            "
                    <script>
                      export default {
                        props: ['count'],
                        methods: {
                          bump() {
                            let vm = this
                            vm.count++
                          }
                        }
                      }
                    </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
    ];

    // Upstream fail cases that mutate props only from within the <template>
    // block are not ported: oxlint lints the <script> part of SFCs.
    let fail = vec![
        (
            "
                    <script>
                      export default {
                        props: {
                          todo: {
                            type: Object,
                            required: true
                          },
                          items: {
                            type: Array,
                            default: []
                          }
                        },
                        methods: {
                          openModal() {
                            ++this.items
                            this.todo.type = 'completed'
                            this.items.push('something')
                            delete this.todo.type
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
            "
                    <script>
                      export default {
                        props: ['foo', 'bar', 'baz'],
                        methods: {
                          openModal() {
                            this?.foo?.push?.('something')
                            ;(this?.bar)?.push?.('something')
                            ;(this?.baz?.push)?.('something')
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
            "
                    <script>
                      export default {
                        setup(props) {
                          props.a ++
                          props.b = 1
                          props.c.push(1)
                          delete props.d
                        }
                      }
                    </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                    <script>
                      export default {
                        setup({a,b,c, d: [e, , f]}) {
                          a.foo ++
                          b.foo = 1
                          c.push(1)
            
                          c.x.push(1)
                          delete c.y
                          e.foo++
                          f.foo++
                        }
                      }
                    </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                    <script>
                      export default {
                        setup({a: foo, b: [...bar], c: baz = 1}) {
                          foo.x ++
                          delete foo.y
                          bar.x = 1
                          baz.push(1)
                        }
                      }
                    </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                    <script>
                      export default {
                        setup({...props}) {
                          props.a ++
                          props.b = 1
                          props.c.push(1)
                          delete props.d
                        }
                      }
                    </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                    <script>
                      export default {
                        setup(props) {
                          props[a] ++
                        }
                      }
                    </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                    <script>
                      export default {
                        setup({[a]: c}) {
                          c.foo ++
                        }
                      }
                    </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                    <script setup>
                    const props = defineProps({
                      value: String,
                    })
                    props.value++
                    </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                    <script setup>
                    const {value} = defineProps({
                      value: Object,
                    })
                    value.value++
                    </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r#"
                    <script setup lang="ts">
                    const props = withDefaults(defineProps<Props>(), {
                      msg: 'hello'
                    })
                    props.value++
                    </script>
                  "#,
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                    <script>
                      export default {
                        setup(props) {
                          props.a ++
                          props.b = 1
                          props.c.push(1)
                          delete props.d
            
                          function foo() {
                            props.a ++
                          }
                        }
                      }
                    </script>
            
                  ",
            Some(serde_json::json!([{ "shallowOnly": true }])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                    <script>
                      export default {
                        props: ['data'],
                        methods: {
                          update() {
                            return Object.assign(this.data, { extra: 'value' })
                          }
                        }
                      }
                    </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // additional case (not in upstream tests): mutation through a const
        // `this` alias, matching upstream `isThis`
        (
            "
                    <script>
                      export default {
                        props: ['count'],
                        methods: {
                          bump() {
                            const vm = this
                            vm.count++
                          }
                        }
                      }
                    </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
    ];

    Tester::new(NoMutatingProps::NAME, NoMutatingProps::PLUGIN, pass, fail).test_and_snapshot();
}
