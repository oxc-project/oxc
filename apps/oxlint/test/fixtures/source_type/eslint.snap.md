# Exit code
1

# stdout
```
<fixture>/files/with_esm.js
  1:1  error  Program.sourceType: module
languageOptions.sourceType: module
scopes: global, module, function                            source-type-plugin/check
  3:1  error  getScope(node): function, isStrict: true
getScope(proxy): module, upper: global                      source-type-plugin/check

<fixture>/files/without_esm.cjs
  1:1  error  Program.sourceType: commonjs
languageOptions.sourceType: commonjs
scopes: global, function, function                          source-type-plugin/check
  1:1  error  getScope(node): function, isStrict: false
getScope(proxy): function, upper: global                    source-type-plugin/check

<fixture>/files/without_esm.js
  1:1  error  Program.sourceType: module
languageOptions.sourceType: module
scopes: global, module, function                            source-type-plugin/check
  1:1  error  getScope(node): function, isStrict: true
getScope(proxy): module, upper: global                      source-type-plugin/check

✖ 6 problems (6 errors, 0 warnings)
```

# stderr
```
```
