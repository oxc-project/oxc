# Exit code
1

# stdout
```
  x weakmap-cache-plugin(cache1): Rules which have accessed this file:
  | filename: <fixture>/files/1.js
  | ident names: a, a, a
  | rule names: cache1, cache2, cache3
   ,-[files/1.js:1:1]
 1 | let a;
   : ^^^^^^^
   `----

  x weakmap-cache-plugin(cache2): Rules which have accessed this file:
  | filename: <fixture>/files/1.js
  | ident names: a, a, a
  | rule names: cache1, cache2, cache3
   ,-[files/1.js:1:1]
 1 | let a;
   : ^^^^^^^
   `----

  x weakmap-cache-plugin(cache3): Rules which have accessed this file:
  | filename: <fixture>/files/1.js
  | ident names: a, a, a
  | rule names: cache1, cache2, cache3
   ,-[files/1.js:1:1]
 1 | let a;
   : ^^^^^^^
   `----

  x weakmap-cache-plugin(cache1): Rules which have accessed this file:
  | filename: <fixture>/files/2.js
  | ident names: b, b, b
  | rule names: cache1, cache2, cache3
   ,-[files/2.js:1:1]
 1 | let b;
   : ^^^^^^^
   `----

  x weakmap-cache-plugin(cache2): Rules which have accessed this file:
  | filename: <fixture>/files/2.js
  | ident names: b, b, b
  | rule names: cache1, cache2, cache3
   ,-[files/2.js:1:1]
 1 | let b;
   : ^^^^^^^
   `----

  x weakmap-cache-plugin(cache3): Rules which have accessed this file:
  | filename: <fixture>/files/2.js
  | ident names: b, b, b
  | rule names: cache1, cache2, cache3
   ,-[files/2.js:1:1]
 1 | let b;
   : ^^^^^^^
   `----

  x weakmap-cache-plugin(cache1): Rules which have accessed this file:
  | filename: <fixture>/files/3.js
  | ident names: c, c, c
  | rule names: cache1, cache2, cache3
   ,-[files/3.js:1:1]
 1 | let c;
   : ^^^^^^^
   `----

  x weakmap-cache-plugin(cache2): Rules which have accessed this file:
  | filename: <fixture>/files/3.js
  | ident names: c, c, c
  | rule names: cache1, cache2, cache3
   ,-[files/3.js:1:1]
 1 | let c;
   : ^^^^^^^
   `----

  x weakmap-cache-plugin(cache3): Rules which have accessed this file:
  | filename: <fixture>/files/3.js
  | ident names: c, c, c
  | rule names: cache1, cache2, cache3
   ,-[files/3.js:1:1]
 1 | let c;
   : ^^^^^^^
   `----

Found 0 warnings and 9 errors.
Finished in Xms on 3 files with 3 rules using X threads.
```

# stderr
```
```
