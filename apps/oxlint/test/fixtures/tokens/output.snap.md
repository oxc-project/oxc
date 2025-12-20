# Exit code
1

# stdout
```
  x tokens-plugin(tokens): Tokens:
  | Keyword           loc= 3:0 - 3:3    range= 20-23   "let"
  | Identifier        loc= 3:4 - 3:5    range= 24-25   "x"
  | Punctuator        loc= 3:6 - 3:7    range= 26-27   "="
  | Numeric           loc= 3:29 - 3:30  range= 49-50   "1"
  | Punctuator        loc= 3:30 - 3:31  range= 50-51   ";"
  | Keyword           loc= 6:0 - 6:3    range= 72-75   "let"
  | Identifier        loc= 6:4 - 6:5    range= 76-77   "y"
  | Punctuator        loc= 6:6 - 6:7    range= 78-79   "="
  | Numeric           loc= 6:8 - 6:9    range= 80-81   "2"
  | Punctuator        loc= 6:9 - 6:10   range= 81-82   ";"
   ,-[files/index.js:1:1]
 1 | ,-> // Leading comment
 2 | |
 3 | |   let x = /* inline comment */ 1;
 4 | |
 5 | |   // Another comment
 6 | |   let y = 2;
 7 | |
 8 | `-> // Trailing comment
   `----

  x tokens-plugin(tokens): Tokens and comments:
  | Line              loc= 1:0 - 1:18   range= 0-18    " Leading comment"
  | Keyword           loc= 3:0 - 3:3    range= 20-23   "let"
  | Identifier        loc= 3:4 - 3:5    range= 24-25   "x"
  | Punctuator        loc= 3:6 - 3:7    range= 26-27   "="
  | Block             loc= 3:8 - 3:28   range= 28-48   " inline comment "
  | Numeric           loc= 3:29 - 3:30  range= 49-50   "1"
  | Punctuator        loc= 3:30 - 3:31  range= 50-51   ";"
  | Line              loc= 5:0 - 5:18   range= 53-71   " Another comment"
  | Keyword           loc= 6:0 - 6:3    range= 72-75   "let"
  | Identifier        loc= 6:4 - 6:5    range= 76-77   "y"
  | Punctuator        loc= 6:6 - 6:7    range= 78-79   "="
  | Numeric           loc= 6:8 - 6:9    range= 80-81   "2"
  | Punctuator        loc= 6:9 - 6:10   range= 81-82   ";"
  | Line              loc= 8:0 - 8:19   range= 84-103  " Trailing comment"
   ,-[files/index.js:1:1]
 1 | ,-> // Leading comment
 2 | |
 3 | |   let x = /* inline comment */ 1;
 4 | |
 5 | |   // Another comment
 6 | |   let y = 2;
 7 | |
 8 | `-> // Trailing comment
   `----

  x tokens-plugin(tokens): Line (" Leading comment")
   ,-[files/index.js:1:1]
 1 | // Leading comment
   : ^^^^^^^^^^^^^^^^^^
 2 |
   `----

  x tokens-plugin(tokens): Keyword ("let")
   ,-[files/index.js:3:1]
 2 |
 3 | let x = /* inline comment */ 1;
   : ^^^
 4 |
   `----

  x tokens-plugin(tokens): Identifier ("x")
   ,-[files/index.js:3:5]
 2 |
 3 | let x = /* inline comment */ 1;
   :     ^
 4 |
   `----

  x tokens-plugin(tokens): Punctuator ("=")
   ,-[files/index.js:3:7]
 2 |
 3 | let x = /* inline comment */ 1;
   :       ^
 4 |
   `----

  x tokens-plugin(tokens): Block (" inline comment ")
   ,-[files/index.js:3:9]
 2 |
 3 | let x = /* inline comment */ 1;
   :         ^^^^^^^^^^^^^^^^^^^^
 4 |
   `----

  x tokens-plugin(tokens): Numeric ("1")
   ,-[files/index.js:3:30]
 2 |
 3 | let x = /* inline comment */ 1;
   :                              ^
 4 |
   `----

  x tokens-plugin(tokens): Punctuator (";")
   ,-[files/index.js:3:31]
 2 |
 3 | let x = /* inline comment */ 1;
   :                               ^
 4 |
   `----

  x tokens-plugin(tokens): Line (" Another comment")
   ,-[files/index.js:5:1]
 4 |
 5 | // Another comment
   : ^^^^^^^^^^^^^^^^^^
 6 | let y = 2;
   `----

  x tokens-plugin(tokens): Keyword ("let")
   ,-[files/index.js:6:1]
 5 | // Another comment
 6 | let y = 2;
   : ^^^
 7 |
   `----

  x tokens-plugin(tokens): Identifier ("y")
   ,-[files/index.js:6:5]
 5 | // Another comment
 6 | let y = 2;
   :     ^
 7 |
   `----

  x tokens-plugin(tokens): Punctuator ("=")
   ,-[files/index.js:6:7]
 5 | // Another comment
 6 | let y = 2;
   :       ^
 7 |
   `----

  x tokens-plugin(tokens): Numeric ("2")
   ,-[files/index.js:6:9]
 5 | // Another comment
 6 | let y = 2;
   :         ^
 7 |
   `----

  x tokens-plugin(tokens): Punctuator (";")
   ,-[files/index.js:6:10]
 5 | // Another comment
 6 | let y = 2;
   :          ^
 7 |
   `----

  x tokens-plugin(tokens): Line (" Trailing comment")
   ,-[files/index.js:8:1]
 7 |
 8 | // Trailing comment
   : ^^^^^^^^^^^^^^^^^^^
   `----

  x tokens-plugin(tokens): Tokens:
  | Keyword           loc= 1:0 - 1:5    range= 0-5     "const"
  | Identifier        loc= 1:6 - 1:9    range= 6-9     "obj"
  | Punctuator        loc= 1:10 - 1:11  range= 10-11   "="
  | Punctuator        loc= 1:12 - 1:13  range= 12-13   "{"
  | Identifier        loc= 2:2 - 2:4    range= 16-18   "fn"
  | Punctuator        loc= 2:4 - 2:5    range= 18-19   ":"
  | Punctuator        loc= 2:6 - 2:7    range= 20-21   "<"
  | Identifier        loc= 2:7 - 2:8    range= 21-22   "T"
  | Punctuator        loc= 2:8 - 2:9    range= 22-23   ">"
  | Punctuator        loc= 2:9 - 2:10   range= 23-24   "("
  | Identifier        loc= 2:10 - 2:13  range= 24-27   "arg"
  | Punctuator        loc= 2:13 - 2:14  range= 27-28   ":"
  | Identifier        loc= 2:15 - 2:16  range= 29-30   "T"
  | Punctuator        loc= 2:16 - 2:17  range= 30-31   ")"
  | Punctuator        loc= 2:17 - 2:18  range= 31-32   ":"
  | Identifier        loc= 2:19 - 2:20  range= 33-34   "T"
  | Punctuator        loc= 2:21 - 2:23  range= 35-37   "=>"
  | Punctuator        loc= 2:24 - 2:25  range= 38-39   "{"
  | Keyword           loc= 3:4 - 3:10   range= 44-50   "return"
  | Identifier        loc= 3:11 - 3:14  range= 51-54   "arg"
  | Punctuator        loc= 3:14 - 3:15  range= 54-55   ";"
  | Punctuator        loc= 4:2 - 4:3    range= 58-59   "}"
  | Punctuator        loc= 4:3 - 4:4    range= 59-60   ","
  | Punctuator        loc= 5:0 - 5:1    range= 61-62   "}"
  | Punctuator        loc= 5:1 - 5:2    range= 62-63   ";"
  | Keyword           loc= 8:0 - 8:6    range= 95-101  "export"
  | Punctuator        loc= 8:7 - 8:8    range= 102-103 "{"
  | Identifier        loc= 8:9 - 8:12   range= 104-107 "obj"
  | Punctuator        loc= 8:13 - 8:14  range= 108-109 "}"
  | Punctuator        loc= 8:14 - 8:15  range= 109-110 ";"
   ,-[files/generic_arrow.ts:1:1]
 1 | ,-> const obj = {
 2 | |     fn: <T>(arg: T): T => {
 3 | |       return arg;
 4 | |     },
 5 | |   };
 6 | |
 7 | |   // A comment after the object
 8 | `-> export { obj };
   `----

  x tokens-plugin(tokens): Tokens and comments:
  | Keyword           loc= 1:0 - 1:5    range= 0-5     "const"
  | Identifier        loc= 1:6 - 1:9    range= 6-9     "obj"
  | Punctuator        loc= 1:10 - 1:11  range= 10-11   "="
  | Punctuator        loc= 1:12 - 1:13  range= 12-13   "{"
  | Identifier        loc= 2:2 - 2:4    range= 16-18   "fn"
  | Punctuator        loc= 2:4 - 2:5    range= 18-19   ":"
  | Punctuator        loc= 2:6 - 2:7    range= 20-21   "<"
  | Identifier        loc= 2:7 - 2:8    range= 21-22   "T"
  | Punctuator        loc= 2:8 - 2:9    range= 22-23   ">"
  | Punctuator        loc= 2:9 - 2:10   range= 23-24   "("
  | Identifier        loc= 2:10 - 2:13  range= 24-27   "arg"
  | Punctuator        loc= 2:13 - 2:14  range= 27-28   ":"
  | Identifier        loc= 2:15 - 2:16  range= 29-30   "T"
  | Punctuator        loc= 2:16 - 2:17  range= 30-31   ")"
  | Punctuator        loc= 2:17 - 2:18  range= 31-32   ":"
  | Identifier        loc= 2:19 - 2:20  range= 33-34   "T"
  | Punctuator        loc= 2:21 - 2:23  range= 35-37   "=>"
  | Punctuator        loc= 2:24 - 2:25  range= 38-39   "{"
  | Keyword           loc= 3:4 - 3:10   range= 44-50   "return"
  | Identifier        loc= 3:11 - 3:14  range= 51-54   "arg"
  | Punctuator        loc= 3:14 - 3:15  range= 54-55   ";"
  | Punctuator        loc= 4:2 - 4:3    range= 58-59   "}"
  | Punctuator        loc= 4:3 - 4:4    range= 59-60   ","
  | Punctuator        loc= 5:0 - 5:1    range= 61-62   "}"
  | Punctuator        loc= 5:1 - 5:2    range= 62-63   ";"
  | Line              loc= 7:0 - 7:29   range= 65-94   " A comment after the object"
  | Keyword           loc= 8:0 - 8:6    range= 95-101  "export"
  | Punctuator        loc= 8:7 - 8:8    range= 102-103 "{"
  | Identifier        loc= 8:9 - 8:12   range= 104-107 "obj"
  | Punctuator        loc= 8:13 - 8:14  range= 108-109 "}"
  | Punctuator        loc= 8:14 - 8:15  range= 109-110 ";"
   ,-[files/generic_arrow.ts:1:1]
 1 | ,-> const obj = {
 2 | |     fn: <T>(arg: T): T => {
 3 | |       return arg;
 4 | |     },
 5 | |   };
 6 | |
 7 | |   // A comment after the object
 8 | `-> export { obj };
   `----

  x tokens-plugin(tokens): Identifier ("obj")
   ,-[files/generic_arrow.ts:1:7]
 1 | const obj = {
   :       ^^^
 2 |   fn: <T>(arg: T): T => {
   `----

  x tokens-plugin(tokens): Punctuator ("=")
   ,-[files/generic_arrow.ts:1:11]
 1 | const obj = {
   :           ^
 2 |   fn: <T>(arg: T): T => {
   `----

  x tokens-plugin(tokens): Punctuator ("{")
   ,-[files/generic_arrow.ts:1:13]
 1 | const obj = {
   :             ^
 2 |   fn: <T>(arg: T): T => {
   `----

  x tokens-plugin(tokens): Identifier ("fn")
   ,-[files/generic_arrow.ts:2:3]
 1 | const obj = {
 2 |   fn: <T>(arg: T): T => {
   :   ^^
 3 |     return arg;
   `----

  x tokens-plugin(tokens): Punctuator (":")
   ,-[files/generic_arrow.ts:2:5]
 1 | const obj = {
 2 |   fn: <T>(arg: T): T => {
   :     ^
 3 |     return arg;
   `----

  x tokens-plugin(tokens): Punctuator ("<")
   ,-[files/generic_arrow.ts:2:7]
 1 | const obj = {
 2 |   fn: <T>(arg: T): T => {
   :       ^
 3 |     return arg;
   `----

  x tokens-plugin(tokens): Identifier ("T")
   ,-[files/generic_arrow.ts:2:8]
 1 | const obj = {
 2 |   fn: <T>(arg: T): T => {
   :        ^
 3 |     return arg;
   `----

  x tokens-plugin(tokens): Punctuator (">")
   ,-[files/generic_arrow.ts:2:9]
 1 | const obj = {
 2 |   fn: <T>(arg: T): T => {
   :         ^
 3 |     return arg;
   `----

  x tokens-plugin(tokens): Punctuator ("(")
   ,-[files/generic_arrow.ts:2:10]
 1 | const obj = {
 2 |   fn: <T>(arg: T): T => {
   :          ^
 3 |     return arg;
   `----

  x tokens-plugin(tokens): Identifier ("arg")
   ,-[files/generic_arrow.ts:2:11]
 1 | const obj = {
 2 |   fn: <T>(arg: T): T => {
   :           ^^^
 3 |     return arg;
   `----

  x tokens-plugin(tokens): Punctuator (":")
   ,-[files/generic_arrow.ts:2:14]
 1 | const obj = {
 2 |   fn: <T>(arg: T): T => {
   :              ^
 3 |     return arg;
   `----

  x tokens-plugin(tokens): Identifier ("T")
   ,-[files/generic_arrow.ts:2:16]
 1 | const obj = {
 2 |   fn: <T>(arg: T): T => {
   :                ^
 3 |     return arg;
   `----

  x tokens-plugin(tokens): Punctuator (")")
   ,-[files/generic_arrow.ts:2:17]
 1 | const obj = {
 2 |   fn: <T>(arg: T): T => {
   :                 ^
 3 |     return arg;
   `----

  x tokens-plugin(tokens): Punctuator (":")
   ,-[files/generic_arrow.ts:2:18]
 1 | const obj = {
 2 |   fn: <T>(arg: T): T => {
   :                  ^
 3 |     return arg;
   `----

  x tokens-plugin(tokens): Identifier ("T")
   ,-[files/generic_arrow.ts:2:20]
 1 | const obj = {
 2 |   fn: <T>(arg: T): T => {
   :                    ^
 3 |     return arg;
   `----

  x tokens-plugin(tokens): Punctuator ("=>")
   ,-[files/generic_arrow.ts:2:22]
 1 | const obj = {
 2 |   fn: <T>(arg: T): T => {
   :                      ^^
 3 |     return arg;
   `----

  x tokens-plugin(tokens): Punctuator ("{")
   ,-[files/generic_arrow.ts:2:25]
 1 | const obj = {
 2 |   fn: <T>(arg: T): T => {
   :                         ^
 3 |     return arg;
   `----

  x tokens-plugin(tokens): Keyword ("return")
   ,-[files/generic_arrow.ts:3:5]
 2 |   fn: <T>(arg: T): T => {
 3 |     return arg;
   :     ^^^^^^
 4 |   },
   `----

  x tokens-plugin(tokens): Identifier ("arg")
   ,-[files/generic_arrow.ts:3:12]
 2 |   fn: <T>(arg: T): T => {
 3 |     return arg;
   :            ^^^
 4 |   },
   `----

  x tokens-plugin(tokens): Punctuator (";")
   ,-[files/generic_arrow.ts:3:15]
 2 |   fn: <T>(arg: T): T => {
 3 |     return arg;
   :               ^
 4 |   },
   `----

  x tokens-plugin(tokens): Punctuator ("}")
   ,-[files/generic_arrow.ts:4:3]
 3 |     return arg;
 4 |   },
   :   ^
 5 | };
   `----

  x tokens-plugin(tokens): Punctuator (",")
   ,-[files/generic_arrow.ts:4:4]
 3 |     return arg;
 4 |   },
   :    ^
 5 | };
   `----

  x tokens-plugin(tokens): Punctuator ("}")
   ,-[files/generic_arrow.ts:5:1]
 4 |   },
 5 | };
   : ^
 6 |
   `----

  x tokens-plugin(tokens): Punctuator (";")
   ,-[files/generic_arrow.ts:5:2]
 4 |   },
 5 | };
   :  ^
 6 |
   `----

  x tokens-plugin(tokens): Line (" A comment after the object")
   ,-[files/generic_arrow.ts:7:1]
 6 |
 7 | // A comment after the object
   : ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
 8 | export { obj };
   `----

  x tokens-plugin(tokens): Keyword ("export")
   ,-[files/generic_arrow.ts:8:1]
 7 | // A comment after the object
 8 | export { obj };
   : ^^^^^^
   `----

  x tokens-plugin(tokens): Punctuator ("{")
   ,-[files/generic_arrow.ts:8:8]
 7 | // A comment after the object
 8 | export { obj };
   :        ^
   `----

  x tokens-plugin(tokens): Identifier ("obj")
   ,-[files/generic_arrow.ts:8:10]
 7 | // A comment after the object
 8 | export { obj };
   :          ^^^
   `----

  x tokens-plugin(tokens): Punctuator ("}")
   ,-[files/generic_arrow.ts:8:14]
 7 | // A comment after the object
 8 | export { obj };
   :              ^
   `----

  x tokens-plugin(tokens): Punctuator (";")
   ,-[files/generic_arrow.ts:8:15]
 7 | // A comment after the object
 8 | export { obj };
   :               ^
   `----

  x tokens-plugin(tokens): Line (" Leading comment")
   ,-[files/index.js:1:1]
 1 | // Leading comment
   : ^^^^^^^^^^^^^^^^^^
 2 |
   `----

  x tokens-plugin(tokens): Tokens:
  | Keyword           loc= 3:0 - 3:3    range= 20-23   "let"
  | Identifier        loc= 3:4 - 3:5    range= 24-25   "x"
  | Punctuator        loc= 3:6 - 3:7    range= 26-27   "="
  | Numeric           loc= 3:29 - 3:30  range= 49-50   "1"
  | Punctuator        loc= 3:30 - 3:31  range= 50-51   ";"
  | Keyword           loc= 6:0 - 6:3    range= 72-75   "let"
  | Identifier        loc= 6:4 - 6:5    range= 76-77   "y"
  | Punctuator        loc= 6:6 - 6:7    range= 78-79   "="
  | Numeric           loc= 6:8 - 6:9    range= 80-81   "2"
  | Punctuator        loc= 6:9 - 6:10   range= 81-82   ";"
   ,-[files/index.js:1:1]
 1 | ,-> // Leading comment
 2 | |
 3 | |   let x = /* inline comment */ 1;
 4 | |
 5 | |   // Another comment
 6 | |   let y = 2;
 7 | |
 8 | `-> // Trailing comment
   `----

  x tokens-plugin(tokens): Tokens and comments:
  | Line              loc= 1:0 - 1:18   range= 0-18    " Leading comment"
  | Keyword           loc= 3:0 - 3:3    range= 20-23   "let"
  | Identifier        loc= 3:4 - 3:5    range= 24-25   "x"
  | Punctuator        loc= 3:6 - 3:7    range= 26-27   "="
  | Block             loc= 3:8 - 3:28   range= 28-48   " inline comment "
  | Numeric           loc= 3:29 - 3:30  range= 49-50   "1"
  | Punctuator        loc= 3:30 - 3:31  range= 50-51   ";"
  | Line              loc= 5:0 - 5:18   range= 53-71   " Another comment"
  | Keyword           loc= 6:0 - 6:3    range= 72-75   "let"
  | Identifier        loc= 6:4 - 6:5    range= 76-77   "y"
  | Punctuator        loc= 6:6 - 6:7    range= 78-79   "="
  | Numeric           loc= 6:8 - 6:9    range= 80-81   "2"
  | Punctuator        loc= 6:9 - 6:10   range= 81-82   ";"
  | Line              loc= 8:0 - 8:19   range= 84-103  " Trailing comment"
   ,-[files/index.js:1:1]
 1 | ,-> // Leading comment
 2 | |
 3 | |   let x = /* inline comment */ 1;
 4 | |
 5 | |   // Another comment
 6 | |   let y = 2;
 7 | |
 8 | `-> // Trailing comment
   `----

  x tokens-plugin(tokens): Keyword ("let")
   ,-[files/index.js:3:1]
 2 |
 3 | let x = /* inline comment */ 1;
   : ^^^
 4 |
   `----

  x tokens-plugin(tokens): Identifier ("x")
   ,-[files/index.js:3:5]
 2 |
 3 | let x = /* inline comment */ 1;
   :     ^
 4 |
   `----

  x tokens-plugin(tokens): Punctuator ("=")
   ,-[files/index.js:3:7]
 2 |
 3 | let x = /* inline comment */ 1;
   :       ^
 4 |
   `----

  x tokens-plugin(tokens): Block (" inline comment ")
   ,-[files/index.js:3:9]
 2 |
 3 | let x = /* inline comment */ 1;
   :         ^^^^^^^^^^^^^^^^^^^^
 4 |
   `----

  x tokens-plugin(tokens): Numeric ("1")
   ,-[files/index.js:3:30]
 2 |
 3 | let x = /* inline comment */ 1;
   :                              ^
 4 |
   `----

  x tokens-plugin(tokens): Punctuator (";")
   ,-[files/index.js:3:31]
 2 |
 3 | let x = /* inline comment */ 1;
   :                               ^
 4 |
   `----

  x tokens-plugin(tokens): Line (" Another comment")
   ,-[files/index.js:5:1]
 4 |
 5 | // Another comment
   : ^^^^^^^^^^^^^^^^^^
 6 | let y = 2;
   `----

  x tokens-plugin(tokens): Keyword ("let")
   ,-[files/index.js:6:1]
 5 | // Another comment
 6 | let y = 2;
   : ^^^
 7 |
   `----

  x tokens-plugin(tokens): Identifier ("y")
   ,-[files/index.js:6:5]
 5 | // Another comment
 6 | let y = 2;
   :     ^
 7 |
   `----

  x tokens-plugin(tokens): Punctuator ("=")
   ,-[files/index.js:6:7]
 5 | // Another comment
 6 | let y = 2;
   :       ^
 7 |
   `----

  x tokens-plugin(tokens): Numeric ("2")
   ,-[files/index.js:6:9]
 5 | // Another comment
 6 | let y = 2;
   :         ^
 7 |
   `----

  x tokens-plugin(tokens): Punctuator (";")
   ,-[files/index.js:6:10]
 5 | // Another comment
 6 | let y = 2;
   :          ^
 7 |
   `----

  x tokens-plugin(tokens): Line (" Trailing comment")
   ,-[files/index.js:8:1]
 7 |
 8 | // Trailing comment
   : ^^^^^^^^^^^^^^^^^^^
   `----

Found 0 warnings and 16 errors.
Finished in Xms on 1 file with 1 rules using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
