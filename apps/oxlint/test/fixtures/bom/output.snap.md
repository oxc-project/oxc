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
  | sourceText: "/*x*/'é';debugger;\n// 😀🤪😆😎🤮\ndebugger;\ndebugger;"
  | Program span: 5-52
  | comments: [{"start":0,"end":5,"text":"/*x*/"},{"start":19,"end":32,"text":"// 😀🤪😆😎🤮"}]
   ,-[files/bom_unicode.js:1:9]
 1 | ,-> ﻿/*x*/'é';debugger;
 2 | |   // 😀🤪😆😎🤮
 3 | |   debugger;
 4 | `-> debugger;
   `----

  x bom-plugin(bom): Debugger statement at 9-18
   ,-[files/bom_unicode.js:1:14]
 1 | ﻿/*x*/'é';debugger;
   :          ^^^^^^^^^
 2 | // 😀🤪😆😎🤮
   `----

  x bom-plugin(bom): Debugger statement at 33-42
   ,-[files/bom_unicode.js:3:1]
 2 | // 😀🤪😆😎🤮
 3 | debugger;
   : ^^^^^^^^^
 4 | debugger;
   `----

  x bom-plugin(bom): Debugger statement at 43-52
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

  x bom-plugin(bom): Debugger statement at 0-9
   ,-[files/no_bom_unicode.js:1:1]
 1 | debugger;
   : ^^^^^^^^^
 2 | // 😀🤪😆😎🤮
   `----

  x bom-plugin(bom):
  | hasBOM: false
  | sourceText: "debugger;\n// 😀🤪😆😎🤮\ndebugger;\ndebugger;"
  | Program span: 0-43
  | comments: [{"start":10,"end":23,"text":"// 😀🤪😆😎🤮"}]
   ,-[files/no_bom_unicode.js:1:1]
 1 | ,-> debugger;
 2 | |   // 😀🤪😆😎🤮
 3 | |   debugger;
 4 | `-> debugger;
   `----

  x bom-plugin(bom): Debugger statement at 24-33
   ,-[files/no_bom_unicode.js:3:1]
 2 | // 😀🤪😆😎🤮
 3 | debugger;
   : ^^^^^^^^^
 4 | debugger;
   `----

  x bom-plugin(bom): Debugger statement at 34-43
   ,-[files/no_bom_unicode.js:4:1]
 3 | debugger;
 4 | debugger;
   : ^^^^^^^^^
   `----

Found 0 warnings and 16 errors.
Finished in Xms on 4 files with 1 rules using X threads.
```

# stderr
```
```
