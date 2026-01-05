# Exit code
1

# stdout
```
  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1meslint-plugin-jsdoc(require-param): Missing JSDoc `@param` declaration for function parameters.[0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:4:17]
 [2m3[0m │  */
 [2m4[0m │ function f(foo, bar) {}
   · [38;2;246;87;248m                ───[0m
   ╰────
[38;2;106;159;181m  help: [0mAdd `@param` tag with name.

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1m@scope(rule): id: @scope/rule[0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:4:1]
 [2m3[0m │  */
 [2m4[0m │ function f(foo, bar) {}
   · [38;2;246;87;248m───────────────────────[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mplugin3-name-from-package(rule): id: plugin3-name-from-package/rule[0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:4:1]
 [2m3[0m │  */
 [2m4[0m │ function f(foo, bar) {}
   · [38;2;246;87;248m───────────────────────[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mno-name-alias2(rule): id: no-name-alias2/rule[0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:4:1]
 [2m3[0m │  */
 [2m4[0m │ function f(foo, bar) {}
   · [38;2;246;87;248m───────────────────────[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mplugin2-name-from-package(rule): id: plugin2-name-from-package/rule[0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:4:1]
 [2m3[0m │  */
 [2m4[0m │ function f(foo, bar) {}
   · [38;2;246;87;248m───────────────────────[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mjs-jsdoc2(rule): id: js-jsdoc2/rule[0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:4:1]
 [2m3[0m │  */
 [2m4[0m │ function f(foo, bar) {}
   · [38;2;246;87;248m───────────────────────[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1m@scope2(rule): id: @scope2/rule[0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:4:1]
 [2m3[0m │  */
 [2m4[0m │ function f(foo, bar) {}
   · [38;2;246;87;248m───────────────────────[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mplugin1-name-from-rule(rule): id: plugin1-name-from-rule/rule[0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:4:1]
 [2m3[0m │  */
 [2m4[0m │ function f(foo, bar) {}
   · [38;2;246;87;248m───────────────────────[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mno-name-alias(rule): id: no-name-alias/rule[0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:4:1]
 [2m3[0m │  */
 [2m4[0m │ function f(foo, bar) {}
   · [38;2;246;87;248m───────────────────────[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1m@scope3/subplugin(rule): id: @scope3/subplugin/rule[0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:4:1]
 [2m3[0m │  */
 [2m4[0m │ function f(foo, bar) {}
   · [38;2;246;87;248m───────────────────────[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mplugin4-name-from-rule(rule): id: plugin4-name-from-rule/rule[0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:4:1]
 [2m3[0m │  */
 [2m4[0m │ function f(foo, bar) {}
   · [38;2;246;87;248m───────────────────────[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mjs-jsdoc(rule): id: js-jsdoc/rule[0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:4:1]
 [2m3[0m │  */
 [2m4[0m │ function f(foo, bar) {}
   · [38;2;246;87;248m───────────────────────[0m
   ╰────

Found 0 warnings and 12 errors.
Finished in Xms on 1 file with 12 rules using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
