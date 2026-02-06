# Exit code
1

# stdout
```
  x Error running JS plugin.
  | File path: <fixture>/files/range_end_negative.js
  | Failed to deserialize JSON returned by `lintFile`: invalid value: integer `-10`, expected u32 at line 1 column 111

  x Plugin `fixes-plugin/fixes` returned invalid fixes.
  | File path: <fixture>/files/range_end_out_of_bounds.js
  | Invalid range: 7..7

  x Plugin `fixes-plugin/fixes` returned invalid fixes.
  | File path: <fixture>/files/range_end_out_of_bounds.js
  | Invalid range: 7..7

  x Error running JS plugin.
  | File path: <fixture>/files/range_end_too_large.js
  | Failed to deserialize JSON returned by `lintFile`: invalid value: integer `4294967296`, expected u32 at line 1 column 119

  x Plugin `fixes-plugin/fixes` returned invalid fixes.
  | File path: <fixture>/files/range_start_after_end.js
  | Negative range is invalid: Span { start: 3, end: 2 }

  x Plugin `fixes-plugin/fixes` returned invalid fixes.
  | File path: <fixture>/files/range_start_after_end.js
  | Negative range is invalid: Span { start: 3, end: 2 }

  x Error running JS plugin.
  | File path: <fixture>/files/range_start_negative.js
  | Failed to deserialize JSON returned by `lintFile`: invalid value: integer `-10`, expected u32 at line 1 column 111

  x Error running JS plugin.
  | File path: <fixture>/files/range_start_too_large.js
  | Failed to deserialize JSON returned by `lintFile`: invalid value: integer `4294967296`, expected u32 at line 1 column 119

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

Found 0 warnings and 12 errors.
Finished in Xms on 10 files with 1 rules using X threads.
```

# stderr
```
```

# File altered: files/bom.js
```
﻿daddy = magic;
rage = abacus
```

# File altered: files/bom_and_unicode.js
```
﻿daddy = magic;
// 😀🤪😆😎🤮
rage = abacus
```

# File altered: files/index.js
```


let daddy = 1;
let abacus = 2;
let magic = 3;
let damned = 4;
let elephant = 5;
let feck = 6;
let rage = 7;
let dangermouse = 8;
let granular = 9;
let cowabunga = 10;


```

# File altered: files/unicode.js
```
daddy = magic;
// 😀🤪😆😎🤮
rage = abacus
```
