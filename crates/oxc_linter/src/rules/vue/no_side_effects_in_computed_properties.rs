use oxc_ast::{
    AstKind,
    ast::{CallExpression, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    context::LintContext,
    frameworks::FrameworkOptions,
    module_record::ImportImportName,
    rule::Rule,
    utils::{is_this_object, is_vue_component_options_object},
};

fn unexpected_side_effect_in_property(span: Span, key: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Unexpected side effect in \"{key}\" computed property."))
        .with_label(span)
}

fn unexpected_side_effect_in_function(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected side effect in computed function.").with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoSideEffectsInComputedProperties;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow side effects in computed properties.
    ///
    /// ### Why is this bad?
    ///
    /// It is considered a very bad practice to introduce side effects inside computed properties.
    /// It makes the code unpredictable and hard to understand.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```vue
    /// <script>
    /// export default {
    ///   computed: {
    ///     fullName() {
    ///       this.firstName = 'lorem' // side effect
    ///       return `${this.firstName} ${this.lastName}`
    ///     }
    ///   }
    /// }
    /// </script>
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```vue
    /// <script>
    /// export default {
    ///   computed: {
    ///     fullName() {
    ///       return `${this.firstName} ${this.lastName}`
    ///     }
    ///   }
    /// }
    /// </script>
    /// ```
    NoSideEffectsInComputedProperties,
    vue,
    correctness,
    none,
    version = "next",
);

impl Rule for NoSideEffectsInComputedProperties {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::StaticMemberExpression(mem) = node.kind() else { return };

        match &mem.object {
            // Options API: `this.xxx` mutations
            expr if is_this_object(expr, ctx) => {
                // Special case: `this.$set(...)` — report on the property name span
                if mem.property.name == "$set" {
                    let parent = ctx.nodes().parent_node(node.id());
                    if matches!(parent.kind(), AstKind::CallExpression(_))
                        && let Some(ComputedContext::OptionsApi(key)) =
                            find_computed_context(node, ctx)
                    {
                        let key = key.as_deref().unwrap_or("Unknown");
                        ctx.diagnostic(unexpected_side_effect_in_property(mem.property.span, key));
                    }
                    return;
                }

                // General mutation detection
                let Some(mutation_span) = find_mutation_span(node, ctx) else { return };
                if let Some(ComputedContext::OptionsApi(key)) = find_computed_context(node, ctx) {
                    let key = key.as_deref().unwrap_or("Unknown");
                    ctx.diagnostic(unexpected_side_effect_in_property(mutation_span, key));
                }
            }

            // Options API special case: `Vue.set(...)`
            Expression::Identifier(ident) if ident.name == "Vue" && mem.property.name == "set" => {
                let parent = ctx.nodes().parent_node(node.id());
                if matches!(parent.kind(), AstKind::CallExpression(_))
                    && let Some(ComputedContext::OptionsApi(key)) = find_computed_context(node, ctx)
                {
                    let key = key.as_deref().unwrap_or("Unknown");
                    ctx.diagnostic(unexpected_side_effect_in_property(mem.property.span, key));
                }
            }

            // Composition API: `foo.xxx` mutations where foo might be a setup variable
            Expression::Identifier(ident) => {
                let Some(mutation_span) = find_mutation_span(node, ctx) else { return };
                let Some(ComputedContext::CompositionApi(getter_fn_span)) =
                    find_computed_context(node, ctx)
                else {
                    return;
                };

                if !is_setup_variable(ident, getter_fn_span, ctx) {
                    return;
                }

                ctx.diagnostic(unexpected_side_effect_in_function(mutation_span));
            }

            _ => {}
        }
    }
}

// Describe where a computed getter lives
enum ComputedContext {
    OptionsApi(Option<String>), // key name (None if unnamed/unknown)
    CompositionApi(Span),       // span of the getter function (to detect locally-declared vars)
}

/// Find the computed getter context for `node` by walking up to the nearest enclosing function
/// and checking if that function is a computed getter.
/// Returns None if the nearest function is NOT a computed getter (nested function case).
fn find_computed_context(node: &AstNode<'_>, ctx: &LintContext<'_>) -> Option<ComputedContext> {
    let nodes = ctx.nodes();
    let mut current = nodes.parent_node(node.id());

    loop {
        match current.kind() {
            AstKind::Function(_) | AstKind::ArrowFunctionExpression(_) => {
                // Found the nearest enclosing function — check if it's a computed getter
                return get_computed_getter_context(current, ctx);
            }
            AstKind::Program(_) => return None,
            _ => {
                current = nodes.parent_node(current.id());
            }
        }
    }
}

