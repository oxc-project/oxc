commit: 0124e7c7

Passed: 243/1165

# All Passed:
* babel-plugin-transform-export-namespace-from
* babel-plugin-transform-optional-catch-binding
* babel-plugin-transform-react-display-name


# babel-preset-env (32/85)
* dynamic-import/auto-esm-unsupported-import-unsupported/input.mjs
x Output mismatch

* dynamic-import/modules-amd/input.js
env: Amd module is not implemented.

* dynamic-import/modules-cjs/input.mjs
x Output mismatch

* dynamic-import/modules-systemjs/input.mjs
env: Systemjs module is not implemented.

* dynamic-import/modules-umd/input.mjs
env: Umd module is not implemented.

* export-namespace-from/auto-esm-not-supported/input.mjs
x Output mismatch

* export-namespace-from/auto-export-namespace-not-supported/input.mjs
x Output mismatch

* modules/auto-cjs/input.mjs
x Output mismatch

* modules/auto-unknown/input.mjs
x Output mismatch

* modules/modules-cjs/input.mjs
x Output mismatch

* modules/modules-commonjs/input.mjs
x Output mismatch

* modules/modules-systemjs/input.mjs
env: Systemjs module is not implemented.

* modules/modules-umd/input.mjs
env: Umd module is not implemented.

* plugins-integration/block-scoping-inside-generator/input.js
x Output mismatch

* plugins-integration/class-arrow-super-tagged-expr/input.js
x Output mismatch

* plugins-integration/class-features-node-12/input.js
x Output mismatch

* plugins-integration/for-of-array-block-scoping/input.js
x Output mismatch

* plugins-integration/for-of-array-block-scoping-2/input.js
x Output mismatch

* plugins-integration/issue-10662/input.mjs
env: Umd module is not implemented.

* plugins-integration/issue-11278/input.mjs
x Output mismatch

* plugins-integration/issue-15012/input.js
x Output mismatch

* plugins-integration/issue-15170/input.js
x Output mismatch

* plugins-integration/issue-16155/input.mjs
x Output mismatch

* plugins-integration/issue-7527/input.mjs
x Output mismatch

* plugins-integration/issue-9935/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_this":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_this":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_this":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_this":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_superprop_getLoadEntity":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)

* plugins-integration/regression-2892/input.mjs
x Output mismatch

* plugins-integration/regression-4855/input.js
x Output mismatch

* plugins-integration/spread-super-firefox-40/input.js
x Output mismatch

* preset-options/browserslist-config-ignore-config-with-false/input.mjs
x Output mismatch

* preset-options/browserslist-config-ignore-package-with-false/input.mjs
x Output mismatch

* preset-options/browserslist-defaults/input.mjs
x Output mismatch

* preset-options/browserslist-defaults-not-ie/input.mjs
x Output mismatch

* preset-options/deno-1_0/input.mjs
x Output mismatch

* preset-options/destructuring-edge/input.js
x Output mismatch

* preset-options/duplicate-named-capturing-groups-regex-chrome-123/input.js
x Output mismatch

* preset-options/empty-options/input.mjs
x Output mismatch

* preset-options/include/input.mjs
x Output mismatch

* preset-options/include-scoped/input.mjs
x Output mismatch

* preset-options/ios-6/input.mjs
x Output mismatch

* preset-options/no-options/input.mjs
x Output mismatch

* preset-options/regexp-modifiers-chrome-121/input.js
x Output mismatch

* preset-options/reserved-keys-ie8/input.mjs
x Output mismatch

* preset-options/reserved-names-ie8/input.mjs
x Output mismatch

* preset-options/rhino-1_7_13/input.mjs
x Output mismatch

* preset-options/safari-10_3-block-scoped/input.js
x Output mismatch

* preset-options/safari-tagged-template-literals/input.js
x Output mismatch

* preset-options/unicode-property-regex-chrome-49/input.js
x Output mismatch

* preset-options/unicode-sets-regex-chrome-111/input.js
x Output mismatch

* sanity/block-scoping-for-of/input.js
x Output mismatch

* sanity/regex-dot-all/input.js
x Output mismatch

* sanity/transform-duplicate-keys/input.js
x Output mismatch

* shipped-proposals/new-class-features-chrome-90/input.js
x Output mismatch

* shipped-proposals/new-class-features-firefox-70/input.js
x Output mismatch


# babel-plugin-transform-explicit-resource-management (0/28)
* integration/async-to-generator/input.js
Reference flags mismatch for "_fn":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_fn":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)

* integration/commonjs-transform/input.js
x Output mismatch

* source-maps/block/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)

* source-maps/for-of/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)

* source-maps/switch/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx2":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx2":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx2":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)

* source-maps/top-level/input.mjs
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)

* transform-await/mixed/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)

* transform-await/only-using-await/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(13): ReferenceFlags(Read | MemberWriteTarget)

* transform-await/switch/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx2":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx2":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx2":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)

* transform-sync/bare-block/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)

* transform-sync/for-await-head/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)

* transform-sync/for-head/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)

* transform-sync/for-head-shadow/input.js
x Output mismatch

* transform-sync/function-body/input.js
Bindings mismatch:
after transform: ScopeId(1): ["_usingCtx", "x"]
rebuilt        : ScopeId(1): ["_usingCtx"]
Bindings mismatch:
after transform: ScopeId(2): []
rebuilt        : ScopeId(2): ["x"]
Symbol scope ID mismatch for "x":
after transform: SymbolId(1): ScopeId(1)
rebuilt        : SymbolId(2): ScopeId(2)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)

* transform-sync/if-body/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)

* transform-sync/multiple-nested/input.js
Bindings mismatch:
after transform: ScopeId(3): ["_usingCtx3", "z"]
rebuilt        : ScopeId(3): ["_usingCtx3"]
Bindings mismatch:
after transform: ScopeId(10): []
rebuilt        : ScopeId(4): ["z"]
Symbol scope ID mismatch for "z":
after transform: SymbolId(2): ScopeId(3)
rebuilt        : SymbolId(5): ScopeId(4)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(12): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx2":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(17): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx3":
after transform: ReferenceId(16): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx3":
after transform: ReferenceId(18): ReferenceFlags(Read)
rebuilt        : ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx3":
after transform: ReferenceId(20): ReferenceFlags(Read)
rebuilt        : ReferenceId(12): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx2":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(14): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx2":
after transform: ReferenceId(15): ReferenceFlags(Read)
rebuilt        : ReferenceId(16): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(18): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(20): ReferenceFlags(Read | MemberWriteTarget)

* transform-sync/multiple-same-level/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(14): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(12): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(15): ReferenceFlags(Read)
rebuilt        : ReferenceId(15): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(17): ReferenceFlags(Read)
rebuilt        : ReferenceId(17): ReferenceFlags(Read | MemberWriteTarget)

* transform-sync/named-evaluation/input.js
x Output mismatch

* transform-sync/static-block/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)

* transform-sync/switch/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx2":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx2":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx2":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)

* transform-top-level/await-or-not-preserved/input.mjs
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)

* transform-top-level/hoisting/input.mjs
x Output mismatch

* transform-top-level/hoisting-default-clas-anon/input.mjs
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)

* transform-top-level/hoisting-default-class/input.mjs
x Output mismatch

* transform-top-level/hoisting-default-expr/input.mjs
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)

* transform-top-level/hoisting-default-fn/input.mjs
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)

* transform-top-level/hoisting-default-fn-anon/input.mjs
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)

* transform-top-level/hoisting-mutate-outer-class-binding/input.js
x Output mismatch


# babel-plugin-transform-unicode-sets-regex (0/4)
* basic/basic/input.js
x Output mismatch

* basic/string-properties/input.js
x Output mismatch

* transform-u/basic/input.js
x Output mismatch

* transform-u/string-properties/input.js
x Output mismatch


# babel-plugin-transform-class-properties (25/269)
* assumption-constantSuper/complex-super-class/input.js
x Output mismatch

* assumption-constantSuper/instance-field/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)

* assumption-constantSuper/static-field/input.js
x Output mismatch

* assumption-noDocumentAll/optional-chain-before-member-call/input.js
x Output mismatch

* assumption-noDocumentAll/optional-chain-cast-to-boolean/input.js
x Output mismatch

* assumption-noUninitializedPrivateFieldAccess/static-private/input.js
x Output mismatch

* assumption-setPublicClassFields/computed/input.js
x Output mismatch

* assumption-setPublicClassFields/constructor-collision/input.js
Reference flags mismatch for "foo":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Foo":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)

* assumption-setPublicClassFields/instance-computed/input.js
Reference flags mismatch for "_x":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)

* assumption-setPublicClassFields/length-name-use-define/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "A":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "A":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "A":
after transform: ReferenceId(12): ReferenceFlags(Read)
rebuilt        : ReferenceId(13): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(16): ReferenceFlags(Read)
rebuilt        : ReferenceId(15): ReferenceFlags(Read | MemberWriteTarget)

* assumption-setPublicClassFields/non-block-arrow-func/input.mjs
Reference flags mismatch for "_App":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)

* assumption-setPublicClassFields/regression-T2983/input.mjs
Reference flags mismatch for "_Class":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_Class2":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)

* assumption-setPublicClassFields/regression-T6719/input.js
Reference flags mismatch for "_WithContext":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)

* assumption-setPublicClassFields/regression-T7364/input.mjs
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)

* assumption-setPublicClassFields/static/input.js
Reference flags mismatch for "Foo":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)

* assumption-setPublicClassFields/static-class-binding/input.js
Reference flags mismatch for "A":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "A":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)

* assumption-setPublicClassFields/static-export/input.mjs
Reference flags mismatch for "MyClass":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "MyClass2":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)

* assumption-setPublicClassFields/static-infer-name/input.js
x Output mismatch

* assumption-setPublicClassFields/static-super/input.js
Reference flags mismatch for "A":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "B":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "B":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "B":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)

* assumption-setPublicClassFields/static-super-loose/input.js
x Output mismatch

* assumption-setPublicClassFields/static-this/input.js
Reference flags mismatch for "A":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "A":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)

* assumption-setPublicClassFields/static-undefined/input.js
Reference flags mismatch for "Foo":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)

* assumption-setPublicClassFields/super-expression/input.js
Reference flags mismatch for "foo":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)

* assumption-setPublicClassFields/super-with-collision/input.js
Reference flags mismatch for "force":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)

* class-name-tdz/general/input.js
x Output mismatch

* class-name-tdz/static-edgest-case/input.js
x Output mismatch

* class-name-tdz/static-general/input.js
x Output mismatch

* class-name-tdz/static-loose/input.js
x Output mismatch

* class-name-tdz/typescript-type/input.ts
x Output mismatch

* compile-to-class/constructor-collision-ignores-types/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Unresolved references mismatch:
after transform: ["T", "babelHelpers"]
rebuilt        : ["babelHelpers"]

* compile-to-class/constructor-collision-ignores-types-loose/input.js
Unresolved references mismatch:
after transform: ["T"]
rebuilt        : []

* compile-to-class/preserve-comments/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)

* nested-class/super-call-in-decorator/input.js
x Output mismatch

* nested-class/super-call-in-key/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)

* nested-class/super-property-in-accessor-key/input.js
x Output mismatch

* nested-class/super-property-in-decorator/input.js
x Output mismatch

* nested-class/super-property-in-key/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Inner":
after transform: ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(4): ReferenceFlags(Read)

* private/1-helpermemberexpressionfunction/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(12): ReferenceFlags(Read)
rebuilt        : ReferenceId(12): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(15): ReferenceFlags(Read)
rebuilt        : ReferenceId(14): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(14): ReferenceFlags(Read)
rebuilt        : ReferenceId(15): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(18): ReferenceFlags(Read)
rebuilt        : ReferenceId(18): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(21): ReferenceFlags(Read)
rebuilt        : ReferenceId(20): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(20): ReferenceFlags(Read)
rebuilt        : ReferenceId(21): ReferenceFlags(Read | MemberWriteTarget)

* private/assignment/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(15): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "other":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(12): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(13): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(17): ReferenceFlags(Read)
rebuilt        : ReferenceId(16): ReferenceFlags(Read | MemberWriteTarget)

* private/call/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)

* private/canonical/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "x":
after transform: ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(8): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "y":
after transform: ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(11): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(15): ReferenceFlags(Read)
rebuilt        : ReferenceId(12): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(17): ReferenceFlags(Read)
rebuilt        : ReferenceId(14): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "value":
after transform: ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(16): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(19): ReferenceFlags(Read)
rebuilt        : ReferenceId(17): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(21): ReferenceFlags(Read)
rebuilt        : ReferenceId(19): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "value":
after transform: ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(21): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(23): ReferenceFlags(Read)
rebuilt        : ReferenceId(22): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(25): ReferenceFlags(Read)
rebuilt        : ReferenceId(24): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "p":
after transform: ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(26): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(27): ReferenceFlags(Read)
rebuilt        : ReferenceId(27): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(29): ReferenceFlags(Read)
rebuilt        : ReferenceId(29): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "p":
after transform: ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(31): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(31): ReferenceFlags(Read)
rebuilt        : ReferenceId(32): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(33): ReferenceFlags(Read)
rebuilt        : ReferenceId(34): ReferenceFlags(Read | MemberWriteTarget)

* private/class-shadow-builtins/input.mjs
x Output mismatch

* private/constructor-collision/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)

* private/declaration-order/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)

* private/derived/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)

* private/derived-multiple-supers/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)

* private/destructuring-array-pattern/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "props":
after transform: ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(6): ReferenceFlags(Read)

* private/destructuring-array-pattern-1/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "props":
after transform: ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(12): ReferenceFlags(Read)

* private/destructuring-array-pattern-2/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "props":
after transform: ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(7): ReferenceFlags(Read)

* private/destructuring-array-pattern-3/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "props":
after transform: ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(6): ReferenceFlags(Read)

* private/destructuring-array-pattern-static/input.js
Reference flags mismatch for "_client":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)

* private/destructuring-object-pattern/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "props":
after transform: ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(6): ReferenceFlags(Read)

* private/destructuring-object-pattern-1/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "props":
after transform: ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(12): ReferenceFlags(Read)

* private/destructuring-object-pattern-2/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "props":
after transform: ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(7): ReferenceFlags(Read)

* private/destructuring-object-pattern-3/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "props":
after transform: ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(6): ReferenceFlags(Read)

* private/destructuring-object-pattern-static/input.js
Reference flags mismatch for "_client":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)

* private/extracted-this/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)

* private/foobar/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)

* private/instance/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)

* private/instance-undefined/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)

* private/logical-assignment/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(13): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(15): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(15): ReferenceFlags(Read)
rebuilt        : ReferenceId(17): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(17): ReferenceFlags(Read)
rebuilt        : ReferenceId(19): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(21): ReferenceFlags(Read)
rebuilt        : ReferenceId(21): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(23): ReferenceFlags(Read)
rebuilt        : ReferenceId(24): ReferenceFlags(Read | MemberWriteTarget)

* private/multiple/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)

* private/native-classes/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_foo":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)

* private/nested-class/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)

* private/nested-class-computed/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)

* private/nested-class-computed-redeclared/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)

* private/nested-class-extends-computed/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)

* private/nested-class-extends-computed-redeclared/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(12): ReferenceFlags(Read)
rebuilt        : ReferenceId(14): ReferenceFlags(Read | MemberWriteTarget)

* private/nested-class-other-redeclared/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(12): ReferenceFlags(Read)
rebuilt        : ReferenceId(13): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(14): ReferenceFlags(Read)
rebuilt        : ReferenceId(15): ReferenceFlags(Read | MemberWriteTarget)

* private/nested-class-redeclared/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)

* private/optional-chain-before-member-call/input.js
Reference flags mismatch for "o":
after transform: ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(4): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(54): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(48): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(12): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(61): ReferenceFlags(Read)
rebuilt        : ReferenceId(14): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(55): ReferenceFlags(Read)
rebuilt        : ReferenceId(17): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(20): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(68): ReferenceFlags(Read)
rebuilt        : ReferenceId(22): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(62): ReferenceFlags(Read)
rebuilt        : ReferenceId(25): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(76): ReferenceFlags(Read)
rebuilt        : ReferenceId(31): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_deep$very$o":
after transform: ReferenceId(70): ReferenceFlags(Read)
rebuilt        : ReferenceId(34): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(84): ReferenceFlags(Read)
rebuilt        : ReferenceId(40): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_deep$very$o2":
after transform: ReferenceId(78): ReferenceFlags(Read)
rebuilt        : ReferenceId(43): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(92): ReferenceFlags(Read)
rebuilt        : ReferenceId(49): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_deep$very$o3":
after transform: ReferenceId(86): ReferenceFlags(Read)
rebuilt        : ReferenceId(52): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(12): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(55): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(99): ReferenceFlags(Read)
rebuilt        : ReferenceId(57): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(102): ReferenceFlags(Read)
rebuilt        : ReferenceId(60): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(93): ReferenceFlags(Read)
rebuilt        : ReferenceId(62): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(13): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(66): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(109): ReferenceFlags(Read)
rebuilt        : ReferenceId(68): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(112): ReferenceFlags(Read)
rebuilt        : ReferenceId(71): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(103): ReferenceFlags(Read)
rebuilt        : ReferenceId(73): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(14): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(78): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(117): ReferenceFlags(Read)
rebuilt        : ReferenceId(80): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(113): ReferenceFlags(Read)
rebuilt        : ReferenceId(82): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(125): ReferenceFlags(Read)
rebuilt        : ReferenceId(85): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref":
after transform: ReferenceId(119): ReferenceFlags(Read)
rebuilt        : ReferenceId(88): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(15): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(92): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(130): ReferenceFlags(Read)
rebuilt        : ReferenceId(94): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(126): ReferenceFlags(Read)
rebuilt        : ReferenceId(96): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(138): ReferenceFlags(Read)
rebuilt        : ReferenceId(99): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref2":
after transform: ReferenceId(132): ReferenceFlags(Read)
rebuilt        : ReferenceId(102): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(143): ReferenceFlags(Read)
rebuilt        : ReferenceId(108): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(139): ReferenceFlags(Read)
rebuilt        : ReferenceId(110): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(151): ReferenceFlags(Read)
rebuilt        : ReferenceId(113): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_self2":
after transform: ReferenceId(145): ReferenceFlags(Read)
rebuilt        : ReferenceId(116): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(17): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(119): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(158): ReferenceFlags(Read)
rebuilt        : ReferenceId(121): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(161): ReferenceFlags(Read)
rebuilt        : ReferenceId(124): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(152): ReferenceFlags(Read)
rebuilt        : ReferenceId(126): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(18): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(131): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(166): ReferenceFlags(Read)
rebuilt        : ReferenceId(134): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(162): ReferenceFlags(Read)
rebuilt        : ReferenceId(136): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(176): ReferenceFlags(Read)
rebuilt        : ReferenceId(139): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref3":
after transform: ReferenceId(170): ReferenceFlags(Read)
rebuilt        : ReferenceId(142): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(19): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(147): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(181): ReferenceFlags(Read)
rebuilt        : ReferenceId(149): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(177): ReferenceFlags(Read)
rebuilt        : ReferenceId(151): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(189): ReferenceFlags(Read)
rebuilt        : ReferenceId(154): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref4":
after transform: ReferenceId(183): ReferenceFlags(Read)
rebuilt        : ReferenceId(157): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(20): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(162): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(194): ReferenceFlags(Read)
rebuilt        : ReferenceId(164): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(190): ReferenceFlags(Read)
rebuilt        : ReferenceId(166): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(204): ReferenceFlags(Read)
rebuilt        : ReferenceId(169): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref5$getSelf":
after transform: ReferenceId(198): ReferenceFlags(Read)
rebuilt        : ReferenceId(172): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(21): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(177): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(209): ReferenceFlags(Read)
rebuilt        : ReferenceId(179): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(205): ReferenceFlags(Read)
rebuilt        : ReferenceId(181): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(217): ReferenceFlags(Read)
rebuilt        : ReferenceId(184): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref6":
after transform: ReferenceId(211): ReferenceFlags(Read)
rebuilt        : ReferenceId(187): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(222): ReferenceFlags(Read)
rebuilt        : ReferenceId(194): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(218): ReferenceFlags(Read)
rebuilt        : ReferenceId(196): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(232): ReferenceFlags(Read)
rebuilt        : ReferenceId(200): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_call":
after transform: ReferenceId(226): ReferenceFlags(Read)
rebuilt        : ReferenceId(203): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(237): ReferenceFlags(Read)
rebuilt        : ReferenceId(209): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(233): ReferenceFlags(Read)
rebuilt        : ReferenceId(211): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(245): ReferenceFlags(Read)
rebuilt        : ReferenceId(214): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_getSelf":
after transform: ReferenceId(239): ReferenceFlags(Read)
rebuilt        : ReferenceId(217): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(250): ReferenceFlags(Read)
rebuilt        : ReferenceId(223): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(246): ReferenceFlags(Read)
rebuilt        : ReferenceId(225): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(258): ReferenceFlags(Read)
rebuilt        : ReferenceId(228): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_getSelf2":
after transform: ReferenceId(252): ReferenceFlags(Read)
rebuilt        : ReferenceId(231): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(25): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(234): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(265): ReferenceFlags(Read)
rebuilt        : ReferenceId(236): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(259): ReferenceFlags(Read)
rebuilt        : ReferenceId(239): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(26): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(242): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(272): ReferenceFlags(Read)
rebuilt        : ReferenceId(244): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(266): ReferenceFlags(Read)
rebuilt        : ReferenceId(247): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(27): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(250): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(279): ReferenceFlags(Read)
rebuilt        : ReferenceId(252): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(273): ReferenceFlags(Read)
rebuilt        : ReferenceId(255): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(287): ReferenceFlags(Read)
rebuilt        : ReferenceId(261): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_fnDeep$very$o":
after transform: ReferenceId(281): ReferenceFlags(Read)
rebuilt        : ReferenceId(264): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(295): ReferenceFlags(Read)
rebuilt        : ReferenceId(270): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_fnDeep$very$o2":
after transform: ReferenceId(289): ReferenceFlags(Read)
rebuilt        : ReferenceId(273): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(303): ReferenceFlags(Read)
rebuilt        : ReferenceId(279): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_fnDeep$very$o3":
after transform: ReferenceId(297): ReferenceFlags(Read)
rebuilt        : ReferenceId(282): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(31): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(285): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(310): ReferenceFlags(Read)
rebuilt        : ReferenceId(287): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(313): ReferenceFlags(Read)
rebuilt        : ReferenceId(290): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(304): ReferenceFlags(Read)
rebuilt        : ReferenceId(292): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(32): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(296): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(320): ReferenceFlags(Read)
rebuilt        : ReferenceId(298): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(323): ReferenceFlags(Read)
rebuilt        : ReferenceId(301): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(314): ReferenceFlags(Read)
rebuilt        : ReferenceId(303): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(33): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(308): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(328): ReferenceFlags(Read)
rebuilt        : ReferenceId(310): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(324): ReferenceFlags(Read)
rebuilt        : ReferenceId(312): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(336): ReferenceFlags(Read)
rebuilt        : ReferenceId(315): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref7":
after transform: ReferenceId(330): ReferenceFlags(Read)
rebuilt        : ReferenceId(318): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(34): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(322): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(341): ReferenceFlags(Read)
rebuilt        : ReferenceId(324): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(337): ReferenceFlags(Read)
rebuilt        : ReferenceId(326): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(349): ReferenceFlags(Read)
rebuilt        : ReferenceId(329): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref8":
after transform: ReferenceId(343): ReferenceFlags(Read)
rebuilt        : ReferenceId(332): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(354): ReferenceFlags(Read)
rebuilt        : ReferenceId(338): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(350): ReferenceFlags(Read)
rebuilt        : ReferenceId(340): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(362): ReferenceFlags(Read)
rebuilt        : ReferenceId(343): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_self3":
after transform: ReferenceId(356): ReferenceFlags(Read)
rebuilt        : ReferenceId(346): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(36): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(349): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(369): ReferenceFlags(Read)
rebuilt        : ReferenceId(351): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(372): ReferenceFlags(Read)
rebuilt        : ReferenceId(354): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(363): ReferenceFlags(Read)
rebuilt        : ReferenceId(356): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(37): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(361): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(377): ReferenceFlags(Read)
rebuilt        : ReferenceId(364): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(373): ReferenceFlags(Read)
rebuilt        : ReferenceId(366): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(387): ReferenceFlags(Read)
rebuilt        : ReferenceId(369): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref9":
after transform: ReferenceId(381): ReferenceFlags(Read)
rebuilt        : ReferenceId(372): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(38): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(377): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(392): ReferenceFlags(Read)
rebuilt        : ReferenceId(379): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(388): ReferenceFlags(Read)
rebuilt        : ReferenceId(381): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(400): ReferenceFlags(Read)
rebuilt        : ReferenceId(384): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref10":
after transform: ReferenceId(394): ReferenceFlags(Read)
rebuilt        : ReferenceId(387): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(39): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(392): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(405): ReferenceFlags(Read)
rebuilt        : ReferenceId(394): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(401): ReferenceFlags(Read)
rebuilt        : ReferenceId(396): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(415): ReferenceFlags(Read)
rebuilt        : ReferenceId(399): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref11$getSelf":
after transform: ReferenceId(409): ReferenceFlags(Read)
rebuilt        : ReferenceId(402): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(40): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(407): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(420): ReferenceFlags(Read)
rebuilt        : ReferenceId(409): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(416): ReferenceFlags(Read)
rebuilt        : ReferenceId(411): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(428): ReferenceFlags(Read)
rebuilt        : ReferenceId(414): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref12":
after transform: ReferenceId(422): ReferenceFlags(Read)
rebuilt        : ReferenceId(417): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(433): ReferenceFlags(Read)
rebuilt        : ReferenceId(424): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(429): ReferenceFlags(Read)
rebuilt        : ReferenceId(426): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(443): ReferenceFlags(Read)
rebuilt        : ReferenceId(430): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_call2":
after transform: ReferenceId(437): ReferenceFlags(Read)
rebuilt        : ReferenceId(433): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(448): ReferenceFlags(Read)
rebuilt        : ReferenceId(439): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(444): ReferenceFlags(Read)
rebuilt        : ReferenceId(441): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(456): ReferenceFlags(Read)
rebuilt        : ReferenceId(444): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_getSelf3":
after transform: ReferenceId(450): ReferenceFlags(Read)
rebuilt        : ReferenceId(447): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(461): ReferenceFlags(Read)
rebuilt        : ReferenceId(453): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(457): ReferenceFlags(Read)
rebuilt        : ReferenceId(455): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(469): ReferenceFlags(Read)
rebuilt        : ReferenceId(458): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_getSelf4":
after transform: ReferenceId(463): ReferenceFlags(Read)
rebuilt        : ReferenceId(461): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(47): ReferenceFlags(Read)
rebuilt        : ReferenceId(466): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_Foo":
after transform: ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(469): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(471): ReferenceFlags(Read)
rebuilt        : ReferenceId(470): ReferenceFlags(Read | MemberWriteTarget)

* private/optional-chain-before-member-call-with-transform/input.js
Reference flags mismatch for "o":
after transform: ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(4): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(54): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(48): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(12): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(61): ReferenceFlags(Read)
rebuilt        : ReferenceId(14): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(55): ReferenceFlags(Read)
rebuilt        : ReferenceId(17): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(20): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(68): ReferenceFlags(Read)
rebuilt        : ReferenceId(22): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(62): ReferenceFlags(Read)
rebuilt        : ReferenceId(25): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(76): ReferenceFlags(Read)
rebuilt        : ReferenceId(33): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_deep$very$o":
after transform: ReferenceId(70): ReferenceFlags(Read)
rebuilt        : ReferenceId(36): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(86): ReferenceFlags(Read)
rebuilt        : ReferenceId(44): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_deep$very$o2":
after transform: ReferenceId(80): ReferenceFlags(Read)
rebuilt        : ReferenceId(47): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(96): ReferenceFlags(Read)
rebuilt        : ReferenceId(55): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_deep$very$o3":
after transform: ReferenceId(90): ReferenceFlags(Read)
rebuilt        : ReferenceId(58): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(12): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(61): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(105): ReferenceFlags(Read)
rebuilt        : ReferenceId(63): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(108): ReferenceFlags(Read)
rebuilt        : ReferenceId(66): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(99): ReferenceFlags(Read)
rebuilt        : ReferenceId(68): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(13): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(72): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(115): ReferenceFlags(Read)
rebuilt        : ReferenceId(74): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(118): ReferenceFlags(Read)
rebuilt        : ReferenceId(77): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(109): ReferenceFlags(Read)
rebuilt        : ReferenceId(79): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(14): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(84): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(123): ReferenceFlags(Read)
rebuilt        : ReferenceId(86): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(119): ReferenceFlags(Read)
rebuilt        : ReferenceId(88): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(131): ReferenceFlags(Read)
rebuilt        : ReferenceId(91): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref":
after transform: ReferenceId(125): ReferenceFlags(Read)
rebuilt        : ReferenceId(94): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(15): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(98): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(136): ReferenceFlags(Read)
rebuilt        : ReferenceId(100): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(132): ReferenceFlags(Read)
rebuilt        : ReferenceId(102): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(144): ReferenceFlags(Read)
rebuilt        : ReferenceId(105): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref2":
after transform: ReferenceId(138): ReferenceFlags(Read)
rebuilt        : ReferenceId(108): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(16): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(113): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(149): ReferenceFlags(Read)
rebuilt        : ReferenceId(115): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(145): ReferenceFlags(Read)
rebuilt        : ReferenceId(117): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref3":
after transform: ReferenceId(158): ReferenceFlags(Read)
rebuilt        : ReferenceId(120): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(157): ReferenceFlags(Read)
rebuilt        : ReferenceId(122): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_self2":
after transform: ReferenceId(151): ReferenceFlags(Read)
rebuilt        : ReferenceId(125): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(17): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(128): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(167): ReferenceFlags(Read)
rebuilt        : ReferenceId(130): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(170): ReferenceFlags(Read)
rebuilt        : ReferenceId(133): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(161): ReferenceFlags(Read)
rebuilt        : ReferenceId(135): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(18): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(140): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(175): ReferenceFlags(Read)
rebuilt        : ReferenceId(143): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(171): ReferenceFlags(Read)
rebuilt        : ReferenceId(145): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(185): ReferenceFlags(Read)
rebuilt        : ReferenceId(148): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref4":
after transform: ReferenceId(179): ReferenceFlags(Read)
rebuilt        : ReferenceId(151): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(19): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(156): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(190): ReferenceFlags(Read)
rebuilt        : ReferenceId(158): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(186): ReferenceFlags(Read)
rebuilt        : ReferenceId(160): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(198): ReferenceFlags(Read)
rebuilt        : ReferenceId(163): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref5":
after transform: ReferenceId(192): ReferenceFlags(Read)
rebuilt        : ReferenceId(166): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(20): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(172): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(203): ReferenceFlags(Read)
rebuilt        : ReferenceId(174): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(199): ReferenceFlags(Read)
rebuilt        : ReferenceId(176): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref7":
after transform: ReferenceId(214): ReferenceFlags(Read)
rebuilt        : ReferenceId(179): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(213): ReferenceFlags(Read)
rebuilt        : ReferenceId(181): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref6$getSelf":
after transform: ReferenceId(207): ReferenceFlags(Read)
rebuilt        : ReferenceId(184): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(21): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(189): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(221): ReferenceFlags(Read)
rebuilt        : ReferenceId(191): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(217): ReferenceFlags(Read)
rebuilt        : ReferenceId(193): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(229): ReferenceFlags(Read)
rebuilt        : ReferenceId(196): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref8":
after transform: ReferenceId(223): ReferenceFlags(Read)
rebuilt        : ReferenceId(199): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(22): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(204): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(234): ReferenceFlags(Read)
rebuilt        : ReferenceId(207): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(230): ReferenceFlags(Read)
rebuilt        : ReferenceId(209): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref9":
after transform: ReferenceId(245): ReferenceFlags(Read)
rebuilt        : ReferenceId(212): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(244): ReferenceFlags(Read)
rebuilt        : ReferenceId(215): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_call":
after transform: ReferenceId(238): ReferenceFlags(Read)
rebuilt        : ReferenceId(218): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(23): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(223): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(252): ReferenceFlags(Read)
rebuilt        : ReferenceId(225): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(248): ReferenceFlags(Read)
rebuilt        : ReferenceId(227): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref10":
after transform: ReferenceId(261): ReferenceFlags(Read)
rebuilt        : ReferenceId(230): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(260): ReferenceFlags(Read)
rebuilt        : ReferenceId(232): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_getSelf":
after transform: ReferenceId(254): ReferenceFlags(Read)
rebuilt        : ReferenceId(235): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(24): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(240): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(268): ReferenceFlags(Read)
rebuilt        : ReferenceId(242): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(264): ReferenceFlags(Read)
rebuilt        : ReferenceId(244): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref11":
after transform: ReferenceId(277): ReferenceFlags(Read)
rebuilt        : ReferenceId(248): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref11$getSelf":
after transform: ReferenceId(280): ReferenceFlags(Read)
rebuilt        : ReferenceId(250): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(276): ReferenceFlags(Read)
rebuilt        : ReferenceId(253): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_getSelf2":
after transform: ReferenceId(270): ReferenceFlags(Read)
rebuilt        : ReferenceId(256): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(25): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(259): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(290): ReferenceFlags(Read)
rebuilt        : ReferenceId(261): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(284): ReferenceFlags(Read)
rebuilt        : ReferenceId(264): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(26): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(267): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(297): ReferenceFlags(Read)
rebuilt        : ReferenceId(269): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(291): ReferenceFlags(Read)
rebuilt        : ReferenceId(272): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(27): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(275): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(304): ReferenceFlags(Read)
rebuilt        : ReferenceId(277): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(298): ReferenceFlags(Read)
rebuilt        : ReferenceId(280): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(312): ReferenceFlags(Read)
rebuilt        : ReferenceId(288): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_fnDeep$very$o":
after transform: ReferenceId(306): ReferenceFlags(Read)
rebuilt        : ReferenceId(291): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(322): ReferenceFlags(Read)
rebuilt        : ReferenceId(299): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_fnDeep$very$o2":
after transform: ReferenceId(316): ReferenceFlags(Read)
rebuilt        : ReferenceId(302): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(332): ReferenceFlags(Read)
rebuilt        : ReferenceId(310): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_fnDeep$very$o3":
after transform: ReferenceId(326): ReferenceFlags(Read)
rebuilt        : ReferenceId(313): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(31): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(316): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(341): ReferenceFlags(Read)
rebuilt        : ReferenceId(318): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(344): ReferenceFlags(Read)
rebuilt        : ReferenceId(321): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(335): ReferenceFlags(Read)
rebuilt        : ReferenceId(323): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(32): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(327): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(351): ReferenceFlags(Read)
rebuilt        : ReferenceId(329): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(354): ReferenceFlags(Read)
rebuilt        : ReferenceId(332): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(345): ReferenceFlags(Read)
rebuilt        : ReferenceId(334): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(33): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(339): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(359): ReferenceFlags(Read)
rebuilt        : ReferenceId(341): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(355): ReferenceFlags(Read)
rebuilt        : ReferenceId(343): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(367): ReferenceFlags(Read)
rebuilt        : ReferenceId(346): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref12":
after transform: ReferenceId(361): ReferenceFlags(Read)
rebuilt        : ReferenceId(349): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(34): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(353): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(372): ReferenceFlags(Read)
rebuilt        : ReferenceId(355): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(368): ReferenceFlags(Read)
rebuilt        : ReferenceId(357): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(380): ReferenceFlags(Read)
rebuilt        : ReferenceId(360): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref13":
after transform: ReferenceId(374): ReferenceFlags(Read)
rebuilt        : ReferenceId(363): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(35): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(368): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(385): ReferenceFlags(Read)
rebuilt        : ReferenceId(370): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(381): ReferenceFlags(Read)
rebuilt        : ReferenceId(372): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref14":
after transform: ReferenceId(394): ReferenceFlags(Read)
rebuilt        : ReferenceId(375): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(393): ReferenceFlags(Read)
rebuilt        : ReferenceId(377): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_self3":
after transform: ReferenceId(387): ReferenceFlags(Read)
rebuilt        : ReferenceId(380): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(36): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(383): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(403): ReferenceFlags(Read)
rebuilt        : ReferenceId(385): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(406): ReferenceFlags(Read)
rebuilt        : ReferenceId(388): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(397): ReferenceFlags(Read)
rebuilt        : ReferenceId(390): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(37): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(395): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(411): ReferenceFlags(Read)
rebuilt        : ReferenceId(398): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(407): ReferenceFlags(Read)
rebuilt        : ReferenceId(400): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(421): ReferenceFlags(Read)
rebuilt        : ReferenceId(403): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref15":
after transform: ReferenceId(415): ReferenceFlags(Read)
rebuilt        : ReferenceId(406): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(38): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(411): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(426): ReferenceFlags(Read)
rebuilt        : ReferenceId(413): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(422): ReferenceFlags(Read)
rebuilt        : ReferenceId(415): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(434): ReferenceFlags(Read)
rebuilt        : ReferenceId(418): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref16":
after transform: ReferenceId(428): ReferenceFlags(Read)
rebuilt        : ReferenceId(421): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(39): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(427): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(439): ReferenceFlags(Read)
rebuilt        : ReferenceId(429): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(435): ReferenceFlags(Read)
rebuilt        : ReferenceId(431): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref18":
after transform: ReferenceId(450): ReferenceFlags(Read)
rebuilt        : ReferenceId(434): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(449): ReferenceFlags(Read)
rebuilt        : ReferenceId(436): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref17$getSelf":
after transform: ReferenceId(443): ReferenceFlags(Read)
rebuilt        : ReferenceId(439): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(40): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(444): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(457): ReferenceFlags(Read)
rebuilt        : ReferenceId(446): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(453): ReferenceFlags(Read)
rebuilt        : ReferenceId(448): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(465): ReferenceFlags(Read)
rebuilt        : ReferenceId(451): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref19":
after transform: ReferenceId(459): ReferenceFlags(Read)
rebuilt        : ReferenceId(454): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(41): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(459): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(470): ReferenceFlags(Read)
rebuilt        : ReferenceId(462): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(466): ReferenceFlags(Read)
rebuilt        : ReferenceId(464): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref20":
after transform: ReferenceId(481): ReferenceFlags(Read)
rebuilt        : ReferenceId(467): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(480): ReferenceFlags(Read)
rebuilt        : ReferenceId(470): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_call2":
after transform: ReferenceId(474): ReferenceFlags(Read)
rebuilt        : ReferenceId(473): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(42): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(478): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(488): ReferenceFlags(Read)
rebuilt        : ReferenceId(480): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(484): ReferenceFlags(Read)
rebuilt        : ReferenceId(482): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref21":
after transform: ReferenceId(497): ReferenceFlags(Read)
rebuilt        : ReferenceId(485): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(496): ReferenceFlags(Read)
rebuilt        : ReferenceId(487): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_getSelf3":
after transform: ReferenceId(490): ReferenceFlags(Read)
rebuilt        : ReferenceId(490): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(43): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(495): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(504): ReferenceFlags(Read)
rebuilt        : ReferenceId(497): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(500): ReferenceFlags(Read)
rebuilt        : ReferenceId(499): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref22":
after transform: ReferenceId(513): ReferenceFlags(Read)
rebuilt        : ReferenceId(503): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref22$getSelf":
after transform: ReferenceId(516): ReferenceFlags(Read)
rebuilt        : ReferenceId(505): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(512): ReferenceFlags(Read)
rebuilt        : ReferenceId(508): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_getSelf4":
after transform: ReferenceId(506): ReferenceFlags(Read)
rebuilt        : ReferenceId(511): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(47): ReferenceFlags(Read)
rebuilt        : ReferenceId(516): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_Foo":
after transform: ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(519): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(521): ReferenceFlags(Read)
rebuilt        : ReferenceId(520): ReferenceFlags(Read | MemberWriteTarget)

* private/optional-chain-before-property/input.js
Reference flags mismatch for "o":
after transform: ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(4): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(49): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(45): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(10): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(54): ReferenceFlags(Read)
rebuilt        : ReferenceId(12): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(50): ReferenceFlags(Read)
rebuilt        : ReferenceId(14): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(16): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(59): ReferenceFlags(Read)
rebuilt        : ReferenceId(18): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(55): ReferenceFlags(Read)
rebuilt        : ReferenceId(20): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(65): ReferenceFlags(Read)
rebuilt        : ReferenceId(25): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_deep$very$o":
after transform: ReferenceId(61): ReferenceFlags(Read)
rebuilt        : ReferenceId(27): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(71): ReferenceFlags(Read)
rebuilt        : ReferenceId(32): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_deep$very$o2":
after transform: ReferenceId(67): ReferenceFlags(Read)
rebuilt        : ReferenceId(34): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(77): ReferenceFlags(Read)
rebuilt        : ReferenceId(39): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_deep$very$o3":
after transform: ReferenceId(73): ReferenceFlags(Read)
rebuilt        : ReferenceId(41): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(12): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(43): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(82): ReferenceFlags(Read)
rebuilt        : ReferenceId(45): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(85): ReferenceFlags(Read)
rebuilt        : ReferenceId(47): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(78): ReferenceFlags(Read)
rebuilt        : ReferenceId(49): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(13): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(52): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(90): ReferenceFlags(Read)
rebuilt        : ReferenceId(54): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(93): ReferenceFlags(Read)
rebuilt        : ReferenceId(56): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(86): ReferenceFlags(Read)
rebuilt        : ReferenceId(58): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(14): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(62): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(98): ReferenceFlags(Read)
rebuilt        : ReferenceId(64): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(94): ReferenceFlags(Read)
rebuilt        : ReferenceId(66): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(104): ReferenceFlags(Read)
rebuilt        : ReferenceId(69): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref":
after transform: ReferenceId(100): ReferenceFlags(Read)
rebuilt        : ReferenceId(71): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(15): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(74): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(109): ReferenceFlags(Read)
rebuilt        : ReferenceId(76): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(105): ReferenceFlags(Read)
rebuilt        : ReferenceId(78): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(115): ReferenceFlags(Read)
rebuilt        : ReferenceId(81): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref2":
after transform: ReferenceId(111): ReferenceFlags(Read)
rebuilt        : ReferenceId(83): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(120): ReferenceFlags(Read)
rebuilt        : ReferenceId(88): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(116): ReferenceFlags(Read)
rebuilt        : ReferenceId(90): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(126): ReferenceFlags(Read)
rebuilt        : ReferenceId(93): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_self2":
after transform: ReferenceId(122): ReferenceFlags(Read)
rebuilt        : ReferenceId(95): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(17): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(97): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(131): ReferenceFlags(Read)
rebuilt        : ReferenceId(99): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(134): ReferenceFlags(Read)
rebuilt        : ReferenceId(101): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(127): ReferenceFlags(Read)
rebuilt        : ReferenceId(103): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(18): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(107): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(139): ReferenceFlags(Read)
rebuilt        : ReferenceId(110): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(135): ReferenceFlags(Read)
rebuilt        : ReferenceId(112): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(147): ReferenceFlags(Read)
rebuilt        : ReferenceId(115): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref3":
after transform: ReferenceId(143): ReferenceFlags(Read)
rebuilt        : ReferenceId(117): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(19): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(121): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(152): ReferenceFlags(Read)
rebuilt        : ReferenceId(123): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(148): ReferenceFlags(Read)
rebuilt        : ReferenceId(125): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(158): ReferenceFlags(Read)
rebuilt        : ReferenceId(128): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref4":
after transform: ReferenceId(154): ReferenceFlags(Read)
rebuilt        : ReferenceId(130): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(20): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(134): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(163): ReferenceFlags(Read)
rebuilt        : ReferenceId(136): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(159): ReferenceFlags(Read)
rebuilt        : ReferenceId(138): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(171): ReferenceFlags(Read)
rebuilt        : ReferenceId(141): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref5$getSelf":
after transform: ReferenceId(167): ReferenceFlags(Read)
rebuilt        : ReferenceId(143): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(21): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(147): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(176): ReferenceFlags(Read)
rebuilt        : ReferenceId(149): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(172): ReferenceFlags(Read)
rebuilt        : ReferenceId(151): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(182): ReferenceFlags(Read)
rebuilt        : ReferenceId(154): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref6":
after transform: ReferenceId(178): ReferenceFlags(Read)
rebuilt        : ReferenceId(156): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(187): ReferenceFlags(Read)
rebuilt        : ReferenceId(162): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(183): ReferenceFlags(Read)
rebuilt        : ReferenceId(164): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(195): ReferenceFlags(Read)
rebuilt        : ReferenceId(168): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_call":
after transform: ReferenceId(191): ReferenceFlags(Read)
rebuilt        : ReferenceId(170): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(200): ReferenceFlags(Read)
rebuilt        : ReferenceId(175): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(196): ReferenceFlags(Read)
rebuilt        : ReferenceId(177): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(206): ReferenceFlags(Read)
rebuilt        : ReferenceId(180): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_getSelf":
after transform: ReferenceId(202): ReferenceFlags(Read)
rebuilt        : ReferenceId(182): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(211): ReferenceFlags(Read)
rebuilt        : ReferenceId(187): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(207): ReferenceFlags(Read)
rebuilt        : ReferenceId(189): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(217): ReferenceFlags(Read)
rebuilt        : ReferenceId(192): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_getSelf2":
after transform: ReferenceId(213): ReferenceFlags(Read)
rebuilt        : ReferenceId(194): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(25): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(196): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(222): ReferenceFlags(Read)
rebuilt        : ReferenceId(198): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(218): ReferenceFlags(Read)
rebuilt        : ReferenceId(200): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(26): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(202): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(227): ReferenceFlags(Read)
rebuilt        : ReferenceId(204): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(223): ReferenceFlags(Read)
rebuilt        : ReferenceId(206): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(27): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(208): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(232): ReferenceFlags(Read)
rebuilt        : ReferenceId(210): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(228): ReferenceFlags(Read)
rebuilt        : ReferenceId(212): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(238): ReferenceFlags(Read)
rebuilt        : ReferenceId(217): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_fnDeep$very$o":
after transform: ReferenceId(234): ReferenceFlags(Read)
rebuilt        : ReferenceId(219): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(244): ReferenceFlags(Read)
rebuilt        : ReferenceId(224): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_fnDeep$very$o2":
after transform: ReferenceId(240): ReferenceFlags(Read)
rebuilt        : ReferenceId(226): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(250): ReferenceFlags(Read)
rebuilt        : ReferenceId(231): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_fnDeep$very$o3":
after transform: ReferenceId(246): ReferenceFlags(Read)
rebuilt        : ReferenceId(233): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(31): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(235): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(255): ReferenceFlags(Read)
rebuilt        : ReferenceId(237): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(258): ReferenceFlags(Read)
rebuilt        : ReferenceId(239): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(251): ReferenceFlags(Read)
rebuilt        : ReferenceId(241): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(32): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(244): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(263): ReferenceFlags(Read)
rebuilt        : ReferenceId(246): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(266): ReferenceFlags(Read)
rebuilt        : ReferenceId(248): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(259): ReferenceFlags(Read)
rebuilt        : ReferenceId(250): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(33): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(254): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(271): ReferenceFlags(Read)
rebuilt        : ReferenceId(256): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(267): ReferenceFlags(Read)
rebuilt        : ReferenceId(258): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(277): ReferenceFlags(Read)
rebuilt        : ReferenceId(261): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref7":
after transform: ReferenceId(273): ReferenceFlags(Read)
rebuilt        : ReferenceId(263): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(34): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(266): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(282): ReferenceFlags(Read)
rebuilt        : ReferenceId(268): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(278): ReferenceFlags(Read)
rebuilt        : ReferenceId(270): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(288): ReferenceFlags(Read)
rebuilt        : ReferenceId(273): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref8":
after transform: ReferenceId(284): ReferenceFlags(Read)
rebuilt        : ReferenceId(275): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(293): ReferenceFlags(Read)
rebuilt        : ReferenceId(280): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(289): ReferenceFlags(Read)
rebuilt        : ReferenceId(282): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(299): ReferenceFlags(Read)
rebuilt        : ReferenceId(285): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_self3":
after transform: ReferenceId(295): ReferenceFlags(Read)
rebuilt        : ReferenceId(287): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(36): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(289): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(304): ReferenceFlags(Read)
rebuilt        : ReferenceId(291): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(307): ReferenceFlags(Read)
rebuilt        : ReferenceId(293): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(300): ReferenceFlags(Read)
rebuilt        : ReferenceId(295): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(37): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(299): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(312): ReferenceFlags(Read)
rebuilt        : ReferenceId(302): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(308): ReferenceFlags(Read)
rebuilt        : ReferenceId(304): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(320): ReferenceFlags(Read)
rebuilt        : ReferenceId(307): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref9":
after transform: ReferenceId(316): ReferenceFlags(Read)
rebuilt        : ReferenceId(309): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(38): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(313): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(325): ReferenceFlags(Read)
rebuilt        : ReferenceId(315): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(321): ReferenceFlags(Read)
rebuilt        : ReferenceId(317): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(331): ReferenceFlags(Read)
rebuilt        : ReferenceId(320): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref10":
after transform: ReferenceId(327): ReferenceFlags(Read)
rebuilt        : ReferenceId(322): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(39): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(326): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(336): ReferenceFlags(Read)
rebuilt        : ReferenceId(328): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(332): ReferenceFlags(Read)
rebuilt        : ReferenceId(330): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(344): ReferenceFlags(Read)
rebuilt        : ReferenceId(333): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref11$getSelf":
after transform: ReferenceId(340): ReferenceFlags(Read)
rebuilt        : ReferenceId(335): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(40): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(339): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(349): ReferenceFlags(Read)
rebuilt        : ReferenceId(341): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(345): ReferenceFlags(Read)
rebuilt        : ReferenceId(343): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(355): ReferenceFlags(Read)
rebuilt        : ReferenceId(346): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref12":
after transform: ReferenceId(351): ReferenceFlags(Read)
rebuilt        : ReferenceId(348): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(360): ReferenceFlags(Read)
rebuilt        : ReferenceId(354): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(356): ReferenceFlags(Read)
rebuilt        : ReferenceId(356): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(368): ReferenceFlags(Read)
rebuilt        : ReferenceId(360): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_call2":
after transform: ReferenceId(364): ReferenceFlags(Read)
rebuilt        : ReferenceId(362): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(373): ReferenceFlags(Read)
rebuilt        : ReferenceId(367): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(369): ReferenceFlags(Read)
rebuilt        : ReferenceId(369): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(379): ReferenceFlags(Read)
rebuilt        : ReferenceId(372): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_getSelf3":
after transform: ReferenceId(375): ReferenceFlags(Read)
rebuilt        : ReferenceId(374): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(384): ReferenceFlags(Read)
rebuilt        : ReferenceId(379): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(380): ReferenceFlags(Read)
rebuilt        : ReferenceId(381): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(390): ReferenceFlags(Read)
rebuilt        : ReferenceId(384): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_getSelf4":
after transform: ReferenceId(386): ReferenceFlags(Read)
rebuilt        : ReferenceId(386): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(392): ReferenceFlags(Read)
rebuilt        : ReferenceId(391): ReferenceFlags(Read | MemberWriteTarget)

* private/optional-chain-before-property-with-transform/input.js
Reference flags mismatch for "o":
after transform: ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(4): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(49): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(45): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(10): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(54): ReferenceFlags(Read)
rebuilt        : ReferenceId(12): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(50): ReferenceFlags(Read)
rebuilt        : ReferenceId(14): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(16): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(59): ReferenceFlags(Read)
rebuilt        : ReferenceId(18): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(55): ReferenceFlags(Read)
rebuilt        : ReferenceId(20): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(65): ReferenceFlags(Read)
rebuilt        : ReferenceId(27): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_deep$very$o":
after transform: ReferenceId(61): ReferenceFlags(Read)
rebuilt        : ReferenceId(29): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(73): ReferenceFlags(Read)
rebuilt        : ReferenceId(36): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_deep$very$o2":
after transform: ReferenceId(69): ReferenceFlags(Read)
rebuilt        : ReferenceId(38): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(81): ReferenceFlags(Read)
rebuilt        : ReferenceId(45): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_deep$very$o3":
after transform: ReferenceId(77): ReferenceFlags(Read)
rebuilt        : ReferenceId(47): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(12): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(49): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(88): ReferenceFlags(Read)
rebuilt        : ReferenceId(51): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(91): ReferenceFlags(Read)
rebuilt        : ReferenceId(53): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(84): ReferenceFlags(Read)
rebuilt        : ReferenceId(55): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(13): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(58): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(96): ReferenceFlags(Read)
rebuilt        : ReferenceId(60): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(99): ReferenceFlags(Read)
rebuilt        : ReferenceId(62): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(92): ReferenceFlags(Read)
rebuilt        : ReferenceId(64): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(14): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(68): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(104): ReferenceFlags(Read)
rebuilt        : ReferenceId(70): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(100): ReferenceFlags(Read)
rebuilt        : ReferenceId(72): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(110): ReferenceFlags(Read)
rebuilt        : ReferenceId(75): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref":
after transform: ReferenceId(106): ReferenceFlags(Read)
rebuilt        : ReferenceId(77): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(15): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(80): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(115): ReferenceFlags(Read)
rebuilt        : ReferenceId(82): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(111): ReferenceFlags(Read)
rebuilt        : ReferenceId(84): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(121): ReferenceFlags(Read)
rebuilt        : ReferenceId(87): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref2":
after transform: ReferenceId(117): ReferenceFlags(Read)
rebuilt        : ReferenceId(89): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(16): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(93): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(126): ReferenceFlags(Read)
rebuilt        : ReferenceId(95): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(122): ReferenceFlags(Read)
rebuilt        : ReferenceId(97): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref3":
after transform: ReferenceId(133): ReferenceFlags(Read)
rebuilt        : ReferenceId(100): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(132): ReferenceFlags(Read)
rebuilt        : ReferenceId(102): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_self2":
after transform: ReferenceId(128): ReferenceFlags(Read)
rebuilt        : ReferenceId(104): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(17): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(106): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(140): ReferenceFlags(Read)
rebuilt        : ReferenceId(108): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(143): ReferenceFlags(Read)
rebuilt        : ReferenceId(110): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(136): ReferenceFlags(Read)
rebuilt        : ReferenceId(112): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(18): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(116): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(148): ReferenceFlags(Read)
rebuilt        : ReferenceId(119): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(144): ReferenceFlags(Read)
rebuilt        : ReferenceId(121): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(156): ReferenceFlags(Read)
rebuilt        : ReferenceId(124): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref4":
after transform: ReferenceId(152): ReferenceFlags(Read)
rebuilt        : ReferenceId(126): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(19): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(130): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(161): ReferenceFlags(Read)
rebuilt        : ReferenceId(132): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(157): ReferenceFlags(Read)
rebuilt        : ReferenceId(134): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(167): ReferenceFlags(Read)
rebuilt        : ReferenceId(137): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref5":
after transform: ReferenceId(163): ReferenceFlags(Read)
rebuilt        : ReferenceId(139): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(20): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(144): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(172): ReferenceFlags(Read)
rebuilt        : ReferenceId(146): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(168): ReferenceFlags(Read)
rebuilt        : ReferenceId(148): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref7":
after transform: ReferenceId(181): ReferenceFlags(Read)
rebuilt        : ReferenceId(151): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(180): ReferenceFlags(Read)
rebuilt        : ReferenceId(153): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref6$getSelf":
after transform: ReferenceId(176): ReferenceFlags(Read)
rebuilt        : ReferenceId(155): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(21): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(159): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(188): ReferenceFlags(Read)
rebuilt        : ReferenceId(161): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(184): ReferenceFlags(Read)
rebuilt        : ReferenceId(163): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(194): ReferenceFlags(Read)
rebuilt        : ReferenceId(166): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref8":
after transform: ReferenceId(190): ReferenceFlags(Read)
rebuilt        : ReferenceId(168): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(22): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(172): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(199): ReferenceFlags(Read)
rebuilt        : ReferenceId(175): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(195): ReferenceFlags(Read)
rebuilt        : ReferenceId(177): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref9":
after transform: ReferenceId(208): ReferenceFlags(Read)
rebuilt        : ReferenceId(180): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(207): ReferenceFlags(Read)
rebuilt        : ReferenceId(183): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_call":
after transform: ReferenceId(203): ReferenceFlags(Read)
rebuilt        : ReferenceId(185): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(23): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(189): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(215): ReferenceFlags(Read)
rebuilt        : ReferenceId(191): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(211): ReferenceFlags(Read)
rebuilt        : ReferenceId(193): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref10":
after transform: ReferenceId(222): ReferenceFlags(Read)
rebuilt        : ReferenceId(196): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(221): ReferenceFlags(Read)
rebuilt        : ReferenceId(198): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_getSelf":
after transform: ReferenceId(217): ReferenceFlags(Read)
rebuilt        : ReferenceId(200): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(24): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(204): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(229): ReferenceFlags(Read)
rebuilt        : ReferenceId(206): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(225): ReferenceFlags(Read)
rebuilt        : ReferenceId(208): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref11":
after transform: ReferenceId(236): ReferenceFlags(Read)
rebuilt        : ReferenceId(212): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref11$getSelf":
after transform: ReferenceId(239): ReferenceFlags(Read)
rebuilt        : ReferenceId(214): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(235): ReferenceFlags(Read)
rebuilt        : ReferenceId(217): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_getSelf2":
after transform: ReferenceId(231): ReferenceFlags(Read)
rebuilt        : ReferenceId(219): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(25): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(221): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(247): ReferenceFlags(Read)
rebuilt        : ReferenceId(223): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(243): ReferenceFlags(Read)
rebuilt        : ReferenceId(225): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(26): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(227): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(252): ReferenceFlags(Read)
rebuilt        : ReferenceId(229): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(248): ReferenceFlags(Read)
rebuilt        : ReferenceId(231): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(27): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(233): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(257): ReferenceFlags(Read)
rebuilt        : ReferenceId(235): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(253): ReferenceFlags(Read)
rebuilt        : ReferenceId(237): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(263): ReferenceFlags(Read)
rebuilt        : ReferenceId(244): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_fnDeep$very$o":
after transform: ReferenceId(259): ReferenceFlags(Read)
rebuilt        : ReferenceId(246): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(271): ReferenceFlags(Read)
rebuilt        : ReferenceId(253): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_fnDeep$very$o2":
after transform: ReferenceId(267): ReferenceFlags(Read)
rebuilt        : ReferenceId(255): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(279): ReferenceFlags(Read)
rebuilt        : ReferenceId(262): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_fnDeep$very$o3":
after transform: ReferenceId(275): ReferenceFlags(Read)
rebuilt        : ReferenceId(264): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(31): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(266): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(286): ReferenceFlags(Read)
rebuilt        : ReferenceId(268): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(289): ReferenceFlags(Read)
rebuilt        : ReferenceId(270): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(282): ReferenceFlags(Read)
rebuilt        : ReferenceId(272): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(32): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(275): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(294): ReferenceFlags(Read)
rebuilt        : ReferenceId(277): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(297): ReferenceFlags(Read)
rebuilt        : ReferenceId(279): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(290): ReferenceFlags(Read)
rebuilt        : ReferenceId(281): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(33): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(285): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(302): ReferenceFlags(Read)
rebuilt        : ReferenceId(287): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(298): ReferenceFlags(Read)
rebuilt        : ReferenceId(289): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(308): ReferenceFlags(Read)
rebuilt        : ReferenceId(292): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref12":
after transform: ReferenceId(304): ReferenceFlags(Read)
rebuilt        : ReferenceId(294): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(34): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(297): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(313): ReferenceFlags(Read)
rebuilt        : ReferenceId(299): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(309): ReferenceFlags(Read)
rebuilt        : ReferenceId(301): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(319): ReferenceFlags(Read)
rebuilt        : ReferenceId(304): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref13":
after transform: ReferenceId(315): ReferenceFlags(Read)
rebuilt        : ReferenceId(306): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(35): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(310): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(324): ReferenceFlags(Read)
rebuilt        : ReferenceId(312): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(320): ReferenceFlags(Read)
rebuilt        : ReferenceId(314): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref14":
after transform: ReferenceId(331): ReferenceFlags(Read)
rebuilt        : ReferenceId(317): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(330): ReferenceFlags(Read)
rebuilt        : ReferenceId(319): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_self3":
after transform: ReferenceId(326): ReferenceFlags(Read)
rebuilt        : ReferenceId(321): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(36): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(323): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(338): ReferenceFlags(Read)
rebuilt        : ReferenceId(325): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(341): ReferenceFlags(Read)
rebuilt        : ReferenceId(327): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(334): ReferenceFlags(Read)
rebuilt        : ReferenceId(329): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(37): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(333): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(346): ReferenceFlags(Read)
rebuilt        : ReferenceId(336): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(342): ReferenceFlags(Read)
rebuilt        : ReferenceId(338): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(354): ReferenceFlags(Read)
rebuilt        : ReferenceId(341): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref15":
after transform: ReferenceId(350): ReferenceFlags(Read)
rebuilt        : ReferenceId(343): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(38): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(347): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(359): ReferenceFlags(Read)
rebuilt        : ReferenceId(349): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(355): ReferenceFlags(Read)
rebuilt        : ReferenceId(351): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(365): ReferenceFlags(Read)
rebuilt        : ReferenceId(354): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref16":
after transform: ReferenceId(361): ReferenceFlags(Read)
rebuilt        : ReferenceId(356): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(39): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(361): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(370): ReferenceFlags(Read)
rebuilt        : ReferenceId(363): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(366): ReferenceFlags(Read)
rebuilt        : ReferenceId(365): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref18":
after transform: ReferenceId(379): ReferenceFlags(Read)
rebuilt        : ReferenceId(368): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(378): ReferenceFlags(Read)
rebuilt        : ReferenceId(370): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref17$getSelf":
after transform: ReferenceId(374): ReferenceFlags(Read)
rebuilt        : ReferenceId(372): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(40): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(376): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(386): ReferenceFlags(Read)
rebuilt        : ReferenceId(378): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(382): ReferenceFlags(Read)
rebuilt        : ReferenceId(380): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(392): ReferenceFlags(Read)
rebuilt        : ReferenceId(383): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref19":
after transform: ReferenceId(388): ReferenceFlags(Read)
rebuilt        : ReferenceId(385): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(41): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(389): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(397): ReferenceFlags(Read)
rebuilt        : ReferenceId(392): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(393): ReferenceFlags(Read)
rebuilt        : ReferenceId(394): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref20":
after transform: ReferenceId(406): ReferenceFlags(Read)
rebuilt        : ReferenceId(397): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(405): ReferenceFlags(Read)
rebuilt        : ReferenceId(400): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_call2":
after transform: ReferenceId(401): ReferenceFlags(Read)
rebuilt        : ReferenceId(402): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(42): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(406): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(413): ReferenceFlags(Read)
rebuilt        : ReferenceId(408): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(409): ReferenceFlags(Read)
rebuilt        : ReferenceId(410): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref21":
after transform: ReferenceId(420): ReferenceFlags(Read)
rebuilt        : ReferenceId(413): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(419): ReferenceFlags(Read)
rebuilt        : ReferenceId(415): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_getSelf3":
after transform: ReferenceId(415): ReferenceFlags(Read)
rebuilt        : ReferenceId(417): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(43): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(421): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(427): ReferenceFlags(Read)
rebuilt        : ReferenceId(423): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(423): ReferenceFlags(Read)
rebuilt        : ReferenceId(425): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref22":
after transform: ReferenceId(434): ReferenceFlags(Read)
rebuilt        : ReferenceId(429): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref22$getSelf":
after transform: ReferenceId(437): ReferenceFlags(Read)
rebuilt        : ReferenceId(431): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(433): ReferenceFlags(Read)
rebuilt        : ReferenceId(434): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_getSelf4":
after transform: ReferenceId(429): ReferenceFlags(Read)
rebuilt        : ReferenceId(436): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(442): ReferenceFlags(Read)
rebuilt        : ReferenceId(441): ReferenceFlags(Read | MemberWriteTarget)

* private/optional-chain-cast-to-boolean/input.js
x Output mismatch

* private/optional-chain-delete-property/input.js
Reference flags mismatch for "Foo":
after transform: ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(4): ReferenceFlags(Read)
Reference flags mismatch for "_self":
after transform: ReferenceId(39): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(45): ReferenceFlags(Read)
rebuilt        : ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_deep$very$o":
after transform: ReferenceId(41): ReferenceFlags(Read)
rebuilt        : ReferenceId(12): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(14): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(50): ReferenceFlags(Read)
rebuilt        : ReferenceId(16): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(46): ReferenceFlags(Read)
rebuilt        : ReferenceId(18): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(20): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(55): ReferenceFlags(Read)
rebuilt        : ReferenceId(22): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(51): ReferenceFlags(Read)
rebuilt        : ReferenceId(24): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(60): ReferenceFlags(Read)
rebuilt        : ReferenceId(28): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(56): ReferenceFlags(Read)
rebuilt        : ReferenceId(30): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(65): ReferenceFlags(Read)
rebuilt        : ReferenceId(34): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(61): ReferenceFlags(Read)
rebuilt        : ReferenceId(36): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(70): ReferenceFlags(Read)
rebuilt        : ReferenceId(40): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(66): ReferenceFlags(Read)
rebuilt        : ReferenceId(42): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(13): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(44): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(75): ReferenceFlags(Read)
rebuilt        : ReferenceId(46): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(71): ReferenceFlags(Read)
rebuilt        : ReferenceId(48): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(80): ReferenceFlags(Read)
rebuilt        : ReferenceId(53): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(76): ReferenceFlags(Read)
rebuilt        : ReferenceId(55): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(87): ReferenceFlags(Read)
rebuilt        : ReferenceId(60): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(83): ReferenceFlags(Read)
rebuilt        : ReferenceId(62): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(92): ReferenceFlags(Read)
rebuilt        : ReferenceId(66): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(88): ReferenceFlags(Read)
rebuilt        : ReferenceId(68): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(97): ReferenceFlags(Read)
rebuilt        : ReferenceId(72): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(93): ReferenceFlags(Read)
rebuilt        : ReferenceId(74): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(102): ReferenceFlags(Read)
rebuilt        : ReferenceId(79): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(98): ReferenceFlags(Read)
rebuilt        : ReferenceId(81): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(109): ReferenceFlags(Read)
rebuilt        : ReferenceId(86): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(105): ReferenceFlags(Read)
rebuilt        : ReferenceId(88): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(114): ReferenceFlags(Read)
rebuilt        : ReferenceId(92): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(110): ReferenceFlags(Read)
rebuilt        : ReferenceId(94): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(21): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(96): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(119): ReferenceFlags(Read)
rebuilt        : ReferenceId(98): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(115): ReferenceFlags(Read)
rebuilt        : ReferenceId(100): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(125): ReferenceFlags(Read)
rebuilt        : ReferenceId(105): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_fnDeep$very$o":
after transform: ReferenceId(121): ReferenceFlags(Read)
rebuilt        : ReferenceId(107): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(23): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(109): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(130): ReferenceFlags(Read)
rebuilt        : ReferenceId(111): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(126): ReferenceFlags(Read)
rebuilt        : ReferenceId(113): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(24): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(115): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(135): ReferenceFlags(Read)
rebuilt        : ReferenceId(117): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(131): ReferenceFlags(Read)
rebuilt        : ReferenceId(119): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(140): ReferenceFlags(Read)
rebuilt        : ReferenceId(123): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(136): ReferenceFlags(Read)
rebuilt        : ReferenceId(125): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(145): ReferenceFlags(Read)
rebuilt        : ReferenceId(129): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(141): ReferenceFlags(Read)
rebuilt        : ReferenceId(131): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(150): ReferenceFlags(Read)
rebuilt        : ReferenceId(135): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(146): ReferenceFlags(Read)
rebuilt        : ReferenceId(137): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(28): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(139): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(155): ReferenceFlags(Read)
rebuilt        : ReferenceId(141): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(151): ReferenceFlags(Read)
rebuilt        : ReferenceId(143): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(160): ReferenceFlags(Read)
rebuilt        : ReferenceId(148): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(156): ReferenceFlags(Read)
rebuilt        : ReferenceId(150): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(167): ReferenceFlags(Read)
rebuilt        : ReferenceId(155): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(163): ReferenceFlags(Read)
rebuilt        : ReferenceId(157): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(172): ReferenceFlags(Read)
rebuilt        : ReferenceId(161): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(168): ReferenceFlags(Read)
rebuilt        : ReferenceId(163): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(177): ReferenceFlags(Read)
rebuilt        : ReferenceId(167): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(173): ReferenceFlags(Read)
rebuilt        : ReferenceId(169): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(182): ReferenceFlags(Read)
rebuilt        : ReferenceId(174): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(178): ReferenceFlags(Read)
rebuilt        : ReferenceId(176): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(189): ReferenceFlags(Read)
rebuilt        : ReferenceId(181): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(185): ReferenceFlags(Read)
rebuilt        : ReferenceId(183): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(194): ReferenceFlags(Read)
rebuilt        : ReferenceId(187): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(190): ReferenceFlags(Read)
rebuilt        : ReferenceId(189): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(196): ReferenceFlags(Read)
rebuilt        : ReferenceId(194): ReferenceFlags(Read | MemberWriteTarget)

* private/optional-chain-delete-property-with-transform/input.js
x Output mismatch

* private/optional-chain-in-function-param/input.js
x Output mismatch

* private/optional-chain-in-function-param-with-transform/input.js
x Output mismatch

* private/optional-chain-member-optional-call/input.js
Reference flags mismatch for "_m":
after transform: ReferenceId(51): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Foo":
after transform: ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(5): ReferenceFlags(Read)
Reference flags mismatch for "_m":
after transform: ReferenceId(52): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Foo":
after transform: ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(7): ReferenceFlags(Read)
Reference flags mismatch for "_m":
after transform: ReferenceId(53): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Foo":
after transform: ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(9): ReferenceFlags(Read)
Reference flags mismatch for "o":
after transform: ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(10): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(60): ReferenceFlags(Read)
rebuilt        : ReferenceId(12): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(54): ReferenceFlags(Read)
rebuilt        : ReferenceId(15): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(18): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(67): ReferenceFlags(Read)
rebuilt        : ReferenceId(20): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(61): ReferenceFlags(Read)
rebuilt        : ReferenceId(23): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(26): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(74): ReferenceFlags(Read)
rebuilt        : ReferenceId(28): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(68): ReferenceFlags(Read)
rebuilt        : ReferenceId(31): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(82): ReferenceFlags(Read)
rebuilt        : ReferenceId(37): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_deep$very$o":
after transform: ReferenceId(76): ReferenceFlags(Read)
rebuilt        : ReferenceId(40): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(90): ReferenceFlags(Read)
rebuilt        : ReferenceId(46): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_deep$very$o2":
after transform: ReferenceId(84): ReferenceFlags(Read)
rebuilt        : ReferenceId(49): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(98): ReferenceFlags(Read)
rebuilt        : ReferenceId(55): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_deep$very$o3":
after transform: ReferenceId(92): ReferenceFlags(Read)
rebuilt        : ReferenceId(58): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(15): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(61): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(105): ReferenceFlags(Read)
rebuilt        : ReferenceId(63): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(108): ReferenceFlags(Read)
rebuilt        : ReferenceId(66): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(99): ReferenceFlags(Read)
rebuilt        : ReferenceId(68): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(16): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(72): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(115): ReferenceFlags(Read)
rebuilt        : ReferenceId(74): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(118): ReferenceFlags(Read)
rebuilt        : ReferenceId(77): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(109): ReferenceFlags(Read)
rebuilt        : ReferenceId(79): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(17): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(84): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(123): ReferenceFlags(Read)
rebuilt        : ReferenceId(86): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(119): ReferenceFlags(Read)
rebuilt        : ReferenceId(88): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(131): ReferenceFlags(Read)
rebuilt        : ReferenceId(91): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref":
after transform: ReferenceId(125): ReferenceFlags(Read)
rebuilt        : ReferenceId(94): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(18): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(98): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(136): ReferenceFlags(Read)
rebuilt        : ReferenceId(100): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(132): ReferenceFlags(Read)
rebuilt        : ReferenceId(102): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(144): ReferenceFlags(Read)
rebuilt        : ReferenceId(105): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref2":
after transform: ReferenceId(138): ReferenceFlags(Read)
rebuilt        : ReferenceId(108): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(149): ReferenceFlags(Read)
rebuilt        : ReferenceId(114): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(145): ReferenceFlags(Read)
rebuilt        : ReferenceId(116): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(157): ReferenceFlags(Read)
rebuilt        : ReferenceId(119): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_self2":
after transform: ReferenceId(151): ReferenceFlags(Read)
rebuilt        : ReferenceId(122): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(20): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(125): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(164): ReferenceFlags(Read)
rebuilt        : ReferenceId(127): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(167): ReferenceFlags(Read)
rebuilt        : ReferenceId(130): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(158): ReferenceFlags(Read)
rebuilt        : ReferenceId(132): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(21): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(137): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(172): ReferenceFlags(Read)
rebuilt        : ReferenceId(140): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(168): ReferenceFlags(Read)
rebuilt        : ReferenceId(142): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(182): ReferenceFlags(Read)
rebuilt        : ReferenceId(145): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref3":
after transform: ReferenceId(176): ReferenceFlags(Read)
rebuilt        : ReferenceId(148): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(22): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(153): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(187): ReferenceFlags(Read)
rebuilt        : ReferenceId(155): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(183): ReferenceFlags(Read)
rebuilt        : ReferenceId(157): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(195): ReferenceFlags(Read)
rebuilt        : ReferenceId(160): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref4":
after transform: ReferenceId(189): ReferenceFlags(Read)
rebuilt        : ReferenceId(163): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(23): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(168): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(200): ReferenceFlags(Read)
rebuilt        : ReferenceId(170): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(196): ReferenceFlags(Read)
rebuilt        : ReferenceId(172): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(210): ReferenceFlags(Read)
rebuilt        : ReferenceId(175): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref5$getSelf":
after transform: ReferenceId(204): ReferenceFlags(Read)
rebuilt        : ReferenceId(178): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(24): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(183): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(215): ReferenceFlags(Read)
rebuilt        : ReferenceId(185): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(211): ReferenceFlags(Read)
rebuilt        : ReferenceId(187): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(223): ReferenceFlags(Read)
rebuilt        : ReferenceId(190): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref6":
after transform: ReferenceId(217): ReferenceFlags(Read)
rebuilt        : ReferenceId(193): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(228): ReferenceFlags(Read)
rebuilt        : ReferenceId(200): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(224): ReferenceFlags(Read)
rebuilt        : ReferenceId(202): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(238): ReferenceFlags(Read)
rebuilt        : ReferenceId(206): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_call":
after transform: ReferenceId(232): ReferenceFlags(Read)
rebuilt        : ReferenceId(209): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(243): ReferenceFlags(Read)
rebuilt        : ReferenceId(215): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(239): ReferenceFlags(Read)
rebuilt        : ReferenceId(217): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(251): ReferenceFlags(Read)
rebuilt        : ReferenceId(220): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_getSelf":
after transform: ReferenceId(245): ReferenceFlags(Read)
rebuilt        : ReferenceId(223): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(256): ReferenceFlags(Read)
rebuilt        : ReferenceId(229): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(252): ReferenceFlags(Read)
rebuilt        : ReferenceId(231): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(264): ReferenceFlags(Read)
rebuilt        : ReferenceId(234): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_getSelf2":
after transform: ReferenceId(258): ReferenceFlags(Read)
rebuilt        : ReferenceId(237): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(28): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(240): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(271): ReferenceFlags(Read)
rebuilt        : ReferenceId(242): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(265): ReferenceFlags(Read)
rebuilt        : ReferenceId(245): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(29): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(248): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(278): ReferenceFlags(Read)
rebuilt        : ReferenceId(250): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(272): ReferenceFlags(Read)
rebuilt        : ReferenceId(253): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(30): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(256): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(285): ReferenceFlags(Read)
rebuilt        : ReferenceId(258): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(279): ReferenceFlags(Read)
rebuilt        : ReferenceId(261): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(293): ReferenceFlags(Read)
rebuilt        : ReferenceId(267): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_fnDeep$very$o":
after transform: ReferenceId(287): ReferenceFlags(Read)
rebuilt        : ReferenceId(270): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(301): ReferenceFlags(Read)
rebuilt        : ReferenceId(276): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_fnDeep$very$o2":
after transform: ReferenceId(295): ReferenceFlags(Read)
rebuilt        : ReferenceId(279): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(309): ReferenceFlags(Read)
rebuilt        : ReferenceId(285): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_fnDeep$very$o3":
after transform: ReferenceId(303): ReferenceFlags(Read)
rebuilt        : ReferenceId(288): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(34): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(291): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(316): ReferenceFlags(Read)
rebuilt        : ReferenceId(293): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(319): ReferenceFlags(Read)
rebuilt        : ReferenceId(296): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(310): ReferenceFlags(Read)
rebuilt        : ReferenceId(298): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(35): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(302): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(326): ReferenceFlags(Read)
rebuilt        : ReferenceId(304): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(329): ReferenceFlags(Read)
rebuilt        : ReferenceId(307): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(320): ReferenceFlags(Read)
rebuilt        : ReferenceId(309): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(36): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(314): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(334): ReferenceFlags(Read)
rebuilt        : ReferenceId(316): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(330): ReferenceFlags(Read)
rebuilt        : ReferenceId(318): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(342): ReferenceFlags(Read)
rebuilt        : ReferenceId(321): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref7":
after transform: ReferenceId(336): ReferenceFlags(Read)
rebuilt        : ReferenceId(324): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(37): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(328): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(347): ReferenceFlags(Read)
rebuilt        : ReferenceId(330): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(343): ReferenceFlags(Read)
rebuilt        : ReferenceId(332): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(355): ReferenceFlags(Read)
rebuilt        : ReferenceId(335): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref8":
after transform: ReferenceId(349): ReferenceFlags(Read)
rebuilt        : ReferenceId(338): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(360): ReferenceFlags(Read)
rebuilt        : ReferenceId(344): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(356): ReferenceFlags(Read)
rebuilt        : ReferenceId(346): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(368): ReferenceFlags(Read)
rebuilt        : ReferenceId(349): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_self3":
after transform: ReferenceId(362): ReferenceFlags(Read)
rebuilt        : ReferenceId(352): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(39): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(355): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(375): ReferenceFlags(Read)
rebuilt        : ReferenceId(357): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(378): ReferenceFlags(Read)
rebuilt        : ReferenceId(360): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(369): ReferenceFlags(Read)
rebuilt        : ReferenceId(362): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(40): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(367): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(383): ReferenceFlags(Read)
rebuilt        : ReferenceId(370): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(379): ReferenceFlags(Read)
rebuilt        : ReferenceId(372): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(393): ReferenceFlags(Read)
rebuilt        : ReferenceId(375): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref9":
after transform: ReferenceId(387): ReferenceFlags(Read)
rebuilt        : ReferenceId(378): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(41): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(383): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(398): ReferenceFlags(Read)
rebuilt        : ReferenceId(385): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(394): ReferenceFlags(Read)
rebuilt        : ReferenceId(387): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(406): ReferenceFlags(Read)
rebuilt        : ReferenceId(390): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref10":
after transform: ReferenceId(400): ReferenceFlags(Read)
rebuilt        : ReferenceId(393): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(42): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(398): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(411): ReferenceFlags(Read)
rebuilt        : ReferenceId(400): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(407): ReferenceFlags(Read)
rebuilt        : ReferenceId(402): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(421): ReferenceFlags(Read)
rebuilt        : ReferenceId(405): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref11$getSelf":
after transform: ReferenceId(415): ReferenceFlags(Read)
rebuilt        : ReferenceId(408): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(43): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(413): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(426): ReferenceFlags(Read)
rebuilt        : ReferenceId(415): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(422): ReferenceFlags(Read)
rebuilt        : ReferenceId(417): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(434): ReferenceFlags(Read)
rebuilt        : ReferenceId(420): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref12":
after transform: ReferenceId(428): ReferenceFlags(Read)
rebuilt        : ReferenceId(423): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(439): ReferenceFlags(Read)
rebuilt        : ReferenceId(430): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(435): ReferenceFlags(Read)
rebuilt        : ReferenceId(432): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(449): ReferenceFlags(Read)
rebuilt        : ReferenceId(436): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_call2":
after transform: ReferenceId(443): ReferenceFlags(Read)
rebuilt        : ReferenceId(439): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(454): ReferenceFlags(Read)
rebuilt        : ReferenceId(445): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(450): ReferenceFlags(Read)
rebuilt        : ReferenceId(447): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(462): ReferenceFlags(Read)
rebuilt        : ReferenceId(450): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_getSelf3":
after transform: ReferenceId(456): ReferenceFlags(Read)
rebuilt        : ReferenceId(453): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(467): ReferenceFlags(Read)
rebuilt        : ReferenceId(459): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(463): ReferenceFlags(Read)
rebuilt        : ReferenceId(461): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(475): ReferenceFlags(Read)
rebuilt        : ReferenceId(464): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_getSelf4":
after transform: ReferenceId(469): ReferenceFlags(Read)
rebuilt        : ReferenceId(467): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(50): ReferenceFlags(Read)
rebuilt        : ReferenceId(472): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_Foo":
after transform: ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(475): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(477): ReferenceFlags(Read)
rebuilt        : ReferenceId(476): ReferenceFlags(Read | MemberWriteTarget)

* private/optional-chain-member-optional-call-spread-arguments/input.js
x Output mismatch

* private/optional-chain-member-optional-call-with-transform/input.js
x Output mismatch

* private/optional-chain-optional-member-call/input.js
Reference flags mismatch for "Foo":
after transform: ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(4): ReferenceFlags(Read)
Reference flags mismatch for "_m":
after transform: ReferenceId(53): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Foo":
after transform: ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(8): ReferenceFlags(Read)
Reference flags mismatch for "_m":
after transform: ReferenceId(56): ReferenceFlags(Read)
rebuilt        : ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Foo":
after transform: ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(12): ReferenceFlags(Read)
Reference flags mismatch for "_m":
after transform: ReferenceId(59): ReferenceFlags(Read)
rebuilt        : ReferenceId(14): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(16): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(66): ReferenceFlags(Read)
rebuilt        : ReferenceId(18): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(60): ReferenceFlags(Read)
rebuilt        : ReferenceId(21): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(24): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(73): ReferenceFlags(Read)
rebuilt        : ReferenceId(26): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(67): ReferenceFlags(Read)
rebuilt        : ReferenceId(29): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(32): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(80): ReferenceFlags(Read)
rebuilt        : ReferenceId(34): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(74): ReferenceFlags(Read)
rebuilt        : ReferenceId(37): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(88): ReferenceFlags(Read)
rebuilt        : ReferenceId(43): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_deep$very$o":
after transform: ReferenceId(82): ReferenceFlags(Read)
rebuilt        : ReferenceId(46): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(96): ReferenceFlags(Read)
rebuilt        : ReferenceId(52): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_deep$very$o2":
after transform: ReferenceId(90): ReferenceFlags(Read)
rebuilt        : ReferenceId(55): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(104): ReferenceFlags(Read)
rebuilt        : ReferenceId(61): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_deep$very$o3":
after transform: ReferenceId(98): ReferenceFlags(Read)
rebuilt        : ReferenceId(64): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(15): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(67): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(111): ReferenceFlags(Read)
rebuilt        : ReferenceId(69): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(114): ReferenceFlags(Read)
rebuilt        : ReferenceId(72): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(105): ReferenceFlags(Read)
rebuilt        : ReferenceId(74): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(16): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(78): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(121): ReferenceFlags(Read)
rebuilt        : ReferenceId(80): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(124): ReferenceFlags(Read)
rebuilt        : ReferenceId(83): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(115): ReferenceFlags(Read)
rebuilt        : ReferenceId(85): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(17): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(90): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(129): ReferenceFlags(Read)
rebuilt        : ReferenceId(92): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(125): ReferenceFlags(Read)
rebuilt        : ReferenceId(94): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(137): ReferenceFlags(Read)
rebuilt        : ReferenceId(97): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref":
after transform: ReferenceId(131): ReferenceFlags(Read)
rebuilt        : ReferenceId(100): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(18): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(104): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(142): ReferenceFlags(Read)
rebuilt        : ReferenceId(106): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(138): ReferenceFlags(Read)
rebuilt        : ReferenceId(108): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(150): ReferenceFlags(Read)
rebuilt        : ReferenceId(111): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref2":
after transform: ReferenceId(144): ReferenceFlags(Read)
rebuilt        : ReferenceId(114): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(155): ReferenceFlags(Read)
rebuilt        : ReferenceId(120): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(151): ReferenceFlags(Read)
rebuilt        : ReferenceId(122): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(163): ReferenceFlags(Read)
rebuilt        : ReferenceId(125): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_self2":
after transform: ReferenceId(157): ReferenceFlags(Read)
rebuilt        : ReferenceId(128): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(20): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(131): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(170): ReferenceFlags(Read)
rebuilt        : ReferenceId(133): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(173): ReferenceFlags(Read)
rebuilt        : ReferenceId(136): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(164): ReferenceFlags(Read)
rebuilt        : ReferenceId(138): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(21): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(143): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(178): ReferenceFlags(Read)
rebuilt        : ReferenceId(146): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(174): ReferenceFlags(Read)
rebuilt        : ReferenceId(148): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(188): ReferenceFlags(Read)
rebuilt        : ReferenceId(151): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref3":
after transform: ReferenceId(182): ReferenceFlags(Read)
rebuilt        : ReferenceId(154): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(22): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(159): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(193): ReferenceFlags(Read)
rebuilt        : ReferenceId(161): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(189): ReferenceFlags(Read)
rebuilt        : ReferenceId(163): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(201): ReferenceFlags(Read)
rebuilt        : ReferenceId(166): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref4":
after transform: ReferenceId(195): ReferenceFlags(Read)
rebuilt        : ReferenceId(169): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(23): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(174): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(206): ReferenceFlags(Read)
rebuilt        : ReferenceId(176): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(202): ReferenceFlags(Read)
rebuilt        : ReferenceId(178): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(216): ReferenceFlags(Read)
rebuilt        : ReferenceId(181): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref5$getSelf":
after transform: ReferenceId(210): ReferenceFlags(Read)
rebuilt        : ReferenceId(184): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(24): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(189): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(221): ReferenceFlags(Read)
rebuilt        : ReferenceId(191): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(217): ReferenceFlags(Read)
rebuilt        : ReferenceId(193): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(229): ReferenceFlags(Read)
rebuilt        : ReferenceId(196): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref6":
after transform: ReferenceId(223): ReferenceFlags(Read)
rebuilt        : ReferenceId(199): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(234): ReferenceFlags(Read)
rebuilt        : ReferenceId(206): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(230): ReferenceFlags(Read)
rebuilt        : ReferenceId(208): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(244): ReferenceFlags(Read)
rebuilt        : ReferenceId(212): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_call":
after transform: ReferenceId(238): ReferenceFlags(Read)
rebuilt        : ReferenceId(215): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(249): ReferenceFlags(Read)
rebuilt        : ReferenceId(221): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(245): ReferenceFlags(Read)
rebuilt        : ReferenceId(223): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(257): ReferenceFlags(Read)
rebuilt        : ReferenceId(226): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_getSelf":
after transform: ReferenceId(251): ReferenceFlags(Read)
rebuilt        : ReferenceId(229): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(262): ReferenceFlags(Read)
rebuilt        : ReferenceId(235): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(258): ReferenceFlags(Read)
rebuilt        : ReferenceId(237): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(270): ReferenceFlags(Read)
rebuilt        : ReferenceId(240): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_getSelf2":
after transform: ReferenceId(264): ReferenceFlags(Read)
rebuilt        : ReferenceId(243): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(28): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(246): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(277): ReferenceFlags(Read)
rebuilt        : ReferenceId(248): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(271): ReferenceFlags(Read)
rebuilt        : ReferenceId(251): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(29): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(254): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(284): ReferenceFlags(Read)
rebuilt        : ReferenceId(256): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(278): ReferenceFlags(Read)
rebuilt        : ReferenceId(259): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(30): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(262): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(291): ReferenceFlags(Read)
rebuilt        : ReferenceId(264): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(285): ReferenceFlags(Read)
rebuilt        : ReferenceId(267): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(299): ReferenceFlags(Read)
rebuilt        : ReferenceId(273): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_fnDeep$very$o":
after transform: ReferenceId(293): ReferenceFlags(Read)
rebuilt        : ReferenceId(276): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(307): ReferenceFlags(Read)
rebuilt        : ReferenceId(282): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_fnDeep$very$o2":
after transform: ReferenceId(301): ReferenceFlags(Read)
rebuilt        : ReferenceId(285): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(315): ReferenceFlags(Read)
rebuilt        : ReferenceId(291): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_fnDeep$very$o3":
after transform: ReferenceId(309): ReferenceFlags(Read)
rebuilt        : ReferenceId(294): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(34): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(297): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(322): ReferenceFlags(Read)
rebuilt        : ReferenceId(299): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(325): ReferenceFlags(Read)
rebuilt        : ReferenceId(302): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(316): ReferenceFlags(Read)
rebuilt        : ReferenceId(304): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(35): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(308): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(332): ReferenceFlags(Read)
rebuilt        : ReferenceId(310): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(335): ReferenceFlags(Read)
rebuilt        : ReferenceId(313): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(326): ReferenceFlags(Read)
rebuilt        : ReferenceId(315): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(36): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(320): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(340): ReferenceFlags(Read)
rebuilt        : ReferenceId(322): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(336): ReferenceFlags(Read)
rebuilt        : ReferenceId(324): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(348): ReferenceFlags(Read)
rebuilt        : ReferenceId(327): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref7":
after transform: ReferenceId(342): ReferenceFlags(Read)
rebuilt        : ReferenceId(330): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(37): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(334): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(353): ReferenceFlags(Read)
rebuilt        : ReferenceId(336): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(349): ReferenceFlags(Read)
rebuilt        : ReferenceId(338): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(361): ReferenceFlags(Read)
rebuilt        : ReferenceId(341): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref8":
after transform: ReferenceId(355): ReferenceFlags(Read)
rebuilt        : ReferenceId(344): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(366): ReferenceFlags(Read)
rebuilt        : ReferenceId(350): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(362): ReferenceFlags(Read)
rebuilt        : ReferenceId(352): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(374): ReferenceFlags(Read)
rebuilt        : ReferenceId(355): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_self3":
after transform: ReferenceId(368): ReferenceFlags(Read)
rebuilt        : ReferenceId(358): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(39): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(361): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(381): ReferenceFlags(Read)
rebuilt        : ReferenceId(363): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(384): ReferenceFlags(Read)
rebuilt        : ReferenceId(366): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(375): ReferenceFlags(Read)
rebuilt        : ReferenceId(368): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(40): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(373): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(389): ReferenceFlags(Read)
rebuilt        : ReferenceId(376): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(385): ReferenceFlags(Read)
rebuilt        : ReferenceId(378): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(399): ReferenceFlags(Read)
rebuilt        : ReferenceId(381): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref9":
after transform: ReferenceId(393): ReferenceFlags(Read)
rebuilt        : ReferenceId(384): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(41): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(389): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(404): ReferenceFlags(Read)
rebuilt        : ReferenceId(391): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(400): ReferenceFlags(Read)
rebuilt        : ReferenceId(393): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(412): ReferenceFlags(Read)
rebuilt        : ReferenceId(396): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref10":
after transform: ReferenceId(406): ReferenceFlags(Read)
rebuilt        : ReferenceId(399): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(42): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(404): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(417): ReferenceFlags(Read)
rebuilt        : ReferenceId(406): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(413): ReferenceFlags(Read)
rebuilt        : ReferenceId(408): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(427): ReferenceFlags(Read)
rebuilt        : ReferenceId(411): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref11$getSelf":
after transform: ReferenceId(421): ReferenceFlags(Read)
rebuilt        : ReferenceId(414): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(43): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(419): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(432): ReferenceFlags(Read)
rebuilt        : ReferenceId(421): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(428): ReferenceFlags(Read)
rebuilt        : ReferenceId(423): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(440): ReferenceFlags(Read)
rebuilt        : ReferenceId(426): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref12":
after transform: ReferenceId(434): ReferenceFlags(Read)
rebuilt        : ReferenceId(429): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(445): ReferenceFlags(Read)
rebuilt        : ReferenceId(436): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(441): ReferenceFlags(Read)
rebuilt        : ReferenceId(438): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(455): ReferenceFlags(Read)
rebuilt        : ReferenceId(442): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_call2":
after transform: ReferenceId(449): ReferenceFlags(Read)
rebuilt        : ReferenceId(445): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(460): ReferenceFlags(Read)
rebuilt        : ReferenceId(451): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(456): ReferenceFlags(Read)
rebuilt        : ReferenceId(453): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(468): ReferenceFlags(Read)
rebuilt        : ReferenceId(456): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_getSelf3":
after transform: ReferenceId(462): ReferenceFlags(Read)
rebuilt        : ReferenceId(459): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(473): ReferenceFlags(Read)
rebuilt        : ReferenceId(465): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(469): ReferenceFlags(Read)
rebuilt        : ReferenceId(467): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(481): ReferenceFlags(Read)
rebuilt        : ReferenceId(470): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_getSelf4":
after transform: ReferenceId(475): ReferenceFlags(Read)
rebuilt        : ReferenceId(473): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(50): ReferenceFlags(Read)
rebuilt        : ReferenceId(478): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_Foo":
after transform: ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(481): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(483): ReferenceFlags(Read)
rebuilt        : ReferenceId(482): ReferenceFlags(Read | MemberWriteTarget)

* private/optional-chain-optional-member-call-with-transform/input.js
Reference flags mismatch for "Foo":
after transform: ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(4): ReferenceFlags(Read)
Reference flags mismatch for "_m":
after transform: ReferenceId(53): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Foo":
after transform: ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(8): ReferenceFlags(Read)
Reference flags mismatch for "_m":
after transform: ReferenceId(56): ReferenceFlags(Read)
rebuilt        : ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Foo":
after transform: ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(12): ReferenceFlags(Read)
Reference flags mismatch for "_m":
after transform: ReferenceId(59): ReferenceFlags(Read)
rebuilt        : ReferenceId(14): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(66): ReferenceFlags(Read)
rebuilt        : ReferenceId(21): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(75): ReferenceFlags(Read)
rebuilt        : ReferenceId(31): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(84): ReferenceFlags(Read)
rebuilt        : ReferenceId(41): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_deep$very$o":
after transform: ReferenceId(96): ReferenceFlags(Read)
rebuilt        : ReferenceId(52): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(93): ReferenceFlags(Read)
rebuilt        : ReferenceId(54): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_deep$very$o2":
after transform: ReferenceId(108): ReferenceFlags(Read)
rebuilt        : ReferenceId(65): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(105): ReferenceFlags(Read)
rebuilt        : ReferenceId(67): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_deep$very$o3":
after transform: ReferenceId(120): ReferenceFlags(Read)
rebuilt        : ReferenceId(78): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(117): ReferenceFlags(Read)
rebuilt        : ReferenceId(80): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(15): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(86): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(127): ReferenceFlags(Read)
rebuilt        : ReferenceId(88): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(123): ReferenceFlags(Read)
rebuilt        : ReferenceId(90): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(134): ReferenceFlags(Read)
rebuilt        : ReferenceId(93): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(16): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(99): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(139): ReferenceFlags(Read)
rebuilt        : ReferenceId(101): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(135): ReferenceFlags(Read)
rebuilt        : ReferenceId(103): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(146): ReferenceFlags(Read)
rebuilt        : ReferenceId(106): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(17): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(113): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(151): ReferenceFlags(Read)
rebuilt        : ReferenceId(115): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(147): ReferenceFlags(Read)
rebuilt        : ReferenceId(117): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref3":
after transform: ReferenceId(159): ReferenceFlags(Read)
rebuilt        : ReferenceId(120): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(158): ReferenceFlags(Read)
rebuilt        : ReferenceId(122): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(18): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(129): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(166): ReferenceFlags(Read)
rebuilt        : ReferenceId(131): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(162): ReferenceFlags(Read)
rebuilt        : ReferenceId(133): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref4":
after transform: ReferenceId(174): ReferenceFlags(Read)
rebuilt        : ReferenceId(136): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(173): ReferenceFlags(Read)
rebuilt        : ReferenceId(138): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(19): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(145): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(181): ReferenceFlags(Read)
rebuilt        : ReferenceId(147): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(177): ReferenceFlags(Read)
rebuilt        : ReferenceId(149): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref5":
after transform: ReferenceId(189): ReferenceFlags(Read)
rebuilt        : ReferenceId(153): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref5":
after transform: ReferenceId(192): ReferenceFlags(Read)
rebuilt        : ReferenceId(155): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(188): ReferenceFlags(Read)
rebuilt        : ReferenceId(157): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(20): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(163): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(199): ReferenceFlags(Read)
rebuilt        : ReferenceId(165): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(195): ReferenceFlags(Read)
rebuilt        : ReferenceId(167): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(206): ReferenceFlags(Read)
rebuilt        : ReferenceId(170): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(21): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(177): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(211): ReferenceFlags(Read)
rebuilt        : ReferenceId(180): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(207): ReferenceFlags(Read)
rebuilt        : ReferenceId(182): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref7":
after transform: ReferenceId(221): ReferenceFlags(Read)
rebuilt        : ReferenceId(185): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(220): ReferenceFlags(Read)
rebuilt        : ReferenceId(188): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(22): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(195): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(228): ReferenceFlags(Read)
rebuilt        : ReferenceId(197): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(224): ReferenceFlags(Read)
rebuilt        : ReferenceId(199): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref8":
after transform: ReferenceId(236): ReferenceFlags(Read)
rebuilt        : ReferenceId(202): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(235): ReferenceFlags(Read)
rebuilt        : ReferenceId(204): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(23): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(211): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(243): ReferenceFlags(Read)
rebuilt        : ReferenceId(213): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(239): ReferenceFlags(Read)
rebuilt        : ReferenceId(215): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref9":
after transform: ReferenceId(251): ReferenceFlags(Read)
rebuilt        : ReferenceId(219): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref9$getSelf":
after transform: ReferenceId(254): ReferenceFlags(Read)
rebuilt        : ReferenceId(221): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(250): ReferenceFlags(Read)
rebuilt        : ReferenceId(224): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(24): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(231): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(262): ReferenceFlags(Read)
rebuilt        : ReferenceId(233): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(258): ReferenceFlags(Read)
rebuilt        : ReferenceId(235): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref10":
after transform: ReferenceId(270): ReferenceFlags(Read)
rebuilt        : ReferenceId(238): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(269): ReferenceFlags(Read)
rebuilt        : ReferenceId(240): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(25): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(247): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(277): ReferenceFlags(Read)
rebuilt        : ReferenceId(250): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(273): ReferenceFlags(Read)
rebuilt        : ReferenceId(252): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref11":
after transform: ReferenceId(287): ReferenceFlags(Read)
rebuilt        : ReferenceId(256): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref11":
after transform: ReferenceId(290): ReferenceFlags(Read)
rebuilt        : ReferenceId(259): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(286): ReferenceFlags(Read)
rebuilt        : ReferenceId(261): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(26): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(268): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(297): ReferenceFlags(Read)
rebuilt        : ReferenceId(270): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(293): ReferenceFlags(Read)
rebuilt        : ReferenceId(272): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref12":
after transform: ReferenceId(305): ReferenceFlags(Read)
rebuilt        : ReferenceId(276): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref12":
after transform: ReferenceId(308): ReferenceFlags(Read)
rebuilt        : ReferenceId(278): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(304): ReferenceFlags(Read)
rebuilt        : ReferenceId(280): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(27): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(287): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(315): ReferenceFlags(Read)
rebuilt        : ReferenceId(289): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(311): ReferenceFlags(Read)
rebuilt        : ReferenceId(291): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref13":
after transform: ReferenceId(323): ReferenceFlags(Read)
rebuilt        : ReferenceId(295): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref13$getSelf":
after transform: ReferenceId(326): ReferenceFlags(Read)
rebuilt        : ReferenceId(298): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref13$getSelf":
after transform: ReferenceId(330): ReferenceFlags(Read)
rebuilt        : ReferenceId(301): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(322): ReferenceFlags(Read)
rebuilt        : ReferenceId(303): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(339): ReferenceFlags(Read)
rebuilt        : ReferenceId(313): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(348): ReferenceFlags(Read)
rebuilt        : ReferenceId(323): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(357): ReferenceFlags(Read)
rebuilt        : ReferenceId(333): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_fnDeep$very$o":
after transform: ReferenceId(369): ReferenceFlags(Read)
rebuilt        : ReferenceId(344): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(366): ReferenceFlags(Read)
rebuilt        : ReferenceId(346): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_fnDeep$very$o2":
after transform: ReferenceId(381): ReferenceFlags(Read)
rebuilt        : ReferenceId(357): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(378): ReferenceFlags(Read)
rebuilt        : ReferenceId(359): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_fnDeep$very$o3":
after transform: ReferenceId(393): ReferenceFlags(Read)
rebuilt        : ReferenceId(370): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(390): ReferenceFlags(Read)
rebuilt        : ReferenceId(372): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(34): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(378): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(400): ReferenceFlags(Read)
rebuilt        : ReferenceId(380): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(396): ReferenceFlags(Read)
rebuilt        : ReferenceId(382): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(407): ReferenceFlags(Read)
rebuilt        : ReferenceId(385): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(35): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(391): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(412): ReferenceFlags(Read)
rebuilt        : ReferenceId(393): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(408): ReferenceFlags(Read)
rebuilt        : ReferenceId(395): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(419): ReferenceFlags(Read)
rebuilt        : ReferenceId(398): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(36): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(405): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(424): ReferenceFlags(Read)
rebuilt        : ReferenceId(407): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(420): ReferenceFlags(Read)
rebuilt        : ReferenceId(409): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref16":
after transform: ReferenceId(432): ReferenceFlags(Read)
rebuilt        : ReferenceId(412): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(431): ReferenceFlags(Read)
rebuilt        : ReferenceId(414): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(37): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(421): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(439): ReferenceFlags(Read)
rebuilt        : ReferenceId(423): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(435): ReferenceFlags(Read)
rebuilt        : ReferenceId(425): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref17":
after transform: ReferenceId(447): ReferenceFlags(Read)
rebuilt        : ReferenceId(428): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(446): ReferenceFlags(Read)
rebuilt        : ReferenceId(430): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(38): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(437): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(454): ReferenceFlags(Read)
rebuilt        : ReferenceId(439): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(450): ReferenceFlags(Read)
rebuilt        : ReferenceId(441): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref18":
after transform: ReferenceId(462): ReferenceFlags(Read)
rebuilt        : ReferenceId(445): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref18":
after transform: ReferenceId(465): ReferenceFlags(Read)
rebuilt        : ReferenceId(447): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(461): ReferenceFlags(Read)
rebuilt        : ReferenceId(449): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(39): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(455): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(472): ReferenceFlags(Read)
rebuilt        : ReferenceId(457): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(468): ReferenceFlags(Read)
rebuilt        : ReferenceId(459): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(479): ReferenceFlags(Read)
rebuilt        : ReferenceId(462): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(40): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(469): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(484): ReferenceFlags(Read)
rebuilt        : ReferenceId(472): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(480): ReferenceFlags(Read)
rebuilt        : ReferenceId(474): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref20":
after transform: ReferenceId(494): ReferenceFlags(Read)
rebuilt        : ReferenceId(477): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(493): ReferenceFlags(Read)
rebuilt        : ReferenceId(480): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(41): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(487): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(501): ReferenceFlags(Read)
rebuilt        : ReferenceId(489): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(497): ReferenceFlags(Read)
rebuilt        : ReferenceId(491): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref21":
after transform: ReferenceId(509): ReferenceFlags(Read)
rebuilt        : ReferenceId(494): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(508): ReferenceFlags(Read)
rebuilt        : ReferenceId(496): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(42): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(503): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(516): ReferenceFlags(Read)
rebuilt        : ReferenceId(505): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(512): ReferenceFlags(Read)
rebuilt        : ReferenceId(507): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref22":
after transform: ReferenceId(524): ReferenceFlags(Read)
rebuilt        : ReferenceId(511): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref22$getSelf":
after transform: ReferenceId(527): ReferenceFlags(Read)
rebuilt        : ReferenceId(513): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(523): ReferenceFlags(Read)
rebuilt        : ReferenceId(516): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(43): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(523): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(535): ReferenceFlags(Read)
rebuilt        : ReferenceId(525): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(531): ReferenceFlags(Read)
rebuilt        : ReferenceId(527): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref23":
after transform: ReferenceId(543): ReferenceFlags(Read)
rebuilt        : ReferenceId(530): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(542): ReferenceFlags(Read)
rebuilt        : ReferenceId(532): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(44): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(539): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(550): ReferenceFlags(Read)
rebuilt        : ReferenceId(542): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(546): ReferenceFlags(Read)
rebuilt        : ReferenceId(544): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref24":
after transform: ReferenceId(560): ReferenceFlags(Read)
rebuilt        : ReferenceId(548): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref24":
after transform: ReferenceId(563): ReferenceFlags(Read)
rebuilt        : ReferenceId(551): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(559): ReferenceFlags(Read)
rebuilt        : ReferenceId(553): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(45): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(560): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(570): ReferenceFlags(Read)
rebuilt        : ReferenceId(562): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(566): ReferenceFlags(Read)
rebuilt        : ReferenceId(564): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref25":
after transform: ReferenceId(578): ReferenceFlags(Read)
rebuilt        : ReferenceId(568): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref25":
after transform: ReferenceId(581): ReferenceFlags(Read)
rebuilt        : ReferenceId(570): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(577): ReferenceFlags(Read)
rebuilt        : ReferenceId(572): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(46): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(579): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(588): ReferenceFlags(Read)
rebuilt        : ReferenceId(581): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(584): ReferenceFlags(Read)
rebuilt        : ReferenceId(583): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref26":
after transform: ReferenceId(596): ReferenceFlags(Read)
rebuilt        : ReferenceId(587): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref26$getSelf":
after transform: ReferenceId(599): ReferenceFlags(Read)
rebuilt        : ReferenceId(590): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref26$getSelf":
after transform: ReferenceId(603): ReferenceFlags(Read)
rebuilt        : ReferenceId(593): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(595): ReferenceFlags(Read)
rebuilt        : ReferenceId(595): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(50): ReferenceFlags(Read)
rebuilt        : ReferenceId(602): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_Foo":
after transform: ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(605): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(607): ReferenceFlags(Read)
rebuilt        : ReferenceId(606): ReferenceFlags(Read | MemberWriteTarget)

* private/optional-chain-optional-property/input.js
Reference flags mismatch for "Foo":
after transform: ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(4): ReferenceFlags(Read)
Reference flags mismatch for "_x":
after transform: ReferenceId(50): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Foo":
after transform: ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(7): ReferenceFlags(Read)
Reference flags mismatch for "_x":
after transform: ReferenceId(53): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Foo":
after transform: ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(10): ReferenceFlags(Read)
Reference flags mismatch for "_x":
after transform: ReferenceId(56): ReferenceFlags(Read)
rebuilt        : ReferenceId(12): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(62): ReferenceFlags(Read)
rebuilt        : ReferenceId(16): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(68): ReferenceFlags(Read)
rebuilt        : ReferenceId(23): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(74): ReferenceFlags(Read)
rebuilt        : ReferenceId(30): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(80): ReferenceFlags(Read)
rebuilt        : ReferenceId(37): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(86): ReferenceFlags(Read)
rebuilt        : ReferenceId(44): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(92): ReferenceFlags(Read)
rebuilt        : ReferenceId(51): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(15): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(56): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(97): ReferenceFlags(Read)
rebuilt        : ReferenceId(58): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(93): ReferenceFlags(Read)
rebuilt        : ReferenceId(60): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(103): ReferenceFlags(Read)
rebuilt        : ReferenceId(63): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(16): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(68): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(108): ReferenceFlags(Read)
rebuilt        : ReferenceId(70): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(104): ReferenceFlags(Read)
rebuilt        : ReferenceId(72): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(114): ReferenceFlags(Read)
rebuilt        : ReferenceId(75): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(119): ReferenceFlags(Read)
rebuilt        : ReferenceId(82): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(115): ReferenceFlags(Read)
rebuilt        : ReferenceId(84): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(125): ReferenceFlags(Read)
rebuilt        : ReferenceId(87): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(130): ReferenceFlags(Read)
rebuilt        : ReferenceId(94): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(126): ReferenceFlags(Read)
rebuilt        : ReferenceId(96): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(136): ReferenceFlags(Read)
rebuilt        : ReferenceId(99): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(141): ReferenceFlags(Read)
rebuilt        : ReferenceId(106): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(137): ReferenceFlags(Read)
rebuilt        : ReferenceId(108): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(147): ReferenceFlags(Read)
rebuilt        : ReferenceId(111): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(20): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(116): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(152): ReferenceFlags(Read)
rebuilt        : ReferenceId(118): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(148): ReferenceFlags(Read)
rebuilt        : ReferenceId(120): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(158): ReferenceFlags(Read)
rebuilt        : ReferenceId(123): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(163): ReferenceFlags(Read)
rebuilt        : ReferenceId(131): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(159): ReferenceFlags(Read)
rebuilt        : ReferenceId(133): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(171): ReferenceFlags(Read)
rebuilt        : ReferenceId(137): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(176): ReferenceFlags(Read)
rebuilt        : ReferenceId(144): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(172): ReferenceFlags(Read)
rebuilt        : ReferenceId(146): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(182): ReferenceFlags(Read)
rebuilt        : ReferenceId(149): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(187): ReferenceFlags(Read)
rebuilt        : ReferenceId(156): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(183): ReferenceFlags(Read)
rebuilt        : ReferenceId(158): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(193): ReferenceFlags(Read)
rebuilt        : ReferenceId(161): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(198): ReferenceFlags(Read)
rebuilt        : ReferenceId(168): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(194): ReferenceFlags(Read)
rebuilt        : ReferenceId(170): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(204): ReferenceFlags(Read)
rebuilt        : ReferenceId(173): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(209): ReferenceFlags(Read)
rebuilt        : ReferenceId(181): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(205): ReferenceFlags(Read)
rebuilt        : ReferenceId(183): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(217): ReferenceFlags(Read)
rebuilt        : ReferenceId(187): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(222): ReferenceFlags(Read)
rebuilt        : ReferenceId(194): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(218): ReferenceFlags(Read)
rebuilt        : ReferenceId(196): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(228): ReferenceFlags(Read)
rebuilt        : ReferenceId(199): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(233): ReferenceFlags(Read)
rebuilt        : ReferenceId(206): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(229): ReferenceFlags(Read)
rebuilt        : ReferenceId(208): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(239): ReferenceFlags(Read)
rebuilt        : ReferenceId(211): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(245): ReferenceFlags(Read)
rebuilt        : ReferenceId(218): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(251): ReferenceFlags(Read)
rebuilt        : ReferenceId(225): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(257): ReferenceFlags(Read)
rebuilt        : ReferenceId(232): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(263): ReferenceFlags(Read)
rebuilt        : ReferenceId(239): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(269): ReferenceFlags(Read)
rebuilt        : ReferenceId(246): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(275): ReferenceFlags(Read)
rebuilt        : ReferenceId(253): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(34): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(258): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(280): ReferenceFlags(Read)
rebuilt        : ReferenceId(260): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(276): ReferenceFlags(Read)
rebuilt        : ReferenceId(262): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(286): ReferenceFlags(Read)
rebuilt        : ReferenceId(265): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(35): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(270): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(291): ReferenceFlags(Read)
rebuilt        : ReferenceId(272): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(287): ReferenceFlags(Read)
rebuilt        : ReferenceId(274): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(297): ReferenceFlags(Read)
rebuilt        : ReferenceId(277): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(302): ReferenceFlags(Read)
rebuilt        : ReferenceId(284): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(298): ReferenceFlags(Read)
rebuilt        : ReferenceId(286): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(308): ReferenceFlags(Read)
rebuilt        : ReferenceId(289): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(313): ReferenceFlags(Read)
rebuilt        : ReferenceId(296): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(309): ReferenceFlags(Read)
rebuilt        : ReferenceId(298): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(319): ReferenceFlags(Read)
rebuilt        : ReferenceId(301): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(324): ReferenceFlags(Read)
rebuilt        : ReferenceId(308): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(320): ReferenceFlags(Read)
rebuilt        : ReferenceId(310): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(330): ReferenceFlags(Read)
rebuilt        : ReferenceId(313): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(39): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(318): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(335): ReferenceFlags(Read)
rebuilt        : ReferenceId(320): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(331): ReferenceFlags(Read)
rebuilt        : ReferenceId(322): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(341): ReferenceFlags(Read)
rebuilt        : ReferenceId(325): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(346): ReferenceFlags(Read)
rebuilt        : ReferenceId(333): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(342): ReferenceFlags(Read)
rebuilt        : ReferenceId(335): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(354): ReferenceFlags(Read)
rebuilt        : ReferenceId(339): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(359): ReferenceFlags(Read)
rebuilt        : ReferenceId(346): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(355): ReferenceFlags(Read)
rebuilt        : ReferenceId(348): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(365): ReferenceFlags(Read)
rebuilt        : ReferenceId(351): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(370): ReferenceFlags(Read)
rebuilt        : ReferenceId(358): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(366): ReferenceFlags(Read)
rebuilt        : ReferenceId(360): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(376): ReferenceFlags(Read)
rebuilt        : ReferenceId(363): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(381): ReferenceFlags(Read)
rebuilt        : ReferenceId(370): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(377): ReferenceFlags(Read)
rebuilt        : ReferenceId(372): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(387): ReferenceFlags(Read)
rebuilt        : ReferenceId(375): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(392): ReferenceFlags(Read)
rebuilt        : ReferenceId(383): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(388): ReferenceFlags(Read)
rebuilt        : ReferenceId(385): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(400): ReferenceFlags(Read)
rebuilt        : ReferenceId(389): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(405): ReferenceFlags(Read)
rebuilt        : ReferenceId(396): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(401): ReferenceFlags(Read)
rebuilt        : ReferenceId(398): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(411): ReferenceFlags(Read)
rebuilt        : ReferenceId(401): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(416): ReferenceFlags(Read)
rebuilt        : ReferenceId(408): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(412): ReferenceFlags(Read)
rebuilt        : ReferenceId(410): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(422): ReferenceFlags(Read)
rebuilt        : ReferenceId(413): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(424): ReferenceFlags(Read)
rebuilt        : ReferenceId(420): ReferenceFlags(Read | MemberWriteTarget)

* private/optional-chain-optional-property-with-transform/input.js
Reference flags mismatch for "Foo":
after transform: ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(4): ReferenceFlags(Read)
Reference flags mismatch for "_x":
after transform: ReferenceId(50): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Foo":
after transform: ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(7): ReferenceFlags(Read)
Reference flags mismatch for "_x":
after transform: ReferenceId(53): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Foo":
after transform: ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(10): ReferenceFlags(Read)
Reference flags mismatch for "_x":
after transform: ReferenceId(56): ReferenceFlags(Read)
rebuilt        : ReferenceId(12): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(62): ReferenceFlags(Read)
rebuilt        : ReferenceId(18): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(70): ReferenceFlags(Read)
rebuilt        : ReferenceId(27): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(78): ReferenceFlags(Read)
rebuilt        : ReferenceId(36): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_deep$very$o":
after transform: ReferenceId(89): ReferenceFlags(Read)
rebuilt        : ReferenceId(46): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(86): ReferenceFlags(Read)
rebuilt        : ReferenceId(48): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_deep$very$o2":
after transform: ReferenceId(100): ReferenceFlags(Read)
rebuilt        : ReferenceId(58): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(97): ReferenceFlags(Read)
rebuilt        : ReferenceId(60): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_deep$very$o3":
after transform: ReferenceId(111): ReferenceFlags(Read)
rebuilt        : ReferenceId(70): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(108): ReferenceFlags(Read)
rebuilt        : ReferenceId(72): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(15): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(77): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(118): ReferenceFlags(Read)
rebuilt        : ReferenceId(79): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(114): ReferenceFlags(Read)
rebuilt        : ReferenceId(81): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(124): ReferenceFlags(Read)
rebuilt        : ReferenceId(84): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(16): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(89): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(129): ReferenceFlags(Read)
rebuilt        : ReferenceId(91): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(125): ReferenceFlags(Read)
rebuilt        : ReferenceId(93): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(135): ReferenceFlags(Read)
rebuilt        : ReferenceId(96): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(17): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(102): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(140): ReferenceFlags(Read)
rebuilt        : ReferenceId(104): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(136): ReferenceFlags(Read)
rebuilt        : ReferenceId(106): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref3":
after transform: ReferenceId(147): ReferenceFlags(Read)
rebuilt        : ReferenceId(109): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(146): ReferenceFlags(Read)
rebuilt        : ReferenceId(111): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(18): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(117): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(154): ReferenceFlags(Read)
rebuilt        : ReferenceId(119): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(150): ReferenceFlags(Read)
rebuilt        : ReferenceId(121): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref4":
after transform: ReferenceId(161): ReferenceFlags(Read)
rebuilt        : ReferenceId(124): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(160): ReferenceFlags(Read)
rebuilt        : ReferenceId(126): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(19): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(132): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(168): ReferenceFlags(Read)
rebuilt        : ReferenceId(134): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(164): ReferenceFlags(Read)
rebuilt        : ReferenceId(136): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref5":
after transform: ReferenceId(175): ReferenceFlags(Read)
rebuilt        : ReferenceId(140): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref5":
after transform: ReferenceId(178): ReferenceFlags(Read)
rebuilt        : ReferenceId(142): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(174): ReferenceFlags(Read)
rebuilt        : ReferenceId(144): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(20): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(149): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(185): ReferenceFlags(Read)
rebuilt        : ReferenceId(151): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(181): ReferenceFlags(Read)
rebuilt        : ReferenceId(153): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(191): ReferenceFlags(Read)
rebuilt        : ReferenceId(156): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(21): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(162): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(196): ReferenceFlags(Read)
rebuilt        : ReferenceId(165): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(192): ReferenceFlags(Read)
rebuilt        : ReferenceId(167): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref7":
after transform: ReferenceId(205): ReferenceFlags(Read)
rebuilt        : ReferenceId(170): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(204): ReferenceFlags(Read)
rebuilt        : ReferenceId(173): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(22): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(179): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(212): ReferenceFlags(Read)
rebuilt        : ReferenceId(181): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(208): ReferenceFlags(Read)
rebuilt        : ReferenceId(183): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref8":
after transform: ReferenceId(219): ReferenceFlags(Read)
rebuilt        : ReferenceId(186): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(218): ReferenceFlags(Read)
rebuilt        : ReferenceId(188): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(23): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(194): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(226): ReferenceFlags(Read)
rebuilt        : ReferenceId(196): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(222): ReferenceFlags(Read)
rebuilt        : ReferenceId(198): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref9":
after transform: ReferenceId(233): ReferenceFlags(Read)
rebuilt        : ReferenceId(202): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref9$getSelf":
after transform: ReferenceId(236): ReferenceFlags(Read)
rebuilt        : ReferenceId(204): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(232): ReferenceFlags(Read)
rebuilt        : ReferenceId(207): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(24): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(213): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(244): ReferenceFlags(Read)
rebuilt        : ReferenceId(215): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(240): ReferenceFlags(Read)
rebuilt        : ReferenceId(217): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref10":
after transform: ReferenceId(251): ReferenceFlags(Read)
rebuilt        : ReferenceId(220): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(250): ReferenceFlags(Read)
rebuilt        : ReferenceId(222): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(25): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(228): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(258): ReferenceFlags(Read)
rebuilt        : ReferenceId(231): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(254): ReferenceFlags(Read)
rebuilt        : ReferenceId(233): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref11":
after transform: ReferenceId(267): ReferenceFlags(Read)
rebuilt        : ReferenceId(237): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref11":
after transform: ReferenceId(270): ReferenceFlags(Read)
rebuilt        : ReferenceId(240): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(266): ReferenceFlags(Read)
rebuilt        : ReferenceId(242): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(26): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(248): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(277): ReferenceFlags(Read)
rebuilt        : ReferenceId(250): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(273): ReferenceFlags(Read)
rebuilt        : ReferenceId(252): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref12":
after transform: ReferenceId(284): ReferenceFlags(Read)
rebuilt        : ReferenceId(256): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref12":
after transform: ReferenceId(287): ReferenceFlags(Read)
rebuilt        : ReferenceId(258): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(283): ReferenceFlags(Read)
rebuilt        : ReferenceId(260): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(27): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(266): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(294): ReferenceFlags(Read)
rebuilt        : ReferenceId(268): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(290): ReferenceFlags(Read)
rebuilt        : ReferenceId(270): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref13":
after transform: ReferenceId(301): ReferenceFlags(Read)
rebuilt        : ReferenceId(274): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref13$getSelf":
after transform: ReferenceId(304): ReferenceFlags(Read)
rebuilt        : ReferenceId(277): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref13$getSelf":
after transform: ReferenceId(308): ReferenceFlags(Read)
rebuilt        : ReferenceId(280): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(300): ReferenceFlags(Read)
rebuilt        : ReferenceId(282): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(316): ReferenceFlags(Read)
rebuilt        : ReferenceId(291): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(324): ReferenceFlags(Read)
rebuilt        : ReferenceId(300): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(332): ReferenceFlags(Read)
rebuilt        : ReferenceId(309): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_fnDeep$very$o":
after transform: ReferenceId(343): ReferenceFlags(Read)
rebuilt        : ReferenceId(319): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(340): ReferenceFlags(Read)
rebuilt        : ReferenceId(321): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_fnDeep$very$o2":
after transform: ReferenceId(354): ReferenceFlags(Read)
rebuilt        : ReferenceId(331): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(351): ReferenceFlags(Read)
rebuilt        : ReferenceId(333): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_fnDeep$very$o3":
after transform: ReferenceId(365): ReferenceFlags(Read)
rebuilt        : ReferenceId(343): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(362): ReferenceFlags(Read)
rebuilt        : ReferenceId(345): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(34): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(350): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(372): ReferenceFlags(Read)
rebuilt        : ReferenceId(352): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(368): ReferenceFlags(Read)
rebuilt        : ReferenceId(354): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(378): ReferenceFlags(Read)
rebuilt        : ReferenceId(357): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(35): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(362): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(383): ReferenceFlags(Read)
rebuilt        : ReferenceId(364): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(379): ReferenceFlags(Read)
rebuilt        : ReferenceId(366): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(389): ReferenceFlags(Read)
rebuilt        : ReferenceId(369): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(36): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(375): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(394): ReferenceFlags(Read)
rebuilt        : ReferenceId(377): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(390): ReferenceFlags(Read)
rebuilt        : ReferenceId(379): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref16":
after transform: ReferenceId(401): ReferenceFlags(Read)
rebuilt        : ReferenceId(382): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(400): ReferenceFlags(Read)
rebuilt        : ReferenceId(384): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(37): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(390): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(408): ReferenceFlags(Read)
rebuilt        : ReferenceId(392): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(404): ReferenceFlags(Read)
rebuilt        : ReferenceId(394): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref17":
after transform: ReferenceId(415): ReferenceFlags(Read)
rebuilt        : ReferenceId(397): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(414): ReferenceFlags(Read)
rebuilt        : ReferenceId(399): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(38): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(405): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(422): ReferenceFlags(Read)
rebuilt        : ReferenceId(407): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(418): ReferenceFlags(Read)
rebuilt        : ReferenceId(409): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref18":
after transform: ReferenceId(429): ReferenceFlags(Read)
rebuilt        : ReferenceId(413): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref18":
after transform: ReferenceId(432): ReferenceFlags(Read)
rebuilt        : ReferenceId(415): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(428): ReferenceFlags(Read)
rebuilt        : ReferenceId(417): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(39): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(422): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(439): ReferenceFlags(Read)
rebuilt        : ReferenceId(424): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(435): ReferenceFlags(Read)
rebuilt        : ReferenceId(426): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(445): ReferenceFlags(Read)
rebuilt        : ReferenceId(429): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(40): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(435): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(450): ReferenceFlags(Read)
rebuilt        : ReferenceId(438): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(446): ReferenceFlags(Read)
rebuilt        : ReferenceId(440): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref20":
after transform: ReferenceId(459): ReferenceFlags(Read)
rebuilt        : ReferenceId(443): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(458): ReferenceFlags(Read)
rebuilt        : ReferenceId(446): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(41): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(452): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(466): ReferenceFlags(Read)
rebuilt        : ReferenceId(454): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(462): ReferenceFlags(Read)
rebuilt        : ReferenceId(456): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref21":
after transform: ReferenceId(473): ReferenceFlags(Read)
rebuilt        : ReferenceId(459): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(472): ReferenceFlags(Read)
rebuilt        : ReferenceId(461): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(42): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(467): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(480): ReferenceFlags(Read)
rebuilt        : ReferenceId(469): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(476): ReferenceFlags(Read)
rebuilt        : ReferenceId(471): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref22":
after transform: ReferenceId(487): ReferenceFlags(Read)
rebuilt        : ReferenceId(475): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref22$getSelf":
after transform: ReferenceId(490): ReferenceFlags(Read)
rebuilt        : ReferenceId(477): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(486): ReferenceFlags(Read)
rebuilt        : ReferenceId(480): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(43): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(486): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(498): ReferenceFlags(Read)
rebuilt        : ReferenceId(488): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(494): ReferenceFlags(Read)
rebuilt        : ReferenceId(490): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref23":
after transform: ReferenceId(505): ReferenceFlags(Read)
rebuilt        : ReferenceId(493): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(504): ReferenceFlags(Read)
rebuilt        : ReferenceId(495): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(44): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(501): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(512): ReferenceFlags(Read)
rebuilt        : ReferenceId(504): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(508): ReferenceFlags(Read)
rebuilt        : ReferenceId(506): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref24":
after transform: ReferenceId(521): ReferenceFlags(Read)
rebuilt        : ReferenceId(510): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref24":
after transform: ReferenceId(524): ReferenceFlags(Read)
rebuilt        : ReferenceId(513): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(520): ReferenceFlags(Read)
rebuilt        : ReferenceId(515): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(45): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(521): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(531): ReferenceFlags(Read)
rebuilt        : ReferenceId(523): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(527): ReferenceFlags(Read)
rebuilt        : ReferenceId(525): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref25":
after transform: ReferenceId(538): ReferenceFlags(Read)
rebuilt        : ReferenceId(529): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref25":
after transform: ReferenceId(541): ReferenceFlags(Read)
rebuilt        : ReferenceId(531): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(537): ReferenceFlags(Read)
rebuilt        : ReferenceId(533): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(46): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(539): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(548): ReferenceFlags(Read)
rebuilt        : ReferenceId(541): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(544): ReferenceFlags(Read)
rebuilt        : ReferenceId(543): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref26":
after transform: ReferenceId(555): ReferenceFlags(Read)
rebuilt        : ReferenceId(547): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref26$getSelf":
after transform: ReferenceId(558): ReferenceFlags(Read)
rebuilt        : ReferenceId(550): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref26$getSelf":
after transform: ReferenceId(562): ReferenceFlags(Read)
rebuilt        : ReferenceId(553): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(554): ReferenceFlags(Read)
rebuilt        : ReferenceId(555): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(566): ReferenceFlags(Read)
rebuilt        : ReferenceId(562): ReferenceFlags(Read | MemberWriteTarget)

* private/parenthesized-optional-member-call/input.js
Reference flags mismatch for "Foo":
after transform: ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(0): ReferenceFlags(Read)
Reference flags mismatch for "Foo":
after transform: ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(3): ReferenceFlags(Read)
Reference flags mismatch for "_m":
after transform: ReferenceId(20): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_m":
after transform: ReferenceId(23): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_m":
after transform: ReferenceId(26): ReferenceFlags(Read)
rebuilt        : ReferenceId(13): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(15): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(33): ReferenceFlags(Read)
rebuilt        : ReferenceId(17): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(27): ReferenceFlags(Read)
rebuilt        : ReferenceId(20): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(40): ReferenceFlags(Read)
rebuilt        : ReferenceId(25): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(34): ReferenceFlags(Read)
rebuilt        : ReferenceId(28): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(47): ReferenceFlags(Read)
rebuilt        : ReferenceId(33): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(41): ReferenceFlags(Read)
rebuilt        : ReferenceId(36): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(54): ReferenceFlags(Read)
rebuilt        : ReferenceId(42): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(61): ReferenceFlags(Read)
rebuilt        : ReferenceId(50): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(68): ReferenceFlags(Read)
rebuilt        : ReferenceId(58): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(75): ReferenceFlags(Read)
rebuilt        : ReferenceId(66): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(77): ReferenceFlags(Read)
rebuilt        : ReferenceId(73): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(17): ReferenceFlags(Read)
rebuilt        : ReferenceId(76): ReferenceFlags(Read | MemberWriteTarget)

* private/parenthesized-optional-member-call-with-transform/input.js
x Output mismatch

* private/preserve-comments/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)

* private/private-in-derived/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)

* private/reevaluated/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Foo":
after transform: ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(9): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "inst":
after transform: ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(13): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(14): ReferenceFlags(Read)
rebuilt        : ReferenceId(14): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Foo":
after transform: ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(16): ReferenceFlags(Read)

* private/reference-in-other-property/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(12): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(13): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(15): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(15): ReferenceFlags(Read)
rebuilt        : ReferenceId(17): ReferenceFlags(Read | MemberWriteTarget)

* private/regression-T7364/input.mjs
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(15): ReferenceFlags(Read)
rebuilt        : ReferenceId(14): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(17): ReferenceFlags(Read)
rebuilt        : ReferenceId(16): ReferenceFlags(Read | MemberWriteTarget)

* private/static/input.js
Reference flags mismatch for "_bar":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_bar":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)

* private/static-call/input.js
Reference flags mismatch for "_foo":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Foo":
after transform: ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(1): ReferenceFlags(Read)

* private/static-infer-name/input.js
x Output mismatch

* private/static-inherited/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_foo":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(12): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "val":
after transform: ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(6): ReferenceFlags(Read)
Reference flags mismatch for "_foo":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_foo":
after transform: ReferenceId(14): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_foo2":
after transform: ReferenceId(15): ReferenceFlags(Read)
rebuilt        : ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(17): ReferenceFlags(Read)
rebuilt        : ReferenceId(12): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "val":
after transform: ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(14): ReferenceFlags(Read)

* private/static-self-field/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)

* private/static-self-method/input.js
Scope flags mismatch:
after transform: ScopeId(4): ScopeFlags(StrictMode | Function)
rebuilt        : ScopeId(4): ScopeFlags(Function)
Scope flags mismatch:
after transform: ScopeId(6): ScopeFlags(StrictMode | Function)
rebuilt        : ScopeId(6): ScopeFlags(Function)

* private/static-shadow/input.js
x Output mismatch

* private/static-undefined/input.js
Reference flags mismatch for "_bar":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_bar":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)

* private/super-call/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)

* private/super-expression/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)

* private/super-statement/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)

* private/tagged-template/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Foo":
after transform: ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(5): ReferenceFlags(Read)

* private/tagged-template-static/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)

* private/update/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(18): ReferenceFlags(Read)
rebuilt        : ReferenceId(12): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(14): ReferenceFlags(Read)
rebuilt        : ReferenceId(15): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(28): ReferenceFlags(Read)
rebuilt        : ReferenceId(18): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "other":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(21): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(22): ReferenceFlags(Read)
rebuilt        : ReferenceId(23): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(37): ReferenceFlags(Read)
rebuilt        : ReferenceId(30): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "other":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(33): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(33): ReferenceFlags(Read)
rebuilt        : ReferenceId(35): ReferenceFlags(Read | MemberWriteTarget)

* private-loose/assignment/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(16): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "other":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(14): ReferenceFlags(Read)
rebuilt        : ReferenceId(13): ReferenceFlags(Read | MemberWriteTarget)

* private-loose/call/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)

* private-loose/canonical/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(46): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(47): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "x":
after transform: ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(9): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(14): ReferenceFlags(Read)
rebuilt        : ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "y":
after transform: ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(13): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(17): ReferenceFlags(Read)
rebuilt        : ReferenceId(14): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(20): ReferenceFlags(Read)
rebuilt        : ReferenceId(17): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "value":
after transform: ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(20): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(23): ReferenceFlags(Read)
rebuilt        : ReferenceId(21): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(26): ReferenceFlags(Read)
rebuilt        : ReferenceId(24): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "value":
after transform: ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(27): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(29): ReferenceFlags(Read)
rebuilt        : ReferenceId(28): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(32): ReferenceFlags(Read)
rebuilt        : ReferenceId(31): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "p":
after transform: ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(32): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(35): ReferenceFlags(Read)
rebuilt        : ReferenceId(35): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(38): ReferenceFlags(Read)
rebuilt        : ReferenceId(38): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "p":
after transform: ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(39): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(41): ReferenceFlags(Read)
rebuilt        : ReferenceId(42): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(44): ReferenceFlags(Read)
rebuilt        : ReferenceId(45): ReferenceFlags(Read | MemberWriteTarget)

* private-loose/class-shadow-builtins/input.mjs
x Output mismatch

* private-loose/constructor-collision/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)

* private-loose/declaration-order/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)

* private-loose/derived/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)

* private-loose/derived-multiple-supers/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)

* private-loose/destructuring-array-pattern/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "props":
after transform: ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(6): ReferenceFlags(Read)

* private-loose/destructuring-array-pattern-1/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(15): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(12): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "props":
after transform: ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(15): ReferenceFlags(Read)

* private-loose/destructuring-array-pattern-2/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "props":
after transform: ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(7): ReferenceFlags(Read)

* private-loose/destructuring-array-pattern-3/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "props":
after transform: ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(6): ReferenceFlags(Read)

* private-loose/destructuring-array-pattern-static/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Foo":
after transform: ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(2): ReferenceFlags(Read)
Reference flags mismatch for "Object":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)

* private-loose/destructuring-object-pattern/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "props":
after transform: ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(6): ReferenceFlags(Read)

* private-loose/destructuring-object-pattern-1/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(15): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(12): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "props":
after transform: ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(15): ReferenceFlags(Read)

* private-loose/destructuring-object-pattern-2/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "props":
after transform: ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(7): ReferenceFlags(Read)

* private-loose/destructuring-object-pattern-3/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "props":
after transform: ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(6): ReferenceFlags(Read)

* private-loose/destructuring-object-pattern-static/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Foo":
after transform: ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(2): ReferenceFlags(Read)
Reference flags mismatch for "Object":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)

* private-loose/extracted-this/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)

* private-loose/foobar/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_this":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)

* private-loose/instance/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)

* private-loose/instance-undefined/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)

* private-loose/logical-assignment/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(30): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(31): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(32): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_babelHelpers$classPr":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(13): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(16): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_babelHelpers$classPr2":
after transform: ReferenceId(16): ReferenceFlags(Read)
rebuilt        : ReferenceId(19): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(19): ReferenceFlags(Read)
rebuilt        : ReferenceId(22): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_babelHelpers$classPr3":
after transform: ReferenceId(22): ReferenceFlags(Read)
rebuilt        : ReferenceId(25): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(25): ReferenceFlags(Read)
rebuilt        : ReferenceId(28): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_nullish":
after transform: ReferenceId(24): ReferenceFlags(Read)
rebuilt        : ReferenceId(29): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_babelHelpers$classPr4":
after transform: ReferenceId(28): ReferenceFlags(Read)
rebuilt        : ReferenceId(31): ReferenceFlags(Read | MemberWriteTarget)

* private-loose/multiple/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)

* private-loose/native-classes/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(21): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(22): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Foo":
after transform: ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(5): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(16): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(19): ReferenceFlags(Read)
rebuilt        : ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Foo":
after transform: ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(14): ReferenceFlags(Read)

* private-loose/nested-class/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)

* private-loose/nested-class-computed/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)

* private-loose/nested-class-computed-redeclared/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)

* private-loose/nested-class-extends-computed/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_this$foo":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)

* private-loose/nested-class-extends-computed-redeclared/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(15): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(14): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_this$foo":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(12): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(14): ReferenceFlags(Read | MemberWriteTarget)

* private-loose/nested-class-other-redeclared/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(19): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(20): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(12): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(12): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(14): ReferenceFlags(Read)
rebuilt        : ReferenceId(15): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(17): ReferenceFlags(Read)
rebuilt        : ReferenceId(18): ReferenceFlags(Read | MemberWriteTarget)

* private-loose/nested-class-redeclared/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)

* private-loose/non-block-arrow-func/input.mjs
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)

* private-loose/optional-chain-before-member-call/input.js
x Output mismatch

* private-loose/optional-chain-before-member-call-with-transform/input.js
x Output mismatch

* private-loose/optional-chain-before-property/input.js
x Output mismatch

* private-loose/optional-chain-before-property-with-transform/input.js
x Output mismatch

* private-loose/optional-chain-cast-to-boolean/input.js
x Output mismatch

* private-loose/optional-chain-delete-property/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(206): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(207): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Foo":
after transform: ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(6): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(40): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(46): ReferenceFlags(Read)
rebuilt        : ReferenceId(15): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_deep$very$o":
after transform: ReferenceId(43): ReferenceFlags(Read)
rebuilt        : ReferenceId(16): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(19): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(51): ReferenceFlags(Read)
rebuilt        : ReferenceId(21): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(48): ReferenceFlags(Read)
rebuilt        : ReferenceId(22): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(25): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(56): ReferenceFlags(Read)
rebuilt        : ReferenceId(27): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(53): ReferenceFlags(Read)
rebuilt        : ReferenceId(28): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(61): ReferenceFlags(Read)
rebuilt        : ReferenceId(33): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(58): ReferenceFlags(Read)
rebuilt        : ReferenceId(34): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(66): ReferenceFlags(Read)
rebuilt        : ReferenceId(39): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(63): ReferenceFlags(Read)
rebuilt        : ReferenceId(40): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(71): ReferenceFlags(Read)
rebuilt        : ReferenceId(45): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(68): ReferenceFlags(Read)
rebuilt        : ReferenceId(46): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(13): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(49): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(76): ReferenceFlags(Read)
rebuilt        : ReferenceId(51): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(73): ReferenceFlags(Read)
rebuilt        : ReferenceId(52): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(81): ReferenceFlags(Read)
rebuilt        : ReferenceId(58): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(78): ReferenceFlags(Read)
rebuilt        : ReferenceId(59): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(88): ReferenceFlags(Read)
rebuilt        : ReferenceId(65): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(85): ReferenceFlags(Read)
rebuilt        : ReferenceId(66): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(93): ReferenceFlags(Read)
rebuilt        : ReferenceId(71): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(90): ReferenceFlags(Read)
rebuilt        : ReferenceId(72): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(98): ReferenceFlags(Read)
rebuilt        : ReferenceId(77): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(95): ReferenceFlags(Read)
rebuilt        : ReferenceId(78): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(103): ReferenceFlags(Read)
rebuilt        : ReferenceId(84): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(100): ReferenceFlags(Read)
rebuilt        : ReferenceId(85): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(110): ReferenceFlags(Read)
rebuilt        : ReferenceId(91): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(107): ReferenceFlags(Read)
rebuilt        : ReferenceId(92): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(115): ReferenceFlags(Read)
rebuilt        : ReferenceId(97): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(112): ReferenceFlags(Read)
rebuilt        : ReferenceId(98): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(21): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(101): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(120): ReferenceFlags(Read)
rebuilt        : ReferenceId(103): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(117): ReferenceFlags(Read)
rebuilt        : ReferenceId(104): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(126): ReferenceFlags(Read)
rebuilt        : ReferenceId(110): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_fnDeep$very$o":
after transform: ReferenceId(123): ReferenceFlags(Read)
rebuilt        : ReferenceId(111): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(23): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(114): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(131): ReferenceFlags(Read)
rebuilt        : ReferenceId(116): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(128): ReferenceFlags(Read)
rebuilt        : ReferenceId(117): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(24): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(120): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(136): ReferenceFlags(Read)
rebuilt        : ReferenceId(122): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(133): ReferenceFlags(Read)
rebuilt        : ReferenceId(123): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(141): ReferenceFlags(Read)
rebuilt        : ReferenceId(128): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(138): ReferenceFlags(Read)
rebuilt        : ReferenceId(129): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(146): ReferenceFlags(Read)
rebuilt        : ReferenceId(134): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(143): ReferenceFlags(Read)
rebuilt        : ReferenceId(135): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(151): ReferenceFlags(Read)
rebuilt        : ReferenceId(140): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(148): ReferenceFlags(Read)
rebuilt        : ReferenceId(141): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(28): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(144): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(156): ReferenceFlags(Read)
rebuilt        : ReferenceId(146): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(153): ReferenceFlags(Read)
rebuilt        : ReferenceId(147): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(161): ReferenceFlags(Read)
rebuilt        : ReferenceId(153): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(158): ReferenceFlags(Read)
rebuilt        : ReferenceId(154): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(168): ReferenceFlags(Read)
rebuilt        : ReferenceId(160): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(165): ReferenceFlags(Read)
rebuilt        : ReferenceId(161): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(173): ReferenceFlags(Read)
rebuilt        : ReferenceId(166): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(170): ReferenceFlags(Read)
rebuilt        : ReferenceId(167): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(178): ReferenceFlags(Read)
rebuilt        : ReferenceId(172): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(175): ReferenceFlags(Read)
rebuilt        : ReferenceId(173): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(183): ReferenceFlags(Read)
rebuilt        : ReferenceId(179): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(180): ReferenceFlags(Read)
rebuilt        : ReferenceId(180): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(190): ReferenceFlags(Read)
rebuilt        : ReferenceId(186): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(187): ReferenceFlags(Read)
rebuilt        : ReferenceId(187): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(195): ReferenceFlags(Read)
rebuilt        : ReferenceId(192): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(192): ReferenceFlags(Read)
rebuilt        : ReferenceId(193): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(198): ReferenceFlags(Read)
rebuilt        : ReferenceId(198): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(201): ReferenceFlags(Read)
rebuilt        : ReferenceId(201): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Foo":
after transform: ReferenceId(203): ReferenceFlags(Read)
rebuilt        : ReferenceId(205): ReferenceFlags(Read | MemberWriteTarget)

* private-loose/optional-chain-delete-property-with-transform/input.js
x Output mismatch

* private-loose/optional-chain-in-function-param/input.js
x Output mismatch

* private-loose/optional-chain-in-function-param-with-transform/input.js
x Output mismatch

* private-loose/optional-chain-member-optional-call/input.js
x Output mismatch

* private-loose/optional-chain-member-optional-call-spread-arguments/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(19): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "args":
after transform: ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(6): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(14): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "f":
after transform: ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(10): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(17): ReferenceFlags(Read)
rebuilt        : ReferenceId(15): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "f":
after transform: ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(16): ReferenceFlags(Read)

* private-loose/optional-chain-member-optional-call-with-transform/input.js
x Output mismatch

* private-loose/optional-chain-optional-member-call/input.js
x Output mismatch

* private-loose/optional-chain-optional-member-call-with-transform/input.js
x Output mismatch

* private-loose/optional-chain-optional-property/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(438): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(439): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Foo":
after transform: ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(6): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(51): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Foo":
after transform: ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(12): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(56): ReferenceFlags(Read)
rebuilt        : ReferenceId(14): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Foo":
after transform: ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(18): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(61): ReferenceFlags(Read)
rebuilt        : ReferenceId(20): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(67): ReferenceFlags(Read)
rebuilt        : ReferenceId(27): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(73): ReferenceFlags(Read)
rebuilt        : ReferenceId(34): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(79): ReferenceFlags(Read)
rebuilt        : ReferenceId(41): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(85): ReferenceFlags(Read)
rebuilt        : ReferenceId(48): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(91): ReferenceFlags(Read)
rebuilt        : ReferenceId(55): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(97): ReferenceFlags(Read)
rebuilt        : ReferenceId(62): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(15): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(67): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(102): ReferenceFlags(Read)
rebuilt        : ReferenceId(69): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(99): ReferenceFlags(Read)
rebuilt        : ReferenceId(70): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(108): ReferenceFlags(Read)
rebuilt        : ReferenceId(74): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(16): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(79): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(113): ReferenceFlags(Read)
rebuilt        : ReferenceId(81): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(110): ReferenceFlags(Read)
rebuilt        : ReferenceId(82): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(119): ReferenceFlags(Read)
rebuilt        : ReferenceId(86): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(124): ReferenceFlags(Read)
rebuilt        : ReferenceId(93): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(121): ReferenceFlags(Read)
rebuilt        : ReferenceId(94): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(130): ReferenceFlags(Read)
rebuilt        : ReferenceId(98): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(135): ReferenceFlags(Read)
rebuilt        : ReferenceId(105): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(132): ReferenceFlags(Read)
rebuilt        : ReferenceId(106): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(141): ReferenceFlags(Read)
rebuilt        : ReferenceId(110): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(146): ReferenceFlags(Read)
rebuilt        : ReferenceId(117): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(143): ReferenceFlags(Read)
rebuilt        : ReferenceId(118): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(152): ReferenceFlags(Read)
rebuilt        : ReferenceId(122): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(20): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(127): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(157): ReferenceFlags(Read)
rebuilt        : ReferenceId(129): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(154): ReferenceFlags(Read)
rebuilt        : ReferenceId(130): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(163): ReferenceFlags(Read)
rebuilt        : ReferenceId(134): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(168): ReferenceFlags(Read)
rebuilt        : ReferenceId(142): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(165): ReferenceFlags(Read)
rebuilt        : ReferenceId(143): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(176): ReferenceFlags(Read)
rebuilt        : ReferenceId(148): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(181): ReferenceFlags(Read)
rebuilt        : ReferenceId(155): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(178): ReferenceFlags(Read)
rebuilt        : ReferenceId(156): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(187): ReferenceFlags(Read)
rebuilt        : ReferenceId(160): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(192): ReferenceFlags(Read)
rebuilt        : ReferenceId(167): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(189): ReferenceFlags(Read)
rebuilt        : ReferenceId(168): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(198): ReferenceFlags(Read)
rebuilt        : ReferenceId(172): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(203): ReferenceFlags(Read)
rebuilt        : ReferenceId(179): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(200): ReferenceFlags(Read)
rebuilt        : ReferenceId(180): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(209): ReferenceFlags(Read)
rebuilt        : ReferenceId(184): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(214): ReferenceFlags(Read)
rebuilt        : ReferenceId(192): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(211): ReferenceFlags(Read)
rebuilt        : ReferenceId(193): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(222): ReferenceFlags(Read)
rebuilt        : ReferenceId(198): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(227): ReferenceFlags(Read)
rebuilt        : ReferenceId(205): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(224): ReferenceFlags(Read)
rebuilt        : ReferenceId(206): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(233): ReferenceFlags(Read)
rebuilt        : ReferenceId(210): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(238): ReferenceFlags(Read)
rebuilt        : ReferenceId(217): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(235): ReferenceFlags(Read)
rebuilt        : ReferenceId(218): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(244): ReferenceFlags(Read)
rebuilt        : ReferenceId(222): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(250): ReferenceFlags(Read)
rebuilt        : ReferenceId(229): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(256): ReferenceFlags(Read)
rebuilt        : ReferenceId(236): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(262): ReferenceFlags(Read)
rebuilt        : ReferenceId(243): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(268): ReferenceFlags(Read)
rebuilt        : ReferenceId(250): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(274): ReferenceFlags(Read)
rebuilt        : ReferenceId(257): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(280): ReferenceFlags(Read)
rebuilt        : ReferenceId(264): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(34): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(269): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(285): ReferenceFlags(Read)
rebuilt        : ReferenceId(271): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(282): ReferenceFlags(Read)
rebuilt        : ReferenceId(272): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(291): ReferenceFlags(Read)
rebuilt        : ReferenceId(276): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(35): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(281): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(296): ReferenceFlags(Read)
rebuilt        : ReferenceId(283): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(293): ReferenceFlags(Read)
rebuilt        : ReferenceId(284): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(302): ReferenceFlags(Read)
rebuilt        : ReferenceId(288): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(307): ReferenceFlags(Read)
rebuilt        : ReferenceId(295): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(304): ReferenceFlags(Read)
rebuilt        : ReferenceId(296): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(313): ReferenceFlags(Read)
rebuilt        : ReferenceId(300): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(318): ReferenceFlags(Read)
rebuilt        : ReferenceId(307): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(315): ReferenceFlags(Read)
rebuilt        : ReferenceId(308): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(324): ReferenceFlags(Read)
rebuilt        : ReferenceId(312): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(329): ReferenceFlags(Read)
rebuilt        : ReferenceId(319): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(326): ReferenceFlags(Read)
rebuilt        : ReferenceId(320): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(335): ReferenceFlags(Read)
rebuilt        : ReferenceId(324): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(39): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(329): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(340): ReferenceFlags(Read)
rebuilt        : ReferenceId(331): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(337): ReferenceFlags(Read)
rebuilt        : ReferenceId(332): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(346): ReferenceFlags(Read)
rebuilt        : ReferenceId(336): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(351): ReferenceFlags(Read)
rebuilt        : ReferenceId(344): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(348): ReferenceFlags(Read)
rebuilt        : ReferenceId(345): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(359): ReferenceFlags(Read)
rebuilt        : ReferenceId(350): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(364): ReferenceFlags(Read)
rebuilt        : ReferenceId(357): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(361): ReferenceFlags(Read)
rebuilt        : ReferenceId(358): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(370): ReferenceFlags(Read)
rebuilt        : ReferenceId(362): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(375): ReferenceFlags(Read)
rebuilt        : ReferenceId(369): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(372): ReferenceFlags(Read)
rebuilt        : ReferenceId(370): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(381): ReferenceFlags(Read)
rebuilt        : ReferenceId(374): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(386): ReferenceFlags(Read)
rebuilt        : ReferenceId(381): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(383): ReferenceFlags(Read)
rebuilt        : ReferenceId(382): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(392): ReferenceFlags(Read)
rebuilt        : ReferenceId(386): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(397): ReferenceFlags(Read)
rebuilt        : ReferenceId(394): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(394): ReferenceFlags(Read)
rebuilt        : ReferenceId(395): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(405): ReferenceFlags(Read)
rebuilt        : ReferenceId(400): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(410): ReferenceFlags(Read)
rebuilt        : ReferenceId(407): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(407): ReferenceFlags(Read)
rebuilt        : ReferenceId(408): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(416): ReferenceFlags(Read)
rebuilt        : ReferenceId(412): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(421): ReferenceFlags(Read)
rebuilt        : ReferenceId(419): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(418): ReferenceFlags(Read)
rebuilt        : ReferenceId(420): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(427): ReferenceFlags(Read)
rebuilt        : ReferenceId(424): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(430): ReferenceFlags(Read)
rebuilt        : ReferenceId(430): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(433): ReferenceFlags(Read)
rebuilt        : ReferenceId(433): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Foo":
after transform: ReferenceId(435): ReferenceFlags(Read)
rebuilt        : ReferenceId(437): ReferenceFlags(Read | MemberWriteTarget)

* private-loose/optional-chain-optional-property-with-transform/input.js
x Output mismatch

* private-loose/parenthesized-optional-member-call/input.js
x Output mismatch

* private-loose/parenthesized-optional-member-call-with-transform/input.js
x Output mismatch

* private-loose/preserve-comments/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)

* private-loose/private-in-derived/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)

* private-loose/reevaluated/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(54): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(56): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(37): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(40): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(43): ReferenceFlags(Read)
rebuilt        : ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Foo":
after transform: ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(11): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(46): ReferenceFlags(Read)
rebuilt        : ReferenceId(14): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "inst":
after transform: ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(15): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(49): ReferenceFlags(Read)
rebuilt        : ReferenceId(18): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Foo":
after transform: ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(19): ReferenceFlags(Read)
Reference flags mismatch for "Object":
after transform: ReferenceId(52): ReferenceFlags(Read)
rebuilt        : ReferenceId(22): ReferenceFlags(Read | MemberWriteTarget)

* private-loose/reference-in-other-property/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(18): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(19): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(20): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(13): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(16): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(16): ReferenceFlags(Read)
rebuilt        : ReferenceId(18): ReferenceFlags(Read | MemberWriteTarget)

* private-loose/static/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Foo":
after transform: ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(2): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Foo":
after transform: ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(6): ReferenceFlags(Read)
Reference flags mismatch for "Object":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)

* private-loose/static-call/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Foo":
after transform: ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(2): ReferenceFlags(Read)
Reference flags mismatch for "Object":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)

* private-loose/static-class-binding/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)

* private-loose/static-export/input.mjs
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)

* private-loose/static-infer-name/input.js
x Output mismatch

* private-loose/static-inherited/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(22): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "val":
after transform: ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(7): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(14): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Base":
after transform: ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(9): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(17): ReferenceFlags(Read)
rebuilt        : ReferenceId(12): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Base":
after transform: ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(13): ReferenceFlags(Read)
Reference flags mismatch for "Object":
after transform: ReferenceId(20): ReferenceFlags(Read)
rebuilt        : ReferenceId(17): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(29): ReferenceFlags(Read)
rebuilt        : ReferenceId(20): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(24): ReferenceFlags(Read)
rebuilt        : ReferenceId(22): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "val":
after transform: ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(25): ReferenceFlags(Read)
Reference flags mismatch for "Object":
after transform: ReferenceId(27): ReferenceFlags(Read)
rebuilt        : ReferenceId(26): ReferenceFlags(Read | MemberWriteTarget)

* private-loose/static-shadow/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Test":
after transform: ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(4): ReferenceFlags(Read)
Reference flags mismatch for "Object":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)

* private-loose/static-this/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)

* private-loose/static-undefined/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Foo":
after transform: ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(2): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Foo":
after transform: ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(6): ReferenceFlags(Read)
Reference flags mismatch for "Object":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)

* private-loose/super-expression/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)

* private-loose/super-statement/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)

* private-loose/update/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(16): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "other":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "other":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(14): ReferenceFlags(Read | MemberWriteTarget)

* public/assignment/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)

* public/call/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)

* public/class-shadow-builtins/input.mjs
x Output mismatch

* public/computed/input.js
x Output mismatch

* public/computed-toPrimitive/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(25): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "foo":
after transform: ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(5): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(28): ReferenceFlags(Read)
rebuilt        : ReferenceId(15): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "foo":
after transform: ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(16): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(30): ReferenceFlags(Read)
rebuilt        : ReferenceId(20): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(33): ReferenceFlags(Read)
rebuilt        : ReferenceId(32): ReferenceFlags(Read | MemberWriteTarget)

* public/computed-without-block/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)

* public/constructor-collision/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)

* public/delete-super-property/input.js
x Output mismatch

* public/delete-this/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)

* public/derived/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)

* public/derived-multiple-supers/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)

* public/derived-super-in-default-params/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_super":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)

* public/derived-super-in-default-params-complex/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_super":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)

* public/derived-super-in-default-params-in-arrow/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_super":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)

* public/extracted-this/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)

* public/foobar/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)

* public/instance/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)

* public/instance-computed/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)

* public/instance-undefined/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)

* public/native-classes/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)

* public/non-block-arrow-func/input.mjs
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)

* public/numeric/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)

* public/preserve-comments/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)

* public/regression-T2983/input.mjs
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)

* public/regression-T6719/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)

* public/regression-T7364/input.mjs
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)

* public/static/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)

* public/static-class-binding/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)

* public/static-export/input.mjs
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)

* public/static-infer-name/input.js
x Output mismatch

* public/static-super/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(14): ReferenceFlags(Read)
rebuilt        : ReferenceId(12): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(12): ReferenceFlags(Read)
rebuilt        : ReferenceId(14): ReferenceFlags(Read | MemberWriteTarget)

* public/static-this/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)

* public/static-undefined/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)

* public/super-call/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)

* public/super-expression/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)

* public/super-statement/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)

* public/super-with-collision/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)

* public/update/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)

* public-loose/class-shadow-builtins/input.mjs
x Output mismatch

* public-loose/computed/input.js
x Output mismatch

* public-loose/constructor-collision/input.js
Reference flags mismatch for "foo":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Foo":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)

* public-loose/foobar/input.js
Reference flags mismatch for "_this":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)

* public-loose/instance-computed/input.js
Reference flags mismatch for "_x":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)

* public-loose/non-block-arrow-func/input.mjs
Reference flags mismatch for "_App":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)

* public-loose/preserve-comments/input.js
Reference flags mismatch for "C":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)

* public-loose/regression-T2983/input.mjs
Reference flags mismatch for "_Class":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_Class2":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)

* public-loose/regression-T6719/input.js
Reference flags mismatch for "_WithContext":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)

* public-loose/regression-T7364/input.mjs
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)

* public-loose/static/input.js
Reference flags mismatch for "Foo":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)

* public-loose/static-class-binding/input.js
Reference flags mismatch for "A":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "A":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)

* public-loose/static-export/input.mjs
Reference flags mismatch for "MyClass":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "MyClass2":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)

* public-loose/static-infer-name/input.js
x Output mismatch

* public-loose/static-super/input.js
x Output mismatch

* public-loose/static-this/input.js
Reference flags mismatch for "A":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "A":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)

* public-loose/static-undefined/input.js
Reference flags mismatch for "Foo":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)

* public-loose/super-expression/input.js
Reference flags mismatch for "foo":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)

* public-loose/super-with-collision/input.js
Reference flags mismatch for "force":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)

* regression/15098/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)

* regression/6153/input.js
x Output mismatch

* regression/6154/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)

* regression/7371/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(24): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(25): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(28): ReferenceFlags(Read)
rebuilt        : ReferenceId(18): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_super2":
after transform: ReferenceId(29): ReferenceFlags(Read)
rebuilt        : ReferenceId(21): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(31): ReferenceFlags(Read)
rebuilt        : ReferenceId(27): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(36): ReferenceFlags(Read)
rebuilt        : ReferenceId(31): ReferenceFlags(Read | MemberWriteTarget)

* regression/7951/input.mjs
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)

* regression/8110/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)

* regression/8882/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(14): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(15): ReferenceFlags(Read)
rebuilt        : ReferenceId(16): ReferenceFlags(Read | MemberWriteTarget)

* regression/T2983/input.mjs
Reference flags mismatch for "_Class":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_Class2":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)

* regression/T6719/input.js
Reference flags mismatch for "_WithContext":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)

* regression/T7364/input.mjs
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)

* regression/multiple-super-in-termary/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)

* source-maps/private-get/input.js
x Output mismatch

* source-maps/private-set/input.js
x Output mismatch


# babel-plugin-transform-class-static-block (0/24)
* class-static-block/before-static-fields/input.js
x Output mismatch

* class-static-block/class-binding/input.js
x Output mismatch

* class-static-block/class-declaration/input.js
x Output mismatch

* class-static-block/class-inferred-name/input.js
x Output mismatch

* class-static-block/in-class-heritage/input.js
x Output mismatch

* class-static-block/multiple-static-initializers/input.js
x Output mismatch

* class-static-block/name-conflict/input.js
x Output mismatch

* class-static-block/new-target/input.js
x Output mismatch

* class-static-block/preserve-comments/input.js
x Output mismatch

* class-static-block/var-scope/input.js
x Output mismatch

* integration/class-binding/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_Foo":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)

* integration/class-declaration/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_Foo":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_Foo":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)

* integration/in-class-heritage/input.js
x Output mismatch

* integration/multiple-static-initializers/input.js
Reference flags mismatch for "_Foo":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_Foo":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_Foo":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_Foo":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_Foo":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(12): ReferenceFlags(Read | MemberWriteTarget)

* integration/name-conflict/input.js
Reference flags mismatch for "_Foo":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)

* integration/new-target/input.js
x Output mismatch

* integration/preserve-comments/input.js
Reference flags mismatch for "_C":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_C":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_C":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)

* integration-loose/class-binding/input.js
Reference flags mismatch for "Foo":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_Foo":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)

* integration-loose/class-declaration/input.js
Reference flags mismatch for "Foo":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_Foo":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_Foo":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)

* integration-loose/in-class-heritage/input.js
x Output mismatch

* integration-loose/multiple-static-initializers/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(15): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_Foo":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_Foo":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_Foo":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(12): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Foo":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(13): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_Foo":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(14): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_Foo":
after transform: ReferenceId(12): ReferenceFlags(Read)
rebuilt        : ReferenceId(15): ReferenceFlags(Read | MemberWriteTarget)

* integration-loose/name-conflict/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(12): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_Foo":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)

* integration-loose/preserve-comments/input.js
Reference flags mismatch for "_C":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_C":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_C":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)

* integration-loose/super-static-block/input.js
x Output mismatch


# babel-plugin-transform-private-methods (10/148)
* accessors/arguments/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_get_privateFieldValue":
after transform: ReferenceId(14): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(16): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(20): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_set_privateFieldValue":
after transform: ReferenceId(17): ReferenceFlags(Read)
rebuilt        : ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(19): ReferenceFlags(Read)
rebuilt        : ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "newValue":
after transform: ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(13): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(16): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(20): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "newValue":
after transform: ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(22): ReferenceFlags(Read)

* accessors/basic/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_get_privateFieldValue":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(12): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(16): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_set_privateFieldValue":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(15): ReferenceFlags(Read)
rebuilt        : ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "newValue":
after transform: ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(13): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(14): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(16): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "newValue":
after transform: ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(18): ReferenceFlags(Read)

* accessors/class-binding/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)

* accessors/destructuring/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_set_setter":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)

* accessors/get-only-setter/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "newValue":
after transform: ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(9): ReferenceFlags(Read)

* accessors/preserve-comments/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)

* accessors/reassignment/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)

* accessors/set-only-getter/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Cl":
after transform: ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(11): ReferenceFlags(Read)

* accessors/tagged-template/input.js
Scope flags mismatch:
after transform: ScopeId(3): ScopeFlags(StrictMode | Function | Arrow)
rebuilt        : ScopeId(4): ScopeFlags(Function | Arrow)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_get_tag":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Foo":
after transform: ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(6): ReferenceFlags(Read)

* accessors/updates/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_get_privateFieldValue":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(17): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_set_privateFieldValue":
after transform: ReferenceId(14): ReferenceFlags(Read)
rebuilt        : ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(16): ReferenceFlags(Read)
rebuilt        : ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "newValue":
after transform: ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(13): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(19): ReferenceFlags(Read)
rebuilt        : ReferenceId(15): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(23): ReferenceFlags(Read)
rebuilt        : ReferenceId(17): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_set_privateFieldValue":
after transform: ReferenceId(20): ReferenceFlags(Read)
rebuilt        : ReferenceId(18): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(22): ReferenceFlags(Read)
rebuilt        : ReferenceId(19): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_set_privateFieldValue":
after transform: ReferenceId(31): ReferenceFlags(Read)
rebuilt        : ReferenceId(21): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(33): ReferenceFlags(Read)
rebuilt        : ReferenceId(22): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_get_privateFieldValue":
after transform: ReferenceId(24): ReferenceFlags(Read)
rebuilt        : ReferenceId(25): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(26): ReferenceFlags(Read)
rebuilt        : ReferenceId(26): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(42): ReferenceFlags(Read)
rebuilt        : ReferenceId(33): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_get_privateFieldValue":
after transform: ReferenceId(35): ReferenceFlags(Read)
rebuilt        : ReferenceId(36): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(37): ReferenceFlags(Read)
rebuilt        : ReferenceId(37): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(48): ReferenceFlags(Read)
rebuilt        : ReferenceId(41): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_get_privateFieldValue":
after transform: ReferenceId(43): ReferenceFlags(Read)
rebuilt        : ReferenceId(43): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(45): ReferenceFlags(Read)
rebuilt        : ReferenceId(44): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(52): ReferenceFlags(Read)
rebuilt        : ReferenceId(46): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_set_privateFieldValue":
after transform: ReferenceId(49): ReferenceFlags(Read)
rebuilt        : ReferenceId(47): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(51): ReferenceFlags(Read)
rebuilt        : ReferenceId(48): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_get_privateFieldValue":
after transform: ReferenceId(53): ReferenceFlags(Read)
rebuilt        : ReferenceId(50): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(55): ReferenceFlags(Read)
rebuilt        : ReferenceId(51): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_get_privateFieldValue":
after transform: ReferenceId(56): ReferenceFlags(Read)
rebuilt        : ReferenceId(53): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(58): ReferenceFlags(Read)
rebuilt        : ReferenceId(54): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(56): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(58): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "newValue":
after transform: ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(60): ReferenceFlags(Read)

* accessors/updates-bigint/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_get_privateFieldValue":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(17): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_set_privateFieldValue":
after transform: ReferenceId(14): ReferenceFlags(Read)
rebuilt        : ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(16): ReferenceFlags(Read)
rebuilt        : ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "newValue":
after transform: ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(13): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(19): ReferenceFlags(Read)
rebuilt        : ReferenceId(15): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(23): ReferenceFlags(Read)
rebuilt        : ReferenceId(17): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_set_privateFieldValue":
after transform: ReferenceId(20): ReferenceFlags(Read)
rebuilt        : ReferenceId(18): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(22): ReferenceFlags(Read)
rebuilt        : ReferenceId(19): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_set_privateFieldValue":
after transform: ReferenceId(31): ReferenceFlags(Read)
rebuilt        : ReferenceId(21): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(33): ReferenceFlags(Read)
rebuilt        : ReferenceId(22): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_get_privateFieldValue":
after transform: ReferenceId(24): ReferenceFlags(Read)
rebuilt        : ReferenceId(25): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(26): ReferenceFlags(Read)
rebuilt        : ReferenceId(26): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(42): ReferenceFlags(Read)
rebuilt        : ReferenceId(33): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_get_privateFieldValue":
after transform: ReferenceId(35): ReferenceFlags(Read)
rebuilt        : ReferenceId(36): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(37): ReferenceFlags(Read)
rebuilt        : ReferenceId(37): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(48): ReferenceFlags(Read)
rebuilt        : ReferenceId(41): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_get_privateFieldValue":
after transform: ReferenceId(43): ReferenceFlags(Read)
rebuilt        : ReferenceId(43): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(45): ReferenceFlags(Read)
rebuilt        : ReferenceId(44): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(52): ReferenceFlags(Read)
rebuilt        : ReferenceId(46): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_set_privateFieldValue":
after transform: ReferenceId(49): ReferenceFlags(Read)
rebuilt        : ReferenceId(47): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(51): ReferenceFlags(Read)
rebuilt        : ReferenceId(48): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_get_privateFieldValue":
after transform: ReferenceId(53): ReferenceFlags(Read)
rebuilt        : ReferenceId(50): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(55): ReferenceFlags(Read)
rebuilt        : ReferenceId(51): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_get_privateFieldValue":
after transform: ReferenceId(56): ReferenceFlags(Read)
rebuilt        : ReferenceId(53): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(58): ReferenceFlags(Read)
rebuilt        : ReferenceId(54): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(56): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(58): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "newValue":
after transform: ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(60): ReferenceFlags(Read)

* accessors-loose/basic/input.js
x Output mismatch

* accessors-loose/class-binding/input.js
x Output mismatch

* accessors-loose/get-only-setter/input.js
x Output mismatch

* accessors-loose/preserve-comments/input.js
x Output mismatch

* accessors-loose/reassignment/input.js
x Output mismatch

* accessors-loose/set-only-getter/input.js
x Output mismatch

* accessors-loose/updates/input.js
x Output mismatch

* accessors-privateFieldsAsProperties/basic/input.js
x Output mismatch

* accessors-privateFieldsAsProperties/class-binding/input.js
x Output mismatch

* accessors-privateFieldsAsProperties/get-only-setter/input.js
x Output mismatch

* accessors-privateFieldsAsProperties/preserve-comments/input.js
x Output mismatch

* accessors-privateFieldsAsProperties/set-only-getter/input.js
x Output mismatch

* accessors-privateFieldsAsProperties/updates/input.js
x Output mismatch

* accessors-privateFieldsAsSymbols/basic/input.js
x Output mismatch

* accessors-privateFieldsAsSymbols/class-binding/input.js
x Output mismatch

* accessors-privateFieldsAsSymbols/get-only-setter/input.js
x Output mismatch

* accessors-privateFieldsAsSymbols/preserve-comments/input.js
x Output mismatch

* accessors-privateFieldsAsSymbols/set-only-getter/input.js
x Output mismatch

* accessors-privateFieldsAsSymbols/updates/input.js
x Output mismatch

* assumption-constantSuper/private-method-super/input.js
x Output mismatch

* duplicated-names/get-set/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "newValue":
after transform: ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(10): ReferenceFlags(Read)

* duplicated-names/set-get/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "newValue":
after transform: ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(8): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)

* misc/multiple/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_get_getter":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_set_setter":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_get_getset":
after transform: ReferenceId(12): ReferenceFlags(Read)
rebuilt        : ReferenceId(13): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(14): ReferenceFlags(Read)
rebuilt        : ReferenceId(14): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(18): ReferenceFlags(Read)
rebuilt        : ReferenceId(16): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_set_getset":
after transform: ReferenceId(15): ReferenceFlags(Read)
rebuilt        : ReferenceId(17): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(17): ReferenceFlags(Read)
rebuilt        : ReferenceId(18): ReferenceFlags(Read | MemberWriteTarget)

* private-method/assignment/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)

* private-method/async/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)

* private-method/before-fields/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(13): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "x":
after transform: ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(15): ReferenceFlags(Read)

* private-method/class-binding/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)

* private-method/class-expression/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)

* private-method/context/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(14): ReferenceFlags(Read)
rebuilt        : ReferenceId(13): ReferenceFlags(Read | MemberWriteTarget)

* private-method/destructuring/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)

* private-method/exfiltrated/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)

* private-method/generator/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)

* private-method/preserve-comments/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)

* private-method/read-only/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)

* private-method/reassignment/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)

* private-method/super/input.js
x Output mismatch

* private-method/tagged-template/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Foo":
after transform: ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(6): ReferenceFlags(Read)

* private-method-loose/assignment/input.js
x Output mismatch

* private-method-loose/async/input.js
x Output mismatch

* private-method-loose/before-fields/input.js
x Output mismatch

* private-method-loose/class-binding/input.js
x Output mismatch

* private-method-loose/class-expression/input.js
x Output mismatch

* private-method-loose/context/input.js
x Output mismatch

* private-method-loose/exfiltrated/input.js
x Output mismatch

* private-method-loose/generator/input.js
x Output mismatch

* private-method-loose/preserve-comments/input.js
x Output mismatch

* private-method-loose/reassignment/input.js
x Output mismatch

* private-method-loose/super/input.js
x Output mismatch

* private-method-privateFieldsAsProperties/assignment/input.js
x Output mismatch

* private-method-privateFieldsAsProperties/async/input.js
x Output mismatch

* private-method-privateFieldsAsProperties/before-fields/input.js
x Output mismatch

* private-method-privateFieldsAsProperties/class-binding/input.js
x Output mismatch

* private-method-privateFieldsAsProperties/class-expression/input.js
x Output mismatch

* private-method-privateFieldsAsProperties/context/input.js
x Output mismatch

* private-method-privateFieldsAsProperties/exfiltrated/input.js
x Output mismatch

* private-method-privateFieldsAsProperties/generator/input.js
x Output mismatch

* private-method-privateFieldsAsProperties/super/input.js
x Output mismatch

* private-method-privateFieldsAsSymbols/assignment/input.js
x Output mismatch

* private-method-privateFieldsAsSymbols/async/input.js
x Output mismatch

* private-method-privateFieldsAsSymbols/before-fields/input.js
x Output mismatch

* private-method-privateFieldsAsSymbols/class-binding/input.js
x Output mismatch

* private-method-privateFieldsAsSymbols/class-expression/input.js
x Output mismatch

* private-method-privateFieldsAsSymbols/context/input.js
x Output mismatch

* private-method-privateFieldsAsSymbols/exfiltrated/input.js
x Output mismatch

* private-method-privateFieldsAsSymbols/generator/input.js
x Output mismatch

* private-method-privateFieldsAsSymbols/super/input.js
x Output mismatch

* private-static-method/async/input.js

  x TS(1108): A 'return' statement can only be used within a function body.
    ,-[tasks/coverage/babel/packages/babel-plugin-transform-private-methods/test/fixtures/private-static-method/async/input.js:11:1]
 10 | 
 11 | return new Cl().test().then(val => {
    : ^^^^^^
 12 |   expect(val).toBe(2);
    `----


* private-static-method/basic/input.js
Reference flags mismatch for "_privateStaticMethod":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Cl":
after transform: ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(1): ReferenceFlags(Read)
Reference flags mismatch for "_privateStaticMethod":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Cl":
after transform: ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(3): ReferenceFlags(Read)
Reference flags mismatch for "_privateStaticMethod":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Cl":
after transform: ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(5): ReferenceFlags(Read)
Reference flags mismatch for "_privateStaticMethod":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Cl":
after transform: ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(7): ReferenceFlags(Read)
Reference flags mismatch for "_privateStaticMethod":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Cl":
after transform: ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(9): ReferenceFlags(Read)

* private-static-method/class-check/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "checked":
after transform: ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(2): ReferenceFlags(Read)

* private-static-method/class-expression/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)

* private-static-method/generator/input.js
Reference flags mismatch for "_foo":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Cl":
after transform: ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(1): ReferenceFlags(Read)

* private-static-method/read-only/input.js
x Output mismatch

* private-static-method/super/input.js
Reference flags mismatch for "_subStaticPrivateMethod":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Sub":
after transform: ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(2): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)

* private-static-method/tagged-template/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Foo":
after transform: ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(5): ReferenceFlags(Read)

* private-static-method/this/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)

* private-static-method-loose/async/input.js

  x TS(1108): A 'return' statement can only be used within a function body.
    ,-[tasks/coverage/babel/packages/babel-plugin-transform-private-methods/test/fixtures/private-static-method-loose/async/input.js:11:1]
 10 | 
 11 | return new Cl().test().then(val => {
    : ^^^^^^
 12 |   expect(val).toBe(2);
    `----


* private-static-method-loose/basic/input.js
x Output mismatch

* private-static-method-loose/class-check/input.js
x Output mismatch

* private-static-method-loose/class-expression/input.js
x Output mismatch

* private-static-method-loose/exfiltrated/input.js
x Output mismatch

* private-static-method-loose/generator/input.js
x Output mismatch

* private-static-method-loose/preserve-comments/input.js
x Output mismatch

* private-static-method-loose/reassignment/input.js
x Output mismatch

* private-static-method-loose/super/input.js
x Output mismatch

* private-static-method-loose/this/input.js
x Output mismatch

* private-static-method-privateFieldsAsProperties/async/input.js

  x TS(1108): A 'return' statement can only be used within a function body.
    ,-[tasks/coverage/babel/packages/babel-plugin-transform-private-methods/test/fixtures/private-static-method-privateFieldsAsProperties/async/input.js:11:1]
 10 | 
 11 | return new Cl().test().then(val => {
    : ^^^^^^
 12 |   expect(val).toBe(2);
    `----


* private-static-method-privateFieldsAsProperties/basic/input.js
x Output mismatch

* private-static-method-privateFieldsAsProperties/class-check/input.js
x Output mismatch

* private-static-method-privateFieldsAsProperties/class-expression/input.js
x Output mismatch

* private-static-method-privateFieldsAsProperties/exfiltrated/input.js
x Output mismatch

* private-static-method-privateFieldsAsProperties/generator/input.js
x Output mismatch

* private-static-method-privateFieldsAsProperties/reassignment/input.js
x Output mismatch

* private-static-method-privateFieldsAsProperties/super/input.js
x Output mismatch

* private-static-method-privateFieldsAsProperties/this/input.js
x Output mismatch

* private-static-method-privateFieldsAsSymbols/async/input.js

  x TS(1108): A 'return' statement can only be used within a function body.
    ,-[tasks/coverage/babel/packages/babel-plugin-transform-private-methods/test/fixtures/private-static-method-privateFieldsAsSymbols/async/input.js:11:1]
 10 | 
 11 | return new Cl().test().then(val => {
    : ^^^^^^
 12 |   expect(val).toBe(2);
    `----


* private-static-method-privateFieldsAsSymbols/basic/input.js
x Output mismatch

* private-static-method-privateFieldsAsSymbols/class-check/input.js
x Output mismatch

* private-static-method-privateFieldsAsSymbols/class-expression/input.js
x Output mismatch

* private-static-method-privateFieldsAsSymbols/exfiltrated/input.js
x Output mismatch

* private-static-method-privateFieldsAsSymbols/generator/input.js
x Output mismatch

* private-static-method-privateFieldsAsSymbols/reassignment/input.js
x Output mismatch

* private-static-method-privateFieldsAsSymbols/super/input.js
x Output mismatch

* private-static-method-privateFieldsAsSymbols/this/input.js
x Output mismatch

* static-accessors/basic/input.js
Reference flags mismatch for "_get_privateStaticFieldValue":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Cl":
after transform: ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(1): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_set_privateStaticFieldValue":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Cl":
after transform: ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(4): ReferenceFlags(Read)
Reference flags mismatch for "_PRIVATE_STATIC_FIELD":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_PRIVATE_STATIC_FIELD":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)

* static-accessors/destructure-set/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_set_p":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "C":
after transform: ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(2): ReferenceFlags(Read)
Reference flags mismatch for "_q":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)

* static-accessors/get-only-setter/input.js
x Output mismatch

* static-accessors/set-only-getter/input.js
x Output mismatch

* static-accessors/tagged-template/input.js
Scope flags mismatch:
after transform: ScopeId(3): ScopeFlags(StrictMode | Function)
rebuilt        : ScopeId(4): ScopeFlags(Function)
Reference flags mismatch for "_get_tag":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)

* static-accessors/updates/input.js
Reference flags mismatch for "_get_privateFieldValue":
after transform: ReferenceId(27): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Cl":
after transform: ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(1): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(29): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_set_privateFieldValue":
after transform: ReferenceId(28): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Cl":
after transform: ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(4): ReferenceFlags(Read)
Reference flags mismatch for "_privateField":
after transform: ReferenceId(30): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(32): ReferenceFlags(Read)
rebuilt        : ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_set_privateFieldValue":
after transform: ReferenceId(31): ReferenceFlags(Read)
rebuilt        : ReferenceId(12): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Cl":
after transform: ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(13): ReferenceFlags(Read)
Reference flags mismatch for "_set_privateFieldValue":
after transform: ReferenceId(41): ReferenceFlags(Read)
rebuilt        : ReferenceId(14): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(43): ReferenceFlags(Read)
rebuilt        : ReferenceId(15): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_get_privateFieldValue":
after transform: ReferenceId(34): ReferenceFlags(Read)
rebuilt        : ReferenceId(19): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(36): ReferenceFlags(Read)
rebuilt        : ReferenceId(20): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_set_privateFieldValue":
after transform: ReferenceId(51): ReferenceFlags(Read)
rebuilt        : ReferenceId(29): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(53): ReferenceFlags(Read)
rebuilt        : ReferenceId(30): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_get_privateFieldValue":
after transform: ReferenceId(46): ReferenceFlags(Read)
rebuilt        : ReferenceId(34): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(48): ReferenceFlags(Read)
rebuilt        : ReferenceId(35): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_set_privateFieldValue":
after transform: ReferenceId(58): ReferenceFlags(Read)
rebuilt        : ReferenceId(40): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(60): ReferenceFlags(Read)
rebuilt        : ReferenceId(41): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_get_privateFieldValue":
after transform: ReferenceId(55): ReferenceFlags(Read)
rebuilt        : ReferenceId(44): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(57): ReferenceFlags(Read)
rebuilt        : ReferenceId(45): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(62): ReferenceFlags(Read)
rebuilt        : ReferenceId(49): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_set_privateFieldValue":
after transform: ReferenceId(61): ReferenceFlags(Read)
rebuilt        : ReferenceId(50): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Cl":
after transform: ReferenceId(19): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(51): ReferenceFlags(Read)
Reference flags mismatch for "_get_privateFieldValue":
after transform: ReferenceId(63): ReferenceFlags(Read)
rebuilt        : ReferenceId(52): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Cl":
after transform: ReferenceId(20): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(53): ReferenceFlags(Read)
Reference flags mismatch for "_get_privateFieldValue":
after transform: ReferenceId(64): ReferenceFlags(Read)
rebuilt        : ReferenceId(54): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Cl":
after transform: ReferenceId(21): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(55): ReferenceFlags(Read)
Reference flags mismatch for "_privateField":
after transform: ReferenceId(25): ReferenceFlags(Read)
rebuilt        : ReferenceId(59): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_privateField":
after transform: ReferenceId(26): ReferenceFlags(Read)
rebuilt        : ReferenceId(60): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(66): ReferenceFlags(Read)
rebuilt        : ReferenceId(62): ReferenceFlags(Read | MemberWriteTarget)

* static-accessors-loose/basic/input.js
x Output mismatch

* static-accessors-loose/destructure-set/input.js
x Output mismatch

* static-accessors-loose/get-only-setter/input.js
x Output mismatch

* static-accessors-loose/preserve-comments/input.js
x Output mismatch

* static-accessors-loose/set-only-getter/input.js
x Output mismatch

* static-accessors-loose/updates/input.js
x Output mismatch

* static-accessors-privateFieldsAsProperties/basic/input.js
x Output mismatch

* static-accessors-privateFieldsAsProperties/destructure-set/input.js
x Output mismatch

* static-accessors-privateFieldsAsProperties/get-only-setter/input.js
x Output mismatch

* static-accessors-privateFieldsAsProperties/preserve-comments/input.js
x Output mismatch

* static-accessors-privateFieldsAsProperties/set-only-getter/input.js
x Output mismatch

* static-accessors-privateFieldsAsProperties/updates/input.js
x Output mismatch

* static-accessors-privateFieldsAsSymbols/basic/input.js
x Output mismatch

* static-accessors-privateFieldsAsSymbols/destructure-set/input.js
x Output mismatch

* static-accessors-privateFieldsAsSymbols/get-only-setter/input.js
x Output mismatch

* static-accessors-privateFieldsAsSymbols/preserve-comments/input.js
x Output mismatch

* static-accessors-privateFieldsAsSymbols/set-only-getter/input.js
x Output mismatch

* static-accessors-privateFieldsAsSymbols/updates/input.js
x Output mismatch


# babel-plugin-transform-private-property-in-object (0/59)
* assumption-privateFieldsAsProperties/accessor/input.js
x Output mismatch

* assumption-privateFieldsAsProperties/compiled-classes/input.js
x Output mismatch

* assumption-privateFieldsAsProperties/field/input.js
x Output mismatch

* assumption-privateFieldsAsProperties/method/input.js
x Output mismatch

* assumption-privateFieldsAsProperties/nested-class/input.js
x Output mismatch

* assumption-privateFieldsAsProperties/nested-class-other-redeclared/input.js
x Output mismatch

* assumption-privateFieldsAsProperties/nested-class-redeclared/input.js
x Output mismatch

* assumption-privateFieldsAsProperties/static-accessor/input.js
x Output mismatch

* assumption-privateFieldsAsProperties/static-field/input.js
x Output mismatch

* assumption-privateFieldsAsProperties/static-method/input.js
x Output mismatch

* assumption-privateFieldsAsSymbols/accessor/input.js
x Output mismatch

* assumption-privateFieldsAsSymbols/compiled-classes/input.js
x Output mismatch

* assumption-privateFieldsAsSymbols/field/input.js
x Output mismatch

* assumption-privateFieldsAsSymbols/method/input.js
x Output mismatch

* assumption-privateFieldsAsSymbols/nested-class/input.js
x Output mismatch

* assumption-privateFieldsAsSymbols/nested-class-other-redeclared/input.js
x Output mismatch

* assumption-privateFieldsAsSymbols/nested-class-redeclared/input.js
x Output mismatch

* assumption-privateFieldsAsSymbols/static-accessor/input.js
x Output mismatch

* assumption-privateFieldsAsSymbols/static-field/input.js
x Output mismatch

* assumption-privateFieldsAsSymbols/static-method/input.js
x Output mismatch

* private/accessor/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_Foo_brand":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)

* private/field/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_foo":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)

* private/method/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_Foo_brand":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)

* private/native-classes/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_bar":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)

* private/nested-class/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_foo":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_foo":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)

* private/nested-class-other-redeclared/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_foo":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_bar2":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(12): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_foo":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(13): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(12): ReferenceFlags(Read)
rebuilt        : ReferenceId(14): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_bar":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(15): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(14): ReferenceFlags(Read)
rebuilt        : ReferenceId(16): ReferenceFlags(Read | MemberWriteTarget)

* private/nested-class-redeclared/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_foo2":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_foo":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)

* private/static-accessor/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)

* private/static-field/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)

* private/static-method/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)

* private/static-shadow/input.js
x Output mismatch

* private-loose/accessor/input.js
x Output mismatch

* private-loose/field/input.js
x Output mismatch

* private-loose/method/input.js
x Output mismatch

* private-loose/native-classes/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_bar":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)

* private-loose/nested-class/input.js
x Output mismatch

* private-loose/nested-class-other-redeclared/input.js
x Output mismatch

* private-loose/nested-class-redeclared/input.js
x Output mismatch

* private-loose/static-accessor/input.js
x Output mismatch

* private-loose/static-field/input.js
x Output mismatch

* private-loose/static-method/input.js
x Output mismatch

* private-loose/static-shadow/input.js
x Output mismatch

* to-native-fields/accessor/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_Foo_brand":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)

* to-native-fields/class-expression-in-default-param/input.js
x Output mismatch

* to-native-fields/class-expression-instance/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_priv":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)

* to-native-fields/class-expression-static/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)

* to-native-fields/field/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_foo":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)

* to-native-fields/half-constructed-instance/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_F_brand":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_x":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(12): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_y":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(13): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(14): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_F_brand":
after transform: ReferenceId(12): ReferenceFlags(Read)
rebuilt        : ReferenceId(15): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(16): ReferenceFlags(Read | MemberWriteTarget)

* to-native-fields/half-constructed-static/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)

* to-native-fields/method/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_Foo_brand":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)

* to-native-fields/multiple-checks/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_x":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_A_brand":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_x":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_A_brand":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(12): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(13): ReferenceFlags(Read | MemberWriteTarget)

* to-native-fields/nested-class/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_foo":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_foo":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)

* to-native-fields/nested-class-other-redeclared/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_foo":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_bar2":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(12): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_foo":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(13): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(12): ReferenceFlags(Read)
rebuilt        : ReferenceId(14): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_bar":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(15): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(14): ReferenceFlags(Read)
rebuilt        : ReferenceId(16): ReferenceFlags(Read | MemberWriteTarget)

* to-native-fields/nested-class-redeclared/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_foo2":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_foo":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)

* to-native-fields/static-accessor/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)

* to-native-fields/static-field/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)

* to-native-fields/static-method/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)

* to-native-fields/static-shadow/input.js
x Output mismatch

* to-native-fields/static-shadowed-binding/input.js
x Output mismatch


# babel-plugin-transform-logical-assignment-operators (3/6)
* logical-assignment/general-semantics/input.js
Reference flags mismatch for "obj":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "obj":
after transform: ReferenceId(93): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "obj":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "obj":
after transform: ReferenceId(94): ReferenceFlags(Read)
rebuilt        : ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "obj":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(15): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "obj":
after transform: ReferenceId(95): ReferenceFlags(Read)
rebuilt        : ReferenceId(16): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "obj":
after transform: ReferenceId(17): ReferenceFlags(Read)
rebuilt        : ReferenceId(20): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "obj":
after transform: ReferenceId(96): ReferenceFlags(Read)
rebuilt        : ReferenceId(21): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "deep":
after transform: ReferenceId(23): ReferenceFlags(Read)
rebuilt        : ReferenceId(28): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_deep$obj":
after transform: ReferenceId(98): ReferenceFlags(Read)
rebuilt        : ReferenceId(29): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "deep":
after transform: ReferenceId(27): ReferenceFlags(Read)
rebuilt        : ReferenceId(34): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_deep$obj2":
after transform: ReferenceId(100): ReferenceFlags(Read)
rebuilt        : ReferenceId(35): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "deep":
after transform: ReferenceId(31): ReferenceFlags(Read)
rebuilt        : ReferenceId(40): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_deep$obj3":
after transform: ReferenceId(102): ReferenceFlags(Read)
rebuilt        : ReferenceId(41): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "deep":
after transform: ReferenceId(35): ReferenceFlags(Read)
rebuilt        : ReferenceId(46): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_deep$obj4":
after transform: ReferenceId(104): ReferenceFlags(Read)
rebuilt        : ReferenceId(47): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "obj":
after transform: ReferenceId(39): ReferenceFlags(Read)
rebuilt        : ReferenceId(51): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "obj":
after transform: ReferenceId(105): ReferenceFlags(Read)
rebuilt        : ReferenceId(54): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "obj":
after transform: ReferenceId(45): ReferenceFlags(Read)
rebuilt        : ReferenceId(60): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "obj":
after transform: ReferenceId(108): ReferenceFlags(Read)
rebuilt        : ReferenceId(63): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "obj":
after transform: ReferenceId(51): ReferenceFlags(Read)
rebuilt        : ReferenceId(69): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "obj":
after transform: ReferenceId(111): ReferenceFlags(Read)
rebuilt        : ReferenceId(72): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "obj":
after transform: ReferenceId(57): ReferenceFlags(Read)
rebuilt        : ReferenceId(78): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "obj":
after transform: ReferenceId(114): ReferenceFlags(Read)
rebuilt        : ReferenceId(81): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "deep":
after transform: ReferenceId(63): ReferenceFlags(Read)
rebuilt        : ReferenceId(88): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_deep$obj5":
after transform: ReferenceId(118): ReferenceFlags(Read)
rebuilt        : ReferenceId(91): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "deep":
after transform: ReferenceId(71): ReferenceFlags(Read)
rebuilt        : ReferenceId(100): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_deep$obj6":
after transform: ReferenceId(122): ReferenceFlags(Read)
rebuilt        : ReferenceId(103): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "deep":
after transform: ReferenceId(79): ReferenceFlags(Read)
rebuilt        : ReferenceId(112): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_deep$obj7":
after transform: ReferenceId(126): ReferenceFlags(Read)
rebuilt        : ReferenceId(115): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "deep":
after transform: ReferenceId(87): ReferenceFlags(Read)
rebuilt        : ReferenceId(124): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_deep$obj8":
after transform: ReferenceId(130): ReferenceFlags(Read)
rebuilt        : ReferenceId(127): ReferenceFlags(Read | MemberWriteTarget)

* logical-assignment/null-coalescing/input.js
Reference flags mismatch for "obj":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "obj":
after transform: ReferenceId(56): ReferenceFlags(Read)
rebuilt        : ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "obj":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(15): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "obj":
after transform: ReferenceId(60): ReferenceFlags(Read)
rebuilt        : ReferenceId(18): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "deep":
after transform: ReferenceId(18): ReferenceFlags(Read)
rebuilt        : ReferenceId(28): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_deep$obj":
after transform: ReferenceId(65): ReferenceFlags(Read)
rebuilt        : ReferenceId(31): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "deep":
after transform: ReferenceId(22): ReferenceFlags(Read)
rebuilt        : ReferenceId(37): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_deep$obj2":
after transform: ReferenceId(70): ReferenceFlags(Read)
rebuilt        : ReferenceId(40): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "obj":
after transform: ReferenceId(28): ReferenceFlags(Read)
rebuilt        : ReferenceId(47): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "obj":
after transform: ReferenceId(74): ReferenceFlags(Read)
rebuilt        : ReferenceId(52): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "obj":
after transform: ReferenceId(34): ReferenceFlags(Read)
rebuilt        : ReferenceId(59): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "obj":
after transform: ReferenceId(80): ReferenceFlags(Read)
rebuilt        : ReferenceId(64): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "deep":
after transform: ReferenceId(42): ReferenceFlags(Read)
rebuilt        : ReferenceId(74): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_deep$obj3":
after transform: ReferenceId(87): ReferenceFlags(Read)
rebuilt        : ReferenceId(79): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "deep":
after transform: ReferenceId(50): ReferenceFlags(Read)
rebuilt        : ReferenceId(89): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_deep$obj4":
after transform: ReferenceId(94): ReferenceFlags(Read)
rebuilt        : ReferenceId(94): ReferenceFlags(Read | MemberWriteTarget)

* logical-assignment/null-coalescing-without-other/input.js
Reference flags mismatch for "_o":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o2":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)


# babel-plugin-transform-nullish-coalescing-operator (9/24)
* assumption-noDocumentAll/transform/input.js
x Output mismatch

* assumption-noDocumentAll/transform-in-default-destructuring/input.js
x Output mismatch

* assumption-noDocumentAll/transform-in-default-param/input.js
x Output mismatch

* assumption-noDocumentAll/transform-in-function/input.js
x Output mismatch

* assumption-noDocumentAll/transform-static-refs-in-default/input.js
x Output mismatch

* assumption-noDocumentAll/transform-static-refs-in-function/input.js
x Output mismatch

* assumption-pureGetters/logical-assignment/input.js
x Output mismatch

* assumption-pureGetters/logical-assignment-undeclared/input.js
x Output mismatch

* assumption-pureGetters/transform-in-default-param/input.js
x Output mismatch

* assumption-pureGetters/transform-static-refs-in-function/input.js
x Output mismatch

* nullish-coalescing/logical-assignment/input.js
x Output mismatch

* nullish-coalescing/logical-assignment-undeclared/input.js
x Output mismatch

* nullish-coalescing/transform-loose/input.js
x Output mismatch

* nullish-coalescing/transform-static-refs-in-function/input.js
x Output mismatch

* nullish-coalescing/undeclared/input.js
x Output mismatch


# babel-plugin-transform-optional-chaining (1/45)
* assumption-noDocumentAll/assignment/input.js
Reference flags mismatch for "_obj$a":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_obj$b":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_obj$a2":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(14): ReferenceFlags(Read | MemberWriteTarget)

* assumption-noDocumentAll/cast-to-boolean/input.js
Reference flags mismatch for "_o$a$b":
after transform: ReferenceId(32): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$a$b$c":
after transform: ReferenceId(39): ReferenceFlags(Read)
rebuilt        : ReferenceId(17): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$a$b2":
after transform: ReferenceId(44): ReferenceFlags(Read)
rebuilt        : ReferenceId(25): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$a$b3":
after transform: ReferenceId(49): ReferenceFlags(Read)
rebuilt        : ReferenceId(31): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$obj":
after transform: ReferenceId(52): ReferenceFlags(Read)
rebuilt        : ReferenceId(36): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$obj":
after transform: ReferenceId(55): ReferenceFlags(Read)
rebuilt        : ReferenceId(38): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$obj2":
after transform: ReferenceId(58): ReferenceFlags(Read)
rebuilt        : ReferenceId(43): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$obj2":
after transform: ReferenceId(61): ReferenceFlags(Read)
rebuilt        : ReferenceId(45): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$obj3":
after transform: ReferenceId(64): ReferenceFlags(Read)
rebuilt        : ReferenceId(49): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$obj4":
after transform: ReferenceId(67): ReferenceFlags(Read)
rebuilt        : ReferenceId(54): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$obj4":
after transform: ReferenceId(70): ReferenceFlags(Read)
rebuilt        : ReferenceId(56): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$obj5":
after transform: ReferenceId(73): ReferenceFlags(Read)
rebuilt        : ReferenceId(63): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$obj5":
after transform: ReferenceId(76): ReferenceFlags(Read)
rebuilt        : ReferenceId(65): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$obj6":
after transform: ReferenceId(79): ReferenceFlags(Read)
rebuilt        : ReferenceId(70): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$obj6":
after transform: ReferenceId(82): ReferenceFlags(Read)
rebuilt        : ReferenceId(72): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$a$b4":
after transform: ReferenceId(87): ReferenceFlags(Read)
rebuilt        : ReferenceId(78): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$a":
after transform: ReferenceId(92): ReferenceFlags(Read)
rebuilt        : ReferenceId(84): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$a$b5":
after transform: ReferenceId(97): ReferenceFlags(Read)
rebuilt        : ReferenceId(90): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$a2":
after transform: ReferenceId(102): ReferenceFlags(Read)
rebuilt        : ReferenceId(96): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$a$b6":
after transform: ReferenceId(110): ReferenceFlags(Read)
rebuilt        : ReferenceId(103): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$a$b7":
after transform: ReferenceId(115): ReferenceFlags(Read)
rebuilt        : ReferenceId(111): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$a$b8":
after transform: ReferenceId(123): ReferenceFlags(Read)
rebuilt        : ReferenceId(118): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$a$b9":
after transform: ReferenceId(128): ReferenceFlags(Read)
rebuilt        : ReferenceId(126): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$a$b10":
after transform: ReferenceId(136): ReferenceFlags(Read)
rebuilt        : ReferenceId(133): ReferenceFlags(Read | MemberWriteTarget)

* assumption-noDocumentAll/in-function-params/input.js
Reference flags mismatch for "x":
after transform: ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(1): ReferenceFlags(Read)
Reference flags mismatch for "_x":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_a$b":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_a$b":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_a$b2":
after transform: ReferenceId(12): ReferenceFlags(Read)
rebuilt        : ReferenceId(13): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_a$b2":
after transform: ReferenceId(14): ReferenceFlags(Read)
rebuilt        : ReferenceId(14): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_a$b3":
after transform: ReferenceId(17): ReferenceFlags(Read)
rebuilt        : ReferenceId(18): ReferenceFlags(Read | MemberWriteTarget)

* assumption-noDocumentAll/memoize/input.js
Reference flags mismatch for "_foo$bar":
after transform: ReferenceId(27): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_foo$bar$baz":
after transform: ReferenceId(38): ReferenceFlags(Read)
rebuilt        : ReferenceId(25): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_foo$bar$baz2":
after transform: ReferenceId(42): ReferenceFlags(Read)
rebuilt        : ReferenceId(31): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_foo$bar6":
after transform: ReferenceId(45): ReferenceFlags(Read)
rebuilt        : ReferenceId(35): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_foo$bar7":
after transform: ReferenceId(49): ReferenceFlags(Read)
rebuilt        : ReferenceId(41): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_foo$bar8":
after transform: ReferenceId(52): ReferenceFlags(Read)
rebuilt        : ReferenceId(46): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_foo$bar9":
after transform: ReferenceId(55): ReferenceFlags(Read)
rebuilt        : ReferenceId(51): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_foo$bar10":
after transform: ReferenceId(57): ReferenceFlags(Read)
rebuilt        : ReferenceId(56): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_foo$bar10$baz":
after transform: ReferenceId(59): ReferenceFlags(Read)
rebuilt        : ReferenceId(57): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_foo$bar11":
after transform: ReferenceId(63): ReferenceFlags(Read)
rebuilt        : ReferenceId(64): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_foo$bar11$baz":
after transform: ReferenceId(65): ReferenceFlags(Read)
rebuilt        : ReferenceId(65): ReferenceFlags(Read | MemberWriteTarget)

* assumption-noDocumentAll/optional-eval-call/input.js
Reference flags mismatch for "eval":
after transform: ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(14): ReferenceFlags(Read)
Reference flags mismatch for "_eval2":
after transform: ReferenceId(18): ReferenceFlags(Read)
rebuilt        : ReferenceId(15): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_foo$eval":
after transform: ReferenceId(20): ReferenceFlags(Read)
rebuilt        : ReferenceId(18): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_eval$foo":
after transform: ReferenceId(23): ReferenceFlags(Read)
rebuilt        : ReferenceId(23): ReferenceFlags(Read | MemberWriteTarget)

* assumption-noDocumentAll/super-method-call/input.js
Reference flags mismatch for "_super$method":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)

* assumption-pureGetters/function-call/input.js
Reference flags mismatch for "foo":
after transform: ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(25): ReferenceFlags(Read)
Reference flags mismatch for "_foo":
after transform: ReferenceId(29): ReferenceFlags(Read)
rebuilt        : ReferenceId(27): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_foo$bar3":
after transform: ReferenceId(32): ReferenceFlags(Read)
rebuilt        : ReferenceId(31): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_foo$bar4":
after transform: ReferenceId(38): ReferenceFlags(Read)
rebuilt        : ReferenceId(38): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_foo$bar5":
after transform: ReferenceId(43): ReferenceFlags(Read)
rebuilt        : ReferenceId(44): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_foo$bar6":
after transform: ReferenceId(51): ReferenceFlags(Read)
rebuilt        : ReferenceId(53): ReferenceFlags(Read | MemberWriteTarget)

* assumption-pureGetters/memoize/input.js
Reference flags mismatch for "_foo$bar":
after transform: ReferenceId(29): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_foo$bar6":
after transform: ReferenceId(59): ReferenceFlags(Read)
rebuilt        : ReferenceId(54): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_foo$bar7":
after transform: ReferenceId(64): ReferenceFlags(Read)
rebuilt        : ReferenceId(61): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_foo$bar8":
after transform: ReferenceId(67): ReferenceFlags(Read)
rebuilt        : ReferenceId(67): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_foo$bar9":
after transform: ReferenceId(75): ReferenceFlags(Read)
rebuilt        : ReferenceId(77): ReferenceFlags(Read | MemberWriteTarget)

* assumption-pureGetters/super-method-call/input.js
Reference flags mismatch for "_super$method":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)

* general/assignment/input.js
Reference flags mismatch for "_obj$a":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_obj$b":
after transform: ReferenceId(14): ReferenceFlags(Read)
rebuilt        : ReferenceId(14): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_obj$a2":
after transform: ReferenceId(19): ReferenceFlags(Read)
rebuilt        : ReferenceId(21): ReferenceFlags(Read | MemberWriteTarget)

* general/cast-to-boolean/input.js
Reference flags mismatch for "_o$a$b":
after transform: ReferenceId(32): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$a$b$c":
after transform: ReferenceId(39): ReferenceFlags(Read)
rebuilt        : ReferenceId(17): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$a$b2":
after transform: ReferenceId(44): ReferenceFlags(Read)
rebuilt        : ReferenceId(25): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$a$b3":
after transform: ReferenceId(49): ReferenceFlags(Read)
rebuilt        : ReferenceId(31): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$obj":
after transform: ReferenceId(52): ReferenceFlags(Read)
rebuilt        : ReferenceId(36): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$obj":
after transform: ReferenceId(55): ReferenceFlags(Read)
rebuilt        : ReferenceId(38): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$obj2":
after transform: ReferenceId(58): ReferenceFlags(Read)
rebuilt        : ReferenceId(43): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$obj2":
after transform: ReferenceId(61): ReferenceFlags(Read)
rebuilt        : ReferenceId(45): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$obj3":
after transform: ReferenceId(64): ReferenceFlags(Read)
rebuilt        : ReferenceId(49): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$obj4":
after transform: ReferenceId(67): ReferenceFlags(Read)
rebuilt        : ReferenceId(54): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$obj4":
after transform: ReferenceId(70): ReferenceFlags(Read)
rebuilt        : ReferenceId(56): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$obj5":
after transform: ReferenceId(73): ReferenceFlags(Read)
rebuilt        : ReferenceId(63): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$obj5":
after transform: ReferenceId(76): ReferenceFlags(Read)
rebuilt        : ReferenceId(65): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$obj6":
after transform: ReferenceId(79): ReferenceFlags(Read)
rebuilt        : ReferenceId(70): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$obj6":
after transform: ReferenceId(82): ReferenceFlags(Read)
rebuilt        : ReferenceId(72): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$a$b4":
after transform: ReferenceId(87): ReferenceFlags(Read)
rebuilt        : ReferenceId(78): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$a":
after transform: ReferenceId(92): ReferenceFlags(Read)
rebuilt        : ReferenceId(84): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$a$b5":
after transform: ReferenceId(97): ReferenceFlags(Read)
rebuilt        : ReferenceId(90): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$a2":
after transform: ReferenceId(102): ReferenceFlags(Read)
rebuilt        : ReferenceId(96): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$a$b6":
after transform: ReferenceId(110): ReferenceFlags(Read)
rebuilt        : ReferenceId(103): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$a$b7":
after transform: ReferenceId(115): ReferenceFlags(Read)
rebuilt        : ReferenceId(111): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$a$b8":
after transform: ReferenceId(123): ReferenceFlags(Read)
rebuilt        : ReferenceId(118): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$a$b9":
after transform: ReferenceId(128): ReferenceFlags(Read)
rebuilt        : ReferenceId(126): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$a$b10":
after transform: ReferenceId(136): ReferenceFlags(Read)
rebuilt        : ReferenceId(133): ReferenceFlags(Read | MemberWriteTarget)

* general/containers/input.js
Reference flags mismatch for "_user$address":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_user$address2":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "a":
after transform: ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(11): ReferenceFlags(Read)
Reference flags mismatch for "_a":
after transform: ReferenceId(14): ReferenceFlags(Read)
rebuilt        : ReferenceId(13): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "a":
after transform: ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(16): ReferenceFlags(Read)
Reference flags mismatch for "_a2":
after transform: ReferenceId(17): ReferenceFlags(Read)
rebuilt        : ReferenceId(18): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "a":
after transform: ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(20): ReferenceFlags(Read)
Reference flags mismatch for "_a3":
after transform: ReferenceId(20): ReferenceFlags(Read)
rebuilt        : ReferenceId(22): ReferenceFlags(Read | MemberWriteTarget)

* general/delete/input.js
Reference flags mismatch for "_obj$a":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_obj$b":
after transform: ReferenceId(15): ReferenceFlags(Read)
rebuilt        : ReferenceId(16): ReferenceFlags(Read | MemberWriteTarget)

* general/delete-in-function-params/input.js
Reference flags mismatch for "a":
after transform: ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(1): ReferenceFlags(Read)
Reference flags mismatch for "_a":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)

* general/function-call/input.js
Reference flags mismatch for "foo":
after transform: ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(6): ReferenceFlags(Read)
Reference flags mismatch for "_foo2":
after transform: ReferenceId(16): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "foo":
after transform: ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(11): ReferenceFlags(Read)
Reference flags mismatch for "_foo$bar":
after transform: ReferenceId(20): ReferenceFlags(Read)
rebuilt        : ReferenceId(13): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "foo":
after transform: ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(17): ReferenceFlags(Read)
Reference flags mismatch for "_foo4":
after transform: ReferenceId(24): ReferenceFlags(Read)
rebuilt        : ReferenceId(20): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_foo4$bar":
after transform: ReferenceId(27): ReferenceFlags(Read)
rebuilt        : ReferenceId(22): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "foo":
after transform: ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(26): ReferenceFlags(Read)
Reference flags mismatch for "_foo5":
after transform: ReferenceId(31): ReferenceFlags(Read)
rebuilt        : ReferenceId(28): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "foo":
after transform: ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(30): ReferenceFlags(Read)
Reference flags mismatch for "_foo6":
after transform: ReferenceId(37): ReferenceFlags(Read)
rebuilt        : ReferenceId(35): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "foo":
after transform: ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(38): ReferenceFlags(Read)
Reference flags mismatch for "_foo$bar2":
after transform: ReferenceId(41): ReferenceFlags(Read)
rebuilt        : ReferenceId(40): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "foo":
after transform: ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(44): ReferenceFlags(Read)
Reference flags mismatch for "_foo$bar3":
after transform: ReferenceId(46): ReferenceFlags(Read)
rebuilt        : ReferenceId(47): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_foo$bar3":
after transform: ReferenceId(50): ReferenceFlags(Read)
rebuilt        : ReferenceId(50): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "foo":
after transform: ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(52): ReferenceFlags(Read)
Reference flags mismatch for "_foo9":
after transform: ReferenceId(53): ReferenceFlags(Read)
rebuilt        : ReferenceId(55): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_foo9$bar":
after transform: ReferenceId(56): ReferenceFlags(Read)
rebuilt        : ReferenceId(57): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "foo":
after transform: ReferenceId(12): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(60): ReferenceFlags(Read)
Reference flags mismatch for "_foo10":
after transform: ReferenceId(60): ReferenceFlags(Read)
rebuilt        : ReferenceId(63): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_foo10$bar":
after transform: ReferenceId(63): ReferenceFlags(Read)
rebuilt        : ReferenceId(66): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_foo10$bar":
after transform: ReferenceId(67): ReferenceFlags(Read)
rebuilt        : ReferenceId(69): ReferenceFlags(Read | MemberWriteTarget)

* general/function-call-loose/input.js
Reference flags mismatch for "foo":
after transform: ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(6): ReferenceFlags(Read)
Reference flags mismatch for "_foo2":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "foo":
after transform: ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(11): ReferenceFlags(Read)
Reference flags mismatch for "_foo$bar":
after transform: ReferenceId(14): ReferenceFlags(Read)
rebuilt        : ReferenceId(13): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "foo":
after transform: ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(17): ReferenceFlags(Read)
Reference flags mismatch for "_foo4":
after transform: ReferenceId(18): ReferenceFlags(Read)
rebuilt        : ReferenceId(20): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_foo4$bar":
after transform: ReferenceId(21): ReferenceFlags(Read)
rebuilt        : ReferenceId(22): ReferenceFlags(Read | MemberWriteTarget)

* general/function-call-spread/input.js
Reference flags mismatch for "a":
after transform: ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(6): ReferenceFlags(Read)
Reference flags mismatch for "_a2":
after transform: ReferenceId(12): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "a":
after transform: ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(11): ReferenceFlags(Read)
Reference flags mismatch for "_a3":
after transform: ReferenceId(15): ReferenceFlags(Read)
rebuilt        : ReferenceId(13): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "a":
after transform: ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(16): ReferenceFlags(Read)
Reference flags mismatch for "_a4":
after transform: ReferenceId(18): ReferenceFlags(Read)
rebuilt        : ReferenceId(18): ReferenceFlags(Read | MemberWriteTarget)

* general/in-function-params/input.js
Reference flags mismatch for "x":
after transform: ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(1): ReferenceFlags(Read)
Reference flags mismatch for "_x":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_a$b":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_a$b":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(13): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_a$b2":
after transform: ReferenceId(16): ReferenceFlags(Read)
rebuilt        : ReferenceId(18): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_a$b2":
after transform: ReferenceId(19): ReferenceFlags(Read)
rebuilt        : ReferenceId(20): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_a$b3":
after transform: ReferenceId(24): ReferenceFlags(Read)
rebuilt        : ReferenceId(26): ReferenceFlags(Read | MemberWriteTarget)

* general/in-function-params-loose/input.js
Reference flags mismatch for "x":
after transform: ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(1): ReferenceFlags(Read)
Reference flags mismatch for "_x":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_a$b":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_a$b":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(13): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_a$b2":
after transform: ReferenceId(16): ReferenceFlags(Read)
rebuilt        : ReferenceId(18): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_a$b2":
after transform: ReferenceId(19): ReferenceFlags(Read)
rebuilt        : ReferenceId(20): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_a$b3":
after transform: ReferenceId(24): ReferenceFlags(Read)
rebuilt        : ReferenceId(26): ReferenceFlags(Read | MemberWriteTarget)

* general/in-method-key/input.js
Reference flags mismatch for "_x$y":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)

* general/in-method-key-loose/input.js
Reference flags mismatch for "_x$y":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)

* general/in-var-destructuring/input.js
Reference flags mismatch for "x":
after transform: ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(1): ReferenceFlags(Read)
Reference flags mismatch for "_x":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)

* general/member-access/input.js
Reference flags mismatch for "foo":
after transform: ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(1): ReferenceFlags(Read)
Reference flags mismatch for "_foo":
after transform: ReferenceId(12): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "a":
after transform: ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(5): ReferenceFlags(Read)
Reference flags mismatch for "_a":
after transform: ReferenceId(15): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_a":
after transform: ReferenceId(18): ReferenceFlags(Read)
rebuilt        : ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_a$b":
after transform: ReferenceId(21): ReferenceFlags(Read)
rebuilt        : ReferenceId(15): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_a$b":
after transform: ReferenceId(24): ReferenceFlags(Read)
rebuilt        : ReferenceId(17): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_a$b$c":
after transform: ReferenceId(27): ReferenceFlags(Read)
rebuilt        : ReferenceId(22): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_a$b$c":
after transform: ReferenceId(30): ReferenceFlags(Read)
rebuilt        : ReferenceId(24): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "orders":
after transform: ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(26): ReferenceFlags(Read)
Reference flags mismatch for "_orders":
after transform: ReferenceId(33): ReferenceFlags(Read)
rebuilt        : ReferenceId(28): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "orders":
after transform: ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(30): ReferenceFlags(Read)
Reference flags mismatch for "_orders2":
after transform: ReferenceId(36): ReferenceFlags(Read)
rebuilt        : ReferenceId(33): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_orders2":
after transform: ReferenceId(39): ReferenceFlags(Read)
rebuilt        : ReferenceId(35): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "client":
after transform: ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(38): ReferenceFlags(Read)
Reference flags mismatch for "_client":
after transform: ReferenceId(42): ReferenceFlags(Read)
rebuilt        : ReferenceId(40): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_orders$client$key":
after transform: ReferenceId(45): ReferenceFlags(Read)
rebuilt        : ReferenceId(45): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "a":
after transform: ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(47): ReferenceFlags(Read)
Reference flags mismatch for "_a2":
after transform: ReferenceId(48): ReferenceFlags(Read)
rebuilt        : ReferenceId(49): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "a":
after transform: ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(52): ReferenceFlags(Read)
Reference flags mismatch for "_a3":
after transform: ReferenceId(54): ReferenceFlags(Read)
rebuilt        : ReferenceId(54): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_c":
after transform: ReferenceId(51): ReferenceFlags(Read)
rebuilt        : ReferenceId(56): ReferenceFlags(Read | MemberWriteTarget)

* general/memoize/input.js
Reference flags mismatch for "_foo$bar":
after transform: ReferenceId(21): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_foo$bar2":
after transform: ReferenceId(28): ReferenceFlags(Read)
rebuilt        : ReferenceId(19): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_foo$bar3":
after transform: ReferenceId(34): ReferenceFlags(Read)
rebuilt        : ReferenceId(27): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_foo$bar4":
after transform: ReferenceId(38): ReferenceFlags(Read)
rebuilt        : ReferenceId(33): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_foo$bar5":
after transform: ReferenceId(43): ReferenceFlags(Read)
rebuilt        : ReferenceId(40): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_foo$bar6":
after transform: ReferenceId(46): ReferenceFlags(Read)
rebuilt        : ReferenceId(46): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_foo$bar6$baz":
after transform: ReferenceId(49): ReferenceFlags(Read)
rebuilt        : ReferenceId(48): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_foo$bar7":
after transform: ReferenceId(55): ReferenceFlags(Read)
rebuilt        : ReferenceId(57): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_foo$bar7$baz":
after transform: ReferenceId(58): ReferenceFlags(Read)
rebuilt        : ReferenceId(59): ReferenceFlags(Read | MemberWriteTarget)

* general/memoize-loose/input.js
Reference flags mismatch for "_foo$bar":
after transform: ReferenceId(29): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_foo$bar$baz":
after transform: ReferenceId(46): ReferenceFlags(Read)
rebuilt        : ReferenceId(34): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_foo$bar$baz2":
after transform: ReferenceId(51): ReferenceFlags(Read)
rebuilt        : ReferenceId(41): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_foo$bar6":
after transform: ReferenceId(55): ReferenceFlags(Read)
rebuilt        : ReferenceId(46): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_foo$bar7":
after transform: ReferenceId(61): ReferenceFlags(Read)
rebuilt        : ReferenceId(54): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_foo$bar8":
after transform: ReferenceId(65): ReferenceFlags(Read)
rebuilt        : ReferenceId(60): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_foo$bar9":
after transform: ReferenceId(70): ReferenceFlags(Read)
rebuilt        : ReferenceId(67): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_foo$bar10":
after transform: ReferenceId(73): ReferenceFlags(Read)
rebuilt        : ReferenceId(73): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_foo$bar10$baz":
after transform: ReferenceId(76): ReferenceFlags(Read)
rebuilt        : ReferenceId(75): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_foo$bar11":
after transform: ReferenceId(82): ReferenceFlags(Read)
rebuilt        : ReferenceId(84): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_foo$bar11$baz":
after transform: ReferenceId(85): ReferenceFlags(Read)
rebuilt        : ReferenceId(86): ReferenceFlags(Read | MemberWriteTarget)

* general/optional-eval-call/input.js
Reference flags mismatch for "eval":
after transform: ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(19): ReferenceFlags(Read)
Reference flags mismatch for "_eval2":
after transform: ReferenceId(23): ReferenceFlags(Read)
rebuilt        : ReferenceId(21): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_foo$eval":
after transform: ReferenceId(26): ReferenceFlags(Read)
rebuilt        : ReferenceId(25): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_eval$foo":
after transform: ReferenceId(30): ReferenceFlags(Read)
rebuilt        : ReferenceId(31): ReferenceFlags(Read | MemberWriteTarget)

* general/optional-eval-call-loose/input.js
Reference flags mismatch for "eval":
after transform: ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(19): ReferenceFlags(Read)
Reference flags mismatch for "_eval2":
after transform: ReferenceId(23): ReferenceFlags(Read)
rebuilt        : ReferenceId(21): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_foo$eval":
after transform: ReferenceId(26): ReferenceFlags(Read)
rebuilt        : ReferenceId(25): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_eval$foo":
after transform: ReferenceId(30): ReferenceFlags(Read)
rebuilt        : ReferenceId(31): ReferenceFlags(Read | MemberWriteTarget)

* general/parenthesized-expression-containers/input.js
Reference flags mismatch for "_user$address":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_user$address2":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "a":
after transform: ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(11): ReferenceFlags(Read)
Reference flags mismatch for "_a":
after transform: ReferenceId(14): ReferenceFlags(Read)
rebuilt        : ReferenceId(13): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "a":
after transform: ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(16): ReferenceFlags(Read)
Reference flags mismatch for "_a2":
after transform: ReferenceId(17): ReferenceFlags(Read)
rebuilt        : ReferenceId(18): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "a":
after transform: ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(20): ReferenceFlags(Read)
Reference flags mismatch for "_a3":
after transform: ReferenceId(20): ReferenceFlags(Read)
rebuilt        : ReferenceId(22): ReferenceFlags(Read | MemberWriteTarget)

* general/parenthesized-member-call/input.js
Reference flags mismatch for "Foo":
after transform: ReferenceId(16): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Foo":
after transform: ReferenceId(19): ReferenceFlags(Read)
rebuilt        : ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(26): ReferenceFlags(Read)
rebuilt        : ReferenceId(19): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(30): ReferenceFlags(Read)
rebuilt        : ReferenceId(24): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$Foo4":
after transform: ReferenceId(34): ReferenceFlags(Read)
rebuilt        : ReferenceId(33): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$Foo4":
after transform: ReferenceId(37): ReferenceFlags(Read)
rebuilt        : ReferenceId(35): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$Foo$self":
after transform: ReferenceId(41): ReferenceFlags(Read)
rebuilt        : ReferenceId(41): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$Foo$self":
after transform: ReferenceId(44): ReferenceFlags(Read)
rebuilt        : ReferenceId(43): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(46): ReferenceFlags(Read)
Reference flags mismatch for "_fn":
after transform: ReferenceId(48): ReferenceFlags(Read)
rebuilt        : ReferenceId(49): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_fn":
after transform: ReferenceId(51): ReferenceFlags(Read)
rebuilt        : ReferenceId(52): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_fn":
after transform: ReferenceId(54): ReferenceFlags(Read)
rebuilt        : ReferenceId(54): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_fn$Foo$self":
after transform: ReferenceId(60): ReferenceFlags(Read)
rebuilt        : ReferenceId(62): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_fn$Foo$self":
after transform: ReferenceId(63): ReferenceFlags(Read)
rebuilt        : ReferenceId(64): ReferenceFlags(Read | MemberWriteTarget)

* general/parenthesized-member-call-loose/input.js
Reference flags mismatch for "Foo":
after transform: ReferenceId(16): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Foo":
after transform: ReferenceId(19): ReferenceFlags(Read)
rebuilt        : ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(26): ReferenceFlags(Read)
rebuilt        : ReferenceId(19): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "o":
after transform: ReferenceId(30): ReferenceFlags(Read)
rebuilt        : ReferenceId(24): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$Foo4":
after transform: ReferenceId(34): ReferenceFlags(Read)
rebuilt        : ReferenceId(33): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$Foo4":
after transform: ReferenceId(37): ReferenceFlags(Read)
rebuilt        : ReferenceId(35): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$Foo$self":
after transform: ReferenceId(41): ReferenceFlags(Read)
rebuilt        : ReferenceId(41): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$Foo$self":
after transform: ReferenceId(44): ReferenceFlags(Read)
rebuilt        : ReferenceId(43): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(46): ReferenceFlags(Read)
Reference flags mismatch for "_fn":
after transform: ReferenceId(48): ReferenceFlags(Read)
rebuilt        : ReferenceId(49): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_fn":
after transform: ReferenceId(51): ReferenceFlags(Read)
rebuilt        : ReferenceId(52): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_fn":
after transform: ReferenceId(54): ReferenceFlags(Read)
rebuilt        : ReferenceId(54): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_fn$Foo$self":
after transform: ReferenceId(60): ReferenceFlags(Read)
rebuilt        : ReferenceId(62): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_fn$Foo$self":
after transform: ReferenceId(63): ReferenceFlags(Read)
rebuilt        : ReferenceId(64): ReferenceFlags(Read | MemberWriteTarget)

* general/super-method-call/input.js
Reference flags mismatch for "_super$method":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_super$method":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)

* general/super-method-call-loose/input.js
Reference flags mismatch for "_super$method":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_super$method":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)

* general/unary/input.js
Reference flags mismatch for "_obj$a":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_obj$b":
after transform: ReferenceId(16): ReferenceFlags(Read)
rebuilt        : ReferenceId(16): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_obj$b2":
after transform: ReferenceId(21): ReferenceFlags(Read)
rebuilt        : ReferenceId(23): ReferenceFlags(Read | MemberWriteTarget)

* loose/cast-to-boolean/input.js
Reference flags mismatch for "_o$a$b":
after transform: ReferenceId(32): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$a$b$c":
after transform: ReferenceId(39): ReferenceFlags(Read)
rebuilt        : ReferenceId(17): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$a$b2":
after transform: ReferenceId(44): ReferenceFlags(Read)
rebuilt        : ReferenceId(25): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$a$b3":
after transform: ReferenceId(49): ReferenceFlags(Read)
rebuilt        : ReferenceId(31): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$obj":
after transform: ReferenceId(52): ReferenceFlags(Read)
rebuilt        : ReferenceId(36): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$obj":
after transform: ReferenceId(55): ReferenceFlags(Read)
rebuilt        : ReferenceId(38): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$obj2":
after transform: ReferenceId(58): ReferenceFlags(Read)
rebuilt        : ReferenceId(43): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$obj2":
after transform: ReferenceId(61): ReferenceFlags(Read)
rebuilt        : ReferenceId(45): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$obj3":
after transform: ReferenceId(64): ReferenceFlags(Read)
rebuilt        : ReferenceId(49): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$obj4":
after transform: ReferenceId(67): ReferenceFlags(Read)
rebuilt        : ReferenceId(54): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$obj4":
after transform: ReferenceId(70): ReferenceFlags(Read)
rebuilt        : ReferenceId(56): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$obj5":
after transform: ReferenceId(73): ReferenceFlags(Read)
rebuilt        : ReferenceId(63): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$obj5":
after transform: ReferenceId(76): ReferenceFlags(Read)
rebuilt        : ReferenceId(65): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$obj6":
after transform: ReferenceId(79): ReferenceFlags(Read)
rebuilt        : ReferenceId(70): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$obj6":
after transform: ReferenceId(82): ReferenceFlags(Read)
rebuilt        : ReferenceId(72): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$a$b4":
after transform: ReferenceId(87): ReferenceFlags(Read)
rebuilt        : ReferenceId(78): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$a":
after transform: ReferenceId(92): ReferenceFlags(Read)
rebuilt        : ReferenceId(84): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$a$b5":
after transform: ReferenceId(97): ReferenceFlags(Read)
rebuilt        : ReferenceId(90): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$a2":
after transform: ReferenceId(102): ReferenceFlags(Read)
rebuilt        : ReferenceId(96): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$a$b6":
after transform: ReferenceId(110): ReferenceFlags(Read)
rebuilt        : ReferenceId(103): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$a$b7":
after transform: ReferenceId(115): ReferenceFlags(Read)
rebuilt        : ReferenceId(111): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$a$b8":
after transform: ReferenceId(123): ReferenceFlags(Read)
rebuilt        : ReferenceId(118): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$a$b9":
after transform: ReferenceId(128): ReferenceFlags(Read)
rebuilt        : ReferenceId(126): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_o$a$b10":
after transform: ReferenceId(136): ReferenceFlags(Read)
rebuilt        : ReferenceId(133): ReferenceFlags(Read | MemberWriteTarget)

* regression/10959-transform-optional-chaining/input.ts
Reference flags mismatch for "a":
after transform: ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(1): ReferenceFlags(Read)
Reference flags mismatch for "_a":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "a":
after transform: ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(5): ReferenceFlags(Read)
Reference flags mismatch for "_a2":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "a":
after transform: ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(9): ReferenceFlags(Read)
Reference flags mismatch for "_a3":
after transform: ReferenceId(14): ReferenceFlags(Read)
rebuilt        : ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_b":
after transform: ReferenceId(17): ReferenceFlags(Read)
rebuilt        : ReferenceId(15): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "a":
after transform: ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(17): ReferenceFlags(Read)
Reference flags mismatch for "_a4":
after transform: ReferenceId(20): ReferenceFlags(Read)
rebuilt        : ReferenceId(20): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_a4":
after transform: ReferenceId(23): ReferenceFlags(Read)
rebuilt        : ReferenceId(22): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "a":
after transform: ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(24): ReferenceFlags(Read)
Reference flags mismatch for "_a5":
after transform: ReferenceId(26): ReferenceFlags(Read)
rebuilt        : ReferenceId(27): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_a5":
after transform: ReferenceId(29): ReferenceFlags(Read)
rebuilt        : ReferenceId(29): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "a":
after transform: ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(31): ReferenceFlags(Read)
Reference flags mismatch for "_a6":
after transform: ReferenceId(32): ReferenceFlags(Read)
rebuilt        : ReferenceId(33): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "a":
after transform: ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(35): ReferenceFlags(Read)
Reference flags mismatch for "_a7":
after transform: ReferenceId(35): ReferenceFlags(Read)
rebuilt        : ReferenceId(37): ReferenceFlags(Read | MemberWriteTarget)

* regression/10959-transform-ts-and-optional-chaining/input.ts
Reference flags mismatch for "a":
after transform: ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(1): ReferenceFlags(Read)
Reference flags mismatch for "_a":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "a":
after transform: ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(5): ReferenceFlags(Read)
Reference flags mismatch for "_a2":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "a":
after transform: ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(9): ReferenceFlags(Read)
Reference flags mismatch for "_a3":
after transform: ReferenceId(14): ReferenceFlags(Read)
rebuilt        : ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_b":
after transform: ReferenceId(17): ReferenceFlags(Read)
rebuilt        : ReferenceId(15): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "a":
after transform: ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(17): ReferenceFlags(Read)
Reference flags mismatch for "_a4":
after transform: ReferenceId(20): ReferenceFlags(Read)
rebuilt        : ReferenceId(20): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_a4":
after transform: ReferenceId(23): ReferenceFlags(Read)
rebuilt        : ReferenceId(22): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "a":
after transform: ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(24): ReferenceFlags(Read)
Reference flags mismatch for "_a5":
after transform: ReferenceId(26): ReferenceFlags(Read)
rebuilt        : ReferenceId(27): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_a5":
after transform: ReferenceId(29): ReferenceFlags(Read)
rebuilt        : ReferenceId(29): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "a":
after transform: ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(31): ReferenceFlags(Read)
Reference flags mismatch for "_a6":
after transform: ReferenceId(32): ReferenceFlags(Read)
rebuilt        : ReferenceId(33): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "a":
after transform: ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(35): ReferenceFlags(Read)
Reference flags mismatch for "_a7":
after transform: ReferenceId(35): ReferenceFlags(Read)
rebuilt        : ReferenceId(37): ReferenceFlags(Read | MemberWriteTarget)

* regression/15887/input.js
Reference flags mismatch for "_ref":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref2":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)

* regression/7642/input.js
Reference flags mismatch for "_ref":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)

* transparent-expr-wrappers/ts-as-call-context/input.ts
Reference flags mismatch for "a":
after transform: ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(2): ReferenceFlags(Read)
Reference flags mismatch for "_a$b":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)

* transparent-expr-wrappers/ts-as-call-context-in-if/input.ts
Reference flags mismatch for "_a$b":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)

* transparent-expr-wrappers/ts-as-function-call-loose/input.ts
Reference flags mismatch for "foo":
after transform: ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(2): ReferenceFlags(Read)
Reference flags mismatch for "_bar":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Unresolved references mismatch:
after transform: ["A", "B", "foo"]
rebuilt        : ["foo"]

* transparent-expr-wrappers/ts-as-in-conditional/input.ts
Reference flags mismatch for "_a$c":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)

* transparent-expr-wrappers/ts-as-member-expression/input.ts
Reference flags mismatch for "a":
after transform: ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(1): ReferenceFlags(Read)
Reference flags mismatch for "_a":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_a":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Unresolved references mismatch:
after transform: ["ExampleType", "ExampleType2", "a"]
rebuilt        : ["a"]

* transparent-expr-wrappers/ts-parenthesized-expression-member-call/input.ts
Reference flags mismatch for "o":
after transform: ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(1): ReferenceFlags(Read)
Reference flags mismatch for "_o":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Unresolved references mismatch:
after transform: ["ExampleType", "o"]
rebuilt        : ["o"]


# babel-plugin-transform-async-generator-functions (0/20)
* async-generators/class-method/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)

* async-generators/class-private-method/input.js
x Output mismatch

* async-generators/declaration/input.js
Reference flags mismatch for "_agf":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_agf":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)

* async-generators/expression/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_agf":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)

* async-generators/object-method/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)

* async-generators/static-method/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)

* async-generators/transform-class-keys/input.js
Reference flags mismatch for "_fn":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_fn":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)

* async-generators/yield-star/input.js
Reference flags mismatch for "_g":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_g":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)

* for-await/async-arrow/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(17): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_iterator":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_step":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_iterator":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(14): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_iterator":
after transform: ReferenceId(14): ReferenceFlags(Read)
rebuilt        : ReferenceId(15): ReferenceFlags(Read | MemberWriteTarget)

* for-await/async-function/input.js
Reference flags mismatch for "_f":
after transform: ReferenceId(18): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(19): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_iterator":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_step":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_iterator":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(17): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_iterator":
after transform: ReferenceId(14): ReferenceFlags(Read)
rebuilt        : ReferenceId(18): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_f":
after transform: ReferenceId(22): ReferenceFlags(Read)
rebuilt        : ReferenceId(21): ReferenceFlags(Read | MemberWriteTarget)

* for-await/async-function-no-transform/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_iterator":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_step":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_iterator":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_iterator":
after transform: ReferenceId(12): ReferenceFlags(Read)
rebuilt        : ReferenceId(12): ReferenceFlags(Read | MemberWriteTarget)

* for-await/async-generator/input.js
Reference flags mismatch for "_g":
after transform: ReferenceId(20): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(21): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(17): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_iterator":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_step":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_iterator":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(18): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(18): ReferenceFlags(Read)
rebuilt        : ReferenceId(19): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_iterator":
after transform: ReferenceId(14): ReferenceFlags(Read)
rebuilt        : ReferenceId(20): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_g":
after transform: ReferenceId(24): ReferenceFlags(Read)
rebuilt        : ReferenceId(23): ReferenceFlags(Read | MemberWriteTarget)

* for-await/create-async-from-sync-iterator/input.js
Reference flags mismatch for "_fn":
after transform: ReferenceId(19): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(20): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(16): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_iterator":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_step":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_iterator":
after transform: ReferenceId(12): ReferenceFlags(Read)
rebuilt        : ReferenceId(17): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(17): ReferenceFlags(Read)
rebuilt        : ReferenceId(18): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_iterator":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(19): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_fn":
after transform: ReferenceId(23): ReferenceFlags(Read)
rebuilt        : ReferenceId(22): ReferenceFlags(Read | MemberWriteTarget)

* for-await/destructuring/input.js
Reference flags mismatch for "_f":
after transform: ReferenceId(19): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(20): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_iterator":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_step":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_iterator":
after transform: ReferenceId(14): ReferenceFlags(Read)
rebuilt        : ReferenceId(18): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_iterator":
after transform: ReferenceId(15): ReferenceFlags(Read)
rebuilt        : ReferenceId(19): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_f":
after transform: ReferenceId(23): ReferenceFlags(Read)
rebuilt        : ReferenceId(22): ReferenceFlags(Read | MemberWriteTarget)

* for-await/lhs-member-expression/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(16): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_iterator":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_step":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_iterator":
after transform: ReferenceId(12): ReferenceFlags(Read)
rebuilt        : ReferenceId(13): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_iterator":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(14): ReferenceFlags(Read | MemberWriteTarget)

* for-await/re-declare-var-in-init-body/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(14): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_iterator":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_step":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_iterator":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_iterator":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(12): ReferenceFlags(Read | MemberWriteTarget)

* nested/arrows-in-declaration/input.js
Reference flags mismatch for "_g":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_g":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)

* nested/async-in-params/input.js
Reference flags mismatch for "_g":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_g":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)

* nested/generator-in-async/input.js
Reference flags mismatch for "_f":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_g":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_g":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_f":
after transform: ReferenceId(12): ReferenceFlags(Read)
rebuilt        : ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)

* yield-star/create-async-from-sync-iterator/input.js
Reference flags mismatch for "_fn":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_fn":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)


# babel-plugin-transform-object-rest-spread (0/40)
* object-rest/assignment-expression/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(12): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(16): ReferenceFlags(Read)
rebuilt        : ReferenceId(15): ReferenceFlags(Read | MemberWriteTarget)

* object-rest/catch-clause/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(15): ReferenceFlags(Read)
rebuilt        : ReferenceId(14): ReferenceFlags(Read | MemberWriteTarget)

* object-rest/duplicate-decl-bug/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)

* object-rest/export/input.mjs
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)

* object-rest/for-x/input.js
x Output mismatch

* object-rest/for-x-array-pattern/input.js
x Output mismatch

* object-rest/for-x-array-pattern-rest-only/input.js
x Output mismatch

* object-rest/for-x-assignment-shadowed-block-scoped-bindings/input.js
x Output mismatch

* object-rest/for-x-completion-record/input.js
x Output mismatch

* object-rest/for-x-declaration-shadowed-block-scoped-bindings/input.js
x Output mismatch

* object-rest/impure-computed/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(30): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_key":
after transform: ReferenceId(27): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(29): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(38): ReferenceFlags(Read)
rebuilt        : ReferenceId(20): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_key2":
after transform: ReferenceId(33): ReferenceFlags(Read)
rebuilt        : ReferenceId(22): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(37): ReferenceFlags(Read)
rebuilt        : ReferenceId(24): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(45): ReferenceFlags(Read)
rebuilt        : ReferenceId(36): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "key":
after transform: ReferenceId(40): ReferenceFlags(Read)
rebuilt        : ReferenceId(38): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(44): ReferenceFlags(Read)
rebuilt        : ReferenceId(39): ReferenceFlags(Read | MemberWriteTarget)

* object-rest/nested/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)

* object-rest/nested-2/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)

* object-rest/nested-array/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(17): ReferenceFlags(Read)
rebuilt        : ReferenceId(16): ReferenceFlags(Read | MemberWriteTarget)

* object-rest/nested-array-2/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)

* object-rest/nested-computed-key/input.js
Bindings mismatch:
after transform: ScopeId(0): ["_ref3", "a", "c"]
rebuilt        : ScopeId(0): ["a", "c"]
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "_ref3":
after transform: SymbolId(6) "_ref3"
rebuilt        : <None>
Reference symbol mismatch for "_ref3":
after transform: SymbolId(6) "_ref3"
rebuilt        : <None>
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(14): ReferenceFlags(Read)
rebuilt        : ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(12): ReferenceFlags(Read)
rebuilt        : ReferenceId(12): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "_ref3":
after transform: SymbolId(6) "_ref3"
rebuilt        : <None>
Reference symbol mismatch for "_ref3":
after transform: SymbolId(6) "_ref3"
rebuilt        : <None>
Reference symbol mismatch for "_ref3":
after transform: SymbolId(6) "_ref3"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["babelHelpers", "d"]
rebuilt        : ["_ref3", "babelHelpers", "d"]

* object-rest/nested-default-value/input.js
Bindings mismatch:
after transform: ScopeId(0): ["_ref3", "a", "c"]
rebuilt        : ScopeId(0): ["a", "c"]
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "_ref3":
after transform: SymbolId(6) "_ref3"
rebuilt        : <None>
Reference symbol mismatch for "_ref3":
after transform: SymbolId(6) "_ref3"
rebuilt        : <None>
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(14): ReferenceFlags(Read)
rebuilt        : ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(12): ReferenceFlags(Read)
rebuilt        : ReferenceId(12): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "_ref3":
after transform: SymbolId(6) "_ref3"
rebuilt        : <None>
Reference symbol mismatch for "_ref3":
after transform: SymbolId(6) "_ref3"
rebuilt        : <None>
Reference symbol mismatch for "_ref3":
after transform: SymbolId(6) "_ref3"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["babelHelpers", "d"]
rebuilt        : ["_ref3", "babelHelpers", "d"]

* object-rest/nested-literal-property/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)

* object-rest/nested-order/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)

* object-rest/non-string-computed/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(40): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(48): ReferenceFlags(Read)
rebuilt        : ReferenceId(19): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "k1":
after transform: ReferenceId(42): ReferenceFlags(Read)
rebuilt        : ReferenceId(21): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(47): ReferenceFlags(Read)
rebuilt        : ReferenceId(26): ReferenceFlags(Read | MemberWriteTarget)

* object-rest/null-destructuring/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)

* object-rest/object-ref-computed/input.js
x Output mismatch

* object-rest/parameters/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(15): ReferenceFlags(Read)
rebuilt        : ReferenceId(13): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(18): ReferenceFlags(Read)
rebuilt        : ReferenceId(17): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(23): ReferenceFlags(Read)
rebuilt        : ReferenceId(22): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(26): ReferenceFlags(Read)
rebuilt        : ReferenceId(25): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(31): ReferenceFlags(Read)
rebuilt        : ReferenceId(30): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(35): ReferenceFlags(Read)
rebuilt        : ReferenceId(34): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(40): ReferenceFlags(Read)
rebuilt        : ReferenceId(37): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(38): ReferenceFlags(Read)
rebuilt        : ReferenceId(38): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(43): ReferenceFlags(Read)
rebuilt        : ReferenceId(42): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(48): ReferenceFlags(Read)
rebuilt        : ReferenceId(45): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(46): ReferenceFlags(Read)
rebuilt        : ReferenceId(46): ReferenceFlags(Read | MemberWriteTarget)

* object-rest/parameters-object-rest-used-in-default/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(14): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(12): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(17): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(22): ReferenceFlags(Read)
rebuilt        : ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(20): ReferenceFlags(Read)
rebuilt        : ReferenceId(12): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(26): ReferenceFlags(Read)
rebuilt        : ReferenceId(17): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(24): ReferenceFlags(Read)
rebuilt        : ReferenceId(18): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(30): ReferenceFlags(Read)
rebuilt        : ReferenceId(23): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(28): ReferenceFlags(Read)
rebuilt        : ReferenceId(24): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(34): ReferenceFlags(Read)
rebuilt        : ReferenceId(28): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(32): ReferenceFlags(Read)
rebuilt        : ReferenceId(29): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(38): ReferenceFlags(Read)
rebuilt        : ReferenceId(33): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(36): ReferenceFlags(Read)
rebuilt        : ReferenceId(34): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(42): ReferenceFlags(Read)
rebuilt        : ReferenceId(39): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(40): ReferenceFlags(Read)
rebuilt        : ReferenceId(40): ReferenceFlags(Read | MemberWriteTarget)

* object-rest/symbol/input.js
x Output mismatch

* object-rest/template-literal-allLiterals-true-no-hoisting/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)

* object-rest/template-literal-property-allLiterals-false/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)

* object-rest/template-literal-property-allLiterals-true/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)

* object-rest/variable-destructuring/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(36): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "d":
after transform: ReferenceId(34): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(35): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(41): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(39): ReferenceFlags(Read)
rebuilt        : ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(42): ReferenceFlags(Read)
rebuilt        : ReferenceId(13): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(47): ReferenceFlags(Read)
rebuilt        : ReferenceId(17): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(45): ReferenceFlags(Read)
rebuilt        : ReferenceId(18): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(50): ReferenceFlags(Read)
rebuilt        : ReferenceId(24): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(54): ReferenceFlags(Read)
rebuilt        : ReferenceId(29): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(58): ReferenceFlags(Read)
rebuilt        : ReferenceId(34): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(62): ReferenceFlags(Read)
rebuilt        : ReferenceId(38): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(65): ReferenceFlags(Read)
rebuilt        : ReferenceId(43): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(68): ReferenceFlags(Read)
rebuilt        : ReferenceId(46): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(72): ReferenceFlags(Read)
rebuilt        : ReferenceId(51): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(76): ReferenceFlags(Read)
rebuilt        : ReferenceId(56): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(80): ReferenceFlags(Read)
rebuilt        : ReferenceId(61): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(90): ReferenceFlags(Read)
rebuilt        : ReferenceId(75): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(88): ReferenceFlags(Read)
rebuilt        : ReferenceId(76): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(93): ReferenceFlags(Read)
rebuilt        : ReferenceId(82): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(99): ReferenceFlags(Read)
rebuilt        : ReferenceId(87): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(97): ReferenceFlags(Read)
rebuilt        : ReferenceId(88): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(104): ReferenceFlags(Read)
rebuilt        : ReferenceId(96): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(102): ReferenceFlags(Read)
rebuilt        : ReferenceId(97): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(107): ReferenceFlags(Read)
rebuilt        : ReferenceId(103): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(111): ReferenceFlags(Read)
rebuilt        : ReferenceId(108): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(117): ReferenceFlags(Read)
rebuilt        : ReferenceId(112): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(115): ReferenceFlags(Read)
rebuilt        : ReferenceId(113): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(122): ReferenceFlags(Read)
rebuilt        : ReferenceId(118): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(120): ReferenceFlags(Read)
rebuilt        : ReferenceId(119): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(127): ReferenceFlags(Read)
rebuilt        : ReferenceId(124): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(125): ReferenceFlags(Read)
rebuilt        : ReferenceId(125): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(132): ReferenceFlags(Read)
rebuilt        : ReferenceId(129): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(130): ReferenceFlags(Read)
rebuilt        : ReferenceId(130): ReferenceFlags(Read | MemberWriteTarget)

* object-rest/with-array-rest/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)

* object-spread/assignment/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)

* object-spread/expression/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(12): ReferenceFlags(Read)
rebuilt        : ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(12): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(13): ReferenceFlags(Read | MemberWriteTarget)

* object-spread/side-effect/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(12): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(18): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(17): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(16): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(15): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(14): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(20): ReferenceFlags(Read)
rebuilt        : ReferenceId(17): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(19): ReferenceFlags(Read)
rebuilt        : ReferenceId(18): ReferenceFlags(Read | MemberWriteTarget)

* object-spread/variable-declaration/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)

* regression/gh-17274/input.js
x Output mismatch

* regression/gh-4904/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(17): ReferenceFlags(Read)
rebuilt        : ReferenceId(13): ReferenceFlags(Read | MemberWriteTarget)

* regression/gh-5151/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(15): ReferenceFlags(Read)
rebuilt        : ReferenceId(15): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(18): ReferenceFlags(Read)
rebuilt        : ReferenceId(17): ReferenceFlags(Read | MemberWriteTarget)

* regression/gh-7304/input.mjs
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)

* regression/gh-7388/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)

* regression/gh-8323/input.js

  x Option `loose` is not implemented for object-rest-spread.



# babel-plugin-transform-dotall-regex (0/3)
* dotall-regex/simple/input.js
x Output mismatch

* dotall-regex/with-unicode-flag/input.js
x Output mismatch

* dotall-regex/with-unicode-property-escape/input.js
x Output mismatch


# babel-plugin-transform-async-to-generator (1/31)
* assumption-ignoreFunctionLength-true/basic/input.mjs

  x Compiler assumption `ignoreFunctionLength` is not implemented for object-
  | rest-spread.


* assumption-ignoreFunctionLength-true/export-default-function/input.mjs

  x Compiler assumption `ignoreFunctionLength` is not implemented for object-
  | rest-spread.


* assumption-noNewArrows-false/basic/input.js
x Output mismatch

* async-to-generator/async-complex-params/input.js
x Output mismatch

* async-to-generator/async-iife-with-regenerator/input.js
x Output mismatch

* async-to-generator/async-iife-with-regenerator-spec/input.js
x Output mismatch

* async-to-generator/class-method-arity/input.js
x Output mismatch

* async-to-generator/class-method-arity-ignore-length/input.js

  x Compiler assumption `ignoreFunctionLength` is not implemented for object-
  | rest-spread.


* async-to-generator/function-arity/input.js
Reference flags mismatch for "_one":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_one":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_two":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_two":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_three":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(12): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(14): ReferenceFlags(Read)
rebuilt        : ReferenceId(15): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_three":
after transform: ReferenceId(17): ReferenceFlags(Read)
rebuilt        : ReferenceId(16): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_four":
after transform: ReferenceId(19): ReferenceFlags(Read)
rebuilt        : ReferenceId(18): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(20): ReferenceFlags(Read)
rebuilt        : ReferenceId(21): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_four":
after transform: ReferenceId(23): ReferenceFlags(Read)
rebuilt        : ReferenceId(22): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_five":
after transform: ReferenceId(25): ReferenceFlags(Read)
rebuilt        : ReferenceId(24): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(26): ReferenceFlags(Read)
rebuilt        : ReferenceId(27): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_five":
after transform: ReferenceId(29): ReferenceFlags(Read)
rebuilt        : ReferenceId(28): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_six":
after transform: ReferenceId(31): ReferenceFlags(Read)
rebuilt        : ReferenceId(30): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(32): ReferenceFlags(Read)
rebuilt        : ReferenceId(33): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_six":
after transform: ReferenceId(35): ReferenceFlags(Read)
rebuilt        : ReferenceId(34): ReferenceFlags(Read | MemberWriteTarget)

* async-to-generator/object-method-with-super/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_superprop_getMethod":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_superprop_getMethod":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)

* async-to-generator/shadowed-promise/input.js
Reference flags mismatch for "_foo":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_foo":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)

* async-to-generator/shadowed-promise-import/input.mjs
Reference flags mismatch for "_foo":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_foo":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)

* async-to-generator/shadowed-promise-nested/input.js
Reference flags mismatch for "_foo":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_bar":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_bar":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_foo":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(12): ReferenceFlags(Read | MemberWriteTarget)

* bluebird-coroutines/arrow-function/input.js
x Output mismatch

* bluebird-coroutines/class/input.js
x Output mismatch

* bluebird-coroutines/expression/input.js
x Output mismatch

* bluebird-coroutines/named-expression/input.js
x Output mismatch

* bluebird-coroutines/statement/input.js
x Output mismatch

* export-async/default-arrow-export/input.mjs
x Output mismatch

* export-async/default-export/input.mjs
x Output mismatch

* export-async/import-and-export/input.mjs
x Output mismatch

* export-async/lone-export/input.mjs
x Output mismatch

* regression/15978/input.js
x Output mismatch

* regression/4599/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)

* regression/8783/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_poll":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)

* regression/T7108/input.js
x Output mismatch

* regression/T7194/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)

* regression/gh-6923/input.js
x Output mismatch

* regression/in-uncompiled-class-fields/input.js
x Output mismatch

* regression/regression-2765/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref2":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)


# babel-plugin-transform-exponentiation-operator (0/7)
* exponentiation-operator/assignment/input.js
Reference flags mismatch for "Math":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)

* exponentiation-operator/binary/input.js
Reference flags mismatch for "Math":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)

* exponentiation-operator/memoise-object/input.js
x Output mismatch

* exponentiation-operator/memoise-object-in-default-args/input.js
x Output mismatch

* regression/4349/input.js
x Output mismatch

* regression/4349-keep-super/input.js
x Output mismatch

* regression/4403/input.js
x Output mismatch


# babel-plugin-transform-arrow-functions (3/11)
* arrow-functions/implicit-var-arguments/input.js
x Output mismatch

* arrow-functions/self-referential/input.js
x Output mismatch

* arrow-functions/spec/input.js
x Output mismatch

* assumption-newableArrowFunctions-false/basic/input.js
x Output mismatch

* assumption-newableArrowFunctions-false/naming/input.js
x Output mismatch

* assumption-newableArrowFunctions-false/self-referential/input.js
x Output mismatch

* spec/newableArrowFunction-default/input.js
x Output mismatch

* spec/newableArrowFunction-vs-spec-false/input.js
x Output mismatch


# babel-preset-typescript (5/12)
* jsx-compat/js-valid/input.js
Reference flags mismatch for "React":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)

* jsx-compat/tsx-valid/input.tsx
Reference flags mismatch for "React":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)

* node-extensions/import-in-cts/input.cts
x Output mismatch

* node-extensions/type-assertion-in-ts/input.ts
Unresolved references mismatch:
after transform: ["T", "x"]
rebuilt        : ["x"]

* opts/optimizeConstEnums/input.ts
x Output mismatch

* opts/rewriteImportExtensions/input.ts
x Output mismatch

* opts/rewriteImportExtensions-createImportExpressions/input.ts
x Output mismatch


# babel-plugin-transform-typescript (48/157)
* cast/as-expression/input.ts
Unresolved references mismatch:
after transform: ["T", "x"]
rebuilt        : ["x"]

* cast/type-assertion/input.ts
Unresolved references mismatch:
after transform: ["T", "x"]
rebuilt        : ["x"]

* class/accessor-allowDeclareFields-false/input.ts

  x TS(18010): An accessibility modifier cannot be used with a private
  | identifier.
   ,-[tasks/coverage/babel/packages/babel-plugin-transform-typescript/test/fixtures/class/accessor-allowDeclareFields-false/input.ts:8:3]
 7 |   abstract accessor prop6: number;
 8 |   private accessor #p: any;
   :   ^^^^^^^
 9 | 
   `----
  help: Private identifiers are enforced at runtime, while accessibility
        modifiers only affect type checking, so using both is redundant.


  x TS(1243): 'accessor' modifier cannot be used with 'readonly' modifier.
    ,-[tasks/coverage/babel/packages/babel-plugin-transform-typescript/test/fixtures/class/accessor-allowDeclareFields-false/input.ts:14:3]
 13 |   abstract accessor f = 1;
 14 |   readonly accessor g;
    :   ^^^^^^^^
 15 | }
    `----
  help: Allowed modifiers are: private, protected, public, static, abstract,
        override


* class/accessor-allowDeclareFields-true/input.ts

  x TS(18010): An accessibility modifier cannot be used with a private
  | identifier.
   ,-[tasks/coverage/babel/packages/babel-plugin-transform-typescript/test/fixtures/class/accessor-allowDeclareFields-true/input.ts:8:3]
 7 |   abstract accessor prop6: number;
 8 |   private accessor #p: any;
   :   ^^^^^^^
 9 | 
   `----
  help: Private identifiers are enforced at runtime, while accessibility
        modifiers only affect type checking, so using both is redundant.


  x TS(1243): 'accessor' modifier cannot be used with 'readonly' modifier.
    ,-[tasks/coverage/babel/packages/babel-plugin-transform-typescript/test/fixtures/class/accessor-allowDeclareFields-true/input.ts:14:3]
 13 |   abstract accessor f = 1;
 14 |   readonly accessor g;
    :   ^^^^^^^^
 15 | }
    `----
  help: Allowed modifiers are: private, protected, public, static, abstract,
        override


* class/head/input.ts
Unresolved references mismatch:
after transform: ["D", "I"]
rebuilt        : ["D"]

* class/parameter-properties/input.ts
Reference flags mismatch for "x":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "y":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "z":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)

* class/parameter-properties-late-super/input.ts
x Output mismatch

* class/parameter-properties-with-super/input.ts
Reference flags mismatch for "x":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)

* class/private-method-override-transform-private/input.ts
x Output mismatch

* declarations/const-enum/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["E"]
rebuilt        : ScopeId(0): []

* declarations/erased/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["C", "E", "M", "N", "f", "x"]
rebuilt        : ScopeId(0): []

* declarations/export-declare-enum/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["A"]
rebuilt        : ScopeId(0): []

* declarations/nested-namespace/input.mjs
Bindings mismatch:
after transform: ScopeId(0): ["P"]
rebuilt        : ScopeId(0): []

* enum/boolean-value/input.ts
Bindings mismatch:
after transform: ScopeId(1): ["A", "E"]
rebuilt        : ScopeId(1): ["E"]
Scope flags mismatch:
after transform: ScopeId(1): ScopeFlags(0x0)
rebuilt        : ScopeId(1): ScopeFlags(Function)
Symbol flags mismatch for "E":
after transform: SymbolId(0): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(0): SymbolFlags(FunctionScopedVariable)
Reference flags mismatch for "E":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "E":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)

* enum/const/input.ts
Scope flags mismatch:
after transform: ScopeId(1): ScopeFlags(0x0)
rebuilt        : ScopeId(1): ScopeFlags(Function)
Symbol flags mismatch for "E":
after transform: SymbolId(0): SymbolFlags(ConstEnum)
rebuilt        : SymbolId(0): SymbolFlags(FunctionScopedVariable)

* enum/constant-folding/input.ts
Bindings mismatch:
after transform: ScopeId(1): ["E", "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r"]
rebuilt        : ScopeId(1): ["E"]
Scope flags mismatch:
after transform: ScopeId(1): ScopeFlags(0x0)
rebuilt        : ScopeId(1): ScopeFlags(Function)
Symbol flags mismatch for "E":
after transform: SymbolId(0): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(0): SymbolFlags(FunctionScopedVariable)
Reference flags mismatch for "E":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "E":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "E":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "E":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "E":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "E":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "E":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "E":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "E":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "E":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "E":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "E":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "E":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(12): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "E":
after transform: ReferenceId(12): ReferenceFlags(Read)
rebuilt        : ReferenceId(13): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "E":
after transform: ReferenceId(15): ReferenceFlags(Read)
rebuilt        : ReferenceId(14): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "E":
after transform: ReferenceId(14): ReferenceFlags(Read)
rebuilt        : ReferenceId(15): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "E":
after transform: ReferenceId(17): ReferenceFlags(Read)
rebuilt        : ReferenceId(16): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "E":
after transform: ReferenceId(16): ReferenceFlags(Read)
rebuilt        : ReferenceId(17): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "E":
after transform: ReferenceId(19): ReferenceFlags(Read)
rebuilt        : ReferenceId(18): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "E":
after transform: ReferenceId(18): ReferenceFlags(Read)
rebuilt        : ReferenceId(19): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "E":
after transform: ReferenceId(21): ReferenceFlags(Read)
rebuilt        : ReferenceId(20): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "E":
after transform: ReferenceId(20): ReferenceFlags(Read)
rebuilt        : ReferenceId(21): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "E":
after transform: ReferenceId(23): ReferenceFlags(Read)
rebuilt        : ReferenceId(22): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "E":
after transform: ReferenceId(22): ReferenceFlags(Read)
rebuilt        : ReferenceId(23): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "E":
after transform: ReferenceId(25): ReferenceFlags(Read)
rebuilt        : ReferenceId(24): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "E":
after transform: ReferenceId(24): ReferenceFlags(Read)
rebuilt        : ReferenceId(25): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "E":
after transform: ReferenceId(27): ReferenceFlags(Read)
rebuilt        : ReferenceId(26): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "E":
after transform: ReferenceId(26): ReferenceFlags(Read)
rebuilt        : ReferenceId(27): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "E":
after transform: ReferenceId(29): ReferenceFlags(Read)
rebuilt        : ReferenceId(28): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "E":
after transform: ReferenceId(28): ReferenceFlags(Read)
rebuilt        : ReferenceId(29): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "E":
after transform: ReferenceId(31): ReferenceFlags(Read)
rebuilt        : ReferenceId(30): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "E":
after transform: ReferenceId(30): ReferenceFlags(Read)
rebuilt        : ReferenceId(31): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "E":
after transform: ReferenceId(33): ReferenceFlags(Read)
rebuilt        : ReferenceId(32): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "E":
after transform: ReferenceId(32): ReferenceFlags(Read)
rebuilt        : ReferenceId(33): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "E":
after transform: ReferenceId(35): ReferenceFlags(Read)
rebuilt        : ReferenceId(34): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "E":
after transform: ReferenceId(34): ReferenceFlags(Read)
rebuilt        : ReferenceId(35): ReferenceFlags(Read | MemberWriteTarget)

* enum/enum-merging-inner-references/input.ts
Bindings mismatch:
after transform: ScopeId(1): ["Animals", "Cat", "Dog"]
rebuilt        : ScopeId(1): ["Animals"]
Scope flags mismatch:
after transform: ScopeId(1): ScopeFlags(0x0)
rebuilt        : ScopeId(1): ScopeFlags(Function)
Bindings mismatch:
after transform: ScopeId(2): ["Animals", "CatDog"]
rebuilt        : ScopeId(2): ["Animals"]
Scope flags mismatch:
after transform: ScopeId(2): ScopeFlags(0x0)
rebuilt        : ScopeId(2): ScopeFlags(Function)
Symbol flags mismatch for "Animals":
after transform: SymbolId(0): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(0): SymbolFlags(FunctionScopedVariable)
Symbol redeclarations mismatch for "Animals":
after transform: SymbolId(0): [Span { start: 5, end: 12 }, Span { start: 41, end: 48 }]
rebuilt        : SymbolId(0): []
Reference flags mismatch for "Animals":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Animals":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Animals":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Animals":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Animals":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Animals":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Unresolved references mismatch:
after transform: ["Cat", "Dog"]
rebuilt        : []

* enum/enum-merging-inner-references-shadow/input.ts
Bindings mismatch:
after transform: ScopeId(1): ["Animals", "Cat"]
rebuilt        : ScopeId(1): ["Animals"]
Scope flags mismatch:
after transform: ScopeId(1): ScopeFlags(0x0)
rebuilt        : ScopeId(1): ScopeFlags(Function)
Bindings mismatch:
after transform: ScopeId(2): ["Animals", "Dog"]
rebuilt        : ScopeId(2): ["Animals"]
Scope flags mismatch:
after transform: ScopeId(2): ScopeFlags(0x0)
rebuilt        : ScopeId(2): ScopeFlags(Function)
Bindings mismatch:
after transform: ScopeId(3): ["Animals", "CatDog"]
rebuilt        : ScopeId(3): ["Animals"]
Scope flags mismatch:
after transform: ScopeId(3): ScopeFlags(0x0)
rebuilt        : ScopeId(3): ScopeFlags(Function)
Symbol reference IDs mismatch for "Cat":
after transform: SymbolId(0): [ReferenceId(0)]
rebuilt        : SymbolId(0): []
Symbol reference IDs mismatch for "Dog":
after transform: SymbolId(1): [ReferenceId(1)]
rebuilt        : SymbolId(1): []
Symbol flags mismatch for "Animals":
after transform: SymbolId(2): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(2): SymbolFlags(FunctionScopedVariable)
Symbol redeclarations mismatch for "Animals":
after transform: SymbolId(2): [Span { start: 38, end: 45 }, Span { start: 65, end: 72 }, Span { start: 92, end: 99 }]
rebuilt        : SymbolId(2): []
Reference flags mismatch for "Animals":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Animals":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Animals":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Animals":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Animals":
after transform: ReferenceId(12): ReferenceFlags(Read)
rebuilt        : ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Animals":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)

* enum/export/input.ts
Bindings mismatch:
after transform: ScopeId(1): ["A", "E"]
rebuilt        : ScopeId(1): ["E"]
Scope flags mismatch:
after transform: ScopeId(1): ScopeFlags(StrictMode)
rebuilt        : ScopeId(1): ScopeFlags(StrictMode | Function)
Bindings mismatch:
after transform: ScopeId(2): ["B", "E"]
rebuilt        : ScopeId(2): ["E"]
Scope flags mismatch:
after transform: ScopeId(2): ScopeFlags(StrictMode)
rebuilt        : ScopeId(2): ScopeFlags(StrictMode | Function)
Symbol flags mismatch for "E":
after transform: SymbolId(0): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(0): SymbolFlags(BlockScopedVariable)
Symbol redeclarations mismatch for "E":
after transform: SymbolId(0): [Span { start: 12, end: 13 }, Span { start: 40, end: 41 }]
rebuilt        : SymbolId(0): []
Reference flags mismatch for "E":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "E":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "E":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "E":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)

* enum/inferred/input.ts
Bindings mismatch:
after transform: ScopeId(1): ["E", "x", "y"]
rebuilt        : ScopeId(1): ["E"]
Scope flags mismatch:
after transform: ScopeId(1): ScopeFlags(0x0)
rebuilt        : ScopeId(1): ScopeFlags(Function)
Symbol flags mismatch for "E":
after transform: SymbolId(0): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(0): SymbolFlags(FunctionScopedVariable)
Reference flags mismatch for "E":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "E":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "E":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "E":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)

* enum/inner-references/input.ts
Bindings mismatch:
after transform: ScopeId(1): ["E", "a", "b"]
rebuilt        : ScopeId(1): ["E"]
Scope flags mismatch:
after transform: ScopeId(1): ScopeFlags(0x0)
rebuilt        : ScopeId(1): ScopeFlags(Function)
Symbol flags mismatch for "E":
after transform: SymbolId(0): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(0): SymbolFlags(FunctionScopedVariable)
Reference flags mismatch for "E":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "E":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "E":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "E":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)

* enum/mix-references/input.ts
x Output mismatch

* enum/non-constant-member-reference/input.ts
Missing ReferenceId: "Foo"
Missing ReferenceId: "Foo"
Bindings mismatch:
after transform: ScopeId(1): ["A", "B", "C", "D", "Foo"]
rebuilt        : ScopeId(1): ["Foo"]
Scope flags mismatch:
after transform: ScopeId(1): ScopeFlags(0x0)
rebuilt        : ScopeId(1): ScopeFlags(Function)
Symbol flags mismatch for "Foo":
after transform: SymbolId(1): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(1): SymbolFlags(FunctionScopedVariable)
Symbol reference IDs mismatch for "Foo":
after transform: SymbolId(7): [ReferenceId(4), ReferenceId(5), ReferenceId(6), ReferenceId(7), ReferenceId(8), ReferenceId(9), ReferenceId(10), ReferenceId(11), ReferenceId(12)]
rebuilt        : SymbolId(2): [ReferenceId(0), ReferenceId(1), ReferenceId(3), ReferenceId(4), ReferenceId(5), ReferenceId(6), ReferenceId(7), ReferenceId(8), ReferenceId(9), ReferenceId(10), ReferenceId(12)]
Reference flags mismatch for "Foo":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Foo":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Foo":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Foo":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Foo":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Foo":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Foo":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Foo":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)

* enum/non-foldable-constant/input.ts
Bindings mismatch:
after transform: ScopeId(1): ["E", "a", "b"]
rebuilt        : ScopeId(1): ["E"]
Scope flags mismatch:
after transform: ScopeId(1): ScopeFlags(0x0)
rebuilt        : ScopeId(1): ScopeFlags(Function)
Symbol flags mismatch for "E":
after transform: SymbolId(0): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(0): SymbolFlags(FunctionScopedVariable)
Reference flags mismatch for "E":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "E":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "E":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "E":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "E":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)

* enum/non-scoped/input.ts
Bindings mismatch:
after transform: ScopeId(1): ["E", "x", "y"]
rebuilt        : ScopeId(1): ["E"]
Scope flags mismatch:
after transform: ScopeId(1): ScopeFlags(0x0)
rebuilt        : ScopeId(1): ScopeFlags(Function)
Bindings mismatch:
after transform: ScopeId(2): ["E", "z"]
rebuilt        : ScopeId(2): ["E"]
Scope flags mismatch:
after transform: ScopeId(2): ScopeFlags(0x0)
rebuilt        : ScopeId(2): ScopeFlags(Function)
Symbol flags mismatch for "E":
after transform: SymbolId(0): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(0): SymbolFlags(FunctionScopedVariable)
Symbol redeclarations mismatch for "E":
after transform: SymbolId(0): [Span { start: 5, end: 6 }, Span { start: 40, end: 41 }]
rebuilt        : SymbolId(0): []
Reference flags mismatch for "E":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "E":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "E":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "E":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "E":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "E":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)

* enum/outer-references/input.ts
Bindings mismatch:
after transform: ScopeId(1): ["IPC", "SERVER", "SOCKET", "socketType"]
rebuilt        : ScopeId(1): ["socketType"]
Scope flags mismatch:
after transform: ScopeId(1): ScopeFlags(0x0)
rebuilt        : ScopeId(1): ScopeFlags(Function)
Bindings mismatch:
after transform: ScopeId(2): ["IPC", "SERVER", "SOCKET", "UV_READABLE", "UV_WRITABLE", "constants"]
rebuilt        : ScopeId(2): ["constants"]
Scope flags mismatch:
after transform: ScopeId(2): ScopeFlags(0x0)
rebuilt        : ScopeId(2): ScopeFlags(Function)
Symbol flags mismatch for "socketType":
after transform: SymbolId(0): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(0): SymbolFlags(FunctionScopedVariable)
Symbol reference IDs mismatch for "socketType":
after transform: SymbolId(0): [ReferenceId(0), ReferenceId(1), ReferenceId(2), ReferenceId(10)]
rebuilt        : SymbolId(0): [ReferenceId(7)]
Symbol flags mismatch for "constants":
after transform: SymbolId(4): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(2): SymbolFlags(FunctionScopedVariable)
Reference flags mismatch for "socketType":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "socketType":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "socketType":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "socketType":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "socketType":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "socketType":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "constants":
after transform: ReferenceId(12): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "constants":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "constants":
after transform: ReferenceId(14): ReferenceFlags(Read)
rebuilt        : ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "constants":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "constants":
after transform: ReferenceId(16): ReferenceFlags(Read)
rebuilt        : ReferenceId(12): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "constants":
after transform: ReferenceId(15): ReferenceFlags(Read)
rebuilt        : ReferenceId(13): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "constants":
after transform: ReferenceId(18): ReferenceFlags(Read)
rebuilt        : ReferenceId(14): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "constants":
after transform: ReferenceId(17): ReferenceFlags(Read)
rebuilt        : ReferenceId(15): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "constants":
after transform: ReferenceId(20): ReferenceFlags(Read)
rebuilt        : ReferenceId(16): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "constants":
after transform: ReferenceId(19): ReferenceFlags(Read)
rebuilt        : ReferenceId(17): ReferenceFlags(Read | MemberWriteTarget)

* enum/reverse-mappings-syntactically-determinable/input.ts
x Output mismatch

* enum/scoped/input.ts
Scope flags mismatch:
after transform: ScopeId(2): ScopeFlags(0x0)
rebuilt        : ScopeId(2): ScopeFlags(Function)
Symbol flags mismatch for "E":
after transform: SymbolId(0): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(0): SymbolFlags(BlockScopedVariable)

* enum/string-value/input.ts
Bindings mismatch:
after transform: ScopeId(1): ["A", "A2", "B", "B2", "E"]
rebuilt        : ScopeId(1): ["E"]
Scope flags mismatch:
after transform: ScopeId(1): ScopeFlags(0x0)
rebuilt        : ScopeId(1): ScopeFlags(Function)
Symbol flags mismatch for "E":
after transform: SymbolId(0): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(0): SymbolFlags(FunctionScopedVariable)
Reference flags mismatch for "E":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "E":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "E":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "E":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "E":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "E":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)

* enum/string-value-template/input.ts
Bindings mismatch:
after transform: ScopeId(1): ["A", "E"]
rebuilt        : ScopeId(1): ["E"]
Scope flags mismatch:
after transform: ScopeId(1): ScopeFlags(0x0)
rebuilt        : ScopeId(1): ScopeFlags(Function)
Symbol flags mismatch for "E":
after transform: SymbolId(0): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(0): SymbolFlags(FunctionScopedVariable)
Reference flags mismatch for "E":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)

* enum/string-values-computed/input.ts
Bindings mismatch:
after transform: ScopeId(1): ["A", "E"]
rebuilt        : ScopeId(1): ["E"]
Scope flags mismatch:
after transform: ScopeId(1): ScopeFlags(0x0)
rebuilt        : ScopeId(1): ScopeFlags(Function)
Symbol flags mismatch for "E":
after transform: SymbolId(0): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(0): SymbolFlags(FunctionScopedVariable)
Reference flags mismatch for "E":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)

* enum/ts5.0-const-foldable/input.ts
x Output mismatch

* exports/declare-namespace/input.ts
Symbol flags mismatch for "N":
after transform: SymbolId(0): SymbolFlags(Class | NamespaceModule | Ambient)
rebuilt        : SymbolId(0): SymbolFlags(Class)
Symbol reference IDs mismatch for "N":
after transform: SymbolId(0): [ReferenceId(0), ReferenceId(2)]
rebuilt        : SymbolId(0): [ReferenceId(1)]
Symbol redeclarations mismatch for "N":
after transform: SymbolId(0): [Span { start: 13, end: 14 }, Span { start: 83, end: 84 }]
rebuilt        : SymbolId(0): []

* exports/declare-shadowed/input.ts
Symbol flags mismatch for "Signal":
after transform: SymbolId(0): SymbolFlags(Class | Function | Ambient)
rebuilt        : SymbolId(0): SymbolFlags(Function)
Symbol span mismatch for "Signal":
after transform: SymbolId(0): Span { start: 14, end: 20 }
rebuilt        : SymbolId(0): Span { start: 54, end: 60 }
Symbol reference IDs mismatch for "Signal":
after transform: SymbolId(0): [ReferenceId(1), ReferenceId(3)]
rebuilt        : SymbolId(0): [ReferenceId(1)]
Symbol redeclarations mismatch for "Signal":
after transform: SymbolId(0): [Span { start: 14, end: 20 }, Span { start: 54, end: 60 }]
rebuilt        : SymbolId(0): []
Symbol flags mismatch for "Signal2":
after transform: SymbolId(3): SymbolFlags(Class | Function | Ambient)
rebuilt        : SymbolId(2): SymbolFlags(Function)
Symbol reference IDs mismatch for "Signal2":
after transform: SymbolId(3): [ReferenceId(4), ReferenceId(7)]
rebuilt        : SymbolId(2): [ReferenceId(3)]
Symbol redeclarations mismatch for "Signal2":
after transform: SymbolId(3): [Span { start: 147, end: 154 }, Span { start: 225, end: 232 }]
rebuilt        : SymbolId(2): []

* exports/declared-types/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["BB", "BB2", "C", "C2", "E", "N", "f", "foo", "x"]
rebuilt        : ScopeId(0): ["BB", "BB2", "C2", "foo"]
Bindings mismatch:
after transform: ScopeId(11): ["BB", "K"]
rebuilt        : ScopeId(2): ["BB"]
Scope flags mismatch:
after transform: ScopeId(11): ScopeFlags(StrictMode)
rebuilt        : ScopeId(2): ScopeFlags(StrictMode | Function)
Bindings mismatch:
after transform: ScopeId(12): ["BB", "L"]
rebuilt        : ScopeId(3): ["BB"]
Scope flags mismatch:
after transform: ScopeId(12): ScopeFlags(StrictMode)
rebuilt        : ScopeId(3): ScopeFlags(StrictMode | Function)
Scope flags mismatch:
after transform: ScopeId(15): ScopeFlags(StrictMode)
rebuilt        : ScopeId(4): ScopeFlags(StrictMode | Function)
Symbol flags mismatch for "BB":
after transform: SymbolId(10): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(1): SymbolFlags(FunctionScopedVariable)
Symbol redeclarations mismatch for "BB":
after transform: SymbolId(10): [Span { start: 445, end: 447 }, Span { start: 461, end: 463 }]
rebuilt        : SymbolId(1): []
Symbol flags mismatch for "BB2":
after transform: SymbolId(15): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(4): SymbolFlags(FunctionScopedVariable)
Reference flags mismatch for "BB":
after transform: ReferenceId(30): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "BB":
after transform: ReferenceId(29): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "BB":
after transform: ReferenceId(33): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)

* exports/export-const-enums/input.ts
Scope flags mismatch:
after transform: ScopeId(1): ScopeFlags(StrictMode)
rebuilt        : ScopeId(1): ScopeFlags(StrictMode | Function)
Symbol flags mismatch for "None":
after transform: SymbolId(0): SymbolFlags(ConstEnum)
rebuilt        : SymbolId(0): SymbolFlags(FunctionScopedVariable)
Symbol reference IDs mismatch for "None":
after transform: SymbolId(0): [ReferenceId(0), ReferenceId(2)]
rebuilt        : SymbolId(0): [ReferenceId(1)]

* exports/export-import=/input.ts
Reference flags mismatch for "joint":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)

* exports/export=-to-cjs/input.ts
Reference flags mismatch for "module":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)

* exports/imported-types/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["A", "B", "C"]
rebuilt        : ScopeId(0): ["C"]

* exports/imported-types-only-remove-type-imports/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["A", "B", "C"]
rebuilt        : ScopeId(0): ["C"]

* exports/interface/input.ts
x Output mismatch

* exports/issue-9916-1/input.ts
Unresolved references mismatch:
after transform: ["PromiseLike"]
rebuilt        : []

* exports/issue-9916-2/input.ts
Unresolved references mismatch:
after transform: ["PromiseLike"]
rebuilt        : []

* exports/issue-9916-3/input.ts
Unresolved references mismatch:
after transform: ["PromiseLike"]
rebuilt        : []

* exports/type-only-export-specifier-1/input.ts
Symbol reference IDs mismatch for "Foo":
after transform: SymbolId(0): [ReferenceId(0)]
rebuilt        : SymbolId(0): []

* function/overloads/input.ts
Symbol span mismatch for "f":
after transform: SymbolId(0): Span { start: 9, end: 10 }
rebuilt        : SymbolId(0): Span { start: 29, end: 30 }
Symbol redeclarations mismatch for "f":
after transform: SymbolId(0): [Span { start: 9, end: 10 }, Span { start: 29, end: 30 }]
rebuilt        : SymbolId(0): []

* function/overloads-exports/input.mjs
Symbol span mismatch for "f":
after transform: SymbolId(0): Span { start: 9, end: 10 }
rebuilt        : SymbolId(0): Span { start: 29, end: 30 }
Symbol redeclarations mismatch for "f":
after transform: SymbolId(0): [Span { start: 9, end: 10 }, Span { start: 29, end: 30 }]
rebuilt        : SymbolId(0): []

* imports/elide-preact/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["FooBar", "Fragment", "h", "x"]
rebuilt        : ScopeId(0): ["x"]

* imports/elide-preact-no-1/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["Fragment", "h", "render"]
rebuilt        : ScopeId(0): ["Fragment", "h"]

* imports/elide-preact-no-2/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["Fragment", "render"]
rebuilt        : ScopeId(0): ["Fragment"]

* imports/elide-react/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["React", "x"]
rebuilt        : ScopeId(0): ["x"]

* imports/elide-type-referenced-in-imports-equal-no/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["nsa", "nsb"]
rebuilt        : ScopeId(0): []

* imports/elide-typeof/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["A", "x"]
rebuilt        : ScopeId(0): ["x"]

* imports/elision/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["A", "B", "C", "D", "Used", "Used2", "Used3", "x", "y", "z"]
rebuilt        : ScopeId(0): ["Used", "Used2", "Used3", "x", "y", "z"]

* imports/elision-export-type/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["A", "B", "T", "T1"]
rebuilt        : ScopeId(0): ["A", "B"]

* imports/elision-locations/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["A", "B", "C", "Class", "D", "E", "F", "G", "H", "x", "y"]
rebuilt        : ScopeId(0): ["A", "Class", "x", "y"]

* imports/elision-qualifiedname/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["A", "x"]
rebuilt        : ScopeId(0): ["x"]

* imports/elision-rename/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["B", "x"]
rebuilt        : ScopeId(0): ["x"]

* imports/enum-id/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["A", "Enum"]
rebuilt        : ScopeId(0): ["Enum"]
Bindings mismatch:
after transform: ScopeId(1): ["A", "Enum"]
rebuilt        : ScopeId(1): ["Enum"]
Scope flags mismatch:
after transform: ScopeId(1): ScopeFlags(StrictMode)
rebuilt        : ScopeId(1): ScopeFlags(StrictMode | Function)
Symbol flags mismatch for "Enum":
after transform: SymbolId(1): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(0): SymbolFlags(FunctionScopedVariable)
Reference flags mismatch for "Enum":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Enum":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)

* imports/enum-value/input.ts
Bindings mismatch:
after transform: ScopeId(1): ["Enum", "id"]
rebuilt        : ScopeId(1): ["Enum"]
Scope flags mismatch:
after transform: ScopeId(1): ScopeFlags(StrictMode)
rebuilt        : ScopeId(1): ScopeFlags(StrictMode | Function)
Symbol flags mismatch for "Enum":
after transform: SymbolId(1): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(1): SymbolFlags(FunctionScopedVariable)
Reference flags mismatch for "Enum":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Enum":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)

* imports/import-removed-exceptions/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["H", "I", "I2", "J", "a", "b", "c2", "d", "d2", "e", "e4"]
rebuilt        : ScopeId(0): []

* imports/import-type/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["A", "B", "T", "Types"]
rebuilt        : ScopeId(0): []

* imports/import-type-func-with-duplicate-name/input.ts
Symbol flags mismatch for "Foo":
after transform: SymbolId(0): SymbolFlags(Function | TypeImport)
rebuilt        : SymbolId(0): SymbolFlags(Function)
Symbol span mismatch for "Foo":
after transform: SymbolId(0): Span { start: 13, end: 16 }
rebuilt        : SymbolId(0): Span { start: 70, end: 73 }
Symbol redeclarations mismatch for "Foo":
after transform: SymbolId(0): [Span { start: 13, end: 16 }, Span { start: 70, end: 73 }]
rebuilt        : SymbolId(0): []
Symbol flags mismatch for "Foo2":
after transform: SymbolId(1): SymbolFlags(Function | TypeImport)
rebuilt        : SymbolId(1): SymbolFlags(Function)
Symbol span mismatch for "Foo2":
after transform: SymbolId(1): Span { start: 43, end: 47 }
rebuilt        : SymbolId(1): Span { start: 87, end: 91 }
Symbol redeclarations mismatch for "Foo2":
after transform: SymbolId(1): [Span { start: 43, end: 47 }, Span { start: 87, end: 91 }]
rebuilt        : SymbolId(1): []

* imports/import-type-not-removed/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["A", "B"]
rebuilt        : ScopeId(0): []

* imports/only-remove-type-imports/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["H", "I", "I2", "J", "K1", "K2", "L1", "L2", "L3", "a", "b", "c2", "d", "d2", "e", "e4", "x"]
rebuilt        : ScopeId(0): ["L2", "a", "b", "c2", "d", "d2", "e", "e4", "x"]

* imports/property-signature/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["A", "obj"]
rebuilt        : ScopeId(0): ["obj"]

* imports/type-only-export-specifier-1/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["bar", "baz", "foo"]
rebuilt        : ScopeId(0): []

* imports/type-only-export-specifier-2/input.ts
x Output mismatch

* imports/type-only-import-specifier-1/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["Foo1", "Foo2"]
rebuilt        : ScopeId(0): ["Foo1"]

* imports/type-only-import-specifier-2/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["Foo1", "Foo2"]
rebuilt        : ScopeId(0): []

* imports/type-only-import-specifier-3/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["Foo1", "Foo2"]
rebuilt        : ScopeId(0): []

* imports/type-only-import-specifier-4/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["A"]
rebuilt        : ScopeId(0): []

* namespace/alias/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["AliasModule", "LongNameModule", "babel", "bar", "baz", "node", "some", "str"]
rebuilt        : ScopeId(0): ["AliasModule", "bar", "baz", "node", "some", "str"]
Symbol reference IDs mismatch for "AliasModule":
after transform: SymbolId(8): [ReferenceId(2), ReferenceId(3), ReferenceId(4)]
rebuilt        : SymbolId(0): [ReferenceId(1), ReferenceId(2)]
Reference symbol mismatch for "LongNameModule":
after transform: SymbolId(0) "LongNameModule"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["console"]
rebuilt        : ["LongNameModule", "console"]

* namespace/clobber-class/input.ts
Symbol flags mismatch for "A":
after transform: SymbolId(0): SymbolFlags(Class | ValueModule)
rebuilt        : SymbolId(0): SymbolFlags(Class)
Symbol redeclarations mismatch for "A":
after transform: SymbolId(0): [Span { start: 6, end: 7 }, Span { start: 22, end: 23 }]
rebuilt        : SymbolId(0): []
Reference flags mismatch for "_A":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)

* namespace/clobber-enum/input.ts
Bindings mismatch:
after transform: ScopeId(1): ["A", "C"]
rebuilt        : ScopeId(1): ["A"]
Scope flags mismatch:
after transform: ScopeId(1): ScopeFlags(StrictMode)
rebuilt        : ScopeId(1): ScopeFlags(StrictMode | Function)
Symbol flags mismatch for "A":
after transform: SymbolId(0): SymbolFlags(RegularEnum | ValueModule)
rebuilt        : SymbolId(0): SymbolFlags(FunctionScopedVariable)
Symbol redeclarations mismatch for "A":
after transform: SymbolId(0): [Span { start: 5, end: 6 }, Span { start: 30, end: 31 }]
rebuilt        : SymbolId(0): []
Reference flags mismatch for "A":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "A":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_A":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)

* namespace/clobber-export/input.ts
Symbol flags mismatch for "N":
after transform: SymbolId(0): SymbolFlags(Class | ValueModule)
rebuilt        : SymbolId(0): SymbolFlags(Class)
Symbol redeclarations mismatch for "N":
after transform: SymbolId(0): [Span { start: 13, end: 14 }, Span { start: 35, end: 36 }]
rebuilt        : SymbolId(0): []

* namespace/contentious-names/input.ts
Symbol flags mismatch for "N":
after transform: SymbolId(0): SymbolFlags(ValueModule)
rebuilt        : SymbolId(0): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "N":
after transform: SymbolId(0): Span { start: 10, end: 11 }
rebuilt        : SymbolId(0): Span { start: 0, end: 0 }
Symbol flags mismatch for "N":
after transform: SymbolId(1): SymbolFlags(ValueModule)
rebuilt        : SymbolId(2): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "N":
after transform: SymbolId(1): Span { start: 26, end: 27 }
rebuilt        : SymbolId(2): Span { start: 0, end: 0 }
Symbol flags mismatch for "constructor":
after transform: SymbolId(3): SymbolFlags(ValueModule)
rebuilt        : SymbolId(5): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "constructor":
after transform: SymbolId(3): Span { start: 50, end: 61 }
rebuilt        : SymbolId(5): Span { start: 0, end: 0 }
Symbol flags mismatch for "length":
after transform: SymbolId(5): SymbolFlags(ValueModule)
rebuilt        : SymbolId(8): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "length":
after transform: SymbolId(5): Span { start: 84, end: 90 }
rebuilt        : SymbolId(8): Span { start: 0, end: 0 }
Symbol flags mismatch for "concat":
after transform: SymbolId(7): SymbolFlags(ValueModule)
rebuilt        : SymbolId(11): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "concat":
after transform: SymbolId(7): Span { start: 113, end: 119 }
rebuilt        : SymbolId(11): Span { start: 0, end: 0 }
Symbol flags mismatch for "copyWithin":
after transform: SymbolId(9): SymbolFlags(ValueModule)
rebuilt        : SymbolId(14): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "copyWithin":
after transform: SymbolId(9): Span { start: 142, end: 152 }
rebuilt        : SymbolId(14): Span { start: 0, end: 0 }
Symbol flags mismatch for "fill":
after transform: SymbolId(11): SymbolFlags(ValueModule)
rebuilt        : SymbolId(17): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "fill":
after transform: SymbolId(11): Span { start: 175, end: 179 }
rebuilt        : SymbolId(17): Span { start: 0, end: 0 }
Symbol flags mismatch for "find":
after transform: SymbolId(13): SymbolFlags(ValueModule)
rebuilt        : SymbolId(20): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "find":
after transform: SymbolId(13): Span { start: 202, end: 206 }
rebuilt        : SymbolId(20): Span { start: 0, end: 0 }
Symbol flags mismatch for "findIndex":
after transform: SymbolId(15): SymbolFlags(ValueModule)
rebuilt        : SymbolId(23): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "findIndex":
after transform: SymbolId(15): Span { start: 229, end: 238 }
rebuilt        : SymbolId(23): Span { start: 0, end: 0 }
Symbol flags mismatch for "lastIndexOf":
after transform: SymbolId(17): SymbolFlags(ValueModule)
rebuilt        : SymbolId(26): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "lastIndexOf":
after transform: SymbolId(17): Span { start: 261, end: 272 }
rebuilt        : SymbolId(26): Span { start: 0, end: 0 }
Symbol flags mismatch for "pop":
after transform: SymbolId(19): SymbolFlags(ValueModule)
rebuilt        : SymbolId(29): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "pop":
after transform: SymbolId(19): Span { start: 295, end: 298 }
rebuilt        : SymbolId(29): Span { start: 0, end: 0 }
Symbol flags mismatch for "push":
after transform: SymbolId(21): SymbolFlags(ValueModule)
rebuilt        : SymbolId(32): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "push":
after transform: SymbolId(21): Span { start: 321, end: 325 }
rebuilt        : SymbolId(32): Span { start: 0, end: 0 }
Symbol flags mismatch for "reverse":
after transform: SymbolId(23): SymbolFlags(ValueModule)
rebuilt        : SymbolId(35): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "reverse":
after transform: SymbolId(23): Span { start: 348, end: 355 }
rebuilt        : SymbolId(35): Span { start: 0, end: 0 }
Symbol flags mismatch for "shift":
after transform: SymbolId(25): SymbolFlags(ValueModule)
rebuilt        : SymbolId(38): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "shift":
after transform: SymbolId(25): Span { start: 378, end: 383 }
rebuilt        : SymbolId(38): Span { start: 0, end: 0 }
Symbol flags mismatch for "unshift":
after transform: SymbolId(27): SymbolFlags(ValueModule)
rebuilt        : SymbolId(41): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "unshift":
after transform: SymbolId(27): Span { start: 406, end: 413 }
rebuilt        : SymbolId(41): Span { start: 0, end: 0 }
Symbol flags mismatch for "slice":
after transform: SymbolId(29): SymbolFlags(ValueModule)
rebuilt        : SymbolId(44): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "slice":
after transform: SymbolId(29): Span { start: 436, end: 441 }
rebuilt        : SymbolId(44): Span { start: 0, end: 0 }
Symbol flags mismatch for "sort":
after transform: SymbolId(31): SymbolFlags(ValueModule)
rebuilt        : SymbolId(47): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "sort":
after transform: SymbolId(31): Span { start: 464, end: 468 }
rebuilt        : SymbolId(47): Span { start: 0, end: 0 }
Symbol flags mismatch for "splice":
after transform: SymbolId(33): SymbolFlags(ValueModule)
rebuilt        : SymbolId(50): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "splice":
after transform: SymbolId(33): Span { start: 491, end: 497 }
rebuilt        : SymbolId(50): Span { start: 0, end: 0 }
Symbol flags mismatch for "includes":
after transform: SymbolId(35): SymbolFlags(ValueModule)
rebuilt        : SymbolId(53): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "includes":
after transform: SymbolId(35): Span { start: 520, end: 528 }
rebuilt        : SymbolId(53): Span { start: 0, end: 0 }
Symbol flags mismatch for "indexOf":
after transform: SymbolId(37): SymbolFlags(ValueModule)
rebuilt        : SymbolId(56): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "indexOf":
after transform: SymbolId(37): Span { start: 551, end: 558 }
rebuilt        : SymbolId(56): Span { start: 0, end: 0 }
Symbol flags mismatch for "join":
after transform: SymbolId(39): SymbolFlags(ValueModule)
rebuilt        : SymbolId(59): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "join":
after transform: SymbolId(39): Span { start: 581, end: 585 }
rebuilt        : SymbolId(59): Span { start: 0, end: 0 }
Symbol flags mismatch for "keys":
after transform: SymbolId(41): SymbolFlags(ValueModule)
rebuilt        : SymbolId(62): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "keys":
after transform: SymbolId(41): Span { start: 608, end: 612 }
rebuilt        : SymbolId(62): Span { start: 0, end: 0 }
Symbol flags mismatch for "entries":
after transform: SymbolId(43): SymbolFlags(ValueModule)
rebuilt        : SymbolId(65): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "entries":
after transform: SymbolId(43): Span { start: 635, end: 642 }
rebuilt        : SymbolId(65): Span { start: 0, end: 0 }
Symbol flags mismatch for "values":
after transform: SymbolId(45): SymbolFlags(ValueModule)
rebuilt        : SymbolId(68): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "values":
after transform: SymbolId(45): Span { start: 665, end: 671 }
rebuilt        : SymbolId(68): Span { start: 0, end: 0 }
Symbol flags mismatch for "forEach":
after transform: SymbolId(47): SymbolFlags(ValueModule)
rebuilt        : SymbolId(71): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "forEach":
after transform: SymbolId(47): Span { start: 694, end: 701 }
rebuilt        : SymbolId(71): Span { start: 0, end: 0 }
Symbol flags mismatch for "filter":
after transform: SymbolId(49): SymbolFlags(ValueModule)
rebuilt        : SymbolId(74): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "filter":
after transform: SymbolId(49): Span { start: 724, end: 730 }
rebuilt        : SymbolId(74): Span { start: 0, end: 0 }
Symbol flags mismatch for "map":
after transform: SymbolId(51): SymbolFlags(ValueModule)
rebuilt        : SymbolId(77): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "map":
after transform: SymbolId(51): Span { start: 753, end: 756 }
rebuilt        : SymbolId(77): Span { start: 0, end: 0 }
Symbol flags mismatch for "every":
after transform: SymbolId(53): SymbolFlags(ValueModule)
rebuilt        : SymbolId(80): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "every":
after transform: SymbolId(53): Span { start: 779, end: 784 }
rebuilt        : SymbolId(80): Span { start: 0, end: 0 }
Symbol flags mismatch for "some":
after transform: SymbolId(55): SymbolFlags(ValueModule)
rebuilt        : SymbolId(83): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "some":
after transform: SymbolId(55): Span { start: 807, end: 811 }
rebuilt        : SymbolId(83): Span { start: 0, end: 0 }
Symbol flags mismatch for "reduce":
after transform: SymbolId(57): SymbolFlags(ValueModule)
rebuilt        : SymbolId(86): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "reduce":
after transform: SymbolId(57): Span { start: 834, end: 840 }
rebuilt        : SymbolId(86): Span { start: 0, end: 0 }
Symbol flags mismatch for "reduceRight":
after transform: SymbolId(59): SymbolFlags(ValueModule)
rebuilt        : SymbolId(89): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "reduceRight":
after transform: SymbolId(59): Span { start: 863, end: 874 }
rebuilt        : SymbolId(89): Span { start: 0, end: 0 }
Symbol flags mismatch for "toLocaleString":
after transform: SymbolId(61): SymbolFlags(ValueModule)
rebuilt        : SymbolId(92): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "toLocaleString":
after transform: SymbolId(61): Span { start: 897, end: 911 }
rebuilt        : SymbolId(92): Span { start: 0, end: 0 }
Symbol flags mismatch for "toString":
after transform: SymbolId(63): SymbolFlags(ValueModule)
rebuilt        : SymbolId(95): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "toString":
after transform: SymbolId(63): Span { start: 934, end: 942 }
rebuilt        : SymbolId(95): Span { start: 0, end: 0 }
Symbol flags mismatch for "flat":
after transform: SymbolId(65): SymbolFlags(ValueModule)
rebuilt        : SymbolId(98): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "flat":
after transform: SymbolId(65): Span { start: 965, end: 969 }
rebuilt        : SymbolId(98): Span { start: 0, end: 0 }
Symbol flags mismatch for "flatMap":
after transform: SymbolId(67): SymbolFlags(ValueModule)
rebuilt        : SymbolId(101): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "flatMap":
after transform: SymbolId(67): Span { start: 992, end: 999 }
rebuilt        : SymbolId(101): Span { start: 0, end: 0 }

* namespace/declare/input.ts
Bindings mismatch:
after transform: ScopeId(1): ["B", "C", "_N", "e", "f", "v"]
rebuilt        : ScopeId(1): ["_N"]
Symbol flags mismatch for "N":
after transform: SymbolId(0): SymbolFlags(ValueModule)
rebuilt        : SymbolId(0): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "N":
after transform: SymbolId(0): Span { start: 17, end: 18 }
rebuilt        : SymbolId(0): Span { start: 0, end: 0 }

* namespace/declare-global-nested-namespace/input.ts
Symbol flags mismatch for "X":
after transform: SymbolId(2): SymbolFlags(ValueModule)
rebuilt        : SymbolId(0): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "X":
after transform: SymbolId(2): Span { start: 70, end: 71 }
rebuilt        : SymbolId(0): Span { start: 0, end: 0 }

* namespace/empty-removed/input.ts
Bindings mismatch:
after transform: ScopeId(1): ["_a", "b", "c", "d"]
rebuilt        : ScopeId(1): ["_a", "c"]
Bindings mismatch:
after transform: ScopeId(6): ["_WithTypes", "a", "b", "c", "d"]
rebuilt        : ScopeId(3): ["_WithTypes", "d"]
Bindings mismatch:
after transform: ScopeId(12): ["D", "_d"]
rebuilt        : ScopeId(4): ["_d"]
Scope flags mismatch:
after transform: ScopeId(18): ScopeFlags(StrictMode)
rebuilt        : ScopeId(9): ScopeFlags(StrictMode | Function)
Symbol flags mismatch for "a":
after transform: SymbolId(0): SymbolFlags(ValueModule)
rebuilt        : SymbolId(0): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "a":
after transform: SymbolId(0): Span { start: 10, end: 11 }
rebuilt        : SymbolId(0): Span { start: 0, end: 0 }
Symbol reference IDs mismatch for "a":
after transform: SymbolId(0): [ReferenceId(0), ReferenceId(4), ReferenceId(5)]
rebuilt        : SymbolId(0): [ReferenceId(2), ReferenceId(3)]
Symbol flags mismatch for "c":
after transform: SymbolId(2): SymbolFlags(ValueModule)
rebuilt        : SymbolId(2): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "c":
after transform: SymbolId(2): Span { start: 43, end: 44 }
rebuilt        : SymbolId(2): Span { start: 0, end: 0 }
Symbol flags mismatch for "WithTypes":
after transform: SymbolId(6): SymbolFlags(ValueModule)
rebuilt        : SymbolId(5): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "WithTypes":
after transform: SymbolId(6): Span { start: 107, end: 116 }
rebuilt        : SymbolId(5): Span { start: 0, end: 0 }
Symbol flags mismatch for "d":
after transform: SymbolId(13): SymbolFlags(ValueModule)
rebuilt        : SymbolId(7): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "d":
after transform: SymbolId(13): Span { start: 224, end: 225 }
rebuilt        : SymbolId(7): Span { start: 0, end: 0 }
Symbol flags mismatch for "WithValues":
after transform: SymbolId(15): SymbolFlags(ValueModule)
rebuilt        : SymbolId(9): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "WithValues":
after transform: SymbolId(15): Span { start: 262, end: 272 }
rebuilt        : SymbolId(9): Span { start: 0, end: 0 }
Symbol flags mismatch for "a":
after transform: SymbolId(16): SymbolFlags(ValueModule)
rebuilt        : SymbolId(11): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "a":
after transform: SymbolId(16): Span { start: 287, end: 288 }
rebuilt        : SymbolId(11): Span { start: 0, end: 0 }
Symbol flags mismatch for "b":
after transform: SymbolId(18): SymbolFlags(ValueModule)
rebuilt        : SymbolId(14): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "b":
after transform: SymbolId(18): Span { start: 316, end: 317 }
rebuilt        : SymbolId(14): Span { start: 0, end: 0 }
Symbol flags mismatch for "B":
after transform: SymbolId(19): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(16): SymbolFlags(BlockScopedVariable)
Symbol flags mismatch for "c":
after transform: SymbolId(20): SymbolFlags(ValueModule)
rebuilt        : SymbolId(18): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "c":
after transform: SymbolId(20): Span { start: 344, end: 345 }
rebuilt        : SymbolId(18): Span { start: 0, end: 0 }
Symbol flags mismatch for "d":
after transform: SymbolId(22): SymbolFlags(ValueModule)
rebuilt        : SymbolId(21): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "d":
after transform: SymbolId(22): Span { start: 378, end: 379 }
rebuilt        : SymbolId(21): Span { start: 0, end: 0 }
Symbol flags mismatch for "e":
after transform: SymbolId(24): SymbolFlags(ValueModule)
rebuilt        : SymbolId(24): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "e":
after transform: SymbolId(24): Span { start: 402, end: 403 }
rebuilt        : SymbolId(24): Span { start: 0, end: 0 }

* namespace/export/input.ts
Symbol flags mismatch for "N":
after transform: SymbolId(0): SymbolFlags(ValueModule)
rebuilt        : SymbolId(0): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "N":
after transform: SymbolId(0): Span { start: 17, end: 18 }
rebuilt        : SymbolId(0): Span { start: 0, end: 0 }

* namespace/export-type-only/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["Platform"]
rebuilt        : ScopeId(0): []

* namespace/multiple/input.ts
Symbol flags mismatch for "N":
after transform: SymbolId(0): SymbolFlags(ValueModule)
rebuilt        : SymbolId(0): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "N":
after transform: SymbolId(0): Span { start: 10, end: 11 }
rebuilt        : SymbolId(0): Span { start: 0, end: 0 }
Symbol redeclarations mismatch for "N":
after transform: SymbolId(0): [Span { start: 10, end: 11 }, Span { start: 33, end: 34 }]
rebuilt        : SymbolId(0): []

* namespace/mutable-fail/input.ts

  ! Namespaces exporting non-const are not supported by Oxc. Change
  | to const or see: https://oxc.rs/docs/guide/usage/transformer/
  | typescript.html#partial-namespace-support
   ,-[tasks/coverage/babel/packages/babel-plugin-transform-typescript/test/fixtures/namespace/mutable-fail/input.ts:2:14]
 1 | namespace N {
 2 |   export let V;
   :              ^
 3 | }
   `----


* namespace/namespace-flag/input.ts

  ! Namespace not marked type-only declare are disabled. To enable and
  | review caveats see: https://oxc.rs/docs/guide/usage/transformer/
  | typescript.html#partial-namespace-support
   ,-[tasks/coverage/babel/packages/babel-plugin-transform-typescript/test/fixtures/namespace/namespace-flag/input.ts:1:1]
 1 | namespace N {}
   : ^^^^^^^^^^^^^^
   `----


* namespace/nested/input.ts
Bindings mismatch:
after transform: ScopeId(9): ["H", "I", "J", "K"]
rebuilt        : ScopeId(9): ["H"]
Scope flags mismatch:
after transform: ScopeId(9): ScopeFlags(StrictMode)
rebuilt        : ScopeId(9): ScopeFlags(StrictMode | Function)
Bindings mismatch:
after transform: ScopeId(13): ["L", "M"]
rebuilt        : ScopeId(13): ["L"]
Scope flags mismatch:
after transform: ScopeId(13): ScopeFlags(StrictMode)
rebuilt        : ScopeId(13): ScopeFlags(StrictMode | Function)
Symbol flags mismatch for "A":
after transform: SymbolId(0): SymbolFlags(Class | ValueModule)
rebuilt        : SymbolId(0): SymbolFlags(Class)
Symbol redeclarations mismatch for "A":
after transform: SymbolId(0): [Span { start: 6, end: 7 }, Span { start: 22, end: 23 }]
rebuilt        : SymbolId(0): []
Symbol flags mismatch for "C":
after transform: SymbolId(1): SymbolFlags(ValueModule)
rebuilt        : SymbolId(2): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "C":
after transform: SymbolId(1): Span { start: 45, end: 46 }
rebuilt        : SymbolId(2): Span { start: 0, end: 0 }
Symbol flags mismatch for "M":
after transform: SymbolId(4): SymbolFlags(Function | ValueModule)
rebuilt        : SymbolId(6): SymbolFlags(Function)
Symbol redeclarations mismatch for "M":
after transform: SymbolId(4): [Span { start: 110, end: 111 }, Span { start: 129, end: 130 }]
rebuilt        : SymbolId(6): []
Symbol flags mismatch for "D":
after transform: SymbolId(6): SymbolFlags(Function | ValueModule)
rebuilt        : SymbolId(9): SymbolFlags(Function)
Symbol redeclarations mismatch for "D":
after transform: SymbolId(6): [Span { start: 181, end: 182 }, Span { start: 207, end: 208 }]
rebuilt        : SymbolId(9): []
Symbol flags mismatch for "H":
after transform: SymbolId(8): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(12): SymbolFlags(BlockScopedVariable)
Symbol flags mismatch for "F":
after transform: SymbolId(12): SymbolFlags(Class | ValueModule)
rebuilt        : SymbolId(14): SymbolFlags(Class)
Symbol redeclarations mismatch for "F":
after transform: SymbolId(12): [Span { start: 308, end: 309 }, Span { start: 325, end: 326 }]
rebuilt        : SymbolId(14): []
Symbol flags mismatch for "G":
after transform: SymbolId(14): SymbolFlags(ValueModule)
rebuilt        : SymbolId(17): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "G":
after transform: SymbolId(14): Span { start: 350, end: 351 }
rebuilt        : SymbolId(17): Span { start: 0, end: 0 }
Symbol flags mismatch for "L":
after transform: SymbolId(16): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(20): SymbolFlags(BlockScopedVariable)
Reference flags mismatch for "_C":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_C":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_A":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_A":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_M":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_A":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "H":
after transform: ReferenceId(26): ReferenceFlags(Read)
rebuilt        : ReferenceId(13): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "H":
after transform: ReferenceId(25): ReferenceFlags(Read)
rebuilt        : ReferenceId(14): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "H":
after transform: ReferenceId(28): ReferenceFlags(Read)
rebuilt        : ReferenceId(15): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "H":
after transform: ReferenceId(27): ReferenceFlags(Read)
rebuilt        : ReferenceId(16): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "H":
after transform: ReferenceId(30): ReferenceFlags(Read)
rebuilt        : ReferenceId(17): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "H":
after transform: ReferenceId(29): ReferenceFlags(Read)
rebuilt        : ReferenceId(18): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_D":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(20): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_A":
after transform: ReferenceId(18): ReferenceFlags(Read)
rebuilt        : ReferenceId(24): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_A":
after transform: ReferenceId(16): ReferenceFlags(Read)
rebuilt        : ReferenceId(25): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "L":
after transform: ReferenceId(33): ReferenceFlags(Read)
rebuilt        : ReferenceId(30): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "L":
after transform: ReferenceId(32): ReferenceFlags(Read)
rebuilt        : ReferenceId(31): ReferenceFlags(Read | MemberWriteTarget)

* namespace/nested-namespace/input.ts
Bindings mismatch:
after transform: ScopeId(1): ["B", "G", "_A"]
rebuilt        : ScopeId(1): ["G", "_A"]
Bindings mismatch:
after transform: ScopeId(4): ["G", "H"]
rebuilt        : ScopeId(2): ["G"]
Scope flags mismatch:
after transform: ScopeId(4): ScopeFlags(StrictMode)
rebuilt        : ScopeId(2): ScopeFlags(StrictMode | Function)
Symbol flags mismatch for "A":
after transform: SymbolId(0): SymbolFlags(ValueModule)
rebuilt        : SymbolId(0): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "A":
after transform: SymbolId(0): Span { start: 17, end: 18 }
rebuilt        : SymbolId(0): Span { start: 0, end: 0 }
Symbol flags mismatch for "G":
after transform: SymbolId(3): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(2): SymbolFlags(BlockScopedVariable)
Reference flags mismatch for "G":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "G":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_A":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)

* namespace/nested-shorthand/input.ts
Symbol flags mismatch for "X":
after transform: SymbolId(0): SymbolFlags(ValueModule)
rebuilt        : SymbolId(0): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "X":
after transform: SymbolId(0): Span { start: 10, end: 11 }
rebuilt        : SymbolId(0): Span { start: 0, end: 0 }
Symbol flags mismatch for "Y":
after transform: SymbolId(1): SymbolFlags(ValueModule)
rebuilt        : SymbolId(2): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "Y":
after transform: SymbolId(1): Span { start: 12, end: 13 }
rebuilt        : SymbolId(2): Span { start: 0, end: 0 }
Symbol flags mismatch for "proj":
after transform: SymbolId(3): SymbolFlags(ValueModule)
rebuilt        : SymbolId(5): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "proj":
after transform: SymbolId(3): Span { start: 51, end: 55 }
rebuilt        : SymbolId(5): Span { start: 0, end: 0 }
Symbol flags mismatch for "data":
after transform: SymbolId(4): SymbolFlags(ValueModule)
rebuilt        : SymbolId(7): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "data":
after transform: SymbolId(4): Span { start: 56, end: 60 }
rebuilt        : SymbolId(7): Span { start: 0, end: 0 }
Symbol flags mismatch for "util":
after transform: SymbolId(5): SymbolFlags(ValueModule)
rebuilt        : SymbolId(9): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "util":
after transform: SymbolId(5): Span { start: 61, end: 65 }
rebuilt        : SymbolId(9): Span { start: 0, end: 0 }
Symbol flags mismatch for "api":
after transform: SymbolId(6): SymbolFlags(ValueModule)
rebuilt        : SymbolId(11): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "api":
after transform: SymbolId(6): Span { start: 66, end: 69 }
rebuilt        : SymbolId(11): Span { start: 0, end: 0 }
Reference flags mismatch for "_Y":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_X":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_X":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_api":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_util":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_util":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_data":
after transform: ReferenceId(15): ReferenceFlags(Read)
rebuilt        : ReferenceId(14): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_data":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(15): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_proj":
after transform: ReferenceId(19): ReferenceFlags(Read)
rebuilt        : ReferenceId(18): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_proj":
after transform: ReferenceId(17): ReferenceFlags(Read)
rebuilt        : ReferenceId(19): ReferenceFlags(Read | MemberWriteTarget)

* namespace/same-name/input.ts
Scope flags mismatch:
after transform: ScopeId(8): ScopeFlags(StrictMode)
rebuilt        : ScopeId(8): ScopeFlags(StrictMode | Function)
Symbol flags mismatch for "N":
after transform: SymbolId(0): SymbolFlags(ValueModule)
rebuilt        : SymbolId(0): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "N":
after transform: SymbolId(0): Span { start: 10, end: 11 }
rebuilt        : SymbolId(0): Span { start: 0, end: 0 }
Symbol flags mismatch for "_N7":
after transform: SymbolId(1): SymbolFlags(ValueModule)
rebuilt        : SymbolId(2): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "_N7":
after transform: SymbolId(1): Span { start: 26, end: 29 }
rebuilt        : SymbolId(2): Span { start: 0, end: 0 }
Symbol flags mismatch for "N":
after transform: SymbolId(3): SymbolFlags(ValueModule)
rebuilt        : SymbolId(5): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "N":
after transform: SymbolId(3): Span { start: 59, end: 60 }
rebuilt        : SymbolId(5): Span { start: 0, end: 0 }
Symbol redeclarations mismatch for "N":
after transform: SymbolId(3): [Span { start: 59, end: 60 }, Span { start: 115, end: 116 }, Span { start: 166, end: 167 }]
rebuilt        : SymbolId(5): []
Symbol flags mismatch for "_N":
after transform: SymbolId(6): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(11): SymbolFlags(BlockScopedVariable)
Reference flags mismatch for "_N10":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_N8":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_N8":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_N11":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_N8":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(12): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_N8":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(13): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_N12":
after transform: ReferenceId(14): ReferenceFlags(Read)
rebuilt        : ReferenceId(15): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_N8":
after transform: ReferenceId(19): ReferenceFlags(Read)
rebuilt        : ReferenceId(19): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_N8":
after transform: ReferenceId(17): ReferenceFlags(Read)
rebuilt        : ReferenceId(20): ReferenceFlags(Read | MemberWriteTarget)

* namespace/undeclared/input.ts
Symbol flags mismatch for "N":
after transform: SymbolId(0): SymbolFlags(ValueModule)
rebuilt        : SymbolId(0): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "N":
after transform: SymbolId(0): Span { start: 10, end: 11 }
rebuilt        : SymbolId(0): Span { start: 0, end: 0 }

* optimize-const-enums/custom-values/input.ts
x Output mismatch

* optimize-const-enums/custom-values-exported/input.ts
x Output mismatch

* optimize-const-enums/declare/input.ts
x Output mismatch

* optimize-const-enums/export-const-enum/input.ts
x Output mismatch

* optimize-const-enums/export-const-enum-type-and-value/input.ts
x Output mismatch

* optimize-const-enums/export-const-enum-type-no-deopt/input.ts
x Output mismatch

* optimize-const-enums/exported/input.ts
x Output mismatch

* optimize-const-enums/local/input.ts
x Output mismatch

* optimize-const-enums/local-shadowed/input.ts
x Output mismatch

* optimize-const-enums/merged/input.ts
x Output mismatch

* optimize-const-enums/merged-exported/input.ts
x Output mismatch

* optimize-const-enums/namespace/input.ts
x Output mismatch

* regression/15768/input.ts
x Output mismatch

* type-arguments/call/input.ts
Unresolved references mismatch:
after transform: ["T", "f"]
rebuilt        : ["f"]

* type-arguments/expr/input.ts
Unresolved references mismatch:
after transform: ["T", "f"]
rebuilt        : ["f"]

* type-arguments/new/input.ts
Unresolved references mismatch:
after transform: ["C", "T"]
rebuilt        : ["C"]

* type-arguments/optional-call/input.ts
Unresolved references mismatch:
after transform: ["Q", "T", "f", "x"]
rebuilt        : ["f", "x"]

* type-arguments/tagged-template/input.ts
Unresolved references mismatch:
after transform: ["T", "f"]
rebuilt        : ["f"]

* variable-declaration/non-null-in-optional-chain/input.ts
Reference flags mismatch for "_a$b":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)


# babel-preset-react (1/10)
* preset-options/development/input.js
react: unknown field `developmentSourceSelf`, expected one of `runtime`, `development`, `throwIfNamespace`, `pure`, `importSource`, `pragma`, `pragmaFrag`, `useBuiltIns`, `useSpread`, `refresh`

* preset-options/development-no-source-self/input.js
x Output mismatch

* preset-options/development-runtime-automatic/input.js
react: unknown field `developmentSourceSelf`, expected one of `runtime`, `development`, `throwIfNamespace`, `pure`, `importSource`, `pragma`, `pragmaFrag`, `useBuiltIns`, `useSpread`, `refresh`

* preset-options/empty-options/input.js
Reference flags mismatch for "_reactJsxRuntime":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)

* preset-options/pure/input.js
x Output mismatch

* preset-options/runtime-automatic/input.js
Reference flags mismatch for "_reactJsxRuntime":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)

* preset-options/runtime-classic/input.js
Reference flags mismatch for "React":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)

* preset-options/runtime-classic-pragma-no-frag/input.js
x Output mismatch

* regression/11294/input.mjs
x Output mismatch


# babel-plugin-transform-react-jsx (81/149)
* autoImport/after-polyfills-compiled-to-cjs/input.mjs
x Output mismatch

* autoImport/after-polyfills-script-not-supported/input.js
Reference flags mismatch for "_reactJsxRuntime":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)

* autoImport/auto-import-react-source-type-script/input.js
Reference flags mismatch for "_reactJsxRuntime":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_reactJsxRuntime":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_reactJsxRuntime":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_reactJsxRuntime":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_reactJsxRuntime":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_reactJsxRuntime":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_react":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)

* autoImport/complicated-scope-script/input.js
Reference flags mismatch for "_reactJsxRuntime":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_reactJsxRuntime":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)

* pure/false-default-pragma-automatic-runtime/input.js
x Output mismatch

* pure/false-default-pragma-classic-runtime/input.js
x Output mismatch

* pure/true-default-pragma-automatic-runtime/input.js
Reference flags mismatch for "_reactJsxRuntime":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)

* pure/true-default-pragma-classic-runtime/input.js
Reference flags mismatch for "React":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)

* pure/unset-default-pragma-automatic-runtime/input.js
Reference flags mismatch for "_reactJsxRuntime":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)

* pure/unset-default-pragma-classic-runtime/input.js
Reference flags mismatch for "React":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)

* react/adds-appropriate-newlines-when-using-spread-attribute/input.js
Reference flags mismatch for "React":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)

* react/arrow-functions/input.js
Reference flags mismatch for "React":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "React":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_this2":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)

* react/assignment/input.js
Reference flags mismatch for "React":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)

* react/comments/input.js
Reference flags mismatch for "React":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "React":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "React":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "React":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "React":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "React":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "React":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "React":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "React":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "React":
after transform: ReferenceId(12): ReferenceFlags(Read)
rebuilt        : ReferenceId(12): ReferenceFlags(Read | MemberWriteTarget)

* react/concatenates-adjacent-string-literals/input.js
Reference flags mismatch for "React":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "React":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)

* react/does-not-add-source-self/input.mjs
Reference flags mismatch for "React":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "React":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "React":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "React":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "React":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "React":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "React":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)

* react/dont-coerce-expression-containers/input.js
Reference flags mismatch for "React":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)

* react/duplicate-props/input.js
Reference flags mismatch for "React":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "React":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "React":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "React":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)

* react/flattens-spread/input.js
Reference flags mismatch for "React":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "React":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "React":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "React":
after transform: ReferenceId(12): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "React":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(12): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "React":
after transform: ReferenceId(14): ReferenceFlags(Read)
rebuilt        : ReferenceId(13): ReferenceFlags(Read | MemberWriteTarget)

* react/handle-spread-with-proto/input.js
Reference flags mismatch for "React":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "React":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)

* react/jsx-with-retainlines-option/input.js
Reference flags mismatch for "React":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)

* react/jsx-without-retainlines-option/input.js
Reference flags mismatch for "React":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)

* react/proto-in-jsx-attribute/input.js
Reference flags mismatch for "React":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)

* react/should-allow-constructor-as-prop/input.js
Reference flags mismatch for "React":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)

* react/should-allow-deeper-js-namespacing/input.js
Reference flags mismatch for "React":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Namespace":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)

* react/should-allow-elements-as-attributes/input.js
Reference flags mismatch for "React":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "React":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)

* react/should-allow-js-namespacing/input.js
Reference flags mismatch for "React":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Namespace":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)

* react/should-allow-nested-fragments/input.js
Reference flags mismatch for "React":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "React":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "React":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "React":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "React":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "React":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "React":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "React":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "React":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "React":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "React":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)

* react/should-avoid-wrapping-in-extra-parens-if-not-needed/input.js
Reference flags mismatch for "React":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "React":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "React":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "React":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "React":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "React":
after transform: ReferenceId(12): ReferenceFlags(Read)
rebuilt        : ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)

* react/should-convert-simple-tags/input.js
Reference flags mismatch for "React":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)

* react/should-convert-simple-text/input.js
Reference flags mismatch for "React":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)

* react/should-disallow-valueless-key/input.js

  ! Please provide an explicit key value. Using "key" as a shorthand for
  | "key={true}" is not allowed.
   ,-[tasks/coverage/babel/packages/babel-plugin-transform-react-jsx/test/fixtures/react/should-disallow-valueless-key/input.js:2:15]
 1 | 
 2 | var x = [<div key></div>];
   :               ^^^
   `----


* react/should-disallow-xml-namespacing/input.js

  ! Namespace tags are not supported by default. React's JSX doesn't support
  | namespace tags. You can set `throwIfNamespace: false` to bypass this
  | warning.
   ,-[tasks/coverage/babel/packages/babel-plugin-transform-react-jsx/test/fixtures/react/should-disallow-xml-namespacing/input.js:1:2]
 1 | <Namespace:Component />;
   :  ^^^^^^^^^^^^^^^^^^^
   `----


* react/should-escape-xhtml-jsxattribute/input.js
Reference flags mismatch for "React":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "React":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "React":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)

* react/should-escape-xhtml-jsxtext/input.js
Reference flags mismatch for "React":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "React":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "React":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "React":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "React":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "React":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "React":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "React":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "React":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)

* react/should-handle-attributed-elements/input.js
Reference flags mismatch for "React":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "React":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "React":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)

* react/should-handle-has-own-property-correctly/input.js
Reference flags mismatch for "React":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)

* react/should-have-correct-comma-in-nested-children/input.js
Reference flags mismatch for "React":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "React":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "React":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "React":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "React":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "React":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)

* react/should-insert-commas-after-expressions-before-whitespace/input.js
Reference flags mismatch for "React":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)

* react/should-not-add-quotes-to-identifier-names/input.js
Reference flags mismatch for "React":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)

* react/should-not-mangle-expressioncontainer-attribute-values/input.js
Reference flags mismatch for "React":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)

* react/should-not-strip-nbsp-even-coupled-with-other-whitespace/input.js
Reference flags mismatch for "React":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)

* react/should-not-strip-tags-with-a-single-child-of-nbsp/input.js
Reference flags mismatch for "React":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)

* react/should-properly-handle-comments-between-props/input.js
Reference flags mismatch for "React":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "React":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)

* react/should-quote-jsx-attributes/input.js
Reference flags mismatch for "React":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)

* react/should-throw-error-namespaces-if-not-flag/input.js

  ! Namespace tags are not supported by default. React's JSX doesn't support
  | namespace tags. You can set `throwIfNamespace: false` to bypass this
  | warning.
   ,-[tasks/coverage/babel/packages/babel-plugin-transform-react-jsx/test/fixtures/react/should-throw-error-namespaces-if-not-flag/input.js:1:2]
 1 | <f:image />;
   :  ^^^^^^^
   `----


* react/should-transform-known-hyphenated-tags/input.js
Reference flags mismatch for "React":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)

* react/this-tag-name/input.js
Reference flags mismatch for "React":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)

* react/weird-symbols/input.js
Reference flags mismatch for "React":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)

* react/wraps-props-in-react-spread-for-first-spread-attributes/input.js
Reference flags mismatch for "React":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)

* react/wraps-props-in-react-spread-for-last-spread-attributes/input.js
Reference flags mismatch for "React":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)

* react/wraps-props-in-react-spread-for-middle-spread-attributes/input.js
Reference flags mismatch for "React":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)

* react-automatic/arrow-functions/input.js
Reference flags mismatch for "_this2":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)

* react-automatic/does-not-add-source-self-automatic/input.mjs
transform-react-jsx: unknown field `autoImport`, expected one of `runtime`, `development`, `throwIfNamespace`, `pure`, `importSource`, `pragma`, `pragmaFrag`, `useBuiltIns`, `useSpread`, `refresh`

* react-automatic/handle-fragments-with-key/input.js
Reference flags mismatch for "React":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)

* react-automatic/should-allow-deeper-js-namespacing/input.js
Reference flags mismatch for "Namespace":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)

* react-automatic/should-allow-js-namespacing/input.js
Reference flags mismatch for "Namespace":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)

* react-automatic/should-disallow-valueless-key/input.js

  ! Please provide an explicit key value. Using "key" as a shorthand for
  | "key={true}" is not allowed.
   ,-[tasks/coverage/babel/packages/babel-plugin-transform-react-jsx/test/fixtures/react-automatic/should-disallow-valueless-key/input.js:2:15]
 1 | 
 2 | var x = [<div key></div>];
   :               ^^^
   `----


* react-automatic/should-disallow-xml-namespacing/input.js

  ! Namespace tags are not supported by default. React's JSX doesn't support
  | namespace tags. You can set `throwIfNamespace: false` to bypass this
  | warning.
   ,-[tasks/coverage/babel/packages/babel-plugin-transform-react-jsx/test/fixtures/react-automatic/should-disallow-xml-namespacing/input.js:1:2]
 1 | <Namespace:Component />;
   :  ^^^^^^^^^^^^^^^^^^^
   `----


* react-automatic/should-throw-error-namespaces-if-not-flag/input.js

  ! Namespace tags are not supported by default. React's JSX doesn't support
  | namespace tags. You can set `throwIfNamespace: false` to bypass this
  | warning.
   ,-[tasks/coverage/babel/packages/babel-plugin-transform-react-jsx/test/fixtures/react-automatic/should-throw-error-namespaces-if-not-flag/input.js:1:2]
 1 | <f:image />;
   :  ^^^^^^^
   `----


* regression/pragma-frag-set-default-classic-runtime/input.js
Reference flags mismatch for "React":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)

* runtime/classic/input.js
Reference flags mismatch for "React":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "React":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)

* runtime/defaults-to-automatic/input.js
Reference flags mismatch for "_reactJsxRuntime":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_reactJsxRuntime":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)

* runtime/pragma-runtime-classsic/input.js
Reference flags mismatch for "React":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "React":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)

* runtime/runtime-automatic/input.js
Reference flags mismatch for "_reactJsxRuntime":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_reactJsxRuntime":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)

* sourcemaps/JSXText/input.js
Reference flags mismatch for "React":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "React":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "React":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)

* spread-transform/transform-to-babel-extend/input.js

  x Option `loose` is not implemented for object-rest-spread.


* spread-transform/transform-to-object-assign/input.js

  x Option `loose` is not implemented for object-rest-spread.


  x Option `useBuiltIns` is not implemented for object-rest-spread.



# babel-plugin-transform-react-jsx-development (0/9)
* cross-platform/auto-import-dev/input.js
transform-react-jsx-development: unknown field `sourceSelf`, expected one of `runtime`, `development`, `throwIfNamespace`, `pure`, `importSource`, `pragma`, `pragmaFrag`, `useBuiltIns`, `useSpread`, `refresh`

* cross-platform/classic-runtime/input.js
transform-react-jsx-development: unknown field `sourceSelf`, expected one of `runtime`, `development`, `throwIfNamespace`, `pure`, `importSource`, `pragma`, `pragmaFrag`, `useBuiltIns`, `useSpread`, `refresh`

* cross-platform/fragments/input.js
transform-react-jsx-development: unknown field `sourceSelf`, expected one of `runtime`, `development`, `throwIfNamespace`, `pure`, `importSource`, `pragma`, `pragmaFrag`, `useBuiltIns`, `useSpread`, `refresh`

* cross-platform/handle-fragments-with-key/input.js
transform-react-jsx-development: unknown field `sourceSelf`, expected one of `runtime`, `development`, `throwIfNamespace`, `pure`, `importSource`, `pragma`, `pragmaFrag`, `useBuiltIns`, `useSpread`, `refresh`

* cross-platform/handle-nonstatic-children/input.js
transform-react-jsx-development: unknown field `sourceSelf`, expected one of `runtime`, `development`, `throwIfNamespace`, `pure`, `importSource`, `pragma`, `pragmaFrag`, `useBuiltIns`, `useSpread`, `refresh`

* cross-platform/handle-static-children/input.js
transform-react-jsx-development: unknown field `sourceSelf`, expected one of `runtime`, `development`, `throwIfNamespace`, `pure`, `importSource`, `pragma`, `pragmaFrag`, `useBuiltIns`, `useSpread`, `refresh`

* cross-platform/no-source-self/input.js
x Output mismatch

* cross-platform/within-derived-classes-constructor/input.js
transform-react-jsx-development: unknown field `sourceSelf`, expected one of `runtime`, `development`, `throwIfNamespace`, `pure`, `importSource`, `pragma`, `pragmaFrag`, `useBuiltIns`, `useSpread`, `refresh`

* cross-platform/within-ts-module-block/input.ts
transform-react-jsx-development: unknown field `sourceSelf`, expected one of `runtime`, `development`, `throwIfNamespace`, `pure`, `importSource`, `pragma`, `pragmaFrag`, `useBuiltIns`, `useSpread`, `refresh`


