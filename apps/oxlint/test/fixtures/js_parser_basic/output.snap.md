# Exit code
1

# stdout
```
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

  ! custom-parser-plugin(source-code-check): fn `greet`: scope=function, firstToken=function, tokenAfterFirst=greet, parserServices.isToyParser=true
   ,-[files/test.custom:6:1]
 5 |     
 6 | ,-> function greet(name) {
 7 | |     return "hello " + name;
 8 | `-> }
   `----

Found 1 warning and 2 errors.
Finished in Xms on 1 file with 97 rules using X threads.
```

# stderr
```
```