/// Given a function node, return Some(ComputedContext) if it is a computed getter.
fn get_computed_getter_context(
    fn_node: &AstNode<'_>,
    ctx: &LintContext<'_>,
) -> Option<ComputedContext> {
    let nodes = ctx.nodes();
    let fn_span = fn_node.span();
    let parent = nodes.parent_node(fn_node.id());

    match parent.kind() {
        // Composition API: `computed(fn)` or `computed(() => ...)`
        AstKind::CallExpression(call) if is_vue_computed_call(call, ctx) => {
            return Some(ComputedContext::CompositionApi(fn_span));
        }

        AstKind::ObjectProperty(prop) => {
            // setter is not a getter
            if prop.key.is_specific_static_name("set") {
                return None;
            }

            let grandparent = nodes.parent_node(parent.id());
            let AstKind::ObjectExpression(_) = grandparent.kind() else { return None };
            let great = nodes.parent_node(grandparent.id());

            match great.kind() {
                // Case A: `computed: { key() {} }` or `computed: { key: function() {} }`
                AstKind::ObjectProperty(outer) if outer.key.is_specific_static_name("computed") => {
                    if is_under_vue_root(great, ctx) {
                        let key = prop.key.static_name().map(|s| s.to_string());
                        return Some(ComputedContext::OptionsApi(key));
                    }
                }

                // Case B: `computed: { key: { get() {} } }`
                // fn -> ObjectProperty(get) -> ObjectExpression -> ObjectProperty(key)
                //     -> ObjectExpression(computed body) -> ObjectProperty(computed)
                AstKind::ObjectProperty(key_prop) if prop.key.is_specific_static_name("get") => {
                    let key_obj_expr = nodes.parent_node(great.id());
                    let AstKind::ObjectExpression(_) = key_obj_expr.kind() else { return None };
                    let computed_prop_node = nodes.parent_node(key_obj_expr.id());
                    if let AstKind::ObjectProperty(cp) = computed_prop_node.kind()
                        && cp.key.is_specific_static_name("computed")
                        && is_under_vue_root(computed_prop_node, ctx)
                    {
                        let key = key_prop.key.static_name().map(|s| s.to_string());
                        return Some(ComputedContext::OptionsApi(key));
                    }
                }

                // Case C: Composition API with `computed({ get() {}, set() {} })`
                AstKind::CallExpression(call)
                    if prop.key.is_specific_static_name("get")
                        && is_vue_computed_call(call, ctx) =>
                {
                    return Some(ComputedContext::CompositionApi(fn_span));
                }

                _ => {}
            }
        }

        _ => {}
    }

    None
}

fn is_under_vue_root(node: &AstNode<'_>, ctx: &LintContext<'_>) -> bool {
    ctx.nodes().ancestors(node.id()).any(|a| is_vue_component_options_object(a, ctx))
}

/// Check if a `computed(...)` call uses `computed` imported from `vue`.
fn is_vue_computed_call(call: &CallExpression<'_>, ctx: &LintContext<'_>) -> bool {
    let Expression::Identifier(ident) = call.callee.get_inner_expression() else {
        return false;
    };
    if ident.name != "computed" {
        return false;
    }
    let scoping = ctx.scoping();
    let Some(symbol_id) = scoping.get_reference(ident.reference_id()).symbol_id() else {
        return false;
    };
    ctx.module_record().import_entries.iter().any(|entry| {
        if entry.module_request.name() != "vue" {
            return false;
        }
        let ImportImportName::Name(name_span) = &entry.import_name else { return false };
        if name_span.name() != "computed" {
            return false;
        }
        scoping.get_root_binding(entry.local_name.name().into()) == Some(symbol_id)
    })
}

const MUTATING_METHODS: &[&str] =
    &["push", "pop", "shift", "unshift", "reverse", "splice", "sort", "copyWithin", "fill"];

