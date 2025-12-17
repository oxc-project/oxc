# Exit code
1

# stdout
```
  x globals-plugin(globals): {
  |   "React": "readonly",
  |   "console": "readonly",
  |   "baz": "writable",
  |   "foo": "writable",
  |   "process": "writable",
  |   "bar": "readonly",
  |   "qux": "readonly",
  |   "window": "off"
  | }
   ,-[files/index.js:1:1]
 1 | debugger;
   : ^
   `----

  x globals-plugin(globals): {
  |   "React": "writable",
  |   "console": "readonly",
  |   "baz": "writable",
  |   "foo": "writable",
  |   "process": "off",
  |   "bar": "readonly",
  |   "qux": "readonly",
  |   "customGlobal": "readonly",
  |   "window": "off"
  | }
   ,-[files/nested/index.js:1:1]
 1 | let x;
   : ^
   `----

Found 0 warnings and 2 errors.
Finished in Xms on 2 files using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
