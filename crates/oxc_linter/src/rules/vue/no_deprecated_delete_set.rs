use oxc_ast::{
    AstKind,
    ast::{BindingPattern, Expression, MemberExpression, StaticMemberExpression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::SymbolId;
use oxc_span::Span;
use rustc_hash::FxHashSet;

use crate::module_record::ImportImportName;
use crate::{AstNode, context::LintContext, rule::Rule};

fn no_deprecated_delete_set_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("The `$delete`, `$set` is deprecated.").with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoDeprecatedDeleteSet;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow using deprecated `$set` / `$delete` (in Vue.js 3.0.0+).
    ///
    /// ### Why is this bad?
    ///
    /// In Vue 3, the instance methods `$set` / `$delete` and the global
    /// `Vue.set` / `Vue.delete` were removed. Reactivity is now backed by
    /// Proxies, so plain assignment and the `delete` operator work as
    /// expected and these helpers are no longer needed.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```vue
    /// <script>
    /// export default {
    ///   mounted() {
    ///     this.$set(obj, key, value)
    ///     this.$delete(obj, key)
    ///     Vue.set(obj, key, value)
    ///     Vue.delete(obj, key)
    ///   }
    /// }
    /// </script>
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```vue
    /// <script>
    /// export default {
    ///   mounted() {
    ///     obj[key] = value
    ///     delete obj[key]
    ///   }
    /// }
    /// </script>
    /// ```
    NoDeprecatedDeleteSet,
    vue,
    correctness,
    version = "next",
);

impl Rule for NoDeprecatedDeleteSet {
    fn should_run(&self, ctx: &crate::context::ContextHost) -> bool {
        ctx.file_extension().is_some_and(|ext| ext == "vue")
    }

    fn run_once(&self, ctx: &LintContext) {
        let module_record = ctx.module_record();
        let scoping = ctx.scoping();

        // Imports of `set` / `del` from `'vue'` (also handles `set as X` / `del as X`).
        let mut imported: FxHashSet<SymbolId> = FxHashSet::default();
        for import_entry in &module_record.import_entries {
            if import_entry.module_request.name() != "vue" {
                continue;
            }
            let ImportImportName::Name(name_span) = &import_entry.import_name else {
                continue;
            };
            if !matches!(name_span.name(), "set" | "del") {
                continue;
            }
            if let Some(symbol_id) = scoping.get_root_binding(import_entry.local_name.name().into())
            {
                imported.insert(symbol_id);
            }
        }

        for node in ctx.nodes().iter() {
            let AstKind::CallExpression(call) = node.kind() else { continue };

            // Phase 1+2: `Vue.set` / `Vue.delete` and `this.$set` / `this.$delete`
            if let Some(member) = static_member_callee(&call.callee) {
                let prop_name = member.property.name.as_str();
                let object = member.object.get_inner_expression();

                if matches!(prop_name, "set" | "delete")
                    && let Expression::Identifier(ident) = object
                    && ident.name == "Vue"
                {
                    ctx.diagnostic(no_deprecated_delete_set_diagnostic(member.property.span));
                    continue;
                }

                if matches!(prop_name, "$set" | "$delete")
                    && is_this_or_alias(object, ctx)
                    && is_in_vue_component(node, ctx)
                {
                    ctx.diagnostic(no_deprecated_delete_set_diagnostic(member.property.span));
                    continue;
                }
            }

            // Phase 3: `import { set, del } from 'vue'; set()` / `del()`
            if !imported.is_empty()
                && let Expression::Identifier(ident) = call.callee.get_inner_expression()
                && let Some(symbol_id) = scoping.get_reference(ident.reference_id()).symbol_id()
                && imported.contains(&symbol_id)
            {
                ctx.diagnostic(no_deprecated_delete_set_diagnostic(ident.span));
            }
        }
    }
}

/// Returns the callee as a `StaticMemberExpression`, peeling parens and an
/// outer `ChainExpression` if present.
fn static_member_callee<'a, 'b>(
    callee: &'b Expression<'a>,
) -> Option<&'b StaticMemberExpression<'a>> {
    let inner = callee.get_inner_expression();
    let member = match inner {
        Expression::ChainExpression(chain) => chain.expression.as_member_expression()?,
        _ => inner.as_member_expression()?,
    };
    match member {
        MemberExpression::StaticMemberExpression(m) => Some(m),
        _ => None,
    }
}

fn is_this_or_alias<'a>(expr: &Expression<'a>, ctx: &LintContext<'a>) -> bool {
    match expr {
        Expression::ThisExpression(_) => true,
        Expression::Identifier(ident) => {
            let scoping = ctx.scoping();
            let reference = scoping.get_reference(ident.reference_id());
            let Some(symbol_id) = reference.symbol_id() else { return false };
            let declaration = ctx.symbol_declaration(symbol_id);
            let AstKind::VariableDeclarator(decl) = declaration.kind() else { return false };
            let BindingPattern::BindingIdentifier(_) = &decl.id else { return false };
            let Some(init) = &decl.init else { return false };
            matches!(init.get_inner_expression(), Expression::ThisExpression(_))
        }
        _ => false,
    }
}

