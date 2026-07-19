use oxc_ast::{
    AstKind,
    ast::{
        CallExpression, Expression, ExpressionKind, ExpressionTag, IdentifierReference,
        UnaryOperator,
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
    utils::{
        ComputedContext, find_computed_context, is_this_object, is_vue_component_options_object,
    },
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
    version = "1.70.0",
    short_description = "Disallow side effects in computed properties.",
);

impl Rule for NoSideEffectsInComputedProperties {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::StaticMemberExpression(mem) => {
                // Special case: `this.$set(...)` — report on the property name span and return.
                // If `this.$set` is not called (e.g. assignment `this.$set = fn`), fall through
                // to general mutation detection so it is caught there.
                if mem.property.name == "$set" && is_this_object(&mem.object, ctx) {
                    let mut parent = ctx.nodes().parent_node(node.id());
                    // `(this.$set)(...)` — skip parenthesized wrappers between the member and the call
                    while matches!(parent.kind(), AstKind::ParenthesizedExpression(_)) {
                        parent = ctx.nodes().parent_node(parent.id());
                    }
                    if matches!(parent.kind(), AstKind::CallExpression(_)) {
                        if let Some(ComputedContext::OptionsApi(key)) =
                            find_computed_context(node, ctx)
                        {
                            let key = key.as_deref().unwrap_or("Unknown");
                            ctx.diagnostic(unexpected_side_effect_in_property(
                                mem.property.span,
                                key,
                            ));
                        }
                        return;
                    }
                }

                // Special case: `Vue.set(...)`
                if mem.property.name == "set"
                    && matches!(mem.object.kind(), ExpressionKind::Identifier(ident) if ident.name == "Vue")
                {
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

                check_mutation(node, &mem.object, ctx);
            }
            AstKind::ComputedMemberExpression(mem) => {
                check_mutation(node, &mem.object, ctx);
            }
            _ => {}
        }
    }
}

/// General mutation detection shared by static and computed member expressions.
///
/// Mirrors ESLint's split between the two contexts:
/// - In an Options API computed property only `this` (and its aliases) is checked,
///   and the mutation walk starts AT the member expression — so a first-level
///   method call like `this.reverse()` is a component method, not a mutation.
/// - In a Composition API computed function `this` is ignored entirely, and
///   identifiers (including `this` aliases) are checked as setup variables with
///   the walk starting at the identifier — so `foo.reverse()` IS a mutation.
fn check_mutation<'a>(node: &AstNode<'a>, object: &Expression<'a>, ctx: &LintContext<'a>) {
    if !matches!(object.tag(), ExpressionTag::ThisExpression | ExpressionTag::Identifier) {
        return;
    }

    // The seeded walk finds a superset of the unseeded one — use it as a cheap gate
    // before resolving the computed context.
    let Some(mutation_span) = find_mutation_span(node, ctx, true) else { return };

    match find_computed_context(node, ctx) {
        Some(ComputedContext::OptionsApi(key)) => {
            if !is_this_object(object, ctx) {
                return;
            }
            let Some(mutation_span) = find_mutation_span(node, ctx, false) else { return };
            let key = key.as_deref().unwrap_or("Unknown");
            ctx.diagnostic(unexpected_side_effect_in_property(mutation_span, key));
        }
        Some(ComputedContext::CompositionApi(getter_fn_span)) => {
            let ExpressionKind::Identifier(ident) = object.kind() else { return };
            if !is_setup_variable(ident, getter_fn_span, ctx) {
                return;
            }
            ctx.diagnostic(unexpected_side_effect_in_function(mutation_span));
        }
        None => {}
    }
}

const MUTATING_METHODS: &[&str] =
    &["push", "pop", "shift", "unshift", "reverse", "splice", "sort", "copyWithin", "fill"];

