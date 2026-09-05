# Exit code
1

# stdout
```
  ! eslint(no-unused-vars): Variable 'answer' is declared but never used. Unused variables should start with a '_'.
   ,-[files/test.custom:1:7]
 1 | const answer = `The answer is ${21 * 2}`;
   :       ^^^|^^
   :          `-- 'answer' is declared here
 2 | 
   `----
  help: Consider removing this declaration.

  x custom-parser-plugin(no-template): Template initializer
   ,-[files/test.custom:1:16]
 1 | const answer = `The answer is ${21 * 2}`;
   :                ^^^^^^^^^^^^^^^^^^^^^^^^^
 2 | 
   `----

  x custom-parser-plugin(no-template): Unexpected template
   ,-[files/test.custom:1:16]
 1 | const answer = `The answer is ${21 * 2}`;
   :                ^^^^^^^^^^^^^^^^^^^^^^^^^
 2 | 
   `----

  ! eslint(no-unused-vars): Variable 'ignored' is declared but never used. Unused variables should start with a '_'.
   ,-[files/test.custom:4:7]
 3 | /* eslint-disable-next-line custom-parser-plugin/no-template -- directive comes from parser comments */
 4 | const ignored = `disabled ${1}`;
   :       ^^^|^^^
   :          `-- 'ignored' is declared here
 5 | 
   `----
  help: Consider removing this declaration.

  ! custom-parser-plugin(source-code-check): fn `greet`: scope=function, firstToken=function, tokenAfterFirst=greet, parserServices.isToyParser=true
   ,-[files/test.custom:6:1]
 5 |     
 6 | ,-> function greet(name) {
 7 | |     return "hello " + name;
 8 | `-> }
   `----

  ! eslint(no-unused-vars): Function 'greet' is declared but never used.
   ,-[files/test.custom:6:10]
 5 | 
 6 | function greet(name) {
   :          ^^|^^
   :            `-- 'greet' is declared here
 7 |   return "hello " + name;
   `----
  help: Consider removing this declaration.

Found 4 warnings and 2 errors.
Finished in Xms on 1 file with 97 rules using X threads.
```

# stderr
```
```
