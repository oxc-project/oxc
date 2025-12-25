# Exit code
1

# stdout
```
  ! eslint(no-var): Unexpected var, use let or const instead.
   ,-[files/test1.custom:2:1]
 1 | # Test file for custom parser
 2 | var foo
   : ^^^
 3 | var bar
   `----
  help: Replace var with let or const

  ! eslint(no-var): Unexpected var, use let or const instead.
   ,-[files/test1.custom:3:1]
 2 | var foo
 3 | var bar
   : ^^^
 4 | log foo
   `----
  help: Replace var with let or const

  x custom-parser-plugin(count-identifiers): Starting to lint: <fixture>/files/test1.custom
   ,-[files/test1.custom:1:1]
 1 | # Test file for custom parser
   : ^
 2 | var foo
   `----

  x custom-parser-plugin(no-foo-var): Variable "foo" is not allowed in custom DSL files
   ,-[files/test1.custom:2:5]
 1 | # Test file for custom parser
 2 | var foo
   :     ^^^
 3 | var bar
   `----

  x custom-parser-plugin(count-identifiers): Found identifier: foo
   ,-[files/test1.custom:2:5]
 1 | # Test file for custom parser
 2 | var foo
   :     ^^^
 3 | var bar
   `----

  x custom-parser-plugin(count-identifiers): Found identifier: bar
   ,-[files/test1.custom:3:5]
 2 | var foo
 3 | var bar
   :     ^^^
 4 | log foo
   `----

  x custom-parser-plugin(count-identifiers): Found identifier: console.log
   ,-[files/test1.custom:4:1]
 3 | var bar
 4 | log foo
   : ^^^
 5 | log bar
   `----

  x custom-parser-plugin(count-identifiers): Found identifier: foo
   ,-[files/test1.custom:4:5]
 3 | var bar
 4 | log foo
   :     ^^^
 5 | log bar
   `----

  x custom-parser-plugin(count-identifiers): Found identifier: console.log
   ,-[files/test1.custom:5:1]
 4 | log foo
 5 | log bar
   : ^^^
   `----

  x custom-parser-plugin(count-identifiers): Found identifier: bar
   ,-[files/test1.custom:5:5]
 4 | log foo
 5 | log bar
   :     ^^^
   `----

  x custom-parser-plugin(count-identifiers): Total identifiers found: 6
   ,-[files/test1.custom:1:1]
 1 | # Test file for custom parser
   : ^
 2 | var foo
   `----

  ! eslint(no-var): Unexpected var, use let or const instead.
   ,-[files/test2.custom:2:1]
 1 | # Another test file
 2 | var x
   : ^^^
 3 | log x
   `----
  help: Replace var with let or const

  x custom-parser-plugin(count-identifiers): Starting to lint: <fixture>/files/test2.custom
   ,-[files/test2.custom:1:1]
 1 | # Another test file
   : ^
 2 | var x
   `----

  x custom-parser-plugin(count-identifiers): Found identifier: x
   ,-[files/test2.custom:2:5]
 1 | # Another test file
 2 | var x
   :     ^
 3 | log x
   `----

  x custom-parser-plugin(count-identifiers): Found identifier: console.log
   ,-[files/test2.custom:3:1]
 2 | var x
 3 | log x
   : ^^^
   `----

  x custom-parser-plugin(count-identifiers): Found identifier: x
   ,-[files/test2.custom:3:5]
 2 | var x
 3 | log x
   :     ^
   `----

  x custom-parser-plugin(count-identifiers): Total identifiers found: 3
   ,-[files/test2.custom:1:1]
 1 | # Another test file
   : ^
 2 | var x
   `----

Found 3 warnings and 14 errors.
Finished in Xms on 2 files using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
WARNING: JS parsers are experimental and not subject to semver.
Breaking changes are possible while JS parsers support is under development.
```
