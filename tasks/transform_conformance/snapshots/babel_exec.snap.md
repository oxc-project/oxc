commit: 51225612

node: v26.12.1

Passed: 345 of 602 (57.31%)

Failures:

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-accessors--to-es2015-context-name-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-accessors--to-es2015-context-name-exec.test.js:11
  @dec static accessor a;
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-accessors--to-es2015-context-name-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-accessors--to-es2015-private-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-accessors--to-es2015-private-exec.test.js:21
  @dec accessor #a;
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-accessors--to-es2015-private-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-accessors--to-es2015-public-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-accessors--to-es2015-public-exec.test.js:21
  @dec accessor a;
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-accessors--to-es2015-public-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-accessors--to-es2015-static-private-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-accessors--to-es2015-static-private-exec.test.js:21
  @dec static accessor #a;
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-accessors--to-es2015-static-private-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-accessors--to-es2015-static-public-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-accessors--to-es2015-static-public-exec.test.js:21
  @dec static accessor a;
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-accessors--to-es2015-static-public-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-accessors--to-es2015-static-public-this-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-accessors--to-es2015-static-public-this-exec.test.js:3
  static accessor x;
                  ^

SyntaxError: Unexpected identifier 'x'
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-accessors--to-es2015-static-public-this-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-accessors--to-es2015-undecorated-private-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-accessors--to-es2015-undecorated-private-exec.test.js:3
  accessor #a;
           ^^

SyntaxError: Unexpected identifier '#a'
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-accessors--to-es2015-undecorated-private-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-accessors--to-es2015-undecorated-public-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-accessors--to-es2015-undecorated-public-exec.test.js:3
  accessor a;
           ^

SyntaxError: Unexpected identifier 'a'
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-accessors--to-es2015-undecorated-public-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-accessors--to-es2015-undecorated-static-private-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-accessors--to-es2015-undecorated-static-private-exec.test.js:3
  static accessor #a;
                  ^^

SyntaxError: Unexpected identifier '#a'
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-accessors--to-es2015-undecorated-static-private-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-accessors--to-es2015-undecorated-static-public-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-accessors--to-es2015-undecorated-static-public-exec.test.js:3
  static accessor a;
                  ^

SyntaxError: Unexpected identifier 'a'
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-accessors--to-es2015-undecorated-static-public-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-accessors-context-name-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-accessors-context-name-exec.test.js:11
  @dec static accessor a;
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-accessors-context-name-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-accessors-static-public-inherited-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-accessors-static-public-inherited-exec.test.js:3
  static accessor x;
                  ^

