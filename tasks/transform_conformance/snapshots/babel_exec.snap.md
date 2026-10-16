commit: 4f4ef5d4

node: v26.5.0

Passed: 319 of 581 (54.91%)

Failures:

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-accessors--to-es2015-context-name-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-accessors--to-es2015-context-name-exec.test.js:14
	  @dec static accessor a;
	  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-accessors--to-es2015-private-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-accessors--to-es2015-private-exec.test.js:24
	  @dec accessor #a;
	  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-accessors--to-es2015-public-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-accessors--to-es2015-public-exec.test.js:24
	  @dec accessor a;
	  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-accessors--to-es2015-static-private-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-accessors--to-es2015-static-private-exec.test.js:24
	  @dec static accessor #a;
	  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-accessors--to-es2015-static-public-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-accessors--to-es2015-static-public-exec.test.js:24
	  @dec static accessor a;
	  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-accessors--to-es2015-static-public-this-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-accessors--to-es2015-static-public-this-exec.test.js:6
	  static accessor x;
	                  ^

SyntaxError: Unexpected identifier 'x'
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-accessors--to-es2015-undecorated-private-exec.test.js
RolldownError: Parse failure: Parse failed with 1 error:
Unexpected token
 8:     accessor #b = 123;
 9:     getA() {
10:       return _classPrivateFieldGet(, this);
                                       ^
11:     }
12:     setA(v) {
At file: /fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-accessors--to-es2015-undecorated-private-exec.test.js:10:34
    at error (./node_modules/rolldown/dist/shared/logs-D6uA606S.mjs:166:24)
    at normalizeParseError (./node_modules/rolldown/dist/parse-ast-index.mjs:25:9)
    at wrap (./node_modules/rolldown/dist/parse-ast-index.mjs:5:39)
    at parseAstAsync (./node_modules/rolldown/dist/parse-ast-index.mjs:54:9)
    at ssrTransformScript (./node_modules/vite/dist/node/chunks/node.js:16392:9)
    at loadAndTransform (./node_modules/vite/dist/node/chunks/node.js:20032:64)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-accessors--to-es2015-undecorated-public-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-accessors--to-es2015-undecorated-public-exec.test.js:6
	  accessor a;
	           ^

SyntaxError: Unexpected identifier 'a'
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-accessors--to-es2015-undecorated-static-public-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-accessors--to-es2015-undecorated-static-public-exec.test.js:6
	  static accessor a;
	                  ^

SyntaxError: Unexpected identifier 'a'
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-accessors-context-name-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-accessors-context-name-exec.test.js:14
	  @dec static accessor a;
	  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-accessors-static-public-inherited-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-accessors-static-public-inherited-exec.test.js:6
	  static accessor x;
	                  ^

SyntaxError: Unexpected identifier 'x'
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-ctx-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-ctx-exec.test.js:9
	@dec class A {}
	^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-decorator-access-modified-classes-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-decorator-access-modified-classes-exec.test.js:19
	@decorateB @decorateA class Target {}
	^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-decorator-access-modified-fields-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-decorator-access-modified-fields-exec.test.js:12
	@classDec class C {
	^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-decorator-access-modified-methods-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-decorator-access-modified-methods-exec.test.js:11
	@classDec class C {
	^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-inheritance-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-inheritance-exec.test.js:10
	@dec1 class Bar {}
	^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-initializers-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-initializers-exec.test.js:13
	@dec1 class Foo {}
	^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-exec.test.js:12
	@dec class Foo {}
	^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-block-super-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-block-super-exec.test.js:41
	      @dec class B extends idFactory("B") {
	      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-decorator-initializer-this-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-decorator-initializer-this-exec.test.js:22
	@dec class Foo {
	^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-installed-on-correct-class-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-installed-on-correct-class-exec.test.js:14
	@dec class Foo {
	^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-method-private-environment-super-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-method-private-environment-super-exec.test.js:43
	      @dec class B extends idFactory("B") {
	      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-method-private-instance-element-super-assumption-ignoreFunctionLength-exec.test.js
Error: Transform error
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-method-private-instance-element-super-assumption-ignoreFunctionLength-exec.test.js:4:8

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-method-private-instance-element-super-exec.test.js
RolldownError: Parse failure: Parse failed with 1 error:
Unexpected token
49:       getX = _superPropGet(_Foo, "id", this, 2)([(o) => _classPrivateFieldGet(_x, o)]);
50:       setX = _superPropGet(_Foo, "id", this, 2)([(o, v) => _classPrivateFieldSet(_x, o, v)]);
51:       hasA = _superPropGet(_Foo, "id", this, 2)([(o) => .has(_checkInRHS(o))]);
                                                            ^
52:       getA = _superPropGet(_Foo, "id", this, 2)([(o) => _classPrivateFieldGet(, o)]);
53:       setA = _superPropGet(_Foo, "id", this, 2)([(o, v) => _classPrivateFieldSet(, o, v)]);
At file: /fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-method-private-instance-element-super-exec.test.js:51:55
    at error (./node_modules/rolldown/dist/shared/logs-D6uA606S.mjs:166:24)
    at normalizeParseError (./node_modules/rolldown/dist/parse-ast-index.mjs:25:9)
    at wrap (./node_modules/rolldown/dist/parse-ast-index.mjs:5:39)
    at parseAstAsync (./node_modules/rolldown/dist/parse-ast-index.mjs:54:9)
    at ssrTransformScript (./node_modules/vite/dist/node/chunks/node.js:16392:9)
    at loadAndTransform (./node_modules/vite/dist/node/chunks/node.js:20032:64)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-private-instance-element-access-exec.test.js
RolldownError: Parse failure: Parse failed with 1 error:
Unexpected token
39:       getX = (o) => _classPrivateFieldGet(_x, o);
40:       setX = (o, v) => _classPrivateFieldSet(_x, o, v);
41:       hasA = (o) => .has(_checkInRHS(o));
                        ^
42:       getA = (o) => _classPrivateFieldGet(, o);
43:       setA = (o, v) => _classPrivateFieldSet(, o, v);
At file: /fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-private-instance-element-access-exec.test.js:41:19
    at error (./node_modules/rolldown/dist/shared/logs-D6uA606S.mjs:166:24)
    at normalizeParseError (./node_modules/rolldown/dist/parse-ast-index.mjs:25:9)
    at wrap (./node_modules/rolldown/dist/parse-ast-index.mjs:5:39)
    at parseAstAsync (./node_modules/rolldown/dist/parse-ast-index.mjs:54:9)
    at ssrTransformScript (./node_modules/vite/dist/node/chunks/node.js:16392:9)
    at loadAndTransform (./node_modules/vite/dist/node/chunks/node.js:20032:64)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-private-instance-element-destructuring-exec.test.js
Error: Transform error
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-private-instance-element-destructuring-exec.test.js:4:8

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-property-super-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-property-super-exec.test.js:43
	      @dec class B extends idFactory("B") {
	      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-this-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-this-exec.test.js:9
	@((() => Bar)) class Foo {}
	^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-with-expr-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-with-expr-exec.test.js:12
	const Foo = (_Bar = @dec class Bar {}, _defineProperty(_Bar, "bar", new _Bar()), _Bar);
	                    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-ctx-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-ctx-exec.test.js:9
	@dec class A {}
	^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-decorator-access-modified-classes-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-decorator-access-modified-classes-exec.test.js:19
	@decorateB @decorateA class Target {}
	^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-decorator-access-modified-fields-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-decorator-access-modified-fields-exec.test.js:11
	@classDec class C {
	^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-decorator-access-modified-methods-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-decorator-access-modified-methods-exec.test.js:11
	@classDec class C {
	^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-expressions-named-evaluation-exec.test.js
Error: Cannot read properties of null (reading 'name')
    at reviveInvokeError (./node_modules/vite/dist/node/module-runner.js:542:14)
    at Object.invoke (./node_modules/vite/dist/node/module-runner.js:558:33)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.getModuleInformation (./node_modules/vite/dist/node/module-runner.js:1212:7)
    at VitestModuleRunner.import (./node_modules/vite/dist/node/module-runner.js:1133:23)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-replacement-static-block-super-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-replacement-static-block-super-exec.test.js:26
	      @dec class B extends idFactory("B") {
	      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-replacement-static-elements-super-this-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-replacement-static-elements-super-this-exec.test.js:10
	      static accessor ["x" + hint] = hint + "Super" + ".x";
	                      ^

SyntaxError: Unexpected token '['
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-replacement-static-method-private-environment-super-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-replacement-static-method-private-environment-super-exec.test.js:26
	      @dec class B extends idFactory("B") {
	      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-replacement-static-method-private-instance-element-super-assumption-ignoreFunctionLength-exec.test.js
Error: Transform error
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-replacement-static-method-private-instance-element-super-assumption-ignoreFunctionLength-exec.test.js:4:8

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-replacement-static-method-private-instance-element-super-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-replacement-static-method-private-instance-element-super-exec.test.js:17
	  @dec class Foo extends class {} {
	  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-replacement-static-private-instance-element-access-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-replacement-static-private-instance-element-access-exec.test.js:13
	  @dec class Foo {
	  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-replacement-static-private-instance-element-destructuring-exec.test.js
Error: Transform error
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-replacement-static-private-instance-element-destructuring-exec.test.js:4:8

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-replacement-static-property-super-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-replacement-static-property-super-exec.test.js:26
	      @dec class B extends idFactory("B") {
	      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-duplicated-keys--to-es2015-computed-keys-same-ast-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-duplicated-keys--to-es2015-computed-keys-same-ast-exec.test.js:17
	  @dec [getKey()]() {
	  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-duplicated-keys--to-es2015-computed-keys-same-value-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-duplicated-keys--to-es2015-computed-keys-same-value-exec.test.js:14
	    @dec [getKey()]() {
	    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-duplicated-keys--to-es2015-method-and-field-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-duplicated-keys--to-es2015-method-and-field-exec.test.js:17
	  @dec a() {
	  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-duplicated-keys--to-es2015-methods-with-same-key-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-duplicated-keys--to-es2015-methods-with-same-key-exec.test.js:11
	    @dec a() {
	    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-fields--to-es2015-context-name-exec.test.js
AssertionError: expected [ Array(2) ] to strictly equal [ 'computing f', …(10) ]
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-fields--to-es2015-context-name-exec.test.js:25:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-fields--to-es2015-private-exec.test.js
TypeError: Cannot read properties of undefined (reading 'access')
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-fields--to-es2015-private-exec.test.js:22:18

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-fields--to-es2015-public-exec.test.js
TypeError: Cannot read properties of undefined (reading 'access')
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-fields--to-es2015-public-exec.test.js:22:18

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-fields--to-es2015-static-private-exec.test.js
TypeError: Cannot read properties of undefined (reading 'access')
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-fields--to-es2015-static-private-exec.test.js:15:18

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-fields--to-es2015-static-public-exec.test.js
TypeError: Cannot read properties of undefined (reading 'access')
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-fields--to-es2015-static-public-exec.test.js:18:18

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-fields-context-name-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-fields-context-name-exec.test.js:14
	  @dec static a;
	  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-getters--to-es2015-context-name-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-getters--to-es2015-context-name-exec.test.js:16
	  @dec static get a() {}
	  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-getters--to-es2015-private-exec.test.js
TypeError: Cannot read properties of undefined (reading 'access')
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-getters--to-es2015-private-exec.test.js:31:18

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-getters--to-es2015-public-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-getters--to-es2015-public-exec.test.js:19
	  @dec get a() {
	  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-getters--to-es2015-static-private-exec.test.js
TypeError: Cannot read properties of undefined (reading 'access')
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-getters--to-es2015-static-private-exec.test.js:25:18

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-getters--to-es2015-static-public-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-getters--to-es2015-static-public-exec.test.js:16
	  @dec static get a() {
	  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-getters-and-setters--to-es2015-private-exec.test.js
TypeError: Cannot read properties of undefined (reading 'access')
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-getters-and-setters--to-es2015-private-exec.test.js:44:25

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-getters-and-setters--to-es2015-public-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-getters-and-setters--to-es2015-public-exec.test.js:24
	  @dec get a() {
	  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-getters-and-setters--to-es2015-static-private-exec.test.js
TypeError: Cannot read properties of undefined (reading 'access')
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-getters-and-setters--to-es2015-static-private-exec.test.js:38:25

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-getters-and-setters--to-es2015-static-public-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-getters-and-setters--to-es2015-static-public-exec.test.js:21
	  @dec static get a() {
	  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-getters-context-name-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-getters-context-name-exec.test.js:14
	  @dec static get a() {}
	  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-methods--to-es2015-context-name-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-methods--to-es2015-context-name-exec.test.js:16
	  @dec static a() {}
	  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-methods--to-es2015-private-async-and-generator-exec.test.js
AssertionError: expected +0 to be 3 // Object.is equality
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-methods--to-es2015-private-async-and-generator-exec.test.js:15:18

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-methods--to-es2015-private-exec.test.js
TypeError: Cannot read properties of undefined (reading 'access')
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-methods--to-es2015-private-exec.test.js:29:18

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-methods--to-es2015-private-with-initializers-exec.test.js
TypeError: Cannot read properties of undefined (reading 'access')
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-methods--to-es2015-private-with-initializers-exec.test.js:31:18

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-methods--to-es2015-public-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-methods--to-es2015-public-exec.test.js:18
	  @dec a() {
	  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-methods--to-es2015-public-with-initializers-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-methods--to-es2015-public-with-initializers-exec.test.js:19
	  @dec a() {
	  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-methods--to-es2015-static-private-exec.test.js
TypeError: Cannot read properties of undefined (reading 'access')
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-methods--to-es2015-static-private-exec.test.js:23:18

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-methods--to-es2015-static-private-with-initializers-exec.test.js
TypeError: Cannot read properties of undefined (reading 'access')
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-methods--to-es2015-static-private-with-initializers-exec.test.js:25:18

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-methods--to-es2015-static-public-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-methods--to-es2015-static-public-exec.test.js:15
	  @dec static a() {
	  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-methods--to-es2015-static-public-with-initializers-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-methods--to-es2015-static-public-with-initializers-exec.test.js:16
	  @dec static a() {
	  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-methods-context-name-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-methods-context-name-exec.test.js:14
	  @dec static a() {}
	  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-methods-private-async-and-generator-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-methods-private-async-and-generator-exec.test.js:7
	  @((function(fn) {
	  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-class-and-method-decorators-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-class-and-method-decorators-exec.test.js:17
	new @dec class C1 {
	    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-class-and-property-decorators-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-class-and-property-decorators-exec.test.js:18
	new (_C = @dec class C1 {}, _defineProperty(_C, "p", void 0), _C)();
	          ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-class-in-for-loop-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-class-in-for-loop-exec.test.js:29
	    classes.push((_i2 = i, @noop class C {
	                           ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-class-underscore-property-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-class-underscore-property-exec.test.js:26
	  @noopFactory() class C extends B {
	  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-ctx-access-public-other-this-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-ctx-access-public-other-this-exec.test.js:16
	  @dec static accessor a;
	  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-decorator-evaluation-arguments-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-decorator-evaluation-arguments-exec.test.js:33
	    @noop class C {
	    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-decorator-evaluation-new-target-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-decorator-evaluation-new-target-exec.test.js:32
	    @noop class C {
	    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-decorator-evaluation-private-environment-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-decorator-evaluation-private-environment-exec.test.js:28
	        @(capture((_) => _assertClassBrand(_B, _, _X2)._, (_) => _classPrivateFieldGet(_x2, _)), dummy) method() {}
	        ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-decorator-evaluation-super-getter-setter-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-decorator-evaluation-super-getter-setter-exec.test.js:52
	        @(super.setter = super.getter, noop) accessor [(noop(), "p")];
	        ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-decorator-evaluation-super-property-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-decorator-evaluation-super-property-exec.test.js:42
	        @(receivedName = super.name, noop) accessor [(noop(), "p")];
	        ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-decorator-evaluation-this-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-decorator-evaluation-this-exec.test.js:35
	      @noop class C {
	      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-decorator-evaluation-yield-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-decorator-evaluation-yield-exec.test.js:13
	      @(yield 1, noop) method() {}
	      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-decorator-evaluation-yield-private-super-property-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-decorator-evaluation-yield-private-super-property-exec.test.js:26
	        @(yield* super.id((_) => _assertClassBrand(_B, _, _X2)._), dummy) method() {}
	        ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-initField-existing-derived-constructor-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-initField-existing-derived-constructor-exec.test.js:23
	    @deco accessor foo = 42;
	    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-initProto-existing-derived-constructor-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-initProto-existing-derived-constructor-exec.test.js:23
	    @deco method() {}
	    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-initializer-property-ignored-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-initializer-property-ignored-exec.test.js:20
	  @decorator accessor x;
	  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-initializer-timing-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-initializer-timing-exec.test.js:16
	  @dec1 foo() {}
	  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-invalid-non-callable-decorators-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-invalid-non-callable-decorators-exec.test.js:13
	  (0,__vite_ssr_import_0__.expect)(() => @nonCallable class {}).toThrow("A decorator must be a function");
	                                         ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-leaked-context-addInitializer-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-leaked-context-addInitializer-exec.test.js:15
	    @callCapturedFunc @decMethod m() {}
	    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-leaked-context-addInitializer-throw-2-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-leaked-context-addInitializer-throw-2-exec.test.js:12
	    @decMethod m() {}
	    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-leaked-context-addInitializer-throw-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-leaked-context-addInitializer-throw-exec.test.js:12
	    @decMethod m() {}
	    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-outer-class-binding-mutated-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-outer-class-binding-mutated-exec.test.js:66
	  @capture(() => K) @assertUninitialized(() => K) class K {
	  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-private-name-in-class-decorator-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-private-name-in-class-decorator-exec.test.js:8
	    @(_x) class B extends A {}
	    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-this-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-this-exec.test.js:14
	@o1.dec @dec @o2.dec class A {
	^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-class-in-for-loop-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-class-in-for-loop-exec.test.js:11
	      @noop [i];
	      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-class-not-available-during-decorators-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-class-not-available-during-decorators-exec.test.js:10
	@dec(() => {
	^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-class-underscore-property-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-class-underscore-property-exec.test.js:10
	    @noopFactory() static _ = "B";
	    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-ctx-access-public-other-this-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-ctx-access-public-other-this-exec.test.js:12
	  @dec static accessor a;
	  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-decorator-evaluation-arguments-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-decorator-evaluation-arguments-exec.test.js:13
	      @decFactory(arguments) #p;
	      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-decorator-evaluation-new-target-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-decorator-evaluation-new-target-exec.test.js:13
	      @decFactory(new.target) #p;
	      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-decorator-evaluation-private-environment-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-decorator-evaluation-private-environment-exec.test.js:20
	        @(capture((_) => _.#X, (_) => _.#x), dummy) method() {}
	        ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-decorator-evaluation-super-getter-setter-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-decorator-evaluation-super-getter-setter-exec.test.js:21
	        @(super.setter = super.getter, noop) p;
	        ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-decorator-evaluation-super-property-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-decorator-evaluation-super-property-exec.test.js:11
	        @(receivedName = super.name, noop) p;
	        ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-decorator-evaluation-this-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-decorator-evaluation-this-exec.test.js:14
	        @decFactory(this.name) #p;
	        ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-decorator-evaluation-yield-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-decorator-evaluation-yield-exec.test.js:9
	      @(yield 1, noop) method() {}
	      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-decorator-evaluation-yield-private-super-property-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-decorator-evaluation-yield-private-super-property-exec.test.js:18
	        @(yield* super.id((_) => _.#X), dummy) method() {}
	        ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-initField-existing-derived-constructor-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-initField-existing-derived-constructor-exec.test.js:22
	    @deco accessor foo = 42;
	    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-initProto-existing-derived-constructor-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-initProto-existing-derived-constructor-exec.test.js:22
	    @deco method() {}
	    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-initializer-property-ignored-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-initializer-property-ignored-exec.test.js:20
	  @decorator accessor x;
	  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-invalid-non-callable-decorators-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-invalid-non-callable-decorators-exec.test.js:12
	  (0,__vite_ssr_import_0__.expect)(() => @nonCallable class {}).toThrow("A decorator must be a function");
	                                         ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-outer-class-binding-mutated-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-outer-class-binding-mutated-exec.test.js:25
	    @capture(() => K) @assertUninitialized(() => K) [(capture(() => K), assertUninitialized(() => K))];
	    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-private-name-in-class-decorator-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-private-name-in-class-decorator-exec.test.js:11
	    @(A.#x) class B extends A {
	    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-this-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-this-exec.test.js:18
	@o1.dec @dec @o2.dec @(o4.o().dec) class A {
	^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-accessor-initializers-fields-exec.test.js
RolldownError: Parse failure: Parse failed with 1 error:
Unexpected token
44:       _defineProperty(this, "b", void 0);
45:       _classPrivateFieldInitSpec(this, _c, void 0);
46:       this.a = _classPrivateFieldSet(, this, null);
                                         ^
47:     }
48:   }
At file: /fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-accessor-initializers-fields-exec.test.js:46:36
    at error (./node_modules/rolldown/dist/shared/logs-D6uA606S.mjs:166:24)
    at normalizeParseError (./node_modules/rolldown/dist/parse-ast-index.mjs:25:9)
    at wrap (./node_modules/rolldown/dist/parse-ast-index.mjs:5:39)
    at parseAstAsync (./node_modules/rolldown/dist/parse-ast-index.mjs:54:9)
    at ssrTransformScript (./node_modules/vite/dist/node/chunks/node.js:16392:9)
    at loadAndTransform (./node_modules/vite/dist/node/chunks/node.js:20032:64)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-accessor-initializers-setters-exec.test.js
RolldownError: Parse failure: Parse failed with 1 error:
Unexpected token
48:     constructor() {
49:       this.a = null;
50:       _classPrivateFieldSet(, this, null);
                                ^
51:     }
52:   }
At file: /fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-accessor-initializers-setters-exec.test.js:50:27
    at error (./node_modules/rolldown/dist/shared/logs-D6uA606S.mjs:166:24)
    at normalizeParseError (./node_modules/rolldown/dist/parse-ast-index.mjs:25:9)
    at wrap (./node_modules/rolldown/dist/parse-ast-index.mjs:5:39)
    at parseAstAsync (./node_modules/rolldown/dist/parse-ast-index.mjs:54:9)
    at ssrTransformScript (./node_modules/vite/dist/node/chunks/node.js:16392:9)
    at loadAndTransform (./node_modules/vite/dist/node/chunks/node.js:20032:64)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-accessor-method-initializers-exec.test.js
RolldownError: Parse failure: Parse failed with 1 error:
Unexpected token
49:       this.b();
50:       _assertClassBrand(_A_brand, this, _c).call(this);
51:       this.a = _classPrivateFieldSet(, this, null);
                                         ^
52:     }
53:   }
At file: /fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-accessor-method-initializers-exec.test.js:51:36
    at error (./node_modules/rolldown/dist/shared/logs-D6uA606S.mjs:166:24)
    at normalizeParseError (./node_modules/rolldown/dist/parse-ast-index.mjs:25:9)
    at wrap (./node_modules/rolldown/dist/parse-ast-index.mjs:5:39)
    at parseAstAsync (./node_modules/rolldown/dist/parse-ast-index.mjs:54:9)
    at ssrTransformScript (./node_modules/vite/dist/node/chunks/node.js:16392:9)
    at loadAndTransform (./node_modules/vite/dist/node/chunks/node.js:20032:64)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-accessor-static-accessor-initializers-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-accessor-static-accessor-initializers-exec.test.js:30
	@logClassDecoratorRun(0, 19, 29) @logClassDecoratorRun(1, 18, 28) class A {
	^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-accessor-static-method-initializers-exec.test.js
RolldownError: Parse failure: Parse failed with 1 error:
Unexpected token
44:     @logAccessorDecoratorRun(8, 17, 37, 34) @logAccessorDecoratorRun(9, 16, 36, 35) accessor #d;
45:     constructor() {
46:       this.a = _classPrivateFieldSet(, this, null);
                                         ^
47:     }
48:   }
At file: /fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-accessor-static-method-initializers-exec.test.js:46:36
    at error (./node_modules/rolldown/dist/shared/logs-D6uA606S.mjs:166:24)
    at normalizeParseError (./node_modules/rolldown/dist/parse-ast-index.mjs:25:9)
    at wrap (./node_modules/rolldown/dist/parse-ast-index.mjs:5:39)
    at parseAstAsync (./node_modules/rolldown/dist/parse-ast-index.mjs:54:9)
    at ssrTransformScript (./node_modules/vite/dist/node/chunks/node.js:16392:9)
    at loadAndTransform (./node_modules/vite/dist/node/chunks/node.js:20032:64)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-class-decorators-without-element-decorators-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-class-decorators-without-element-decorators-exec.test.js:22
	  @classDec1(log) @classDec2(log) class C {
	  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-decorators-evaluation-with-this-caching-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-decorators-evaluation-with-this-caching-exec.test.js:18
	  @fn() @obj.prop.foo method() {}
	  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-field-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-field-exec.test.js:23
	@logDecoratorRun(0, 21) @logDecoratorRun(1, 20) class A {
	^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-field-initializers-after-methods-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-field-initializers-after-methods-exec.test.js:7
	@(((x) => x)) class A {
	^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-initializers-and-static-blocks-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-initializers-and-static-blocks-exec.test.js:142
	@classDec1 @classDec2 class Foo extends (log.push("extends"), Object) {
	^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-initializers-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-initializers-exec.test.js:22
	@logDecoratorRun(0, 35, 45) @logDecoratorRun(1, 34, 44) class A {
	^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-method-initializers-field-value-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-method-initializers-field-value-exec.test.js:46
	    @methodDec1 @methodDec2 method() {}
	    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-static-accessor-method-initializers-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-static-accessor-method-initializers-exec.test.js:43
	@logClassDecoratorRun(0, 19, 29) @logClassDecoratorRun(1, 18, 28) class A {
	^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-static-field-initializers-after-methods-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-static-field-initializers-after-methods-exec.test.js:8
	@(((x) => x)) class A {
	^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-accessor-init-and-set-consistent-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-accessor-init-and-set-consistent-exec.test.js:26
	  @minusTwo @timesFour accessor bar = 5;
	  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-accessor-initializers-fields-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-accessor-initializers-fields-exec.test.js:37
	@logClassDecoratorRun(0, 19, 21) @logClassDecoratorRun(1, 18, 20) class A {
	^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-accessor-initializers-setters-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-accessor-initializers-setters-exec.test.js:45
	@logClassDecoratorRun(0, 11, 13) @logClassDecoratorRun(1, 10, 12) class A {
	^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-accessor-method-initializers-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-accessor-method-initializers-exec.test.js:40
	@logClassDecoratorRun(0, 19, 21) @logClassDecoratorRun(1, 18, 20) class A {
	^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-accessor-static-accessor-initializers-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-accessor-static-accessor-initializers-exec.test.js:30
	@logClassDecoratorRun(0, 19, 29) @logClassDecoratorRun(1, 18, 28) class A {
	^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-accessor-static-method-initializers-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-accessor-static-method-initializers-exec.test.js:40
	@logClassDecoratorRun(0, 19, 29) @logClassDecoratorRun(1, 18, 28) class A {
	^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-class-decorators-without-element-decorators-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-class-decorators-without-element-decorators-exec.test.js:17
	  @classDec1(log) @classDec2(log) class C {
	  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-field-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-field-exec.test.js:17
	@logDecoratorRun(0, 21) @logDecoratorRun(1, 20) class A {
	^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-field-initializers-after-methods-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-field-initializers-after-methods-exec.test.js:6
	@(((x) => x)) class A {
	^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-field-initializers-after-methods-private-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-field-initializers-after-methods-private-exec.test.js:7
	  @(((_, { addInitializer }) => {
	  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-initializers-and-static-blocks-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-initializers-and-static-blocks-exec.test.js:141
	@classDec1 @classDec2 class Foo extends (log.push("extends"), Object) {
	^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-initializers-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-initializers-exec.test.js:20
	@logDecoratorRun(0, 35, 45) @logDecoratorRun(1, 34, 44) class A {
	^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-method-initializers-field-value-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-method-initializers-field-value-exec.test.js:47
	    @methodDec1 @methodDec2 method() {}
	    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-static-accessor-method-initializers-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-static-accessor-method-initializers-exec.test.js:40
	@logClassDecoratorRun(0, 19, 29) @logClassDecoratorRun(1, 18, 28) class A {
	^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-static-field-initializers-after-methods-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-static-field-initializers-after-methods-exec.test.js:6
	@(((x) => x)) class A {
	^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-runtime-errors--to-es2015-invalid-accessor-decorator-return-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-runtime-errors--to-es2015-invalid-accessor-decorator-return-exec.test.js:18
	  @returnsNull accessor a;
	  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-runtime-errors--to-es2015-invalid-add-initializer-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-runtime-errors--to-es2015-invalid-add-initializer-exec.test.js:9
	  @decWithInitializer(null) static m() {}
	  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-runtime-errors--to-es2015-invalid-class-decorator-return-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-runtime-errors--to-es2015-invalid-class-decorator-return-exec.test.js:9
	(0,__vite_ssr_import_0__.expect)(() => @returnsNull class {}).toThrow("class decorators must return a function or undefined");
	                                       ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-runtime-errors--to-es2015-invalid-field-decorator-return-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-runtime-errors--to-es2015-invalid-field-decorator-return-exec.test.js:10
	  @returnsNull m() {}
	  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-runtime-errors--to-es2015-invalid-getter-decorator-return-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-runtime-errors--to-es2015-invalid-getter-decorator-return-exec.test.js:10
	  @returnsNull get p() {}
	  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-runtime-errors--to-es2015-invalid-method-decorator-return-exec.test.js
AssertionError: expected [Function] to throw an error
    at Proxy.methodWrapper (./node_modules/chai/index.js:1700:25)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-runtime-errors--to-es2015-invalid-method-decorator-return-exec.test.js:13:5

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-runtime-errors--to-es2015-invalid-setter-decorator-return-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-runtime-errors--to-es2015-invalid-setter-decorator-return-exec.test.js:10
	  @returnsNull set p(v) {}
	  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-setters--to-es2015-context-name-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-setters--to-es2015-context-name-exec.test.js:16
	  @dec static set a(v) {}
	  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-setters--to-es2015-private-exec.test.js
TypeError: Cannot read properties of undefined (reading 'access')
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-setters--to-es2015-private-exec.test.js:32:18

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-setters--to-es2015-public-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-setters--to-es2015-public-exec.test.js:19
	  @dec set a(v) {
	  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-setters--to-es2015-static-private-exec.test.js
TypeError: Cannot read properties of undefined (reading 'access')
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-setters--to-es2015-static-private-exec.test.js:26:18

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-setters--to-es2015-static-public-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-setters--to-es2015-static-public-exec.test.js:16
	  @dec static set a(v) {
	  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-setters-context-name-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-setters-context-name-exec.test.js:14
	  @dec static set a(v) {}
	  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-typescript-computed-key-ts-as-expression-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-typescript-computed-key-ts-as-expression-exec.test.js:13
	    @noopFactory(0) #p;
	    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-metadata-class-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-metadata-class-exec.test.js:9
	@dec class A {}
	^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-metadata-element-exec.test.js
AssertionError: expected undefined to deeply equal { foo: 3 }
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-metadata-element-exec.test.js:14:29

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-metadata-subclass-no-super-decorators-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-metadata-subclass-no-super-decorators-exec.test.js:12
	@dec(3) class A extends B {}
	^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-metadata-subclass-super-decorators-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-metadata-subclass-super-decorators-exec.test.js:11
	@dec(2) class B {}
	^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-transform-arrow-functions-test-fixtures-arrow-functions-implicit-var-arguments-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-arrow-functions-test-fixtures-arrow-functions-implicit-var-arguments-exec.test.js:5
	var arguments = [
	    ^^^^^^^^^

SyntaxError: Unexpected eval or arguments in strict mode
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-transform-async-generator-functions-test-fixtures-async-generators-await-in-finally-exec.test.js
AssertionError: expected { value: 2, done: true } to deeply equal { value: 4, done: false }
    at test (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-async-generator-functions-test-fixtures-async-generators-await-in-finally-exec.test.js:28:44)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)

./fixtures/babel/babel-plugin-transform-async-to-generator-test-fixtures-async-to-generator-async-complex-params-exec.test.js
TypeError: Cannot destructure property 'b' of 'undefined' as it is undefined.
    at Object.foo (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-async-to-generator-test-fixtures-async-to-generator-async-complex-params-exec.test.js:7:13)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-async-to-generator-test-fixtures-async-to-generator-async-complex-params-exec.test.js:10:6

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-assumption-setPublicClassFields-static-infer-name-exec.test.js
AssertionError: expected '_Class' to be 'Foo' // Object.is equality
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-assumption-setPublicClassFields-static-infer-name-exec.test.js:8:19

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-nested-class-super-call-in-decorator-exec.test.js
AssertionError: expected undefined to be 'hello' // Object.is equality
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-nested-class-super-call-in-decorator-exec.test.js:21:28

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-nested-class-super-property-in-accessor-key-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-nested-class-super-property-in-accessor-key-exec.test.js:15
	      accessor [super.toString()] = "hello";
	               ^

SyntaxError: Unexpected token '['
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-nested-class-super-property-in-decorator-exec.test.js
AssertionError: expected undefined to be 'hello' // Object.is equality
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-nested-class-super-property-in-decorator-exec.test.js:22:28

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-before-member-call-with-transform-exec.test.js
AssertionError: expected undefined to deeply equal 1
    at Foo.test (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-before-member-call-with-transform-exec.test.js:42:263)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-before-member-call-with-transform-exec.test.js:128:6

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-before-property-with-transform-exec.test.js
AssertionError: expected undefined to deeply equal 1
    at Foo.test (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-before-property-with-transform-exec.test.js:41:261)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-before-property-with-transform-exec.test.js:121:6

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-delete-property-with-transform-exec.test.js
AssertionError: expected function to throw an error, but it didn't
    at Foo.test (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-delete-property-with-transform-exec.test.js:69:9)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-delete-property-with-transform-exec.test.js:169:6

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-in-function-param-with-transform-exec.test.js
TypeError: Cannot convert undefined or null to object
    at hasOwnProperty (<anonymous>)
    at _classPrivateFieldBase (/Users/qing/p/github/oxc/npm/runtime/src/helpers/classPrivateFieldLooseBase.js:2:26)
    at value (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-in-function-param-with-transform-exec.test.js:63:13)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-in-function-param-with-transform-exec.test.js:44:164
    at j (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-in-function-param-with-transform-exec.test.js:45:8)
    at Foo.test (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-in-function-param-with-transform-exec.test.js:52:13)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-in-function-param-with-transform-exec.test.js:71:6

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-member-optional-call-with-transform-exec.test.js
TypeError: Cannot convert undefined or null to object
    at hasOwnProperty (<anonymous>)
    at _classPrivateFieldBase (/Users/qing/p/github/oxc/npm/runtime/src/helpers/classPrivateFieldLooseBase.js:2:26)
    at value (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-member-optional-call-with-transform-exec.test.js:123:13)
    at Foo.test (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-member-optional-call-with-transform-exec.test.js:24:99)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-member-optional-call-with-transform-exec.test.js:131:6

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-optional-member-call-with-transform-exec.test.js
AssertionError: expected undefined to deeply equal 1
    at Foo.test (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-optional-member-call-with-transform-exec.test.js:41:254)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-optional-member-call-with-transform-exec.test.js:131:6

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-optional-property-with-transform-exec.test.js
AssertionError: expected undefined to deeply equal 1
    at Foo.test (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-optional-property-with-transform-exec.test.js:40:252)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-optional-property-with-transform-exec.test.js:124:6

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-parenthesized-optional-member-call-exec.test.js
TypeError: Cannot read properties of undefined (reading 'bind')
    at Foo.test (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-parenthesized-optional-member-call-exec.test.js:20:61)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-parenthesized-optional-member-call-exec.test.js:78:12

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-parenthesized-optional-member-call-with-transform-exec.test.js
TypeError: Cannot read properties of undefined (reading 'bind')
    at Foo.test (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-parenthesized-optional-member-call-with-transform-exec.test.js:20:42)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-parenthesized-optional-member-call-with-transform-exec.test.js:78:12

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-delete-property-with-transform-exec.test.js
AssertionError: expected function to throw an error, but it didn't
    at Foo.test (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-delete-property-with-transform-exec.test.js:69:9)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-delete-property-with-transform-exec.test.js:169:6

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-static-shadow-exec.test.js
TypeError: e.has is not a function
    at _assertClassBrand (/Users/qing/p/github/oxc/npm/runtime/src/helpers/assertClassBrand.js:2:44)
    at func (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-static-shadow-exec.test.js:10:15)
    at Test.method (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-static-shadow-exec.test.js:12:13)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-static-shadow-exec.test.js:16:14

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-public-computed-toPrimitive-exec.test.js
AssertionError: expected [Function] to throw error including '@@toPrimitive must return a primitive…' but got 'Cannot convert object to primitive va…'
    at Proxy.methodWrapper (./node_modules/chai/index.js:1700:25)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-public-computed-toPrimitive-exec.test.js:37:5

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-public-delete-super-property-exec.test.js
AssertionError: expected function to throw an error, but it didn't
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-public-delete-super-property-exec.test.js:25:5

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-public-loose-static-infer-name-exec.test.js
AssertionError: expected '_Class' to be 'Foo' // Object.is equality
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-public-loose-static-infer-name-exec.test.js:8:19

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-public-static-infer-name-exec.test.js
AssertionError: expected '_Class' to be 'Foo' // Object.is equality
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-public-static-infer-name-exec.test.js:9:19

./fixtures/babel/babel-plugin-transform-class-static-block-test-fixtures-integration-loose-private-methods-access-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (/Users/qing/p/github/oxc/npm/runtime/src/helpers/classPrivateFieldLooseBase.js:2:44)
    at getFoo (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-static-block-test-fixtures-integration-loose-private-methods-access-exec.test.js:12:17)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-static-block-test-fixtures-integration-loose-private-methods-access-exec.test.js:13:9

./fixtures/babel/babel-plugin-transform-explicit-resource-management-test-fixtures-transform-top-level-hoisting-mutate-outer-class-binding-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-explicit-resource-management-test-fixtures-transform-top-level-hoisting-mutate-outer-class-binding-exec.test.js:6
	export { A };
	^^^^^^

SyntaxError: Unexpected token 'export'
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1272:80)
    at processTicksAndRejections (node:internal/process/task_queues:104:5)
    at VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1179:73)

./fixtures/babel/babel-plugin-transform-object-rest-spread-test-fixtures-object-rest-for-x-assignment-shadowed-block-scoped-bindings-exec.test.js
ReferenceError: Cannot access 'a' before initialization
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-object-rest-spread-test-fixtures-object-rest-for-x-assignment-shadowed-block-scoped-bindings-exec.test.js:8:20

./fixtures/babel/babel-plugin-transform-object-rest-spread-test-fixtures-object-rest-for-x-declaration-shadowed-block-scoped-bindings-exec.test.js
ReferenceError: Cannot access 'a' before initialization
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-object-rest-spread-test-fixtures-object-rest-for-x-declaration-shadowed-block-scoped-bindings-exec.test.js:8:13

./fixtures/babel/babel-plugin-transform-optional-chaining-test-fixtures-assumption-noDocumentAll-parenthesized-expression-member-call-exec.test.js
TypeError: Cannot read properties of undefined (reading 'x')
    at m (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-optional-chaining-test-fixtures-assumption-noDocumentAll-parenthesized-expression-member-call-exec.test.js:10:18)
    at Foo.test (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-optional-chaining-test-fixtures-assumption-noDocumentAll-parenthesized-expression-member-call-exec.test.js:25:65)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-optional-chaining-test-fixtures-assumption-noDocumentAll-parenthesized-expression-member-call-exec.test.js:68:12

./fixtures/babel/babel-plugin-transform-optional-chaining-test-fixtures-general-parenthesized-expression-member-call-exec.test.js
TypeError: Cannot read properties of undefined (reading 'x')
    at m (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-optional-chaining-test-fixtures-general-parenthesized-expression-member-call-exec.test.js:10:18)
    at Foo.test (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-optional-chaining-test-fixtures-general-parenthesized-expression-member-call-exec.test.js:25:65)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-optional-chaining-test-fixtures-general-parenthesized-expression-member-call-exec.test.js:68:12

./fixtures/babel/babel-plugin-transform-optional-chaining-test-fixtures-general-parenthesized-expression-member-call-loose-exec.test.js
TypeError: Cannot read properties of undefined (reading 'x')
    at m (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-optional-chaining-test-fixtures-general-parenthesized-expression-member-call-loose-exec.test.js:10:18)
    at Foo.test (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-optional-chaining-test-fixtures-general-parenthesized-expression-member-call-loose-exec.test.js:25:65)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-optional-chaining-test-fixtures-general-parenthesized-expression-member-call-loose-exec.test.js:68:12

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-basic-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-basic-exec.test.js:10:40)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-basic-exec.test.js:30:13

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-class-binding-exec.test.js
ReferenceError: _A_brand is not defined
    at new A (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-class-binding-exec.test.js:9:40)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-class-binding-exec.test.js:21:9

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-get-only-setter-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-get-only-setter-exec.test.js:11:40)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-get-only-setter-exec.test.js:22:13

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-helper-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-helper-exec.test.js:9:40)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-helper-exec.test.js:18:2

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-reassignment-exec.test.js
AssertionError: expected +0 to be 1 // Object.is equality
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-reassignment-exec.test.js:17:18

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-set-only-getter-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-set-only-getter-exec.test.js:11:40)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-set-only-getter-exec.test.js:23:13

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-updates-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-updates-exec.test.js:10:40)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-updates-exec.test.js:49:13

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-basic-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-basic-exec.test.js:10:40)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-basic-exec.test.js:30:13

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-class-binding-exec.test.js
ReferenceError: _A_brand is not defined
    at new A (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-class-binding-exec.test.js:9:40)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-class-binding-exec.test.js:21:9

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-get-only-setter-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-get-only-setter-exec.test.js:11:40)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-get-only-setter-exec.test.js:22:13

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-helper-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-helper-exec.test.js:9:40)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-helper-exec.test.js:18:2

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-set-only-getter-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-set-only-getter-exec.test.js:11:40)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-set-only-getter-exec.test.js:23:13

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-updates-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-updates-exec.test.js:10:40)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-updates-exec.test.js:49:13

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsSymbols-get-only-setter-exec.test.js
TypeError: "#privateFieldValue" is write-only
    at _writeOnlyError (/Users/qing/p/github/oxc/npm/runtime/src/helpers/writeOnlyError.js:2:9)
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsSymbols-get-only-setter-exec.test.js:14:20)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsSymbols-get-only-setter-exec.test.js:20:13

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-assignment-exec.test.js
ReferenceError: _Foo_brand is not defined
    at new Foo (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-assignment-exec.test.js:8:40)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-assignment-exec.test.js:15:9

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-async-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-async-exec.test.js:8:40)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-async-exec.test.js:17:9

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-before-fields-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-before-fields-exec.test.js:10:40)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-before-fields-exec.test.js:24:9

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-class-binding-exec.test.js
ReferenceError: _A_brand is not defined
    at new A (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-class-binding-exec.test.js:9:40)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-class-binding-exec.test.js:21:9

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-context-exec.test.js
ReferenceError: _Foo_brand is not defined
    at new Foo (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-context-exec.test.js:8:40)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-context-exec.test.js:34:12

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-exfiltrated-exec.test.js
ReferenceError: _Foo_brand is not defined
    at new Foo (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-exfiltrated-exec.test.js:9:40)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-exfiltrated-exec.test.js:17:2

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-generator-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-generator-exec.test.js:8:40)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-generator-exec.test.js:18:14

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-reassignment-exec.test.js
AssertionError: expected +0 to be 1 // Object.is equality
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-reassignment-exec.test.js:17:18

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-super-exec.test.js
ReferenceError: _Sub_brand is not defined
    at new Sub (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-super-exec.test.js:16:40)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-super-exec.test.js:29:9

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-assignment-exec.test.js
ReferenceError: _Foo_brand is not defined
    at new Foo (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-assignment-exec.test.js:8:40)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-assignment-exec.test.js:15:9

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-async-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-async-exec.test.js:8:40)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-async-exec.test.js:17:9

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-before-fields-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-before-fields-exec.test.js:11:40)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-before-fields-exec.test.js:25:9

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-class-binding-exec.test.js
ReferenceError: _A_brand is not defined
    at new A (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-class-binding-exec.test.js:9:40)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-class-binding-exec.test.js:21:9

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-context-exec.test.js
ReferenceError: _Foo_brand is not defined
    at new Foo (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-context-exec.test.js:8:40)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-context-exec.test.js:34:12

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-exfiltrated-exec.test.js
ReferenceError: _Foo_brand is not defined
    at new Foo (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-exfiltrated-exec.test.js:9:40)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-exfiltrated-exec.test.js:17:2

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-generator-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-generator-exec.test.js:8:40)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-generator-exec.test.js:18:14

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-super-exec.test.js
ReferenceError: _Sub_brand is not defined
    at new Sub (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-super-exec.test.js:16:40)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-super-exec.test.js:29:9

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsSymbols-async-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsSymbols-async-exec.test.js:8:40)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsSymbols-async-exec.test.js:17:9

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-basic-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (/Users/qing/p/github/oxc/npm/runtime/src/helpers/classPrivateFieldLooseBase.js:2:44)
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-basic-exec.test.js:20:27)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-basic-exec.test.js:26:9

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-class-check-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (/Users/qing/p/github/oxc/npm/runtime/src/helpers/classPrivateFieldLooseBase.js:2:44)
    at Cl.publicMethod (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-class-check-exec.test.js:7:13)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-class-check-exec.test.js:14:12

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-exfiltrated-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (/Users/qing/p/github/oxc/npm/runtime/src/helpers/classPrivateFieldLooseBase.js:2:44)
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-exfiltrated-exec.test.js:9:22)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-exfiltrated-exec.test.js:17:2

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-generator-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (/Users/qing/p/github/oxc/npm/runtime/src/helpers/classPrivateFieldLooseBase.js:2:44)
    at Cl.test (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-generator-exec.test.js:7:13)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-generator-exec.test.js:14:23

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-scopable-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (/Users/qing/p/github/oxc/npm/runtime/src/helpers/classPrivateFieldLooseBase.js:2:44)
    at Cl.publicMethod (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-scopable-exec.test.js:7:13)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-scopable-exec.test.js:18:18

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-super-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (/Users/qing/p/github/oxc/npm/runtime/src/helpers/classPrivateFieldLooseBase.js:2:44)
    at Sub.check (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-super-exec.test.js:17:13)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-super-exec.test.js:24:13

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-this-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (/Users/qing/p/github/oxc/npm/runtime/src/helpers/classPrivateFieldLooseBase.js:2:44)
    at B.extract (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-this-exec.test.js:17:14)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-this-exec.test.js:27:25

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-basic-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (/Users/qing/p/github/oxc/npm/runtime/src/helpers/classPrivateFieldLooseBase.js:2:44)
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-basic-exec.test.js:20:27)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-basic-exec.test.js:26:9

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-class-check-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (/Users/qing/p/github/oxc/npm/runtime/src/helpers/classPrivateFieldLooseBase.js:2:44)
    at Cl.publicMethod (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-class-check-exec.test.js:7:13)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-class-check-exec.test.js:14:12

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-exfiltrated-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (/Users/qing/p/github/oxc/npm/runtime/src/helpers/classPrivateFieldLooseBase.js:2:44)
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-exfiltrated-exec.test.js:9:22)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-exfiltrated-exec.test.js:17:2

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-generator-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (/Users/qing/p/github/oxc/npm/runtime/src/helpers/classPrivateFieldLooseBase.js:2:44)
    at Cl.test (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-generator-exec.test.js:7:13)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-generator-exec.test.js:14:23

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-scopable-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (/Users/qing/p/github/oxc/npm/runtime/src/helpers/classPrivateFieldLooseBase.js:2:44)
    at Cl.publicMethod (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-scopable-exec.test.js:7:13)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-scopable-exec.test.js:18:18

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-super-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (/Users/qing/p/github/oxc/npm/runtime/src/helpers/classPrivateFieldLooseBase.js:2:44)
    at Sub.check (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-super-exec.test.js:17:13)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-super-exec.test.js:24:13

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-this-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (/Users/qing/p/github/oxc/npm/runtime/src/helpers/classPrivateFieldLooseBase.js:2:44)
    at B.extract (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-this-exec.test.js:17:14)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-this-exec.test.js:27:25

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-get-only-setter-exec.test.js
TypeError: "#privateStaticFieldValue" is write-only
    at _writeOnlyError (/Users/qing/p/github/oxc/npm/runtime/src/helpers/writeOnlyError.js:2:9)
    at Cl.getPrivateStaticFieldValue (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-get-only-setter-exec.test.js:7:17)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-get-only-setter-exec.test.js:14:12

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-loose-access-in-static-field-initializer-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (/Users/qing/p/github/oxc/npm/runtime/src/helpers/classPrivateFieldLooseBase.js:2:44)
    at new C (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-loose-access-in-static-field-initializer-exec.test.js:12:12)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-loose-access-in-static-field-initializer-exec.test.js:21:13
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-loose-access-in-static-field-initializer-exec.test.js:24:4

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-loose-basic-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (/Users/qing/p/github/oxc/npm/runtime/src/helpers/classPrivateFieldLooseBase.js:2:44)
    at Cl.getValue (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-loose-basic-exec.test.js:10:13)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-loose-basic-exec.test.js:27:12

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-loose-destructure-set-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (/Users/qing/p/github/oxc/npm/runtime/src/helpers/classPrivateFieldLooseBase.js:2:44)
    at new C (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-loose-destructure-set-exec.test.js:10:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-loose-destructure-set-exec.test.js:22:2

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-loose-get-only-setter-exec.test.js
TypeError: "#privateStaticFieldValue" is write-only
    at _writeOnlyError (/Users/qing/p/github/oxc/npm/runtime/src/helpers/writeOnlyError.js:2:9)
    at Cl.getPrivateStaticFieldValue (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-loose-get-only-setter-exec.test.js:11:17)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-loose-get-only-setter-exec.test.js:22:12

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-loose-updates-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (/Users/qing/p/github/oxc/npm/runtime/src/helpers/classPrivateFieldLooseBase.js:2:44)
    at Cl.testUpdates (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-loose-updates-exec.test.js:24:6)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-loose-updates-exec.test.js:47:5

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-access-in-static-field-initializer-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (/Users/qing/p/github/oxc/npm/runtime/src/helpers/classPrivateFieldLooseBase.js:2:44)
    at new C (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-access-in-static-field-initializer-exec.test.js:12:12)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-access-in-static-field-initializer-exec.test.js:21:13
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-access-in-static-field-initializer-exec.test.js:24:4

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-basic-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (/Users/qing/p/github/oxc/npm/runtime/src/helpers/classPrivateFieldLooseBase.js:2:44)
    at Cl.getValue (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-basic-exec.test.js:10:13)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-basic-exec.test.js:27:12

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-destructure-set-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (/Users/qing/p/github/oxc/npm/runtime/src/helpers/classPrivateFieldLooseBase.js:2:44)
    at new C (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-destructure-set-exec.test.js:10:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-destructure-set-exec.test.js:22:2

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-get-only-setter-exec.test.js
TypeError: "#privateStaticFieldValue" is write-only
    at _writeOnlyError (/Users/qing/p/github/oxc/npm/runtime/src/helpers/writeOnlyError.js:2:9)
    at Cl.getPrivateStaticFieldValue (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-get-only-setter-exec.test.js:11:17)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-get-only-setter-exec.test.js:22:12

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-updates-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (/Users/qing/p/github/oxc/npm/runtime/src/helpers/classPrivateFieldLooseBase.js:2:44)
    at Cl.testUpdates (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-updates-exec.test.js:25:6)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-updates-exec.test.js:48:5

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsSymbols-get-only-setter-exec.test.js
TypeError: "#privateStaticFieldValue" is write-only
    at _writeOnlyError (/Users/qing/p/github/oxc/npm/runtime/src/helpers/writeOnlyError.js:2:9)
    at Cl.getPrivateStaticFieldValue (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsSymbols-get-only-setter-exec.test.js:7:17)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsSymbols-get-only-setter-exec.test.js:14:12

./fixtures/babel/babel-plugin-transform-private-property-in-object-test-fixtures-assumption-privateFieldsAsProperties-method-exec.test.js
ReferenceError: _Foo_brand is not defined
    at new Foo (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-property-in-object-test-fixtures-assumption-privateFieldsAsProperties-method-exec.test.js:8:40)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-property-in-object-test-fixtures-assumption-privateFieldsAsProperties-method-exec.test.js:19:13

./fixtures/babel/babel-plugin-transform-private-property-in-object-test-fixtures-private-half-constructed-static-exec.test.js
AssertionError: expected true to be false // Object.is equality
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-property-in-object-test-fixtures-private-half-constructed-static-exec.test.js:29:15

./fixtures/babel/babel-plugin-transform-private-property-in-object-test-fixtures-private-loose-static-shadow-exec.test.js
AssertionError: expected 2 to be 5 // Object.is equality
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-property-in-object-test-fixtures-private-loose-static-shadow-exec.test.js:18:25

./fixtures/babel/babel-plugin-transform-private-property-in-object-test-fixtures-private-static-field-initialization-exec.test.js
AssertionError: expected [ true, true, true, true, true, …(3) ] to deeply equal [ false, true, true, false, …(4) ]
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-property-in-object-test-fixtures-private-static-field-initialization-exec.test.js:23:17

./fixtures/babel/babel-plugin-transform-private-property-in-object-test-fixtures-private-static-field-redeclared-in-loop-exec.test.js
AssertionError: expected [ true, true, true, true ] to deeply equal [ false, true, true, true ]
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-property-in-object-test-fixtures-private-static-field-redeclared-in-loop-exec.test.js:15:18

./fixtures/babel/babel-plugin-transform-private-property-in-object-test-fixtures-private-static-shadow-exec.test.js
AssertionError: expected 2 to be 5 // Object.is equality
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-property-in-object-test-fixtures-private-static-shadow-exec.test.js:18:25

./fixtures/babel/babel-plugin-transform-private-property-in-object-test-fixtures-to-native-fields-half-constructed-static-exec.test.js
AssertionError: expected true to be false // Object.is equality
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-property-in-object-test-fixtures-to-native-fields-half-constructed-static-exec.test.js:29:15

./fixtures/babel/babel-plugin-transform-private-property-in-object-test-fixtures-to-native-fields-static-shadow-exec.test.js
AssertionError: expected 2 to be 5 // Object.is equality
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-property-in-object-test-fixtures-to-native-fields-static-shadow-exec.test.js:18:25

./fixtures/babel/babel-preset-env-test-fixtures-plugins-integration-issue-15170-exec.test.js
AssertionError: expected [Function] to not throw an error but 'ReferenceError: x is not defined' was thrown
    at Proxy.methodWrapper (./node_modules/chai/index.js:1700:25)
    at ./tasks/transform_conformance/fixtures/babel/babel-preset-env-test-fixtures-plugins-integration-issue-15170-exec.test.js:6:9

./fixtures/babel/babel-preset-env-test-fixtures-sanity-check-es2015-constants-exec.test.js
TypeError: Assignment to constant variable.
    at ./tasks/transform_conformance/fixtures/babel/babel-preset-env-test-fixtures-sanity-check-es2015-constants-exec.test.js:5:6

./fixtures/babel/babel-preset-env-test-fixtures-sanity-regex-dot-all-exec.test.js
AssertionError: expected false to be true // Object.is equality
    at ./tasks/transform_conformance/fixtures/babel/babel-preset-env-test-fixtures-sanity-regex-dot-all-exec.test.js:10:37
