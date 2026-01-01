# Exit code
1

# stdout
```
  x @scope(rule): id: @scope/rule
   ,-[files/index.js:4:1]
 3 |  */
 4 | function f(foo, bar) {}
   : ^^^^^^^^^^^^^^^^^^^^^^^
   `----

  x @scope2(rule): id: @scope2/rule
   ,-[files/index.js:4:1]
 3 |  */
 4 | function f(foo, bar) {}
   : ^^^^^^^^^^^^^^^^^^^^^^^
   `----

  x @scope3/subplugin(rule): id: @scope3/subplugin/rule
   ,-[files/index.js:4:1]
 3 |  */
 4 | function f(foo, bar) {}
   : ^^^^^^^^^^^^^^^^^^^^^^^
   `----

  x js-jsdoc(rule): id: js-jsdoc/rule
   ,-[files/index.js:4:1]
 3 |  */
 4 | function f(foo, bar) {}
   : ^^^^^^^^^^^^^^^^^^^^^^^
   `----

  x js-jsdoc2(rule): id: js-jsdoc2/rule
   ,-[files/index.js:4:1]
 3 |  */
 4 | function f(foo, bar) {}
   : ^^^^^^^^^^^^^^^^^^^^^^^
   `----

  x no-name-alias(rule): id: no-name-alias/rule
   ,-[files/index.js:4:1]
 3 |  */
 4 | function f(foo, bar) {}
   : ^^^^^^^^^^^^^^^^^^^^^^^
   `----

  x no-name-alias2(rule): id: no-name-alias2/rule
   ,-[files/index.js:4:1]
 3 |  */
 4 | function f(foo, bar) {}
   : ^^^^^^^^^^^^^^^^^^^^^^^
   `----

  x plugin1-name-from-rule(rule): id: plugin1-name-from-rule/rule
   ,-[files/index.js:4:1]
 3 |  */
 4 | function f(foo, bar) {}
   : ^^^^^^^^^^^^^^^^^^^^^^^
   `----

  x plugin2-name-from-package(rule): id: plugin2-name-from-package/rule
   ,-[files/index.js:4:1]
 3 |  */
 4 | function f(foo, bar) {}
   : ^^^^^^^^^^^^^^^^^^^^^^^
   `----

  x plugin3-name-from-package(rule): id: plugin3-name-from-package/rule
   ,-[files/index.js:4:1]
 3 |  */
 4 | function f(foo, bar) {}
   : ^^^^^^^^^^^^^^^^^^^^^^^
   `----

  x plugin4-name-from-rule(rule): id: plugin4-name-from-rule/rule
   ,-[files/index.js:4:1]
 3 |  */
 4 | function f(foo, bar) {}
   : ^^^^^^^^^^^^^^^^^^^^^^^
   `----

  x ]8;;https://oxc.rs/docs/guide/usage/linter/rules/jsdoc/require-param.html\eslint-plugin-jsdoc(require-param)]8;;\: Missing JSDoc `@param` declaration for function parameters.
   ,-[files/index.js:4:17]
 3 |  */
 4 | function f(foo, bar) {}
   :                 ^^^
   `----
  help: Add `@param` tag with name.

Found 0 warnings and 12 errors.
Finished in Xms on 1 file with 12 rules using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
