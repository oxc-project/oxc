# Exit code
1

# stdout
```
  x program-only(create-once): after hook #5: visited Program: true
   ,-[files/index.ts:1:1]
 1 | // Leading comment
   : ^
 2 | import { a } from "./a";
   `----

  x program-only(create-once): createOnce Program #1
  | node is sourceCode.ast: true
  | parent: null
  | ancestors: 0
  | body: ImportDeclaration, ExportNamedDeclaration
  | comments: 2
  | tokens: 39
   ,-[files/index.ts:1:1]
 1 | ,-> // Leading comment
 2 | |   import { a } from "./a";
 3 | |   
 4 | |   /* Block comment */
 5 | |   export function f(x: number): number {
 6 | |     const y = { z: [x, a] };
 7 | |     return y.z.length;
 8 | `-> }
   `----

  x program-only(enter-and-exit): Program enter #0
  | node is sourceCode.ast: true
  | parent: null
  | ancestors: 0
  | body: ImportDeclaration, ExportNamedDeclaration
  | comments: 2
  | tokens: 39
   ,-[files/index.ts:1:1]
 1 | ,-> // Leading comment
 2 | |   import { a } from "./a";
 3 | |   
 4 | |   /* Block comment */
 5 | |   export function f(x: number): number {
 6 | |     const y = { z: [x, a] };
 7 | |     return y.z.length;
 8 | `-> }
   `----

  x program-only(enter-and-exit): Program exit #3
  | node is sourceCode.ast: true
  | parent: null
  | ancestors: 0
  | body: ImportDeclaration, ExportNamedDeclaration
  | comments: 2
  | tokens: 39
   ,-[files/index.ts:1:1]
 1 | ,-> // Leading comment
 2 | |   import { a } from "./a";
 3 | |   
 4 | |   /* Block comment */
 5 | |   export function f(x: number): number {
 6 | |     const y = { z: [x, a] };
 7 | |     return y.z.length;
 8 | `-> }
   `----

  x program-only(exit-only): Program:exit only #4
  | node is sourceCode.ast: true
  | parent: null
  | ancestors: 0
  | body: ImportDeclaration, ExportNamedDeclaration
  | comments: 2
  | tokens: 39
   ,-[files/index.ts:1:1]
 1 | ,-> // Leading comment
 2 | |   import { a } from "./a";
 3 | |   
 4 | |   /* Block comment */
 5 | |   export function f(x: number): number {
 6 | |     const y = { z: [x, a] };
 7 | |     return y.z.length;
 8 | `-> }
   `----

  x program-only(selector): Program[sourceType='module'] #2
  | node is sourceCode.ast: true
  | parent: null
  | ancestors: 0
  | body: ImportDeclaration, ExportNamedDeclaration
  | comments: 2
  | tokens: 39
   ,-[files/index.ts:1:1]
 1 | ,-> // Leading comment
 2 | |   import { a } from "./a";
 3 | |   
 4 | |   /* Block comment */
 5 | |   export function f(x: number): number {
 6 | |     const y = { z: [x, a] };
 7 | |     return y.z.length;
 8 | `-> }
   `----

Found 0 warnings and 6 errors.
Finished in Xms on 1 file with 4 rules using X threads.
```

# stderr
```
```
