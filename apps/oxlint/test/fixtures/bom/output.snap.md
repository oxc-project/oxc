# Exit code
1

# stdout
```
  x bom-plugin(bom): Debugger statement at 0-9
   ,-[files/bom.js:1:4]
 1 | ﻿debugger;
   : ^^^^^^^^^
 2 | debugger;
   `----

  x bom-plugin(bom):
  | hasBOM: true
  | sourceText: "debugger;\ndebugger;\ndebugger;"
  | Program span: 0-29
   ,-[files/bom.js:1:4]
 1 | ,-> ﻿debugger;
 2 | |   debugger;
 3 | `-> debugger;
   `----

  x bom-plugin(bom): Debugger statement at 10-19
   ,-[files/bom.js:2:1]
 1 | ﻿debugger;
 2 | debugger;
   : ^^^^^^^^^
 3 | debugger;
   `----

  x bom-plugin(bom): Debugger statement at 20-29
   ,-[files/bom.js:3:1]
 2 | debugger;
 3 | debugger;
   : ^^^^^^^^^
   `----

  x bom-plugin(bom):
  | hasBOM: true
  | sourceText: "é=1;debugger;"
  | Program span: 0-13
   ,-[files/bom_leading_unicode.js:1:4]
 1 | ﻿é=1;debugger;
   : ^^^^^^^^^^^^^
   `----

  x bom-plugin(bom): Debugger statement at 4-13
   ,-[files/bom_leading_unicode.js:1:9]
 1 | ﻿é=1;debugger;
   :     ^^^^^^^^^
   `----

  x bom-plugin(bom):
  | hasBOM: true
  | sourceText: "/*x*/'é';/*y*/ऊ=1;debugger;\n// 😀🤪😆😎🤮\ndebugger;\ndebugger;"
  | Program span: 5-61
  | comments: [{"start":0,"end":5,"text":"/*x*/"},{"start":9,"end":14,"text":"/*y*/"},{"start":28,"end":41,"text":"// 😀🤪😆😎🤮"}]
   ,-[files/bom_unicode.js:1:9]
 1 | ,-> ﻿/*x*/'é';/*y*/ऊ=1;debugger;
 2 | |   // 😀🤪😆😎🤮
 3 | |   debugger;
 4 | `-> debugger;
   `----

  x bom-plugin(bom): Debugger statement at 18-27
   ,-[files/bom_unicode.js:1:25]
 1 | ﻿/*x*/'é';/*y*/ऊ=1;debugger;
   :                   ^^^^^^^^^
 2 | // 😀🤪😆😎🤮
   `----

  x bom-plugin(bom): Debugger statement at 42-51
   ,-[files/bom_unicode.js:3:1]
 2 | // 😀🤪😆😎🤮
 3 | debugger;
   : ^^^^^^^^^
 4 | debugger;
   `----

  x bom-plugin(bom): Debugger statement at 52-61
   ,-[files/bom_unicode.js:4:1]
 3 | debugger;
 4 | debugger;
   : ^^^^^^^^^
   `----

  x bom-plugin(bom): Debugger statement at 0-9
   ,-[files/no_bom.js:1:1]
 1 | debugger;
   : ^^^^^^^^^
 2 | debugger;
   `----

  x bom-plugin(bom):
  | hasBOM: false
  | sourceText: "debugger;\ndebugger;\ndebugger;"
  | Program span: 0-29
   ,-[files/no_bom.js:1:1]
 1 | ,-> debugger;
 2 | |   debugger;
 3 | `-> debugger;
   `----

  x bom-plugin(bom): Debugger statement at 10-19
   ,-[files/no_bom.js:2:1]
 1 | debugger;
 2 | debugger;
   : ^^^^^^^^^
 3 | debugger;
   `----

  x bom-plugin(bom): Debugger statement at 20-29
   ,-[files/no_bom.js:3:1]
 2 | debugger;
 3 | debugger;
   : ^^^^^^^^^
   `----

  x bom-plugin(bom):
  | hasBOM: false
  | sourceText: "é=1;debugger;"
  | Program span: 0-13
   ,-[files/no_bom_leading_unicode.js:1:1]
 1 | é=1;debugger;
   : ^^^^^^^^^^^^^
   `----

  x bom-plugin(bom): Debugger statement at 4-13
   ,-[files/no_bom_leading_unicode.js:1:6]
 1 | é=1;debugger;
   :     ^^^^^^^^^
   `----

  x bom-plugin(bom):
  | hasBOM: false
  | sourceText: "/*x*/'é';/*y*/ऊ=1;debugger;\n// 😀🤪😆😎🤮\ndebugger;\ndebugger;"
  | Program span: 5-61
  | comments: [{"start":0,"end":5,"text":"/*x*/"},{"start":9,"end":14,"text":"/*y*/"},{"start":28,"end":41,"text":"// 😀🤪😆😎🤮"}]
   ,-[files/no_bom_unicode.js:1:6]
 1 | ,-> /*x*/'é';/*y*/ऊ=1;debugger;
 2 | |   // 😀🤪😆😎🤮
 3 | |   debugger;
 4 | `-> debugger;
   `----

  x bom-plugin(bom): Debugger statement at 18-27
   ,-[files/no_bom_unicode.js:1:22]
 1 | /*x*/'é';/*y*/ऊ=1;debugger;
   :                   ^^^^^^^^^
 2 | // 😀🤪😆😎🤮
   `----

  x bom-plugin(bom): Debugger statement at 42-51
   ,-[files/no_bom_unicode.js:3:1]
 2 | // 😀🤪😆😎🤮
 3 | debugger;
   : ^^^^^^^^^
 4 | debugger;
   `----

  x bom-plugin(bom): Debugger statement at 52-61
   ,-[files/no_bom_unicode.js:4:1]
 3 | debugger;
 4 | debugger;
   : ^^^^^^^^^
   `----

Found 0 warnings and 20 errors.
Finished in Xms on 6 files with 1 rules using X threads.
```

# stderr
```
```
