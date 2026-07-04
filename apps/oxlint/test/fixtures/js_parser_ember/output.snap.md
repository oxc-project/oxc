# Exit code
1

# stdout
```
  x ember(template-no-let-reference): update-able variables are not supported in templates, reference a const variable
   ,-[files/late-declaration.gjs:7:10]
 6 |   <template>
 7 |     <p>{{lateMessage}}</p>
   :          ^^^^^^^^^^^
 8 |   </template>
   `----

  x ember(template-no-let-reference): update-able variables are not supported in templates, reference a const variable
   ,-[files/let-reference.gjs:5:8]
 4 | <template>
 5 |   <p>{{message}}</p>
   :        ^^^^^^^
 6 |   <p>{{greeting}}</p>
   `----

  x eslint(no-unused-vars): Variable 'unusedLocal' is declared but never used. Unused variables should start with a '_'.
   ,-[files/native-rules.gts:2:7]
 1 | const styleClass = "primary";
 2 | const unusedLocal = "never used";
   :       ^^^^^|^^^^^
   :            `-- 'unusedLocal' is declared here
 3 | 
   `----
  help: Consider removing this declaration.

  x eslint(no-debugger): `debugger` statement is not allowed
   ,-[files/native-rules.gts:8:5]
 7 |   isZero(value: number): boolean {
 8 |     debugger;
   :     ^^^^^^^^^
 9 |     return value == 0;
   `----
  help: Remove the debugger statement

  x eslint(eqeqeq): Expected === and instead saw ==
    ,-[files/native-rules.gts:9:18]
  8 |     debugger;
  9 |     return value == 0;
    :                  ^^
 10 |   }
    `----
  help: Prefer === operator

  x eslint(class-methods-use-this): Expected method `farewell` to have this.
    ,-[files/this-in-template.gjs:16:3]
 15 | 
 16 |   farewell() {
    :   ^^^^^^^^
 17 |     return <template><p>goodbye</p></template>;
    `----
  help: Consider converting method `farewell` to a static method.

  x ember(template-no-let-reference): update-able variables are not supported in templates, reference a const variable
   ,-[files/typescript.gts:4:11]
 3 | <template>
 4 |   <span>{{count}}</span>
   :           ^^^^^
 5 | </template>
   `----

Found 0 warnings and 7 errors.
Finished in Xms on 7 files with 98 rules using X threads.
```

# stderr
```
```
