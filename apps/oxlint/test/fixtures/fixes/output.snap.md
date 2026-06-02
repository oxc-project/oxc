# Exit code
1

# stdout
```
  x Plugin `fixes-plugin/fixes` returned invalid fixes.
  | File path: <fixture>/files/range_end_negative.js
  | Invalid range: 0..0

  x Plugin `fixes-plugin/fixes` returned invalid fixes.
  | File path: <fixture>/files/range_end_negative.js
  | Invalid range: 0..0

  x Plugin `fixes-plugin/fixes` returned invalid fixes.
  | File path: <fixture>/files/range_end_out_of_bounds.js
  | Invalid range: 7..7

  x Plugin `fixes-plugin/fixes` returned invalid fixes.
  | File path: <fixture>/files/range_end_out_of_bounds.js
  | Invalid range: 7..7

  x Plugin `fixes-plugin/fixes` returned invalid fixes.
  | File path: <fixture>/files/range_end_too_large.js
  | Invalid range: 0..0

  x Plugin `fixes-plugin/fixes` returned invalid fixes.
  | File path: <fixture>/files/range_end_too_large.js
  | Invalid range: 0..0

  x Plugin `fixes-plugin/fixes` returned invalid fixes.
  | File path: <fixture>/files/range_start_after_end.js
  | Negative range is invalid: Span { start: 3, end: 2 }

  x Plugin `fixes-plugin/fixes` returned invalid fixes.
  | File path: <fixture>/files/range_start_after_end.js
  | Negative range is invalid: Span { start: 3, end: 2 }

  x Plugin `fixes-plugin/fixes` returned invalid fixes.
  | File path: <fixture>/files/range_start_negative.js
  | Invalid range: 0..0

  x Plugin `fixes-plugin/fixes` returned invalid fixes.
  | File path: <fixture>/files/range_start_negative.js
  | Invalid range: 0..0

  x Plugin `fixes-plugin/fixes` returned invalid fixes.
  | File path: <fixture>/files/range_start_too_large.js
  | Invalid range: 0..0

  x Plugin `fixes-plugin/fixes` returned invalid fixes.
  | File path: <fixture>/files/range_start_too_large.js
  | Invalid range: 0..0

  x fixes-plugin(fixes): Replace "a" with "daddy"
   ,-[files/bom.js:1:4]
 1 | ﻿a = c;
   : ^
 2 | g = b
   `----

  x fixes-plugin(fixes): Prefix "c" with "magi"
   ,-[files/bom.js:1:8]
 1 | ﻿a = c;
   :     ^
 2 | g = b
   `----

  x fixes-plugin(fixes): Replace "g" with "rage"
   ,-[files/bom.js:2:1]
 1 | ﻿a = c;
 2 | g = b
   : ^
   `----

  x fixes-plugin(fixes): Replace "b" with "abacus"
   ,-[files/bom.js:2:5]
 1 | ﻿a = c;
 2 | g = b
   :     ^
   `----

  x fixes-plugin(fixes): Replace "a" with "daddy"
   ,-[files/bom_and_unicode.js:1:4]
 1 | ﻿a = c;
   : ^
 2 | // 😀🤪😆😎🤮
   `----

  x fixes-plugin(fixes): Prefix "c" with "magi"
   ,-[files/bom_and_unicode.js:1:8]
 1 | ﻿a = c;
   :     ^
 2 | // 😀🤪😆😎🤮
   `----

  x fixes-plugin(fixes): Replace "g" with "rage"
   ,-[files/bom_and_unicode.js:3:1]
 2 | // 😀🤪😆😎🤮
 3 | g = b
   : ^
   `----

  x fixes-plugin(fixes): Replace "b" with "abacus"
   ,-[files/bom_and_unicode.js:3:5]
 2 | // 😀🤪😆😎🤮
 3 | g = b
   :     ^
   `----

  x fixes-plugin(fixes): Replace "a" with "daddy"
   ,-[files/bom_remove.js:1:4]
 1 | ﻿a = c;
   : ^
 2 | d = b
   `----

  x fixes-plugin(fixes): Remove BOM
   ,-[files/bom_remove.js:1:4]
 1 | ,-> ﻿a = c;
 2 | `-> d = b
   `----

  x fixes-plugin(fixes): Prefix "c" with "magi"
   ,-[files/bom_remove.js:1:8]
 1 | ﻿a = c;
   :     ^
 2 | d = b
   `----

  x fixes-plugin(fixes): Prefix "d" with "damne"
   ,-[files/bom_remove.js:2:1]
 1 | ﻿a = c;
 2 | d = b
   : ^
   `----

  x fixes-plugin(fixes): Replace "b" with "abacus"
   ,-[files/bom_remove.js:2:5]
 1 | ﻿a = c;
 2 | d = b
   :     ^
   `----

  x fixes-plugin(fixes): Replace "a" with "daddy"
   ,-[files/bom_remove2.js:1:4]
 1 | ﻿a = c;
   : ^
 2 | d = b
   `----

  x fixes-plugin(fixes): Remove BOM multiple
   ,-[files/bom_remove2.js:1:4]
 1 | ,-> ﻿a = c;
 2 | `-> d = b
   `----

  x fixes-plugin(fixes): Prefix "c" with "magi"
   ,-[files/bom_remove2.js:1:8]
 1 | ﻿a = c;
   :     ^
 2 | d = b
   `----

  x fixes-plugin(fixes): Prefix "d" with "damne"
   ,-[files/bom_remove2.js:2:1]
 1 | ﻿a = c;
 2 | d = b
   : ^
   `----

  x fixes-plugin(fixes): Replace "b" with "abacus"
   ,-[files/bom_remove2.js:2:5]
 1 | ﻿a = c;
 2 | d = b
   :     ^
   `----

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

  x fixes-plugin(fixes): Replace "g" with "rage"
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

  x fixes-plugin(fixes): end negative
   ,-[files/range_end_negative.js:1:5]
 1 | let x;
   :     ^
   `----

  x fixes-plugin(fixes): end negative multiple
   ,-[files/range_end_negative.js:1:5]
 1 | let x;
   :     ^
   `----

  x fixes-plugin(fixes): end out of bounds
   ,-[files/range_end_out_of_bounds.js:1:5]
 1 | let x;
   :     ^
   `----

  x fixes-plugin(fixes): end out of bounds multiple
   ,-[files/range_end_out_of_bounds.js:1:5]
 1 | let x;
   :     ^
   `----

  x fixes-plugin(fixes): end too large
   ,-[files/range_end_too_large.js:1:5]
 1 | let x;
   :     ^
   `----

  x fixes-plugin(fixes): end too large multiple
   ,-[files/range_end_too_large.js:1:5]
 1 | let x;
   :     ^
   `----

  x fixes-plugin(fixes): start after end
   ,-[files/range_start_after_end.js:1:5]
 1 | let x;
   :     ^
   `----

  x fixes-plugin(fixes): start after end multiple
   ,-[files/range_start_after_end.js:1:5]
 1 | let x;
   :     ^
   `----

  x fixes-plugin(fixes): start negative
   ,-[files/range_start_negative.js:1:5]
 1 | let x;
   :     ^
   `----

  x fixes-plugin(fixes): start negative multiple
   ,-[files/range_start_negative.js:1:5]
 1 | let x;
   :     ^
   `----

  x fixes-plugin(fixes): start too large
   ,-[files/range_start_too_large.js:1:5]
 1 | let x;
   :     ^
   `----

  x fixes-plugin(fixes): start too large multiple
   ,-[files/range_start_too_large.js:1:5]
 1 | let x;
   :     ^
   `----

  x fixes-plugin(fixes): Replace "a" with "daddy"
   ,-[files/unicode.js:1:1]
 1 | a = c;
   : ^
 2 | // 😀🤪😆😎🤮
   `----

  x fixes-plugin(fixes): Prefix "c" with "magi"
   ,-[files/unicode.js:1:5]
 1 | a = c;
   :     ^
 2 | // 😀🤪😆😎🤮
   `----

  x fixes-plugin(fixes): Replace "g" with "rage"
   ,-[files/unicode.js:3:1]
 2 | // 😀🤪😆😎🤮
 3 | g = b
   : ^
   `----

  x fixes-plugin(fixes): Replace "b" with "abacus"
   ,-[files/unicode.js:3:5]
 2 | // 😀🤪😆😎🤮
 3 | g = b
   :     ^
   `----

Found 0 warnings and 58 errors.
Finished in Xms on 12 files with 1 rules using X threads.
```

# stderr
```
```
