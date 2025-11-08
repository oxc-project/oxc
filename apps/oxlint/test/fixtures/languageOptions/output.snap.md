# Exit code
1

# stdout
```
  x language-options-plugin(lang): languageOptions:
  | sourceType: script
  | ecmaVersion: 2026
  | parserOptions: {"sourceType":"script"}
  | globals: null
   ,-[files/index.cjs:1:1]
 1 | let x;
   : ^
   `----

  x language-options-plugin(lang): languageOptions:
  | sourceType: module
  | ecmaVersion: 2026
  | parserOptions: {"sourceType":"module"}
  | globals: null
   ,-[files/index.js:1:1]
 1 | let x;
   : ^
   `----

  x language-options-plugin(lang): languageOptions:
  | sourceType: module
  | ecmaVersion: 2026
  | parserOptions: {"sourceType":"module"}
  | globals: null
   ,-[files/index.mjs:1:1]
 1 | let x;
   : ^
   `----

Found 0 warnings and 3 errors.
Finished in Xms on 3 files using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
