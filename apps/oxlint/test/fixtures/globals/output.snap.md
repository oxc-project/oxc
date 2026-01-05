# Exit code
1

# stdout
```
  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mglobals-plugin(globals):
  [38;2;225;80;80;1m│[0m globals: {
  [38;2;225;80;80;1m│[0m   "React": "readonly",
  [38;2;225;80;80;1m│[0m   "console": "readonly",
  [38;2;225;80;80;1m│[0m   "baz": "writable",
  [38;2;225;80;80;1m│[0m   "foo": "writable",
  [38;2;225;80;80;1m│[0m   "process": "writable",
  [38;2;225;80;80;1m│[0m   "bar": "readonly",
  [38;2;225;80;80;1m│[0m   "qux": "readonly",
  [38;2;225;80;80;1m│[0m   "window": "off"
  [38;2;225;80;80;1m│[0m }
  [38;2;225;80;80;1m│[0m env: {
  [38;2;225;80;80;1m│[0m   "builtin": true
  [38;2;225;80;80;1m│[0m }[0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:1:1]
 [2m1[0m │ debugger;
   · [38;2;246;87;248m▲[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mglobals-plugin(globals):
  [38;2;225;80;80;1m│[0m globals: {
  [38;2;225;80;80;1m│[0m   "React": "writable",
  [38;2;225;80;80;1m│[0m   "console": "readonly",
  [38;2;225;80;80;1m│[0m   "baz": "writable",
  [38;2;225;80;80;1m│[0m   "foo": "writable",
  [38;2;225;80;80;1m│[0m   "process": "off",
  [38;2;225;80;80;1m│[0m   "bar": "readonly",
  [38;2;225;80;80;1m│[0m   "qux": "readonly",
  [38;2;225;80;80;1m│[0m   "customGlobal": "readonly",
  [38;2;225;80;80;1m│[0m   "window": "off"
  [38;2;225;80;80;1m│[0m }
  [38;2;225;80;80;1m│[0m env: {
  [38;2;225;80;80;1m│[0m   "astro": true,
  [38;2;225;80;80;1m│[0m   "builtin": true,
  [38;2;225;80;80;1m│[0m   "node": true
  [38;2;225;80;80;1m│[0m }[0m
   ╭─[[38;2;92;157;255;1mfiles/nested/2.js[0m:1:1]
 [2m1[0m │ let y;
   · [38;2;246;87;248m▲[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mglobals-plugin(globals):
  [38;2;225;80;80;1m│[0m globals: {
  [38;2;225;80;80;1m│[0m   "React": "readonly",
  [38;2;225;80;80;1m│[0m   "console": "readonly",
  [38;2;225;80;80;1m│[0m   "baz": "writable",
  [38;2;225;80;80;1m│[0m   "foo": "writable",
  [38;2;225;80;80;1m│[0m   "process": "writable",
  [38;2;225;80;80;1m│[0m   "bar": "readonly",
  [38;2;225;80;80;1m│[0m   "qux": "readonly",
  [38;2;225;80;80;1m│[0m   "window": "off"
  [38;2;225;80;80;1m│[0m }
  [38;2;225;80;80;1m│[0m env: {
  [38;2;225;80;80;1m│[0m   "browser": true,
  [38;2;225;80;80;1m│[0m   "node": true,
  [38;2;225;80;80;1m│[0m   "builtin": true
  [38;2;225;80;80;1m│[0m }[0m
   ╭─[[38;2;92;157;255;1mfiles/nested/1.js[0m:1:1]
 [2m1[0m │ let x;
   · [38;2;246;87;248m▲[0m
   ╰────

Found 0 warnings and 3 errors.
Finished in Xms on 3 files with 1 rules using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
