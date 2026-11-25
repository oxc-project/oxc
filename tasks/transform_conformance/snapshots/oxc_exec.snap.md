commit: 51225612

node: v26.12.1

Passed: 86 of 382 (22.51%)

Failures:

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-auto-accessor-decorators-basic-instance-auto-accessor-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-auto-accessor-decorators-basic-instance-auto-accessor-exec.test.js:71
    @dec("foo", "get foo", "set foo") accessor foo = 0;
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-auto-accessor-decorators-basic-private-instance-auto-accessor-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-auto-accessor-decorators-basic-private-instance-auto-accessor-exec.test.js:74
    @dec accessor #foo = 0;
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-auto-accessor-decorators-basic-private-static-auto-accessor-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-auto-accessor-decorators-basic-private-static-auto-accessor-exec.test.js:73
    @dec static accessor #foo = 0;
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-auto-accessor-decorators-basic-static-auto-accessor-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-auto-accessor-decorators-basic-static-auto-accessor-exec.test.js:71
    @dec("foo", "get foo", "set foo") static accessor foo = 0;
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-auto-accessor-decorators-extra-initializer-instance-auto-accessor-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-auto-accessor-decorators-extra-initializer-instance-auto-accessor-exec.test.js:64
    @dec @dec accessor foo;
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-auto-accessor-decorators-extra-initializer-private-instance-auto-accessor-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-auto-accessor-decorators-extra-initializer-private-instance-auto-accessor-exec.test.js:64
    @dec @dec accessor #foo;
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-auto-accessor-decorators-extra-initializer-private-static-auto-accessor-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-auto-accessor-decorators-extra-initializer-private-static-auto-accessor-exec.test.js:64
    @dec @dec static accessor #foo;
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-auto-accessor-decorators-extra-initializer-static-auto-accessor-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-auto-accessor-decorators-extra-initializer-static-auto-accessor-exec.test.js:64
    @dec @dec static accessor foo;
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-auto-accessor-decorators-return-null-instance-auto-accessor-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-auto-accessor-decorators-return-null-instance-auto-accessor-exec.test.js:55
      @dec accessor foo;
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-auto-accessor-decorators-return-null-private-instance-auto-accessor-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-auto-accessor-decorators-return-null-private-instance-auto-accessor-exec.test.js:55
      @dec accessor #foo;
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-auto-accessor-decorators-return-null-private-static-auto-accessor-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-auto-accessor-decorators-return-null-private-static-auto-accessor-exec.test.js:55
      @dec static accessor #foo;
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-auto-accessor-decorators-return-null-static-auto-accessor-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-auto-accessor-decorators-return-null-static-auto-accessor-exec.test.js:55
      @dec static accessor foo;
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-auto-accessor-decorators-shim-instance-auto-accessor-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-auto-accessor-decorators-shim-instance-auto-accessor-exec.test.js:70
    @dec accessor foo = 123;
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-auto-accessor-decorators-shim-private-instance-auto-accessor-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-auto-accessor-decorators-shim-private-instance-auto-accessor-exec.test.js:72
    @dec accessor #foo = 123;
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-auto-accessor-decorators-shim-private-static-auto-accessor-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-auto-accessor-decorators-shim-private-static-auto-accessor-exec.test.js:75
      get$foo = (x) => x.#foo;
                        ^

SyntaxError: Private field '#foo' must be declared in an enclosing class
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-auto-accessor-decorators-shim-static-auto-accessor-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-auto-accessor-decorators-shim-static-auto-accessor-exec.test.js:74
    @dec static accessor foo = 123;
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-class-decorators-basic-expression-anonymous-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-class-decorators-basic-expression-anonymous-exec.test.js:61
  const Foo = ((x) => x)(@dec("") class {});
                         ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-class-decorators-basic-expression-array-binding-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-class-decorators-basic-expression-array-binding-exec.test.js:61
  const [Foo = @dec("Foo") class {}] = [];
               ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-class-decorators-basic-expression-assignment-array-binding-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-class-decorators-basic-expression-assignment-array-binding-exec.test.js:62
  [Foo = @dec("Foo") class {}] = [];
         ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-class-decorators-basic-expression-assignment-initializer-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-class-decorators-basic-expression-assignment-initializer-exec.test.js:62
  Foo = @dec("Foo") class {};
        ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-class-decorators-basic-expression-assignment-object-binding-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-class-decorators-basic-expression-assignment-object-binding-exec.test.js:62
  ({Foo = @dec("Foo") class {}} = {});
          ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-class-decorators-basic-expression-instance-auto-accessor-initializer-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-class-decorators-basic-expression-instance-auto-accessor-initializer-exec.test.js:62
    accessor Foo = @dec("Foo") class {};
             ^^^

SyntaxError: Unexpected identifier 'Foo'
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-class-decorators-basic-expression-instance-field-initializer-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-class-decorators-basic-expression-instance-field-initializer-exec.test.js:65
      (0,__vite_ssr_import_1__.default)(this, "Foo", @dec("Foo") class {});
                                                     ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-class-decorators-basic-expression-object-binding-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-class-decorators-basic-expression-object-binding-exec.test.js:61
  const { Foo = @dec("Foo") class {} } = {};
                ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-class-decorators-basic-expression-property-value-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-class-decorators-basic-expression-property-value-exec.test.js:61
  const obj = { Foo: @dec("Foo") class {} };
                     ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-class-decorators-basic-expression-static-auto-accessor-initializer-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-class-decorators-basic-expression-static-auto-accessor-initializer-exec.test.js:62
    static accessor Foo = @dec("Foo") class {};
                    ^^^

