# Exit code
1

# stdout
```
  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtest-plugin(is-space-between):
  [38;2;225;80;80;1m│[0m isSpaceBetween(openingElement, closingElement): false
  [38;2;225;80;80;1m│[0m isSpaceBetweenTokens(openingElement, closingElement): false
  [38;2;225;80;80;1m│[0m isSpaceBetween(closingElement, openingElement): false
  [38;2;225;80;80;1m│[0m isSpaceBetweenTokens(closingElement, openingElement): false[0m
   ╭─[[38;2;92;157;255;1mfiles/index.jsx[0m:1:1]
 [2m1[0m │ <Foo>aaa</Foo>;
   · [38;2;246;87;248m──────────────[0m
 [2m2[0m │ 
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtest-plugin(is-space-between):
  [38;2;225;80;80;1m│[0m isSpaceBetween(openingElement, closingElement): false
  [38;2;225;80;80;1m│[0m isSpaceBetweenTokens(openingElement, closingElement): true
  [38;2;225;80;80;1m│[0m isSpaceBetween(closingElement, openingElement): false
  [38;2;225;80;80;1m│[0m isSpaceBetweenTokens(closingElement, openingElement): true[0m
   ╭─[[38;2;92;157;255;1mfiles/index.jsx[0m:3:1]
 [2m2[0m │ 
 [2m3[0m │ <Bar>b c</Bar>;
   · [38;2;246;87;248m──────────────[0m
 [2m4[0m │ 
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtest-plugin(is-space-between):
  [38;2;225;80;80;1m│[0m isSpaceBetween(openingElement, closingElement): false
  [38;2;225;80;80;1m│[0m isSpaceBetweenTokens(openingElement, closingElement): true
  [38;2;225;80;80;1m│[0m isSpaceBetween(closingElement, openingElement): false
  [38;2;225;80;80;1m│[0m isSpaceBetweenTokens(closingElement, openingElement): true[0m
   ╭─[[38;2;92;157;255;1mfiles/index.jsx[0m:6:1]
 [2m5[0m │     // prettier-ignore
 [2m6[0m │ [38;2;246;87;248m╭[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m <Qux>d
 [2m7[0m │ [38;2;246;87;248m╰[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m e</Qux>;
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtest-plugin(is-space-between):
  [38;2;225;80;80;1m│[0m isSpaceBetween(left, right): false
  [38;2;225;80;80;1m│[0m isSpaceBetweenTokens(left, right): false
  [38;2;225;80;80;1m│[0m isSpaceBetween(right, left): false
  [38;2;225;80;80;1m│[0m isSpaceBetweenTokens(right, left): false
  [38;2;225;80;80;1m│[0m isSpaceBetween(left, node): false
  [38;2;225;80;80;1m│[0m isSpaceBetweenTokens(left, node): false
  [38;2;225;80;80;1m│[0m isSpaceBetween(node, left): false
  [38;2;225;80;80;1m│[0m isSpaceBetweenTokens(node, left): false
  [38;2;225;80;80;1m│[0m isSpaceBetween(right, node): false
  [38;2;225;80;80;1m│[0m isSpaceBetweenTokens(right, node): false
  [38;2;225;80;80;1m│[0m isSpaceBetween(node, right): false
  [38;2;225;80;80;1m│[0m isSpaceBetweenTokens(node, right): false[0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:2:1]
 [2m1[0m │ // prettier-ignore
 [2m2[0m │ noSpace=1;
   · [38;2;246;87;248m─────────[0m
 [2m3[0m │ 
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtest-plugin(is-space-between):
  [38;2;225;80;80;1m│[0m isSpaceBetween(leftExtended, right): false
  [38;2;225;80;80;1m│[0m isSpaceBetweenTokens(leftExtended, right): false
  [38;2;225;80;80;1m│[0m isSpaceBetween(right, leftExtended): false
  [38;2;225;80;80;1m│[0m isSpaceBetweenTokens(right, leftExtended): false[0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:2:1]
 [2m1[0m │ // prettier-ignore
 [2m2[0m │ noSpace=1;
   · [38;2;246;87;248m─────────[0m
 [2m3[0m │ 
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtest-plugin(is-space-between):
  [38;2;225;80;80;1m│[0m isSpaceBetween(left, right): true
  [38;2;225;80;80;1m│[0m isSpaceBetweenTokens(left, right): true
  [38;2;225;80;80;1m│[0m isSpaceBetween(right, left): true
  [38;2;225;80;80;1m│[0m isSpaceBetweenTokens(right, left): true
  [38;2;225;80;80;1m│[0m isSpaceBetween(left, node): false
  [38;2;225;80;80;1m│[0m isSpaceBetweenTokens(left, node): false
  [38;2;225;80;80;1m│[0m isSpaceBetween(node, left): false
  [38;2;225;80;80;1m│[0m isSpaceBetweenTokens(node, left): false
  [38;2;225;80;80;1m│[0m isSpaceBetween(right, node): false
  [38;2;225;80;80;1m│[0m isSpaceBetweenTokens(right, node): false
  [38;2;225;80;80;1m│[0m isSpaceBetween(node, right): false
  [38;2;225;80;80;1m│[0m isSpaceBetweenTokens(node, right): false[0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:5:1]
 [2m4[0m │ // prettier-ignore
 [2m5[0m │ singleSpaceBefore =2;
   · [38;2;246;87;248m────────────────────[0m
 [2m6[0m │ 
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtest-plugin(is-space-between):
  [38;2;225;80;80;1m│[0m isSpaceBetween(left, right): true
  [38;2;225;80;80;1m│[0m isSpaceBetweenTokens(left, right): true
  [38;2;225;80;80;1m│[0m isSpaceBetween(right, left): true
  [38;2;225;80;80;1m│[0m isSpaceBetweenTokens(right, left): true
  [38;2;225;80;80;1m│[0m isSpaceBetween(left, node): false
  [38;2;225;80;80;1m│[0m isSpaceBetweenTokens(left, node): false
  [38;2;225;80;80;1m│[0m isSpaceBetween(node, left): false
  [38;2;225;80;80;1m│[0m isSpaceBetweenTokens(node, left): false
  [38;2;225;80;80;1m│[0m isSpaceBetween(right, node): false
  [38;2;225;80;80;1m│[0m isSpaceBetweenTokens(right, node): false
  [38;2;225;80;80;1m│[0m isSpaceBetween(node, right): false
  [38;2;225;80;80;1m│[0m isSpaceBetweenTokens(node, right): false[0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:8:1]
 [2m7[0m │ // prettier-ignore
 [2m8[0m │ singleSpaceAfter= 3;
   · [38;2;246;87;248m───────────────────[0m
 [2m9[0m │ 
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtest-plugin(is-space-between):
  [38;2;225;80;80;1m│[0m isSpaceBetween(left, right): true
  [38;2;225;80;80;1m│[0m isSpaceBetweenTokens(left, right): true
  [38;2;225;80;80;1m│[0m isSpaceBetween(right, left): true
  [38;2;225;80;80;1m│[0m isSpaceBetweenTokens(right, left): true
  [38;2;225;80;80;1m│[0m isSpaceBetween(left, node): false
  [38;2;225;80;80;1m│[0m isSpaceBetweenTokens(left, node): false
  [38;2;225;80;80;1m│[0m isSpaceBetween(node, left): false
  [38;2;225;80;80;1m│[0m isSpaceBetweenTokens(node, left): false
  [38;2;225;80;80;1m│[0m isSpaceBetween(right, node): false
  [38;2;225;80;80;1m│[0m isSpaceBetweenTokens(right, node): false
  [38;2;225;80;80;1m│[0m isSpaceBetween(node, right): false
  [38;2;225;80;80;1m│[0m isSpaceBetweenTokens(node, right): false[0m
    ╭─[[38;2;92;157;255;1mfiles/index.js[0m:11:1]
 [2m10[0m │ // prettier-ignore
 [2m11[0m │ multipleSpaces   =   4;
    · [38;2;246;87;248m──────────────────────[0m
 [2m12[0m │ 
    ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtest-plugin(is-space-between):
  [38;2;225;80;80;1m│[0m isSpaceBetween(left, right): true
  [38;2;225;80;80;1m│[0m isSpaceBetweenTokens(left, right): true
  [38;2;225;80;80;1m│[0m isSpaceBetween(right, left): true
  [38;2;225;80;80;1m│[0m isSpaceBetweenTokens(right, left): true
  [38;2;225;80;80;1m│[0m isSpaceBetween(left, node): false
  [38;2;225;80;80;1m│[0m isSpaceBetweenTokens(left, node): false
  [38;2;225;80;80;1m│[0m isSpaceBetween(node, left): false
  [38;2;225;80;80;1m│[0m isSpaceBetweenTokens(node, left): false
  [38;2;225;80;80;1m│[0m isSpaceBetween(right, node): false
  [38;2;225;80;80;1m│[0m isSpaceBetweenTokens(right, node): false
  [38;2;225;80;80;1m│[0m isSpaceBetween(node, right): false
  [38;2;225;80;80;1m│[0m isSpaceBetweenTokens(node, right): false[0m
    ╭─[[38;2;92;157;255;1mfiles/index.js[0m:14:1]
 [2m13[0m │     // prettier-ignore
 [2m14[0m │ [38;2;246;87;248m╭[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m newlineBefore=
 [2m15[0m │ [38;2;246;87;248m╰[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m 5;
 [2m16[0m │     
    ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtest-plugin(is-space-between):
  [38;2;225;80;80;1m│[0m isSpaceBetween(left, right): true
  [38;2;225;80;80;1m│[0m isSpaceBetweenTokens(left, right): true
  [38;2;225;80;80;1m│[0m isSpaceBetween(right, left): true
  [38;2;225;80;80;1m│[0m isSpaceBetweenTokens(right, left): true
  [38;2;225;80;80;1m│[0m isSpaceBetween(left, node): false
  [38;2;225;80;80;1m│[0m isSpaceBetweenTokens(left, node): false
  [38;2;225;80;80;1m│[0m isSpaceBetween(node, left): false
  [38;2;225;80;80;1m│[0m isSpaceBetweenTokens(node, left): false
  [38;2;225;80;80;1m│[0m isSpaceBetween(right, node): false
  [38;2;225;80;80;1m│[0m isSpaceBetweenTokens(right, node): false
  [38;2;225;80;80;1m│[0m isSpaceBetween(node, right): false
  [38;2;225;80;80;1m│[0m isSpaceBetweenTokens(node, right): false[0m
    ╭─[[38;2;92;157;255;1mfiles/index.js[0m:18:1]
 [2m17[0m │     // prettier-ignore
 [2m18[0m │ [38;2;246;87;248m╭[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m newlineAfter
 [2m19[0m │ [38;2;246;87;248m╰[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m =6;
 [2m20[0m │     
    ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtest-plugin(is-space-between):
  [38;2;225;80;80;1m│[0m isSpaceBetween(left, right): true
  [38;2;225;80;80;1m│[0m isSpaceBetweenTokens(left, right): true
  [38;2;225;80;80;1m│[0m isSpaceBetween(right, left): true
  [38;2;225;80;80;1m│[0m isSpaceBetweenTokens(right, left): true
  [38;2;225;80;80;1m│[0m isSpaceBetween(left, node): false
  [38;2;225;80;80;1m│[0m isSpaceBetweenTokens(left, node): false
  [38;2;225;80;80;1m│[0m isSpaceBetween(node, left): false
  [38;2;225;80;80;1m│[0m isSpaceBetweenTokens(node, left): false
  [38;2;225;80;80;1m│[0m isSpaceBetween(right, node): false
  [38;2;225;80;80;1m│[0m isSpaceBetweenTokens(right, node): false
  [38;2;225;80;80;1m│[0m isSpaceBetween(node, right): false
  [38;2;225;80;80;1m│[0m isSpaceBetweenTokens(node, right): false[0m
    ╭─[[38;2;92;157;255;1mfiles/index.js[0m:22:1]
 [2m21[0m │ // prettier-ignore
 [2m22[0m │ nested = 7 + 8;
    · [38;2;246;87;248m──────────────[0m
 [2m23[0m │ 
    ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtest-plugin(is-space-between):
  [38;2;225;80;80;1m│[0m isSpaceBetween(node, binaryLeft): false
  [38;2;225;80;80;1m│[0m isSpaceBetweenTokens(node, binaryLeft): false
  [38;2;225;80;80;1m│[0m isSpaceBetween(binaryLeft, node): false
  [38;2;225;80;80;1m│[0m isSpaceBetweenTokens(binaryLeft, node): false[0m
    ╭─[[38;2;92;157;255;1mfiles/index.js[0m:22:1]
 [2m21[0m │ // prettier-ignore
 [2m22[0m │ nested = 7 + 8;
    · [38;2;246;87;248m──────────────[0m
 [2m23[0m │ 
    ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtest-plugin(is-space-between):
  [38;2;225;80;80;1m│[0m isSpaceBetween(beforeString, afterString): false
  [38;2;225;80;80;1m│[0m isSpaceBetweenTokens(beforeString, afterString): false
  [38;2;225;80;80;1m│[0m isSpaceBetween(afterString, beforeString): false
  [38;2;225;80;80;1m│[0m isSpaceBetweenTokens(afterString, beforeString): false[0m
    ╭─[[38;2;92;157;255;1mfiles/index.js[0m:25:1]
 [2m24[0m │ // prettier-ignore
 [2m25[0m │ beforeString," ",afterString;
    · [38;2;246;87;248m────────────────────────────[0m
    ╰────

Found 0 warnings and 13 errors.
Finished in Xms on 2 files using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
