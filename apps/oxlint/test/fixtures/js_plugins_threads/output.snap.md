# Exit code
1

# stdout
```
  x threads-fixture(no-debugger): debugger
   ,-[files/a.js:1:1]
 1 | debugger;
   : ^^^^^^^^^
   `----

  x threads-fixture(no-todo): todo
   ,-[files/b.js:1:1]
 1 | todo;
   : ^^^^
   `----

  x threads-fixture(no-debugger): debugger
   ,-[files/c.js:1:1]
 1 | debugger;
   : ^^^^^^^^^
 2 | todo;
   `----

  x threads-fixture(no-todo): todo
   ,-[files/c.js:2:1]
 1 | debugger;
 2 | todo;
   : ^^^^
   `----

  x threads-fixture(no-debugger): debugger
   ,-[files/d.js:2:1]
 1 | const ok = 1;
 2 | debugger;
   : ^^^^^^^^^
   `----

Found 0 warnings and 5 errors.
Finished in Xms on 4 files with 2 rules using X threads.
```

# stderr
```
```
