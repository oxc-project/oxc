# Exit code
1

# stdout
```
  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mdefine-rule-plugin(create): create body:
  [38;2;225;80;80;1m│[0m this === rule: true[0m
   ╭─[[38;2;92;157;255;1mfiles/1.js[0m:1:1]
 [2m1[0m │ let a, b;
   · [38;2;246;87;248m▲[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mdefine-rule-plugin(create-once): before hook:
  [38;2;225;80;80;1m│[0m createOnce call count: 1
  [38;2;225;80;80;1m│[0m this === rule: true
  [38;2;225;80;80;1m│[0m filename: <fixture>/files/1.js[0m
   ╭─[[38;2;92;157;255;1mfiles/1.js[0m:1:1]
 [2m1[0m │ let a, b;
   · [38;2;246;87;248m▲[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mdefine-rule-plugin(create-once-before-false): before hook:
  [38;2;225;80;80;1m│[0m filename: <fixture>/files/1.js[0m
   ╭─[[38;2;92;157;255;1mfiles/1.js[0m:1:1]
 [2m1[0m │ let a, b;
   · [38;2;246;87;248m▲[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mdefine-rule-plugin(create-once-before-only): before hook:
  [38;2;225;80;80;1m│[0m filename: <fixture>/files/1.js[0m
   ╭─[[38;2;92;157;255;1mfiles/1.js[0m:1:1]
 [2m1[0m │ let a, b;
   · [38;2;246;87;248m▲[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mdefine-rule-plugin(create): ident visit fn "a":
  [38;2;225;80;80;1m│[0m filename: <fixture>/files/1.js[0m
   ╭─[[38;2;92;157;255;1mfiles/1.js[0m:1:5]
 [2m1[0m │ let a, b;
   · [38;2;246;87;248m    ─[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mdefine-rule-plugin(create-once): ident visit fn "a":
  [38;2;225;80;80;1m│[0m identNum: 1
  [38;2;225;80;80;1m│[0m filename: <fixture>/files/1.js[0m
   ╭─[[38;2;92;157;255;1mfiles/1.js[0m:1:5]
 [2m1[0m │ let a, b;
   · [38;2;246;87;248m    ─[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mdefine-rule-plugin(create-once-before-only): ident visit fn "a":
  [38;2;225;80;80;1m│[0m filename: <fixture>/files/1.js[0m
   ╭─[[38;2;92;157;255;1mfiles/1.js[0m:1:5]
 [2m1[0m │ let a, b;
   · [38;2;246;87;248m    ─[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mdefine-rule-plugin(create-once-after-only): ident visit fn "a":
  [38;2;225;80;80;1m│[0m filename: <fixture>/files/1.js[0m
   ╭─[[38;2;92;157;255;1mfiles/1.js[0m:1:5]
 [2m1[0m │ let a, b;
   · [38;2;246;87;248m    ─[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mdefine-rule-plugin(create-once-no-hooks): ident visit fn "a":
  [38;2;225;80;80;1m│[0m filename: <fixture>/files/1.js[0m
   ╭─[[38;2;92;157;255;1mfiles/1.js[0m:1:5]
 [2m1[0m │ let a, b;
   · [38;2;246;87;248m    ─[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mdefine-rule-plugin(create): ident visit fn "b":
  [38;2;225;80;80;1m│[0m filename: <fixture>/files/1.js[0m
   ╭─[[38;2;92;157;255;1mfiles/1.js[0m:1:8]
 [2m1[0m │ let a, b;
   · [38;2;246;87;248m       ─[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mdefine-rule-plugin(create-once): ident visit fn "b":
  [38;2;225;80;80;1m│[0m identNum: 2
  [38;2;225;80;80;1m│[0m filename: <fixture>/files/1.js[0m
   ╭─[[38;2;92;157;255;1mfiles/1.js[0m:1:8]
 [2m1[0m │ let a, b;
   · [38;2;246;87;248m       ─[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mdefine-rule-plugin(create-once-before-only): ident visit fn "b":
  [38;2;225;80;80;1m│[0m filename: <fixture>/files/1.js[0m
   ╭─[[38;2;92;157;255;1mfiles/1.js[0m:1:8]
 [2m1[0m │ let a, b;
   · [38;2;246;87;248m       ─[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mdefine-rule-plugin(create-once-after-only): ident visit fn "b":
  [38;2;225;80;80;1m│[0m filename: <fixture>/files/1.js[0m
   ╭─[[38;2;92;157;255;1mfiles/1.js[0m:1:8]
 [2m1[0m │ let a, b;
   · [38;2;246;87;248m       ─[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mdefine-rule-plugin(create-once-no-hooks): ident visit fn "b":
  [38;2;225;80;80;1m│[0m filename: <fixture>/files/1.js[0m
   ╭─[[38;2;92;157;255;1mfiles/1.js[0m:1:8]
 [2m1[0m │ let a, b;
   · [38;2;246;87;248m       ─[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mdefine-rule-plugin(create-once): after hook:
  [38;2;225;80;80;1m│[0m identNum: 2
  [38;2;225;80;80;1m│[0m filename: <fixture>/files/1.js[0m
   ╭─[[38;2;92;157;255;1mfiles/1.js[0m:1:1]
 [2m1[0m │ let a, b;
   · [38;2;246;87;248m▲[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mdefine-rule-plugin(create-once-after-only): after hook:
  [38;2;225;80;80;1m│[0m filename: <fixture>/files/1.js[0m
   ╭─[[38;2;92;157;255;1mfiles/1.js[0m:1:1]
 [2m1[0m │ let a, b;
   · [38;2;246;87;248m▲[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mdefine-rule-plugin(create): create body:
  [38;2;225;80;80;1m│[0m this === rule: true[0m
   ╭─[[38;2;92;157;255;1mfiles/2.js[0m:1:1]
 [2m1[0m │ let c, d;
   · [38;2;246;87;248m▲[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mdefine-rule-plugin(create-once): before hook:
  [38;2;225;80;80;1m│[0m createOnce call count: 1
  [38;2;225;80;80;1m│[0m this === rule: true
  [38;2;225;80;80;1m│[0m filename: <fixture>/files/2.js[0m
   ╭─[[38;2;92;157;255;1mfiles/2.js[0m:1:1]
 [2m1[0m │ let c, d;
   · [38;2;246;87;248m▲[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mdefine-rule-plugin(create-once-before-false): before hook:
  [38;2;225;80;80;1m│[0m filename: <fixture>/files/2.js[0m
   ╭─[[38;2;92;157;255;1mfiles/2.js[0m:1:1]
 [2m1[0m │ let c, d;
   · [38;2;246;87;248m▲[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mdefine-rule-plugin(create-once-before-only): before hook:
  [38;2;225;80;80;1m│[0m filename: <fixture>/files/2.js[0m
   ╭─[[38;2;92;157;255;1mfiles/2.js[0m:1:1]
 [2m1[0m │ let c, d;
   · [38;2;246;87;248m▲[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mdefine-rule-plugin(create): ident visit fn "c":
  [38;2;225;80;80;1m│[0m filename: <fixture>/files/2.js[0m
   ╭─[[38;2;92;157;255;1mfiles/2.js[0m:1:5]
 [2m1[0m │ let c, d;
   · [38;2;246;87;248m    ─[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mdefine-rule-plugin(create-once): ident visit fn "c":
  [38;2;225;80;80;1m│[0m identNum: 1
  [38;2;225;80;80;1m│[0m filename: <fixture>/files/2.js[0m
   ╭─[[38;2;92;157;255;1mfiles/2.js[0m:1:5]
 [2m1[0m │ let c, d;
   · [38;2;246;87;248m    ─[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mdefine-rule-plugin(create-once-before-false): ident visit fn "c":
  [38;2;225;80;80;1m│[0m filename: <fixture>/files/2.js[0m
   ╭─[[38;2;92;157;255;1mfiles/2.js[0m:1:5]
 [2m1[0m │ let c, d;
   · [38;2;246;87;248m    ─[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mdefine-rule-plugin(create-once-before-only): ident visit fn "c":
  [38;2;225;80;80;1m│[0m filename: <fixture>/files/2.js[0m
   ╭─[[38;2;92;157;255;1mfiles/2.js[0m:1:5]
 [2m1[0m │ let c, d;
   · [38;2;246;87;248m    ─[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mdefine-rule-plugin(create-once-after-only): ident visit fn "c":
  [38;2;225;80;80;1m│[0m filename: <fixture>/files/2.js[0m
   ╭─[[38;2;92;157;255;1mfiles/2.js[0m:1:5]
 [2m1[0m │ let c, d;
   · [38;2;246;87;248m    ─[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mdefine-rule-plugin(create-once-no-hooks): ident visit fn "c":
  [38;2;225;80;80;1m│[0m filename: <fixture>/files/2.js[0m
   ╭─[[38;2;92;157;255;1mfiles/2.js[0m:1:5]
 [2m1[0m │ let c, d;
   · [38;2;246;87;248m    ─[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mdefine-rule-plugin(create): ident visit fn "d":
  [38;2;225;80;80;1m│[0m filename: <fixture>/files/2.js[0m
   ╭─[[38;2;92;157;255;1mfiles/2.js[0m:1:8]
 [2m1[0m │ let c, d;
   · [38;2;246;87;248m       ─[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mdefine-rule-plugin(create-once): ident visit fn "d":
  [38;2;225;80;80;1m│[0m identNum: 2
  [38;2;225;80;80;1m│[0m filename: <fixture>/files/2.js[0m
   ╭─[[38;2;92;157;255;1mfiles/2.js[0m:1:8]
 [2m1[0m │ let c, d;
   · [38;2;246;87;248m       ─[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mdefine-rule-plugin(create-once-before-false): ident visit fn "d":
  [38;2;225;80;80;1m│[0m filename: <fixture>/files/2.js[0m
   ╭─[[38;2;92;157;255;1mfiles/2.js[0m:1:8]
 [2m1[0m │ let c, d;
   · [38;2;246;87;248m       ─[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mdefine-rule-plugin(create-once-before-only): ident visit fn "d":
  [38;2;225;80;80;1m│[0m filename: <fixture>/files/2.js[0m
   ╭─[[38;2;92;157;255;1mfiles/2.js[0m:1:8]
 [2m1[0m │ let c, d;
   · [38;2;246;87;248m       ─[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mdefine-rule-plugin(create-once-after-only): ident visit fn "d":
  [38;2;225;80;80;1m│[0m filename: <fixture>/files/2.js[0m
   ╭─[[38;2;92;157;255;1mfiles/2.js[0m:1:8]
 [2m1[0m │ let c, d;
   · [38;2;246;87;248m       ─[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mdefine-rule-plugin(create-once-no-hooks): ident visit fn "d":
  [38;2;225;80;80;1m│[0m filename: <fixture>/files/2.js[0m
   ╭─[[38;2;92;157;255;1mfiles/2.js[0m:1:8]
 [2m1[0m │ let c, d;
   · [38;2;246;87;248m       ─[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mdefine-rule-plugin(create-once): after hook:
  [38;2;225;80;80;1m│[0m identNum: 2
  [38;2;225;80;80;1m│[0m filename: <fixture>/files/2.js[0m
   ╭─[[38;2;92;157;255;1mfiles/2.js[0m:1:1]
 [2m1[0m │ let c, d;
   · [38;2;246;87;248m▲[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mdefine-rule-plugin(create-once-before-false): after hook:
  [38;2;225;80;80;1m│[0m filename: <fixture>/files/2.js[0m
   ╭─[[38;2;92;157;255;1mfiles/2.js[0m:1:1]
 [2m1[0m │ let c, d;
   · [38;2;246;87;248m▲[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mdefine-rule-plugin(create-once-after-only): after hook:
  [38;2;225;80;80;1m│[0m filename: <fixture>/files/2.js[0m
   ╭─[[38;2;92;157;255;1mfiles/2.js[0m:1:1]
 [2m1[0m │ let c, d;
   · [38;2;246;87;248m▲[0m
   ╰────

Found 0 warnings and 35 errors.
Finished in Xms on 2 files using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
