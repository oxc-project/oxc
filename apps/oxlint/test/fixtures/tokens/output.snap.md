# Exit code
1

# stdout
```
  x tokens-plugin(tokens): Identifier ("a")
   ,-[files/bom.js:1:4]
 1 | ﻿a = b;
   : ^
   `----

  x tokens-plugin(tokens): Tokens and comments:
  | Identifier        loc= 1:0 - 1:1    range= 0-1     "a"
  | Punctuator        loc= 1:2 - 1:3    range= 2-3     "="
  | Identifier        loc= 1:4 - 1:5    range= 4-5     "b"
  | Punctuator        loc= 1:5 - 1:6    range= 5-6     ";"
   ,-[files/bom.js:1:4]
 1 | ﻿a = b;
   : ^^^^^^^
   `----

  x tokens-plugin(tokens): Tokens:
  | Identifier        loc= 1:0 - 1:1    range= 0-1     "a"
  | Punctuator        loc= 1:2 - 1:3    range= 2-3     "="
  | Identifier        loc= 1:4 - 1:5    range= 4-5     "b"
  | Punctuator        loc= 1:5 - 1:6    range= 5-6     ";"
   ,-[files/bom.js:1:4]
 1 | ﻿a = b;
   : ^^^^^^^
   `----

  x tokens-plugin(tokens): Punctuator ("=")
   ,-[files/bom.js:1:6]
 1 | ﻿a = b;
   :   ^
   `----

  x tokens-plugin(tokens): Identifier ("b")
   ,-[files/bom.js:1:8]
 1 | ﻿a = b;
   :     ^
   `----

  x tokens-plugin(tokens): Punctuator (";")
   ,-[files/bom.js:1:9]
 1 | ﻿a = b;
   :      ^
   `----

  x tokens-plugin(tokens): Keyword ("const")
   ,-[files/generic_arrow.ts:1:1]
 1 | const obj = {
   : ^^^^^
 2 |   fn: <T>(arg: T): T => {
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
  | RegularExpression loc= 6:8 - 6:15   range= 80-87   "/abc/gu"
  | Punctuator        loc= 6:15 - 6:16  range= 87-88   ";"
  | Line              loc= 8:0 - 8:19   range= 90-109  " Trailing comment"
   ,-[files/index.js:1:1]
 1 | ,-> // Leading comment
 2 | |   
 3 | |   let x = /* inline comment */ 1;
 4 | |   
 5 | |   // Another comment
 6 | |   let y = /abc/gu;
 7 | |   
 8 | `-> // Trailing comment
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
  | RegularExpression loc= 6:8 - 6:15   range= 80-87   "/abc/gu"
  | Punctuator        loc= 6:15 - 6:16  range= 87-88   ";"
   ,-[files/index.js:1:1]
 1 | ,-> // Leading comment
 2 | |   
 3 | |   let x = /* inline comment */ 1;
 4 | |   
 5 | |   // Another comment
 6 | |   let y = /abc/gu;
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
 6 | let y = /abc/gu;
   `----

  x tokens-plugin(tokens): Keyword ("let")
   ,-[files/index.js:6:1]
 5 | // Another comment
 6 | let y = /abc/gu;
   : ^^^
 7 | 
   `----

  x tokens-plugin(tokens): Identifier ("y")
   ,-[files/index.js:6:5]
 5 | // Another comment
 6 | let y = /abc/gu;
   :     ^
 7 | 
   `----

  x tokens-plugin(tokens): Punctuator ("=")
   ,-[files/index.js:6:7]
 5 | // Another comment
 6 | let y = /abc/gu;
   :       ^
 7 | 
   `----

  x tokens-plugin(tokens): RegularExpression ("/abc/gu")
  |   regex: {"pattern":"abc","flags":"gu"}
   ,-[files/index.js:6:9]
 5 | // Another comment
 6 | let y = /abc/gu;
   :         ^^^^^^^
 7 | 
   `----

  x tokens-plugin(tokens): Punctuator (";")
   ,-[files/index.js:6:16]
 5 | // Another comment
 6 | let y = /abc/gu;
   :                ^
 7 | 
   `----

  x tokens-plugin(tokens): Line (" Trailing comment")
   ,-[files/index.js:8:1]
 7 | 
 8 | // Trailing comment
   : ^^^^^^^^^^^^^^^^^^^
   `----

  x tokens-plugin(tokens): Keyword ("const")
   ,-[files/jsx_element.tsx:1:1]
 1 | const Component = () => {
   : ^^^^^
 2 |   return <div className="test">Hello</div>;
   `----

  x tokens-plugin(tokens): Tokens and comments:
  | Keyword           loc= 1:0 - 1:5    range= 0-5     "const"
  | Identifier        loc= 1:6 - 1:15   range= 6-15    "Component"
  | Punctuator        loc= 1:16 - 1:17  range= 16-17   "="
  | Punctuator        loc= 1:18 - 1:19  range= 18-19   "("
  | Punctuator        loc= 1:19 - 1:20  range= 19-20   ")"
  | Punctuator        loc= 1:21 - 1:23  range= 21-23   "=>"
  | Punctuator        loc= 1:24 - 1:25  range= 24-25   "{"
  | Keyword           loc= 2:2 - 2:8    range= 28-34   "return"
  | Punctuator        loc= 2:9 - 2:10   range= 35-36   "<"
  | JSXIdentifier     loc= 2:10 - 2:13  range= 36-39   "div"
  | JSXIdentifier     loc= 2:14 - 2:23  range= 40-49   "className"
  | Punctuator        loc= 2:23 - 2:24  range= 49-50   "="
  | JSXText           loc= 2:24 - 2:30  range= 50-56   "\"test\""
  | Punctuator        loc= 2:30 - 2:31  range= 56-57   ">"
  | JSXText           loc= 2:31 - 2:36  range= 57-62   "Hello"
  | Punctuator        loc= 2:36 - 2:37  range= 62-63   "<"
  | Punctuator        loc= 2:37 - 2:38  range= 63-64   "/"
  | JSXIdentifier     loc= 2:38 - 2:41  range= 64-67   "div"
  | Punctuator        loc= 2:41 - 2:42  range= 67-68   ">"
  | Punctuator        loc= 2:42 - 2:43  range= 68-69   ";"
  | Punctuator        loc= 3:0 - 3:1    range= 70-71   "}"
  | Punctuator        loc= 3:1 - 3:2    range= 71-72   ";"
  | Line              loc= 5:0 - 5:32   range= 74-106  " A comment after the component"
  | Keyword           loc= 6:0 - 6:6    range= 107-113 "export"
  | Punctuator        loc= 6:7 - 6:8    range= 114-115 "{"
  | Identifier        loc= 6:9 - 6:18   range= 116-125 "Component"
  | Punctuator        loc= 6:19 - 6:20  range= 126-127 "}"
  | Punctuator        loc= 6:20 - 6:21  range= 127-128 ";"
   ,-[files/jsx_element.tsx:1:1]
 1 | ,-> const Component = () => {
 2 | |     return <div className="test">Hello</div>;
 3 | |   };
 4 | |   
 5 | |   // A comment after the component
 6 | `-> export { Component };
   `----

  x tokens-plugin(tokens): Tokens:
  | Keyword           loc= 1:0 - 1:5    range= 0-5     "const"
  | Identifier        loc= 1:6 - 1:15   range= 6-15    "Component"
  | Punctuator        loc= 1:16 - 1:17  range= 16-17   "="
  | Punctuator        loc= 1:18 - 1:19  range= 18-19   "("
  | Punctuator        loc= 1:19 - 1:20  range= 19-20   ")"
  | Punctuator        loc= 1:21 - 1:23  range= 21-23   "=>"
  | Punctuator        loc= 1:24 - 1:25  range= 24-25   "{"
  | Keyword           loc= 2:2 - 2:8    range= 28-34   "return"
  | Punctuator        loc= 2:9 - 2:10   range= 35-36   "<"
  | JSXIdentifier     loc= 2:10 - 2:13  range= 36-39   "div"
  | JSXIdentifier     loc= 2:14 - 2:23  range= 40-49   "className"
  | Punctuator        loc= 2:23 - 2:24  range= 49-50   "="
  | JSXText           loc= 2:24 - 2:30  range= 50-56   "\"test\""
  | Punctuator        loc= 2:30 - 2:31  range= 56-57   ">"
  | JSXText           loc= 2:31 - 2:36  range= 57-62   "Hello"
  | Punctuator        loc= 2:36 - 2:37  range= 62-63   "<"
  | Punctuator        loc= 2:37 - 2:38  range= 63-64   "/"
  | JSXIdentifier     loc= 2:38 - 2:41  range= 64-67   "div"
  | Punctuator        loc= 2:41 - 2:42  range= 67-68   ">"
  | Punctuator        loc= 2:42 - 2:43  range= 68-69   ";"
  | Punctuator        loc= 3:0 - 3:1    range= 70-71   "}"
  | Punctuator        loc= 3:1 - 3:2    range= 71-72   ";"
  | Keyword           loc= 6:0 - 6:6    range= 107-113 "export"
  | Punctuator        loc= 6:7 - 6:8    range= 114-115 "{"
  | Identifier        loc= 6:9 - 6:18   range= 116-125 "Component"
  | Punctuator        loc= 6:19 - 6:20  range= 126-127 "}"
  | Punctuator        loc= 6:20 - 6:21  range= 127-128 ";"
   ,-[files/jsx_element.tsx:1:1]
 1 | ,-> const Component = () => {
 2 | |     return <div className="test">Hello</div>;
 3 | |   };
 4 | |   
 5 | |   // A comment after the component
 6 | `-> export { Component };
   `----

  x tokens-plugin(tokens): Identifier ("Component")
   ,-[files/jsx_element.tsx:1:7]
 1 | const Component = () => {
   :       ^^^^^^^^^
 2 |   return <div className="test">Hello</div>;
   `----

  x tokens-plugin(tokens): Punctuator ("=")
   ,-[files/jsx_element.tsx:1:17]
 1 | const Component = () => {
   :                 ^
 2 |   return <div className="test">Hello</div>;
   `----

  x tokens-plugin(tokens): Punctuator ("(")
   ,-[files/jsx_element.tsx:1:19]
 1 | const Component = () => {
   :                   ^
 2 |   return <div className="test">Hello</div>;
   `----

  x tokens-plugin(tokens): Punctuator (")")
   ,-[files/jsx_element.tsx:1:20]
 1 | const Component = () => {
   :                    ^
 2 |   return <div className="test">Hello</div>;
   `----

  x tokens-plugin(tokens): Punctuator ("=>")
   ,-[files/jsx_element.tsx:1:22]
 1 | const Component = () => {
   :                      ^^
 2 |   return <div className="test">Hello</div>;
   `----

  x tokens-plugin(tokens): Punctuator ("{")
   ,-[files/jsx_element.tsx:1:25]
 1 | const Component = () => {
   :                         ^
 2 |   return <div className="test">Hello</div>;
   `----

  x tokens-plugin(tokens): Keyword ("return")
   ,-[files/jsx_element.tsx:2:3]
 1 | const Component = () => {
 2 |   return <div className="test">Hello</div>;
   :   ^^^^^^
 3 | };
   `----

  x tokens-plugin(tokens): Punctuator ("<")
   ,-[files/jsx_element.tsx:2:10]
 1 | const Component = () => {
 2 |   return <div className="test">Hello</div>;
   :          ^
 3 | };
   `----

  x tokens-plugin(tokens): JSXIdentifier ("div")
   ,-[files/jsx_element.tsx:2:11]
 1 | const Component = () => {
 2 |   return <div className="test">Hello</div>;
   :           ^^^
 3 | };
   `----

  x tokens-plugin(tokens): JSXIdentifier ("className")
   ,-[files/jsx_element.tsx:2:15]
 1 | const Component = () => {
 2 |   return <div className="test">Hello</div>;
   :               ^^^^^^^^^
 3 | };
   `----

  x tokens-plugin(tokens): Punctuator ("=")
   ,-[files/jsx_element.tsx:2:24]
 1 | const Component = () => {
 2 |   return <div className="test">Hello</div>;
   :                        ^
 3 | };
   `----

  x tokens-plugin(tokens): JSXText ("\"test\"")
   ,-[files/jsx_element.tsx:2:25]
 1 | const Component = () => {
 2 |   return <div className="test">Hello</div>;
   :                         ^^^^^^
 3 | };
   `----

  x tokens-plugin(tokens): Punctuator (">")
   ,-[files/jsx_element.tsx:2:31]
 1 | const Component = () => {
 2 |   return <div className="test">Hello</div>;
   :                               ^
 3 | };
   `----

  x tokens-plugin(tokens): JSXText ("Hello")
   ,-[files/jsx_element.tsx:2:32]
 1 | const Component = () => {
 2 |   return <div className="test">Hello</div>;
   :                                ^^^^^
 3 | };
   `----

  x tokens-plugin(tokens): Punctuator ("<")
   ,-[files/jsx_element.tsx:2:37]
 1 | const Component = () => {
 2 |   return <div className="test">Hello</div>;
   :                                     ^
 3 | };
   `----

  x tokens-plugin(tokens): Punctuator ("/")
   ,-[files/jsx_element.tsx:2:38]
 1 | const Component = () => {
 2 |   return <div className="test">Hello</div>;
   :                                      ^
 3 | };
   `----

  x tokens-plugin(tokens): JSXIdentifier ("div")
   ,-[files/jsx_element.tsx:2:39]
 1 | const Component = () => {
 2 |   return <div className="test">Hello</div>;
   :                                       ^^^
 3 | };
   `----

  x tokens-plugin(tokens): Punctuator (">")
   ,-[files/jsx_element.tsx:2:42]
 1 | const Component = () => {
 2 |   return <div className="test">Hello</div>;
   :                                          ^
 3 | };
   `----

  x tokens-plugin(tokens): Punctuator (";")
   ,-[files/jsx_element.tsx:2:43]
 1 | const Component = () => {
 2 |   return <div className="test">Hello</div>;
   :                                           ^
 3 | };
   `----

  x tokens-plugin(tokens): Punctuator ("}")
   ,-[files/jsx_element.tsx:3:1]
 2 |   return <div className="test">Hello</div>;
 3 | };
   : ^
 4 | 
   `----

  x tokens-plugin(tokens): Punctuator (";")
   ,-[files/jsx_element.tsx:3:2]
 2 |   return <div className="test">Hello</div>;
 3 | };
   :  ^
 4 | 
   `----

  x tokens-plugin(tokens): Line (" A comment after the component")
   ,-[files/jsx_element.tsx:5:1]
 4 | 
 5 | // A comment after the component
   : ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
 6 | export { Component };
   `----

  x tokens-plugin(tokens): Keyword ("export")
   ,-[files/jsx_element.tsx:6:1]
 5 | // A comment after the component
 6 | export { Component };
   : ^^^^^^
   `----

  x tokens-plugin(tokens): Punctuator ("{")
   ,-[files/jsx_element.tsx:6:8]
 5 | // A comment after the component
 6 | export { Component };
   :        ^
   `----

  x tokens-plugin(tokens): Identifier ("Component")
   ,-[files/jsx_element.tsx:6:10]
 5 | // A comment after the component
 6 | export { Component };
   :          ^^^^^^^^^
   `----

  x tokens-plugin(tokens): Punctuator ("}")
   ,-[files/jsx_element.tsx:6:20]
 5 | // A comment after the component
 6 | export { Component };
   :                    ^
   `----

  x tokens-plugin(tokens): Punctuator (";")
   ,-[files/jsx_element.tsx:6:21]
 5 | // A comment after the component
 6 | export { Component };
   :                     ^
   `----

  x tokens-plugin(tokens): Keyword ("const")
   ,-[files/keywords.js:1:1]
 1 | const obj = {
   : ^^^^^
 2 |   // Identifier tokens
   `----

  x tokens-plugin(tokens): Tokens and comments:
  | Keyword           loc= 1:0 - 1:5    range= 0-5     "const"
  | Identifier        loc= 1:6 - 1:9    range= 6-9     "obj"
  | Punctuator        loc= 1:10 - 1:11  range= 10-11   "="
  | Punctuator        loc= 1:12 - 1:13  range= 12-13   "{"
  | Line              loc= 2:2 - 2:22   range= 16-36   " Identifier tokens"
  | Identifier        loc= 3:2 - 3:5    range= 39-42   "foo"
  | Punctuator        loc= 3:5 - 3:6    range= 42-43   ":"
  | Identifier        loc= 3:7 - 3:10   range= 44-47   "foo"
  | Punctuator        loc= 3:10 - 3:11  range= 47-48   ","
  | Identifier        loc= 4:2 - 4:7    range= 51-56   "async"
  | Punctuator        loc= 4:7 - 4:8    range= 56-57   ":"
  | Identifier        loc= 4:9 - 4:14   range= 58-63   "async"
  | Punctuator        loc= 4:14 - 4:15  range= 63-64   ","
  | Line              loc= 5:2 - 5:19   range= 67-84   " Keyword tokens"
  | Keyword           loc= 6:2 - 6:5    range= 87-90   "let"
  | Punctuator        loc= 6:5 - 6:6    range= 90-91   ":"
  | Keyword           loc= 6:7 - 6:10   range= 92-95   "let"
  | Punctuator        loc= 6:10 - 6:11  range= 95-96   ","
  | Keyword           loc= 7:2 - 7:8    range= 99-105  "static"
  | Punctuator        loc= 7:8 - 7:9    range= 105-106 ":"
  | Keyword           loc= 7:10 - 7:16  range= 107-113 "static"
  | Punctuator        loc= 7:16 - 7:17  range= 113-114 ","
  | Keyword           loc= 8:2 - 8:7    range= 117-122 "yield"
  | Punctuator        loc= 8:7 - 8:8    range= 122-123 ":"
  | Keyword           loc= 8:9 - 8:14   range= 124-129 "yield"
  | Punctuator        loc= 8:14 - 8:15  range= 129-130 ","
  | Punctuator        loc= 9:0 - 9:1    range= 131-132 "}"
  | Punctuator        loc= 9:1 - 9:2    range= 132-133 ";"
   ,-[files/keywords.js:1:1]
 1 | ,-> const obj = {
 2 | |     // Identifier tokens
 3 | |     foo: foo,
 4 | |     async: async,
 5 | |     // Keyword tokens
 6 | |     let: let,
 7 | |     static: static,
 8 | |     yield: yield,
 9 | `-> };
   `----

  x tokens-plugin(tokens): Tokens:
  | Keyword           loc= 1:0 - 1:5    range= 0-5     "const"
  | Identifier        loc= 1:6 - 1:9    range= 6-9     "obj"
  | Punctuator        loc= 1:10 - 1:11  range= 10-11   "="
  | Punctuator        loc= 1:12 - 1:13  range= 12-13   "{"
  | Identifier        loc= 3:2 - 3:5    range= 39-42   "foo"
  | Punctuator        loc= 3:5 - 3:6    range= 42-43   ":"
  | Identifier        loc= 3:7 - 3:10   range= 44-47   "foo"
  | Punctuator        loc= 3:10 - 3:11  range= 47-48   ","
  | Identifier        loc= 4:2 - 4:7    range= 51-56   "async"
  | Punctuator        loc= 4:7 - 4:8    range= 56-57   ":"
  | Identifier        loc= 4:9 - 4:14   range= 58-63   "async"
  | Punctuator        loc= 4:14 - 4:15  range= 63-64   ","
  | Keyword           loc= 6:2 - 6:5    range= 87-90   "let"
  | Punctuator        loc= 6:5 - 6:6    range= 90-91   ":"
  | Keyword           loc= 6:7 - 6:10   range= 92-95   "let"
  | Punctuator        loc= 6:10 - 6:11  range= 95-96   ","
  | Keyword           loc= 7:2 - 7:8    range= 99-105  "static"
  | Punctuator        loc= 7:8 - 7:9    range= 105-106 ":"
  | Keyword           loc= 7:10 - 7:16  range= 107-113 "static"
  | Punctuator        loc= 7:16 - 7:17  range= 113-114 ","
  | Keyword           loc= 8:2 - 8:7    range= 117-122 "yield"
  | Punctuator        loc= 8:7 - 8:8    range= 122-123 ":"
  | Keyword           loc= 8:9 - 8:14   range= 124-129 "yield"
  | Punctuator        loc= 8:14 - 8:15  range= 129-130 ","
  | Punctuator        loc= 9:0 - 9:1    range= 131-132 "}"
  | Punctuator        loc= 9:1 - 9:2    range= 132-133 ";"
   ,-[files/keywords.js:1:1]
 1 | ,-> const obj = {
 2 | |     // Identifier tokens
 3 | |     foo: foo,
 4 | |     async: async,
 5 | |     // Keyword tokens
 6 | |     let: let,
 7 | |     static: static,
 8 | |     yield: yield,
 9 | `-> };
   `----

  x tokens-plugin(tokens): Identifier ("obj")
   ,-[files/keywords.js:1:7]
 1 | const obj = {
   :       ^^^
 2 |   // Identifier tokens
   `----

  x tokens-plugin(tokens): Punctuator ("=")
   ,-[files/keywords.js:1:11]
 1 | const obj = {
   :           ^
 2 |   // Identifier tokens
   `----

  x tokens-plugin(tokens): Punctuator ("{")
   ,-[files/keywords.js:1:13]
 1 | const obj = {
   :             ^
 2 |   // Identifier tokens
   `----

  x tokens-plugin(tokens): Line (" Identifier tokens")
   ,-[files/keywords.js:2:3]
 1 | const obj = {
 2 |   // Identifier tokens
   :   ^^^^^^^^^^^^^^^^^^^^
 3 |   foo: foo,
   `----

  x tokens-plugin(tokens): Identifier ("foo")
   ,-[files/keywords.js:3:3]
 2 |   // Identifier tokens
 3 |   foo: foo,
   :   ^^^
 4 |   async: async,
   `----

  x tokens-plugin(tokens): Punctuator (":")
   ,-[files/keywords.js:3:6]
 2 |   // Identifier tokens
 3 |   foo: foo,
   :      ^
 4 |   async: async,
   `----

  x tokens-plugin(tokens): Identifier ("foo")
   ,-[files/keywords.js:3:8]
 2 |   // Identifier tokens
 3 |   foo: foo,
   :        ^^^
 4 |   async: async,
   `----

  x tokens-plugin(tokens): Punctuator (",")
   ,-[files/keywords.js:3:11]
 2 |   // Identifier tokens
 3 |   foo: foo,
   :           ^
 4 |   async: async,
   `----

  x tokens-plugin(tokens): Identifier ("async")
   ,-[files/keywords.js:4:3]
 3 |   foo: foo,
 4 |   async: async,
   :   ^^^^^
 5 |   // Keyword tokens
   `----

  x tokens-plugin(tokens): Punctuator (":")
   ,-[files/keywords.js:4:8]
 3 |   foo: foo,
 4 |   async: async,
   :        ^
 5 |   // Keyword tokens
   `----

  x tokens-plugin(tokens): Identifier ("async")
   ,-[files/keywords.js:4:10]
 3 |   foo: foo,
 4 |   async: async,
   :          ^^^^^
 5 |   // Keyword tokens
   `----

  x tokens-plugin(tokens): Punctuator (",")
   ,-[files/keywords.js:4:15]
 3 |   foo: foo,
 4 |   async: async,
   :               ^
 5 |   // Keyword tokens
   `----

  x tokens-plugin(tokens): Line (" Keyword tokens")
   ,-[files/keywords.js:5:3]
 4 |   async: async,
 5 |   // Keyword tokens
   :   ^^^^^^^^^^^^^^^^^
 6 |   let: let,
   `----

  x tokens-plugin(tokens): Keyword ("let")
   ,-[files/keywords.js:6:3]
 5 |   // Keyword tokens
 6 |   let: let,
   :   ^^^
 7 |   static: static,
   `----

  x tokens-plugin(tokens): Punctuator (":")
   ,-[files/keywords.js:6:6]
 5 |   // Keyword tokens
 6 |   let: let,
   :      ^
 7 |   static: static,
   `----

  x tokens-plugin(tokens): Keyword ("let")
   ,-[files/keywords.js:6:8]
 5 |   // Keyword tokens
 6 |   let: let,
   :        ^^^
 7 |   static: static,
   `----

  x tokens-plugin(tokens): Punctuator (",")
   ,-[files/keywords.js:6:11]
 5 |   // Keyword tokens
 6 |   let: let,
   :           ^
 7 |   static: static,
   `----

  x tokens-plugin(tokens): Keyword ("static")
   ,-[files/keywords.js:7:3]
 6 |   let: let,
 7 |   static: static,
   :   ^^^^^^
 8 |   yield: yield,
   `----

  x tokens-plugin(tokens): Punctuator (":")
   ,-[files/keywords.js:7:9]
 6 |   let: let,
 7 |   static: static,
   :         ^
 8 |   yield: yield,
   `----

  x tokens-plugin(tokens): Keyword ("static")
   ,-[files/keywords.js:7:11]
 6 |   let: let,
 7 |   static: static,
   :           ^^^^^^
 8 |   yield: yield,
   `----

  x tokens-plugin(tokens): Punctuator (",")
   ,-[files/keywords.js:7:17]
 6 |   let: let,
 7 |   static: static,
   :                 ^
 8 |   yield: yield,
   `----

  x tokens-plugin(tokens): Keyword ("yield")
   ,-[files/keywords.js:8:3]
 7 |   static: static,
 8 |   yield: yield,
   :   ^^^^^
 9 | };
   `----

  x tokens-plugin(tokens): Punctuator (":")
   ,-[files/keywords.js:8:8]
 7 |   static: static,
 8 |   yield: yield,
   :        ^
 9 | };
   `----

  x tokens-plugin(tokens): Keyword ("yield")
   ,-[files/keywords.js:8:10]
 7 |   static: static,
 8 |   yield: yield,
   :          ^^^^^
 9 | };
   `----

  x tokens-plugin(tokens): Punctuator (",")
   ,-[files/keywords.js:8:15]
 7 |   static: static,
 8 |   yield: yield,
   :               ^
 9 | };
   `----

  x tokens-plugin(tokens): Punctuator ("}")
   ,-[files/keywords.js:9:1]
 8 |   yield: yield,
 9 | };
   : ^
   `----

  x tokens-plugin(tokens): Punctuator (";")
   ,-[files/keywords.js:9:2]
 8 |   yield: yield,
 9 | };
   :  ^
   `----

  x tokens-plugin(tokens): Line (" `<<` is disambiguated: speculatively tried as `<` for type args, fails, rewinds to `<<`")
   ,-[files/ts_angle_relex.ts:1:1]
 1 | // `<<` is disambiguated: speculatively tried as `<` for type args, fails, rewinds to `<<`
   : ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
 2 | const a = n << 2;
   `----

  x tokens-plugin(tokens): Tokens and comments:
  | Line              loc= 1:0 - 1:90   range= 0-90    " `<<` is disambiguated: speculatively tried as `<` for type args, fails, rewinds to `<<`"
  | Keyword           loc= 2:0 - 2:5    range= 91-96   "const"
  | Identifier        loc= 2:6 - 2:7    range= 97-98   "a"
  | Punctuator        loc= 2:8 - 2:9    range= 99-100  "="
  | Identifier        loc= 2:10 - 2:11  range= 101-102 "n"
  | Punctuator        loc= 2:12 - 2:14  range= 103-105 "<<"
  | Numeric           loc= 2:15 - 2:16  range= 106-107 "2"
  | Punctuator        loc= 2:16 - 2:17  range= 107-108 ";"
  | Line              loc= 4:0 - 4:52   range= 110-162 " Successful type argument parsing with `<` and `>`"
  | Keyword           loc= 5:0 - 5:5    range= 163-168 "const"
  | Identifier        loc= 5:6 - 5:7    range= 169-170 "b"
  | Punctuator        loc= 5:8 - 5:9    range= 171-172 "="
  | Identifier        loc= 5:10 - 5:12  range= 173-175 "id"
  | Punctuator        loc= 5:12 - 5:13  range= 175-176 "<"
  | Identifier        loc= 5:13 - 5:19  range= 176-182 "number"
  | Punctuator        loc= 5:19 - 5:20  range= 182-183 ">"
  | Punctuator        loc= 5:20 - 5:21  range= 183-184 "("
  | Numeric           loc= 5:21 - 5:23  range= 184-186 "42"
  | Punctuator        loc= 5:23 - 5:24  range= 186-187 ")"
  | Punctuator        loc= 5:24 - 5:25  range= 187-188 ";"
  | Line              loc= 7:0 - 7:88   range= 190-278 " `>` after type args is disambiguated: speculatively tried as end of type args, fails,"
  | Line              loc= 8:0 - 8:48   range= 279-327 " rewinds to binary expression `n < (1 >> (0))`"
  | Keyword           loc= 9:0 - 9:5    range= 328-333 "const"
  | Identifier        loc= 9:6 - 9:7    range= 334-335 "c"
  | Punctuator        loc= 9:8 - 9:9    range= 336-337 "="
  | Identifier        loc= 9:10 - 9:11  range= 338-339 "n"
  | Punctuator        loc= 9:11 - 9:12  range= 339-340 "<"
  | Numeric           loc= 9:12 - 9:13  range= 340-341 "1"
  | Punctuator        loc= 9:13 - 9:15  range= 341-343 ">>"
  | Punctuator        loc= 9:15 - 9:16  range= 343-344 "("
  | Numeric           loc= 9:16 - 9:17  range= 344-345 "0"
  | Punctuator        loc= 9:17 - 9:18  range= 345-346 ")"
  | Punctuator        loc= 9:18 - 9:19  range= 346-347 ";"
   ,-[files/ts_angle_relex.ts:1:1]
 1 | ,-> // `<<` is disambiguated: speculatively tried as `<` for type args, fails, rewinds to `<<`
 2 | |   const a = n << 2;
 3 | |   
 4 | |   // Successful type argument parsing with `<` and `>`
 5 | |   const b = id<number>(42);
 6 | |   
 7 | |   // `>` after type args is disambiguated: speculatively tried as end of type args, fails,
 8 | |   // rewinds to binary expression `n < (1 >> (0))`
 9 | `-> const c = n<1>>(0);
   `----

  x tokens-plugin(tokens): Tokens:
  | Keyword           loc= 2:0 - 2:5    range= 91-96   "const"
  | Identifier        loc= 2:6 - 2:7    range= 97-98   "a"
  | Punctuator        loc= 2:8 - 2:9    range= 99-100  "="
  | Identifier        loc= 2:10 - 2:11  range= 101-102 "n"
  | Punctuator        loc= 2:12 - 2:14  range= 103-105 "<<"
  | Numeric           loc= 2:15 - 2:16  range= 106-107 "2"
  | Punctuator        loc= 2:16 - 2:17  range= 107-108 ";"
  | Keyword           loc= 5:0 - 5:5    range= 163-168 "const"
  | Identifier        loc= 5:6 - 5:7    range= 169-170 "b"
  | Punctuator        loc= 5:8 - 5:9    range= 171-172 "="
  | Identifier        loc= 5:10 - 5:12  range= 173-175 "id"
  | Punctuator        loc= 5:12 - 5:13  range= 175-176 "<"
  | Identifier        loc= 5:13 - 5:19  range= 176-182 "number"
  | Punctuator        loc= 5:19 - 5:20  range= 182-183 ">"
  | Punctuator        loc= 5:20 - 5:21  range= 183-184 "("
  | Numeric           loc= 5:21 - 5:23  range= 184-186 "42"
  | Punctuator        loc= 5:23 - 5:24  range= 186-187 ")"
  | Punctuator        loc= 5:24 - 5:25  range= 187-188 ";"
  | Keyword           loc= 9:0 - 9:5    range= 328-333 "const"
  | Identifier        loc= 9:6 - 9:7    range= 334-335 "c"
  | Punctuator        loc= 9:8 - 9:9    range= 336-337 "="
  | Identifier        loc= 9:10 - 9:11  range= 338-339 "n"
  | Punctuator        loc= 9:11 - 9:12  range= 339-340 "<"
  | Numeric           loc= 9:12 - 9:13  range= 340-341 "1"
  | Punctuator        loc= 9:13 - 9:15  range= 341-343 ">>"
  | Punctuator        loc= 9:15 - 9:16  range= 343-344 "("
  | Numeric           loc= 9:16 - 9:17  range= 344-345 "0"
  | Punctuator        loc= 9:17 - 9:18  range= 345-346 ")"
  | Punctuator        loc= 9:18 - 9:19  range= 346-347 ";"
   ,-[files/ts_angle_relex.ts:1:1]
 1 | ,-> // `<<` is disambiguated: speculatively tried as `<` for type args, fails, rewinds to `<<`
 2 | |   const a = n << 2;
 3 | |   
 4 | |   // Successful type argument parsing with `<` and `>`
 5 | |   const b = id<number>(42);
 6 | |   
 7 | |   // `>` after type args is disambiguated: speculatively tried as end of type args, fails,
 8 | |   // rewinds to binary expression `n < (1 >> (0))`
 9 | `-> const c = n<1>>(0);
   `----

  x tokens-plugin(tokens): Keyword ("const")
   ,-[files/ts_angle_relex.ts:2:1]
 1 | // `<<` is disambiguated: speculatively tried as `<` for type args, fails, rewinds to `<<`
 2 | const a = n << 2;
   : ^^^^^
 3 | 
   `----

  x tokens-plugin(tokens): Identifier ("a")
   ,-[files/ts_angle_relex.ts:2:7]
 1 | // `<<` is disambiguated: speculatively tried as `<` for type args, fails, rewinds to `<<`
 2 | const a = n << 2;
   :       ^
 3 | 
   `----

  x tokens-plugin(tokens): Punctuator ("=")
   ,-[files/ts_angle_relex.ts:2:9]
 1 | // `<<` is disambiguated: speculatively tried as `<` for type args, fails, rewinds to `<<`
 2 | const a = n << 2;
   :         ^
 3 | 
   `----

  x tokens-plugin(tokens): Identifier ("n")
   ,-[files/ts_angle_relex.ts:2:11]
 1 | // `<<` is disambiguated: speculatively tried as `<` for type args, fails, rewinds to `<<`
 2 | const a = n << 2;
   :           ^
 3 | 
   `----

  x tokens-plugin(tokens): Punctuator ("<<")
   ,-[files/ts_angle_relex.ts:2:13]
 1 | // `<<` is disambiguated: speculatively tried as `<` for type args, fails, rewinds to `<<`
 2 | const a = n << 2;
   :             ^^
 3 | 
   `----

  x tokens-plugin(tokens): Numeric ("2")
   ,-[files/ts_angle_relex.ts:2:16]
 1 | // `<<` is disambiguated: speculatively tried as `<` for type args, fails, rewinds to `<<`
 2 | const a = n << 2;
   :                ^
 3 | 
   `----

  x tokens-plugin(tokens): Punctuator (";")
   ,-[files/ts_angle_relex.ts:2:17]
 1 | // `<<` is disambiguated: speculatively tried as `<` for type args, fails, rewinds to `<<`
 2 | const a = n << 2;
   :                 ^
 3 | 
   `----

  x tokens-plugin(tokens): Line (" Successful type argument parsing with `<` and `>`")
   ,-[files/ts_angle_relex.ts:4:1]
 3 | 
 4 | // Successful type argument parsing with `<` and `>`
   : ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
 5 | const b = id<number>(42);
   `----

  x tokens-plugin(tokens): Keyword ("const")
   ,-[files/ts_angle_relex.ts:5:1]
 4 | // Successful type argument parsing with `<` and `>`
 5 | const b = id<number>(42);
   : ^^^^^
 6 | 
   `----

  x tokens-plugin(tokens): Identifier ("b")
   ,-[files/ts_angle_relex.ts:5:7]
 4 | // Successful type argument parsing with `<` and `>`
 5 | const b = id<number>(42);
   :       ^
 6 | 
   `----

  x tokens-plugin(tokens): Punctuator ("=")
   ,-[files/ts_angle_relex.ts:5:9]
 4 | // Successful type argument parsing with `<` and `>`
 5 | const b = id<number>(42);
   :         ^
 6 | 
   `----

  x tokens-plugin(tokens): Identifier ("id")
   ,-[files/ts_angle_relex.ts:5:11]
 4 | // Successful type argument parsing with `<` and `>`
 5 | const b = id<number>(42);
   :           ^^
 6 | 
   `----

  x tokens-plugin(tokens): Punctuator ("<")
   ,-[files/ts_angle_relex.ts:5:13]
 4 | // Successful type argument parsing with `<` and `>`
 5 | const b = id<number>(42);
   :             ^
 6 | 
   `----

  x tokens-plugin(tokens): Identifier ("number")
   ,-[files/ts_angle_relex.ts:5:14]
 4 | // Successful type argument parsing with `<` and `>`
 5 | const b = id<number>(42);
   :              ^^^^^^
 6 | 
   `----

  x tokens-plugin(tokens): Punctuator (">")
   ,-[files/ts_angle_relex.ts:5:20]
 4 | // Successful type argument parsing with `<` and `>`
 5 | const b = id<number>(42);
   :                    ^
 6 | 
   `----

  x tokens-plugin(tokens): Punctuator ("(")
   ,-[files/ts_angle_relex.ts:5:21]
 4 | // Successful type argument parsing with `<` and `>`
 5 | const b = id<number>(42);
   :                     ^
 6 | 
   `----

  x tokens-plugin(tokens): Numeric ("42")
   ,-[files/ts_angle_relex.ts:5:22]
 4 | // Successful type argument parsing with `<` and `>`
 5 | const b = id<number>(42);
   :                      ^^
 6 | 
   `----

  x tokens-plugin(tokens): Punctuator (")")
   ,-[files/ts_angle_relex.ts:5:24]
 4 | // Successful type argument parsing with `<` and `>`
 5 | const b = id<number>(42);
   :                        ^
 6 | 
   `----

  x tokens-plugin(tokens): Punctuator (";")
   ,-[files/ts_angle_relex.ts:5:25]
 4 | // Successful type argument parsing with `<` and `>`
 5 | const b = id<number>(42);
   :                         ^
 6 | 
   `----

  x tokens-plugin(tokens): Line (" `>` after type args is disambiguated: speculatively tried as end of type args, fails,")
   ,-[files/ts_angle_relex.ts:7:1]
 6 | 
 7 | // `>` after type args is disambiguated: speculatively tried as end of type args, fails,
   : ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
 8 | // rewinds to binary expression `n < (1 >> (0))`
   `----

  x tokens-plugin(tokens): Line (" rewinds to binary expression `n < (1 >> (0))`")
   ,-[files/ts_angle_relex.ts:8:1]
 7 | // `>` after type args is disambiguated: speculatively tried as end of type args, fails,
 8 | // rewinds to binary expression `n < (1 >> (0))`
   : ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
 9 | const c = n<1>>(0);
   `----

  x tokens-plugin(tokens): Keyword ("const")
   ,-[files/ts_angle_relex.ts:9:1]
 8 | // rewinds to binary expression `n < (1 >> (0))`
 9 | const c = n<1>>(0);
   : ^^^^^
   `----

  x tokens-plugin(tokens): Identifier ("c")
   ,-[files/ts_angle_relex.ts:9:7]
 8 | // rewinds to binary expression `n < (1 >> (0))`
 9 | const c = n<1>>(0);
   :       ^
   `----

  x tokens-plugin(tokens): Punctuator ("=")
   ,-[files/ts_angle_relex.ts:9:9]
 8 | // rewinds to binary expression `n < (1 >> (0))`
 9 | const c = n<1>>(0);
   :         ^
   `----

  x tokens-plugin(tokens): Identifier ("n")
   ,-[files/ts_angle_relex.ts:9:11]
 8 | // rewinds to binary expression `n < (1 >> (0))`
 9 | const c = n<1>>(0);
   :           ^
   `----

  x tokens-plugin(tokens): Punctuator ("<")
   ,-[files/ts_angle_relex.ts:9:12]
 8 | // rewinds to binary expression `n < (1 >> (0))`
 9 | const c = n<1>>(0);
   :            ^
   `----

  x tokens-plugin(tokens): Numeric ("1")
   ,-[files/ts_angle_relex.ts:9:13]
 8 | // rewinds to binary expression `n < (1 >> (0))`
 9 | const c = n<1>>(0);
   :             ^
   `----

  x tokens-plugin(tokens): Punctuator (">>")
   ,-[files/ts_angle_relex.ts:9:14]
 8 | // rewinds to binary expression `n < (1 >> (0))`
 9 | const c = n<1>>(0);
   :              ^^
   `----

  x tokens-plugin(tokens): Punctuator ("(")
   ,-[files/ts_angle_relex.ts:9:16]
 8 | // rewinds to binary expression `n < (1 >> (0))`
 9 | const c = n<1>>(0);
   :                ^
   `----

  x tokens-plugin(tokens): Numeric ("0")
   ,-[files/ts_angle_relex.ts:9:17]
 8 | // rewinds to binary expression `n < (1 >> (0))`
 9 | const c = n<1>>(0);
   :                 ^
   `----

  x tokens-plugin(tokens): Punctuator (")")
   ,-[files/ts_angle_relex.ts:9:18]
 8 | // rewinds to binary expression `n < (1 >> (0))`
 9 | const c = n<1>>(0);
   :                  ^
   `----

  x tokens-plugin(tokens): Punctuator (";")
   ,-[files/ts_angle_relex.ts:9:19]
 8 | // rewinds to binary expression `n < (1 >> (0))`
 9 | const c = n<1>>(0);
   :                   ^
   `----

  x tokens-plugin(tokens): Identifier ("a")
   ,-[files/unicode.js:1:1]
 1 | a;
   : ^
 2 | // 😀🤪😆😎🤮
   `----

  x tokens-plugin(tokens): Tokens and comments:
  | Identifier        loc= 1:0 - 1:1    range= 0-1     "a"
  | Punctuator        loc= 1:1 - 1:2    range= 1-2     ";"
  | Line              loc= 2:0 - 2:13   range= 3-16    " 😀🤪😆😎🤮"
  | Identifier        loc= 3:0 - 3:1    range= 17-18   "b"
  | Punctuator        loc= 3:1 - 3:2    range= 18-19   ";"
   ,-[files/unicode.js:1:1]
 1 | ,-> a;
 2 | |   // 😀🤪😆😎🤮
 3 | `-> b;
   `----

  x tokens-plugin(tokens): Tokens:
  | Identifier        loc= 1:0 - 1:1    range= 0-1     "a"
  | Punctuator        loc= 1:1 - 1:2    range= 1-2     ";"
  | Identifier        loc= 3:0 - 3:1    range= 17-18   "b"
  | Punctuator        loc= 3:1 - 3:2    range= 18-19   ";"
   ,-[files/unicode.js:1:1]
 1 | ,-> a;
 2 | |   // 😀🤪😆😎🤮
 3 | `-> b;
   `----

  x tokens-plugin(tokens): Punctuator (";")
   ,-[files/unicode.js:1:2]
 1 | a;
   :  ^
 2 | // 😀🤪😆😎🤮
   `----

  x tokens-plugin(tokens): Line (" 😀🤪😆😎🤮")
   ,-[files/unicode.js:2:1]
 1 | a;
 2 | // 😀🤪😆😎🤮
   : ^^^^^^^^^^^^^
 3 | b;
   `----

  x tokens-plugin(tokens): Identifier ("b")
   ,-[files/unicode.js:3:1]
 2 | // 😀🤪😆😎🤮
 3 | b;
   : ^
   `----

  x tokens-plugin(tokens): Punctuator (";")
   ,-[files/unicode.js:3:2]
 2 | // 😀🤪😆😎🤮
 3 | b;
   :  ^
   `----

Found 0 warnings and 157 errors.
Finished in Xms on 7 files with 1 rules using X threads.
```

# stderr
```
```
