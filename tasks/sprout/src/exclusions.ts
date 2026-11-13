// Snapshot-only fixtures must state the concrete runtime limitation.
export const runtimeExclusions = new Map([
  ["ssa-for-trivial-update.js", "Entrypoint contains an intentional infinite loop"],
  ["ssa-while-no-reassign.js", "Entrypoint contains an intentional infinite loop"],
  ["reanimated-no-memo-arg.js", "Requires the React Native Reanimated runtime"],
  [
    "lowercase-context-variable-as-jsx-element-tag.js",
    "Module initialization calls the undefined helper getTitle",
  ],
  [
    "fast-refresh-refresh-on-const-changes-dev.js",
    "Requires Fast Refresh to invalidate caches after changing module globals",
  ],
  [
    "early-return-no-declarations-reassignments-dependencies.js",
    "Intentionally observes render-phase console.log calls removed by upstream memoization",
  ],
  [
    "capturing-function-decl.js",
    "Calls t.foo(), but the fixture only defines t.a; retained for closure lowering snapshots",
  ],
  [
    "capturing-function-skip-computed-path.js",
    "Immediately invokes a callback without its gift argument; retained for computed dependency snapshots",
  ],
  [
    "constant-computed.js",
    "Unconditionally calls x.foo after assigning it NaN; retained for computed-access snapshots",
  ],
  [
    "function-declaration-simple.js",
    "Calls p.foo() on an object that has no foo method; retained for function declaration snapshots",
  ],
  [
    "for-empty-update.js",
    "Loop never increments i and cannot terminate when props.count is positive",
  ],
  [
    "overlapping-scopes-while.js",
    "Loop condition c never changes; retained for overlapping-scope snapshots",
  ],
  ["ssa-nested-loops-no-reassign.js", "Nested loop conditions never change and contain no exit"],
  [
    "while-conditional-continue.js",
    "Loop condition a never changes and every branch continues without an exit",
  ],
  ["while-logical.js", "Loop repeatedly doubles zero without changing its condition"],
  ["while-property.js", "Loop condition a.b.c never changes and contains no exit"],
  [
    "alias-capture-in-method-receiver.js",
    "Analysis-only fixture references undefined helpers: someObj",
  ],
  ["alias-while.js", "Loop condition cond never changes and has no exit"],
  [
    "allocating-primitive-as-dep.js",
    "Analysis-only fixture references undefined helpers: foo, bar",
  ],
  ["allow-merge-refs-pattern.js", "Analysis-only fixture references undefined helpers: mergeRefs"],
  [
    "allow-passing-ref-to-render-helper-props-object.js",
    "Analysis-only fixture references undefined helpers: Foo",
  ],
  [
    "allow-passing-ref-to-render-helper.js",
    "Analysis-only fixture references undefined helpers: Foo",
  ],
  ["allow-passing-refs-as-props.js", "Analysis-only fixture references undefined helpers: Foo"],
  ["array-at-closure.js", "Analysis-only fixture references undefined helpers: foo, bar"],
  ["array-at-effect.js", "Analysis-only fixture references undefined helpers: foo, bar"],
  ["array-at-mutate-after-capture.js", "Analysis-only fixture references undefined helpers: foo"],
  ["array-join.js", "Analysis-only fixture references undefined helpers: foo"],
  ["array-push-effect.js", "Analysis-only fixture references undefined helpers: foo"],
  ["assignment-in-nested-if.js", "Analysis-only fixture references undefined helpers: baz"],
  [
    "async-captured-factory-cycle.js",
    "Mutually recursive make and wrap factories have no base case",
  ],
  [
    "async-sync-wrapper-recursive.js",
    "The stop branch recursively calls start without a base case",
  ],
  [
    "await-side-effecting-promise.js",
    "Analysis-only fixture references undefined helpers: populateData",
  ],
  ["await.js", "Analysis-only fixture references undefined helpers: load"],
  [
    "builtin-jsx-tag-lowered-between-mutations.js",
    "Analysis-only fixture references undefined helpers: MaybeMutable, maybeMutate",
  ],
  ["call-args-assignment.js", "Analysis-only fixture references undefined helpers: makeObject"],
  [
    "call-args-destructuring-assignment.js",
    "Analysis-only fixture references undefined helpers: makeObject",
  ],
  [
    "call-with-independently-memoizable-arg.js",
    "Analysis-only fixture references undefined helpers: makeFunction",
  ],
  [
    "capture-param-mutate.js",
    "Analysis-only fixture references undefined helpers: inspect, LOG_LEVELS, INSPECTOR_LEVELS, INSPECTOR_FRAMES_TO_SKIP, groupStack, groupFormat",
  ],
  [
    "capturing-function-conditional-capture-mutate.js",
    "Analysis-only fixture references undefined helpers: maybeMutate",
  ],
  [
    "capturing-function-member-expr-arguments.js",
    "Analysis-only fixture references undefined helpers: log",
  ],
  [
    "capturing-function-member-expr-call.js",
    "Analysis-only fixture references undefined helpers: Foo",
  ],
  [
    "compiled-function-nested-in-jsx.js",
    "Analysis-only fixture references undefined helpers: Search",
  ],
  ["computed-call-spread.js", "Analysis-only fixture references undefined helpers: foo"],
  [
    "computed-load-primitive-as-dependency.js",
    "Analysis-only fixture references undefined helpers: foo, bar",
  ],
  ["concise-arrow-expr.js", "Analysis-only fixture references undefined helpers: Foo"],
  [
    "const-propagation-into-function-expression-global.js",
    "Analysis-only fixture references undefined helpers: GLOBAL_IS_X, Child",
  ],
  [
    "default-parameter-captures-non-plain-ref-param.js",
    "forwardRef renders its props object as a child; retained for default-parameter analysis",
  ],
  [
    "default-suppression-eslint-block-range.js",
    "Intentionally produces compiler suppression diagnostics",
  ],
  [
    "default-suppression-eslint-next-line.js",
    "Intentionally produces compiler suppression diagnostics",
  ],
  ["default-suppression-flow.js", "Intentionally produces compiler suppression diagnostics"],
  [
    "destructuring-mixed-scope-declarations-and-locals.js",
    "Analysis-only fixture references undefined helpers: Media",
  ],
  [
    "destructuring-property-inference.js",
    "Analysis-only fixture references undefined helpers: foo",
  ],
  [
    "do-while-conditional-break.js",
    "The array x cannot equal zero and props.cond never changes, so the loop cannot terminate",
  ],
  [
    "dont-memoize-primitive-function-call-non-escaping-useMemo.js",
    "Upstream TODO entrypoint intentionally observes render-phase logging removed by memoization",
  ],
  [
    "duplicate-named-function-expression-names.tsx",
    "Renders column descriptor objects as React children; retained for function-name scope snapshots",
  ],
  [
    "effect-local-reassigned-by-interval-callback.js",
    "Leaves a repeating interval active; requires bounded timer steps",
  ],
  [
    "exhaustive-deps/compile-files-with-exhaustive-deps-violation-in-effects.js",
    "Intentionally produces exhaustive-deps suppression diagnostics",
  ],
  [
    "exhaustive-deps/exhaustive-deps-effect-events.js",
    "Analysis-only fixture references undefined helpers: log",
  ],
  [
    "fbt/fbt-params-complex-param-value.js",
    "Analysis-only fixture references undefined helpers: capitalize",
  ],
  [
    "fbt/fbtparam-text-must-use-expression-container.js",
    "Analysis-only fixture references undefined helpers: Foo",
  ],
  [
    "fbt/fbtparam-with-jsx-fragment-value.js",
    "Analysis-only fixture references undefined helpers: Foo",
  ],
  [
    "fbt/lambda-with-fbt.js",
    "Analysis-only fixture references undefined helpers: someCondition, iconOnly, showPrice, item, View, Button",
  ],
  [
    "function-decl-shadowed-by-inner-const.js",
    "Analysis-only fixture references undefined helpers: makeInit",
  ],
  [
    "function-expression-captures-value-later-frozen-jsx.js",
    "Analysis-only fixture references undefined helpers: maybeMutate, Foo",
  ],
  [
    "function-expression-maybe-mutates-hook-return-value.js",
    "Analysis-only fixture references undefined helpers: useSelectedEntitytId, log, Foo",
  ],
  [
    "function-expression-with-store-to-parameter.js",
    "Analysis-only fixture references undefined helpers: makeObject",
  ],
  [
    "gating/multi-arrow-expr-export-default-gating-test.js",
    "Analysis-only fixture references undefined helpers: Foo, Bar",
  ],
  [
    "gating/nested-function-gating.js",
    "The hook is block-scoped inside the inactive __DEV__ branch and has no external entrypoint",
  ],
  [
    "gating/repro-no-gating-import-without-compiled-functions.js",
    "Only exports a constant; asserts that no gating import is emitted",
  ],
  [
    "global-jsx-tag-lowered-between-mutations.js",
    "Analysis-only fixture references undefined helpers: MaybeMutable, View, maybeMutate",
  ],
  ["hir_id_numbering.js", "Analysis-only fixture references undefined helpers: fn, timeoutMs"],
  [
    "hir_loc_diff.js",
    "Analysis-only fixture references undefined helpers: queryParams, value, array, paramParts",
  ],
  [
    "hoisting-setstate-captured-indirectly-jsx.js",
    "Analysis-only fixture references undefined helpers: DISABLED_FORM, ColumnItem",
  ],
  ["hook-call.js", "Component unconditionally renders itself recursively"],
  [
    "idx-method-no-outlining-wildcard.js",
    "Analysis-only fixture references undefined helpers: idx",
  ],
  ["idx-method-no-outlining.js", "Analysis-only fixture references undefined helpers: idx"],
  ["immutable-hooks.js", "Analysis-only fixture references undefined helpers: useFoo, bar"],
  [
    "inadvertent-mutability-readonly-class.js",
    "Analysis-only fixture references undefined helpers: useRelayEnvironment, useOtherHook, foo",
  ],
  [
    "inadvertent-mutability-readonly-lambda.js",
    "Analysis-only fixture references undefined helpers: useOtherHook, foo",
  ],
  ["infer-computed-delete.js", "Analysis-only fixture references undefined helpers: makeObject"],
  [
    "infer-function-expression-component.js",
    "Analysis-only fixture references undefined helpers: Foo",
  ],
  ["infer-property-delete.js", "Analysis-only fixture references undefined helpers: makeObject"],
  [
    "infer-skip-components-without-hooks-or-jsx.js",
    "Calls an undefined render helper; retained for component inference snapshots",
  ],
  [
    "inner-function-param-name-not-scoped.js",
    "Analysis-only fixture references undefined helpers: List",
  ],
  [
    "inner-memo-value-not-promoted-to-outer-scope-dynamic.js",
    "Analysis-only fixture references undefined helpers: FRAGMENT, useFreeze, MaybeMutable, View, maybeMutate",
  ],
  [
    "inner-memo-value-not-promoted-to-outer-scope-static.js",
    "Analysis-only fixture references undefined helpers: MaybeMutable, View, maybeMutate",
  ],
  [
    "jsx-member-expression-tag-grouping.js",
    "Analysis-only fixture references undefined helpers: MaybeMutable, Foo, maybeMutate",
  ],
  ["jsx-member-expression.js", "Analysis-only fixture references undefined helpers: Sathya, Foo"],
  ["jsx-spread.js", "Component unconditionally renders itself recursively"],
  [
    "keep-manual-memo-extra-deps.jsx",
    "Analysis-only fixture references undefined helpers: readGlobal",
  ],
  [
    "keep-manual-memo-missing-deps-rest-props.jsx",
    "Analysis-only fixture references undefined helpers: sortBy",
  ],
  [
    "keep-manual-memo-missing-deps.jsx",
    "Analysis-only fixture references undefined helpers: sortBy, Rows",
  ],
  [
    "keep-manual-memo-no-hoisted-property-read.jsx",
    "Analysis-only fixture references undefined helpers: sortBy",
  ],
  [
    "keep-manual-memo-non-inline-callback.jsx",
    "Analysis-only fixture references undefined helpers: sortBy, Rows",
  ],
  [
    "keep-manual-memo-one-of-two.jsx",
    "Analysis-only fixture references undefined helpers: sortBy, Rows",
  ],
  [
    "keep-manual-memo-set-state-still-bails.jsx",
    "Analysis-only fixture references undefined helpers: sortBy, Rows",
  ],
  [
    "lambda-capture-returned-alias.js",
    "Analysis-only fixture references undefined helpers: foo, bar",
  ],
  ["lone-surrogate-string-values.js", "Analysis-only fixture references undefined helpers: Emoji"],
  ["loop-unused-let.js", "Unconditional while loop has no exit"],
  [
    "meta-property.mjs",
    "Exercises native import.meta semantics unavailable in the CommonJS evaluator",
  ],
  ["method-call-computed.js", "Analysis-only fixture references undefined helpers: makeObject"],
  ["method-call-fn-call.js", "Analysis-only fixture references undefined helpers: makeObject"],
  [
    "mutable-liverange-loop.js",
    "Infinite loop: cond always returns undefined and cannot reach the break",
  ],
  [
    "named-function-expression-self-reference.tsx",
    "Analysis-only fixture references undefined helpers: Tree",
  ],
  [
    "nested-function-discovery-annotation-mode.js",
    "The hook is block-scoped inside the inactive __DEV__ branch and has no external entrypoint",
  ],
  [
    "nested-function-discovery-if-test-expr.js",
    "The hook exists only as an if condition and is not exposed for invocation",
  ],
  [
    "nested-function-discovery-while-test-expr.js",
    "The hook exists only as a while condition and is not exposed for invocation",
  ],
  ["nested-optional-member-expr.js", "Analysis-only fixture references undefined helpers: foo"],
  ["new-mutability/array-push.js", "Analysis-only fixture references undefined helpers: Foo"],
  [
    "new-mutability/basic-mutation-via-function-expression.js",
    "Renders a plain object as a React child; retained for alias analysis snapshots",
  ],
  [
    "new-mutability/basic-mutation.js",
    "Renders a plain object as a React child; retained for alias analysis snapshots",
  ],
  [
    "new-mutability/mutate-through-propertyload.js",
    "Renders a plain object as a React child; retained for property mutation snapshots",
  ],
  [
    "new-mutability/potential-mutation-in-function-expression.js",
    "Analysis-only fixture references undefined helpers: maybeMutate, Foo",
  ],
  [
    "new-mutability/repro-destructure-from-prop-with-default-value.js",
    "Analysis-only fixture references undefined helpers: dateTimeFormat",
  ],
  [
    "new-mutability/repro-invalid-function-expression-effects-phi.js",
    "Renders a plain object as a React child; retained for effect analysis snapshots",
  ],
  [
    "new-mutability/transitive-mutation-before-capturing-value-created-earlier.js",
    "Renders a plain object as a React child; retained for transitive mutation snapshots",
  ],
  [
    "new-mutability/valid-new-date-from-timestamp.js",
    "Analysis-only fixture references undefined helpers: Foo",
  ],
  ["new-spread.js", "Analysis-only fixture references undefined helpers: Foo"],
  [
    "no-alias-callback-return-does-not-flow-to-call-result.js",
    "Requires an unavailable fixture module: ReactCompilerNoAliasTest",
  ],
  [
    "obj-literal-cached-in-if-else.js",
    "Analysis-only fixture references undefined helpers: someVal",
  ],
  [
    "obj-literal-mutated-after-if-else.js",
    "Analysis-only fixture references undefined helpers: someVal",
  ],
  [
    "obj-mutated-after-if-else-with-alias.js",
    "Analysis-only fixture references undefined helpers: someObj",
  ],
  ["obj-mutated-after-if-else.js", "Analysis-only fixture references undefined helpers: someObj"],
  [
    "obj-mutated-after-nested-if-else-with-alias.js",
    "Analysis-only fixture references undefined helpers: someObj",
  ],
  ["object-properties.js", "foo recursively calls itself without a base case"],
  ["optional-call-chained.js", "Analysis-only fixture references undefined helpers: call"],
  ["optional-call-logical.js", "Analysis-only fixture references undefined helpers: renderItem"],
  ["optional-call-simple.js", "Analysis-only fixture references undefined helpers: foo"],
  [
    "optional-call-with-independently-memoizable-arg.js",
    "Analysis-only fixture references undefined helpers: makeOptionalFunction",
  ],
  [
    "optional-call.js",
    "Analysis-only fixture references undefined helpers: makeOptionalFunction, makeObject, foo, bar",
  ],
  [
    "optional-chain-on-known-nonnull.js",
    "Analysis-only fixture references undefined helpers: nullthrows",
  ],
  ["optional-computed-load-static.js", "Analysis-only fixture references undefined helpers: a"],
  [
    "optional-computed-member-expression.js",
    "Analysis-only fixture references undefined helpers: makeObject",
  ],
  [
    "optional-member-expression-call-as-property.js",
    "Analysis-only fixture references undefined helpers: makeObject, foo",
  ],
  [
    "optional-member-expression-inverted-optionals-parallel-paths.js",
    "Analysis-only fixture references undefined helpers: x",
  ],
  [
    "optional-member-expression-with-optional-member-expr-as-property.js",
    "Analysis-only fixture references undefined helpers: makeObject",
  ],
  [
    "optional-member-expression.js",
    "Analysis-only fixture references undefined helpers: bar, useBar",
  ],
  [
    "optional-method-call.js",
    "Analysis-only fixture references undefined helpers: makeObject, foo, bar",
  ],
  [
    "optional-receiver-method-call.js",
    "Analysis-only fixture references undefined helpers: makeOptionalObject, makeObject, foo, bar",
  ],
  [
    "optional-receiver-optional-method.js",
    "Analysis-only fixture references undefined helpers: makeOptionalObject, makeObject, foo, bar",
  ],
  [
    "outlined-function-module-global-dependencies.js",
    "Requires an unavailable fixture module: ./notify",
  ],
  [
    "outlined-function-outer-capture-declaration.js",
    "Requires an unavailable fixture module: ./notify",
  ],
  [
    "pattern4_bare_type.js",
    "Analysis-only fixture references undefined helpers: logs, traceBeginTimeSec",
  ],
  [
    "preserve-computed-property-access.js",
    "Analysis-only fixture references undefined helpers: useParams",
  ],
  [
    "preserve-existing-memoization-guarantees/lambda-with-fbt-preserve-memoization.js",
    "Analysis-only fixture references undefined helpers: someCondition, iconOnly, showPrice, item, View, Button",
  ],
  ["primitive-as-dep.js", "Analysis-only fixture references undefined helpers: foo"],
  [
    "propagate-scope-deps-hir-fork/optional-member-expression-inverted-optionals-parallel-paths.js",
    "Analysis-only fixture references undefined helpers: x",
  ],
  [
    "propagate-scope-deps-hir-fork/switch-non-final-default.js",
    "Component recursively renders itself; some switch branches also leave y uninitialized",
  ],
  ["propagate-scope-deps-hir-fork/switch.js", "Component recursively renders itself"],
  ["property-assignment.js", "Component recursively renders itself"],
  ["property-call-spread.js", "Analysis-only fixture references undefined helpers: foo"],
  [
    "reactivity-analysis-reactive-via-mutation-of-computed-load.js",
    "Analysis-only fixture references undefined helpers: bar, foo",
  ],
  [
    "reactivity-analysis-reactive-via-mutation-of-property-load.js",
    "Analysis-only fixture references undefined helpers: bar, foo",
  ],
  [
    "readonly-object-method-calls-mutable-lambda.js",
    "Analysis-only fixture references undefined helpers: makeObject, Post, foo",
  ],
  [
    "readonly-object-method-calls.js",
    "Analysis-only fixture references undefined helpers: Post, foo",
  ],
  [
    "reanimated-shared-value-writes.jsx",
    "Requires an unavailable fixture module: react-native-reanimated",
  ],
  [
    "reassigned-phi-in-returned-function-expression.js",
    "Returned closure calls the undefined global.log helper",
  ],
  ["reassignment-conditional.js", "Component recursively renders itself"],
  ["reassignment.js", "Component recursively renders itself"],
  [
    "ref-current-aliased-no-added-to-dep.js",
    "Analysis-only fixture references undefined helpers: VideoList",
  ],
  [
    "ref-current-aliased-not-added-to-dep-2.js",
    "Analysis-only fixture references undefined helpers: VideoList",
  ],
  [
    "ref-current-field-not-added-to-dep.js",
    "Analysis-only fixture references undefined helpers: VideoList",
  ],
  [
    "ref-current-not-added-to-dep-2.js",
    "Analysis-only fixture references undefined helpers: VideoList",
  ],
  [
    "ref-current-not-added-to-dep.js",
    "Analysis-only fixture references undefined helpers: VideoList",
  ],
  [
    "ref-current-optional-field-no-added-to-dep.js",
    "Analysis-only fixture references undefined helpers: VideoList",
  ],
  [
    "ref-current-write-not-added-to-dep.js",
    "Analysis-only fixture references undefined helpers: VideoList",
  ],
  ["ref-in-effect.js", "Analysis-only fixture references undefined helpers: Foo"],
  [
    "ref-type-convergence-recursive-closure.jsx",
    "Analysis-only fixture references undefined helpers: subscribe",
  ],
  ["regexp-literal.js", "Analysis-only fixture references undefined helpers: makeValue"],
  [
    "repro-bailout-nopanic-shouldnt-outline.js",
    "Analysis-only fixture references undefined helpers: alert",
  ],
  [
    "repro-memoize-for-of-collection-when-loop-body-returns.js",
    "Analysis-only fixture references undefined helpers: GraphContext, Class, field",
  ],
  [
    "repro-no-declarations-in-reactive-scope-with-early-return.js",
    "Analysis-only fixture references undefined helpers: useItems, item",
  ],
  [
    "repro-propagate-type-of-ternary-jsx.js",
    "Analysis-only fixture references undefined helpers: Component8, va, ComponentC, ComponentE",
  ],
  [
    "repro-propagate-type-of-ternary-nested.js",
    "Analysis-only fixture references undefined helpers: v5, V6, V9, ComponentC, v11, V3, v16",
  ],
  [
    "repro-reassign-to-variable-without-mutable-range.js",
    "Analysis-only fixture references undefined helpers: foo, FLAG, bar, baz",
  ],
  ["repro.js", "Analysis-only fixture references undefined helpers: getBaseVideos, FlatList"],
  [
    "rewrite-phis-in-lambda-capture-context.js",
    "Analysis-only fixture references undefined helpers: bar, baz",
  ],
  [
    "round2_loc_diff.js",
    "Analysis-only fixture references undefined helpers: queryParams, value, array, paramParts",
  ],
  [
    "round3_effect_read_vs_capture.js",
    "Analysis-only fixture references undefined helpers: profile, name, description",
  ],
  [
    "rules-of-hooks/rules-of-hooks-0592bd574811.js",
    "Analysis-only fixture references undefined helpers: each, pixelsWithInferredEvents, useExtendedSelector, extendedButton, foo",
  ],
  [
    "rules-of-hooks/rules-of-hooks-0e2214abc294.js",
    "Analysis-only fixture references undefined helpers: page",
  ],
  [
    "rules-of-hooks/rules-of-hooks-23dc7fffde57.js",
    "Analysis-only fixture references undefined helpers: useHook1, useHook2",
  ],
  [
    "rules-of-hooks/rules-of-hooks-2bec02ac982b.js",
    "Analysis-only fixture references undefined helpers: useHook1, useHook2",
  ],
  [
    "rules-of-hooks/rules-of-hooks-2e405c78cb80.js",
    "Analysis-only fixture references undefined helpers: a",
  ],
  [
    "rules-of-hooks/rules-of-hooks-347b0dae66f1.js",
    "Analysis-only fixture references undefined helpers: doSomething",
  ],
  [
    "rules-of-hooks/rules-of-hooks-485bf041f55f.js",
    "Analysis-only fixture references undefined helpers: cond, userFetch",
  ],
  [
    "rules-of-hooks/rules-of-hooks-7e52f5eec669.js",
    "Analysis-only fixture references undefined helpers: doSomething",
  ],
  [
    "rules-of-hooks/rules-of-hooks-9d7879272ff6.js",
    "Analysis-only fixture references undefined helpers: useHook1, useHook2",
  ],
  [
    "rules-of-hooks/rules-of-hooks-c1e8c7f4c191.js",
    "Analysis-only fixture references undefined helpers: c",
  ],
  [
    "rules-of-hooks/rules-of-hooks-c5d1f3143c4c.js",
    "Analysis-only fixture references undefined helpers: cond, a, b",
  ],
  [
    "rules-of-hooks/rules-of-hooks-cfdfe5572fc7.js",
    "Analysis-only fixture references undefined helpers: useHook1, useHook2",
  ],
  [
    "rules-of-hooks/rules-of-hooks-df4d750736f3.js",
    "Analysis-only fixture references undefined helpers: fooState, _use, _useState, use_hook, jest",
  ],
  [
    "rules-of-hooks/rules-of-hooks-dfde14171fcd.js",
    "Class method calls missing this.useHook and super.useHook methods",
  ],
  [
    "rules-of-hooks/rules-of-hooks-e5dd6caf4084.js",
    "Analysis-only fixture references undefined helpers: cond, doSomething",
  ],
  [
    "rules-of-hooks/rules-of-hooks-fe6042f7628b.js",
    "Analysis-only fixture references undefined helpers: unknownFunction, useNotAHook",
  ],
  [
    "rules-of-hooks/todo.bail.rules-of-hooks-279ac76f53af.js",
    "Analysis-only fixture references undefined helpers: jest, beforeEach",
  ],
  [
    "rules-of-hooks/todo.bail.rules-of-hooks-6949b255e7eb.js",
    "Analysis-only fixture references undefined helpers: FILLER, anyConditionCanEvenBeFalse",
  ],
  [
    "rules-of-hooks/todo.bail.rules-of-hooks-e0a5db3ae21e.js",
    "Analysis-only fixture references undefined helpers: Namespace",
  ],
  [
    "rules-of-hooks/todo.bail.rules-of-hooks-fadd52c1e460.js",
    "Top-level assignments to undeclared e and k intentionally exercise hook-name classification",
  ],
  [
    "rules-of-hooks/todo.invalid.invalid-rules-of-hooks-191029ac48c8.js",
    "Analysis-only fixture references undefined helpers: foo, useCustomHook",
  ],
  [
    "rules-of-hooks/todo.invalid.invalid-rules-of-hooks-206e2811c87c.js",
    "Analysis-only fixture references undefined helpers: Component, cond, FooStore",
  ],
  [
    "rules-of-hooks/todo.invalid.invalid-rules-of-hooks-28a7111f56a7.js",
    "Analysis-only fixture references undefined helpers: require",
  ],
  [
    "rules-of-hooks/todo.invalid.invalid-rules-of-hooks-2c51251df67a.js",
    "Anonymous class with an invalid hook method has no callable component entrypoint",
  ],
  [
    "rules-of-hooks/todo.invalid.invalid-rules-of-hooks-5a7ac9a6e8fa.js",
    "Analysis-only fixture references undefined helpers: useHookInsideNormalFunction",
  ],
  [
    "rules-of-hooks/todo.invalid.invalid-rules-of-hooks-8303403b8e4c.js",
    "Intentionally calls a hook inside a class render method",
  ],
  [
    "rules-of-hooks/todo.invalid.invalid-rules-of-hooks-99b5c750d1d1.js",
    "Analysis-only fixture references undefined helpers: foo, useFeatureFlag",
  ],
  [
    "rules-of-hooks/todo.invalid.invalid-rules-of-hooks-9c79feec4b9b.js",
    "Anonymous class with an invalid hook field has no callable component entrypoint",
  ],
  [
    "rules-of-hooks/todo.invalid.invalid-rules-of-hooks-a63fd4f9dcc0.js",
    "Analysis-only fixture references undefined helpers: useProbablyAHook",
  ],
  [
    "rules-of-hooks/todo.invalid.invalid-rules-of-hooks-acb56658fe7e.js",
    "Analysis-only fixture references undefined helpers: This, Super",
  ],
  [
    "rules-of-hooks/todo.invalid.invalid-rules-of-hooks-c59788ef5676.js",
    "Analysis-only fixture references undefined helpers: useHookInsideNormalFunction",
  ],
  [
    "rules-of-hooks/todo.invalid.invalid-rules-of-hooks-ddeca9708b63.js",
    "Anonymous class with an invalid hook method has no callable component entrypoint",
  ],
  [
    "rules-of-hooks/todo.invalid.invalid-rules-of-hooks-e675f0a672d8.js",
    "Intentionally calls hooks inside an array map callback",
  ],
  [
    "rules-of-hooks/todo.invalid.invalid-rules-of-hooks-e69ffce323c3.js",
    "Anonymous class with an invalid hook field has no callable component entrypoint",
  ],
  [
    "scope-boundary-at-if-branch.js",
    "Analysis-only fixture references undefined helpers: createURI",
  ],
  ["sequence-expression.js", "Unconditional while loop has no exit"],
  ["ssa-reassign-in-rval.js", "Analysis-only fixture references undefined helpers: foo, bar"],
  ["ssr/optimize-ssr.js", "Analysis-only fixture references undefined helpers: log"],
  [
    "ssr/ssr-infer-event-handlers-from-setState.js",
    "Analysis-only fixture references undefined helpers: log, CustomInput",
  ],
  [
    "ssr/ssr-infer-event-handlers-from-startTransition.js",
    "Analysis-only fixture references undefined helpers: log, CustomInput",
  ],
  ["ssr/ssr-use-reducer-initializer.js", "Analysis-only fixture references undefined helpers: log"],
  ["ssr/ssr-use-reducer.js", "Analysis-only fixture references undefined helpers: log"],
  ["store-via-call.js", "foo recursively calls itself without a base case"],
  ["store-via-new.js", "Foo recursively constructs itself without a base case"],
  [
    "switch-global-propertyload-case-test.js",
    "Analysis-only fixture references undefined helpers: Global",
  ],
  [
    "switch-non-final-default.js",
    "Component recursively renders itself; some switch branches also leave y uninitialized",
  ],
  ["switch.js", "Component recursively renders itself"],
  ["symbol-call.js", "Analysis-only fixture references undefined helpers: Widget"],
  ["tagged-template-escaped.js", "Analysis-only fixture references undefined helpers: tag"],
  ["tagged-template-invalid-escape.js", "Analysis-only fixture references undefined helpers: tag"],
  ["template-literal.js", "Analysis-only fixture references undefined helpers: useFoo"],
  [
    "temporary-accessed-outside-scope.js",
    "Analysis-only fixture references undefined helpers: MaybeMutable, maybeMutate",
  ],
  [
    "temporary-at-start-of-value-block.js",
    "Analysis-only fixture references undefined helpers: isMenuShown, Bar",
  ],
  [
    "temporary-property-load-accessed-outside-scope.js",
    "Analysis-only fixture references undefined helpers: MaybeMutable, maybeMutate",
  ],
  [
    "todo-function-expression-captures-value-later-frozen.js",
    "Analysis-only fixture references undefined helpers: maybeMutate, Foo",
  ],
  [
    "todo-hir_identifier_diff.js",
    "Analysis-only fixture references undefined helpers: jest, require, RouterRenderTypeContext",
  ],
  [
    "todo-hir_numeric_format.js",
    "Analysis-only fixture references undefined helpers: describe, afterEach, it, logUnsafeIntResponse, expect, MockLogger",
  ],
  [
    "todo-pattern1b_type_to_primitive.js",
    "Analysis-only fixture references undefined helpers: describe, it, expect, equalsIterable, mapOne, mapTwo, compareKeyValuePair",
  ],
  [
    "todo-round2_unicode_string.js",
    "Analysis-only fixture references undefined helpers: test, device",
  ],
  [
    "todo-round3_promote_used_temps.js",
    "Analysis-only fixture references undefined helpers: require, exports",
  ],
  [
    "todo.memoize-loops-that-produce-memoizeable-values.js",
    "Analysis-only fixture references undefined helpers: GraphContext, Class, field",
  ],
  [
    "todo.unnecessary-lambda-memoization.js",
    "Analysis-only fixture references undefined helpers: useFreeze, Item",
  ],
  [
    "transitive-freeze-function-expressions.js",
    "Analysis-only fixture references undefined helpers: usePaginationFragment",
  ],
  [
    "try-catch-phi-constant-declaration.jsx",
    "Analysis-only fixture references undefined helpers: f",
  ],
  ["try-catch-phi-nested-conditional.jsx", "Analysis-only fixture references undefined helpers: f"],
  [
    "try-catch-phi-pruned-edge-values.jsx",
    "Analysis-only fixture references undefined helpers: read",
  ],
  ["ts-namespace-export-declaration.ts", "Analysis-only fixture references undefined helpers: Foo"],
  [
    "type-binary-operator.js",
    "Analysis-only fixture references undefined helpers: some, someOther",
  ],
  [
    "type-provider-tagged-template-expression.js",
    "Requires an unavailable fixture module: ReactCompilerPureTagTest",
  ],
  [
    "type-test-field-load-binary-op.js",
    "Analysis-only fixture references undefined helpers: makeSomePrimitive",
  ],
  ["type-test-polymorphic.js", "Analysis-only fixture references undefined helpers: makePrimitive"],
  ["type-test-return-type-inference.js", "Analysis-only fixture references undefined helpers: foo"],
  [
    "unclosed-eslint-suppression-skips-all-components.js",
    "Intentionally produces compiler suppression diagnostics",
  ],
  [
    "uninitialized-declaration-in-reactive-scope.js",
    "Analysis-only fixture references undefined helpers: foo",
  ],
  [
    "unknown-hooks-do-not-assert.js",
    "Analysis-only fixture references undefined helpers: useUnknownHook1, useUnknownHook2",
  ],
  [
    "unused-logical-assigned-to-variable.js",
    "Analysis-only fixture references undefined helpers: makeObject",
  ],
  [
    "unused-optional-method-assigned-to-variable.js",
    "Analysis-only fixture references undefined helpers: makeObject",
  ],
  [
    "unused-ternary-assigned-to-variable.js",
    "Analysis-only fixture references undefined helpers: makeObject",
  ],
  ["use-callback-simple.js", "Analysis-only fixture references undefined helpers: Foo"],
  [
    "use-no-memo-module-scope-usememo-function-scope.js",
    "Analysis-only fixture references undefined helpers: Button",
  ],
  [
    "useEffect-arg-memoized.js",
    "Analysis-only fixture references undefined helpers: useDispatch, useFreeze",
  ],
  [
    "useEffect-nested-lambdas.js",
    "Analysis-only fixture references undefined helpers: useMutable, useDispatch, useFreeze, createExitAction, GlobalEventEmitter, maybeMutate",
  ],
  [
    "useMemo-arrow-implicit-return.js",
    "Analysis-only fixture references undefined helpers: computeValue",
  ],
  [
    "useMemo-if-else-multiple-return.js",
    "Analysis-only fixture references undefined helpers: makeObject",
  ],
  [
    "useMemo-independently-memoizeable.js",
    "Analysis-only fixture references undefined helpers: makeObject",
  ],
  [
    "useMemo-multiple-returns.js",
    "Renders the matching item object as a React child; retained for useMemo control-flow snapshots",
  ],
  ["useMemo-simple.js", "Analysis-only fixture references undefined helpers: Foo"],
  [
    "using-declaration-skips-only-that-function.tsx",
    "Exercises explicit resource management syntax unsupported by the fixture Babel evaluator",
  ],
  ["valid-new-date-from-timestamp.js", "Analysis-only fixture references undefined helpers: Foo"],
  [
    "validate-no-set-state-in-render-uncalled-function-with-mutable-range-is-valid.js",
    "Analysis-only fixture references undefined helpers: useLogging, OtherComponent",
  ],
  [
    "invalid-useMemo-no-return-value.js",
    "Intentionally invalid useMemo callbacks produce VoidUseMemo diagnostics and render-phase logs",
  ],
  [
    "useMemo-void-babel-v1-default.tsx",
    "Intentionally invalid useMemo callback has no return value; panicThreshold none permits transformed logging",
  ],
  [
    "ref-type-convergence-recursive-loop.jsx",
    "Builds an object-valued onClick handler; retained for recursive type convergence snapshots",
  ],
  [
    "reactive-dependencies-non-optional-properties-inside-optional-chain.js",
    "References an undefined render callback for nonempty comment edges",
  ],
  [
    "propagate-scope-deps-hir-fork/reactive-dependencies-non-optional-properties-inside-optional-chain.js",
    "References an undefined render callback for nonempty comment edges",
  ],
]);
