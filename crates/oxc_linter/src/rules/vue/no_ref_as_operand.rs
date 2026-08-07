use oxc_ast::{
    AstKind,
    ast::{AssignmentOperator, BindingPattern, CallExpression, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::SymbolId;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    ast_util::get_symbol_id_of_variable,
    context::{ContextHost, LintContext},
    module_record::ImportImportName,
    rule::Rule,
};

fn require_dot_value(span: Span, method: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Must use `.value` to read or write the value wrapped by `{method}()`."
    ))
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoRefAsOperand;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow use of a value wrapped by `ref()` (Composition API) as an operand.
    ///
    /// ### Why is this bad?
    ///
    /// A ref object is a box around the actual value. Using the box directly in an
    /// expression (arithmetic, comparison, `if`, etc.) operates on the wrapper rather
    /// than the inner value, which is almost always a mistake. Read or write via
    /// `.value`.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```vue
    /// <script>
    /// import { ref } from 'vue'
    /// const count = ref(0)
    /// count++          // operates on the ref object
    /// if (count) {}    // truthy check on the ref object
    /// </script>
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```vue
    /// <script>
    /// import { ref } from 'vue'
    /// const count = ref(0)
    /// count.value++
    /// if (count.value) {}
    /// </script>
    /// ```
    NoRefAsOperand,
    vue,
    correctness,
    fix,
    version = "next",
    short_description = "Disallow use of a value wrapped by `ref()` (Composition API) as an operand.",
);

// Mirrors upstream `iterateDefineRefs` in `lib/utils/ref-object-references.js`.
// `triggerRef` is a side-effectful function whose return value is NOT a ref, so it is excluded.
const REF_FACTORIES: &[&str] = &["ref", "shallowRef", "customRef", "toRef", "toRefs", "computed"];

impl Rule for NoRefAsOperand {
    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.file_extension().is_some_and(|ext| ext == "vue")
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::IdentifierReference(ident) = node.kind() else { return };
        let Some(symbol_id) = get_symbol_id_of_variable(ident, ctx) else { return };
        let Some(method) = ref_factory_method_for_ident(symbol_id, ctx, ident.span) else { return };

        let parent = ctx.nodes().parent_node(node.id());
        if !is_operand_position(parent, node, ctx) && !is_emit_call_argument(parent, node, ctx) {
            return;
        }

        ctx.diagnostic_with_fix(require_dot_value(ident.span, method), |fixer| {
            fixer.insert_text_after(&ident.span, ".value")
        });
    }
}

/// Returns the factory method name (e.g. "ref") if the declaration of the given identifier
/// reference resolves to a `ref()`-like value.
///
/// Handles:
/// - `const x = ref(0)` (direct factory call init)
/// - `const [model, mod] = defineModel()` (first element of the macro tuple is the ref)
/// - `const y = x` where `x` is itself a ref (defineChain propagation, recursive)
fn ref_factory_method_for_ident(
    symbol_id: SymbolId,
    ctx: &LintContext<'_>,
    query_span: Span,
) -> Option<&'static str> {
    // The top-level entry mirrors upstream `isRefInit`: only the variable's own `init`
    // (or its chain through other ref identifiers) decides whether the reference is
    // reportable. Later assignments (e.g. `let foo = undefined; foo = ref(5); foo++`)
    // do NOT make the reference reportable on their own.
    ref_factory_method_for_ident_with_seen(symbol_id, ctx, query_span, false, &mut Vec::new())
}

