# Exit code
1

# stdout
```
  x Plugin `suggestions-plugin/suggestions` returned invalid suggestions.
  | File path: <fixture>/files/range_end_negative.js
  | Invalid range: 0..0

  x Plugin `suggestions-plugin/suggestions` returned invalid suggestions.
  | File path: <fixture>/files/range_end_negative.js
  | Invalid range: 0..0

  x Plugin `suggestions-plugin/suggestions` returned invalid suggestions.
  | File path: <fixture>/files/range_end_out_of_bounds.js
  | Invalid range: 7..7

  x Plugin `suggestions-plugin/suggestions` returned invalid suggestions.
  | File path: <fixture>/files/range_end_out_of_bounds.js
  | Invalid range: 7..7

  x Plugin `suggestions-plugin/suggestions` returned invalid suggestions.
  | File path: <fixture>/files/range_end_too_large.js
  | Invalid range: 0..0

  x Plugin `suggestions-plugin/suggestions` returned invalid suggestions.
  | File path: <fixture>/files/range_end_too_large.js
  | Invalid range: 0..0

  x Plugin `suggestions-plugin/suggestions` returned invalid suggestions.
  | File path: <fixture>/files/range_start_after_end.js
  | Negative range is invalid: Span { start: 3, end: 2 }

  x Plugin `suggestions-plugin/suggestions` returned invalid suggestions.
  | File path: <fixture>/files/range_start_after_end.js
  | Negative range is invalid: Span { start: 3, end: 2 }

  x Plugin `suggestions-plugin/suggestions` returned invalid suggestions.
  | File path: <fixture>/files/range_start_negative.js
  | Invalid range: 0..0

  x Plugin `suggestions-plugin/suggestions` returned invalid suggestions.
  | File path: <fixture>/files/range_start_negative.js
  | Invalid range: 0..0

  x Plugin `suggestions-plugin/suggestions` returned invalid suggestions.
  | File path: <fixture>/files/range_start_too_large.js
  | Invalid range: 0..0

  x Plugin `suggestions-plugin/suggestions` returned invalid suggestions.
  | File path: <fixture>/files/range_start_too_large.js
  | Invalid range: 0..0

  x suggestions-plugin(suggestions): end negative
   ,-[files/range_end_negative.js:1:5]
 1 | let x;
   :     ^
   `----

  x suggestions-plugin(suggestions): end negative multiple
   ,-[files/range_end_negative.js:1:5]
 1 | let x;
   :     ^
   `----

  x suggestions-plugin(suggestions): end out of bounds
   ,-[files/range_end_out_of_bounds.js:1:5]
 1 | let x;
   :     ^
   `----

  x suggestions-plugin(suggestions): end out of bounds multiple
   ,-[files/range_end_out_of_bounds.js:1:5]
 1 | let x;
   :     ^
   `----

  x suggestions-plugin(suggestions): end too large
   ,-[files/range_end_too_large.js:1:5]
 1 | let x;
   :     ^
   `----

  x suggestions-plugin(suggestions): end too large multiple
   ,-[files/range_end_too_large.js:1:5]
 1 | let x;
   :     ^
   `----

  x suggestions-plugin(suggestions): start after end
   ,-[files/range_start_after_end.js:1:5]
 1 | let x;
   :     ^
   `----

  x suggestions-plugin(suggestions): start after end multiple
   ,-[files/range_start_after_end.js:1:5]
 1 | let x;
   :     ^
   `----

  x suggestions-plugin(suggestions): start negative
   ,-[files/range_start_negative.js:1:5]
 1 | let x;
   :     ^
   `----

  x suggestions-plugin(suggestions): start negative multiple
   ,-[files/range_start_negative.js:1:5]
 1 | let x;
   :     ^
   `----

  x suggestions-plugin(suggestions): start too large
   ,-[files/range_start_too_large.js:1:5]
 1 | let x;
   :     ^
   `----

  x suggestions-plugin(suggestions): start too large multiple
   ,-[files/range_start_too_large.js:1:5]
 1 | let x;
   :     ^
   `----

Found 0 warnings and 24 errors.
Finished in Xms on 12 files with 1 rules using X threads.
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

# File altered: files/bom_remove.js
```
daddy = magic;
damned = abacus
```

# File altered: files/bom_remove2.js
```
daddy = magic;
damned = abacus
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
let kaboom = 11;


```

# File altered: files/unicode.js
```
daddy = magic;
// 😀🤪😆😎🤮
rage = abacus
```
