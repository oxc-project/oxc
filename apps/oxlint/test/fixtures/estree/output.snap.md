# Exit code
1

# stdout
```
  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mestree-check(check): program:
  [38;2;225;80;80;1m│[0m start/end: [37,281]
  [38;2;225;80;80;1m│[0m range: [37,281]
  [38;2;225;80;80;1m│[0m loc: [{"start":{"line":4,"column":0},"end":{"line":16,"column":0}}][0m
    ╭─[[38;2;92;157;255;1mfiles/index.ts[0m:4:1]
 [2m 3[0m │     // All `Identifier`s
 [2m 4[0m │ [38;2;246;87;248m╭[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m let a = { x: y };
 [2m 5[0m │ [38;2;246;87;248m│[0m   
 [2m 6[0m │ [38;2;246;87;248m│[0m   // No `ParenthesizedExpression`s in AST
 [2m 7[0m │ [38;2;246;87;248m│[0m   // prettier-ignore
 [2m 8[0m │ [38;2;246;87;248m│[0m   const b = (x * ((('str' + ((123))))));
 [2m 9[0m │ [38;2;246;87;248m│[0m   
 [2m10[0m │ [38;2;246;87;248m│[0m   // TS syntax
 [2m11[0m │ [38;2;246;87;248m│[0m   type T = string;
 [2m12[0m │ [38;2;246;87;248m│[0m   
 [2m13[0m │ [38;2;246;87;248m│[0m   // No `TSParenthesizedType`s in AST
 [2m14[0m │ [38;2;246;87;248m│[0m   // prettier-ignore
 [2m15[0m │ [38;2;246;87;248m╰[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m type U = (((((string)) | ((number)))));
    ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mestree-check(check): ident "a":
  [38;2;225;80;80;1m│[0m start/end: [41,42]
  [38;2;225;80;80;1m│[0m range: [41,42]
  [38;2;225;80;80;1m│[0m loc: [{"start":{"line":4,"column":4},"end":{"line":4,"column":5}}][0m
   ╭─[[38;2;92;157;255;1mfiles/index.ts[0m:4:5]
 [2m3[0m │ // All `Identifier`s
 [2m4[0m │ let a = { x: y };
   · [38;2;246;87;248m    ─[0m
 [2m5[0m │ 
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mestree-check(check): ident "x":
  [38;2;225;80;80;1m│[0m start/end: [47,48]
  [38;2;225;80;80;1m│[0m range: [47,48]
  [38;2;225;80;80;1m│[0m loc: [{"start":{"line":4,"column":10},"end":{"line":4,"column":11}}][0m
   ╭─[[38;2;92;157;255;1mfiles/index.ts[0m:4:11]
 [2m3[0m │ // All `Identifier`s
 [2m4[0m │ let a = { x: y };
   · [38;2;246;87;248m          ─[0m
 [2m5[0m │ 
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mestree-check(check): ident "y":
  [38;2;225;80;80;1m│[0m start/end: [50,51]
  [38;2;225;80;80;1m│[0m range: [50,51]
  [38;2;225;80;80;1m│[0m loc: [{"start":{"line":4,"column":13},"end":{"line":4,"column":14}}][0m
   ╭─[[38;2;92;157;255;1mfiles/index.ts[0m:4:14]
 [2m3[0m │ // All `Identifier`s
 [2m4[0m │ let a = { x: y };
   · [38;2;246;87;248m             ─[0m
 [2m5[0m │ 
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mestree-check(check): ident "b":
  [38;2;225;80;80;1m│[0m start/end: [121,122]
  [38;2;225;80;80;1m│[0m range: [121,122]
  [38;2;225;80;80;1m│[0m loc: [{"start":{"line":8,"column":6},"end":{"line":8,"column":7}}][0m
   ╭─[[38;2;92;157;255;1mfiles/index.ts[0m:8:7]
 [2m7[0m │ // prettier-ignore
 [2m8[0m │ const b = (x * ((('str' + ((123))))));
   · [38;2;246;87;248m      ─[0m
 [2m9[0m │ 
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mestree-check(check): ident "x":
  [38;2;225;80;80;1m│[0m start/end: [126,127]
  [38;2;225;80;80;1m│[0m range: [126,127]
  [38;2;225;80;80;1m│[0m loc: [{"start":{"line":8,"column":11},"end":{"line":8,"column":12}}][0m
   ╭─[[38;2;92;157;255;1mfiles/index.ts[0m:8:12]
 [2m7[0m │ // prettier-ignore
 [2m8[0m │ const b = (x * ((('str' + ((123))))));
   · [38;2;246;87;248m           ─[0m
 [2m9[0m │ 
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mestree-check(check): ident "T":
  [38;2;225;80;80;1m│[0m start/end: [173,174]
  [38;2;225;80;80;1m│[0m range: [173,174]
  [38;2;225;80;80;1m│[0m loc: [{"start":{"line":11,"column":5},"end":{"line":11,"column":6}}][0m
    ╭─[[38;2;92;157;255;1mfiles/index.ts[0m:11:6]
 [2m10[0m │ // TS syntax
 [2m11[0m │ type T = string;
    · [38;2;246;87;248m     ─[0m
 [2m12[0m │ 
    ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mestree-check(check): ident "U":
  [38;2;225;80;80;1m│[0m start/end: [246,247]
  [38;2;225;80;80;1m│[0m range: [246,247]
  [38;2;225;80;80;1m│[0m loc: [{"start":{"line":15,"column":5},"end":{"line":15,"column":6}}][0m
    ╭─[[38;2;92;157;255;1mfiles/index.ts[0m:15:6]
 [2m14[0m │ // prettier-ignore
 [2m15[0m │ type U = (((((string)) | ((number)))));
    · [38;2;246;87;248m     ─[0m
    ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mestree-check(check): Visited nodes:
  [38;2;225;80;80;1m│[0m * Program
  [38;2;225;80;80;1m│[0m * VariableDeclaration: let
  [38;2;225;80;80;1m│[0m * VariableDeclarator: (init: ObjectExpression)
  [38;2;225;80;80;1m│[0m * Identifier: a
  [38;2;225;80;80;1m│[0m * ObjectExpression
  [38;2;225;80;80;1m│[0m * Identifier: x
  [38;2;225;80;80;1m│[0m * Identifier: y
  [38;2;225;80;80;1m│[0m * VariableDeclaration:exit: let
  [38;2;225;80;80;1m│[0m * VariableDeclaration: const
  [38;2;225;80;80;1m│[0m * VariableDeclarator: (init: BinaryExpression)
  [38;2;225;80;80;1m│[0m * Identifier: b
  [38;2;225;80;80;1m│[0m * BinaryExpression: * (right: BinaryExpression)
  [38;2;225;80;80;1m│[0m * Identifier: x
  [38;2;225;80;80;1m│[0m * BinaryExpression: + (right: Literal)
  [38;2;225;80;80;1m│[0m * Literal: str
  [38;2;225;80;80;1m│[0m * Literal: 123
  [38;2;225;80;80;1m│[0m * VariableDeclaration:exit: const
  [38;2;225;80;80;1m│[0m * TSTypeAliasDeclaration: (typeAnnotation: TSStringKeyword)
  [38;2;225;80;80;1m│[0m * Identifier: T
  [38;2;225;80;80;1m│[0m * TSStringKeyword
  [38;2;225;80;80;1m│[0m * TSTypeAliasDeclaration:exit: (typeAnnotation: TSStringKeyword)
  [38;2;225;80;80;1m│[0m * TSTypeAliasDeclaration: (typeAnnotation: TSUnionType)
  [38;2;225;80;80;1m│[0m * Identifier: U
  [38;2;225;80;80;1m│[0m * TSUnionType: (types: TSStringKeyword, TSNumberKeyword)
  [38;2;225;80;80;1m│[0m * TSStringKeyword
  [38;2;225;80;80;1m│[0m * TSNumberKeyword
  [38;2;225;80;80;1m│[0m * TSUnionType:exit: (types: TSStringKeyword, TSNumberKeyword)
  [38;2;225;80;80;1m│[0m * TSTypeAliasDeclaration:exit: (typeAnnotation: TSUnionType)
  [38;2;225;80;80;1m│[0m * Program:exit[0m
    ╭─[[38;2;92;157;255;1mfiles/index.ts[0m:4:1]
 [2m 3[0m │     // All `Identifier`s
 [2m 4[0m │ [38;2;246;87;248m╭[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m let a = { x: y };
 [2m 5[0m │ [38;2;246;87;248m│[0m   
 [2m 6[0m │ [38;2;246;87;248m│[0m   // No `ParenthesizedExpression`s in AST
 [2m 7[0m │ [38;2;246;87;248m│[0m   // prettier-ignore
 [2m 8[0m │ [38;2;246;87;248m│[0m   const b = (x * ((('str' + ((123))))));
 [2m 9[0m │ [38;2;246;87;248m│[0m   
 [2m10[0m │ [38;2;246;87;248m│[0m   // TS syntax
 [2m11[0m │ [38;2;246;87;248m│[0m   type T = string;
 [2m12[0m │ [38;2;246;87;248m│[0m   
 [2m13[0m │ [38;2;246;87;248m│[0m   // No `TSParenthesizedType`s in AST
 [2m14[0m │ [38;2;246;87;248m│[0m   // prettier-ignore
 [2m15[0m │ [38;2;246;87;248m╰[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m type U = (((((string)) | ((number)))));
    ╰────

Found 0 warnings and 9 errors.
Finished in Xms on 1 file with 1 rules using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