fn is_in_vue_component<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
    ctx.nodes().ancestors(node.id()).any(|a| match a.kind() {
        AstKind::ExportDefaultDeclaration(_) => true,
        AstKind::CallExpression(call) => call
            .callee
            .get_identifier_reference()
            .is_some_and(|ident| ident.name == "defineComponent"),
        _ => false,
    })
}

#[test]
fn test() {
    use crate::tester::Tester;
    use std::path::PathBuf;

    let pass = vec![
        // Different API (`$nextTick`)
        (
            "
                <script>
                export default {
                  mounted () {
                    this.$nextTick()
                  }
                }
                </script>
            ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // Reference (not call): `a(this.$set)` is allowed
        (
            "
                <script>
                export default {
                  mounted () {
                    a(this.$set)
                    a(this.$delete)
                  }
                }
                </script>
            ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // Locally-defined `set` / `del` (not from 'vue')
        (
            "
                <script>
                function set(obj, key, value) {}
                function del(obj, key) {}
                export default {
                  mounted () {
                    set(obj, key, value)
                    del(obj, key)
                  }
                }
                </script>
            ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // Imported `nextTick`/`provide`, not `set`/`del`
        (
            "
                <script>
                import { nextTick as nt, provide } from 'vue'
                export default {
                  mounted () {
                    nt()
                    provide(key, value)
                  }
                }
                </script>
            ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // CJS `require('vue')` aliasing other members to `set`/`del` names
        (
            "
                <script>
                const { nextTick: set, provide: del } = require('vue')
                export default {
                  mounted () {
                    set()
                    del(key, value)
                  }
                }
                </script>
            ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
    ];

    let fail = vec![
        // Phase 2: `this.$set` / `this.$delete` in `export default`
        (
            "
                <script>
                export default {
                  mounted () {
                    this.$set(obj, key, value)
                    this.$delete(obj, key)
                  }
                }
                </script>
            ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // Phase 2: `defineComponent` + aliased this
        (
            "
                <script>
                defineComponent({
                  mounted () {
                    const vm = this
                    vm.$set(obj, key, value)
                    vm.$delete(obj, key)
                  }
                })
                </script>
            ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // Phase 2: optional chain `this?.$set`
        (
            "
                <script>
                defineComponent({
                  mounted () {
                    this?.$set(obj, key, value)
                    this?.$delete(obj, key)
                  }
                })
                </script>
            ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // Phase 1: `Vue.set` / `Vue.delete`
        (
            "
                <script>
                defineComponent({
                  mounted () {
                    Vue.set(obj, key, value)
                    Vue.delete(obj, key)
                  }
                })
                </script>
            ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // Phase 3 (NOT YET IMPLEMENTED): `import { set, del } from 'vue'`
        (
            "
                <script>
                import { set, del } from 'vue'
                export default {
                  mounted () {
                    set(obj, key, value)
                    del(obj, key)
                  }
                }
                </script>
            ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // Phase 3 (NOT YET IMPLEMENTED): `<script setup>` + import
        (
            "
                <script setup>
                import { set, del } from 'vue'

                set(obj, key, value)
                del(obj, key)
                </script>
            ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // Phase 3 (NOT YET IMPLEMENTED): aliased import `import { set as s }`
        (
            "
                <script setup>
                import { set as s, del as d } from 'vue'

                s(obj, key, value)
                d(obj, key)
                </script>
            ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
    ];

    Tester::new(NoDeprecatedDeleteSet::NAME, NoDeprecatedDeleteSet::PLUGIN, pass, fail)
        .test_and_snapshot();
}
