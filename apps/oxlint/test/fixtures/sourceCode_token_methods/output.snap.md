# Exit code
1

# stdout
```
  x token-plugin(token): Tokens for VariableDeclaration, including comments:
  | Keyword           loc=1:6-1:9     range=6-9 "var"
  | Identifier        loc=1:10-1:16   range=10-16 "answer"
  | Block             loc=1:17-1:22   range=17-22 "B"
  | Punctuator        loc=1:23-1:24   range=23-24 "="
  | Block             loc=1:25-1:30   range=25-30 "C"
  | Identifier        loc=1:31-1:32   range=31-32 "a"
  | Block             loc=1:33-1:38   range=33-38 "D"
  | Punctuator        loc=1:39-1:40   range=39-40 "*"
  | Identifier        loc=1:41-1:42   range=41-42 "b"
  | Punctuator        loc=1:42-1:43   range=42-43 ";"
   ,-[files/eslint_test_case.js:1:7]
 1 | /*A*/ var answer /*B*/ = /*C*/ a /*D*/ * b; /*E*/ //F
   :       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
 2 | call();
   `----

  x token-plugin(token): Tokens for VariableDeclaration:
  | Keyword           loc=1:6-1:9     range=6-9 "var"
  | Identifier        loc=1:10-1:16   range=10-16 "answer"
  | Punctuator        loc=1:23-1:24   range=23-24 "="
  | Identifier        loc=1:31-1:32   range=31-32 "a"
  | Punctuator        loc=1:39-1:40   range=39-40 "*"
  | Identifier        loc=1:41-1:42   range=41-42 "b"
  | Punctuator        loc=1:42-1:43   range=42-43 ";"
   ,-[files/eslint_test_case.js:1:7]
 1 | /*A*/ var answer /*B*/ = /*C*/ a /*D*/ * b; /*E*/ //F
   :       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
 2 | call();
   `----

  x token-plugin(token): Tokens for ExpressionStatement:
  | Identifier        loc=2:0-2:4     range=54-58 "call"
  | Punctuator        loc=2:4-2:5     range=58-59 "("
  | Punctuator        loc=2:5-2:6     range=59-60 ")"
  | Punctuator        loc=2:6-2:7     range=60-61 ";"
   ,-[files/eslint_test_case.js:2:1]
 1 | /*A*/ var answer /*B*/ = /*C*/ a /*D*/ * b; /*E*/ //F
 2 | call();
   : ^^^^^^^
 3 | /*Z*/
   `----

  x token-plugin(token): Tokens for ExpressionStatement:
  | Boolean           loc=5:0-5:4     range=67-71 "true"
  | Punctuator        loc=5:4-5:5     range=71-72 ";"
   ,-[files/index.tsx:5:1]
 4 | // BooleanToken
 5 | true;
   : ^^^^^
 6 | false;
   `----

  x token-plugin(token): Tokens for ExpressionStatement:
  | Boolean           loc=6:0-6:5     range=73-78 "false"
  | Punctuator        loc=6:5-6:6     range=78-79 ";"
   ,-[files/index.tsx:6:1]
 5 | true;
 6 | false;
   : ^^^^^^
 7 | 
   `----

  x token-plugin(token): Tokens for ExpressionStatement:
  | Null              loc=9:0-9:4     range=94-98 "null"
  | Punctuator        loc=9:4-9:5     range=98-99 ";"
    ,-[files/index.tsx:9:1]
  8 | // NullToken
  9 | null;
    : ^^^^^
 10 | let undefined!: void;
    `----

  x token-plugin(token): Tokens for VariableDeclaration:
  | Keyword           loc=10:0-10:3   range=100-103 "let"
  | Identifier        loc=10:4-10:13  range=104-113 "undefined"
  | Punctuator        loc=10:13-10:14 range=113-114 "!"
  | Punctuator        loc=10:14-10:15 range=114-115 ":"
  | Keyword           loc=10:16-10:20 range=116-120 "void"
  | Punctuator        loc=10:20-10:21 range=120-121 ";"
    ,-[files/index.tsx:10:1]
  9 | null;
 10 | let undefined!: void;
    : ^^^^^^^^^^^^^^^^^^^^^
 11 | 
    `----

  x token-plugin(token): Tokens for ExpressionStatement:
  | Numeric           loc=13:0-13:3   range=139-142 "123"
  | Punctuator        loc=13:3-13:4   range=142-143 ";"
    ,-[files/index.tsx:13:1]
 12 | // NumericToken
 13 | 123;
    : ^^^^
 14 | 3.14;
    `----

  x token-plugin(token): Tokens for ExpressionStatement:
  | Numeric           loc=14:0-14:4   range=144-148 "3.14"
  | Punctuator        loc=14:4-14:5   range=148-149 ";"
    ,-[files/index.tsx:14:1]
 13 | 123;
 14 | 3.14;
    : ^^^^^
 15 | 
    `----

  x token-plugin(token): Tokens for ExpressionStatement:
  | Punctuator        loc=17:0-17:1   range=166-167 "("
  | String            loc=17:1-17:9   range=167-175 ""string""
  | Punctuator        loc=17:9-17:10  range=175-176 ")"
  | Punctuator        loc=17:10-17:11 range=176-177 ";"
    ,-[files/index.tsx:17:1]
 16 | // StringToken
 17 | ("string");
    : ^^^^^^^^^^^
 18 | 
    `----

  x token-plugin(token): Tokens for ExpressionStatement:
  | Identifier        loc=20:0-20:6   range=196-202 "tagged"
  | Template          loc=20:6-20:18  range=202-214 "`template ${"
  | String            loc=20:18-20:27 range=214-223 ""literal""
  | Template          loc=20:27-20:29 range=223-225 "}`"
  | Punctuator        loc=20:29-20:30 range=225-226 ";"
    ,-[files/index.tsx:20:1]
 19 | // TemplateToken
 20 | tagged`template ${"literal"}`;
    : ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
 21 | 
    `----

  x token-plugin(token): Tokens for ExpressionStatement:
  | RegularExpression loc=23:0-23:10  range=254-264 "/pattern/g"
  | Punctuator        loc=23:10-23:11 range=264-265 ";"
    ,-[files/index.tsx:23:1]
 22 | // RegularExpressionToken
 23 | /pattern/g;
    : ^^^^^^^^^^^
 24 | // prettier-ignore
    `----

  x token-plugin(token): Tokens for ExpressionStatement:
  | Numeric           loc=26:0-26:1   range=317-318 "1"
  | Punctuator        loc=26:2-26:3   range=319-320 "/"
  | Identifier        loc=26:3-26:14  range=320-331 "not_a_regex"
  | Punctuator        loc=26:14-26:15 range=331-332 "/"
  | Identifier        loc=26:15-26:17 range=332-334 "gu"
  | Punctuator        loc=26:17-26:18 range=334-335 ";"
    ,-[files/index.tsx:26:1]
 25 | // Not a RegularExpressionToken
 26 | 1 /not_a_regex/gu;
    : ^^^^^^^^^^^^^^^^^^
 27 | 
    `----

  x token-plugin(token): Tokens for VariableDeclaration:
  | Keyword           loc=29:0-29:3   range=356-359 "let"
  | Identifier        loc=29:4-29:14  range=360-370 "identifier"
  | Punctuator        loc=29:15-29:16 range=371-372 "="
  | String            loc=29:17-29:24 range=373-380 ""value""
  | Punctuator        loc=29:24-29:25 range=380-381 ";"
    ,-[files/index.tsx:29:1]
 28 | // IdentifierToken
 29 | let identifier = "value";
    : ^^^^^^^^^^^^^^^^^^^^^^^^^
 30 | 
    `----

  x token-plugin(token): Tokens for IfStatement:
  | Keyword           loc=31:0-31:2   range=383-385 "if"
  | Punctuator        loc=31:3-31:4   range=386-387 "("
  | Identifier        loc=31:4-31:14  range=387-397 "identifier"
  | Punctuator        loc=31:14-31:15 range=397-398 ")"
  | Punctuator        loc=31:16-31:17 range=399-400 "{"
  | Keyword           loc=32:2-32:5   range=403-406 "for"
  | Punctuator        loc=32:6-32:7   range=407-408 "("
  | Keyword           loc=32:7-32:10  range=408-411 "let"
  | Identifier        loc=32:11-32:12 range=412-413 "i"
  | Punctuator        loc=32:13-32:14 range=414-415 "="
  | Numeric           loc=32:15-32:16 range=416-417 "0"
  | Punctuator        loc=32:16-32:17 range=417-418 ";"
  | Identifier        loc=32:18-32:19 range=419-420 "i"
  | Punctuator        loc=32:20-32:21 range=421-422 "<"
  | Numeric           loc=32:22-32:24 range=423-425 "10"
  | Punctuator        loc=32:24-32:25 range=425-426 ";"
  | Identifier        loc=32:26-32:27 range=427-428 "i"
  | Punctuator        loc=32:27-32:29 range=428-430 "++"
  | Punctuator        loc=32:29-32:30 range=430-431 ")"
  | Punctuator        loc=32:31-32:32 range=432-433 "{"
  | Punctuator        loc=33:4-33:5   range=438-439 "("
  | Punctuator        loc=33:5-33:6   range=439-440 "("
  | Identifier        loc=33:6-33:9   range=440-443 "NaN"
  | Punctuator        loc=33:10-33:11 range=444-445 "+"
  | String            loc=33:12-33:14 range=446-448 """"
  | Punctuator        loc=33:14-33:15 range=448-449 ")"
  | Identifier        loc=33:16-33:18 range=450-452 "as"
  | Identifier        loc=33:19-33:22 range=453-456 "any"
  | Punctuator        loc=33:22-33:23 range=456-457 ")"
  | Punctuator        loc=33:24-33:26 range=458-460 "**"
  | Numeric           loc=33:27-33:28 range=461-462 "5"
  | Punctuator        loc=33:28-33:29 range=462-463 ";"
  | Punctuator        loc=34:2-34:3   range=466-467 "}"
  | Punctuator        loc=35:0-35:1   range=468-469 "}"
    ,-[files/index.tsx:31:1]
 30 |     
 31 | ,-> if (identifier) {
 32 | |     for (let i = 0; i < 10; i++) {
 33 | |       ((NaN + "") as any) ** 5;
 34 | |     }
 35 | `-> }
 36 |     
    `----

  x token-plugin(token): Tokens for ClassDeclaration, including comments:
  | Keyword           loc=37:0-37:5   range=471-476 "class"
  | Identifier        loc=37:6-37:13  range=477-484 "MyClass"
  | Keyword           loc=37:14-37:21 range=485-492 "extends"
  | Identifier        loc=37:22-37:27 range=493-498 "Error"
  | Punctuator        loc=37:28-37:29 range=499-500 "{"
  | Line              loc=38:2-38:27  range=503-528 " PrivateIdentifierToken"
  | PrivateIdentifier loc=39:2-39:10  range=531-539 "private"
  | Punctuator        loc=39:11-39:12 range=540-541 "="
  | String            loc=39:13-39:20 range=542-549 ""field""
  | Punctuator        loc=39:20-39:21 range=549-550 ";"
  | Identifier        loc=40:2-40:13  range=553-564 "constructor"
  | Punctuator        loc=40:13-40:14 range=564-565 "("
  | Punctuator        loc=40:14-40:15 range=565-566 ")"
  | Punctuator        loc=40:16-40:17 range=567-568 "{"
  | Keyword           loc=41:4-41:9   range=573-578 "super"
  | Punctuator        loc=41:9-41:10  range=578-579 "("
  | Punctuator        loc=41:10-41:11 range=579-580 ")"
  | Punctuator        loc=41:11-41:12 range=580-581 ";"
  | Keyword           loc=42:4-42:8   range=586-590 "this"
  | Punctuator        loc=42:8-42:9   range=590-591 "."
  | PrivateIdentifier loc=42:9-42:17  range=591-599 "private"
  | Punctuator        loc=42:17-42:18 range=599-600 ";"
  | PrivateIdentifier loc=43:4-43:12  range=605-613 "private"
  | Keyword           loc=43:13-43:15 range=614-616 "in"
  | Keyword           loc=43:16-43:20 range=617-621 "this"
  | Punctuator        loc=43:20-43:21 range=621-622 ";"
  | Punctuator        loc=44:2-44:3   range=625-626 "}"
  | Punctuator        loc=45:0-45:1   range=627-628 "}"
    ,-[files/index.tsx:37:1]
 36 |     
 37 | ,-> class MyClass extends Error {
 38 | |     // PrivateIdentifierToken
 39 | |     #private = "field";
 40 | |     constructor() {
 41 | |       super();
 42 | |       this.#private;
 43 | |       #private in this;
 44 | |     }
 45 | `-> }
 46 |     
    `----

  x token-plugin(token): Tokens for ClassDeclaration:
  | Keyword           loc=37:0-37:5   range=471-476 "class"
  | Identifier        loc=37:6-37:13  range=477-484 "MyClass"
  | Keyword           loc=37:14-37:21 range=485-492 "extends"
  | Identifier        loc=37:22-37:27 range=493-498 "Error"
  | Punctuator        loc=37:28-37:29 range=499-500 "{"
  | PrivateIdentifier loc=39:2-39:10  range=531-539 "private"
  | Punctuator        loc=39:11-39:12 range=540-541 "="
  | String            loc=39:13-39:20 range=542-549 ""field""
  | Punctuator        loc=39:20-39:21 range=549-550 ";"
  | Identifier        loc=40:2-40:13  range=553-564 "constructor"
  | Punctuator        loc=40:13-40:14 range=564-565 "("
  | Punctuator        loc=40:14-40:15 range=565-566 ")"
  | Punctuator        loc=40:16-40:17 range=567-568 "{"
  | Keyword           loc=41:4-41:9   range=573-578 "super"
  | Punctuator        loc=41:9-41:10  range=578-579 "("
  | Punctuator        loc=41:10-41:11 range=579-580 ")"
  | Punctuator        loc=41:11-41:12 range=580-581 ";"
  | Keyword           loc=42:4-42:8   range=586-590 "this"
  | Punctuator        loc=42:8-42:9   range=590-591 "."
  | PrivateIdentifier loc=42:9-42:17  range=591-599 "private"
  | Punctuator        loc=42:17-42:18 range=599-600 ";"
  | PrivateIdentifier loc=43:4-43:12  range=605-613 "private"
  | Keyword           loc=43:13-43:15 range=614-616 "in"
  | Keyword           loc=43:16-43:20 range=617-621 "this"
  | Punctuator        loc=43:20-43:21 range=621-622 ";"
  | Punctuator        loc=44:2-44:3   range=625-626 "}"
  | Punctuator        loc=45:0-45:1   range=627-628 "}"
    ,-[files/index.tsx:37:1]
 36 |     
 37 | ,-> class MyClass extends Error {
 38 | |     // PrivateIdentifierToken
 39 | |     #private = "field";
 40 | |     constructor() {
 41 | |       super();
 42 | |       this.#private;
 43 | |       #private in this;
 44 | |     }
 45 | `-> }
 46 |     
    `----

  x token-plugin(token): Tokens for ExpressionStatement:
  | Punctuator        loc=48:0-48:1   range=704-705 "("
  | Boolean           loc=48:1-48:6   range=705-710 "false"
  | Punctuator        loc=48:6-48:7   range=710-711 ","
  | Identifier        loc=48:8-48:16  range=712-720 "Infinity"
  | Punctuator        loc=48:16-48:17 range=720-721 ","
  | Identifier        loc=48:18-48:22 range=722-726 "eval"
  | Punctuator        loc=48:22-48:23 range=726-727 ")"
  | Punctuator        loc=48:23-48:25 range=727-729 "?."
  | Punctuator        loc=48:25-48:26 range=729-730 "("
  | String            loc=48:26-48:38 range=730-742 ""use strict""
  | Punctuator        loc=48:38-48:39 range=742-743 ")"
  | Identifier        loc=48:40-48:49 range=744-753 "satisfies"
  | Identifier        loc=48:50-48:57 range=754-761 "MyClass"
  | Punctuator        loc=48:57-48:58 range=761-762 ";"
    ,-[files/index.tsx:48:1]
 47 | // PunctuatorToken (operators and punctuation: =, +, -, (), {}, [], etc.)
 48 | (false, Infinity, eval)?.("use strict") satisfies MyClass;
    : ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
 49 | [1, 2, 3];
    `----

  x token-plugin(token): Tokens for ExpressionStatement:
  | Punctuator        loc=49:0-49:1   range=763-764 "["
  | Numeric           loc=49:1-49:2   range=764-765 "1"
  | Punctuator        loc=49:2-49:3   range=765-766 ","
  | Numeric           loc=49:4-49:5   range=767-768 "2"
  | Punctuator        loc=49:5-49:6   range=768-769 ","
  | Numeric           loc=49:7-49:8   range=770-771 "3"
  | Punctuator        loc=49:8-49:9   range=771-772 "]"
  | Punctuator        loc=49:9-49:10  range=772-773 ";"
    ,-[files/index.tsx:49:1]
 48 | (false, Infinity, eval)?.("use strict") satisfies MyClass;
 49 | [1, 2, 3];
    : ^^^^^^^^^^
 50 | {
    `----

  x token-plugin(token): Tokens for BlockStatement:
  | Punctuator        loc=50:0-50:1   range=774-775 "{"
  | Identifier        loc=51:2-51:5   range=778-781 "key"
  | Punctuator        loc=51:5-51:6   range=781-782 ":"
  | Punctuator        loc=51:7-51:8   range=783-784 "("
  | String            loc=51:8-51:15  range=784-791 ""value""
  | Punctuator        loc=51:15-51:16 range=791-792 ")"
  | Punctuator        loc=51:16-51:17 range=792-793 ";"
  | Punctuator        loc=52:0-52:1   range=794-795 "}"
    ,-[files/index.tsx:50:1]
 49 |     [1, 2, 3];
 50 | ,-> {
 51 | |     key: ("value");
 52 | `-> }
 53 |     
    `----

  x token-plugin(token): Tokens for TSTypeAliasDeclaration:
  | Identifier        loc=54:0-54:4   range=797-801 "type"
  | Identifier        loc=54:5-54:6   range=802-803 "T"
  | Punctuator        loc=54:7-54:8   range=804-805 "="
  | Punctuator        loc=54:9-54:10  range=806-807 "{"
  | Punctuator        loc=54:11-54:12 range=808-809 "["
  | Identifier        loc=54:12-54:15 range=809-812 "key"
  | Punctuator        loc=54:15-54:16 range=812-813 ":"
  | Identifier        loc=54:17-54:23 range=814-820 "string"
  | Punctuator        loc=54:23-54:24 range=820-821 "]"
  | Punctuator        loc=54:24-54:25 range=821-822 ":"
  | Identifier        loc=54:26-54:32 range=823-829 "number"
  | Punctuator        loc=54:33-54:34 range=830-831 "}"
  | Punctuator        loc=54:34-54:35 range=831-832 ";"
    ,-[files/index.tsx:54:1]
 53 | 
 54 | type T = { [key: string]: number };
    : ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
 55 | interface I extends T {
    `----

  x token-plugin(token): Tokens for TSInterfaceDeclaration:
  | Keyword           loc=55:0-55:9   range=833-842 "interface"
  | Identifier        loc=55:10-55:11 range=843-844 "I"
  | Keyword           loc=55:12-55:19 range=845-852 "extends"
  | Identifier        loc=55:20-55:21 range=853-854 "T"
  | Punctuator        loc=55:22-55:23 range=855-856 "{"
  | Identifier        loc=56:2-56:3   range=859-860 "x"
  | Punctuator        loc=56:3-56:4   range=860-861 ":"
  | Identifier        loc=56:5-56:6   range=862-863 "T"
  | Punctuator        loc=56:6-56:7   range=863-864 ";"
  | Punctuator        loc=57:0-57:1   range=865-866 "}"
    ,-[files/index.tsx:55:1]
 54 |     type T = { [key: string]: number };
 55 | ,-> interface I extends T {
 56 | |     x: T;
 57 | `-> }
 58 |     
    `----

  x token-plugin(token): Tokens for FunctionDeclaration:
  | Keyword           loc=59:0-59:8   range=868-876 "function"
  | Identifier        loc=59:9-59:12  range=877-880 "JSX"
  | Punctuator        loc=59:12-59:13 range=880-881 "("
  | Punctuator        loc=59:13-59:14 range=881-882 ")"
  | Punctuator        loc=59:15-59:16 range=883-884 "{"
  | Keyword           loc=60:2-60:8   range=887-893 "return"
  | Punctuator        loc=60:9-60:10  range=894-895 "<"
  | JSXIdentifier     loc=60:10-60:13 range=895-898 "div"
  | Punctuator        loc=60:13-60:14 range=898-899 ">"
  | JSXText           loc=60:14-60:25 range=899-910 "Hello World"
  | Punctuator        loc=60:25-60:26 range=910-911 "<"
  | Punctuator        loc=60:26-60:27 range=911-912 "/"
  | JSXIdentifier     loc=60:27-60:30 range=912-915 "div"
  | Punctuator        loc=60:30-60:31 range=915-916 ">"
  | Punctuator        loc=60:31-60:32 range=916-917 ";"
  | Punctuator        loc=61:0-61:1   range=918-919 "}"
    ,-[files/index.tsx:59:1]
 58 |     
 59 | ,-> function JSX() {
 60 | |     return <div>Hello World</div>;
 61 | `-> }
    `----

Found 0 warnings and 23 errors.
Finished in Xms on 2 files using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
