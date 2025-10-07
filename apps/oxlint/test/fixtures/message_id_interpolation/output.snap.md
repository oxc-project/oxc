# Exit code
1

# stdout
```
  x interpolation-test(no-var): Variables should not use var
   ,-[files/index.js:1:1]
 1 | var testWithNoData = {};
   : ^^^^^^^^^^^^^^^^^^^^^^^^
 2 | var testWithName = {};
   `----

  x interpolation-test(no-var): Variable `testWithName` should not use var
   ,-[files/index.js:2:1]
 1 | var testWithNoData = {};
 2 | var testWithName = {};
   : ^^^^^^^^^^^^^^^^^^^^^^
 3 | var testWithNameNoData = {};
   `----

  x interpolation-test(no-var): Variable `{{name}}` should not use var
   ,-[files/index.js:3:1]
 2 | var testWithName = {};
 3 | var testWithNameNoData = {};
   : ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
 4 | var testWithMultiple = {};
   `----

  x interpolation-test(no-var): Variable `testWithMultiple` of type `string` should not use var
   ,-[files/index.js:4:1]
 3 | var testWithNameNoData = {};
 4 | var testWithMultiple = {};
   : ^^^^^^^^^^^^^^^^^^^^^^^^^^
 5 | var testWithMultipleNoData = {};
   `----

  x interpolation-test(no-var): Variable `{{name}}` of type `{{type}}` should not use var
   ,-[files/index.js:5:1]
 4 | var testWithMultiple = {};
 5 | var testWithMultipleNoData = {};
   : ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
 6 | var testWithMissingData = {};
   `----

  x interpolation-test(no-var): Value is `example` and name is `{{name}}`
   ,-[files/index.js:6:1]
 5 | var testWithMultipleNoData = {};
 6 | var testWithMissingData = {};
   : ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
 7 | var testWithSpaces = {};
   `----

  x interpolation-test(no-var): Value with spaces is `hello` and name is `world`
   ,-[files/index.js:7:1]
 6 | var testWithMissingData = {};
 7 | var testWithSpaces = {};
   : ^^^^^^^^^^^^^^^^^^^^^^^^
   `----

Found 0 warnings and 7 errors.
Finished in Xms on 1 file using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