/// Starting from `start_node` (a StaticMemberExpression where object = this/ident),
/// walk up through parent nodes and detect if the member chain is being mutated.
/// Returns the span of the outermost mutation expression, or None if no mutation found.
fn find_mutation_span(start_node: &AstNode<'_>, ctx: &LintContext<'_>) -> Option<Span> {
    let nodes = ctx.nodes();
    let mut current = start_node;
    // Seed with the starting node's property name so `arr.reverse()` is immediately detectable.
    let mut last_static_name: Option<String> =
        if let AstKind::StaticMemberExpression(m) = start_node.kind() {
            Some(m.property.name.to_string())
        } else {
            None
        };

    loop {
        let parent = nodes.parent_node(current.id());
        match parent.kind() {
            // Walking through static member chain: this.foo.bar...
            AstKind::StaticMemberExpression(mem) if mem.object.span() == current.span() => {
                last_static_name = Some(mem.property.name.to_string());
                current = parent;
            }

            // Walking through computed member: this.foo[bar]...
            AstKind::ComputedMemberExpression(mem) if mem.object.span() == current.span() => {
                last_static_name = None;
                current = parent;
            }

            // ChainExpression is transparent (optional chaining wrapper)
            AstKind::ChainExpression(_) => {
                current = parent;
            }

            // Assignment: `this.xxx = val` or `this.arr[i] = val`
            AstKind::AssignmentExpression(assign) if assign.left.span() == current.span() => {
                return Some(assign.span());
            }

            // Update: `this.xxx++` / `++this.xxx`
            AstKind::UpdateExpression(upd) => {
                return Some(upd.span());
            }

            // Delete: `delete this.xxx`
            AstKind::UnaryExpression(unary)
                if unary.operator == oxc_ast::ast::UnaryOperator::Delete
                    && unary.argument.span() == current.span() =>
            {
                return Some(unary.span());
            }

            // Call expression — check if we are the callee
            AstKind::CallExpression(call) if call.callee.span() == current.span() => {
                if last_static_name
                    .as_deref()
                    .is_some_and(|name| MUTATING_METHODS.contains(&name))
                {
                    return Some(call.span());
                }
                return None;
            }

            // Call expression — check Object.assign(thisNode, ...)
            AstKind::CallExpression(call) if is_object_assign_first_arg(call, current.span()) => {
                return Some(call.span());
            }

            // Anything else: not a mutation
            _ => return None,
        }
    }
}

/// Return true if `call` is `Object.assign(...)` and `first_arg_span` is the span of its
/// first argument (i.e., the thing being mutated).
fn is_object_assign_first_arg(call: &CallExpression<'_>, first_arg_span: Span) -> bool {
    let Some(first_arg) = call.arguments.first() else { return false };
    if first_arg.span() != first_arg_span {
        return false;
    }
    let Expression::StaticMemberExpression(callee) = call.callee.get_inner_expression() else {
        return false;
    };
    let Expression::Identifier(obj) = callee.object.get_inner_expression() else { return false };
    obj.name == "Object" && callee.property.name == "assign"
}

/// Check if `ident` is a "setup variable": declared outside the computed getter
/// (`getter_fn_span`) but within the setup scope (or `<script setup>`).
///
/// In practice we check:
/// 1. The identifier is resolved to a symbol with a declaration.
/// 2. The declaration is NOT inside the computed getter itself.
/// 3. The declaration IS within the setup function body OR is a top-level binding
///    in a `<script setup>` file (FrameworkOptions::VueSetup).
fn is_setup_variable(
    ident: &oxc_ast::ast::IdentifierReference<'_>,
    getter_fn_span: Span,
    ctx: &LintContext<'_>,
) -> bool {
    let scoping = ctx.scoping();
    let Some(symbol_id) = scoping.get_reference(ident.reference_id()).symbol_id() else {
        return false;
    };
    let decl_node = ctx.nodes().get_node(scoping.symbol_declaration(symbol_id));
    let decl_span = decl_node.span();

    // If the variable is declared INSIDE the computed getter itself, it's local → skip
    if getter_fn_span.contains_inclusive(decl_span) {
        return false;
    }

    // Function or class declarations are valid setup variables (they can still be mutated)
    // but we skip if the declaration is neither inside setup ranges nor a top-level script-setup binding

    // <script setup>: all top-level bindings are setup variables
    if ctx.frameworks_options() == FrameworkOptions::VueSetup {
        return true;
    }

    // Options API `setup()`: check if decl is inside a setup() function body
    // Walk ancestors of the declaration node to find if it's inside `setup()`
    is_inside_setup_function(decl_node, ctx)
}

