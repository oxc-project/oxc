# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.48.2] - 2025-02-02

### Features

- 86b6219 mangler: Use characters in the order of their likely frequency (#8771) (sapphi-red)
- d553318 minifier: Complete `MangleIf` (#8810) (Boshen)
- 5cfea76 minifier: Compress `(a = _a) != null ? a : b` and `(a = _a) != null ? a.b() : undefined` (#8823) (sapphi-red)
- f02d9e9 minifier: Merge single var declarations without init into for-of (#8813) (sapphi-red)
- 99b47ed minifier: Merge single var declarations without init into for-in (#8812) (sapphi-red)
- d9f1d0d minifier: Merge expressions in for-in statement head (#8811) (sapphi-red)
- 18f1b15 minifier: Implement known method `Array.of` (#8805) (7086cmd)
- e525e60 minifier: Compress `a != null ? a.b : undefined` to `a?.b` (#8802) (#8808) (Boshen)
- e353a01 minifier: Compress `a != null ? a.b : undefined` to `a?.b` (#8802) (sapphi-red)
- 72d74a2 minifier: Compress `a != null ? a : b` into `a ?? b` (#8801) (sapphi-red)
- 249895f minifier: Implement variadic Math methods in known methods (#8783) (Ethan Goh)
- 7ea99f4 minifier: Compress array of string literals to `'str1,str2'.split(',')` (#8786) (sapphi-red)
- 6c627df minifier: Implement unary Math functions in known methods (#8781) (7086cmd)
- ad14403 minifier: Compress `typeof a.b === 'undefined'` to `a.b === void 0` (#8751) (sapphi-red)
- f7f2d2f minifier: Compress `a == null && b` to `a ?? b` when return value is ignored (#8749) (sapphi-red)
- 3c1c92c minifier: Support `a[0]` and `this.a` in `has_no_side_effect_for_evaluation_same_target` (#8748) (sapphi-red)
- 29417dd minifier: Minimize `!(a, b)` -> `a, !b` (#8746) (Boshen)
- 3ece991 minifier: Remove unused `import.meta` statement (#8744) (Boshen)
- 3ef980a minifier: Remove unreachable statements after `break` and `continue` (#8743) (Boshen)

### Bug Fixes

- 831928d minifier: Mark as changed when `else if` was converted to `if` (#8837) (翠 / green)
- f8548ec minifier: Unreachable error when compressing string literal arrays with `.split()` (#8806) (sapphi-red)
- 2eac9c0 minifier: Fix `var undefined = 1; foo === null || foo === undefined` should not be compressed (#8803) (sapphi-red)
- ae7f670 minifier: Avoid minifying `+void unknown` to `NaN` and fix typo (#8784) (7086cmd)
- 8781537 minifier: `{ let foo; const bar = undefined; }` -> `{ let foo, bar; }` (#8764) (Boshen)
- 8a6ae8a minifier: Do not change `const` to `let` if assignment to constant variable. (#8761) (Boshen)
- 66c33ed minifier: Remove incorrect not + conditional expression compression (#8759) (翠 / green)
- a3b078a minifier: Fix crash with `[]['concat'](1)` (#8750) (sapphi-red)

### Refactor

- 6aa2dde codegen: Accept SymbolTable instead of Mangler (#8829) (Daniel Bulant)
- 3abf2f7 minifier: Extract `extract_id_or_assign_to_id` method (#8822) (sapphi-red)
- a861d93 minifier: Port esbuild's `mangleStmts` (#8770) (Boshen)
- 0fcff20 minifier: Remove `EmptyStatement` in a single place (#8745) (Boshen)

### Testing

- dc4c388 minifier: Fail tests when parse fails (#8836) (sapphi-red)
- 3ac5020 minifier: Enable more passed esbuild tests (#8804) (Boshen)
- 0c4c739 minifier: Cleanup some tests in substitute_alternate_syntax (#8768) (sapphi-red)
- 79d5481 minifier: Add and enable some tests in fold_constants (#8769) (sapphi-red)
- ef55e7c minifier: Check idempotency for all tests (#8754) (Boshen)
- d072f09 minifier: Enable more ignored tests (#8753) (Boshen)
- e78e468 minifier: Cleanup some tests in minimize_conditions (#8752) (sapphi-red)

## [0.48.1] - 2025-01-26

### Features

- 6589c3b mangler: Reuse variable names (#8562) (翠 / green)
- 29bd215 minifier: Minimize `Infinity.toString(radix)` to `'Infinity'` (#8732) (Boshen)
- e0117db minifier: Replace `const` with `let` for non-exported read-only variables (#8733) (sapphi-red)
- 9e32f55 minifier: Evaluate `Math.sqrt` and `Math.cbrt` (#8731) (sapphi-red)
- 360d49e minifier: Replace `Math.pow` with `**` (#8730) (sapphi-red)
- 2e9a560 minifier: `NaN.toString(radix)` is always `NaN` (#8727) (Boshen)
- cbe0e82 minifier: Minimize `foo(...[])` -> `foo()` (#8726) (Boshen)
- e9fb5fe minifier: Dce pure expressions such as `new Map()` (#8725) (Boshen)

### Bug Fixes

- 33de70a mangler: Handle cases where a var is declared in a block scope (#8706) (翠 / green)
- d982cdb minifier: `Unknown.fromCharCode` should not be treated as `String.fromCharCode` (#8709) (sapphi-red)

### Performance

- e472ced mangler: Optimize handling of collecting lived scope ids (#8724) (Dunqing)
- 8587965 minifier: Normalize `undefined` to `void 0` before everything else (#8699) (Boshen)

### Refactor

- 58002e2 ecmascript: Remove the lifetime annotation on `MayHaveSideEffects` (#8717) (Boshen)
- 6bc906c minifier: Allow mutating arguments in methods called from `try_fold_known_string_methods` (#8729) (sapphi-red)
- bf8be23 minifier: Use `Ctx` (#8716) (Boshen)
- 0af0267 minifier: Side effect detection needs symbols resolution (#8715) (Boshen)
- 32e0e47 minifier: Clean up `Normalize` (#8700) (Boshen)

### Testing

- 03229c5 minifier: Fix broken tests (#8722) (Boshen)

## [0.48.0] - 2025-01-24

### Features

- 343690e minifier: Replace `Number.*_SAFE_INTEGER`/`Number.EPSILON` (#8682) (sapphi-red)
- 0c5bb30 minifier: Replace `Number.POSITIVE_INFINITY`/`Number.NEGATIVE_INFINITY`/`Number.NaN` (#8681) (sapphi-red)
- 835b258 minifier: Compress `typeof foo === 'object' && foo !== null` to `typeof foo == 'object' && !!foo` (#8638) (sapphi-red)
- 2bcbed2 minifier: Compress `(a = b) === null || a === undefined` to `(a = b) == null` (#8637) (sapphi-red)

### Bug Fixes

- 883d25b minifier: Keep esm in dce (#8677) (Boshen)
- 878ce10 minifier: `void 0` equals to `undefined` (#8673) (Boshen)
- ba201a6 minifier: Remove "non esbuild optimizations" which is incorrect (#8668) (Boshen)
- 8c8b5fa minifier: Avoid minifing `String(a)` into `"" + a` for symbols (#8612) (翠 / green)
- 4ff6e85 minifier: Remove expression statement `void 0` (#8602) (Boshen)
- 93d643e minifier: Keep side effects when folding const conditional exprs (#8591) (camc314)

### Performance

- 9953ac7 minifier: Add `LatePeepholeOptimizations` (#8651) (Boshen)
- 00dc63f minifier: Only substitute typed array constructor once (#8649) (Boshen)
- 3e19e4e minifier: Remove the useless empty statement removal code in statement fusion (#8646) (Boshen)
- 5b3c412 minifier: Only run optimizations on local changes (#8644) (Boshen)

### Refactor

- e66da9f isolated_declarations, linter, minifier, prettier, semantic, transformer: Remove unnecessary `ref` / `ref mut` syntax (#8643) (overlookmotel)
- ce2b9da minifier: Remove `wrap_to_avoid_ambiguous_else` (#8676) (Boshen)
- 75a579b minifier: Clean up `has_no_side_effect_for_evaluation_same_target` (#8675) (Boshen)
- 1bb2539 minifier: Move more code into `minimize_conditions` local loop (#8671) (Boshen)
- 13e4a45 minifier: Move conditional assignment to `minimize_conditions` (#8669) (Boshen)
- ae895d8 minifier: Use `NonEmptyStack` for function stack (#8661) (Boshen)
- 3802d28 minifier: Clean up `try_minimize_conditional` (#8660) (Boshen)
- dcc1f2b minifier: Rename `ast_passes` to `peephole` (#8635) (Boshen)
- 52458de minifier: Remove unused code and traits (#8632) (Boshen)
- 6f95cd5 minifier: Remove all the unnecessary fake ast passes (#8618) (Boshen)
- 712cae0 minifier: Run the compressor on all test cases (#8604) (Boshen)
- ac4f98e span: Derive `Copy` on `Atom` (#8596) (branchseer)

### Testing

- d9f5e7f minifier: Enable passed esbuild tests (Boshen)

## [0.47.1] - 2025-01-19

### Bug Fixes

- 7b219a9 minifier: Fix dce shadowed undefined (#8582) (Boshen)

## [0.47.0] - 2025-01-18

### Features

- 4d4e805 minifier: Collapse if stmt with empty consequent (#8577) (camc314)
- 991a22f minifier: Fold `Array::concat` into literal (#8442) (sapphi-red)
- 3dc2d8b minifier: Fold string concat chaining (#8441) (sapphi-red)
- a4ae450 minifier: Fold array concat chaining (#8440) (sapphi-red)
- 7cc81ef minifier: Fold invalid typeof comparisons (#8550) (camc314)
- 927f43f minifier: Improve `.charCodeAt(arg)` when arg is valid (#8534) (Boshen)
- 06f14d5 minifier: Remove empty class static block `class Foo { static {} }` (#8525) (Boshen)
- 1860411 minifier: Remove last redundant return statement (#8523) (Boshen)

### Bug Fixes

- 65c596d minifer: Keep idents if not in scope when minimizing array exprs (#8551) (camc314)
- f57aac2 minifier: Incorrect folding of expr in bool ctx (#8542) (camc314)
- 946ad76 minifier: `(-Infinity).toString()` -> `'-Infinity'` (#8535) (Boshen)
- b1d0186 minifier: Do not fold `!!void b` (#8533) (Boshen)
- 53adde5 minifier: `x['-2147483648']` -> `x[-2147483648]` (#8528) (Boshen)
- 405b73d minifier: Do not change `delete undefined` to `delete void 0` (#8527) (Boshen)
- 92e44cb minifier: Do not remove `undefined` in `var x = undefined` (#8526) (Boshen)
- 209e313 minifier: `class C { ['-1']() {} }` cannot be minifized (#8516) (Boshen)
- 6585463 minifier: Always keep the last value of sequence expression (#8490) (Boshen)

### Refactor

- 8f57929 minifier: Merge `try_compress_type_of_equal_string` into `try_minimize_binary` (#8561) (sapphi-red)

### Testing

- e0f5d6c minifier: Update esbuild test (Boshen)
- 629c417 minifier: Port esbuild minification tests (#8497) (Boshen)

## [0.46.0] - 2025-01-14

### Features

- 8accfef minifier: Minify `var x; void x` -> `void 0` (#8466) (Boshen)
- 870a583 minifier: Fold `false['toString']` (#8447) (Boshen)
- 4ad695d napi/minify: Implement napi (#8478) (Boshen)

### Bug Fixes

- 4c6675c minifier: Do not convert while to fors in DCE (#8484) (Boshen)
- 1d6e84d minifier: Fix incorrect `null.toString()` and `1n.toString()` (#8464) (Boshen)
- 25d4bf9 minifier: Remove usage of empty spans (#8462) (Boshen)
- dd64340 minifier: Keep `return undefined` in async generator function (#8439) (Boshen)

### Performance

- 8fc238a minifier: Merge `Normalize` and `RemoveSyntax` pass (#8467) (Boshen)
- 372eb09 minifier: Preallocate mangler's semantic data (#8451) (Boshen)

## [0.45.0] - 2025-01-11

### Features

- 6c7acac allocator: Implement `IntoIterator` for `&mut Vec` (#8389) (overlookmotel)
- 41ddf60 minfier: Add `CompressOptions::target` (#8179) (Boshen)
- d56020b minifier: Drop `0` from `new Int8Array(0)` and other TypedArrays (#8431) (sapphi-red)
- f935d94 minifier: Remove `new` from NativeErrors / `AggregateError` (#8430) (sapphi-red)
- dab7a51 minifier: Minimize not `!(x === undefined)` -> `x !== undefined` (#8429) (Boshen)
- 0e7bab8 minifier: Remove `if(false){}` in a single pass (#8421) (Boshen)
- 5b5b844 minifier: Fold `ambiguous if else` (#8415) (Boshen)
- 438a6e7 minifier: Minimize conditions in boolean context (#8381) (Boshen)
- 793cb43 minifier: `a != null ? a : b` -> `a ?? b` (#8352) (camc314)
- 814da55 minifier: Compress `x = x || 1` to `x ||= 1` (#8368) (sapphi-red)
- a596821 minifier: Compress `a.b = a.b + c` to `a.b += c` (#8367) (sapphi-red)
- 579eb60 minifier: Compress `a.b || (a.b = c)` to `a.b ||= c` (#8366) (sapphi-red)
- f367a16 minifier: Port esbuild conditional expr minification (#8351) (camc314)
- 8d52cd0 minifier: Merge assign expression in conditional expression (#8345) (sapphi-red)
- a69d15f minifier: Compress `new Array(2)` -> `[,,]` (#8344) (sapphi-red)
- 819c475 minifier: Compress `new Array(7n)` -> `[7n]` (#8343) (sapphi-red)
- e085d66 minifier: Remove empty IIFE (#8340) (Boshen)
- 2c2e483 minifier: Fold object spread `({ ...null })` -> `({})` (#8339) (Boshen)
- 6220e05 minifier: Remove empty if statment `if (test) {}` -> `test` (#8336) (Boshen)
- a76dfae minifier: Remove label statement with empty body (#8333) (Boshen)
- e88a6bd minifier: Minimize `!0 + null !== 1` -> `!0 + null != 1` (#8332) (Boshen)
- ec88c68 minifier: Compress `a || (a = b)` to `a ||= b` (#8315) (sapphi-red)
- e6fe84d minifier: Compress `a = a + b` to `a += b` (#8314) (sapphi-red)
- 9ea4e31 minifier: Remove `new` from `new Error`/`new Function`/`new RegExp` (#8313) (sapphi-red)
- 051fbb6 minifier: Minimize `x["0"]` -> x[0] (#8316) (Boshen)
- a542013 minifier: Minimize `do{}while(true)` -> `do;while(true)` (#8311) (Boshen)
- e3ff81e minifier: Minimize `(x = 1) === 1` -> `(x = 1) == 1` (#8310) (Boshen)
- 4b68cc0 minifier: Minimize empty `try` statement (#8309) (Boshen)
- 922c514 minifier: Fold `.toString()` (#8308) (Boshen)
- 66a2443 minifier: Minify sequence expressions (#8305) (camc314)
- af65c36 minifier: Minimize double negated binary expressions (#8304) (camc314)
- 76c778b minifier: Remove logical nots when arg is a delete expression (#8303) (camc314)
- 5ed439b minifier: Minify typeof in binary expressions (#8302) (camc314)
- 6afc590 minifier: Compress typeof addition string (#8301) (camc314)
- ecc789f minifier: Fold `if(x >> y == 0){}` -> `if(!(x >> y)){}` (#8277) (Boshen)
- 0e3b79a minifier: Fold `String()` -> `''`, `Number()` -> `false` (#8274) (Boshen)
- c9cf593 minifier: Compress  property key `{[1]: _}`  -> {1: _} (#8272) (Boshen)
- b92b2ab minifier: Fold `BigInt(1n)` -> `1n` (#8270) (Boshen)
- a4df387 minifier: Compress loose equals undefined (#8268) (camc314)
- f000596 minifier: Minify call expressionsto `Number` (#8267) (camc314)
- 092aeaf minifier: Flatten spread args in call expressions (#8266) (camc314)
- 04ec38d minifier: Remove unused arrow function expressions (#8262) (camc314)
- e446c15 minifier: Improve minimizing unary not expressions (#8261) (camc314)
- 7f19211 minifier: Minimize unary expression statements (#8256) (camc314)
- cec63e2 minifier: `{}` evals to `f64::NaN` (Boshen)
- 4d8a08d minifier: Improve constant evaluation (#8252) (Boshen)
- e84f267 minifier: Compress more property keys (#8253) (Boshen)
- d1224f9 minifier: Improve minimizing conditional expressions (#8251) (camc314)
- 65f46f5 minifier: Constant fold `String.fromCharCode` (#8248) (Boshen)
- bd8d677 minifier: Minimize `~undefined`, `~null`, `~true`, `~false` (#8247) (Boshen)
- f73dc9e minifier: Constant fold `'x'.toString()` and `true.toString()` (#8246) (Boshen)
- fd5af73 minifier: Minimize `Number` constructor (#8245) (Boshen)
- 2f52f33 minifier: Minsize `!!!foo ? bar : baz` -> `foo ? baz : bar` (#8244) (Boshen)
- ccdc039 minifier: Always put literals on the rhs of equal op `1==x` => `x==1` (#8240) (Boshen)
- 39353b2 minifier: Improve minimizing conditionals (#8238) (Cameron)
- c90fc16 minifier: Restore conditional minification and fix edge case (#8235) (camc314)
- 6c8ee9f minifier: Remove last redundant `return` statement (#8234) (Boshen)
- 51f4792 minifier: Minimize `foo ? foo : bar` and `foo ? bar : foo` (#8229) (Boshen)
- 6e2ec17 minifier: Statement fusion switch cases; improved minimize exit poitns (#8228) (Boshen)
- 574a242 minifier: Minimize all variants of `typeof x == 'undefined'` (#8227) (Boshen)
- 2041477 minifier: Fold `if(x)return;y` -> `if(!x)y` (#8226) (Boshen)
- 9c1afa4 minifier: Optional catch binding when catch param is unused (#8221) (Boshen)
- 4a29845 minifier: Add `ConvertToDottedProperties` (#8212) (Boshen)
- 2786dea minifier: Add `RemoveUnusedCode` (#8210) (Boshen)
- cd274ee minifier: Minimize logical exprs (#8209) (Cameron)
- 4ae15df minifier: Imprve more conditional expr minification with boolean lit (#8208) (camc314)
- 3202b4f minifier: Imprve conditional expr minification with boolean lit (#8207) (camc314)
- 3b45011 minifier: Handle conditional expr with boolean lit (#8206) (camc314)
- 4c2059a minifier: Reverse negated conditional exprs (#8205) (camc314)
- 4804933 minifier: Add `MinimizeExitPoints` and ExploitAssigns` boilerplate (#8203) (Boshen)
- bf266e1 minifier: Try collapse conditional to logical or expr (#8197) (Cameron)
- 06e1780 minifier: Improve `StatementFusion` (#8194) (Boshen)
- 42e211a minifier: Only constant fold numbers when result is smaller (#8092) (Boshen)
- d0de560 minifier: Change `NaN` to `f64::NAN` (#8191) (Boshen)
- cef8eb8 minifier: Change `foo?.['bar']` to `foo?.bar` (#8176) (翠 / green)
- 8149e34 minifier: Optional catch binding when es target >= es2019 (#8180) (Boshen)
- fc43ec5 minifier: Fold `string.length` / `array.length` (#8172) (sapphi-red)
- 29dc0dc minifier: Change `foo['bar']` -> foo.bar (#8169) (Boshen)
- 3c5718d minifier: Fold `typeof foo == undefined` into `foo == undefined` when possible (#8160) (翠 / green)
- f3a36e1 minifier: Fold `typeof foo != "undefined"` into `typeof foo < "u"` (#8159) (翠 / green)
- 37c9959 minifier: Normalize `Infinity` into `f64::Infinity` (#8148) (Boshen)
- 8fb71f5 minifier: Minify string `PropertyKey` (#8147) (Boshen)
- 6615e1e minifier: Constant fold `instanceof` (#8142) (翠 / green)
- 2b2a373 minifier: Minimize `a + 'b' + 'c'` -> `a + 'bc'` (#8137) (Boshen)
- 213364a minifier: Minimize `if (x) if (y) z` -> `if (x && y) z` (#8136) (Boshen)
- 6b51e6d minifier: Minimize `if(foo) bar else baz` -> `foo ? bar : baz` (#8133) (Boshen)
- f615bfa minifier: Minimize `if (x) return; return 1` -> `return x ? void 0 : 1` (#8130) (Boshen)
- f0b1ee5 minifier: Minimize `if(!x) foo()` -> `x || foo()` (#8122) (Boshen)
- f8200a8 minifier: Minimize `if(foo) bar` -> `foo && bar` (#8121) (Boshen)
- 72d9967 minifier: Add `Normalize` ast pass (#8120) (Boshen)
- fef0b25 minifier: Collapse `var` into for loop initializer (#8119) (Boshen)
- 2331ea8 minifier: `typeof foo === 'number'` => `typeof foo == 'number'` (#8112) (Boshen)
- ad9a0a9 mininifier: Minimize variants of `a instanceof b == true` (#8241) (Boshen)

### Bug Fixes

- 74572de ecmascript: Incorrect `to_int_32` value for Infinity (#8144) (翠 / green)
- 5c63414 mangler: Keep exported symbols for `top_level: true` (#7927) (翠 / green)
- 3c93549 minifier: Dce if statement should keep side effects and vars (#8433) (Boshen)
- 52f88c0 minifier: Rotate associative operators to make it more idempotent (#8424) (camc314)
- a80460c minifier: Correctly set `self.changed` when minimizing if stmts (#8420) (camc314)
- d4ca8d4 minifier: `!!x` is not idempotent in `RemoveDeadCode` (#8419) (Boshen)
- 357b61d minifier: Do not minify `Object.defineProperty` in sequence expressions (#8416) (Boshen)
- 0efc845 minifier: `+0n` produces `TypeError` (#8410) (Boshen)
- 7ce6a7c minifier: `a in b` has error throwing side effect (#8406) (Boshen)
- 2f3a9dc minifier: Cannot transform property key `#constructor` (#8405) (Boshen)
- c0a3dda minifier: `instanceof` has error throwing side effect (#8378) (Boshen)
- 5516f7f minifier: Do not fold object comparisons (#8375) (Boshen)
- cb098c7 minifier: Computed property key `prototype` cannot be changed (#8373) (Boshen)
- 82ee77e minifier: Do not remove shadowned `undefined` in return statement (#8371) (Boshen)
- f87da16 minifier: Do not fold literals in `-0 != +0` (#8278) (Boshen)
- 62a2644 minifier: Handle arrow fn expressions correctly in `is_in_boolean_context` (#8260) (camc314)
- d2f8eaa minifier: Fix panic in `peephole_minimize_conditions` (#8242) (Boshen)
- a698def minifier: Fix incorrect return value for `(x ? true : y)` (#8233) (Boshen)
- 56b7f13 minifier: Do not constant fold `0 instanceof F` (#8199) (Boshen)
- 75d5f17 minifier: Minify string `PropertyKey` (#8177) (sapphi-red)

### Documentation

- aaa009d minifier: Clarify assumptions for compressor (#8404) (翠 / green)

### Refactor

- fb2acd8 minifier: Change minimize conditionals into a loop (#8413) (Boshen)
- baaec60 minifier: Remove the buggy `??` transform (#8411) (Boshen)
- 1c4658d minifier: Change ast passes order, `!in_fixed_loop` happen last (#8380) (Boshen)
- 09f0f48 minifier: Remove the buggy `minimize_exit_points` implementation (#8349) (Boshen)
- 9a5c66a minifier: Clean up (#8346) (Boshen)
- 98f2b1c minifier: Clean up `peephole_substitute_alternate_syntax` (#8327) (Boshen)
- fc662b7 minifier: Handle big int values later (#8324) (Boshen)
- d16e598 minifier: Clean up `peephole_replace_known_methods` (#8306) (Boshen)
- b8d26ea minifier: Move optional catch param to peephole_substitute_alternate_syntax (#8282) (Boshen)
- 0845162 minifier: Clean up `ReplaceKnownMethods` (Boshen)
- 7c7f5d7 minifier: Clean up `peephole_fold_constants` (Boshen)
- bf0fbce minifier: Improve constant fold numbers (#8239) (Boshen)
- 62f8fba minifier: Move all conditional minification logic to minimze_conditions (#8231) (camc314)
- cfb51f2 minifier: Fuse ast passes (#8184) (Boshen)
- bf9cafe minifier: Clean up `peephole_substitute_alternate_syntax` a little bit (Boshen)
- 75264ed minifier: Clean up `try_optimize_block` (#8139) (Boshen)
- c22062b minifier: Cleanup peephole_minimize_conditions (#8114) (Boshen)
- e594c39 minifier: Clean up `peephole_substitute_alternate_syntax.rs` (#8111) (Boshen)

### Testing

- 3149fe0 minifier: Add anonymous function test case for logical expression to logical assignment compression (#8347) (sapphi-red)
- 91b42de minifier: Enable some passing tests (#8250) (camc314)
- 1fa5341 minifier: Port tests from ConvertToDottedPropertiesTest (#8175) (sapphi-red)

## [0.44.0] - 2024-12-25

### Features

- 5397fe9 minifier: Constant fold `undefined?.bar` -> `undefined` (#8075) (Boshen)
- 1932f1e minifier: Fold `foo === undefined || foo === null` (#8063) (翠 / green)

### Bug Fixes

- b605baa minifier: Constant fold strings with tab char (#8096) (Boshen)

### Refactor

- 8b54d89 minifier: Remove parens must happen on enter (#8060) (Boshen)
- 7cb84f3 minifier: Only minify on ast node exit (#8059) (Boshen)
- 77d845a minifier: Fuse DCE AST passes (#8058) (Boshen)
- 6123f5e minifier: Fold statements on exit (#8057) (Boshen)

## [0.42.0] - 2024-12-18

### Features

- db9e93b mangler: Mangle top level variables (#7907) (翠 / green)
- 075bd16 minifier: Fold bitwise operation (#7908) (翠 / green)

### Bug Fixes

- 4799471 minfier: Bigint bitwise operation only works with bigint (#7937) (Boshen)
- de8a86e minifier: Incorrect minification in `try_fold_left_child_op` (#7949) (翠 / green)

### Refactor

- 3858221 global: Sort imports (#7883) (overlookmotel)
- 1314c97 minifier: Expose dce as an API instead of an option (#7957) (Boshen)

### Styling

- 7fb9d47 rust: `cargo +nightly fmt` (#7877) (Boshen)

## [0.40.0] - 2024-12-10

- ebc80f6 ast: [**BREAKING**] Change 'raw' from &str to Option<Atom> (#7547) (Song Gao)

### Refactor


## [0.39.0] - 2024-12-04

- f2f31a8 traverse: [**BREAKING**] Remove unsound APIs (#7514) (overlookmotel)

- b0e1c03 ast: [**BREAKING**] Add `StringLiteral::raw` field (#7393) (Boshen)

### Features

- 97af341 minifier: Minify alternated one child if block (#7231) (7086cmd)
- ac0d25c minifier: Minify one child if statement expression (#7230) (Ethan Goh)

### Bug Fixes

- 896ff86 minifier: Do not fold if statement block with lexical declaration (#7519) (Boshen)

### Performance

- c133693 minifier: Fuse ast passes (#7493) (Boshen)

### Refactor

- 63a66cf minifier: Remove unused ast pass from DCE (#7540) (Boshen)
- 625a5ba minifier: Improve ast passes (#7518) (Boshen)

### Testing

- 9d6e14b ecmascript: Move tests to `oxc_minifier` due to cyclic dependency with `oxc_parser` (#7542) (Boshen)

## [0.37.0] - 2024-11-21

### Features

- 39afb48 allocator: Introduce `Vec::from_array_in` (#7331) (overlookmotel)

### Bug Fixes

- cf99be0 minifier: Do not compare bigint with object (#7294) (7086cmd)

### Testing

- 0d6a66a minifier: Fix minimize condition tests (#7222) (7086cmd)

## [0.36.0] - 2024-11-09

### Refactor

- a297765 minifier: Use `map` and `and_then` instead of let else (#7178) (7086cmd)

## [0.35.0] - 2024-11-04

### Refactor

- 97caae1 minifier: Do not use `AstBuilder::*_from_*` methods (#7072) (overlookmotel)
- 2c7ac29 minifier: Remove `Tri`, use `Option<bool>` instead (#6912) (Boshen)

## [0.34.0] - 2024-10-26

### Features

- 4429754 ecmascript: Constant eval `null` to number (#6879) (Boshen)
- fd57e00 ecmascript: Add abstract_relational_comparison to dce (#6846) (Boshen)
- 8bcaf59 minifier: Late peeophole optimization (#6882) (Boshen)
- 860cbca minifier: Implement folding simple arrow fns (#6875) (camc314)
- c26020e minifier: Implement folding String.prototype.replaceAll (#6871) (camc314)
- 50744f3 minifier: Implement folding String.prototype.replace (#6870) (camc314)
- fccf82e minifier: Implement folding `substring` string fns (#6869) (camc314)
- e6a5a1b minifier: Implement folding `charCodeAt` string fns (#6475) (camc314)

### Bug Fixes

- a47c70e minifier: Fix remaining runtime bugs (#6855) (Boshen)
- 686727f minifier: Reference read has side effect (#6851) (Boshen)
- c658d93 minifier: Keep template literals with expressions (#6849) (Boshen)

### Refactor

- 423d54c rust: Remove the annoying `clippy::wildcard_imports` (#6860) (Boshen)

## [0.33.0] - 2024-10-24

### Features

- b4bc300 minifier: Improve folding block stmts (#6793) (camc314)
- 34fe7c0 minifier: Dce meaningless labeled statements (#6688) (7086cmd)

### Bug Fixes

- 2f6ad42 codegen: Print negative bigint `1n- -1n` correctly after constant folding (#6798) (Boshen)
- ca79993 minifier: Do not dce object literals yet (#6839) (Boshen)
- ec5a19b minifier: Do not remove binary expressions (#6829) (Boshen)
- 22355f7 minifier: Do not remove `undefined` for destructuring patterns (#6828) (Boshen)

### Refactor

- 8b25131 minifier: Binary operations use `ConstantEvaluation` (#6700) (Boshen)

## [0.32.0] - 2024-10-19

### Features

- e5ed6a5 codegen: Print negative numbers (#6624) (Boshen)
- 15c04e5 ecmascript: Add feature flag for constant evaluation (Boshen)
- e561880 ecmascript: Add constant_evaluation and side_effects code (#6550) (Boshen)
- 3556062 ecmascript: Add `ConstantEvaluation` (#6549) (Boshen)
- 39c2e66 ecmascript: Add `ToBigInt` and `StringToBigInt` (#6508) (Boshen)
- 6f22538 ecmascript: Add `ToBoolean`, `ToNumber`, `ToString` (#6502) (Boshen)
- 071e564 minifier: Finish implementing folding object expressions (#6586) (camc314)
- 590925a minifier: Finish implementing folding array expressions (#6575) (camc314)
- ef237cf minifier: Complete implementation of statement fusion (#6566) (camc314)
- 97c8a36 minifier: Implement `collapse-variable-declarations` (#6464) (dalaoshu)
- 096e590 minifier: Implement folding `charAt` string fns (#6436) (camc314)
- e5a6f5d minifier: Implement converting template literals to strings (#6486) (camc314)
- 14d0590 minifier: Implement folding of simple function calls (`Boolean`) (#6484) (camc314)
- 7fbc7b6 minifier: Implement folding of simple function calls (`String`) (#6483) (camc314)
- a4f57a4 minifier: Implement folding `indexOf` and `lastIndexOf` string fns (#6435) (camc314)
- 3677ef8 minifier: Dce ExpressionStatements with no side effect (#6457) (7086cmd)
- 06ea121 minifier: Fold for statement (#6450) (7086cmd)
- a9544ae minifier: Partially implement minification for some known string methods (#6424) (camc314)
- 9dc4ee9 minifier: Implement block stmt support for `StatementFusion` (#6422) (camc314)
- ebbf77d minifier: Implement calculations for NumberValue (#6419) (7086cmd)
- 97ac179 minifier: Arithmetic operations for infinity. (#6332) (7086cmd)
- 13b0b0b minifier: Fold literal object constructors on window (#6379) (dalaoshu)

### Bug Fixes

- 389d261 minifier: `~~` operator should only work on numbers (#6598) (Boshen)
- 16bea12 minifier: Use `to_js_string()` instead of `fs64::to_string` (#6597) (Boshen)
- a71e8a0 minifier: Preserve init variable declarations when removing `for` statements during DCE (#6551) (magic-akari)

### Refactor

- 6d041fb ecmascript: Remove `NumberValue` (#6519) (Boshen)
- 856cab5 ecmascript: Move ToInt32 from `oxc_syntax` to `oxc_ecmascript` (#6471) (Boshen)
- f4cdc56 minifier: Use constant folding unary expression from `oxc_ecmascript` (#6647) (Boshen)
- 67ad08a minifier: Unify `ValueType` (#6545) (Boshen)
- bbca743 minifier: Move string methods to `oxc_ecmascript` (#6472) (Boshen)
- 702c049 minifier: Move compress block to dce (#6468) (7086cmd)
- 46a38c6 minifier: Remove allow `clippy::unused_self` (#6441) (Boshen)
- 994b60b minifier: Use builtin get_number_value. (#6335) (7086cmd)
- 435a89c oxc: Remove useless `allocator.alloc(program)` calls (#6571) (Boshen)
- 1a90ec4 rust: Backport v1.82.0 changes to main branch first (#6690) (Boshen)

### Testing

- c5deb32 minifier: Port the rest of tests (#6420) (7086cmd)
- e59da61 minifier: Add all test cases for `collapse_variable_declarations` (#6421) (dalaoshu)
- 73d6a4a minifier: Port all replace_known_methods tests. (#6418) (7086cmd)

## [0.31.0] - 2024-10-08

- 020bb80 codegen: [**BREAKING**] Change to `CodegenReturn::code` and `CodegenReturn::map` (#6310) (Boshen)

- 82ab689 transformer,minifier: [**BREAKING**] Move define and inject plugin from minifier to transformer (#6199) (Boshen)

### Features

- e304e8c minifier: Minify exponential arithmetic operation. (#6281) (7086cmd)
- f9ae70c minifier: Minify basic arithmetic calculations. (#6280) (7086cmd)
- 4008afe minifier: Fold array and object constructors (#6257) (camchenry)
- 115ccc9 minifier: Bitwise not in exceeded value. (#6235) (7086cmd)
- ee6c850 minifier: Scaffold peephole replace known methods. (#6245) (7086cmd)
- c32af57 minifier: Fold demical bitwise not for bigint. (#6233) (7086cmd)
- 23b6464 minifier: Fold true / false comparison. (#6225) (7086cmd)
- 585ccda minifier: Support subtraction assignment. (#6214) (7086cmd)
- cca0034 minifier: Handle positive `NaN` and `Infinity`. (#6207) (7086cmd)
- dac8f09 minifier: Minify unary plus negation. (#6203) (7086cmd)
- 3b79e1b minifier: Evaluate bigint in fold constant (#6178) (Boshen)
- 9e62396 syntax_operations: Add crate `oxc_ecmascript` (#6202) (Boshen)

### Bug Fixes

- d953a6b minifier: Correct the reference link (#6283) (dalaoshu)
- 37cbabb minifier: Should not handle the strict operation for bool comparison. (#6261) (7086cmd)
- e29c067 minifier: Handle exceeded shifts. (#6237) (7086cmd)

### Refactor

- ac5a23f minifier: Use ctx.ast.vec instead of Vec::new. (#6331) (7086cmd)
- 1cee207 minifier: Some boilerplate work for PeepholeFoldConstants (#6054) (Boshen)

### Testing

- 964d71e minifier: Add arithmetic tests for fold constants. (#6269) (7086cmd)
- fcb4651 minifier: Enable null comparison with bigint. (#6252) (7086cmd)

## [0.30.2] - 2024-09-27

### Features

- 60c52ba ast: Allow passing span to `void_0` method (#6065) (Dunqing)

### Bug Fixes

- e0a8959 minifier: Compute `void number` as `undefined` (#6028) (Boshen)

## [0.30.1] - 2024-09-24

### Features

- 5c323a2 minifier: Loop compressor passes (#6013) (Boshen)

### Refactor

- 0a2f687 minifier: Move dce conditional expression to `RemoveDeadCode` (#5971) (Boshen)

## [0.30.0] - 2024-09-23

### Features

- 9076dee minifier: Implement part of `StatementFusion` (#5936) (Boshen)

### Bug Fixes

- 362c427 mangler,codegen: Do not mangle top level symbols (#5965) (Boshen)

### Refactor

- 943bd76 minifier: Move tests to their src files (#5912) (Boshen)
- cbaeea6 minifier: Clean up some tests (#5910) (Boshen)
- 144611e minifier: Align ast pass names with closure compiler (#5908) (Boshen)

## [0.29.0] - 2024-09-13

### Features

- 953fe17 ast: Provide `NONE` type for AST builder calls (#5737) (overlookmotel)
- e968e9f minifier: Constant fold nullish coalescing operator (#5761) (Boshen)
- 6bc13f6 minifier: Add `MinimizeConditions` pass (#5747) (Boshen)

### Bug Fixes

- 8ff013a minifier: Handle dce CallExpression::callee (#5752) (Boshen)

### Performance

- d18c896 rust: Use `cow_utils` instead (#5664) (dalaoshu)

### Refactor

- 2890c98 minifier: Add tests for `remove_syntax` (#5749) (Boshen)
- 9a9d8f6 minifier: Replace `self.ast` with `ctx.ast` (#5748) (Boshen)
- 746f7b3 minifier: Align code with closure compiler (#5717) (Boshen)
- 21e2df5 minifier: Replace `VisitMut` with `Traverse` for inject and define plugins (#5705) (Boshen)

## [0.28.0] - 2024-09-11

- ee4fb42 ast: [**BREAKING**] Reduce size of `WithClause` by `Box`ing it (#5677) (Boshen)

- 4a8aec1 span: [**BREAKING**] Change `SourceType::js` to `SourceType::cjs` and `SourceType::mjs` (#5606) (Boshen)

- b060525 semantic: [**BREAKING**] Remove `source_type` argument from `SemanticBuilder::new` (#5553) (Boshen)

### Features

- 68c3cf5 minifier: Fold `void 1` -> `void 0` (#5670) (Boshen)
- c6bbf94 minifier: Constant fold unary expression (#5669) (Boshen)
- 86256ea minifier: Constant fold `typeof` (#5666) (Boshen)

### Bug Fixes

- b8f8dd6 minifier/replace_global_defines: Do not replace shadowed identifiers (#5691) (Boshen)

### Performance


### Refactor

- 067f9b5 semantic: Introduce `IsGlobalReference` trait (#5672) (Boshen)

## [0.27.0] - 2024-09-06

### Features

- ba4b68c minifier: Remove parenthesized expression for dce (#5439) (Boshen)

## [0.25.0] - 2024-08-23

- 78f135d ast: [**BREAKING**] Remove `ReferenceFlag` from `IdentifierReference` (#5077) (Boshen)

- c4c08a7 ast: [**BREAKING**] Rename `IdentifierReference::reference_flags` field (#5024) (overlookmotel)

- d262a58 syntax: [**BREAKING**] Rename `ReferenceFlag` to `ReferenceFlags` (#5023) (overlookmotel)

- ce4d469 codegen: [**BREAKING**] Remove const generic `MINIFY` (#5001) (Boshen)

### Features

- 2b21be3 oxc_minifier: Define plugin with postfix wildcard (#4979) (IWANABETHATGUY)

### Refactor

- 0f64d10 minifier: Remove duplicated helper `move_out_expression` (#5007) (IWANABETHATGUY)
- b4407c4 oxc,mangler: `oxc` crate add mangler; mangler use options API (Boshen)

## [0.24.3] - 2024-08-18

### Bug Fixes

- 46cb1c1 minifier: Handle `Object.definedPropert(exports` for @babel/types/lib/index.js (#4933) (Boshen)
- 81fd637 minifier: Do not fold `0 && (module.exports = {})` for `cjs-module-lexer` (#4878) (Boshen)
- 879a271 minifier: Do not join `require` calls for `cjs-module-lexer` (#4875) (Boshen)

## [0.24.2] - 2024-08-12

### Performance

- 504ac0b minifier: `InjectGlobalVariables` only add to `replaced_dot_defines` once for each (#4803) (overlookmotel)
- 35f2742 minifier: Avoid repeated `Atom` creation in `InjectGlobalVariables` (#4802) (overlookmotel)

## [0.24.1] - 2024-08-10

### Features

- c519295 minifier: Add `InjectGlobalVariables` plugin (`@rollup/plugin-inject`) (#4759) (Boshen)

## [0.24.0] - 2024-08-08

### Features

- 229a0e9 minifier: Implement dot define for member expressions (#3959) (camc314)

### Bug Fixes

- 94d3c31 minifier: Avoid removing function declaration from `KeepVar` (#4722) (Boshen)
- bf43148 minifier: Do not `remove_syntax` in dead_code_elimination (Boshen)
- bf48c7f minifier: Fix `keep_var` keeping vars from arrow functions (#4680) (Boshen)
- 9be29af minifier: Temporarily fix shadowed `undefined` variable (#4678) (Boshen)
- e8b662a minifier: Various fixes to pass minifier conformance (#4667) (Boshen)

### Performance

- 0f5e982 minifier: Only visit arrow expression after dropping `console.log` (#4677) (Boshen)

### Refactor

- fbfd852 minifier: Add `NodeUtil` trait for accessing symbols on ast nodes (#4734) (Boshen)
- e0832f8 minifier: Use `oxc_traverse` for AST passes (#4725) (Boshen)
- 17602db minifier: Move tests and files around (Boshen)
- 3289477 minifier: Clean up tests (#4724) (Boshen)
- e78cba6 minifier: Ast passes infrastructure (#4625) (Boshen)

## [0.23.1] - 2024-08-06

### Features

- 229a0e9 minifier: Implement dot define for member expressions (#3959) (camc314)

### Bug Fixes

- bf48c7f minifier: Fix `keep_var` keeping vars from arrow functions (#4680) (Boshen)
- 9be29af minifier: Temporarily fix shadowed `undefined` variable (#4678) (Boshen)
- e8b662a minifier: Various fixes to pass minifier conformance (#4667) (Boshen)

### Performance

- 0f5e982 minifier: Only visit arrow expression after dropping `console.log` (#4677) (Boshen)

### Refactor

- e78cba6 minifier: Ast passes infrastructure (#4625) (Boshen)

## [0.23.0] - 2024-08-01

### Features

- a558492 codegen: Implement `BinaryExpressionVisitor` (#4548) (Boshen)

### Bug Fixes

- 6a94e3f codegen: Fixes for esbuild test cases (#4503) (Boshen)

## [0.22.0] - 2024-07-23

### Features

- 0deb027 minfier: Dce `if (xxx) else if (false) { REMOVE }` (#4407) (Boshen)
- e33ec18 minifier: Compress `typeof foo == "undefined"` into `typeof foo > "u"` (#4412) (Boshen)

### Bug Fixes

- 267f7c4 minifier: Skip `Object.defineProperty(exports, ...)` for `cjs-module-lexer` (#4409) (Boshen)

## [0.21.0] - 2024-07-18

### Features

- 83c2c62 codegen: Add option for choosing quotes; remove slow `choose_quot` method (#4219) (Boshen)
- 5d17675 mangler: Add debug mode (#4314) (Boshen)
- e3e663b mangler: Initialize crate and integrate into minifier (#4197) (Boshen)
- c818472 minifier: Dce conditional expression `&&` or `||` (#4190) (Boshen)

### Bug Fixes

- e167ef7 codegen: Print parenthesis properly (#4245) (Boshen)
- f144082 minifier: RemoveDeadCode should visit nested expression (#4268) (underfin)

### Refactor

- 2c7bb9f ast: Pass final `ScopeFlags` into `visit_function` (#4283) (overlookmotel)

## [0.20.0] - 2024-07-11

### Features

- 54cd04a minifier: Implement dce with var hoisting (#4160) (Boshen)
- 44a894a minifier: Implement return statement dce (#4155) (Boshen)

## [0.19.0] - 2024-07-09

- b936162 ast/ast_builder: [**BREAKING**] Shorter allocator utility method names. (#4122) (rzvxa)

### Refactor


## [0.18.0] - 2024-07-09

- d347aed ast: [**BREAKING**] Generate `ast_builder.rs`. (#3890) (rzvxa)

### Features

- c6c16a5 minifier: Dce all conditional expressions (#4135) (Boshen)

## [0.17.1] - 2024-07-06

### Bug Fixes

- 719fb96 minifier: Omit dce `undefined` which can be a shadowed variable (#4073) (Boshen)

## [0.17.0] - 2024-07-05

### Features

- 0da9dfb minifier: Add constant folding to remove dead code (#4058) (Boshen)

### Bug Fixes

- aaac2d8 codegen: Preserve parentheses from AST instead calculating from  operator precedence (#4055) (Boshen)

### Refactor

- edb557c minifier: Add a folder struct for constant folding (#4057) (Boshen)

## [0.16.2] - 2024-06-30

### Performance

- 1eac3d2 semantic: Use `Atom<'a>` for `Reference`s (#3972) (Don Isaac)

## [0.16.0] - 2024-06-26

- 6796891 ast: [**BREAKING**] Rename all instances of `BigintLiteral` to `BigIntLiteral`. (#3898) (rzvxa)

### Features

- dd540c8 minifier: Add skeleton for ReplaceGlobalDefines ast pass (#3803) (Boshen)
- f3c3970 minifier: Add skeleton for RemoveDeadCode ast pass (#3802) (Boshen)

### Bug Fixes


### Refactor

- 8027b1e minifier: Change prepass to ast_passes::remove_parens (#3801) (Boshen)

## [0.15.0] - 2024-06-18

- 5c38a0f codegen: [**BREAKING**] New code gen API (#3740) (Boshen)

- 534242a codegen: [**BREAKING**] Remove `CodegenOptions::enable_typescript` (#3674) (Boshen)

### Features

- 38a75e5 coverage: Improve codegen (#3729) (Boshen)

### Bug Fixes

- 8f64d99 minifier: Respect `join_vars: false` option (#3724) (mysteryven)

## [0.14.0] - 2024-06-12

### Refactor

- e90e6a2 minifier: Make `Prepass` `Copy` (#3603) (overlookmotel)

## [0.13.4] - 2024-06-07

### Bug Fixes

- affb2c8 codegen: Print indentation before directive (#3512) (Dunqing)

## [0.13.2] - 2024-06-03

### Features

- 0cdb45a oxc_codegen: Preserve annotate comment (#3465) (IWANABETHATGUY)

## [0.13.1] - 2024-05-22

### Features

- e2dd8ac syntax: Export `is_reserved_keyword` and `is_global_object` method (#3384) (Boshen)

## [0.13.0] - 2024-05-14

### Refactor

- 7e1fe36 ast: Squash nested enums (#3115) (overlookmotel)
- a8af5de syntax: Move number related functions to number module (#3130) (Boshen)

## [0.11.0] - 2024-03-30

### Refactor

- fc38783 ast: Add walk_mut functions (#2776) (Ali Rezvani)
- d9b77d8 sourcemap: Change sourcemap name to take a reference (#2779) (underfin)

## [0.10.0] - 2024-03-14

### Features

- 4f9dd98 span: Remove `From<String>` and `From<Cow>` API because they create memory leak (#2628) (Boshen)

### Refactor
- cbc2f5f Remove unused dependencies (#2718) (Boshen)- 8001b2f Make `CompactStr` immutable (#2620) (overlookmotel)- 0646bf3 Rename `CompactString` to `CompactStr` (#2619) (overlookmotel)

## [0.9.0] - 2024-03-05

### Refactor

- ef932a3 codegen: Clean up API around building sourcemaps (#2602) (Boshen)
- 903f17c span: Move base54 method to mangler (#2523) (Boshen)

## [0.8.0] - 2024-02-26

### Features

- 6b3b260 Codegen: Improve codegen (#2460) (Andrew McClenaghan)
- e6d536c codegen: Configurable typescript codegen (#2443) (Andrew McClenaghan)

### Refactor

- d08abc6 ast: S/NumberLiteral/NumericLiteral to align with estree (Boshen)
- e6b391a ast: S/ArrowExpression/ArrowFunctionExpression to align estree (Boshen)- a2c173d Remove `panic!` from examples (#2454) (Boshen)- 70a0076 Remove global allocator from non-user facing apps (#2401) (Boshen)

## [0.7.0] - 2024-02-09

### Refactor

- 1822cfe ast: Fix BigInt memory leak by removing it (#2293) (Boshen)

## [0.6.0] - 2024-02-03

### Features

- 1ee6d8c codegen: Move string test to codegen (#2150) (Wenzhe Wang)
- 18a58d4 minifier: Handle more expressions for side effects (#2062) (Bradley Farias)

### Bug Fixes

- 29dc5e6 codegen: Add parenthesis in binary expression by precedence (#2067) (Wenzhe Wang)

## [0.4.0] - 2023-12-08

### Features

- c6ad660 semantic: Support scope descendents starting from a certain scope. (#1629) (Miles Johnson)

### Refactor

- 1a576f6 rust: Move to workspace lint table (#1444) (Boshen)

## [0.3.0] - 2023-11-06

### Features

- e0ca09b codegen: Implement the basics of non-minifying codegen (#987) (Boshen)
- 809f050 codegen: Move minifying printer to codegen crate (#985) (Boshen)
- ef8aaa7 minifier: Re-enable mangler (#972) (Boshen)
- 14e1dac minifier: Reenable minifier tests (#969) (Boshen)
- f0029d5 minifier: Reenable mangler (Boshen)
- 55b2f03 minifier: Partially re-enable minifier (#963) (Boshen)
- 5b1e1e5 parser: TypeScript 5.2 (#811) (Cameron)
- 2e2b758 playground: Add transform and minify (#993) (Boshen)
- ce79bc1 transform_conformance: Move Formatter to codegen (#986) (Boshen)
- 678db1d transformer: ES2020 Nullish Coalescing Operator (#1004) (Boshen)
- 0f72066 transformer: Finish 2016 exponentiation operator (#996) (Boshen)- 0e91044 Adjust the order of print semicolon (#1003) (Wenzhe Wang)

### Refactor

- 4787220 ast: Clean up some methods (Boshen)
- 903854d ast: Fix the lifetime annotations around Vist and VisitMut (#973) (Boshen)
- db5417f clippy: Allow clippy::too_many_lines (Boshen)
- eaeb630 clippy: Allow struct_excessive_bools (Boshen)
- 801d78a minifier: Make the minifier api only accept an ast (#990) (Boshen)
- 110059f rust: Change `RefCell.clone().into_inner()` to `RefCell.get()` (Boshen)

## [0.2.0] - 2023-09-14

### Features

- 027a67d minifier: Constant addition expression folding (#882) (Don Isaac)
- e090b56 minifier: Initialize conditions folding (#658) (阿良仔)
- c5ff534 semantic: Add `node_id` to `Reference` (#689) (Makoto Tateno)

### Refactor

- 3516759 ast: Use `atom` for `Directive` and `Hashbang` (#701) (Yunfei He)