/// Starting from `start_node` (a member expression where object = this/ident),
/// walk up through parent nodes and detect if the member chain is being mutated.
/// Returns the span of the outermost mutation expression, or None if no mutation found.
///
/// `seed_start` controls where the walk conceptually begins, matching ESLint's
/// two findMutating call sites: `true` starts at the identifier (Composition API),
/// so the start node's own property counts as a chain step and a first-level
/// mutating call like `foo.reverse()` is detected; `false` starts at the member
/// expression itself (Options API), so `this.reverse()` is not a mutation.
fn find_mutation_span(
    start_node: &AstNode<'_>,
    ctx: &LintContext<'_>,
    seed_start: bool,
) -> Option<Span> {
    let nodes = ctx.nodes();
    let mut current = start_node;
    let mut last_static_name: Option<String> = if seed_start {
        match start_node.kind() {
            AstKind::StaticMemberExpression(m) => Some(m.property.name.to_string()),
            AstKind::ComputedMemberExpression(m) => m.static_property_name().map(|s| s.to_string()),
            _ => None,
        }
    } else {
        None
    };

    loop {
        let parent = nodes.parent_node(current.id());
        match parent.kind() {
            AstKind::StaticMemberExpression(mem) if mem.object.span() == current.span() => {
                last_static_name = Some(mem.property.name.to_string());
                current = parent;
            }

            // Walking through computed member: this.foo[bar] / this.foo['push']...
            // Resolve string-literal keys (e.g. `this.arr['push']`) so mutating methods are
            // detected; dynamic keys (variables) remain None and correctly produce no match.
            AstKind::ComputedMemberExpression(mem) if mem.object.span() == current.span() => {
                last_static_name = mem.static_property_name().map(|s| s.to_string());
                current = parent;
            }

            // ChainExpression and ParenthesizedExpression are transparent wrappers
            AstKind::ChainExpression(_) | AstKind::ParenthesizedExpression(_) => {
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
                if unary.operator == UnaryOperator::Delete
                    && unary.argument.span() == current.span() =>
            {
                return Some(unary.span());
            }

            // Call expression — check if we are the callee
            AstKind::CallExpression(call) if call.callee.span() == current.span() => {
                if last_static_name.as_deref().is_some_and(|name| MUTATING_METHODS.contains(&name))
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
    let ExpressionKind::StaticMemberExpression(callee) = call.callee.get_inner_expression().kind()
    else {
        return false;
    };
    let ExpressionKind::Identifier(obj) = callee.object.get_inner_expression().kind() else {
        return false;
    };
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
    ident: &IdentifierReference<'_>,
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

    // <script setup>: all top-level bindings are setup variables, except import declarations
    // (ESLint excludes def.type === 'ImportBinding' from setup ranges)
    if ctx.frameworks_options() == FrameworkOptions::VueSetup {
        let is_import = ctx
            .nodes()
            .ancestors(decl_node.id())
            .any(|a| matches!(a.kind(), AstKind::ImportDeclaration(_)));
        return !is_import;
    }

    // Options API `setup()`: check if decl is inside a setup() function body
    // Walk ancestors of the declaration node to find if it's inside `setup()`
    is_inside_setup_function(decl_node, ctx)
}

/// Check if `node` is declared within a `setup()` function body of a Vue component options object.
/// Parameters of setup() are excluded: ESLint's setupRanges holds only node.body.range (the block
/// statement), so formal parameters — whose span is before the opening `{` — fail the range check.
fn is_inside_setup_function(node: &AstNode<'_>, ctx: &LintContext<'_>) -> bool {
    ctx.nodes().ancestors(node.id()).any(|ancestor| {
        let body_span = match ancestor.kind() {
            AstKind::Function(f) => {
                let Some(body) = &f.body else { return false };
                body.span
            }
            AstKind::ArrowFunctionExpression(f) => f.body.span,
            _ => return false,
        };
        // Declaration must be inside the function body (not in the parameter list)
        if !body_span.contains_inclusive(node.span()) {
            return false;
        }
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
        // <script setup>: import bindings are not setup variables
        (
            "
                  <script setup>
                  import { state } from './store'
                  import store from './store'
                  import { computed } from 'vue'

                  const c1 = computed(() => state.items.reverse())
                  const c2 = computed(() => {
                    store.count++
                  })
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // arrow function getter in the {get, set} object form is not analyzed
        (
            "Vue.component('test', {
                    computed: {
                      foo: {
                        get: () => {
                          this.x = 1
                        }
                      }
                    }
                  })",
            None,
            None,
            None,
        ),
        // component methods named like mutating array methods are not mutations
        (
            "Vue.component('test', {
                    computed: {
                      test1() {
                        return this.reverse()
                      },
                      test2() {
                        return this.push('x')
                      },
                      test3() {
                        return this.sort()
                      }
                    }
                  })",
            None,
            None,
            None,
        ),
        // setup() function parameters are not setup variables
        (
            "
                  <script>
                  import { computed } from 'vue'
                  export default {
                    setup(props, context) {
                      const test1 = computed(() => props.something.reverse())
                      const test2 = computed(() => {
                        props.count++
                      })
                    }
                  }
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // arrow function values in Options API computed are not analyzed
        (
            "Vue.component('test', {
                    computed: {
                      foo: () => {
                        this.x = 1
                      }
                    }
                  })",
            None,
            None,
            None,
        ),
        // a `computed:` key nested inside data() is not a Vue computed option
        (
            "Vue.component('test', {
                    data() {
                      return {
                        computed: {
                          foo() {
                            this.x = 1
                          }
                        }
                      }
                    }
                  })",
            None,
            None,
            None,
        ),
        // ESLint ignores `this` inside Composition API computed functions
        (
            "
                  <script>
                  import { computed } from 'vue'
                  export default {
                    setup() {
                      const test1 = computed(() => {
                        this.x = 1
                      })
                      const test2 = computed(() => this['y'].push(1))
                    }
                  }
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // a `this`-alias local to the computed function is a local variable, not a setup variable
        (
            "
                  <script>
                  import { computed } from 'vue'
                  export default {
                    setup() {
                      const test1 = computed(() => {
                        const self = this
                        self.x = 1
                      })
                    }
                  }
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // a `this`-alias declared at module scope is not a setup variable
        (
            "
                  <script>
                  import { computed } from 'vue'
                  const self = this
                  export default {
                    setup() {
                      const test1 = computed(() => {
                        self.x = 1
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
        // a computed property literally named "set" is still a getter
        (
            "Vue.component('test', {
                    computed: {
                      set() {
                        return this.something.reverse()
                      }
                    }
                  })",
            None,
            None,
            None,
        ),
        // assignment to `this.$set` is a plain property mutation
        (
            "Vue.component('test', {
                    computed: {
                      test1() {
                        this.$set = function() {}
                        return this.foo
                      }
                    }
                  })",
            None,
            None,
            None,
        ),
        // bracket notation with a string-literal key for a mutating method
        (
            "Vue.component('test', {
                    computed: {
                      test1() {
                        return this.arr['push']('x')
                      },
                      test2() {
                        return this.arr['reverse']()
                      }
                    }
                  })",
            None,
            None,
            None,
        ),
        // aliased import of `computed` from 'vue'
        (
            "
                  <script>
                  import { computed as c } from 'vue'
                  export default {
                    setup() {
                      const foo = useFoo()
                      const test1 = c(() => foo.something.reverse())
                      const test2 = c(() => {
                        foo.firstName = 'lorem'
                      })
                    }
                  }
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // bracket-notation mutations on `this`
        (
            "Vue.component('test', {
                    computed: {
                      test1() {
                        this['foo'] = 1
                      },
                      test2() {
                        return this['arr'].push(x)
                      },
                      test3() {
                        this['count']++
                      },
                      test4() {
                        delete this['foo']
                      }
                    }
                  })",
            None,
            None,
            None,
        ),
        // bracket-notation mutations on setup variables
        (
            "
                  <script>
                  import { computed } from 'vue'
                  export default {
                    setup() {
                      const foo = useFoo()
                      const test1 = computed(() => {
                        foo['firstName'] = 'lorem'
                      })
                      const test2 = computed(() => foo['arr'].reverse())
                    }
                  }
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // `computed` imported from '@vue/composition-api'
        (
            "
                  <script>
                  import { computed } from '@vue/composition-api'
                  export default {
                    setup() {
                      const foo = useFoo()
                      const test1 = computed(() => foo.something.reverse())
                      const test2 = computed(() => {
                        foo.firstName = 'lorem'
                      })
                    }
                  }
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // a `this`-alias declared in setup() is checked as a setup variable
        (
            "
                  <script>
                  import { computed } from 'vue'
                  export default {
                    setup() {
                      const self = this
                      const test1 = computed(() => self.items.reverse())
                      const test2 = computed(() => {
                        self.firstName = 'lorem'
                      })
                    }
                  }
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // `(this.$set)(...)` — parenthesized callee
        (
            "Vue.component('test', {
                    computed: {
                      test1() {
                        (this.$set)(this, 'foo', 'lorem')
                        return this.foo
                      }
                    }
                  })",
            None,
            None,
            None,
        ),
        // `computed` imported from '#imports' (Nuxt auto-import)
        (
            "
                  <script>
                  import { computed } from '#imports'
                  export default {
                    setup() {
                      const foo = useFoo()
                      const test1 = computed(() => foo.something.reverse())
                      const test2 = computed(() => {
                        foo.firstName = 'lorem'
                      })
                    }
                  }
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // a `this`-alias declared in setup() is an ordinary setup variable:
        // the walk starts at the identifier, so a first-level mutating call counts
        (
            "
                  <script>
                  import { computed } from 'vue'
                  export default {
                    setup() {
                      const self = this
                      const test1 = computed(() => self.reverse())
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
