commit: 16a2d671

node: v26.12.1

Passed: 398 of 406 (98.03%)

Failures:

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-decorator-list-evaluation-class-binding-class-expression-exec.test.js
AssertionError: expected 10 to be +0 // Object.is equality
    at ./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-decorator-list-evaluation-class-binding-class-expression-exec.test.js:101:18
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:59)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)
    at async VitestModuleRunner.import (./node_modules/vite/dist/node/module-runner.js:1150:10)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-decorator-list-evaluation-class-binding-class-statement-exec.test.js
AssertionError: expected 10 to be +0 // Object.is equality
    at ./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-decorator-list-evaluation-class-binding-class-statement-exec.test.js:139:18
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:59)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)
    at async VitestModuleRunner.import (./node_modules/vite/dist/node/module-runner.js:1150:10)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-decorator-list-evaluation-inner-private-name-class-expression-exec.test.js
ReferenceError: Foo is not defined
    at ./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-decorator-list-evaluation-inner-private-name-class-expression-exec.test.js:66:293
    at run (./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-decorator-list-evaluation-inner-private-name-class-expression-exec.test.js:100:34)
    at ./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-decorator-list-evaluation-inner-private-name-class-expression-exec.test.js:103:7
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:59)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)
    at async VitestModuleRunner.import (./node_modules/vite/dist/node/module-runner.js:1150:10)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-decorator-list-evaluation-class-binding-class-expression-exec.test.js
AssertionError: expected 10 to be +0 // Object.is equality
    at ./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-decorator-list-evaluation-class-binding-class-expression-exec.test.js:101:18
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:59)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)
    at async VitestModuleRunner.import (./node_modules/vite/dist/node/module-runner.js:1150:10)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-decorator-list-evaluation-class-binding-class-statement-exec.test.js
AssertionError: expected 10 to be +0 // Object.is equality
    at ./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-decorator-list-evaluation-class-binding-class-statement-exec.test.js:139:18
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:59)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)
    at async VitestModuleRunner.import (./node_modules/vite/dist/node/module-runner.js:1150:10)

./fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-decorator-list-evaluation-inner-private-name-class-expression-exec.test.js
ReferenceError: Foo is not defined
    at ./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-decorator-list-evaluation-inner-private-name-class-expression-exec.test.js:66:297
    at run (./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-decorator-list-evaluation-inner-private-name-class-expression-exec.test.js:101:34)
    at ./tasks/transform_conformance/fixtures/oxc/babel-plugin-proposal-decorators-test-fixtures-esbuild-native-decorator-list-evaluation-inner-private-name-class-expression-exec.test.js:104:7
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:59)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)
    at async VitestModuleRunner.import (./node_modules/vite/dist/node/module-runner.js:1150:10)

./fixtures/oxc/babel-plugin-transform-class-properties-test-fixtures-private-field-resolve-to-method-in-computed-key-exec.test.js
AssertionError: expected [Function] to throw error including 'Receiver must be an instance of class…' but got 'Private element is not present on thi…'
    at Proxy.methodWrapper (./node_modules/chai/index.js:1737:25)
    at ./tasks/transform_conformance/fixtures/oxc/babel-plugin-transform-class-properties-test-fixtures-private-field-resolve-to-method-in-computed-key-exec.test.js:96:33

./fixtures/oxc/babel-plugin-transform-class-properties-test-fixtures-static-super-tagged-template-exec.test.js
AssertionError: expected undefined to be [Function C] // Object.is equality
    at ./tasks/transform_conformance/fixtures/oxc/babel-plugin-transform-class-properties-test-fixtures-static-super-tagged-template-exec.test.js:15:17
