# Exit code
1

# stdout
```
  x scope-manager-plugin(scope): File has 12 scopes:
  | - global
  | - module
  | - function(topLevelFunction)
  | - block
  | - tsModule(TopLevelModule)
  | - type(GenericInterface)
  | - class(TestClass)
  | - class-static-block
  | - function
  | - function
  | - block
  | - block
   ,-[files/index.ts:1:1]
 1 | const { a, b, c } = {};
   : ^
 2 | 
   `----

  x scope-manager-plugin(scope): VariableDeclaration declares 3 variables: a, b, c.
   ,-[files/index.ts:1:1]
 1 | const { a, b, c } = {};
   : ^^^^^^^^^^^^^^^^^^^^^^^
 2 | 
   `----

  x scope-manager-plugin(scope): topLevelFunction has 3 local variables: arguments, param, localVar. Child scopes: 1.
    ,-[files/index.ts:7:1]
  6 |     
  7 | ,-> function topLevelFunction(param: number) {
  8 | |     const localVar = param + x;
  9 | |     {
 10 | |       const deepestVar = y + localVar;
 11 | |       return deepestVar;
 12 | |     }
 13 | |     return localVar;
 14 | `-> }
 15 |     
    `----

  x scope-manager-plugin(scope): TopLevelModule has 3 local variables: ConcreteInterface, GenericInterface, x. Child scopes: 1.
    ,-[files/index.ts:16:8]
 15 |     
 16 | ,-> export module TopLevelModule {
 17 | |     interface ConcreteInterface {
 18 | |       concreteVar: number;
 19 | |     }
 20 | |     export interface GenericInterface<T> extends ConcreteInterface {
 21 | |       genericVar: T;
 22 | |     }
 23 | |     export const x: GenericInterface<string> = {
 24 | |       concreteVar: 42,
 25 | |       genericVar: "string",
 26 | |     };
 27 | `-> }
 28 |     
    `----

  x scope-manager-plugin(scope): TestClass static block has 2 local variables: privateVar, arrowFunc. Child scopes: 1.
    ,-[files/index.ts:37:3]
 36 |       #privateVar: string;
 37 | ,->   static {
 38 | |       const privateVar = "private";
 39 | |       this.prototype.#privateVar = arrowFunc(privateVar);
 40 | |   
 41 | |       const arrowFunc = (param: string) => {
 42 | |         const arrowVar = param;
 43 | |         return arrowVar + y;
 44 | |       };
 45 | `->   }
 46 |     
    `----

  x scope-manager-plugin(scope): LabeledStatement's block has 1 local variables: blockVar. Child scopes: 0.
    ,-[files/index.ts:54:1]
 53 |     
 54 | ,-> label: {
 55 | |     const blockVar = "block";
 56 | |     console.log(blockVar);
 57 | `-> }
 58 |     
    `----

Found 0 warnings and 6 errors.
Finished in Xms on 1 file using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