fn ref_factory_method_for_ident_with_seen(
    symbol_id: SymbolId,
    ctx: &LintContext<'_>,
    query_span: Span,
    via_chain: bool,
    seen: &mut Vec<SymbolId>,
) -> Option<&'static str> {
    if seen.contains(&symbol_id) {
        return None;
    }
    seen.push(symbol_id);

    let scoping = ctx.scoping();
    let decl_node = ctx.nodes().get_node(scoping.symbol_declaration(symbol_id));
    let AstKind::VariableDeclarator(decl) = decl_node.kind() else { return None };

    let id_pattern_kind = match &decl.id {
        BindingPattern::BindingIdentifier(_) => IdPattern::Ident,
        BindingPattern::ArrayPattern(arr) => {
            let first_is_target =
                arr.elements.first().and_then(|el| el.as_ref()).is_some_and(|e| {
                    e.get_binding_identifier().is_some_and(|id| id.symbol_id() == symbol_id)
                });
            if first_is_target { IdPattern::ArrayFirst } else { return None }
        }
        BindingPattern::ObjectPattern(obj) => {
            // `const { foo } = toRefs(state)` — each destructured key binds a ref. We must
            // confirm `symbol_id` is one of the bound names (any property is fine).
            let is_member = obj.properties.iter().any(|prop| {
                prop.value.get_binding_identifier().is_some_and(|id| id.symbol_id() == symbol_id)
            });
            if is_member { IdPattern::ObjectProperty } else { return None }
        }
        BindingPattern::AssignmentPattern(_) => return None,
    };

    // Direct factory call init.
    if let Some(init) = decl.init.as_ref() {
        let inner = init.get_inner_expression();
        if let Some(method) = factory_method_from_expr(inner, ctx) {
            if matches!(id_pattern_kind, IdPattern::ObjectProperty) {
                // ObjectPattern only makes sense for `toRefs(...)`; the destructured value of
                // any other factory call (e.g. `const { value } = ref(0)`) is a raw value, not a ref.
                return if method == "toRefs" { Some("toRefs") } else { None };
            }
            return Some(method);
        }

        // defineChain via init: `const y = x` where x is itself a ref. Only valid for plain
        // identifier patterns. Skip the `undefined` literal — `let foo = undefined` is NOT
        // a chain step (upstream `isRefInit` excludes it).
        if matches!(id_pattern_kind, IdPattern::Ident)
            && let Expression::Identifier(ref_ident) = inner
            && ref_ident.name != "undefined"
            && let Some(src_symbol) = get_symbol_id_of_variable(ref_ident, ctx)
            && let Some(method) =
                ref_factory_method_for_ident_with_seen(src_symbol, ctx, query_span, true, seen)
        {
            return Some(method);
        }
    }

    if !matches!(id_pattern_kind, IdPattern::Ident) {
        return None;
    }

    // Later-assignment chain step. Upstream's `isRefInit` requires the variable's own
    // `init` to be in `defineChain` — so a variable like `let foo = undefined` is NEVER
    // reportable from its own usages, but it CAN propagate the ref to a downstream chain
    // step (e.g. `let bar = foo` where `bar = 4` should be reported).
    if !via_chain {
        return None;
    }

    for reference in scoping.get_resolved_references(symbol_id) {
        let ref_node = ctx.nodes().get_node(reference.node_id());
        let ref_span = ref_node.span();
        if ref_span.end > query_span.start {
            continue;
        }
        let parent = ctx.nodes().parent_node(ref_node.id());
        let AstKind::AssignmentExpression(a) = parent.kind() else { continue };
        if !matches!(a.operator, AssignmentOperator::Assign) {
            continue;
        }
        if a.left.span() != ref_span {
            continue;
        }
        let right_inner = a.right.get_inner_expression();
        if let Some(method) = factory_method_from_expr(right_inner, ctx) {
            return Some(method);
        }
        if let Expression::Identifier(src_ident) = right_inner
            && let Some(src_symbol) = get_symbol_id_of_variable(src_ident, ctx)
            && let Some(method) =
                ref_factory_method_for_ident_with_seen(src_symbol, ctx, ref_span, true, seen)
        {
            return Some(method);
        }
    }
    None
}

enum IdPattern {
    /// `const x = ref(0)` — plain identifier on the left.
    Ident,
    /// `const [model, mod] = defineModel()` — the symbol is the first element of an array pattern.
    ArrayFirst,
    /// `const { foo, bar } = toRefs(state)` — the symbol is one of the destructured keys.
    ObjectProperty,
}

fn factory_method_from_expr<'a>(
    expr: &Expression<'a>,
    ctx: &LintContext<'a>,
) -> Option<&'static str> {
    let Expression::CallExpression(call) = expr else { return None };
    if let Some(method) = is_vue_ref_factory_call(call, ctx) {
        return Some(method);
    }
    // `defineModel()` is a `<script setup>` compiler macro. Upstream `iterateDefineModels`
    // only treats the auto-import / globalScope.through identifier as the macro: a local
    // `function defineModel() {}` shadows it and is NOT a factory.
    if let Expression::Identifier(ident) = call.callee.get_inner_expression()
        && ident.name == "defineModel"
        && get_symbol_id_of_variable(ident, ctx).is_none()
    {
        return Some("defineModel");
    }
    None
}

fn is_vue_ref_factory_call(
    call: &CallExpression<'_>,
    ctx: &LintContext<'_>,
) -> Option<&'static str> {
    let Expression::Identifier(ident) = call.callee.get_inner_expression() else { return None };
    let factory_name = REF_FACTORIES.iter().copied().find(|&f| f == ident.name)?;
    let scoping = ctx.scoping();
    let resolved = get_symbol_id_of_variable(ident, ctx);
    match resolved {
        // Unresolved (global) — upstream `globalScope.through` matches by name only.
        None => Some(factory_name),
        // Resolved — accept only if the binding originates from a Vue-ish import.
        Some(symbol_id) => ctx.module_record().import_entries.iter().find_map(|entry| {
            if !matches!(entry.module_request.name(), "vue" | "@vue/composition-api" | "#imports") {
                return None;
            }
            let ImportImportName::Name(name_span) = &entry.import_name else { return None };
            (name_span.name() == factory_name
                && scoping.get_root_binding(entry.local_name.name().into()) == Some(symbol_id))
            .then_some(factory_name)
        }),
    }
}

