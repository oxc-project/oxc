# Exit code
1

# stdout
```
  x fixes-plugin(fixes): Remove debugger statement
   ,-[files/index.js:1:1]
 1 | debugger;
   : ^^^^^^^^^
 2 | 
   `----

  x fixes-plugin(fixes): Replace "a" with "daddy"
   ,-[files/index.js:3:5]
 2 | 
 3 | let a = 1;
   :     ^
 4 | let b = 2;
   `----

  x fixes-plugin(fixes): Replace "b" with "abacus"
   ,-[files/index.js:4:5]
 3 | let a = 1;
 4 | let b = 2;
   :     ^
 5 | let c = 3;
   `----

  x fixes-plugin(fixes): Prefix "c" with "magi"
   ,-[files/index.js:5:5]
 4 | let b = 2;
 5 | let c = 3;
   :     ^
 6 | let d = 4;
   `----

  x fixes-plugin(fixes): Prefix "d" with "damne"
   ,-[files/index.js:6:5]
 5 | let c = 3;
 6 | let d = 4;
   :     ^
 7 | let e = 5;
   `----

  x fixes-plugin(fixes): Postfix "e" with "lephant"
   ,-[files/index.js:7:5]
 6 | let d = 4;
 7 | let e = 5;
   :     ^
 8 | let f = 6;
   `----

  x fixes-plugin(fixes): Postfix "f" with "eck"
   ,-[files/index.js:8:5]
 7 | let e = 5;
 8 | let f = 6;
   :     ^
 9 | let g = 7;
   `----

  x fixes-plugin(fixes): Replace "g" with "numpty"
    ,-[files/index.js:9:5]
  8 | let f = 6;
  9 | let g = 7;
    :     ^
 10 | let h = 8;
    `----

  x fixes-plugin(fixes): Replace "h" with "dangermouse"
    ,-[files/index.js:10:5]
  9 | let g = 7;
 10 | let h = 8;
    :     ^
 11 | let i = 9;
    `----

  x fixes-plugin(fixes): Replace "i" with "granular"
    ,-[files/index.js:11:5]
 10 | let h = 8;
 11 | let i = 9;
    :     ^
 12 | let j = 10;
    `----

  x fixes-plugin(fixes): Replace "j" with "cowabunga"
    ,-[files/index.js:12:5]
 11 | let i = 9;
 12 | let j = 10;
    :     ^
 13 | 
    `----

  x fixes-plugin(fixes): Remove debugger statement
    ,-[files/index.js:14:1]
 13 | 
 14 | debugger;
    : ^^^^^^^^^
    `----

Found 0 warnings and 12 errors.
Finished in Xms on 1 file using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
