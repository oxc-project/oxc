# Exit code
0

# stdout
```
  ! eslint(no-var): Unexpected var, use let or const instead.
   ,-[files/test.custom:2:1]
 1 | # Test file for custom parser fix support
 2 | var foo
   : ^^^
 3 | var bar
   `----
  help: Replace var with let or const

  ! eslint(no-var): Unexpected var, use let or const instead.
   ,-[files/test.custom:3:1]
 2 | var foo
 3 | var bar
   : ^^^
 4 | log foo
   `----
  help: Replace var with let or const

Found 2 warnings and 0 errors.
Finished in Xms on 1 file using X threads.
```

# stderr
```
WARNING: JS parsers are experimental and not subject to semver.
Breaking changes are possible while JS parsers support is under development.
```