fn is_operand_position(parent: &AstNode<'_>, node: &AstNode<'_>, ctx: &LintContext<'_>) -> bool {
    let ident_span = node.span();
    match parent.kind() {
        AstKind::IfStatement(s) => s.test.span() == ident_span,
        AstKind::SwitchStatement(s) => s.discriminant.span() == ident_span,
        AstKind::UnaryExpression(_)
        | AstKind::UpdateExpression(_)
        | AstKind::BinaryExpression(_) => true,
        AstKind::AssignmentExpression(a) => {
            // For plain `=`, only the left side is an operand: writing through the wrapper
            // (`model = 4` → `model.value = 4`). The right side hands the ref object on
            // (`foo = count`), which is what the user intended.
            // For compound assignments (`+=`, `-=`, ...), both sides ARE operands.
            if matches!(a.operator, AssignmentOperator::Assign) {
                a.left.span() == ident_span
            } else {
                true
            }
        }
        AstKind::LogicalExpression(l) => {
            // Report only when on the left (`refValue || other`) — and only for `const` refs
            // because `let`/`var` could be reassigned. Mutability is judged from the declaring
            // `VariableDeclaration` kind via the symbol.
            if l.left.span() != ident_span {
                return false;
            }
            is_declared_const(node, ctx)
        }
        AstKind::ConditionalExpression(c) => c.test.span() == ident_span,
        AstKind::TemplateLiteral(_) => {
            // Skip tagged template literals: the tag's first arg is the raw quasi array,
            // not the interpolated value.
            !matches!(
                ctx.nodes().parent_node(parent.id()).kind(),
                AstKind::TaggedTemplateExpression(_)
            )
        }
        AstKind::StaticMemberExpression(m) => {
            // `refValue.x` is a mistake, but `refValue.value` / `refValue.effect`
            // (WritableComputedRef) are how the ref is meant to be accessed.
            m.object.span() == ident_span && !matches!(m.property.name.as_str(), "value" | "effect")
        }
        AstKind::ComputedMemberExpression(m) => {
            // Upstream resolves the key via `getStaticPropertyName`. Only string-literal
            // keys are recognised; dynamic keys (e.g. `foo[bar] = 123`) can't be resolved
            // statically and are NOT reported.
            if m.object.span() != ident_span {
                return false;
            }
            let Expression::StringLiteral(lit) = &m.expression else {
                return false;
            };
            !matches!(lit.value.as_str(), "value" | "effect")
        }
        _ => false,
    }
}

/// Whether `node` is an `Identifier` argument passed to an emit-like call:
/// `defineEmits()`'s returned binding, `setup(_, { emit })`'s `emit`, or
/// `setup(_, ctx)`'s `ctx.emit(...)`. The first arg slot is the event name —
/// only subsequent slots count as payload.
fn is_emit_call_argument(parent: &AstNode<'_>, node: &AstNode<'_>, ctx: &LintContext<'_>) -> bool {
    // The identifier may be wrapped in one or more `ParenthesizedExpression`s before
    // reaching the call-argument slot. Walk past them to find the enclosing CallExpression.
    let mut current = parent;
    while matches!(current.kind(), AstKind::ParenthesizedExpression(_)) {
        current = ctx.nodes().parent_node(current.id());
    }
    let AstKind::CallExpression(call) = current.kind() else { return false };
    let ident_span = node.span();
    let idx = call.arguments.iter().position(|arg| {
        let expr =
            arg.as_expression().map_or_else(|| arg.span(), |e| e.get_inner_expression().span());
        expr == ident_span
    });
    let Some(idx) = idx else { return false };
    // The 0th argument is the event name; payload starts at index 1.
    if idx == 0 {
        return false;
    }
    is_emit_callee(&call.callee, ctx)
}

/// Returns `true` when the callee is one of:
/// - `emit` identifier where `emit` resolves to `setup(_, { emit })` binding
///   or `const emit = defineEmits(...)`
/// - `ctx.emit` member expression where `ctx` is `setup`'s second positional arg
fn is_emit_callee(callee: &Expression<'_>, ctx: &LintContext<'_>) -> bool {
    let inner = callee.get_inner_expression();
    if let Expression::Identifier(ident) = inner {
        let Some(symbol_id) = get_symbol_id_of_variable(ident, ctx) else { return false };
        return is_emit_binding(symbol_id, ctx);
    }
    if let Expression::StaticMemberExpression(m) = inner
        && m.property.name == "emit"
        && let Expression::Identifier(obj) = m.object.get_inner_expression()
    {
        let Some(symbol_id) = get_symbol_id_of_variable(obj, ctx) else { return false };
        return is_setup_context_binding(symbol_id, ctx);
    }
    false
}

/// True if `symbol_id` is bound to `defineEmits(...)` or destructured as
/// `setup(_, { emit })`'s `emit` property.
fn is_emit_binding(symbol_id: SymbolId, ctx: &LintContext<'_>) -> bool {
    let scoping = ctx.scoping();
    let decl_node = ctx.nodes().get_node(scoping.symbol_declaration(symbol_id));

    // Case A: `const emits = defineEmits(...)` — declaration is `VariableDeclarator`.
    if let AstKind::VariableDeclarator(decl) = decl_node.kind() {
        return matches!(&decl.id, BindingPattern::BindingIdentifier(_))
            && decl.init.as_ref().is_some_and(|init| {
                let Expression::CallExpression(call) = init.get_inner_expression() else {
                    return false;
                };
                let Expression::Identifier(callee_ident) = call.callee.get_inner_expression()
                else {
                    return false;
                };
                callee_ident.name == "defineEmits"
            });
    }

    // Case B: `setup(_, { emit })` — destructured parameter binding inside the setup
    // function's ObjectPattern. The destructured property MUST literally be named `emit`
    // (upstream uses `findAssignmentProperty(contextParam, 'emit')`), so `setup(_, { foo })`
    // does NOT bind `emit`.
    if scoping.symbol_name(symbol_id) != "emit" {
        return false;
    }
    is_setup_destructured_emit(decl_node, ctx)
}

