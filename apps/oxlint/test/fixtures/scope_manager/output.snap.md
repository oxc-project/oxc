# Exit code
1

# stdout
```
  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mscope-manager-plugin(scope): File has 12 scopes:
  [38;2;225;80;80;1m│[0m - global
  [38;2;225;80;80;1m│[0m - module
  [38;2;225;80;80;1m│[0m - function(topLevelFunction)
  [38;2;225;80;80;1m│[0m - block
  [38;2;225;80;80;1m│[0m - tsModule(TopLevelModule)
  [38;2;225;80;80;1m│[0m - type(GenericInterface)
  [38;2;225;80;80;1m│[0m - class(TestClass)
  [38;2;225;80;80;1m│[0m - class-static-block
  [38;2;225;80;80;1m│[0m - function
  [38;2;225;80;80;1m│[0m - function
  [38;2;225;80;80;1m│[0m - block
  [38;2;225;80;80;1m│[0m - block[0m
   ╭─[[38;2;92;157;255;1mfiles/index.ts[0m:1:1]
 [2m1[0m │ const { a, b, c } = {};
   · [38;2;246;87;248m▲[0m
 [2m2[0m │ 
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mscope-manager-plugin(scope): VariableDeclaration declares 3 variables: a, b, c.[0m
   ╭─[[38;2;92;157;255;1mfiles/index.ts[0m:1:1]
 [2m1[0m │ const { a, b, c } = {};
   · [38;2;246;87;248m───────────────────────[0m
 [2m2[0m │ 
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mscope-manager-plugin(scope): topLevelFunction has 3 local variables: arguments, param, localVar. Child scopes: 1.[0m
    ╭─[[38;2;92;157;255;1mfiles/index.ts[0m:7:1]
 [2m 6[0m │     
 [2m 7[0m │ [38;2;246;87;248m╭[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m function topLevelFunction(param: number) {
 [2m 8[0m │ [38;2;246;87;248m│[0m     const localVar = param + x;
 [2m 9[0m │ [38;2;246;87;248m│[0m     {
 [2m10[0m │ [38;2;246;87;248m│[0m       const deepestVar = y + localVar;
 [2m11[0m │ [38;2;246;87;248m│[0m       return deepestVar;
 [2m12[0m │ [38;2;246;87;248m│[0m     }
 [2m13[0m │ [38;2;246;87;248m│[0m     return localVar;
 [2m14[0m │ [38;2;246;87;248m╰[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m }
 [2m15[0m │     
    ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mscope-manager-plugin(scope): TopLevelModule has 3 local variables: ConcreteInterface, GenericInterface, x. Child scopes: 1.[0m
    ╭─[[38;2;92;157;255;1mfiles/index.ts[0m:16:8]
 [2m15[0m │     
 [2m16[0m │ [38;2;246;87;248m╭[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m export module TopLevelModule {
 [2m17[0m │ [38;2;246;87;248m│[0m     interface ConcreteInterface {
 [2m18[0m │ [38;2;246;87;248m│[0m       concreteVar: number;
 [2m19[0m │ [38;2;246;87;248m│[0m     }
 [2m20[0m │ [38;2;246;87;248m│[0m     export interface GenericInterface<T> extends ConcreteInterface {
 [2m21[0m │ [38;2;246;87;248m│[0m       genericVar: T;
 [2m22[0m │ [38;2;246;87;248m│[0m     }
 [2m23[0m │ [38;2;246;87;248m│[0m     export const x: GenericInterface<string> = {
 [2m24[0m │ [38;2;246;87;248m│[0m       concreteVar: 42,
 [2m25[0m │ [38;2;246;87;248m│[0m       genericVar: "string",
 [2m26[0m │ [38;2;246;87;248m│[0m     };
 [2m27[0m │ [38;2;246;87;248m╰[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m }
 [2m28[0m │     
    ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mscope-manager-plugin(scope): TestClass static block has 2 local variables: privateVar, arrowFunc. Child scopes: 1.[0m
    ╭─[[38;2;92;157;255;1mfiles/index.ts[0m:37:3]
 [2m36[0m │       #privateVar: string;
 [2m37[0m │ [38;2;246;87;248m╭[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m   static {
 [2m38[0m │ [38;2;246;87;248m│[0m       const privateVar = "private";
 [2m39[0m │ [38;2;246;87;248m│[0m       this.prototype.#privateVar = arrowFunc(privateVar);
 [2m40[0m │ [38;2;246;87;248m│[0m   
 [2m41[0m │ [38;2;246;87;248m│[0m       const arrowFunc = (param: string) => {
 [2m42[0m │ [38;2;246;87;248m│[0m         const arrowVar = param;
 [2m43[0m │ [38;2;246;87;248m│[0m         return arrowVar + y;
 [2m44[0m │ [38;2;246;87;248m│[0m       };
 [2m45[0m │ [38;2;246;87;248m╰[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m   }
 [2m46[0m │     
    ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mscope-manager-plugin(scope): LabeledStatement's block has 1 local variables: blockVar. Child scopes: 0.[0m
    ╭─[[38;2;92;157;255;1mfiles/index.ts[0m:54:1]
 [2m53[0m │     
 [2m54[0m │ [38;2;246;87;248m╭[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m label: {
 [2m55[0m │ [38;2;246;87;248m│[0m     const blockVar = "block";
 [2m56[0m │ [38;2;246;87;248m│[0m     console.log(blockVar);
 [2m57[0m │ [38;2;246;87;248m╰[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m }
 [2m58[0m │     
    ╰────

Found 0 warnings and 6 errors.
Finished in Xms on 1 file with 1 rules using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