SyntaxError: Unexpected identifier 'Foo'
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-class-decorators-basic-expression-static-field-initializer-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-class-decorators-basic-expression-static-field-initializer-exec.test.js:64
  (0,__vite_ssr_import_1__.default)(Class, "Foo", @dec("Foo") class {});
                                                  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-class-decorators-basic-expression-variable-initializer-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-class-decorators-basic-expression-variable-initializer-exec.test.js:61
  const Foo = @dec("Foo") class {};
              ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-class-decorators-basic-statement-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-class-decorators-basic-statement-exec.test.js:61
  @dec("Foo") class Foo {}
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-class-decorators-extra-initializer-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-class-decorators-extra-initializer-exec.test.js:63
  @dec @dec class Foo {}
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-class-decorators-order-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-class-decorators-order-exec.test.js:70
  @dec1 @dec2 class Foo {
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-class-decorators-return-null-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-class-decorators-return-null-exec.test.js:54
    @dec class Foo {}
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-class-decorators-return-object-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-class-decorators-return-object-exec.test.js:54
    @dec class Foo {}
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-decorator-list-evaluation-await-class-expression-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-decorator-list-evaluation-await-class-expression-exec.test.js:53
    (@(log.push(await Promise.resolve(0)), dummy) class extends (log.push(await Promise.resolve(1)), Object) {
     ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-decorator-list-evaluation-await-class-statement-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-decorator-list-evaluation-await-class-statement-exec.test.js:53
    @(log.push(await Promise.resolve(0)), dummy) class Foo extends (log.push(await Promise.resolve(1)), Object) {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-decorator-list-evaluation-class-binding-class-expression-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-decorator-list-evaluation-class-binding-class-expression-exec.test.js:55
  const originalFoo = @(capture(() => Foo)) class Foo {
                      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-decorator-list-evaluation-class-binding-class-statement-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-decorator-list-evaluation-class-binding-class-statement-exec.test.js:56
  @(capture(() => Foo)) class Foo {
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-decorator-list-evaluation-computed-names-class-expression-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-decorator-list-evaluation-computed-names-class-expression-exec.test.js:77
  (@foo(0) class extends (foo(1), Object) {
   ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-decorator-list-evaluation-computed-names-class-statement-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-decorator-list-evaluation-computed-names-class-statement-exec.test.js:77
  @foo(0) class Foo extends (foo(1), Object) {
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-decorator-list-evaluation-inner-private-name-class-expression-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-decorator-list-evaluation-inner-private-name-class-expression-exec.test.js:57
  (@(capture(() => _foo._ + 0)) class Foo {
   ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-decorator-list-evaluation-inner-private-name-class-statement-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-decorator-list-evaluation-inner-private-name-class-statement-exec.test.js:62
    @(capture(() => (0,__vite_ssr_import_1__.default)(_Dummy, new Foo(), _foo)._ + 0)) class Foo {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-decorator-list-evaluation-outer-private-name-class-expression-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-decorator-list-evaluation-outer-private-name-class-expression-exec.test.js:62
    (@((0,__vite_ssr_import_1__.default)(_Dummy, dummy, _foo).call(dummy, 0)) class extends ((0,__vite_ssr_import_1__.default)(_Dummy, dummy, _foo).call(dummy, 1), Object) {
     ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-decorator-list-evaluation-outer-private-name-class-statement-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-decorator-list-evaluation-outer-private-name-class-statement-exec.test.js:62
    @((0,__vite_ssr_import_1__.default)(_Dummy, dummy, _foo).call(dummy, 0)) class Foo extends ((0,__vite_ssr_import_1__.default)(_Dummy, dummy, _foo).call(dummy, 1), Object) {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-decorator-list-evaluation-this-class-expression-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-decorator-list-evaluation-this-class-expression-exec.test.js:56
    (@(assertEq(() => this.foo(0), undefined), dummy) class extends (assertEq(() => this.foo(1), undefined), Object) {
     ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-decorator-list-evaluation-this-class-statement-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-decorator-list-evaluation-this-class-statement-exec.test.js:56
    @(assertEq(() => this.foo(0), undefined), dummy) class Foo extends (assertEq(() => this.foo(1), undefined), Object) {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-decorator-metadata-class-expression-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-decorator-metadata-class-expression-exec.test.js:54
  const Foo = @dec class {
              ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-decorator-metadata-class-statement-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-decorator-metadata-class-statement-exec.test.js:54
  @dec class Foo {
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-field-decorators-basic-instance-field-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-field-decorators-basic-instance-field-exec.test.js:68
    @dec("foo") foo = 123;
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-field-decorators-basic-private-instance-field-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-field-decorators-basic-private-instance-field-exec.test.js:70
    @dec #foo = 123;
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-field-decorators-basic-private-static-field-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-field-decorators-basic-private-static-field-exec.test.js:69
    @dec static #foo = 123;
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-field-decorators-basic-static-field-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-field-decorators-basic-static-field-exec.test.js:68
    @dec("foo") static foo = 123;
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-field-decorators-extra-initializer-instance-field-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-field-decorators-extra-initializer-instance-field-exec.test.js:64
    @dec @dec foo;
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-field-decorators-extra-initializer-private-instance-field-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-field-decorators-extra-initializer-private-instance-field-exec.test.js:64
    @dec @dec #foo;
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-field-decorators-extra-initializer-private-static-field-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-field-decorators-extra-initializer-private-static-field-exec.test.js:64
    @dec @dec static #foo;
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-field-decorators-extra-initializer-static-field-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-field-decorators-extra-initializer-static-field-exec.test.js:64
    @dec @dec static foo;
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-field-decorators-order-instance-field-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-field-decorators-order-instance-field-exec.test.js:61
    @dec1 @dec2 foo = 123;
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-field-decorators-order-private-instance-field-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-field-decorators-order-private-instance-field-exec.test.js:62
    @dec1 @dec2 #foo = 123;
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-field-decorators-order-private-static-field-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-field-decorators-order-private-static-field-exec.test.js:62
    @dec1 @dec2 static #foo = 123;
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-field-decorators-order-static-field-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-field-decorators-order-static-field-exec.test.js:61
    @dec1 @dec2 static foo = 123;
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-field-decorators-return-null-instance-field-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-field-decorators-return-null-instance-field-exec.test.js:55
      @dec foo;
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-field-decorators-return-null-private-instance-field-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-field-decorators-return-null-private-instance-field-exec.test.js:55
      @dec #foo;
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-field-decorators-return-null-private-static-field-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-field-decorators-return-null-private-static-field-exec.test.js:55
      @dec static #foo;
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-field-decorators-return-null-static-field-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-field-decorators-return-null-static-field-exec.test.js:55
      @dec static foo;
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-field-decorators-return-object-instance-field-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-field-decorators-return-object-instance-field-exec.test.js:55
      @dec foo;
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-field-decorators-return-object-private-instance-field-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-field-decorators-return-object-private-instance-field-exec.test.js:55
      @dec #foo;
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-field-decorators-return-object-private-static-field-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-field-decorators-return-object-private-static-field-exec.test.js:55
      @dec static #foo;
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-field-decorators-return-object-static-field-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-field-decorators-return-object-static-field-exec.test.js:55
      @dec static foo;
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-field-decorators-shim-instance-field-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-field-decorators-shim-instance-field-exec.test.js:58
    @dec foo = 123;
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-field-decorators-shim-private-instance-field-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-field-decorators-shim-private-instance-field-exec.test.js:62
    @dec #foo = 123;
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-field-decorators-shim-private-static-field-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-field-decorators-shim-private-static-field-exec.test.js:66
      has$foo = (x) => #foo in x;
                    ^

SyntaxError: Private field '#foo' must be declared in an enclosing class
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-field-decorators-shim-static-field-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-field-decorators-shim-static-field-exec.test.js:63
    @dec static foo = 123;
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-getter-decorators-basic-instance-getter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-getter-decorators-basic-instance-getter-exec.test.js:66
    @dec("foo", "get foo") get foo() {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-getter-decorators-basic-private-instance-getter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-getter-decorators-basic-private-instance-getter-exec.test.js:68
    @dec get #foo() {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-getter-decorators-basic-private-static-getter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-getter-decorators-basic-private-static-getter-exec.test.js:68
    @dec static get #foo() {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-getter-decorators-basic-static-getter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-getter-decorators-basic-static-getter-exec.test.js:66
    @dec("foo", "get foo") static get foo() {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-getter-decorators-extra-initializer-instance-getter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-getter-decorators-extra-initializer-instance-getter-exec.test.js:64
    @dec @dec get foo() {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-getter-decorators-extra-initializer-private-instance-getter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-getter-decorators-extra-initializer-private-instance-getter-exec.test.js:64
    @dec @dec get #foo() {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-getter-decorators-extra-initializer-private-static-getter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-getter-decorators-extra-initializer-private-static-getter-exec.test.js:64
    @dec @dec static get #foo() {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-getter-decorators-extra-initializer-static-getter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-getter-decorators-extra-initializer-static-getter-exec.test.js:64
    @dec @dec static get foo() {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-getter-decorators-order-instance-getter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-getter-decorators-order-instance-getter-exec.test.js:71
    @dec1 @dec2 get foo() {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-getter-decorators-order-private-instance-getter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-getter-decorators-order-private-instance-getter-exec.test.js:72
    @dec1 @dec2 get #foo() {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-getter-decorators-order-private-static-getter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-getter-decorators-order-private-static-getter-exec.test.js:72
    @dec1 @dec2 static get #foo() {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-getter-decorators-order-static-getter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-getter-decorators-order-static-getter-exec.test.js:71
    @dec1 @dec2 static get foo() {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-getter-decorators-return-null-instance-getter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-getter-decorators-return-null-instance-getter-exec.test.js:55
      @dec get foo() {
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-getter-decorators-return-null-private-instance-getter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-getter-decorators-return-null-private-instance-getter-exec.test.js:55
      @dec get #foo() {
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-getter-decorators-return-null-private-static-getter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-getter-decorators-return-null-private-static-getter-exec.test.js:55
      @dec static get #foo() {
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-getter-decorators-return-null-static-getter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-getter-decorators-return-null-static-getter-exec.test.js:55
      @dec static get foo() {
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-getter-decorators-return-object-instance-getter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-getter-decorators-return-object-instance-getter-exec.test.js:55
      @dec get foo() {
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-getter-decorators-return-object-private-instance-getter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-getter-decorators-return-object-private-instance-getter-exec.test.js:55
      @dec get #foo() {
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-getter-decorators-return-object-private-static-getter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-getter-decorators-return-object-private-static-getter-exec.test.js:55
      @dec static get #foo() {
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-getter-decorators-return-object-static-getter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-getter-decorators-return-object-static-getter-exec.test.js:55
      @dec static get foo() {
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-getter-decorators-shim-instance-getter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-getter-decorators-shim-instance-getter-exec.test.js:59
    @dec get foo() {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-getter-decorators-shim-private-instance-getter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-getter-decorators-shim-private-instance-getter-exec.test.js:60
    @dec get #foo() {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-getter-decorators-shim-private-static-getter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-getter-decorators-shim-private-static-getter-exec.test.js:60
    @dec static get #foo() {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-getter-decorators-shim-static-getter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-getter-decorators-shim-static-getter-exec.test.js:59
    @dec static get foo() {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-initializer-order-private-members-class-expression-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-initializer-order-private-members-class-expression-exec.test.js:208
  const Foo = @classDec1 @classDec2 class extends (log.push("extends"), Object) {
              ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-initializer-order-private-members-class-statement-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-initializer-order-private-members-class-statement-exec.test.js:208
  @classDec1 @classDec2 class Foo extends (log.push("extends"), Object) {
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-initializer-order-public-members-class-expression-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-initializer-order-public-members-class-expression-exec.test.js:208
  const Foo = @classDec1 @classDec2 class extends (log.push("extends"), Object) {
              ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-initializer-order-public-members-class-statement-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-initializer-order-public-members-class-statement-exec.test.js:208
  @classDec1 @classDec2 class Foo extends (log.push("extends"), Object) {
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-method-decorators-basic-instance-method-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-method-decorators-basic-instance-method-exec.test.js:67
    @dec("foo", "foo") foo() {}
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-method-decorators-basic-private-instance-method-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-method-decorators-basic-private-instance-method-exec.test.js:69
    @dec #foo() {}
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-method-decorators-basic-private-static-method-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-method-decorators-basic-private-static-method-exec.test.js:69
    @dec static #foo() {}
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-method-decorators-basic-static-method-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-method-decorators-basic-static-method-exec.test.js:67
    @dec("foo", "foo") static foo() {}
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-method-decorators-extra-initializer-instance-method-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-method-decorators-extra-initializer-instance-method-exec.test.js:64
    @dec @dec foo() {}
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-method-decorators-extra-initializer-private-instance-method-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-method-decorators-extra-initializer-private-instance-method-exec.test.js:64
    @dec @dec #foo() {}
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-method-decorators-extra-initializer-private-static-method-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-method-decorators-extra-initializer-private-static-method-exec.test.js:64
    @dec @dec static #foo() {}
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-method-decorators-extra-initializer-static-method-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-method-decorators-extra-initializer-static-method-exec.test.js:64
    @dec @dec static foo() {}
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-method-decorators-order-instance-method-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-method-decorators-order-instance-method-exec.test.js:71
    @dec1 @dec2 foo() {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-method-decorators-order-private-instance-method-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-method-decorators-order-private-instance-method-exec.test.js:72
    @dec1 @dec2 #foo() {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-method-decorators-order-private-static-method-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-method-decorators-order-private-static-method-exec.test.js:72
    @dec1 @dec2 static #foo() {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-method-decorators-order-static-method-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-method-decorators-order-static-method-exec.test.js:71
    @dec1 @dec2 static foo() {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-method-decorators-return-null-instance-method-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-method-decorators-return-null-instance-method-exec.test.js:55
      @dec foo() {}
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-method-decorators-return-null-private-instance-method-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-method-decorators-return-null-private-instance-method-exec.test.js:55
      @dec #foo() {}
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-method-decorators-return-null-private-static-method-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-method-decorators-return-null-private-static-method-exec.test.js:55
      @dec static #foo() {}
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-method-decorators-return-null-static-method-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-method-decorators-return-null-static-method-exec.test.js:55
      @dec static foo() {}
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-method-decorators-return-object-instance-method-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-method-decorators-return-object-instance-method-exec.test.js:55
      @dec foo() {}
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-method-decorators-return-object-private-instance-method-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-method-decorators-return-object-private-instance-method-exec.test.js:55
      @dec #foo() {}
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-method-decorators-return-object-private-static-method-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-method-decorators-return-object-private-static-method-exec.test.js:55
      @dec static #foo() {}
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-method-decorators-return-object-static-method-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-method-decorators-return-object-static-method-exec.test.js:55
      @dec static foo() {}
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-method-decorators-shim-instance-method-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-method-decorators-shim-instance-method-exec.test.js:59
    @dec foo() {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-method-decorators-shim-private-instance-method-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-method-decorators-shim-private-instance-method-exec.test.js:60
    @dec #foo() {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-method-decorators-shim-private-static-method-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-method-decorators-shim-private-static-method-exec.test.js:60
    @dec static #foo() {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-method-decorators-shim-static-method-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-method-decorators-shim-static-method-exec.test.js:59
    @dec static foo() {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-auto-accessor-decorators-basic-instance-auto-accessor-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-auto-accessor-decorators-basic-instance-auto-accessor-exec.test.js:71
    @dec("foo", "get foo", "set foo") accessor foo = 0;
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-auto-accessor-decorators-basic-private-instance-auto-accessor-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-auto-accessor-decorators-basic-private-instance-auto-accessor-exec.test.js:74
    @dec accessor #foo = 0;
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-auto-accessor-decorators-basic-private-static-auto-accessor-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-auto-accessor-decorators-basic-private-static-auto-accessor-exec.test.js:73
    @dec static accessor #foo = 0;
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-auto-accessor-decorators-basic-static-auto-accessor-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-auto-accessor-decorators-basic-static-auto-accessor-exec.test.js:71
    @dec("foo", "get foo", "set foo") static accessor foo = 0;
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-auto-accessor-decorators-extra-initializer-instance-auto-accessor-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-auto-accessor-decorators-extra-initializer-instance-auto-accessor-exec.test.js:64
    @dec @dec accessor foo;
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-auto-accessor-decorators-extra-initializer-private-instance-auto-accessor-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-auto-accessor-decorators-extra-initializer-private-instance-auto-accessor-exec.test.js:64
    @dec @dec accessor #foo;
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-auto-accessor-decorators-extra-initializer-private-static-auto-accessor-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-auto-accessor-decorators-extra-initializer-private-static-auto-accessor-exec.test.js:64
    @dec @dec static accessor #foo;
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-auto-accessor-decorators-extra-initializer-static-auto-accessor-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-auto-accessor-decorators-extra-initializer-static-auto-accessor-exec.test.js:64
    @dec @dec static accessor foo;
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-auto-accessor-decorators-return-null-instance-auto-accessor-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-auto-accessor-decorators-return-null-instance-auto-accessor-exec.test.js:55
      @dec accessor foo;
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-auto-accessor-decorators-return-null-private-instance-auto-accessor-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-auto-accessor-decorators-return-null-private-instance-auto-accessor-exec.test.js:55
      @dec accessor #foo;
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-auto-accessor-decorators-return-null-private-static-auto-accessor-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-auto-accessor-decorators-return-null-private-static-auto-accessor-exec.test.js:55
      @dec static accessor #foo;
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-auto-accessor-decorators-return-null-static-auto-accessor-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-auto-accessor-decorators-return-null-static-auto-accessor-exec.test.js:55
      @dec static accessor foo;
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-auto-accessor-decorators-shim-instance-auto-accessor-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-auto-accessor-decorators-shim-instance-auto-accessor-exec.test.js:70
    @dec accessor foo = 123;
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-auto-accessor-decorators-shim-private-instance-auto-accessor-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-auto-accessor-decorators-shim-private-instance-auto-accessor-exec.test.js:72
    @dec accessor #foo = 123;
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-auto-accessor-decorators-shim-private-static-auto-accessor-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-auto-accessor-decorators-shim-private-static-auto-accessor-exec.test.js:75
      get$foo = (x) => x.#foo;
                        ^

SyntaxError: Private field '#foo' must be declared in an enclosing class
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-auto-accessor-decorators-shim-static-auto-accessor-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-auto-accessor-decorators-shim-static-auto-accessor-exec.test.js:74
    @dec static accessor foo = 123;
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-class-decorators-basic-expression-anonymous-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-class-decorators-basic-expression-anonymous-exec.test.js:61
  const Foo = ((x) => x)(@dec("") class {});
                         ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-class-decorators-basic-expression-array-binding-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-class-decorators-basic-expression-array-binding-exec.test.js:61
  const [Foo = @dec("Foo") class {}] = [];
               ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-class-decorators-basic-expression-assignment-array-binding-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-class-decorators-basic-expression-assignment-array-binding-exec.test.js:62
  [Foo = @dec("Foo") class {}] = [];
         ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-class-decorators-basic-expression-assignment-initializer-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-class-decorators-basic-expression-assignment-initializer-exec.test.js:62
  Foo = @dec("Foo") class {};
        ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-class-decorators-basic-expression-assignment-object-binding-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-class-decorators-basic-expression-assignment-object-binding-exec.test.js:62
  ({Foo = @dec("Foo") class {}} = {});
          ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-class-decorators-basic-expression-instance-auto-accessor-initializer-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-class-decorators-basic-expression-instance-auto-accessor-initializer-exec.test.js:62
    accessor Foo = @dec("Foo") class {};
             ^^^

SyntaxError: Unexpected identifier 'Foo'
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-class-decorators-basic-expression-instance-field-initializer-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-class-decorators-basic-expression-instance-field-initializer-exec.test.js:62
    Foo = @dec("Foo") class {};
          ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-class-decorators-basic-expression-object-binding-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-class-decorators-basic-expression-object-binding-exec.test.js:61
  const { Foo = @dec("Foo") class {} } = {};
                ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-class-decorators-basic-expression-property-value-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-class-decorators-basic-expression-property-value-exec.test.js:61
  const obj = { Foo: @dec("Foo") class {} };
                     ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-class-decorators-basic-expression-static-auto-accessor-initializer-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-class-decorators-basic-expression-static-auto-accessor-initializer-exec.test.js:62
    static accessor Foo = @dec("Foo") class {};
                    ^^^

SyntaxError: Unexpected identifier 'Foo'
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-class-decorators-basic-expression-static-field-initializer-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-class-decorators-basic-expression-static-field-initializer-exec.test.js:62
    static Foo = @dec("Foo") class {};
                 ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-class-decorators-basic-expression-variable-initializer-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-class-decorators-basic-expression-variable-initializer-exec.test.js:61
  const Foo = @dec("Foo") class {};
              ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-class-decorators-basic-statement-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-class-decorators-basic-statement-exec.test.js:61
  @dec("Foo") class Foo {}
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-class-decorators-extra-initializer-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-class-decorators-extra-initializer-exec.test.js:63
  @dec @dec class Foo {}
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-class-decorators-order-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-class-decorators-order-exec.test.js:70
  @dec1 @dec2 class Foo {
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-class-decorators-return-null-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-class-decorators-return-null-exec.test.js:54
    @dec class Foo {}
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-class-decorators-return-object-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-class-decorators-return-object-exec.test.js:54
    @dec class Foo {}
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-decorator-list-evaluation-await-class-expression-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-decorator-list-evaluation-await-class-expression-exec.test.js:53
    (@(log.push(await Promise.resolve(0)), dummy) class extends (log.push(await Promise.resolve(1)), Object) {
     ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-decorator-list-evaluation-await-class-statement-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-decorator-list-evaluation-await-class-statement-exec.test.js:53
    @(log.push(await Promise.resolve(0)), dummy) class Foo extends (log.push(await Promise.resolve(1)), Object) {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-decorator-list-evaluation-class-binding-class-expression-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-decorator-list-evaluation-class-binding-class-expression-exec.test.js:55
  const originalFoo = @(capture(() => Foo)) class Foo {
                      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-decorator-list-evaluation-class-binding-class-statement-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-decorator-list-evaluation-class-binding-class-statement-exec.test.js:56
  @(capture(() => Foo)) class Foo {
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-decorator-list-evaluation-computed-names-class-expression-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-decorator-list-evaluation-computed-names-class-expression-exec.test.js:77
  (@foo(0) class extends (foo(1), Object) {
   ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-decorator-list-evaluation-computed-names-class-statement-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-decorator-list-evaluation-computed-names-class-statement-exec.test.js:77
  @foo(0) class Foo extends (foo(1), Object) {
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-decorator-list-evaluation-inner-private-name-class-expression-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-decorator-list-evaluation-inner-private-name-class-expression-exec.test.js:58
      (@(capture(() => Outer.#foo + 0)) class Foo {
       ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-decorator-list-evaluation-inner-private-name-class-statement-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-decorator-list-evaluation-inner-private-name-class-statement-exec.test.js:58
      @(capture(() => new Foo().#foo + 0)) class Foo {
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-decorator-list-evaluation-outer-private-name-class-expression-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-decorator-list-evaluation-outer-private-name-class-expression-exec.test.js:58
      (@(dummy.#foo(0)) class extends (dummy.#foo(1), Object) {
       ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-decorator-list-evaluation-outer-private-name-class-statement-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-decorator-list-evaluation-outer-private-name-class-statement-exec.test.js:58
      @(dummy.#foo(0)) class Foo extends (dummy.#foo(1), Object) {
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-decorator-list-evaluation-this-class-expression-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-decorator-list-evaluation-this-class-expression-exec.test.js:56
    (@(assertEq(() => this.foo(0), undefined), dummy) class extends (assertEq(() => this.foo(1), undefined), Object) {
     ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-decorator-list-evaluation-this-class-statement-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-decorator-list-evaluation-this-class-statement-exec.test.js:56
    @(assertEq(() => this.foo(0), undefined), dummy) class Foo extends (assertEq(() => this.foo(1), undefined), Object) {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-decorator-metadata-class-expression-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-decorator-metadata-class-expression-exec.test.js:54
  const Foo = @dec class {
              ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-decorator-metadata-class-statement-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-decorator-metadata-class-statement-exec.test.js:54
  @dec class Foo {
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-field-decorators-basic-instance-field-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-field-decorators-basic-instance-field-exec.test.js:68
    @dec("foo") foo = 123;
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-field-decorators-basic-private-instance-field-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-field-decorators-basic-private-instance-field-exec.test.js:70
    @dec #foo = 123;
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-field-decorators-basic-private-static-field-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-field-decorators-basic-private-static-field-exec.test.js:69
    @dec static #foo = 123;
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-field-decorators-basic-static-field-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-field-decorators-basic-static-field-exec.test.js:68
    @dec("foo") static foo = 123;
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-field-decorators-extra-initializer-instance-field-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-field-decorators-extra-initializer-instance-field-exec.test.js:64
    @dec @dec foo;
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-field-decorators-extra-initializer-private-instance-field-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-field-decorators-extra-initializer-private-instance-field-exec.test.js:64
    @dec @dec #foo;
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-field-decorators-extra-initializer-private-static-field-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-field-decorators-extra-initializer-private-static-field-exec.test.js:64
    @dec @dec static #foo;
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-field-decorators-extra-initializer-static-field-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-field-decorators-extra-initializer-static-field-exec.test.js:64
    @dec @dec static foo;
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-field-decorators-order-instance-field-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-field-decorators-order-instance-field-exec.test.js:61
    @dec1 @dec2 foo = 123;
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-field-decorators-order-private-instance-field-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-field-decorators-order-private-instance-field-exec.test.js:62
    @dec1 @dec2 #foo = 123;
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-field-decorators-order-private-static-field-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-field-decorators-order-private-static-field-exec.test.js:62
    @dec1 @dec2 static #foo = 123;
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-field-decorators-order-static-field-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-field-decorators-order-static-field-exec.test.js:61
    @dec1 @dec2 static foo = 123;
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-field-decorators-return-null-instance-field-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-field-decorators-return-null-instance-field-exec.test.js:55
      @dec foo;
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-field-decorators-return-null-private-instance-field-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-field-decorators-return-null-private-instance-field-exec.test.js:55
      @dec #foo;
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-field-decorators-return-null-private-static-field-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-field-decorators-return-null-private-static-field-exec.test.js:55
      @dec static #foo;
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-field-decorators-return-null-static-field-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-field-decorators-return-null-static-field-exec.test.js:55
      @dec static foo;
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-field-decorators-return-object-instance-field-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-field-decorators-return-object-instance-field-exec.test.js:55
      @dec foo;
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-field-decorators-return-object-private-instance-field-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-field-decorators-return-object-private-instance-field-exec.test.js:55
      @dec #foo;
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-field-decorators-return-object-private-static-field-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-field-decorators-return-object-private-static-field-exec.test.js:55
      @dec static #foo;
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-field-decorators-return-object-static-field-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-field-decorators-return-object-static-field-exec.test.js:55
      @dec static foo;
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-field-decorators-shim-instance-field-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-field-decorators-shim-instance-field-exec.test.js:58
    @dec foo = 123;
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-field-decorators-shim-private-instance-field-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-field-decorators-shim-private-instance-field-exec.test.js:62
    @dec #foo = 123;
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-field-decorators-shim-private-static-field-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-field-decorators-shim-private-static-field-exec.test.js:66
      has$foo = (x) => #foo in x;
                    ^

SyntaxError: Private field '#foo' must be declared in an enclosing class
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-field-decorators-shim-static-field-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-field-decorators-shim-static-field-exec.test.js:63
    @dec static foo = 123;
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-getter-decorators-basic-instance-getter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-getter-decorators-basic-instance-getter-exec.test.js:66
    @dec("foo", "get foo") get foo() {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-getter-decorators-basic-private-instance-getter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-getter-decorators-basic-private-instance-getter-exec.test.js:68
    @dec get #foo() {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-getter-decorators-basic-private-static-getter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-getter-decorators-basic-private-static-getter-exec.test.js:68
    @dec static get #foo() {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-getter-decorators-basic-static-getter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-getter-decorators-basic-static-getter-exec.test.js:66
    @dec("foo", "get foo") static get foo() {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-getter-decorators-extra-initializer-instance-getter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-getter-decorators-extra-initializer-instance-getter-exec.test.js:64
    @dec @dec get foo() {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-getter-decorators-extra-initializer-private-instance-getter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-getter-decorators-extra-initializer-private-instance-getter-exec.test.js:64
    @dec @dec get #foo() {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-getter-decorators-extra-initializer-private-static-getter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-getter-decorators-extra-initializer-private-static-getter-exec.test.js:64
    @dec @dec static get #foo() {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-getter-decorators-extra-initializer-static-getter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-getter-decorators-extra-initializer-static-getter-exec.test.js:64
    @dec @dec static get foo() {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-getter-decorators-order-instance-getter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-getter-decorators-order-instance-getter-exec.test.js:71
    @dec1 @dec2 get foo() {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-getter-decorators-order-private-instance-getter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-getter-decorators-order-private-instance-getter-exec.test.js:72
    @dec1 @dec2 get #foo() {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-getter-decorators-order-private-static-getter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-getter-decorators-order-private-static-getter-exec.test.js:72
    @dec1 @dec2 static get #foo() {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-getter-decorators-order-static-getter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-getter-decorators-order-static-getter-exec.test.js:71
    @dec1 @dec2 static get foo() {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-getter-decorators-return-null-instance-getter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-getter-decorators-return-null-instance-getter-exec.test.js:55
      @dec get foo() {
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-getter-decorators-return-null-private-instance-getter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-getter-decorators-return-null-private-instance-getter-exec.test.js:55
      @dec get #foo() {
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-getter-decorators-return-null-private-static-getter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-getter-decorators-return-null-private-static-getter-exec.test.js:55
      @dec static get #foo() {
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-getter-decorators-return-null-static-getter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-getter-decorators-return-null-static-getter-exec.test.js:55
      @dec static get foo() {
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-getter-decorators-return-object-instance-getter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-getter-decorators-return-object-instance-getter-exec.test.js:55
      @dec get foo() {
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-getter-decorators-return-object-private-instance-getter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-getter-decorators-return-object-private-instance-getter-exec.test.js:55
      @dec get #foo() {
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-getter-decorators-return-object-private-static-getter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-getter-decorators-return-object-private-static-getter-exec.test.js:55
      @dec static get #foo() {
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-getter-decorators-return-object-static-getter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-getter-decorators-return-object-static-getter-exec.test.js:55
      @dec static get foo() {
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-getter-decorators-shim-instance-getter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-getter-decorators-shim-instance-getter-exec.test.js:59
    @dec get foo() {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-getter-decorators-shim-private-instance-getter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-getter-decorators-shim-private-instance-getter-exec.test.js:60
    @dec get #foo() {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-getter-decorators-shim-private-static-getter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-getter-decorators-shim-private-static-getter-exec.test.js:60
    @dec static get #foo() {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-getter-decorators-shim-static-getter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-getter-decorators-shim-static-getter-exec.test.js:59
    @dec static get foo() {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-initializer-order-private-members-class-expression-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-initializer-order-private-members-class-expression-exec.test.js:208
  const Foo = @classDec1 @classDec2 class extends (log.push("extends"), Object) {
              ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-initializer-order-private-members-class-statement-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-initializer-order-private-members-class-statement-exec.test.js:208
  @classDec1 @classDec2 class Foo extends (log.push("extends"), Object) {
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-initializer-order-public-members-class-expression-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-initializer-order-public-members-class-expression-exec.test.js:208
  const Foo = @classDec1 @classDec2 class extends (log.push("extends"), Object) {
              ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-initializer-order-public-members-class-statement-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-initializer-order-public-members-class-statement-exec.test.js:208
  @classDec1 @classDec2 class Foo extends (log.push("extends"), Object) {
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-method-decorators-basic-instance-method-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-method-decorators-basic-instance-method-exec.test.js:67
    @dec("foo", "foo") foo() {}
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-method-decorators-basic-private-instance-method-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-method-decorators-basic-private-instance-method-exec.test.js:69
    @dec #foo() {}
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-method-decorators-basic-private-static-method-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-method-decorators-basic-private-static-method-exec.test.js:69
    @dec static #foo() {}
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-method-decorators-basic-static-method-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-method-decorators-basic-static-method-exec.test.js:67
    @dec("foo", "foo") static foo() {}
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-method-decorators-extra-initializer-instance-method-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-method-decorators-extra-initializer-instance-method-exec.test.js:64
    @dec @dec foo() {}
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-method-decorators-extra-initializer-private-instance-method-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-method-decorators-extra-initializer-private-instance-method-exec.test.js:64
    @dec @dec #foo() {}
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-method-decorators-extra-initializer-private-static-method-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-method-decorators-extra-initializer-private-static-method-exec.test.js:64
    @dec @dec static #foo() {}
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-method-decorators-extra-initializer-static-method-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-method-decorators-extra-initializer-static-method-exec.test.js:64
    @dec @dec static foo() {}
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-method-decorators-order-instance-method-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-method-decorators-order-instance-method-exec.test.js:71
    @dec1 @dec2 foo() {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-method-decorators-order-private-instance-method-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-method-decorators-order-private-instance-method-exec.test.js:72
    @dec1 @dec2 #foo() {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-method-decorators-order-private-static-method-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-method-decorators-order-private-static-method-exec.test.js:72
    @dec1 @dec2 static #foo() {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-method-decorators-order-static-method-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-method-decorators-order-static-method-exec.test.js:71
    @dec1 @dec2 static foo() {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-method-decorators-return-null-instance-method-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-method-decorators-return-null-instance-method-exec.test.js:55
      @dec foo() {}
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-method-decorators-return-null-private-instance-method-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-method-decorators-return-null-private-instance-method-exec.test.js:55
      @dec #foo() {}
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-method-decorators-return-null-private-static-method-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-method-decorators-return-null-private-static-method-exec.test.js:55
      @dec static #foo() {}
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-method-decorators-return-null-static-method-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-method-decorators-return-null-static-method-exec.test.js:55
      @dec static foo() {}
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-method-decorators-return-object-instance-method-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-method-decorators-return-object-instance-method-exec.test.js:55
      @dec foo() {}
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-method-decorators-return-object-private-instance-method-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-method-decorators-return-object-private-instance-method-exec.test.js:55
      @dec #foo() {}
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-method-decorators-return-object-private-static-method-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-method-decorators-return-object-private-static-method-exec.test.js:55
      @dec static #foo() {}
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-method-decorators-return-object-static-method-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-method-decorators-return-object-static-method-exec.test.js:55
      @dec static foo() {}
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-method-decorators-shim-instance-method-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-method-decorators-shim-instance-method-exec.test.js:59
    @dec foo() {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-method-decorators-shim-private-instance-method-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-method-decorators-shim-private-instance-method-exec.test.js:60
    @dec #foo() {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-method-decorators-shim-private-static-method-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-method-decorators-shim-private-static-method-exec.test.js:60
    @dec static #foo() {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-method-decorators-shim-static-method-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-method-decorators-shim-static-method-exec.test.js:59
    @dec static foo() {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-setter-decorators-basic-instance-setter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-setter-decorators-basic-instance-setter-exec.test.js:69
    @dec("foo", "set foo") set foo(x) {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-setter-decorators-basic-private-instance-setter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-setter-decorators-basic-private-instance-setter-exec.test.js:72
    @dec set #foo(x) {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-setter-decorators-basic-private-static-setter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-setter-decorators-basic-private-static-setter-exec.test.js:71
    @dec static set #foo(x) {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-setter-decorators-basic-static-setter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-setter-decorators-basic-static-setter-exec.test.js:69
    @dec("foo", "set foo") static set foo(x) {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-setter-decorators-extra-initializer-instance-setter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-setter-decorators-extra-initializer-instance-setter-exec.test.js:64
    @dec @dec set foo(x) {}
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-setter-decorators-extra-initializer-private-instance-setter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-setter-decorators-extra-initializer-private-instance-setter-exec.test.js:64
    @dec @dec set #foo(x) {}
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-setter-decorators-extra-initializer-private-static-setter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-setter-decorators-extra-initializer-private-static-setter-exec.test.js:64
    @dec @dec static set #foo(x) {}
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-setter-decorators-extra-initializer-static-setter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-setter-decorators-extra-initializer-static-setter-exec.test.js:64
    @dec @dec static set foo(x) {}
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-setter-decorators-order-instance-setter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-setter-decorators-order-instance-setter-exec.test.js:71
    @dec1 @dec2 set foo(x) {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-setter-decorators-order-private-instance-setter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-setter-decorators-order-private-instance-setter-exec.test.js:72
    @dec1 @dec2 set #foo(x) {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-setter-decorators-order-private-static-setter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-setter-decorators-order-private-static-setter-exec.test.js:72
    @dec1 @dec2 static set #foo(x) {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-setter-decorators-order-static-setter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-setter-decorators-order-static-setter-exec.test.js:71
    @dec1 @dec2 static set foo(x) {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-setter-decorators-return-null-instance-setter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-setter-decorators-return-null-instance-setter-exec.test.js:55
      @dec set foo(x) {}
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-setter-decorators-return-null-private-instance-setter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-setter-decorators-return-null-private-instance-setter-exec.test.js:55
      @dec set #foo(x) {}
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-setter-decorators-return-null-private-static-setter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-setter-decorators-return-null-private-static-setter-exec.test.js:55
      @dec static set #foo(x) {}
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-setter-decorators-return-null-static-setter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-setter-decorators-return-null-static-setter-exec.test.js:55
      @dec static set foo(x) {}
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-setter-decorators-return-object-instance-setter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-setter-decorators-return-object-instance-setter-exec.test.js:55
      @dec set foo(x) {}
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-setter-decorators-return-object-private-instance-setter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-setter-decorators-return-object-private-instance-setter-exec.test.js:55
      @dec set #foo(x) {}
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-setter-decorators-return-object-private-static-setter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-setter-decorators-return-object-private-static-setter-exec.test.js:55
      @dec static set #foo(x) {}
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-setter-decorators-return-object-static-setter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-setter-decorators-return-object-static-setter-exec.test.js:55
      @dec static set foo(x) {}
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-setter-decorators-shim-instance-setter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-setter-decorators-shim-instance-setter-exec.test.js:59
    @dec set foo(x) {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-setter-decorators-shim-private-instance-setter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-setter-decorators-shim-private-instance-setter-exec.test.js:60
    @dec set #foo(x) {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-setter-decorators-shim-private-static-setter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-setter-decorators-shim-private-static-setter-exec.test.js:60
    @dec static set #foo(x) {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-setter-decorators-shim-static-setter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-setter-decorators-shim-static-setter-exec.test.js:59
    @dec static set foo(x) {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-setter-decorators-basic-instance-setter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-setter-decorators-basic-instance-setter-exec.test.js:69
    @dec("foo", "set foo") set foo(x) {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-setter-decorators-basic-private-instance-setter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-setter-decorators-basic-private-instance-setter-exec.test.js:72
    @dec set #foo(x) {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-setter-decorators-basic-private-static-setter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-setter-decorators-basic-private-static-setter-exec.test.js:71
    @dec static set #foo(x) {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-setter-decorators-basic-static-setter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-setter-decorators-basic-static-setter-exec.test.js:69
    @dec("foo", "set foo") static set foo(x) {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-setter-decorators-extra-initializer-instance-setter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-setter-decorators-extra-initializer-instance-setter-exec.test.js:64
    @dec @dec set foo(x) {}
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-setter-decorators-extra-initializer-private-instance-setter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-setter-decorators-extra-initializer-private-instance-setter-exec.test.js:64
    @dec @dec set #foo(x) {}
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-setter-decorators-extra-initializer-private-static-setter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-setter-decorators-extra-initializer-private-static-setter-exec.test.js:64
    @dec @dec static set #foo(x) {}
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-setter-decorators-extra-initializer-static-setter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-setter-decorators-extra-initializer-static-setter-exec.test.js:64
    @dec @dec static set foo(x) {}
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-setter-decorators-order-instance-setter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-setter-decorators-order-instance-setter-exec.test.js:71
    @dec1 @dec2 set foo(x) {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-setter-decorators-order-private-instance-setter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-setter-decorators-order-private-instance-setter-exec.test.js:72
    @dec1 @dec2 set #foo(x) {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-setter-decorators-order-private-static-setter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-setter-decorators-order-private-static-setter-exec.test.js:72
    @dec1 @dec2 static set #foo(x) {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-setter-decorators-order-static-setter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-setter-decorators-order-static-setter-exec.test.js:71
    @dec1 @dec2 static set foo(x) {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-setter-decorators-return-null-instance-setter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-setter-decorators-return-null-instance-setter-exec.test.js:55
      @dec set foo(x) {}
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-setter-decorators-return-null-private-instance-setter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-setter-decorators-return-null-private-instance-setter-exec.test.js:55
      @dec set #foo(x) {}
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-setter-decorators-return-null-private-static-setter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-setter-decorators-return-null-private-static-setter-exec.test.js:55
      @dec static set #foo(x) {}
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-setter-decorators-return-null-static-setter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-setter-decorators-return-null-static-setter-exec.test.js:55
      @dec static set foo(x) {}
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-setter-decorators-return-object-instance-setter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-setter-decorators-return-object-instance-setter-exec.test.js:55
      @dec set foo(x) {}
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-setter-decorators-return-object-private-instance-setter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-setter-decorators-return-object-private-instance-setter-exec.test.js:55
      @dec set #foo(x) {}
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-setter-decorators-return-object-private-static-setter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-setter-decorators-return-object-private-static-setter-exec.test.js:55
      @dec static set #foo(x) {}
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-setter-decorators-return-object-static-setter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-setter-decorators-return-object-static-setter-exec.test.js:55
      @dec static set foo(x) {}
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-setter-decorators-shim-instance-setter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-setter-decorators-shim-instance-setter-exec.test.js:59
    @dec set foo(x) {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-setter-decorators-shim-private-instance-setter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-setter-decorators-shim-private-instance-setter-exec.test.js:60
    @dec set #foo(x) {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-setter-decorators-shim-private-static-setter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-setter-decorators-shim-private-static-setter-exec.test.js:60
    @dec static set #foo(x) {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-setter-decorators-shim-static-setter-exec.test.js
./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-setter-decorators-shim-static-setter-exec.test.js:59
    @dec static set foo(x) {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/oxc/babel-plugin-transform-class-properties-test-fixtures-private-field-resolve-to-method-in-computed-key-exec.test.js
AssertionError: expected [Function] to throw error including 'Receiver must be an instance of class…' but got 'Private element is not present on thi…'
    at Proxy.methodWrapper (./node_modules/chai/index.js:1737:25)
    at ./tasks/transform_conformance/fixtures/oxc/babel-plugin-transform-class-properties-test-fixtures-private-field-resolve-to-method-in-computed-key-exec.test.js:96:33

./fixtures/oxc/babel-plugin-transform-class-properties-test-fixtures-static-super-tagged-template-exec.test.js
AssertionError: expected undefined to be [Function C] // Object.is equality
    at ./tasks/transform_conformance/fixtures/oxc/babel-plugin-transform-class-properties-test-fixtures-static-super-tagged-template-exec.test.js:15:17