/// True if `start` is inside a `setup(props, { emit })` function — the second parameter
/// is an `ObjectPattern` and the caller has already confirmed the destructured property
/// name is `emit`.
fn is_setup_destructured_emit<'a>(start: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
    let Some(func) = walk_to_setup_function(start, ctx) else { return false };
    func.params.items.get(1).is_some_and(|p| matches!(&p.pattern, BindingPattern::ObjectPattern(_)))
}

/// True if `symbol_id` is the second positional parameter (`ctx`) of a Vue
/// `setup(props, ctx)` function — making `ctx.emit(...)` an emit call.
fn is_setup_context_binding(symbol_id: SymbolId, ctx: &LintContext<'_>) -> bool {
    let scoping = ctx.scoping();
    let decl_node = ctx.nodes().get_node(scoping.symbol_declaration(symbol_id));
    let Some(func) = walk_to_setup_function(decl_node, ctx) else { return false };
    func.params.items.get(1).is_some_and(|p| {
        matches!(
            &p.pattern,
            BindingPattern::BindingIdentifier(id)
                if id.symbol_id() == symbol_id
        )
    })
}

/// Walk up at most 8 nodes from `start` looking for the enclosing `Function` node
/// that is the `setup` property of a Vue component options object (`{ setup() {...} }`
/// shorthand or `{ setup: function() {...} }`).
fn walk_to_setup_function<'a>(
    start: &AstNode<'a>,
    ctx: &LintContext<'a>,
) -> Option<&'a oxc_ast::ast::Function<'a>> {
    let nodes = ctx.nodes();
    let mut current = start;
    for _ in 0..8 {
        if let AstKind::Function(func) = current.kind() {
            let parent = nodes.parent_node(current.id());
            if let AstKind::ObjectProperty(prop) = parent.kind()
                && prop.key.is_specific_static_name("setup")
            {
                return Some(func);
            }
            return None;
        }
        let parent_id = current.id();
        let next = nodes.parent_node(parent_id);
        if next.id() == parent_id {
            return None;
        }
        current = next;
    }
    None
}

fn is_declared_const(node: &AstNode<'_>, ctx: &LintContext<'_>) -> bool {
    let AstKind::IdentifierReference(ident) = node.kind() else { return false };
    let Some(symbol_id) = get_symbol_id_of_variable(ident, ctx) else { return false };
    let decl_node = ctx.nodes().get_node(ctx.scoping().symbol_declaration(symbol_id));
    let AstKind::VariableDeclarator(_) = decl_node.kind() else { return false };
    let parent = ctx.nodes().parent_node(decl_node.id());
    matches!(
        parent.kind(),
        AstKind::VariableDeclaration(v) if matches!(v.kind, oxc_ast::ast::VariableDeclarationKind::Const)
    )
}

