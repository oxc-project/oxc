# Exit code
1

# stdout
```
  x tokens-plugin(tokens): Tokens:
  | Keyword           loc=3:0-3:3     range=20-23 "let"
  | Identifier        loc=3:4-3:5     range=24-25 "x"
  | Punctuator        loc=3:6-3:7     range=26-27 "="
  | Numeric           loc=3:29-3:30   range=49-50 "1"
  | Punctuator        loc=3:30-3:31   range=50-51 ";"
  | Keyword           loc=6:0-6:3     range=72-75 "let"
  | Identifier        loc=6:4-6:5     range=76-77 "y"
  | Punctuator        loc=6:6-6:7     range=78-79 "="
  | Numeric           loc=6:8-6:9     range=80-81 "2"
  | Punctuator        loc=6:9-6:10    range=81-82 ";"
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
  | Line              loc=1:0-1:18    range=0-18 " Leading comment"
  | Keyword           loc=3:0-3:3     range=20-23 "let"
  | Identifier        loc=3:4-3:5     range=24-25 "x"
  | Punctuator        loc=3:6-3:7     range=26-27 "="
  | Block             loc=3:8-3:28    range=28-48 " inline comment "
  | Numeric           loc=3:29-3:30   range=49-50 "1"
  | Punctuator        loc=3:30-3:31   range=50-51 ";"
  | Line              loc=5:0-5:18    range=53-71 " Another comment"
  | Keyword           loc=6:0-6:3     range=72-75 "let"
  | Identifier        loc=6:4-6:5     range=76-77 "y"
  | Punctuator        loc=6:6-6:7     range=78-79 "="
  | Numeric           loc=6:8-6:9     range=80-81 "2"
  | Punctuator        loc=6:9-6:10    range=81-82 ";"
  | Line              loc=8:0-8:19    range=84-103 " Trailing comment"
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

Found 0 warnings and 2 errors.
Finished in Xms on 1 file using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
