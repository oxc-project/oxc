commit: 4f4ef5d4

node: v26.5.0

Passed: 39 of 72 (54.17%)

Failures:

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-2023-11-exec--to-es2015-accessor-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-2023-11-exec--to-es2015-accessor-exec.test.js:21
	  @dec accessor x = 1;
	  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-2023-11-exec--to-es2015-accessor-without-decorators-exec.test.js
RolldownError: Parse failure: Parse failed with 1 error:
Unexpected token
10:     accessor #d = 4;
11:     getD() {
12:       return _classPrivateFieldGet(, this);
                                       ^
13:     }
14:     setD(v) {
At file: /fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-2023-11-exec--to-es2015-accessor-without-decorators-exec.test.js:12:34
    at error (./node_modules/rolldown/dist/shared/logs-D6uA606S.mjs:166:24)
    at normalizeParseError (./node_modules/rolldown/dist/parse-ast-index.mjs:25:9)
    at wrap (./node_modules/rolldown/dist/parse-ast-index.mjs:5:39)
    at parseAstAsync (./node_modules/rolldown/dist/parse-ast-index.mjs:54:9)
    at ssrTransformScript (./node_modules/vite/dist/node/chunks/node.js:16392:9)
    at loadAndTransform (./node_modules/vite/dist/node/chunks/node.js:20032:64)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-2023-11-exec--to-es2015-class-binding-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-2023-11-exec--to-es2015-class-binding-exec.test.js:12
	@capture(() => Foo) class Foo {
	^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-2023-11-exec--to-es2015-class-decorator-replacement-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-2023-11-exec--to-es2015-class-decorator-replacement-exec.test.js:21
	@replace class Foo {
	^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-2023-11-exec--to-es2015-class-expression-name-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-2023-11-exec--to-es2015-class-expression-name-exec.test.js:11
	const A = @dec class {};
	          ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-2023-11-exec--to-es2015-decorator-this-receiver-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-2023-11-exec--to-es2015-decorator-this-receiver-exec.test.js:13
	@obj.dec class Foo {
	^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-2023-11-exec--to-es2015-derived-class-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-2023-11-exec--to-es2015-derived-class-exec.test.js:19
	  @dec method() {}
	  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-2023-11-exec--to-es2015-evaluation-order-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-2023-11-exec--to-es2015-evaluation-order-exec.test.js:20
	@dec(0) class Foo extends (log.push(1), Object) {
	^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-2023-11-exec--to-es2015-field-initializers-exec.test.js
AssertionError: expected 1 to be 10 // Object.is equality
    at ./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-2023-11-exec--to-es2015-field-initializers-exec.test.js:28:16

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-2023-11-exec--to-es2015-initializer-order-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-2023-11-exec--to-es2015-initializer-order-exec.test.js:20
	@dec("class") class Foo {
	^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-2023-11-exec--to-es2015-metadata-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-2023-11-exec--to-es2015-metadata-exec.test.js:10
	@dec class Base {
	^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-2023-11-exec--to-es2015-methods-getters-setters-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-2023-11-exec--to-es2015-methods-getters-setters-exec.test.js:27
	  @dec method() {
	  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-2023-11-exec--to-es2015-nested-class-private-names-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-2023-11-exec--to-es2015-nested-class-private-names-exec.test.js:13
	  @dec method() {
	  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-2023-11-exec--to-es2015-private-elements-exec.test.js
RolldownError: Parse failure: Parse failed with 1 error:
Unexpected token
27:         _assertClassBrand(_Foo_brand, this, _method).call(this),
28:         _get_getter.call(_assertClassBrand(_Foo_brand, this)),
29:         _classPrivateFieldGet(, this),
                                  ^
30:         _sMethod.call(Foo)
31:       ];
At file: /fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-2023-11-exec--to-es2015-private-elements-exec.test.js:29:29
    at error (./node_modules/rolldown/dist/shared/logs-D6uA606S.mjs:166:24)
    at normalizeParseError (./node_modules/rolldown/dist/parse-ast-index.mjs:25:9)
    at wrap (./node_modules/rolldown/dist/parse-ast-index.mjs:5:39)
    at parseAstAsync (./node_modules/rolldown/dist/parse-ast-index.mjs:54:9)
    at ssrTransformScript (./node_modules/vite/dist/node/chunks/node.js:16392:9)
    at loadAndTransform (./node_modules/vite/dist/node/chunks/node.js:20032:64)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-2023-11-exec-accessor-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-2023-11-exec-accessor-exec.test.js:21
	  @dec accessor x = 1;
	  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-2023-11-exec-accessor-without-decorators-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-2023-11-exec-accessor-without-decorators-exec.test.js:6
	  accessor a = 1;
	           ^

SyntaxError: Unexpected identifier 'a'
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-2023-11-exec-class-binding-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-2023-11-exec-class-binding-exec.test.js:11
	@capture(() => Foo) class Foo {
	^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-2023-11-exec-class-decorator-replacement-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-2023-11-exec-class-decorator-replacement-exec.test.js:16
	@replace class Foo {
	^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-2023-11-exec-class-expression-name-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-2023-11-exec-class-expression-name-exec.test.js:10
	const A = @dec class {};
	          ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-2023-11-exec-decorator-this-receiver-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-2023-11-exec-decorator-this-receiver-exec.test.js:12
	@obj.dec class Foo {
	^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-2023-11-exec-derived-class-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-2023-11-exec-derived-class-exec.test.js:18
	  @dec y = this.x + 1;
	  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-2023-11-exec-evaluation-order-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-2023-11-exec-evaluation-order-exec.test.js:14
	@dec(0) class Foo extends (log.push(1), Object) {
	^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-2023-11-exec-field-initializers-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-2023-11-exec-field-initializers-exec.test.js:19
	  @dec("a") a = 1;
	  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-2023-11-exec-initializer-order-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-2023-11-exec-initializer-order-exec.test.js:19
	@dec("class") class Foo {
	^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-2023-11-exec-metadata-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-2023-11-exec-metadata-exec.test.js:9
	@dec class Base {
	^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-2023-11-exec-methods-getters-setters-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-2023-11-exec-methods-getters-setters-exec.test.js:21
	  @dec method() {
	  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-2023-11-exec-nested-class-private-names-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-2023-11-exec-nested-class-private-names-exec.test.js:8
	  @dec method() {
	  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-2023-11-exec-private-elements-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-2023-11-exec-private-elements-exec.test.js:14
	  @dec #field = 1;
	  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-2023-11-exec-typescript-accessor-types-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-2023-11-exec-typescript-accessor-types-exec.test.js:12
	  @tracked accessor name = "a";
	  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-2023-11-exec-typescript-parameter-properties-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-2023-11-exec-typescript-parameter-properties-exec.test.js:18
	  @logged greet() {
	  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-2023-11-exec-typescript-type-only-members-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-2023-11-exec-typescript-type-only-members-exec.test.js:9
	  @dec count = 1;
	  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/oxc/babel-plugin-transform-class-properties-test-fixtures-private-field-resolve-to-method-in-computed-key-exec.test.js
AssertionError: expected [Function] to throw error including 'Receiver must be an instance of class…' but got 'Private element is not present on thi…'
    at Proxy.methodWrapper (./node_modules/chai/index.js:1700:25)
    at ./tasks/transform_conformance/fixtures/oxc/babel-plugin-transform-class-properties-test-fixtures-private-field-resolve-to-method-in-computed-key-exec.test.js:96:33

./fixtures/oxc/babel-plugin-transform-class-properties-test-fixtures-static-super-tagged-template-exec.test.js
AssertionError: expected undefined to be [Function C] // Object.is equality
    at ./tasks/transform_conformance/fixtures/oxc/babel-plugin-transform-class-properties-test-fixtures-static-super-tagged-template-exec.test.js:15:17
