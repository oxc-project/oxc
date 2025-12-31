# Exit code
1

# stdout
```
  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mscope-plugin(scope): getDeclaredVariables(): topLevelConstant[0m
   ╭─[[38;2;92;157;255;1mfiles/index.cjs[0m:1:1]
 [2m1[0m │ const topLevelConstant = 1;
   · [38;2;246;87;248m───────────────────────────[0m
 [2m2[0m │ let topLevelLet = 2;
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mscope-plugin(scope): isGlobalReference(topLevelConstant): false[0m
   ╭─[[38;2;92;157;255;1mfiles/index.cjs[0m:1:7]
 [2m1[0m │ const topLevelConstant = 1;
   · [38;2;246;87;248m      ────────────────[0m
 [2m2[0m │ let topLevelLet = 2;
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mscope-plugin(scope): getDeclaredVariables(): topLevelLet[0m
   ╭─[[38;2;92;157;255;1mfiles/index.cjs[0m:2:1]
 [2m1[0m │ const topLevelConstant = 1;
 [2m2[0m │ let topLevelLet = 2;
   · [38;2;246;87;248m────────────────────[0m
 [2m3[0m │ var topLevelVar = 3;
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mscope-plugin(scope): isGlobalReference(topLevelLet): false[0m
   ╭─[[38;2;92;157;255;1mfiles/index.cjs[0m:2:5]
 [2m1[0m │ const topLevelConstant = 1;
 [2m2[0m │ let topLevelLet = 2;
   · [38;2;246;87;248m    ───────────[0m
 [2m3[0m │ var topLevelVar = 3;
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mscope-plugin(scope): getDeclaredVariables(): topLevelVar[0m
   ╭─[[38;2;92;157;255;1mfiles/index.cjs[0m:3:1]
 [2m2[0m │ let topLevelLet = 2;
 [2m3[0m │ var topLevelVar = 3;
   · [38;2;246;87;248m────────────────────[0m
 [2m4[0m │ 
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mscope-plugin(scope): isGlobalReference(topLevelVar): false[0m
   ╭─[[38;2;92;157;255;1mfiles/index.cjs[0m:3:5]
 [2m2[0m │ let topLevelLet = 2;
 [2m3[0m │ var topLevelVar = 3;
   · [38;2;246;87;248m    ───────────[0m
 [2m4[0m │ 
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mscope-plugin(scope): getScope(topLevelFunction):
  [38;2;225;80;80;1m│[0m type: function
  [38;2;225;80;80;1m│[0m isStrict: false
  [38;2;225;80;80;1m│[0m variables: [arguments, innerFunction]
  [38;2;225;80;80;1m│[0m through: [Object]
  [38;2;225;80;80;1m│[0m upper type: global[0m
    ╭─[[38;2;92;157;255;1mfiles/index.cjs[0m:5:1]
 [2m 4[0m │     
 [2m 5[0m │ [38;2;246;87;248m╭[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m function topLevelFunction() {
 [2m 6[0m │ [38;2;246;87;248m│[0m     function innerFunction() {
 [2m 7[0m │ [38;2;246;87;248m│[0m       function nestedFunction() {
 [2m 8[0m │ [38;2;246;87;248m│[0m         "use strict";
 [2m 9[0m │ [38;2;246;87;248m│[0m       }
 [2m10[0m │ [38;2;246;87;248m│[0m     }
 [2m11[0m │ [38;2;246;87;248m│[0m     return Object;
 [2m12[0m │ [38;2;246;87;248m╰[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m }
 [2m13[0m │     
    ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mscope-plugin(scope): isGlobalReference(topLevelFunction): false[0m
   ╭─[[38;2;92;157;255;1mfiles/index.cjs[0m:5:10]
 [2m4[0m │ 
 [2m5[0m │ function topLevelFunction() {
   · [38;2;246;87;248m         ────────────────[0m
 [2m6[0m │   function innerFunction() {
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mscope-plugin(scope): getScope(innerFunction):
  [38;2;225;80;80;1m│[0m type: function
  [38;2;225;80;80;1m│[0m isStrict: false
  [38;2;225;80;80;1m│[0m variables: [arguments, nestedFunction]
  [38;2;225;80;80;1m│[0m through: []
  [38;2;225;80;80;1m│[0m upper type: function[0m
    ╭─[[38;2;92;157;255;1mfiles/index.cjs[0m:6:3]
 [2m 5[0m │     function topLevelFunction() {
 [2m 6[0m │ [38;2;246;87;248m╭[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m   function innerFunction() {
 [2m 7[0m │ [38;2;246;87;248m│[0m       function nestedFunction() {
 [2m 8[0m │ [38;2;246;87;248m│[0m         "use strict";
 [2m 9[0m │ [38;2;246;87;248m│[0m       }
 [2m10[0m │ [38;2;246;87;248m╰[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m   }
 [2m11[0m │       return Object;
    ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mscope-plugin(scope): isGlobalReference(innerFunction): false[0m
   ╭─[[38;2;92;157;255;1mfiles/index.cjs[0m:6:12]
 [2m5[0m │ function topLevelFunction() {
 [2m6[0m │   function innerFunction() {
   · [38;2;246;87;248m           ─────────────[0m
 [2m7[0m │     function nestedFunction() {
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mscope-plugin(scope): getScope(nestedFunction):
  [38;2;225;80;80;1m│[0m type: function
  [38;2;225;80;80;1m│[0m isStrict: true
  [38;2;225;80;80;1m│[0m variables: [arguments]
  [38;2;225;80;80;1m│[0m through: []
  [38;2;225;80;80;1m│[0m upper type: function[0m
    ╭─[[38;2;92;157;255;1mfiles/index.cjs[0m:7:5]
 [2m 6[0m │       function innerFunction() {
 [2m 7[0m │ [38;2;246;87;248m╭[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m     function nestedFunction() {
 [2m 8[0m │ [38;2;246;87;248m│[0m         "use strict";
 [2m 9[0m │ [38;2;246;87;248m╰[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m     }
 [2m10[0m │       }
    ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mscope-plugin(scope): isGlobalReference(nestedFunction): false[0m
   ╭─[[38;2;92;157;255;1mfiles/index.cjs[0m:7:14]
 [2m6[0m │   function innerFunction() {
 [2m7[0m │     function nestedFunction() {
   · [38;2;246;87;248m             ──────────────[0m
 [2m8[0m │       "use strict";
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mscope-plugin(scope): isGlobalReference(Object): true[0m
    ╭─[[38;2;92;157;255;1mfiles/index.cjs[0m:11:10]
 [2m10[0m │   }
 [2m11[0m │   return Object;
    · [38;2;246;87;248m         ──────[0m
 [2m12[0m │ }
    ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mscope-plugin(scope): isGlobalReference(module): false[0m
    ╭─[[38;2;92;157;255;1mfiles/index.cjs[0m:14:1]
 [2m13[0m │ 
 [2m14[0m │ module.exports = topLevelFunction();
    · [38;2;246;87;248m──────[0m
    ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mscope-plugin(scope): isGlobalReference(exports): false[0m
    ╭─[[38;2;92;157;255;1mfiles/index.cjs[0m:14:8]
 [2m13[0m │ 
 [2m14[0m │ module.exports = topLevelFunction();
    · [38;2;246;87;248m       ───────[0m
    ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mscope-plugin(scope): isGlobalReference(topLevelFunction): false[0m
    ╭─[[38;2;92;157;255;1mfiles/index.cjs[0m:14:18]
 [2m13[0m │ 
 [2m14[0m │ module.exports = topLevelFunction();
    · [38;2;246;87;248m                 ────────────────[0m
    ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mscope-plugin(scope): getDeclaredVariables(): topLevelConstant, secondTopLevelConstant[0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:1:1]
 [2m1[0m │ [38;2;246;87;248m╭[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m const topLevelConstant = 1,
 [2m2[0m │ [38;2;246;87;248m╰[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m   secondTopLevelConstant = 2;
 [2m3[0m │     
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mscope-plugin(scope): isGlobalReference(topLevelConstant): false[0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:1:7]
 [2m1[0m │ const topLevelConstant = 1,
   · [38;2;246;87;248m      ────────────────[0m
 [2m2[0m │   secondTopLevelConstant = 2;
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mscope-plugin(scope): isGlobalReference(secondTopLevelConstant): false[0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:2:3]
 [2m1[0m │ const topLevelConstant = 1,
 [2m2[0m │   secondTopLevelConstant = 2;
   · [38;2;246;87;248m  ──────────────────────[0m
 [2m3[0m │ 
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mscope-plugin(scope): getScope(topLevelFunction):
  [38;2;225;80;80;1m│[0m type: function
  [38;2;225;80;80;1m│[0m isStrict: true
  [38;2;225;80;80;1m│[0m variables: [arguments, param, localConstant]
  [38;2;225;80;80;1m│[0m through: [topLevelConstant, Math]
  [38;2;225;80;80;1m│[0m upper type: module[0m
    ╭─[[38;2;92;157;255;1mfiles/index.js[0m:4:1]
 [2m 3[0m │     
 [2m 4[0m │ [38;2;246;87;248m╭[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m function topLevelFunction(param) {
 [2m 5[0m │ [38;2;246;87;248m│[0m     const localConstant = topLevelConstant + param;
 [2m 6[0m │ [38;2;246;87;248m│[0m     return function innerFunction() {
 [2m 7[0m │ [38;2;246;87;248m│[0m       return localConstant + Math.PI;
 [2m 8[0m │ [38;2;246;87;248m│[0m     };
 [2m 9[0m │ [38;2;246;87;248m╰[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m }
 [2m10[0m │     
    ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mscope-plugin(scope): isGlobalReference(topLevelFunction): false[0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:4:10]
 [2m3[0m │ 
 [2m4[0m │ function topLevelFunction(param) {
   · [38;2;246;87;248m         ────────────────[0m
 [2m5[0m │   const localConstant = topLevelConstant + param;
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mscope-plugin(scope): isGlobalReference(param): false[0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:4:27]
 [2m3[0m │ 
 [2m4[0m │ function topLevelFunction(param) {
   · [38;2;246;87;248m                          ─────[0m
 [2m5[0m │   const localConstant = topLevelConstant + param;
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mscope-plugin(scope): getDeclaredVariables(): localConstant[0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:5:3]
 [2m4[0m │ function topLevelFunction(param) {
 [2m5[0m │   const localConstant = topLevelConstant + param;
   · [38;2;246;87;248m  ───────────────────────────────────────────────[0m
 [2m6[0m │   return function innerFunction() {
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mscope-plugin(scope): isGlobalReference(localConstant): false[0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:5:9]
 [2m4[0m │ function topLevelFunction(param) {
 [2m5[0m │   const localConstant = topLevelConstant + param;
   · [38;2;246;87;248m        ─────────────[0m
 [2m6[0m │   return function innerFunction() {
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mscope-plugin(scope): isGlobalReference(topLevelConstant): false[0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:5:25]
 [2m4[0m │ function topLevelFunction(param) {
 [2m5[0m │   const localConstant = topLevelConstant + param;
   · [38;2;246;87;248m                        ────────────────[0m
 [2m6[0m │   return function innerFunction() {
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mscope-plugin(scope): isGlobalReference(param): false[0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:5:44]
 [2m4[0m │ function topLevelFunction(param) {
 [2m5[0m │   const localConstant = topLevelConstant + param;
   · [38;2;246;87;248m                                           ─────[0m
 [2m6[0m │   return function innerFunction() {
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mscope-plugin(scope): isGlobalReference(innerFunction): false[0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:6:19]
 [2m5[0m │   const localConstant = topLevelConstant + param;
 [2m6[0m │   return function innerFunction() {
   · [38;2;246;87;248m                  ─────────────[0m
 [2m7[0m │     return localConstant + Math.PI;
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mscope-plugin(scope): isGlobalReference(localConstant): false[0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:7:12]
 [2m6[0m │   return function innerFunction() {
 [2m7[0m │     return localConstant + Math.PI;
   · [38;2;246;87;248m           ─────────────[0m
 [2m8[0m │   };
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mscope-plugin(scope): isGlobalReference(Math): true[0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:7:28]
 [2m6[0m │   return function innerFunction() {
 [2m7[0m │     return localConstant + Math.PI;
   · [38;2;246;87;248m                           ────[0m
 [2m8[0m │   };
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mscope-plugin(scope): isGlobalReference(PI): false[0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:7:33]
 [2m6[0m │   return function innerFunction() {
 [2m7[0m │     return localConstant + Math.PI;
   · [38;2;246;87;248m                                ──[0m
 [2m8[0m │   };
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mscope-plugin(scope): getDeclaredVariables(): topLevelExport[0m
    ╭─[[38;2;92;157;255;1mfiles/index.js[0m:11:8]
 [2m10[0m │ 
 [2m11[0m │ export const topLevelExport = topLevelFunction(2);
    · [38;2;246;87;248m       ───────────────────────────────────────────[0m
    ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mscope-plugin(scope): isGlobalReference(topLevelExport): false[0m
    ╭─[[38;2;92;157;255;1mfiles/index.js[0m:11:14]
 [2m10[0m │ 
 [2m11[0m │ export const topLevelExport = topLevelFunction(2);
    · [38;2;246;87;248m             ──────────────[0m
    ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mscope-plugin(scope): isGlobalReference(topLevelFunction): false[0m
    ╭─[[38;2;92;157;255;1mfiles/index.js[0m:11:31]
 [2m10[0m │ 
 [2m11[0m │ export const topLevelExport = topLevelFunction(2);
    · [38;2;246;87;248m                              ────────────────[0m
    ╰────

Found 0 warnings and 33 errors.
Finished in Xms on 2 files using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
