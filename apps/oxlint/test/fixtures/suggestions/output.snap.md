# Exit code
1

# stdout
```
  x suggestions-plugin(suggestions): Replace "a" with "daddy"
   ,-[files/bom.js:1:4]
 1 | ﻿a = c;
   : ^
 2 | g = b
   `----

  x suggestions-plugin(suggestions): Prefix "c" with "magi"
   ,-[files/bom.js:1:8]
 1 | ﻿a = c;
   :     ^
 2 | g = b
   `----

  x suggestions-plugin(suggestions): Replace "g" with "rage"
   ,-[files/bom.js:2:1]
 1 | ﻿a = c;
 2 | g = b
   : ^
   `----

  x suggestions-plugin(suggestions): Replace "b" with "abacus"
   ,-[files/bom.js:2:5]
 1 | ﻿a = c;
 2 | g = b
   :     ^
   `----

  x suggestions-plugin(suggestions): Replace "a" with "daddy"
   ,-[files/bom_and_unicode.js:1:4]
 1 | ﻿a = c;
   : ^
 2 | // 😀🤪😆😎🤮
   `----

  x suggestions-plugin(suggestions): Prefix "c" with "magi"
   ,-[files/bom_and_unicode.js:1:8]
 1 | ﻿a = c;
   :     ^
 2 | // 😀🤪😆😎🤮
   `----

  x suggestions-plugin(suggestions): Replace "g" with "rage"
   ,-[files/bom_and_unicode.js:3:1]
 2 | // 😀🤪😆😎🤮
 3 | g = b
   : ^
   `----

  x suggestions-plugin(suggestions): Replace "b" with "abacus"
   ,-[files/bom_and_unicode.js:3:5]
 2 | // 😀🤪😆😎🤮
 3 | g = b
   :     ^
   `----

  x suggestions-plugin(suggestions): Remove debugger statement
   ,-[files/index.js:1:1]
 1 | debugger;
   : ^^^^^^^^^
 2 | 
   `----

  x suggestions-plugin(suggestions): Replace "a" with "daddy"
   ,-[files/index.js:3:5]
 2 | 
 3 | let a = 1;
   :     ^
 4 | let b = 2;
   `----

  x suggestions-plugin(suggestions): Replace "b" with "abacus"
   ,-[files/index.js:4:5]
 3 | let a = 1;
 4 | let b = 2;
   :     ^
 5 | let c = 3;
   `----

  x suggestions-plugin(suggestions): Prefix "c" with "magi"
   ,-[files/index.js:5:5]
 4 | let b = 2;
 5 | let c = 3;
   :     ^
 6 | let d = 4;
   `----

  x suggestions-plugin(suggestions): Prefix "d" with "damne"
   ,-[files/index.js:6:5]
 5 | let c = 3;
 6 | let d = 4;
   :     ^
 7 | let e = 5;
   `----

  x suggestions-plugin(suggestions): Postfix "e" with "lephant"
   ,-[files/index.js:7:5]
 6 | let d = 4;
 7 | let e = 5;
   :     ^
 8 | let f = 6;
   `----

  x suggestions-plugin(suggestions): Postfix "f" with "eck"
   ,-[files/index.js:8:5]
 7 | let e = 5;
 8 | let f = 6;
   :     ^
 9 | let g = 7;
   `----

  x suggestions-plugin(suggestions): Replace "g" with "rage"
    ,-[files/index.js:9:5]
  8 | let f = 6;
  9 | let g = 7;
    :     ^
 10 | let h = 8;
    `----

  x suggestions-plugin(suggestions): Replace "h" with "dangermouse"
    ,-[files/index.js:10:5]
  9 | let g = 7;
 10 | let h = 8;
    :     ^
 11 | let i = 9;
    `----

  x suggestions-plugin(suggestions): Replace "i" with "granular"
    ,-[files/index.js:11:5]
 10 | let h = 8;
 11 | let i = 9;
    :     ^
 12 | let j = 10;
    `----

  x suggestions-plugin(suggestions): Replace "j" with "cowabunga"
    ,-[files/index.js:12:5]
 11 | let i = 9;
 12 | let j = 10;
    :     ^
 13 | let k = 11;
    `----

  x suggestions-plugin(suggestions): Replace "k" with "kaboom"
    ,-[files/index.js:13:5]
 12 | let j = 10;
 13 | let k = 11;
    :     ^
 14 | 
    `----

  x suggestions-plugin(suggestions): Remove debugger statement
    ,-[files/index.js:15:1]
 14 | 
 15 | debugger;
    : ^^^^^^^^^
    `----

  x suggestions-plugin(suggestions): Replace "a" with "daddy"
   ,-[files/unicode.js:1:1]
 1 | a = c;
   : ^
 2 | // 😀🤪😆😎🤮
   `----

  x suggestions-plugin(suggestions): Prefix "c" with "magi"
   ,-[files/unicode.js:1:5]
 1 | a = c;
   :     ^
 2 | // 😀🤪😆😎🤮
   `----

  x suggestions-plugin(suggestions): Replace "g" with "rage"
   ,-[files/unicode.js:3:1]
 2 | // 😀🤪😆😎🤮
 3 | g = b
   : ^
   `----

  x suggestions-plugin(suggestions): Replace "b" with "abacus"
   ,-[files/unicode.js:3:5]
 2 | // 😀🤪😆😎🤮
 3 | g = b
   :     ^
   `----

Found 0 warnings and 25 errors.
Finished in Xms on 4 files with 1 rules using X threads.
```

# stderr
```
```
