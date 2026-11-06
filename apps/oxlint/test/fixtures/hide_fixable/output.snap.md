# Exit code
1

# stdout
```
  x hide-fixable(check): Needs a manual fix
   ,-[files/index.js:2:1]
 1 | fixable;
 2 | manual;
   : ^^^^^^
 3 | suggestion;
   `----

  x hide-fixable(check): Needs a decision
   ,-[files/index.js:3:1]
 2 | manual;
 3 | suggestion;
   : ^^^^^^^^^^
 4 | noFix;
   `----

  x hide-fixable(check): No fix available
   ,-[files/index.js:4:1]
 3 | suggestion;
 4 | noFix;
   : ^^^^^
   `----

Found 0 warnings and 3 errors.
Finished in Xms on 1 file with 1 rules using X threads.
```

# stderr
```
```
