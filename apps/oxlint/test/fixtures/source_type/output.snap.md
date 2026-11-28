# Exit code
1

# stdout
```
  x source-type-plugin(check): getScope(node): function, isStrict: true
  | getScope(proxy): module, upper: global
   ,-[files/component.astro:2:1]
 1 | ---
 2 | function frontmatter() {}
   : ^^^^^^^^^^^^^^^^^^^^^^^^^
 3 | ---
   `----

  x source-type-plugin(check): Program.sourceType: module
  | languageOptions.sourceType: module
  | scopes: global, module, function
   ,-[files/component.astro:2:1]
 1 | ---
 2 | function frontmatter() {}
   : ^^^^^^^^^^^^^^^^^^^^^^^^^^
 3 | ---
 4 | 
   `----

  x source-type-plugin(check): Program.sourceType: module
  | languageOptions.sourceType: module
  | scopes: global, module, function
    ,-[files/component.astro:8:1]
  7 | <script>
  8 |   function client() {}
    : ^^^^^^^^^^^^^^^^^^^^^^^
  9 | </script>
 10 | 
    `----

  x source-type-plugin(check): getScope(node): function, isStrict: true
  | getScope(proxy): module, upper: global
   ,-[files/component.astro:8:3]
 7 | <script>
 8 |   function client() {}
   :   ^^^^^^^^^^^^^^^^^^^^
 9 | </script>
   `----

  x source-type-plugin(check): Program.sourceType: module
  | languageOptions.sourceType: module
  | scopes: global, module, function
    ,-[files/component.astro:12:1]
 11 | <script is:inline>
 12 |   function inline() {}
    : ^^^^^^^^^^^^^^^^^^^^^^^
 13 | </script>
    `----

  x source-type-plugin(check): getScope(node): function, isStrict: true
  | getScope(proxy): module, upper: global
    ,-[files/component.astro:12:3]
 11 | <script is:inline>
 12 |   function inline() {}
    :   ^^^^^^^^^^^^^^^^^^^^
 13 | </script>
    `----

  x source-type-plugin(check): Program.sourceType: module
  | languageOptions.sourceType: module
  | scopes: global, module, function
   ,-[files/with_esm.js:1:1]
 1 | ,-> export {};
 2 | |   
 3 | `-> function foo() {}
   `----

  x source-type-plugin(check): getScope(node): function, isStrict: true
  | getScope(proxy): module, upper: global
   ,-[files/with_esm.js:3:1]
 2 | 
 3 | function foo() {}
   : ^^^^^^^^^^^^^^^^^
   `----

  x source-type-plugin(check): getScope(node): function, isStrict: false
  | getScope(proxy): function, upper: global
   ,-[files/without_esm.cjs:1:1]
 1 | function foo() {}
   : ^^^^^^^^^^^^^^^^^
   `----

  x source-type-plugin(check): Program.sourceType: commonjs
  | languageOptions.sourceType: commonjs
  | scopes: global, function, function
   ,-[files/without_esm.cjs:1:1]
 1 | function foo() {}
   : ^^^^^^^^^^^^^^^^^^
   `----

  x source-type-plugin(check): getScope(node): function, isStrict: true
  | getScope(proxy): module, upper: global
   ,-[files/without_esm.js:1:1]
 1 | function foo() {}
   : ^^^^^^^^^^^^^^^^^
   `----

  x source-type-plugin(check): Program.sourceType: module
  | languageOptions.sourceType: module
  | scopes: global, module, function
   ,-[files/without_esm.js:1:1]
 1 | function foo() {}
   : ^^^^^^^^^^^^^^^^^^
   `----

  x eslint(no-implicit-globals): Unexpected function declaration in the global scope.
   ,-[files/without_esm.js:1:10]
 1 | function foo() {}
   :          ^^^
   `----
  help: Wrap it in an IIFE for a local variable, or assign it as a global property for a global variable.

  x source-type-plugin(check): getScope(node): function, isStrict: true
  | getScope(proxy): module, upper: global
   ,-[files/without_esm.ts:1:1]
 1 | function foo(): void {}
   : ^^^^^^^^^^^^^^^^^^^^^^^
   `----

  x source-type-plugin(check): Program.sourceType: module
  | languageOptions.sourceType: module
  | scopes: global, module, function
   ,-[files/without_esm.ts:1:1]
 1 | function foo(): void {}
   : ^^^^^^^^^^^^^^^^^^^^^^^^
   `----

Found 0 warnings and 15 errors.
Finished in Xms on 5 files with 2 rules using X threads.
```

# stderr
```
```
