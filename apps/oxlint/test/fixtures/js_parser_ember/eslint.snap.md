# Exit code
1

# stdout
```
<fixture>/files/late-declaration.gjs
  7:10  error  update-able variables are not supported in templates, reference a const variable    ember/template-no-let-reference

<fixture>/files/let-reference.gjs
  5:8  error  update-able variables are not supported in templates, reference a const variable    ember/template-no-let-reference

<fixture>/files/native-rules.gts
  2:7   error  'unusedLocal' is assigned a value but never used  no-unused-vars
  8:5   error  Unexpected 'debugger' statement                   no-debugger
  9:18  error  Expected '===' and instead saw '=='               eqeqeq

<fixture>/files/typescript.gts
  4:11  error  update-able variables are not supported in templates, reference a const variable    ember/template-no-let-reference

✖ 6 problems (6 errors, 0 warnings)
```

# stderr
```
```