#[test]
fn test() {
    use std::path::PathBuf;

    use crate::tester::Tester;

    let pass = vec![
        (
            "
<script>                import { ref } from 'vue'
                const count = ref(0)
                console.log(count.value) // 0

                count.value++
                console.log(count.value) // 1
                </script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                <script>
                  import { ref } from 'vue'
                  export default {
                    setup() {
                      const count = ref(0)
                      console.log(count.value) // 0

                      count.value++
                      console.log(count.value) // 1
                      return {
                        count
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
                  import { ref } from '@vue/composition-api'
                  export default {
                    setup() {
                      const count = ref(0)
                      console.log(count.value) // 0

                      count.value++
                      console.log(count.value) // 1
                      return {
                        count
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
<script>                import { ref } from 'vue'
                const count = ref(0)
                if (count.value) {}
                switch (count.value) {}
                var foo = -count.value
                var foo = +count.value
                count.value++
                count.value--
                count.value + 1
                1 - count.value
                count.value || other
                count.value && other
                var foo = count.value ? x : y
                </script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
<script>                import { ref } from 'vue'
                const foo = ref(true)
                if (bar) foo
                </script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
<script>                import { ref } from 'vue'
                const foo = ref(true)
                var a = other || foo // ignore
                var b = other && foo // ignore

                let bar = ref(true)
                var a = bar || other
                var b = bar || other
                </script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
<script>                import { ref } from 'vue'
                let count = not_ref(0)

                count++
                </script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
<script>                import { ref } from 'vue'
                const foo = ref(0)
                const bar = ref(0)
                var baz = x ? foo : bar
                </script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
<script>                import { ref } from 'vue'
                // Probably wrong, but not checked by this rule.
                const {value} = ref(0)
                value++
                </script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
<script>                import { ref } from 'vue'
                const count = ref(0)
                function foo() {
                  let count = 0
                  count++
                }
                </script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
<script>                import { ref } from 'unknown'
                const count = ref(0)
                count++
                </script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
<script>                import { ref } from 'vue'
                const count = ref
                count++
                </script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
<script>                import { ref } from 'vue'
                const count = ref(0)
                foo = count
                </script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
<script>                import { ref } from 'vue'
                const count = ref(0)
                const foo = count
                </script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                  <script>
                    import { ref, computed, toRef, customRef, shallowRef } from 'vue'
                    const foo = shallowRef({})
                    foo[bar] = 123
                  </script>

",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                  <script>
                    import { ref, computed, toRef, customRef, shallowRef } from 'vue'
                    const foo = shallowRef({})
                    const isComp = foo.effect
                  </script>

",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                  <script>
                  import { ref } from 'vue'
                  let foo;

                  if (!foo) {
                    foo = ref(5);
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
                  import { ref } from 'vue'
                  let foo = undefined;

                  if (!foo) {
                    foo = ref(5);
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
                  import { ref } from 'vue'
                  const foo = ref(0)
                  func(foo)
                  function func(foo) {}
                  </script>

",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                  <script>
                  import { ref } from 'vue'
                  const foo = ref(0)
                  tag`${foo}`
                  function tag(arr, ...args) {}
                  </script>

",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                <script setup>
                const model = defineModel();
                console.log(model.value);
                function process() {
                  if (model.value) console.log('foo')
                }
                function update(value) {
                  model.value = value;
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
                const [model, mod] = defineModel();
                console.log(model.value);
                function process() {
                  if (model.value) console.log('foo')
                }
                function update(value) {
                  model.value = value;
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
                const emit = defineEmits(['test'])
                const [model, mod] = defineModel();

                function update() {
                  emit('test', model.value)
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
                import { ref, defineComponent } from 'vue'

                export default defineComponent({
                  emits: ['incremented'],
                  setup(_, ctx) {
                    const counter = ref(0)

                    ctx.emit('incremented', counter.value)

                    return {
                      counter
                    }
                  }
                })
                </script>

",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                <script>
                import { ref, defineComponent } from 'vue'

                export default defineComponent({
                  emits: ['incremented'],
                  setup(_, { emit }) {
                    const counter = ref(0)

                    emit('incremented', counter.value)

                    return {
                      counter
                    }
                  }
                })
                </script>

",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                <script>
                import { ref, defineComponent } from 'vue'

                export default defineComponent({
                  emits: ['incremented'],
                  setup(_, { emit }) {
                    const counter = ref(0)

                    emit('incremented', counter.value, 'xxx')

                    return {
                      counter
                    }
                  }
                })
                </script>

",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                <script>
                import { ref, defineComponent } from 'vue'

                export default defineComponent({
                  emits: ['incremented'],
                  setup(_, { emit }) {
                    const counter = ref(0)

                    emit('incremented', 'xxx')

                    return {
                      counter
                    }
                  }
                })
                </script>

",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                <script>
                import { ref, defineComponent } from 'vue'

                export default defineComponent({
                  emits: ['incremented'],
                  setup(_, { emit }) {
                    const counter = ref(0)

                    emit('incremented')

                    return {
                      counter
                    }
                  }
                })
                </script>

",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // === user-reported regressions (TDD failing tests, to be made pass by fixing the rule) ===
        // #1: triggerRef is a side-effectful function, NOT a factory. Return value is not a ref.
        (
            "
<script>
                import { ref, triggerRef } from 'vue'
                const foo = triggerRef
                foo()
                const bar = triggerRef(ref(0))
                bar++
                </script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // #2: setup(_, { foo }) — destructured prop is NOT `emit`, so passing a ref to `foo` is
        // NOT an emit-payload usage. Pass the ref directly (without `.value`) so the rule's
        // emit-arg detection would report it if `foo` were mistakenly treated as `emit`.
        (
            "
<script>
                import { ref, defineComponent } from 'vue'
                export default defineComponent({
                  setup(_, { foo }) {
                    const counter = ref(0)
                    foo('e', counter)
                    return { counter }
                  }
                })
                </script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // #4: `let foo = undefined; foo = ref(5); foo++` — upstream isRefInit requires init in defineChain;
        // `undefined` init is NOT a chain entry, so no usage of foo should be reported.
        (
            "
<script>
                import { ref } from 'vue'
                let foo = undefined
                foo = ref(5)
                foo++
                </script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // #6: local defineModel override is NOT the compiler macro.
        (
            "
<script>
                function defineModel() { return 0 }
                const x = defineModel()
                x++
                </script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
    ];

    let fail = vec![
        (
            "
<script>                  import { ref } from 'vue'
                  let count = ref(0)

                  count++ // error
                  console.log(count + 1) // error
                  console.log(1 + count) // error
                  </script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                  <script>
                    import { ref } from 'vue'
                    export default {
                      setup() {
                        let count = ref(0)

                        count++ // error
                        console.log(count + 1) // error
                        console.log(1 + count) // error
                        return {
                          count
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
                    import { ref } from '@vue/composition-api'
                    export default {
                      setup() {
                        let count = ref(0)

                        count++ // error
                        console.log(count + 1) // error
                        console.log(1 + count) // error
                        return {
                          count
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
<script>                  import { ref } from 'vue'
                  const foo = ref(true)
                  if (foo) {
                    //
                  }
                  </script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
<script>                  import { ref } from 'vue'
                  const foo = ref(true)
                  switch (foo) {
                    //
                  }
                  </script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
<script>                  import { ref } from 'vue'
                  const foo = ref(0)
                  var a = -foo
                  var b = +foo
                  var c = !foo
                  var d = ~foo
                  </script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
<script>                  import { ref } from 'vue'
                  let foo = ref(0)
                  foo += 1
                  foo -= 1
                  baz += foo
                  baz -= foo
                  </script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
<script>                  import { ref } from 'vue'
                  const foo = ref(true)
                  var a = foo || other
                  var b = foo && other
                  </script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
<script>                  import { ref } from 'vue'
                  let foo = ref(true)
                  var a = foo ? x : y
                  </script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                  <script>
                    import { ref } from 'vue'
                    let count = ref(0)
                    export default {
                      setup() {
                        count++ // error
                        console.log(count + 1) // error
                        console.log(1 + count) // error
                        return {
                          count
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
                    import { ref, computed, toRef, customRef, shallowRef } from 'vue'
                    let count = ref(0)
                    let cntcnt = computed(()=>count.value+count.value)

                    const state = reactive({
                      foo: 1,
                      bar: 2
                    })

                    const fooRef = toRef(state, 'foo')

                    let value = 'hello'
                    const cref = customRef((track, trigger) => {
                      return {
                        get() {
                          track()
                          return value
                        },
                        set(newValue) {
                          clearTimeout(timeout)
                          timeout = setTimeout(() => {
                            value = newValue
                            trigger()
                          }, delay)
                        }
                      }
                    })

                    const foo = shallowRef({})

                    count++ // error
                    cntcnt++ // error

                    const s = `${fooRef} : ${cref}` // error x 2

                    const n = foo + 1 // error
                  </script>

",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                  <script>
                    import { ref, computed, toRef, customRef, shallowRef } from 'vue'
                    const foo = shallowRef({})
                    foo.bar = 123
                  </script>

",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                  <script>
                    import { ref } from 'vue'
                    const foo = ref(123)
                    const bar = foo?.bar
                  </script>

",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                  <script>
                  import { ref } from 'vue'
                  let foo = undefined;

                  if (!foo) {
                    foo = ref(5);
                  }
                  let bar = foo;
                  bar = 4;
                  </script>

",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                  <script>
                  let model = defineModel();
                  console.log(model);
                  function process() {
                    if (model) console.log('foo')
                  }
                  function update(value) {
                    model = value;
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
                  let [model, mod] = defineModel();
                  console.log(model);
                  function process() {
                    if (model) console.log('foo')
                  }
                  function update(value) {
                    model = value;
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
                  import { ref } from 'vue'
                  const emits = defineEmits(['test'])
                  const count = ref(0)

                  function update() {
                    emits('test', count)
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
                  import { ref, defineComponent } from 'vue'

                  export default defineComponent({
                    emits: ['incremented'],
                    setup(_, ctx) {
                      const counter = ref(0)

                      ctx.emit('incremented', counter)

                      return {
                        counter
                      }
                    }
                  })
                  </script>

",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                  <script>
                  import { ref, defineComponent } from 'vue'

                  export default defineComponent({
                    emits: ['incremented'],
                    setup(_, { emit }) {
                      const counter = ref(0)

                      emit('incremented', counter)

                      return {
                        counter
                      }
                    }
                  })
                  </script>

",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                  <script>
                  import { ref, defineComponent } from 'vue'

                  export default defineComponent({
                    emits: ['incremented'],
                    setup(_, { emit }) {
                      const counter = ref(0)

                      emit('incremented', 'xxx', counter)

                      return {
                        counter
                      }
                    }
                  })
                  </script>

",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r"
<script>
                  let count = ref(0)

                  count++ // error
                  console.log(count + 1) // error
                  console.log(1 + count) // error
                  </script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // === user-reported regressions (TDD failing tests, to be made pass by fixing the rule) ===
        // #3: toRefs object destructure — each destructured property is a ref.
        (
            "
<script>
                import { toRefs } from 'vue'
                const state = { count: 0 }
                const { count } = toRefs(state)
                count++
                </script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // #5: foo['bar'] computed member with a string literal key — upstream uses
        // getStaticPropertyName, so foo['bar'] is reported the same as foo.bar.
        (
            "
<script>
                import { ref } from 'vue'
                const foo = ref(0)
                foo['bar']
                </script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // #7: emit('e', (count)) — parenthesised identifier argument; upstream sees through parens.
        (
            "
<script>
                import { ref, defineComponent } from 'vue'
                export default defineComponent({
                  emits: ['e'],
                  setup(_, { emit }) {
                    const count = ref(0)
                    emit('e', (count))
                    return { count }
                  }
                })
                </script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
    ];

    let fix = vec![
        (
            "
<script>
                  import { ref } from 'vue'
                  let count = ref(0)

                  count++ // error
                  console.log(count + 1) // error
                  console.log(1 + count) // error
                  </script>
",
            "
<script>
                  import { ref } from 'vue'
                  let count = ref(0)

                  count.value++ // error
                  console.log(count.value + 1) // error
                  console.log(1 + count.value) // error
                  </script>
",
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "

                  <script>
                    import { ref } from 'vue'
                    export default {
                      setup() {
                        let count = ref(0)

                        count++ // error
                        console.log(count + 1) // error
                        console.log(1 + count) // error
                        return {
                          count
                        }
                      }
                    }
                  </script>

",
            "

                  <script>
                    import { ref } from 'vue'
                    export default {
                      setup() {
                        let count = ref(0)

                        count.value++ // error
                        console.log(count.value + 1) // error
                        console.log(1 + count.value) // error
                        return {
                          count
                        }
                      }
                    }
                  </script>

",
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "

                  <script>
                    import { ref } from '@vue/composition-api'
                    export default {
                      setup() {
                        let count = ref(0)

                        count++ // error
                        console.log(count + 1) // error
                        console.log(1 + count) // error
                        return {
                          count
                        }
                      }
                    }
                  </script>

",
            "

                  <script>
                    import { ref } from '@vue/composition-api'
                    export default {
                      setup() {
                        let count = ref(0)

                        count.value++ // error
                        console.log(count.value + 1) // error
                        console.log(1 + count.value) // error
                        return {
                          count
                        }
                      }
                    }
                  </script>

",
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
<script>
                  import { ref } from 'vue'
                  const foo = ref(true)
                  if (foo) {
                    //
                  }
                  </script>
",
            "
<script>
                  import { ref } from 'vue'
                  const foo = ref(true)
                  if (foo.value) {
                    //
                  }
                  </script>
",
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
<script>
                  import { ref } from 'vue'
                  const foo = ref(true)
                  switch (foo) {
                    //
                  }
                  </script>
",
            "
<script>
                  import { ref } from 'vue'
                  const foo = ref(true)
                  switch (foo.value) {
                    //
                  }
                  </script>
",
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
<script>
                  import { ref } from 'vue'
                  const foo = ref(0)
                  var a = -foo
                  var b = +foo
                  var c = !foo
                  var d = ~foo
                  </script>
",
            "
<script>
                  import { ref } from 'vue'
                  const foo = ref(0)
                  var a = -foo.value
                  var b = +foo.value
                  var c = !foo.value
                  var d = ~foo.value
                  </script>
",
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
<script>
                  import { ref } from 'vue'
                  let foo = ref(0)
                  foo += 1
                  foo -= 1
                  baz += foo
                  baz -= foo
                  </script>
",
            "
<script>
                  import { ref } from 'vue'
                  let foo = ref(0)
                  foo.value += 1
                  foo.value -= 1
                  baz += foo.value
                  baz -= foo.value
                  </script>
",
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
<script>
                  import { ref } from 'vue'
                  const foo = ref(true)
                  var a = foo || other
                  var b = foo && other
                  </script>
",
            "
<script>
                  import { ref } from 'vue'
                  const foo = ref(true)
                  var a = foo.value || other
                  var b = foo.value && other
                  </script>
",
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
<script>
                  import { ref } from 'vue'
                  let foo = ref(true)
                  var a = foo ? x : y
                  </script>
",
            "
<script>
                  import { ref } from 'vue'
                  let foo = ref(true)
                  var a = foo.value ? x : y
                  </script>
",
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "

                  <script>
                    import { ref } from 'vue'
                    let count = ref(0)
                    export default {
                      setup() {
                        count++ // error
                        console.log(count + 1) // error
                        console.log(1 + count) // error
                        return {
                          count
                        }
                      }
                    }
                  </script>

",
            "

                  <script>
                    import { ref } from 'vue'
                    let count = ref(0)
                    export default {
                      setup() {
                        count.value++ // error
                        console.log(count.value + 1) // error
                        console.log(1 + count.value) // error
                        return {
                          count
                        }
                      }
                    }
                  </script>

",
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "

                  <script>
                    import { ref, computed, toRef, customRef, shallowRef } from 'vue'
                    let count = ref(0)
                    let cntcnt = computed(()=>count.value+count.value)

                    const state = reactive({
                      foo: 1,
                      bar: 2
                    })

                    const fooRef = toRef(state, 'foo')

                    let value = 'hello'
                    const cref = customRef((track, trigger) => {
                      return {
                        get() {
                          track()
                          return value
                        },
                        set(newValue) {
                          clearTimeout(timeout)
                          timeout = setTimeout(() => {
                            value = newValue
                            trigger()
                          }, delay)
                        }
                      }
                    })

                    const foo = shallowRef({})

                    count++ // error
                    cntcnt++ // error

                    const s = `${fooRef} : ${cref}` // error x 2

                    const n = foo + 1 // error
                  </script>

",
            "

                  <script>
                    import { ref, computed, toRef, customRef, shallowRef } from 'vue'
                    let count = ref(0)
                    let cntcnt = computed(()=>count.value+count.value)

                    const state = reactive({
                      foo: 1,
                      bar: 2
                    })

                    const fooRef = toRef(state, 'foo')

                    let value = 'hello'
                    const cref = customRef((track, trigger) => {
                      return {
                        get() {
                          track()
                          return value
                        },
                        set(newValue) {
                          clearTimeout(timeout)
                          timeout = setTimeout(() => {
                            value = newValue
                            trigger()
                          }, delay)
                        }
                      }
                    })

                    const foo = shallowRef({})

                    count.value++ // error
                    cntcnt.value++ // error

                    const s = `${fooRef.value} : ${cref.value}` // error x 2

                    const n = foo.value + 1 // error
                  </script>

",
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "

                  <script>
                    import { ref, computed, toRef, customRef, shallowRef } from 'vue'
                    const foo = shallowRef({})
                    foo.bar = 123
                  </script>

",
            "

                  <script>
                    import { ref, computed, toRef, customRef, shallowRef } from 'vue'
                    const foo = shallowRef({})
                    foo.value.bar = 123
                  </script>

",
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "

                  <script>
                    import { ref } from 'vue'
                    const foo = ref(123)
                    const bar = foo?.bar
                  </script>

",
            "

                  <script>
                    import { ref } from 'vue'
                    const foo = ref(123)
                    const bar = foo.value?.bar
                  </script>

",
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "

                  <script>
                  import { ref } from 'vue'
                  let foo = undefined;

                  if (!foo) {
                    foo = ref(5);
                  }
                  let bar = foo;
                  bar = 4;
                  </script>

",
            "

                  <script>
                  import { ref } from 'vue'
                  let foo = undefined;

                  if (!foo) {
                    foo = ref(5);
                  }
                  let bar = foo;
                  bar.value = 4;
                  </script>

",
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "

                  <script>
                  let model = defineModel();
                  console.log(model);
                  function process() {
                    if (model) console.log('foo')
                  }
                  function update(value) {
                    model = value;
                  }
                  </script>

",
            "

                  <script>
                  let model = defineModel();
                  console.log(model);
                  function process() {
                    if (model.value) console.log('foo')
                  }
                  function update(value) {
                    model.value = value;
                  }
                  </script>

",
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "

                  <script setup>
                  let [model, mod] = defineModel();
                  console.log(model);
                  function process() {
                    if (model) console.log('foo')
                  }
                  function update(value) {
                    model = value;
                  }
                  </script>

",
            "

                  <script setup>
                  let [model, mod] = defineModel();
                  console.log(model);
                  function process() {
                    if (model.value) console.log('foo')
                  }
                  function update(value) {
                    model.value = value;
                  }
                  </script>

",
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "

                  <script setup>
                  import { ref } from 'vue'
                  const emits = defineEmits(['test'])
                  const count = ref(0)

                  function update() {
                    emits('test', count)
                  }
                  </script>

",
            "

                  <script setup>
                  import { ref } from 'vue'
                  const emits = defineEmits(['test'])
                  const count = ref(0)

                  function update() {
                    emits('test', count.value)
                  }
                  </script>

",
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "

                  <script>
                  import { ref, defineComponent } from 'vue'

                  export default defineComponent({
                    emits: ['incremented'],
                    setup(_, ctx) {
                      const counter = ref(0)

                      ctx.emit('incremented', counter)

                      return {
                        counter
                      }
                    }
                  })
                  </script>

",
            "

                  <script>
                  import { ref, defineComponent } from 'vue'

                  export default defineComponent({
                    emits: ['incremented'],
                    setup(_, ctx) {
                      const counter = ref(0)

                      ctx.emit('incremented', counter.value)

                      return {
                        counter
                      }
                    }
                  })
                  </script>

",
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "

                  <script>
                  import { ref, defineComponent } from 'vue'

                  export default defineComponent({
                    emits: ['incremented'],
                    setup(_, { emit }) {
                      const counter = ref(0)

                      emit('incremented', counter)

                      return {
                        counter
                      }
                    }
                  })
                  </script>

",
            "

                  <script>
                  import { ref, defineComponent } from 'vue'

                  export default defineComponent({
                    emits: ['incremented'],
                    setup(_, { emit }) {
                      const counter = ref(0)

                      emit('incremented', counter.value)

                      return {
                        counter
                      }
                    }
                  })
                  </script>

",
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "

                  <script>
                  import { ref, defineComponent } from 'vue'

                  export default defineComponent({
                    emits: ['incremented'],
                    setup(_, { emit }) {
                      const counter = ref(0)

                      emit('incremented', 'xxx', counter)

                      return {
                        counter
                      }
                    }
                  })
                  </script>

",
            "

                  <script>
                  import { ref, defineComponent } from 'vue'

                  export default defineComponent({
                    emits: ['incremented'],
                    setup(_, { emit }) {
                      const counter = ref(0)

                      emit('incremented', 'xxx', counter.value)

                      return {
                        counter
                      }
                    }
                  })
                  </script>

",
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
<script>
                  let count = ref(0)

                  count++ // error
                  console.log(count + 1) // error
                  console.log(1 + count) // error
                  </script>
",
            "
<script>
                  let count = ref(0)

                  count.value++ // error
                  console.log(count.value + 1) // error
                  console.log(1 + count.value) // error
                  </script>
",
            None,
            Some(PathBuf::from("test.vue")),
        ),
    ];

    Tester::new(NoRefAsOperand::NAME, NoRefAsOperand::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