SyntaxError: Unexpected identifier 'x'
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-accessors-static-public-inherited-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-ctx-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-ctx-exec.test.js:6
@dec class A {}
^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-ctx-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-decorator-access-modified-classes-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-decorator-access-modified-classes-exec.test.js:16
@decorateB @decorateA class Target {}
^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-decorator-access-modified-classes-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-decorator-access-modified-fields-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-decorator-access-modified-fields-exec.test.js:8
@classDec class C {
^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-decorator-access-modified-fields-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-decorator-access-modified-methods-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-decorator-access-modified-methods-exec.test.js:8
@classDec class C {
^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-decorator-access-modified-methods-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-inheritance-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-inheritance-exec.test.js:7
@dec1 class Bar {}
^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-inheritance-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-initializers-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-initializers-exec.test.js:8
@dec1 class Foo {
^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-initializers-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-exec.test.js:7
@dec class Foo {
^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-block-super-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-block-super-exec.test.js:34
      @dec class B extends idFactory("B") {
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-block-super-exec.test.js:12:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-decorator-initializer-this-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-decorator-initializer-this-exec.test.js:18
@dec class Foo {
^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-decorator-initializer-this-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-installed-on-correct-class-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-installed-on-correct-class-exec.test.js:8
@dec class Foo {
^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-installed-on-correct-class-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-method-private-environment-super-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-method-private-environment-super-exec.test.js:34
      @dec class B extends idFactory("B") {
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-method-private-environment-super-exec.test.js:12:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-method-private-instance-element-super-assumption-ignoreFunctionLength-exec.test.js
Error: Transform error
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-method-private-instance-element-super-assumption-ignoreFunctionLength-exec.test.js:2:1
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-method-private-instance-element-super-assumption-ignoreFunctionLength-exec.test.js:13:7

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-method-private-instance-element-super-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-method-private-instance-element-super-exec.test.js:14
  @dec class Foo extends class {} {
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-method-private-instance-element-super-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-private-instance-element-access-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-private-instance-element-access-exec.test.js:10
  @dec class Foo {
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-private-instance-element-access-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-private-instance-element-destructuring-exec.test.js
Error: Transform error
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-private-instance-element-destructuring-exec.test.js:2:1
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-private-instance-element-destructuring-exec.test.js:13:7

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-property-super-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-property-super-exec.test.js:34
      @dec class B extends idFactory("B") {
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-property-super-exec.test.js:12:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-this-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-this-exec.test.js:4
@((() => Bar)) class Foo {
^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-this-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-with-expr-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-with-expr-exec.test.js:7
const Foo = @dec class Bar {
            ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-with-expr-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-ctx-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-ctx-exec.test.js:6
@dec class A {}
^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-ctx-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-decorator-access-modified-classes-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-decorator-access-modified-classes-exec.test.js:16
@decorateB @decorateA class Target {}
^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-decorator-access-modified-classes-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-decorator-access-modified-fields-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-decorator-access-modified-fields-exec.test.js:8
@classDec class C {
^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-decorator-access-modified-fields-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-decorator-access-modified-methods-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-decorator-access-modified-methods-exec.test.js:8
@classDec class C {
^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-decorator-access-modified-methods-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-expressions-named-evaluation-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-expressions-named-evaluation-exec.test.js:12
const __vite_ssr_export_default__ = @dec class {};
                                    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at createScript (node:vm:270:10)
    at Object.runInThisContext (node:vm:318:10)
    at VitestModuleRunner.directRequest (./node_modules/vite/dist/node/module-runner.js:1288:80)
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async VitestModuleRunner.cachedRequest (./node_modules/vite/dist/node/module-runner.js:1195:73)

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-replacement-static-block-super-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-replacement-static-block-super-exec.test.js:23
      @dec class B extends idFactory("B") {
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-replacement-static-block-super-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-replacement-static-elements-super-this-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-replacement-static-elements-super-this-exec.test.js:7
      static accessor ["x" + hint] = hint + "Super" + ".x";
                      ^

SyntaxError: Unexpected token '['
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-replacement-static-elements-super-this-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-replacement-static-method-private-environment-super-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-replacement-static-method-private-environment-super-exec.test.js:23
      @dec class B extends idFactory("B") {
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-replacement-static-method-private-environment-super-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-replacement-static-method-private-instance-element-super-assumption-ignoreFunctionLength-exec.test.js
Error: Transform error
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-replacement-static-method-private-instance-element-super-assumption-ignoreFunctionLength-exec.test.js:2:1
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-replacement-static-method-private-instance-element-super-assumption-ignoreFunctionLength-exec.test.js:13:7

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-replacement-static-method-private-instance-element-super-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-replacement-static-method-private-instance-element-super-exec.test.js:14
  @dec class Foo extends class {} {
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-replacement-static-method-private-instance-element-super-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-replacement-static-private-instance-element-access-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-replacement-static-private-instance-element-access-exec.test.js:10
  @dec class Foo {
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-replacement-static-private-instance-element-access-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-replacement-static-private-instance-element-destructuring-exec.test.js
Error: Transform error
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-replacement-static-private-instance-element-destructuring-exec.test.js:2:1
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-replacement-static-private-instance-element-destructuring-exec.test.js:13:7

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-replacement-static-property-super-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-replacement-static-property-super-exec.test.js:23
      @dec class B extends idFactory("B") {
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-replacement-static-property-super-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-duplicated-keys--to-es2015-computed-keys-same-ast-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-duplicated-keys--to-es2015-computed-keys-same-ast-exec.test.js:14
  @dec [getKey()]() {
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-duplicated-keys--to-es2015-computed-keys-same-ast-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-duplicated-keys--to-es2015-computed-keys-same-value-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-duplicated-keys--to-es2015-computed-keys-same-value-exec.test.js:11
    @dec [getKey()]() {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-duplicated-keys--to-es2015-computed-keys-same-value-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-duplicated-keys--to-es2015-method-and-field-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-duplicated-keys--to-es2015-method-and-field-exec.test.js:10
  @dec a = 123;
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-duplicated-keys--to-es2015-method-and-field-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-duplicated-keys--to-es2015-methods-with-same-key-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-duplicated-keys--to-es2015-methods-with-same-key-exec.test.js:8
    @dec a() {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-duplicated-keys--to-es2015-methods-with-same-key-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-fields--to-es2015-context-name-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-fields--to-es2015-context-name-exec.test.js:11
  @dec static a;
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-fields--to-es2015-context-name-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-fields--to-es2015-private-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-fields--to-es2015-private-exec.test.js:9
  @dec #a;
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-fields--to-es2015-private-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-fields--to-es2015-public-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-fields--to-es2015-public-exec.test.js:9
  @dec a;
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-fields--to-es2015-public-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-fields--to-es2015-static-private-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-fields--to-es2015-static-private-exec.test.js:9
  @dec static #a;
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-fields--to-es2015-static-private-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-fields--to-es2015-static-public-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-fields--to-es2015-static-public-exec.test.js:9
  @dec static a;
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-fields--to-es2015-static-public-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-fields-context-name-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-fields-context-name-exec.test.js:11
  @dec static a;
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-fields-context-name-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-getters--to-es2015-context-name-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-getters--to-es2015-context-name-exec.test.js:11
  @dec static get a() {}
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-getters--to-es2015-context-name-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-getters--to-es2015-private-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-getters--to-es2015-private-exec.test.js:13
  @dec get #a() {
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-getters--to-es2015-private-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-getters--to-es2015-public-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-getters--to-es2015-public-exec.test.js:13
  @dec get a() {
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-getters--to-es2015-public-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-getters--to-es2015-static-private-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-getters--to-es2015-static-private-exec.test.js:13
  @dec static get #a() {
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-getters--to-es2015-static-private-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-getters--to-es2015-static-public-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-getters--to-es2015-static-public-exec.test.js:13
  @dec static get a() {
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-getters--to-es2015-static-public-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-getters-and-setters--to-es2015-private-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-getters-and-setters--to-es2015-private-exec.test.js:18
  @dec get #a() {
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-getters-and-setters--to-es2015-private-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-getters-and-setters--to-es2015-public-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-getters-and-setters--to-es2015-public-exec.test.js:18
  @dec get a() {
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-getters-and-setters--to-es2015-public-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-getters-and-setters--to-es2015-static-private-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-getters-and-setters--to-es2015-static-private-exec.test.js:18
  @dec static get #a() {
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-getters-and-setters--to-es2015-static-private-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-getters-and-setters--to-es2015-static-public-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-getters-and-setters--to-es2015-static-public-exec.test.js:18
  @dec static get a() {
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-getters-and-setters--to-es2015-static-public-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-getters-context-name-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-getters-context-name-exec.test.js:11
  @dec static get a() {}
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-getters-context-name-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-methods--to-es2015-context-name-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-methods--to-es2015-context-name-exec.test.js:11
  @dec static a() {}
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-methods--to-es2015-context-name-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-methods--to-es2015-private-async-and-generator-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-methods--to-es2015-private-async-and-generator-exec.test.js:4
  @((function(fn) {
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-methods--to-es2015-private-async-and-generator-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-methods--to-es2015-private-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-methods--to-es2015-private-exec.test.js:12
  @dec #a() {
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-methods--to-es2015-private-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-methods--to-es2015-private-with-initializers-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-methods--to-es2015-private-with-initializers-exec.test.js:13
  @dec #a() {
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-methods--to-es2015-private-with-initializers-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-methods--to-es2015-public-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-methods--to-es2015-public-exec.test.js:12
  @dec a() {
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-methods--to-es2015-public-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-methods--to-es2015-public-with-initializers-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-methods--to-es2015-public-with-initializers-exec.test.js:13
  @dec a() {
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-methods--to-es2015-public-with-initializers-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-methods--to-es2015-static-private-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-methods--to-es2015-static-private-exec.test.js:12
  @dec static #a() {
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-methods--to-es2015-static-private-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-methods--to-es2015-static-private-with-initializers-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-methods--to-es2015-static-private-with-initializers-exec.test.js:13
  @dec static #a() {
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-methods--to-es2015-static-private-with-initializers-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-methods--to-es2015-static-public-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-methods--to-es2015-static-public-exec.test.js:12
  @dec static a() {
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-methods--to-es2015-static-public-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-methods--to-es2015-static-public-with-initializers-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-methods--to-es2015-static-public-with-initializers-exec.test.js:13
  @dec static a() {
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-methods--to-es2015-static-public-with-initializers-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-methods-context-name-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-methods-context-name-exec.test.js:11
  @dec static a() {}
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-methods-context-name-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-methods-private-async-and-generator-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-methods-private-async-and-generator-exec.test.js:4
  @((function(fn) {
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-methods-private-async-and-generator-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-class-and-method-decorators-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-class-and-method-decorators-exec.test.js:12
new @dec class C1 {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-class-and-method-decorators-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-class-and-property-decorators-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-class-and-property-decorators-exec.test.js:12
new @dec class C1 {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-class-and-property-decorators-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-class-in-for-loop-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-class-in-for-loop-exec.test.js:8
      @noop [i];
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-class-in-for-loop-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-class-underscore-property-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-class-underscore-property-exec.test.js:7
    @noopFactory() static _ = "B";
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-class-underscore-property-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-ctx-access-public-other-this-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-ctx-access-public-other-this-exec.test.js:9
  @dec static accessor a;
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-ctx-access-public-other-this-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-decorator-evaluation-arguments-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-decorator-evaluation-arguments-exec.test.js:10
      @decFactory(arguments) #p;
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-decorator-evaluation-arguments-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-decorator-evaluation-new-target-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-decorator-evaluation-new-target-exec.test.js:10
      @decFactory(new.target) #p;
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-decorator-evaluation-new-target-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-decorator-evaluation-private-environment-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-decorator-evaluation-private-environment-exec.test.js:20
        @(capture((_) => _.#X, (_) => _.#x), dummy) method() {}
        ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-decorator-evaluation-private-environment-exec.test.js:16:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-decorator-evaluation-super-getter-setter-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-decorator-evaluation-super-getter-setter-exec.test.js:18
        @(super.setter = super.getter, noop) p;
        ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-decorator-evaluation-super-getter-setter-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-decorator-evaluation-super-property-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-decorator-evaluation-super-property-exec.test.js:8
        @(receivedName = super.name, noop) p;
        ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-decorator-evaluation-super-property-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-decorator-evaluation-this-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-decorator-evaluation-this-exec.test.js:11
        @decFactory(this.name) #p;
        ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-decorator-evaluation-this-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-decorator-evaluation-yield-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-decorator-evaluation-yield-exec.test.js:6
      @(yield 1, noop) method() {}
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-decorator-evaluation-yield-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-decorator-evaluation-yield-private-super-property-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-decorator-evaluation-yield-private-super-property-exec.test.js:16
        @(yield* super.id((_) => _.#X), dummy) method() {}
        ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-decorator-evaluation-yield-private-super-property-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-initField-existing-derived-constructor-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-initField-existing-derived-constructor-exec.test.js:20
    @deco accessor foo = 42;
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-initField-existing-derived-constructor-exec.test.js:12:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-initProto-existing-derived-constructor-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-initProto-existing-derived-constructor-exec.test.js:20
    @deco method() {}
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-initProto-existing-derived-constructor-exec.test.js:12:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-initializer-property-ignored-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-initializer-property-ignored-exec.test.js:17
  @decorator accessor x;
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-initializer-property-ignored-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-initializer-timing-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-initializer-timing-exec.test.js:10
  @dec1 foo() {}
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-initializer-timing-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-invalid-non-callable-decorators-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-invalid-non-callable-decorators-exec.test.js:9
  expect(() => @nonCallable class {}).toThrow("A decorator must be a function");
               ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-invalid-non-callable-decorators-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-leaked-context-addInitializer-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-leaked-context-addInitializer-exec.test.js:12
    @callCapturedFunc @decMethod m() {}
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-leaked-context-addInitializer-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-leaked-context-addInitializer-throw-2-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-leaked-context-addInitializer-throw-2-exec.test.js:9
    @decMethod m() {}
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-leaked-context-addInitializer-throw-2-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-leaked-context-addInitializer-throw-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-leaked-context-addInitializer-throw-exec.test.js:9
    @decMethod m() {}
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-leaked-context-addInitializer-throw-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-outer-class-binding-mutated-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-outer-class-binding-mutated-exec.test.js:22
    @capture(() => K) @assertUninitialized(() => K) [(capture(() => K), assertUninitialized(() => K))];
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-outer-class-binding-mutated-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-private-name-in-class-decorator-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-private-name-in-class-decorator-exec.test.js:5
    @(_x) class B extends A {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-private-name-in-class-decorator-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-this-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-this-exec.test.js:10
@o1.dec @dec @o2.dec class A {
^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-this-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-class-in-for-loop-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-class-in-for-loop-exec.test.js:8
      @noop [i];
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-class-in-for-loop-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-class-not-available-during-decorators-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-class-not-available-during-decorators-exec.test.js:7
@dec(() => {
^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-class-not-available-during-decorators-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-class-underscore-property-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-class-underscore-property-exec.test.js:7
    @noopFactory() static _ = "B";
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-class-underscore-property-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-ctx-access-public-other-this-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-ctx-access-public-other-this-exec.test.js:9
  @dec static accessor a;
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-ctx-access-public-other-this-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-decorator-evaluation-arguments-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-decorator-evaluation-arguments-exec.test.js:10
      @decFactory(arguments) #p;
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-decorator-evaluation-arguments-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-decorator-evaluation-new-target-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-decorator-evaluation-new-target-exec.test.js:10
      @decFactory(new.target) #p;
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-decorator-evaluation-new-target-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-decorator-evaluation-private-environment-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-decorator-evaluation-private-environment-exec.test.js:17
        @(capture((_) => _.#X, (_) => _.#x), dummy) method() {}
        ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-decorator-evaluation-private-environment-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-decorator-evaluation-super-getter-setter-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-decorator-evaluation-super-getter-setter-exec.test.js:18
        @(super.setter = super.getter, noop) p;
        ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-decorator-evaluation-super-getter-setter-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-decorator-evaluation-super-property-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-decorator-evaluation-super-property-exec.test.js:8
        @(receivedName = super.name, noop) p;
        ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-decorator-evaluation-super-property-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-decorator-evaluation-this-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-decorator-evaluation-this-exec.test.js:11
        @decFactory(this.name) #p;
        ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-decorator-evaluation-this-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-decorator-evaluation-yield-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-decorator-evaluation-yield-exec.test.js:6
      @(yield 1, noop) method() {}
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-decorator-evaluation-yield-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-decorator-evaluation-yield-private-super-property-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-decorator-evaluation-yield-private-super-property-exec.test.js:15
        @(yield* super.id((_) => _.#X), dummy) method() {}
        ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-decorator-evaluation-yield-private-super-property-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-initField-existing-derived-constructor-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-initField-existing-derived-constructor-exec.test.js:19
    @deco accessor foo = 42;
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-initField-existing-derived-constructor-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-initProto-existing-derived-constructor-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-initProto-existing-derived-constructor-exec.test.js:19
    @deco method() {}
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-initProto-existing-derived-constructor-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-initializer-property-ignored-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-initializer-property-ignored-exec.test.js:17
  @decorator accessor x;
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-initializer-property-ignored-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-invalid-non-callable-decorators-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-invalid-non-callable-decorators-exec.test.js:9
  expect(() => @nonCallable class {}).toThrow("A decorator must be a function");
               ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-invalid-non-callable-decorators-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-outer-class-binding-mutated-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-outer-class-binding-mutated-exec.test.js:22
    @capture(() => K) @assertUninitialized(() => K) [(capture(() => K), assertUninitialized(() => K))];
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-outer-class-binding-mutated-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-private-name-in-class-decorator-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-private-name-in-class-decorator-exec.test.js:8
    @(A.#x) class B extends A {
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-private-name-in-class-decorator-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-this-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-this-exec.test.js:15
@o1.dec @dec @o2.dec @(o4.o().dec) class A {
^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc-this-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-accessor-initializers-fields-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-accessor-initializers-fields-exec.test.js:34
@logClassDecoratorRun(0, 19, 21) @logClassDecoratorRun(1, 18, 20) class A {
^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-accessor-initializers-fields-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-accessor-initializers-setters-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-accessor-initializers-setters-exec.test.js:42
@logClassDecoratorRun(0, 11, 13) @logClassDecoratorRun(1, 10, 12) class A {
^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-accessor-initializers-setters-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-accessor-method-initializers-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-accessor-method-initializers-exec.test.js:37
@logClassDecoratorRun(0, 19, 21) @logClassDecoratorRun(1, 18, 20) class A {
^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-accessor-method-initializers-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-accessor-static-accessor-initializers-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-accessor-static-accessor-initializers-exec.test.js:27
@logClassDecoratorRun(0, 19, 29) @logClassDecoratorRun(1, 18, 28) class A {
^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-accessor-static-accessor-initializers-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-accessor-static-method-initializers-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-accessor-static-method-initializers-exec.test.js:37
@logClassDecoratorRun(0, 19, 29) @logClassDecoratorRun(1, 18, 28) class A {
^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-accessor-static-method-initializers-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-class-decorators-without-element-decorators-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-class-decorators-without-element-decorators-exec.test.js:14
  @classDec1(log) @classDec2(log) class C {
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-class-decorators-without-element-decorators-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-decorators-evaluation-with-this-caching-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-decorators-evaluation-with-this-caching-exec.test.js:15
  @fn() @obj.prop.foo method() {}
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-decorators-evaluation-with-this-caching-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-field-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-field-exec.test.js:14
@logDecoratorRun(0, 21) @logDecoratorRun(1, 20) class A {
^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-field-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-field-initializers-after-methods-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-field-initializers-after-methods-exec.test.js:3
@(((x) => x)) class A {
^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-field-initializers-after-methods-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-initializers-and-static-blocks-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-initializers-and-static-blocks-exec.test.js:138
@classDec1 @classDec2 class Foo extends (log.push("extends"), Object) {
^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-initializers-and-static-blocks-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-initializers-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-initializers-exec.test.js:17
@logDecoratorRun(0, 35, 45) @logDecoratorRun(1, 34, 44) class A {
^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-initializers-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-method-initializers-field-value-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-method-initializers-field-value-exec.test.js:44
    @methodDec1 @methodDec2 method() {}
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-method-initializers-field-value-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-static-accessor-method-initializers-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-static-accessor-method-initializers-exec.test.js:37
@logClassDecoratorRun(0, 19, 29) @logClassDecoratorRun(1, 18, 28) class A {
^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-static-accessor-method-initializers-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-static-field-initializers-after-methods-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-static-field-initializers-after-methods-exec.test.js:3
@(((x) => x)) class A {
^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering--to-es2015-static-field-initializers-after-methods-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-accessor-init-and-set-consistent-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-accessor-init-and-set-consistent-exec.test.js:23
  @minusTwo @timesFour accessor bar = 5;
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-accessor-init-and-set-consistent-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-accessor-initializers-fields-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-accessor-initializers-fields-exec.test.js:34
@logClassDecoratorRun(0, 19, 21) @logClassDecoratorRun(1, 18, 20) class A {
^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-accessor-initializers-fields-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-accessor-initializers-setters-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-accessor-initializers-setters-exec.test.js:42
@logClassDecoratorRun(0, 11, 13) @logClassDecoratorRun(1, 10, 12) class A {
^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-accessor-initializers-setters-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-accessor-method-initializers-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-accessor-method-initializers-exec.test.js:37
@logClassDecoratorRun(0, 19, 21) @logClassDecoratorRun(1, 18, 20) class A {
^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-accessor-method-initializers-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-accessor-static-accessor-initializers-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-accessor-static-accessor-initializers-exec.test.js:27
@logClassDecoratorRun(0, 19, 29) @logClassDecoratorRun(1, 18, 28) class A {
^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-accessor-static-accessor-initializers-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-accessor-static-method-initializers-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-accessor-static-method-initializers-exec.test.js:37
@logClassDecoratorRun(0, 19, 29) @logClassDecoratorRun(1, 18, 28) class A {
^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-accessor-static-method-initializers-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-class-decorators-without-element-decorators-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-class-decorators-without-element-decorators-exec.test.js:14
  @classDec1(log) @classDec2(log) class C {
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-class-decorators-without-element-decorators-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-field-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-field-exec.test.js:14
@logDecoratorRun(0, 21) @logDecoratorRun(1, 20) class A {
^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-field-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-field-initializers-after-methods-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-field-initializers-after-methods-exec.test.js:3
@(((x) => x)) class A {
^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-field-initializers-after-methods-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-field-initializers-after-methods-private-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-field-initializers-after-methods-private-exec.test.js:4
  @(((_, { addInitializer }) => {
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-field-initializers-after-methods-private-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-initializers-and-static-blocks-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-initializers-and-static-blocks-exec.test.js:138
@classDec1 @classDec2 class Foo extends (log.push("extends"), Object) {
^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-initializers-and-static-blocks-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-initializers-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-initializers-exec.test.js:17
@logDecoratorRun(0, 35, 45) @logDecoratorRun(1, 34, 44) class A {
^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-initializers-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-method-initializers-field-value-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-method-initializers-field-value-exec.test.js:44
    @methodDec1 @methodDec2 method() {}
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-method-initializers-field-value-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-static-accessor-method-initializers-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-static-accessor-method-initializers-exec.test.js:37
@logClassDecoratorRun(0, 19, 29) @logClassDecoratorRun(1, 18, 28) class A {
^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-static-accessor-method-initializers-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-static-field-initializers-after-methods-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-static-field-initializers-after-methods-exec.test.js:3
@(((x) => x)) class A {
^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-ordering-static-field-initializers-after-methods-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-runtime-errors--to-es2015-invalid-accessor-decorator-return-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-runtime-errors--to-es2015-invalid-accessor-decorator-return-exec.test.js:15
  @returnsNull accessor a;
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-runtime-errors--to-es2015-invalid-accessor-decorator-return-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-runtime-errors--to-es2015-invalid-add-initializer-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-runtime-errors--to-es2015-invalid-add-initializer-exec.test.js:6
  @decWithInitializer(null) static m() {}
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-runtime-errors--to-es2015-invalid-add-initializer-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-runtime-errors--to-es2015-invalid-class-decorator-return-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-runtime-errors--to-es2015-invalid-class-decorator-return-exec.test.js:6
expect(() => @returnsNull class {}).toThrow("class decorators must return a function or undefined");
             ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-runtime-errors--to-es2015-invalid-class-decorator-return-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-runtime-errors--to-es2015-invalid-field-decorator-return-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-runtime-errors--to-es2015-invalid-field-decorator-return-exec.test.js:7
  @returnsNull m() {}
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-runtime-errors--to-es2015-invalid-field-decorator-return-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-runtime-errors--to-es2015-invalid-getter-decorator-return-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-runtime-errors--to-es2015-invalid-getter-decorator-return-exec.test.js:7
  @returnsNull get p() {}
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-runtime-errors--to-es2015-invalid-getter-decorator-return-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-runtime-errors--to-es2015-invalid-method-decorator-return-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-runtime-errors--to-es2015-invalid-method-decorator-return-exec.test.js:7
  @returnsNull p;
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-runtime-errors--to-es2015-invalid-method-decorator-return-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-runtime-errors--to-es2015-invalid-setter-decorator-return-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-runtime-errors--to-es2015-invalid-setter-decorator-return-exec.test.js:7
  @returnsNull set p(v) {}
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-runtime-errors--to-es2015-invalid-setter-decorator-return-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-setters--to-es2015-context-name-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-setters--to-es2015-context-name-exec.test.js:11
  @dec static set a(v) {}
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-setters--to-es2015-context-name-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-setters--to-es2015-private-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-setters--to-es2015-private-exec.test.js:13
  @dec set #a(v) {
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-setters--to-es2015-private-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-setters--to-es2015-public-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-setters--to-es2015-public-exec.test.js:13
  @dec set a(v) {
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-setters--to-es2015-public-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-setters--to-es2015-static-private-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-setters--to-es2015-static-private-exec.test.js:13
  @dec static set #a(v) {
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-setters--to-es2015-static-private-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-setters--to-es2015-static-public-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-setters--to-es2015-static-public-exec.test.js:13
  @dec static set a(v) {
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-setters--to-es2015-static-public-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-setters-context-name-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-setters-context-name-exec.test.js:11
  @dec static set a(v) {}
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-setters-context-name-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-typescript-computed-key-ts-as-expression-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-typescript-computed-key-ts-as-expression-exec.test.js:10
    @noopFactory(0) #p;
    ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-typescript-computed-key-ts-as-expression-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-typescript-member-expression-decorator-declare-field-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-typescript-member-expression-decorator-declare-field-exec.test.js:7
  @lib.dec(() => Example) target = 2;
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-typescript-member-expression-decorator-declare-field-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-typescript-member-expression-decorator-multiple-fields-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-typescript-member-expression-decorator-multiple-fields-exec.test.js:6
@lib.dec() class Example {
^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-typescript-member-expression-decorator-multiple-fields-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-metadata-class-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-metadata-class-exec.test.js:6
@dec class A {}
^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-metadata-class-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-metadata-element-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-metadata-element-exec.test.js:7
  @dec foo;
  ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-metadata-element-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-metadata-subclass-no-super-decorators-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-metadata-subclass-no-super-decorators-exec.test.js:9
@dec(3) class A extends B {}
^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-metadata-subclass-no-super-decorators-exec.test.js:10:15

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-metadata-subclass-super-decorators-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-metadata-subclass-super-decorators-exec.test.js:8
@dec(2) class B {}
^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-metadata-subclass-super-decorators-exec.test.js:10:15

./fixtures/babel/babel-plugin-transform-arrow-functions-test-fixtures-arrow-functions-implicit-var-arguments-exec.test.js
AssertionError: expected undefined to strictly equal 1
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-arrow-functions-test-fixtures-arrow-functions-implicit-var-arguments-exec.test.js:7:15
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-arrow-functions-test-fixtures-arrow-functions-implicit-var-arguments-exec.test.js:13:7

./fixtures/babel/babel-plugin-transform-async-to-generator-test-fixtures-async-to-generator-async-complex-params-exec.test.js
TypeError: Cannot destructure property 'b' of 'undefined' as it is undefined.
    at Object.foo (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-async-to-generator-test-fixtures-async-to-generator-async-complex-params-exec.test.js:5:12)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-async-to-generator-test-fixtures-async-to-generator-async-complex-params-exec.test.js:8:5
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-async-to-generator-test-fixtures-async-to-generator-async-complex-params-exec.test.js:15:7

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-nested-class-super-call-in-decorator-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-nested-class-super-call-in-decorator-exec.test.js:11
      @(super()) hello;
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-nested-class-super-call-in-decorator-exec.test.js:10:15

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-nested-class-super-property-in-accessor-key-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-nested-class-super-property-in-accessor-key-exec.test.js:12
      accessor [super.toString()] = "hello";
               ^

SyntaxError: Unexpected token '['
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-nested-class-super-property-in-accessor-key-exec.test.js:10:15

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-nested-class-super-property-in-decorator-exec.test.js
./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-nested-class-super-property-in-decorator-exec.test.js:12
      @(super.dec) hello;
      ^

SyntaxError: Invalid or unexpected token
    at new Script (node:vm:118:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-nested-class-super-property-in-decorator-exec.test.js:10:15

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-before-member-call-with-transform-exec.test.js
AssertionError: expected undefined to deeply equal 1
    at Foo.test (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-before-member-call-with-transform-exec.test.js:40:262)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-before-member-call-with-transform-exec.test.js:126:5
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-before-member-call-with-transform-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-before-property-with-transform-exec.test.js
AssertionError: expected undefined to deeply equal 1
    at Foo.test (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-before-property-with-transform-exec.test.js:39:260)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-before-property-with-transform-exec.test.js:119:5
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-before-property-with-transform-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-delete-property-with-transform-exec.test.js
AssertionError: expected function to throw an error, but it didn't
    at Foo.test (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-delete-property-with-transform-exec.test.js:67:8)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-delete-property-with-transform-exec.test.js:167:5
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-delete-property-with-transform-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-in-function-param-with-transform-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./npm/runtime/src/helpers/esm/classPrivateFieldLooseBase.js:2:44)
    at value (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-in-function-param-with-transform-exec.test.js:61:12)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-in-function-param-with-transform-exec.test.js:42:163
    at j (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-in-function-param-with-transform-exec.test.js:43:7)
    at Foo.test (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-in-function-param-with-transform-exec.test.js:50:12)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-in-function-param-with-transform-exec.test.js:69:5
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-in-function-param-with-transform-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-member-optional-call-with-transform-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./npm/runtime/src/helpers/esm/classPrivateFieldLooseBase.js:2:44)
    at value (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-member-optional-call-with-transform-exec.test.js:121:12)
    at Foo.test (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-member-optional-call-with-transform-exec.test.js:17:40)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-member-optional-call-with-transform-exec.test.js:129:5
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-member-optional-call-with-transform-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-optional-member-call-with-transform-exec.test.js
AssertionError: expected undefined to deeply equal 1
    at Foo.test (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-optional-member-call-with-transform-exec.test.js:39:253)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-optional-member-call-with-transform-exec.test.js:129:5
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-optional-member-call-with-transform-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-optional-property-with-transform-exec.test.js
AssertionError: expected undefined to deeply equal 1
    at Foo.test (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-optional-property-with-transform-exec.test.js:38:251)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-optional-property-with-transform-exec.test.js:122:5
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-optional-property-with-transform-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-parenthesized-optional-member-call-exec.test.js
TypeError: Cannot read properties of undefined (reading 'bind')
    at Foo.test (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-parenthesized-optional-member-call-exec.test.js:12:21)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-parenthesized-optional-member-call-exec.test.js:76:11
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-parenthesized-optional-member-call-exec.test.js:19:7

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-parenthesized-optional-member-call-with-transform-exec.test.js
TypeError: Cannot read properties of undefined (reading 'bind')
    at Foo.test (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-parenthesized-optional-member-call-with-transform-exec.test.js:12:21)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-parenthesized-optional-member-call-with-transform-exec.test.js:76:11
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-parenthesized-optional-member-call-with-transform-exec.test.js:19:7

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-delete-property-with-transform-exec.test.js
AssertionError: expected function to throw an error, but it didn't
    at Foo.test (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-delete-property-with-transform-exec.test.js:67:8)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-delete-property-with-transform-exec.test.js:167:5
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-delete-property-with-transform-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-public-computed-toPrimitive-exec.test.js
AssertionError: expected [Function] to throw error including '@@toPrimitive must return a primitive…' but got 'Cannot convert object to primitive va…'
    at Proxy.methodWrapper (./node_modules/chai/index.js:1737:25)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-public-computed-toPrimitive-exec.test.js:35:4
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-public-computed-toPrimitive-exec.test.js:15:7

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-public-delete-super-property-exec.test.js
AssertionError: expected function to throw an error, but it didn't
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-public-delete-super-property-exec.test.js:18:2
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-public-delete-super-property-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-class-static-block-test-fixtures-integration-assumption-privateFieldsAsProperties-setPublicClassFields-constantSuper-private-methods-access-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./npm/runtime/src/helpers/esm/classPrivateFieldLooseBase.js:2:44)
    at getFoo (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-static-block-test-fixtures-integration-assumption-privateFieldsAsProperties-setPublicClassFields-constantSuper-private-methods-access-exec.test.js:6:16)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-static-block-test-fixtures-integration-assumption-privateFieldsAsProperties-setPublicClassFields-constantSuper-private-methods-access-exec.test.js:7:2
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-static-block-test-fixtures-integration-assumption-privateFieldsAsProperties-setPublicClassFields-constantSuper-private-methods-access-exec.test.js:15:7

./fixtures/babel/babel-plugin-transform-object-rest-spread-test-fixtures-object-rest-for-x-assignment-shadowed-block-scoped-bindings-exec.test.js
ReferenceError: Cannot access 'a' before initialization
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-object-rest-spread-test-fixtures-object-rest-for-x-assignment-shadowed-block-scoped-bindings-exec.test.js:7:20
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-object-rest-spread-test-fixtures-object-rest-for-x-assignment-shadowed-block-scoped-bindings-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-object-rest-spread-test-fixtures-object-rest-for-x-declaration-shadowed-block-scoped-bindings-exec.test.js
ReferenceError: Cannot access 'a' before initialization
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-object-rest-spread-test-fixtures-object-rest-for-x-declaration-shadowed-block-scoped-bindings-exec.test.js:6:12
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-object-rest-spread-test-fixtures-object-rest-for-x-declaration-shadowed-block-scoped-bindings-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-object-rest-spread-test-fixtures-object-spread-assumption-ignoreFunctionLength-pureGetters-setSpreadProperties-expression-exec.test.js
Error: Transform error
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-object-rest-spread-test-fixtures-object-spread-assumption-ignoreFunctionLength-pureGetters-setSpreadProperties-expression-exec.test.js:2:1
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-object-rest-spread-test-fixtures-object-spread-assumption-ignoreFunctionLength-pureGetters-setSpreadProperties-expression-exec.test.js:13:7

./fixtures/babel/babel-plugin-transform-object-rest-spread-test-fixtures-object-spread-assumption-ignoreFunctionLength-pureGetters-setSpreadProperties-no-getOwnPropertyDescriptors-exec.test.js
Error: Transform error
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-object-rest-spread-test-fixtures-object-spread-assumption-ignoreFunctionLength-pureGetters-setSpreadProperties-no-getOwnPropertyDescriptors-exec.test.js:2:1
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-object-rest-spread-test-fixtures-object-spread-assumption-ignoreFunctionLength-pureGetters-setSpreadProperties-no-getOwnPropertyDescriptors-exec.test.js:13:7

./fixtures/babel/babel-plugin-transform-object-rest-spread-test-fixtures-object-spread-assumption-ignoreFunctionLength-pureGetters-setSpreadProperties-no-object-assign-exec-exec.test.js
Error: Transform error
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-object-rest-spread-test-fixtures-object-spread-assumption-ignoreFunctionLength-pureGetters-setSpreadProperties-no-object-assign-exec-exec.test.js:2:1
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-object-rest-spread-test-fixtures-object-spread-assumption-ignoreFunctionLength-pureGetters-setSpreadProperties-no-object-assign-exec-exec.test.js:13:7

./fixtures/babel/babel-plugin-transform-object-rest-spread-test-fixtures-object-spread-assumption-ignoreFunctionLength-pureGetters-setSpreadProperties-side-effect-exec.test.js
Error: Transform error
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-object-rest-spread-test-fixtures-object-spread-assumption-ignoreFunctionLength-pureGetters-setSpreadProperties-side-effect-exec.test.js:2:1
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-object-rest-spread-test-fixtures-object-spread-assumption-ignoreFunctionLength-pureGetters-setSpreadProperties-side-effect-exec.test.js:13:7

./fixtures/babel/babel-plugin-transform-object-rest-spread-test-fixtures-object-spread-assumption-ignoreFunctionLength-pureGetters-setSpreadProperties-string-exec-exec.test.js
Error: Transform error
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-object-rest-spread-test-fixtures-object-spread-assumption-ignoreFunctionLength-pureGetters-setSpreadProperties-string-exec-exec.test.js:2:1
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-object-rest-spread-test-fixtures-object-spread-assumption-ignoreFunctionLength-pureGetters-setSpreadProperties-string-exec-exec.test.js:13:7

./fixtures/babel/babel-plugin-transform-object-rest-spread-test-fixtures-object-spread-assumption-pureGetters-setSpreadProperties-builtins-expression-exec.test.js
Error: Transform error
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-object-rest-spread-test-fixtures-object-spread-assumption-pureGetters-setSpreadProperties-builtins-expression-exec.test.js:2:1
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-object-rest-spread-test-fixtures-object-spread-assumption-pureGetters-setSpreadProperties-builtins-expression-exec.test.js:13:7

./fixtures/babel/babel-plugin-transform-object-rest-spread-test-fixtures-object-spread-assumption-pureGetters-setSpreadProperties-builtins-no-getOwnPropertyDescriptors-exec.test.js
Error: Transform error
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-object-rest-spread-test-fixtures-object-spread-assumption-pureGetters-setSpreadProperties-builtins-no-getOwnPropertyDescriptors-exec.test.js:2:1
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-object-rest-spread-test-fixtures-object-spread-assumption-pureGetters-setSpreadProperties-builtins-no-getOwnPropertyDescriptors-exec.test.js:13:7

./fixtures/babel/babel-plugin-transform-object-rest-spread-test-fixtures-object-spread-assumption-pureGetters-setSpreadProperties-builtins-no-object-assign-exec-exec.test.js
Error: Transform error
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-object-rest-spread-test-fixtures-object-spread-assumption-pureGetters-setSpreadProperties-builtins-no-object-assign-exec-exec.test.js:2:1
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-object-rest-spread-test-fixtures-object-spread-assumption-pureGetters-setSpreadProperties-builtins-no-object-assign-exec-exec.test.js:13:7

./fixtures/babel/babel-plugin-transform-object-rest-spread-test-fixtures-object-spread-assumption-pureGetters-setSpreadProperties-builtins-side-effect-exec.test.js
Error: Transform error
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-object-rest-spread-test-fixtures-object-spread-assumption-pureGetters-setSpreadProperties-builtins-side-effect-exec.test.js:2:1
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-object-rest-spread-test-fixtures-object-spread-assumption-pureGetters-setSpreadProperties-builtins-side-effect-exec.test.js:13:7

./fixtures/babel/babel-plugin-transform-object-rest-spread-test-fixtures-object-spread-assumption-pureGetters-setSpreadProperties-builtins-string-exec-exec.test.js
Error: Transform error
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-object-rest-spread-test-fixtures-object-spread-assumption-pureGetters-setSpreadProperties-builtins-string-exec-exec.test.js:2:1
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-object-rest-spread-test-fixtures-object-spread-assumption-pureGetters-setSpreadProperties-builtins-string-exec-exec.test.js:13:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-assumption-privateFieldsAsProperties-basic-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-assumption-privateFieldsAsProperties-basic-exec.test.js:8:39)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-assumption-privateFieldsAsProperties-basic-exec.test.js:28:12
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-assumption-privateFieldsAsProperties-basic-exec.test.js:19:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-assumption-privateFieldsAsProperties-class-binding-exec.test.js
ReferenceError: _A_brand is not defined
    at new A (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-assumption-privateFieldsAsProperties-class-binding-exec.test.js:7:39)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-assumption-privateFieldsAsProperties-class-binding-exec.test.js:14:8
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-assumption-privateFieldsAsProperties-class-binding-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-assumption-privateFieldsAsProperties-get-only-setter-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-assumption-privateFieldsAsProperties-get-only-setter-exec.test.js:9:39)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-assumption-privateFieldsAsProperties-get-only-setter-exec.test.js:13:2
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-assumption-privateFieldsAsProperties-get-only-setter-exec.test.js:21:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-assumption-privateFieldsAsProperties-helper-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-assumption-privateFieldsAsProperties-helper-exec.test.js:7:39)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-assumption-privateFieldsAsProperties-helper-exec.test.js:11:1
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-assumption-privateFieldsAsProperties-helper-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-assumption-privateFieldsAsProperties-reassignment-exec.test.js
AssertionError: expected +0 to be 1 // Object.is equality
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-assumption-privateFieldsAsProperties-reassignment-exec.test.js:10:7
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-assumption-privateFieldsAsProperties-reassignment-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-assumption-privateFieldsAsProperties-set-only-getter-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-assumption-privateFieldsAsProperties-set-only-getter-exec.test.js:9:39)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-assumption-privateFieldsAsProperties-set-only-getter-exec.test.js:14:7
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-assumption-privateFieldsAsProperties-set-only-getter-exec.test.js:21:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-assumption-privateFieldsAsProperties-updates-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-assumption-privateFieldsAsProperties-updates-exec.test.js:8:39)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-assumption-privateFieldsAsProperties-updates-exec.test.js:47:12
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-assumption-privateFieldsAsProperties-updates-exec.test.js:19:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-basic-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-basic-exec.test.js:8:39)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-basic-exec.test.js:28:12
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-basic-exec.test.js:19:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-class-binding-exec.test.js
ReferenceError: _A_brand is not defined
    at new A (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-class-binding-exec.test.js:7:39)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-class-binding-exec.test.js:14:8
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-class-binding-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-get-only-setter-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-get-only-setter-exec.test.js:9:39)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-get-only-setter-exec.test.js:13:2
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-get-only-setter-exec.test.js:21:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-helper-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-helper-exec.test.js:7:39)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-helper-exec.test.js:11:1
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-helper-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-set-only-getter-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-set-only-getter-exec.test.js:9:39)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-set-only-getter-exec.test.js:14:7
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-set-only-getter-exec.test.js:21:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-updates-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-updates-exec.test.js:8:39)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-updates-exec.test.js:47:12
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-updates-exec.test.js:19:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsSymbols-get-only-setter-exec.test.js
TypeError: "#privateFieldValue" is write-only
    at _writeOnlyError (./npm/runtime/src/helpers/esm/writeOnlyError.js:2:9)
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsSymbols-get-only-setter-exec.test.js:12:19)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsSymbols-get-only-setter-exec.test.js:11:9
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsSymbols-get-only-setter-exec.test.js:21:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-assumption-privateFieldsAsProperties-setPublicClassFields-constantSuper-assignment-exec.test.js
ReferenceError: _Foo_brand is not defined
    at new Foo (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-assumption-privateFieldsAsProperties-setPublicClassFields-constantSuper-assignment-exec.test.js:6:39)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-assumption-privateFieldsAsProperties-setPublicClassFields-constantSuper-assignment-exec.test.js:8:8
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-assumption-privateFieldsAsProperties-setPublicClassFields-constantSuper-assignment-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-assumption-privateFieldsAsProperties-setPublicClassFields-constantSuper-async-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-assumption-privateFieldsAsProperties-setPublicClassFields-constantSuper-async-exec.test.js:6:39)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-assumption-privateFieldsAsProperties-setPublicClassFields-constantSuper-async-exec.test.js:10:7
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-assumption-privateFieldsAsProperties-setPublicClassFields-constantSuper-async-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-assumption-privateFieldsAsProperties-setPublicClassFields-constantSuper-before-fields-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-assumption-privateFieldsAsProperties-setPublicClassFields-constantSuper-before-fields-exec.test.js:8:39)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-assumption-privateFieldsAsProperties-setPublicClassFields-constantSuper-before-fields-exec.test.js:16:8
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-assumption-privateFieldsAsProperties-setPublicClassFields-constantSuper-before-fields-exec.test.js:19:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-assumption-privateFieldsAsProperties-setPublicClassFields-constantSuper-class-binding-exec.test.js
ReferenceError: _A_brand is not defined
    at new A (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-assumption-privateFieldsAsProperties-setPublicClassFields-constantSuper-class-binding-exec.test.js:7:39)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-assumption-privateFieldsAsProperties-setPublicClassFields-constantSuper-class-binding-exec.test.js:14:8
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-assumption-privateFieldsAsProperties-setPublicClassFields-constantSuper-class-binding-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-assumption-privateFieldsAsProperties-setPublicClassFields-constantSuper-context-exec.test.js
ReferenceError: _Foo_brand is not defined
    at new Foo (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-assumption-privateFieldsAsProperties-setPublicClassFields-constantSuper-context-exec.test.js:6:39)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-assumption-privateFieldsAsProperties-setPublicClassFields-constantSuper-context-exec.test.js:32:11
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-assumption-privateFieldsAsProperties-setPublicClassFields-constantSuper-context-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-assumption-privateFieldsAsProperties-setPublicClassFields-constantSuper-exfiltrated-exec.test.js
ReferenceError: _Foo_brand is not defined
    at new Foo (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-assumption-privateFieldsAsProperties-setPublicClassFields-constantSuper-exfiltrated-exec.test.js:7:39)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-assumption-privateFieldsAsProperties-setPublicClassFields-constantSuper-exfiltrated-exec.test.js:10:1
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-assumption-privateFieldsAsProperties-setPublicClassFields-constantSuper-exfiltrated-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-assumption-privateFieldsAsProperties-setPublicClassFields-constantSuper-generator-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-assumption-privateFieldsAsProperties-setPublicClassFields-constantSuper-generator-exec.test.js:6:39)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-assumption-privateFieldsAsProperties-setPublicClassFields-constantSuper-generator-exec.test.js:11:7
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-assumption-privateFieldsAsProperties-setPublicClassFields-constantSuper-generator-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-assumption-privateFieldsAsProperties-setPublicClassFields-constantSuper-reassignment-exec.test.js
AssertionError: expected +0 to be 1 // Object.is equality
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-assumption-privateFieldsAsProperties-setPublicClassFields-constantSuper-reassignment-exec.test.js:10:7
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-assumption-privateFieldsAsProperties-setPublicClassFields-constantSuper-reassignment-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-assumption-privateFieldsAsProperties-setPublicClassFields-constantSuper-super-exec.test.js
ReferenceError: _Sub_brand is not defined
    at new Sub (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-assumption-privateFieldsAsProperties-setPublicClassFields-constantSuper-super-exec.test.js:8:33)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-assumption-privateFieldsAsProperties-setPublicClassFields-constantSuper-super-exec.test.js:27:8
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-assumption-privateFieldsAsProperties-setPublicClassFields-constantSuper-super-exec.test.js:19:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-assignment-exec.test.js
ReferenceError: _Foo_brand is not defined
    at new Foo (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-assignment-exec.test.js:6:39)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-assignment-exec.test.js:8:8
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-assignment-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-async-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-async-exec.test.js:6:39)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-async-exec.test.js:10:7
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-async-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-before-fields-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-before-fields-exec.test.js:9:39)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-before-fields-exec.test.js:16:7
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-before-fields-exec.test.js:21:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-class-binding-exec.test.js
ReferenceError: _A_brand is not defined
    at new A (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-class-binding-exec.test.js:7:39)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-class-binding-exec.test.js:14:8
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-class-binding-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-context-exec.test.js
ReferenceError: _Foo_brand is not defined
    at new Foo (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-context-exec.test.js:6:39)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-context-exec.test.js:32:11
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-context-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-exfiltrated-exec.test.js
ReferenceError: _Foo_brand is not defined
    at new Foo (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-exfiltrated-exec.test.js:7:39)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-exfiltrated-exec.test.js:10:1
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-exfiltrated-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-generator-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-generator-exec.test.js:6:39)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-generator-exec.test.js:11:7
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-generator-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-super-exec.test.js
ReferenceError: _Sub_brand is not defined
    at new Sub (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-super-exec.test.js:8:33)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-super-exec.test.js:27:8
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-super-exec.test.js:19:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsSymbols-async-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsSymbols-async-exec.test.js:6:39)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsSymbols-async-exec.test.js:10:7
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsSymbols-async-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-assumption-privateFieldsAsProperties-constantSuper-basic-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./npm/runtime/src/helpers/esm/classPrivateFieldLooseBase.js:2:44)
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-assumption-privateFieldsAsProperties-constantSuper-basic-exec.test.js:14:24)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-assumption-privateFieldsAsProperties-constantSuper-basic-exec.test.js:24:8
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-assumption-privateFieldsAsProperties-constantSuper-basic-exec.test.js:15:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-assumption-privateFieldsAsProperties-constantSuper-class-check-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./npm/runtime/src/helpers/esm/classPrivateFieldLooseBase.js:2:44)
    at Cl.publicMethod (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-assumption-privateFieldsAsProperties-constantSuper-class-check-exec.test.js:5:12)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-assumption-privateFieldsAsProperties-constantSuper-class-check-exec.test.js:8:7
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-assumption-privateFieldsAsProperties-constantSuper-class-check-exec.test.js:15:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-assumption-privateFieldsAsProperties-constantSuper-exfiltrated-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./npm/runtime/src/helpers/esm/classPrivateFieldLooseBase.js:2:44)
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-assumption-privateFieldsAsProperties-constantSuper-exfiltrated-exec.test.js:7:21)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-assumption-privateFieldsAsProperties-constantSuper-exfiltrated-exec.test.js:11:1
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-assumption-privateFieldsAsProperties-constantSuper-exfiltrated-exec.test.js:15:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-assumption-privateFieldsAsProperties-constantSuper-generator-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./npm/runtime/src/helpers/esm/classPrivateFieldLooseBase.js:2:44)
    at Cl.test (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-assumption-privateFieldsAsProperties-constantSuper-generator-exec.test.js:5:12)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-assumption-privateFieldsAsProperties-constantSuper-generator-exec.test.js:8:21
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-assumption-privateFieldsAsProperties-constantSuper-generator-exec.test.js:15:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-assumption-privateFieldsAsProperties-constantSuper-scopable-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./npm/runtime/src/helpers/esm/classPrivateFieldLooseBase.js:2:44)
    at Cl.publicMethod (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-assumption-privateFieldsAsProperties-constantSuper-scopable-exec.test.js:5:12)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-assumption-privateFieldsAsProperties-constantSuper-scopable-exec.test.js:12:15
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-assumption-privateFieldsAsProperties-constantSuper-scopable-exec.test.js:15:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-assumption-privateFieldsAsProperties-constantSuper-super-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./npm/runtime/src/helpers/esm/classPrivateFieldLooseBase.js:2:44)
    at Sub.check (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-assumption-privateFieldsAsProperties-constantSuper-super-exec.test.js:10:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-assumption-privateFieldsAsProperties-constantSuper-super-exec.test.js:17:12
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-assumption-privateFieldsAsProperties-constantSuper-super-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-assumption-privateFieldsAsProperties-constantSuper-this-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./npm/runtime/src/helpers/esm/classPrivateFieldLooseBase.js:2:44)
    at B.extract (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-assumption-privateFieldsAsProperties-constantSuper-this-exec.test.js:10:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-assumption-privateFieldsAsProperties-constantSuper-this-exec.test.js:25:24
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-assumption-privateFieldsAsProperties-constantSuper-this-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-basic-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./npm/runtime/src/helpers/esm/classPrivateFieldLooseBase.js:2:44)
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-basic-exec.test.js:14:24)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-basic-exec.test.js:24:8
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-basic-exec.test.js:15:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-class-check-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./npm/runtime/src/helpers/esm/classPrivateFieldLooseBase.js:2:44)
    at Cl.publicMethod (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-class-check-exec.test.js:5:12)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-class-check-exec.test.js:8:7
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-class-check-exec.test.js:15:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-exfiltrated-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./npm/runtime/src/helpers/esm/classPrivateFieldLooseBase.js:2:44)
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-exfiltrated-exec.test.js:7:21)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-exfiltrated-exec.test.js:11:1
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-exfiltrated-exec.test.js:15:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-generator-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./npm/runtime/src/helpers/esm/classPrivateFieldLooseBase.js:2:44)
    at Cl.test (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-generator-exec.test.js:5:12)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-generator-exec.test.js:8:21
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-generator-exec.test.js:15:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-scopable-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./npm/runtime/src/helpers/esm/classPrivateFieldLooseBase.js:2:44)
    at Cl.publicMethod (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-scopable-exec.test.js:5:12)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-scopable-exec.test.js:12:15
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-scopable-exec.test.js:15:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-super-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./npm/runtime/src/helpers/esm/classPrivateFieldLooseBase.js:2:44)
    at Sub.check (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-super-exec.test.js:10:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-super-exec.test.js:17:12
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-super-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-this-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./npm/runtime/src/helpers/esm/classPrivateFieldLooseBase.js:2:44)
    at B.extract (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-this-exec.test.js:10:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-this-exec.test.js:25:24
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-this-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-assumption-privateFieldsAsProperties-setPublicClassFields-access-in-static-field-initializer-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./npm/runtime/src/helpers/esm/classPrivateFieldLooseBase.js:2:44)
    at new C (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-assumption-privateFieldsAsProperties-setPublicClassFields-access-in-static-field-initializer-exec.test.js:10:11)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-assumption-privateFieldsAsProperties-setPublicClassFields-access-in-static-field-initializer-exec.test.js:14:12
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-assumption-privateFieldsAsProperties-setPublicClassFields-access-in-static-field-initializer-exec.test.js:17:3
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-assumption-privateFieldsAsProperties-setPublicClassFields-access-in-static-field-initializer-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-assumption-privateFieldsAsProperties-setPublicClassFields-basic-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./npm/runtime/src/helpers/esm/classPrivateFieldLooseBase.js:2:44)
    at Cl.getValue (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-assumption-privateFieldsAsProperties-setPublicClassFields-basic-exec.test.js:8:12)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-assumption-privateFieldsAsProperties-setPublicClassFields-basic-exec.test.js:25:11
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-assumption-privateFieldsAsProperties-setPublicClassFields-basic-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-assumption-privateFieldsAsProperties-setPublicClassFields-destructure-set-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./npm/runtime/src/helpers/esm/classPrivateFieldLooseBase.js:2:44)
    at new C (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-assumption-privateFieldsAsProperties-setPublicClassFields-destructure-set-exec.test.js:8:6)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-assumption-privateFieldsAsProperties-setPublicClassFields-destructure-set-exec.test.js:15:1
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-assumption-privateFieldsAsProperties-setPublicClassFields-destructure-set-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-assumption-privateFieldsAsProperties-setPublicClassFields-get-only-setter-exec.test.js
TypeError: "#privateStaticFieldValue" is write-only
    at _writeOnlyError (./npm/runtime/src/helpers/esm/writeOnlyError.js:2:9)
    at Cl.getPrivateStaticFieldValue (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-assumption-privateFieldsAsProperties-setPublicClassFields-get-only-setter-exec.test.js:9:16)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-assumption-privateFieldsAsProperties-setPublicClassFields-get-only-setter-exec.test.js:14:11
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-assumption-privateFieldsAsProperties-setPublicClassFields-get-only-setter-exec.test.js:19:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-assumption-privateFieldsAsProperties-setPublicClassFields-updates-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./npm/runtime/src/helpers/esm/classPrivateFieldLooseBase.js:2:44)
    at Cl.testUpdates (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-assumption-privateFieldsAsProperties-setPublicClassFields-updates-exec.test.js:17:3)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-assumption-privateFieldsAsProperties-setPublicClassFields-updates-exec.test.js:45:4
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-assumption-privateFieldsAsProperties-setPublicClassFields-updates-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-get-only-setter-exec.test.js
TypeError: "#privateStaticFieldValue" is write-only
    at _writeOnlyError (./npm/runtime/src/helpers/esm/writeOnlyError.js:2:9)
    at Cl.getPrivateStaticFieldValue (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-get-only-setter-exec.test.js:5:16)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-get-only-setter-exec.test.js:8:7
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-get-only-setter-exec.test.js:15:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-access-in-static-field-initializer-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./npm/runtime/src/helpers/esm/classPrivateFieldLooseBase.js:2:44)
    at new C (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-access-in-static-field-initializer-exec.test.js:10:11)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-access-in-static-field-initializer-exec.test.js:14:12
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-access-in-static-field-initializer-exec.test.js:17:3
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-access-in-static-field-initializer-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-basic-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./npm/runtime/src/helpers/esm/classPrivateFieldLooseBase.js:2:44)
    at Cl.getValue (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-basic-exec.test.js:8:12)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-basic-exec.test.js:25:11
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-basic-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-destructure-set-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./npm/runtime/src/helpers/esm/classPrivateFieldLooseBase.js:2:44)
    at new C (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-destructure-set-exec.test.js:8:6)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-destructure-set-exec.test.js:15:1
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-destructure-set-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-get-only-setter-exec.test.js
TypeError: "#privateStaticFieldValue" is write-only
    at _writeOnlyError (./npm/runtime/src/helpers/esm/writeOnlyError.js:2:9)
    at Cl.getPrivateStaticFieldValue (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-get-only-setter-exec.test.js:9:16)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-get-only-setter-exec.test.js:14:11
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-get-only-setter-exec.test.js:19:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-updates-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./npm/runtime/src/helpers/esm/classPrivateFieldLooseBase.js:2:44)
    at Cl.testUpdates (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-updates-exec.test.js:17:5)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-updates-exec.test.js:46:4
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-updates-exec.test.js:19:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsSymbols-get-only-setter-exec.test.js
TypeError: "#privateStaticFieldValue" is write-only
    at _writeOnlyError (./npm/runtime/src/helpers/esm/writeOnlyError.js:2:9)
    at Cl.getPrivateStaticFieldValue (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsSymbols-get-only-setter-exec.test.js:5:16)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsSymbols-get-only-setter-exec.test.js:8:7
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsSymbols-get-only-setter-exec.test.js:15:7

./fixtures/babel/babel-plugin-transform-private-property-in-object-test-fixtures-assumption-privateFieldsAsProperties-method-exec.test.js
ReferenceError: _Foo_brand is not defined
    at new Foo (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-property-in-object-test-fixtures-assumption-privateFieldsAsProperties-method-exec.test.js:6:39)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-property-in-object-test-fixtures-assumption-privateFieldsAsProperties-method-exec.test.js:12:12
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-property-in-object-test-fixtures-assumption-privateFieldsAsProperties-method-exec.test.js:17:7

./fixtures/babel/babel-preset-env-test-fixtures-sanity-check-es2015-constants-exec.test.js
TypeError: Assignment to constant variable.
    at ./tasks/transform_conformance/fixtures/babel/babel-preset-env-test-fixtures-sanity-check-es2015-constants-exec.test.js:3:1
    at ./tasks/transform_conformance/fixtures/babel/babel-preset-env-test-fixtures-sanity-check-es2015-constants-exec.test.js:13:7
