# Exit code
1

# stdout
```
  x define-rule-plugin(create): ident visit fn "a":
  | filename: <fixture>/files/1.js
   ,-[files/1.js:1:5]
 1 | let a, b;
   :     ^
   `----

  x define-rule-plugin(create-once): ident visit fn "a":
  | filename: <fixture>/files/1.js
   ,-[files/1.js:1:5]
 1 | let a, b;
   :     ^
   `----

  x define-rule-plugin(create): ident visit fn "b":
  | filename: <fixture>/files/1.js
   ,-[files/1.js:1:8]
 1 | let a, b;
   :        ^
   `----

  x define-rule-plugin(create-once): ident visit fn "b":
  | filename: <fixture>/files/1.js
   ,-[files/1.js:1:8]
 1 | let a, b;
   :        ^
   `----

  x define-rule-plugin(create): ident visit fn "c":
  | filename: <fixture>/files/2.js
   ,-[files/2.js:1:5]
 1 | let c, d;
   :     ^
   `----

  x define-rule-plugin(create-once): ident visit fn "c":
  | filename: <fixture>/files/2.js
   ,-[files/2.js:1:5]
 1 | let c, d;
   :     ^
   `----

  x define-rule-plugin(create): ident visit fn "d":
  | filename: <fixture>/files/2.js
   ,-[files/2.js:1:8]
 1 | let c, d;
   :        ^
   `----

  x define-rule-plugin(create-once): ident visit fn "d":
  | filename: <fixture>/files/2.js
   ,-[files/2.js:1:8]
 1 | let c, d;
   :        ^
   `----

Found 0 warnings and 8 errors.
Finished in Xms on 2 files with 2 rules using X threads.
```

# stderr
```
```
