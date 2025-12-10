# Exit code
1

# stdout
```
  x wrapped-context(wrapped-rule): wrapped 1: filename: <fixture>/files/index.js
   ,-[files/index.js:1:1]
 1 | console.log("Hello, world!");
   : ^
   `----

  x wrapped-context(wrapped-rule): wrapped 1: id: wrapped-context/wrapped-rule
   ,-[files/index.js:1:1]
 1 | console.log("Hello, world!");
   : ^
   `----

  x wrapped-context(wrapped-rule): wrapped 1: source text: "console.log(\"Hello, world!\");\n"
   ,-[files/index.js:1:1]
 1 | console.log("Hello, world!");
   : ^
   `----

  x wrapped-context(wrapped-rule2): wrapped 2: filename: <fixture>/files/index.js
   ,-[files/index.js:1:1]
 1 | console.log("Hello, world!");
   : ^
   `----

  x wrapped-context(wrapped-rule2): wrapped 2: id: wrapped-context/wrapped-rule2
   ,-[files/index.js:1:1]
 1 | console.log("Hello, world!");
   : ^
   `----

  x wrapped-context(wrapped-rule2): wrapped 2: source text: "console.log(\"Hello, world!\");\n"
   ,-[files/index.js:1:1]
 1 | console.log("Hello, world!");
   : ^
   `----

  x wrapped-context(wrapped-rule): wrapped 1: Identifier: 'console'
   ,-[files/index.js:1:1]
 1 | console.log("Hello, world!");
   : ^^^^^^^
   `----

  x wrapped-context(wrapped-rule2): wrapped 2: Identifier: 'console'
   ,-[files/index.js:1:1]
 1 | console.log("Hello, world!");
   : ^^^^^^^
   `----

  x wrapped-context(wrapped-rule): wrapped 1: Identifier: 'log'
   ,-[files/index.js:1:9]
 1 | console.log("Hello, world!");
   :         ^^^
   `----

  x wrapped-context(wrapped-rule2): wrapped 2: Identifier: 'log'
   ,-[files/index.js:1:9]
 1 | console.log("Hello, world!");
   :         ^^^
   `----

Found 0 warnings and 10 errors.
Finished in Xms on 1 file using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