/// Check if `node` is declared within a `setup()` function of a Vue component options object.
fn is_inside_setup_function(node: &AstNode<'_>, ctx: &LintContext<'_>) -> bool {
    // Walk up from node's declaration and check if any ancestor is a setup() function
    // whose parent is a Vue component options property named "setup"
    ctx.nodes().ancestors(node.id()).any(|ancestor| {
        let (AstKind::Function(_) | AstKind::ArrowFunctionExpression(_)) = ancestor.kind() else {
            return false;
        };
        let parent = ctx.nodes().parent_node(ancestor.id());
        let AstKind::ObjectProperty(prop) = parent.kind() else { return false };
        if !prop.key.is_specific_static_name("setup") {
            return false;
        }
        let grandparent = ctx.nodes().parent_node(parent.id());
        let AstKind::ObjectExpression(_) = grandparent.kind() else { return false };
        is_vue_component_options_object(grandparent, ctx)
    })
}

#[test]
fn test() {
    use std::path::PathBuf;

    use crate::tester::Tester;

    let pass = vec![
        (
            "
                  Vue.component('test', {
                    ...foo,
                    computed: {
                      ...test0({}),
                      test1() {
                        return this.firstName + ' ' + this.lastName
                      },
                      test2() {
                        return this.something.slice(0).reverse()
                      },
                      test3() {
                        const example = this.something * 2
                        return example + 'test'
                      },
                      test4() {
                        return {
                          ...this.something,
                          test: 'example'
                        }
                      },
                      test5: {
                        get() {
                          return this.firstName + ' ' + this.lastName
                        },
                        set(newValue) {
                          const names = newValue.split(' ')
                          this.firstName = names[0]
                          this.lastName = names[names.length - 1]
                        }
                      },
                      test6: {
                        get() {
                          return this.something.slice(0).reverse()
                        }
                      },
                      test7: {
                        get() {
                          const example = this.something * 2
                          return example + 'test'
                        }
                      },
                      test8: {
                        get() {
                          return {
                            ...this.something,
                            test: 'example'
                          }
                        }
                      },
                      test9() {
                        return Object.keys(this.a).sort()
                      },
                      test10: {
                        get() {
                          return Object.keys(this.a).sort()
                        }
                      },
                      test11() {
                        const categories = {}

                        this.types.forEach(c => {
                          categories[c.category] = categories[c.category] || []
                          categories[c.category].push(c)
                        })

                        return categories
                      },
                      test12() {
                        return this.types.map(t => {
                          // [].push('xxx')
                          return t
                        })
                      },
                      test13 () {
                        this.someArray.forEach(arr => console.log(arr))
                      }
                    }
                  })
                ",
            None,
            None,
            None,
        ),
        (
            "
                  Vue.component('test', {
                    computed: {
                      ...mapGetters(['example']),
                      test1() {
                        const num = 0
                        const something = {
                          a: 'val',
                          b: ['1', '2']
                        }
                        num++
                        something.a = 'something'
                        something.b.reverse()
                        return something.b
                      }
                    }
                  })
                ",
            None,
            None,
            None,
        ),
        (
            "
                  Vue.component('test', {
                    name: 'something',
                    data() {
                      return {}
                    }
                  })
                ",
            None,
            None,
            None,
        ),
        (
            "
                  Vue.component('test', {
                    computed: {
                      test () {
                        let a;
                        a = this.something
                        return a
                      },
                    }
                  })
                ",
            None,
            None,
            None,
        ),
        (
            "
                  Vue.component('test', {
                    computed: {
                      test () {
                        return {
                          action1() {
                            this.something++
                          },
                          action2() {
                            this.something = 1
                          },
                          action3() {
                            this.something.reverse()
                          }
                        }
                      },
                    }
                  })
                ",
            None,
            None,
            None,
        ),
        (
            "
                  Vue.component('test', {
                    computed: {
                      test () {
                        return this.something['a']().reverse()
                      },
                    }
                  })
                ",
            None,
            None,
            None,
        ),
        (
            "
                  const test = { el: '#app' }
                    Vue.component('test', {
                    el: test.el
                  })
                ",
            None,
            None,
            None,
        ),
        (
            "
                  Vue.component('test', {
                    computed: {
                      test () {
                        return [...this.items].reverse()
                      },
                    }
                  })
                ",
            None,
            None,
            None,
        ),
        (
            "
                  <script>
                  import { computed } from 'vue'
                  const utils = {}
                  export default {
                    setup() {
                      const foo = useFoo()
                      function bar () {}
                      class Baz {}

                      const test0 = computed(test0f)
                      const test1 = computed(() => foo.firstName + ' ' + foo.lastName)
                      const test2 = computed(() => foo.something.slice(0).reverse())
                      const test3 = computed(() => {
                        return {
                          ...foo.something,
                          test: 'example'
                        }
                      })
                      const test5 = computed({
                        get: () => foo.firstName + ' ' + foo.lastName,
                        set: newValue => {
                          const names = newValue.split(' ')
                          foo.firstName = names[0]
                          foo.lastName = names[names.length - 1]
                        }
                      })
                      const test6 = computed({
                        get: () => foo.something.slice(0).reverse()
                      })
                      const test7 = computed({
                        get: () => {
                          const example = foo.something * 2
                          return example + 'test'
                        }
                      })
                      const test8 = computed({
                        get: () => {
                          return {
                            ...foo.something,
                            test: 'example'
                          }
                        }
                      })
                      const test9 = computed(() => Object.keys(foo.a).sort())
                      const test10 = computed({
                        get: () => Object.keys(foo.a).sort()
                      })
                      const test11 = computed(() => {
                        const categories = {}

                        foo.types.forEach(c => {
                          categories[c.category] = categories[c.category] || []
                          categories[c.category].push(c)
                        })

                        return categories
                      })
                      const test12 = computed(() => {
                        return foo.types.map(t => {
                          // [].push('xxx')
                          return t
                        })
                      })
                      const test13 = computed(() => {
                        foo.someArray.forEach(arr => console.log(arr))
                      })
                      const test14 = computed(() => bar.name)
                      const test15 = computed(() => Baz.name)
                      const test16 = computed(() => {
                        function b () {}
                        b.name = 'c'
                      })
                      const test17 = computed(() => {
                        class C {}
                        C.name = 'D'
                      })
                      const test18 = computed(() => (console.log('a'), true))
                      const test19 = computed(() => utils.reverse(foo.array))
                      const test20 = computed(() => Object.assign({}, foo.data, { extra: 'value' }))
                    }
                  }
                  </script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
    ];

    let fail = vec![
        (
            "Vue.component('test', {
                    computed: {
                      test1() {
                        this.firstName = 'lorem'
                        asd.qwe.zxc = 'lorem'
                        return this.firstName + ' ' + this.lastName
                      },
                      test2() {
                        this.count += 2;
                        this.count++;
                        return this.count;
                      },
                      test3() {
                        return this.something.reverse()
                      },
                      test4() {
                        const test = this.another.something.push('example')
                        return 'something'
                      },
                      test5() {
                        this.something[index] = thing[index]
                        return this.something
                      },
                      test6() {
                        return this.something.keys.sort()
                      }
                    }
                  })",
            None,
            None,
            None,
        ),
        (
            "Vue.component('test', {
                    computed: {
                      test1: {
                        get() {
                          this.firstName = 'lorem'
                          return this.firstName + ' ' + this.lastName
                        }
                      },
                      test2: {
                        get() {
                          this.count += 2;
                          this.count++;
                          return this.count;
                        }
                      },
                      test3: {
                        get() {
                          return this.something.reverse()
                        }
                      },
                      test4: {
                        get() {
                          const test = this.another.something.push('example')
                          return 'something'
                        },
                        set(newValue) {
                          this.something = newValue
                        }
                      },
                    }
                  })",
            None,
            None,
            None,
        ),
        (
            "
                  <script lang=\"ts\">
                  export default Vue.extend({
                    computed: {
                      test1() : string {
                        return this.something.reverse()
                      }
                    }
                  });
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "app.component('test', {
                    computed: {
                      test1() {
                        this.firstName = 'lorem'
                        asd.qwe.zxc = 'lorem'
                        return this.firstName + ' ' + this.lastName
                      },
                    }
                  })",
            None,
            None,
            None,
        ),
        (
            "Vue.component('test', {
                    computed: {
                      test1() {
                        return this?.something?.reverse?.()
                      },
                      test2() {
                        return (this?.something)?.reverse?.()
                      },
                      test3() {
                        return (this?.something?.reverse)?.()
                      },
                    }
                  })",
            None,
            None,
            None,
        ),
        (
            "app.component('test', {
                    computed: {
                      fooBar() {
                        this.$set(this, 'foo', 'lorem');
                        Vue.set(this, 'bar', 'ipsum');
                        return this.foo + ' ' + this.bar
                      },
                    }
                  })",
            None,
            None,
            None,
        ),
        (
            "
                  <script>
                  import {ref, computed} from 'vue'
                  export default {
                    setup() {
                      const foo = useFoo()
                      const asd = { qwe: {} }
                      function a () {}
                      class A {}

                      const test1 = computed(() => {
                        foo.firstName = 'lorem'
                        asd.qwe.zxc = 'lorem'
                        return foo.firstName + ' ' + foo.lastName
                      })
                      const test2 = computed(() => {
                        foo.count += 2;
                        foo.count++;
                        return foo.count;
                      })
                      const test3 = computed(() => foo.something.reverse())
                      const test4 = computed(() => {
                        const test = foo.another.something.push('example')
                        return 'something'
                      })
                      const test5 = computed(() => {
                        foo.something[index] = foo.thing[index]
                        return foo.something
                      })
                      const test6 = computed(() => foo.something.keys.sort())
                      const test7 = computed({
                        get() {
                          return foo.something.reverse()
                        }
                      })
                      const test8 = computed(() => {
                        a.name = ''
                      })
                      const test9 = computed(() => {
                        A.name = ''
                      })
                      const test10 = computed(() => (foo.a = '', true))

                      const test100 = computed(() => {
                        const a = foo
                        a.count++ // false negative
                      })
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
                  import {reactive, computed} from 'vue'
                  export default {
                    setup() {
                      const arr = reactive([])

                      const test1 = computed(() => arr.reverse())
                    }
                  }
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r#"
                  <script lang="ts">
                  import {ref, computed} from 'vue'
                  export default {
                    setup() {
                      const foo = useFoo()

                      const test1 = computed(() => foo.something.reverse())
                    }
                  }
                  </script>
                  "#,
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                  <script setup>
                  import {ref, computed} from 'vue'
                  const foo = useFoo()
                  const asd = { qwe: {} }
                  function a () {}
                  class A {}

                  const test1 = computed(() => {
                    foo.firstName = 'lorem'
                    asd.qwe.zxc = 'lorem'
                    return foo.firstName + ' ' + foo.lastName
                  })
                  const test2 = computed(() => {
                    foo.count += 2;
                    foo.count++;
                    return foo.count;
                  })
                  const test3 = computed(() => foo.something.reverse())
                  const test4 = computed(() => {
                    const test = foo.another.something.push('example')
                    return 'something'
                  })
                  const test5 = computed(() => {
                    foo.something[index] = foo.thing[index]
                    return foo.something
                  })
                  const test6 = computed(() => foo.something.keys.sort())
                  const test7 = computed({
                    get() {
                      return foo.something.reverse()
                    }
                  })
                  const test8 = computed(() => {
                    a.name = ''
                  })
                  const test9 = computed(() => {
                    A.name = ''
                  })
                  const test10 = computed(() => (foo.a = '', true))

                  const test100 = computed(() => {
                    const a = foo
                    a.count++ // false negative
                  })
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                  <script setup>
                  import {reactive, computed} from 'vue'
                  const arr = reactive([])

                  const test1 = computed(() => arr.reverse())
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r#"
                  <script lang="ts" setup>
                  import {ref, computed} from 'vue'
                  const foo = useFoo()

                  const test1 = computed(() => foo.something.reverse())
                  </script>
                  "#,
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                  <script>
                  import {ref, computed} from 'vue'
                  export default {
                    setup() {
                      const foo = useFoo()

                      const test1 = computed(() => Object.assign(foo.data, { extra: 'value' }))
                      const test2 = computed(() => {
                        return Object.assign(foo.user, foo.updates)
                      })
                      const test3 = computed({
                        get() {
                          Object.assign(foo.settings, { theme: 'dark' })
                          return foo.settings
                        }
                      })
                    }
                  }
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
    ];

    Tester::new(
        NoSideEffectsInComputedProperties::NAME,
        NoSideEffectsInComputedProperties::PLUGIN,
        pass,
        fail,
    )
    .test_and_snapshot();
}
