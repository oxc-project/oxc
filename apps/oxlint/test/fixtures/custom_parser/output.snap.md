# Exit code
1

# stdout
```
  [38;2;244;191;117;1m⚠[0m [38;2;244;191;117;1meslint(no-var): Unexpected var, use let or const instead.[0m
   ╭─[[38;2;92;157;255;1mfiles/test1.custom[0m:2:1]
 [2m1[0m │ # Test file for custom parser
 [2m2[0m │ var foo
   · [38;2;246;87;248m───[0m
 [2m3[0m │ var bar
   ╰────
[38;2;106;159;181m  help: [0mReplace var with let or const

  [38;2;244;191;117;1m⚠[0m [38;2;244;191;117;1meslint(no-var): Unexpected var, use let or const instead.[0m
   ╭─[[38;2;92;157;255;1mfiles/test1.custom[0m:3:1]
 [2m2[0m │ var foo
 [2m3[0m │ var bar
   · [38;2;246;87;248m───[0m
 [2m4[0m │ log foo
   ╰────
[38;2;106;159;181m  help: [0mReplace var with let or const

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mcustom-parser-plugin(count-identifiers): Starting to lint: <fixture>/files/test1.custom[0m
   ╭─[[38;2;92;157;255;1mfiles/test1.custom[0m:1:1]
 [2m1[0m │ # Test file for custom parser
   · [38;2;246;87;248m▲[0m
 [2m2[0m │ var foo
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mcustom-parser-plugin(no-foo-var): Variable "foo" is not allowed in custom DSL files[0m
   ╭─[[38;2;92;157;255;1mfiles/test1.custom[0m:2:5]
 [2m1[0m │ # Test file for custom parser
 [2m2[0m │ var foo
   · [38;2;246;87;248m    ───[0m
 [2m3[0m │ var bar
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mcustom-parser-plugin(count-identifiers): Found identifier: foo[0m
   ╭─[[38;2;92;157;255;1mfiles/test1.custom[0m:2:5]
 [2m1[0m │ # Test file for custom parser
 [2m2[0m │ var foo
   · [38;2;246;87;248m    ───[0m
 [2m3[0m │ var bar
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mcustom-parser-plugin(count-identifiers): Found identifier: bar[0m
   ╭─[[38;2;92;157;255;1mfiles/test1.custom[0m:3:5]
 [2m2[0m │ var foo
 [2m3[0m │ var bar
   · [38;2;246;87;248m    ───[0m
 [2m4[0m │ log foo
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mcustom-parser-plugin(count-identifiers): Found identifier: console.log[0m
   ╭─[[38;2;92;157;255;1mfiles/test1.custom[0m:4:1]
 [2m3[0m │ var bar
 [2m4[0m │ log foo
   · [38;2;246;87;248m───[0m
 [2m5[0m │ log bar
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mcustom-parser-plugin(count-identifiers): Found identifier: foo[0m
   ╭─[[38;2;92;157;255;1mfiles/test1.custom[0m:4:5]
 [2m3[0m │ var bar
 [2m4[0m │ log foo
   · [38;2;246;87;248m    ───[0m
 [2m5[0m │ log bar
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mcustom-parser-plugin(count-identifiers): Found identifier: console.log[0m
   ╭─[[38;2;92;157;255;1mfiles/test1.custom[0m:5:1]
 [2m4[0m │ log foo
 [2m5[0m │ log bar
   · [38;2;246;87;248m───[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mcustom-parser-plugin(count-identifiers): Found identifier: bar[0m
   ╭─[[38;2;92;157;255;1mfiles/test1.custom[0m:5:5]
 [2m4[0m │ log foo
 [2m5[0m │ log bar
   · [38;2;246;87;248m    ───[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mcustom-parser-plugin(count-identifiers): Total identifiers found: 6[0m
   ╭─[[38;2;92;157;255;1mfiles/test1.custom[0m:1:1]
 [2m1[0m │ # Test file for custom parser
   · [38;2;246;87;248m▲[0m
 [2m2[0m │ var foo
   ╰────

  [38;2;244;191;117;1m⚠[0m [38;2;244;191;117;1meslint(no-var): Unexpected var, use let or const instead.[0m
   ╭─[[38;2;92;157;255;1mfiles/test2.custom[0m:2:1]
 [2m1[0m │ # Another test file
 [2m2[0m │ var x
   · [38;2;246;87;248m───[0m
 [2m3[0m │ log x
   ╰────
[38;2;106;159;181m  help: [0mReplace var with let or const

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mcustom-parser-plugin(count-identifiers): Starting to lint: <fixture>/files/test2.custom[0m
   ╭─[[38;2;92;157;255;1mfiles/test2.custom[0m:1:1]
 [2m1[0m │ # Another test file
   · [38;2;246;87;248m▲[0m
 [2m2[0m │ var x
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mcustom-parser-plugin(count-identifiers): Found identifier: x[0m
   ╭─[[38;2;92;157;255;1mfiles/test2.custom[0m:2:5]
 [2m1[0m │ # Another test file
 [2m2[0m │ var x
   · [38;2;246;87;248m    ─[0m
 [2m3[0m │ log x
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mcustom-parser-plugin(count-identifiers): Found identifier: console.log[0m
   ╭─[[38;2;92;157;255;1mfiles/test2.custom[0m:3:1]
 [2m2[0m │ var x
 [2m3[0m │ log x
   · [38;2;246;87;248m───[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mcustom-parser-plugin(count-identifiers): Found identifier: x[0m
   ╭─[[38;2;92;157;255;1mfiles/test2.custom[0m:3:5]
 [2m2[0m │ var x
 [2m3[0m │ log x
   · [38;2;246;87;248m    ─[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mcustom-parser-plugin(count-identifiers): Total identifiers found: 3[0m
   ╭─[[38;2;92;157;255;1mfiles/test2.custom[0m:1:1]
 [2m1[0m │ # Another test file
   · [38;2;246;87;248m▲[0m
 [2m2[0m │ var x
   ╰────

Found 3 warnings and 14 errors.
Finished in Xms on 2 files using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
WARNING: JS parsers are experimental and not subject to semver.
Breaking changes are possible while JS parsers support is under development.
```
