# Exit code
1

# stdout
```
  x globals-plugin(globals):
  | globals: {
  |   "React": "readonly",
  |   "console": "readonly",
  |   "baz": "writable",
  |   "foo": "writable",
  |   "process": "writable",
  |   "bar": "readonly",
  |   "qux": "readonly",
  |   "window": "off"
  | }
  | env: {
  |   "builtin": true
  | }
   ,-[files/index.js:1:1]
 1 | debugger;
   : ^
   `----

  x globals-plugin(globals):
  | globals: {
  |   "React": "readonly",
  |   "console": "readonly",
  |   "baz": "writable",
  |   "foo": "writable",
  |   "process": "writable",
  |   "bar": "readonly",
  |   "qux": "readonly",
  |   "window": "off"
  | }
  | env: {
  |   "browser": true,
  |   "node": true,
  |   "builtin": true
  | }
   ,-[files/nested/1.js:1:1]
 1 | let x;
   : ^
   `----

  x globals-plugin(globals):
  | globals: {
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
  | env: {
  |   "astro": true,
  |   "builtin": true,
  |   "node": true
  | }
   ,-[files/nested/2.js:1:1]
 1 | let y;
   : ^
   `----

Found 0 warnings and 3 errors.
Finished in Xms on 3 files with 1 rules using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
