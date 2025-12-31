# Exit code
1

# stdout
```
  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtokens-plugin(tokens): Tokens:
  [38;2;225;80;80;1m│[0m Keyword           loc= 3:0 - 3:3    range= 20-23   "let"
  [38;2;225;80;80;1m│[0m Identifier        loc= 3:4 - 3:5    range= 24-25   "x"
  [38;2;225;80;80;1m│[0m Punctuator        loc= 3:6 - 3:7    range= 26-27   "="
  [38;2;225;80;80;1m│[0m Numeric           loc= 3:29 - 3:30  range= 49-50   "1"
  [38;2;225;80;80;1m│[0m Punctuator        loc= 3:30 - 3:31  range= 50-51   ";"
  [38;2;225;80;80;1m│[0m Keyword           loc= 6:0 - 6:3    range= 72-75   "let"
  [38;2;225;80;80;1m│[0m Identifier        loc= 6:4 - 6:5    range= 76-77   "y"
  [38;2;225;80;80;1m│[0m Punctuator        loc= 6:6 - 6:7    range= 78-79   "="
  [38;2;225;80;80;1m│[0m Numeric           loc= 6:8 - 6:9    range= 80-81   "2"
  [38;2;225;80;80;1m│[0m Punctuator        loc= 6:9 - 6:10   range= 81-82   ";"[0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:1:1]
 [2m1[0m │ [38;2;246;87;248m╭[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m // Leading comment
 [2m2[0m │ [38;2;246;87;248m│[0m   
 [2m3[0m │ [38;2;246;87;248m│[0m   let x = /* inline comment */ 1;
 [2m4[0m │ [38;2;246;87;248m│[0m   
 [2m5[0m │ [38;2;246;87;248m│[0m   // Another comment
 [2m6[0m │ [38;2;246;87;248m│[0m   let y = 2;
 [2m7[0m │ [38;2;246;87;248m│[0m   
 [2m8[0m │ [38;2;246;87;248m╰[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m // Trailing comment
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtokens-plugin(tokens): Tokens and comments:
  [38;2;225;80;80;1m│[0m Line              loc= 1:0 - 1:18   range= 0-18    " Leading comment"
  [38;2;225;80;80;1m│[0m Keyword           loc= 3:0 - 3:3    range= 20-23   "let"
  [38;2;225;80;80;1m│[0m Identifier        loc= 3:4 - 3:5    range= 24-25   "x"
  [38;2;225;80;80;1m│[0m Punctuator        loc= 3:6 - 3:7    range= 26-27   "="
  [38;2;225;80;80;1m│[0m Block             loc= 3:8 - 3:28   range= 28-48   " inline comment "
  [38;2;225;80;80;1m│[0m Numeric           loc= 3:29 - 3:30  range= 49-50   "1"
  [38;2;225;80;80;1m│[0m Punctuator        loc= 3:30 - 3:31  range= 50-51   ";"
  [38;2;225;80;80;1m│[0m Line              loc= 5:0 - 5:18   range= 53-71   " Another comment"
  [38;2;225;80;80;1m│[0m Keyword           loc= 6:0 - 6:3    range= 72-75   "let"
  [38;2;225;80;80;1m│[0m Identifier        loc= 6:4 - 6:5    range= 76-77   "y"
  [38;2;225;80;80;1m│[0m Punctuator        loc= 6:6 - 6:7    range= 78-79   "="
  [38;2;225;80;80;1m│[0m Numeric           loc= 6:8 - 6:9    range= 80-81   "2"
  [38;2;225;80;80;1m│[0m Punctuator        loc= 6:9 - 6:10   range= 81-82   ";"
  [38;2;225;80;80;1m│[0m Line              loc= 8:0 - 8:19   range= 84-103  " Trailing comment"[0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:1:1]
 [2m1[0m │ [38;2;246;87;248m╭[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m // Leading comment
 [2m2[0m │ [38;2;246;87;248m│[0m   
 [2m3[0m │ [38;2;246;87;248m│[0m   let x = /* inline comment */ 1;
 [2m4[0m │ [38;2;246;87;248m│[0m   
 [2m5[0m │ [38;2;246;87;248m│[0m   // Another comment
 [2m6[0m │ [38;2;246;87;248m│[0m   let y = 2;
 [2m7[0m │ [38;2;246;87;248m│[0m   
 [2m8[0m │ [38;2;246;87;248m╰[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m // Trailing comment
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtokens-plugin(tokens): Line (" Leading comment")[0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:1:1]
 [2m1[0m │ // Leading comment
   · [38;2;246;87;248m──────────────────[0m
 [2m2[0m │ 
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtokens-plugin(tokens): Keyword ("let")[0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:3:1]
 [2m2[0m │ 
 [2m3[0m │ let x = /* inline comment */ 1;
   · [38;2;246;87;248m───[0m
 [2m4[0m │ 
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtokens-plugin(tokens): Identifier ("x")[0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:3:5]
 [2m2[0m │ 
 [2m3[0m │ let x = /* inline comment */ 1;
   · [38;2;246;87;248m    ─[0m
 [2m4[0m │ 
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtokens-plugin(tokens): Punctuator ("=")[0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:3:7]
 [2m2[0m │ 
 [2m3[0m │ let x = /* inline comment */ 1;
   · [38;2;246;87;248m      ─[0m
 [2m4[0m │ 
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtokens-plugin(tokens): Block (" inline comment ")[0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:3:9]
 [2m2[0m │ 
 [2m3[0m │ let x = /* inline comment */ 1;
   · [38;2;246;87;248m        ────────────────────[0m
 [2m4[0m │ 
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtokens-plugin(tokens): Numeric ("1")[0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:3:30]
 [2m2[0m │ 
 [2m3[0m │ let x = /* inline comment */ 1;
   · [38;2;246;87;248m                             ─[0m
 [2m4[0m │ 
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtokens-plugin(tokens): Punctuator (";")[0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:3:31]
 [2m2[0m │ 
 [2m3[0m │ let x = /* inline comment */ 1;
   · [38;2;246;87;248m                              ─[0m
 [2m4[0m │ 
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtokens-plugin(tokens): Line (" Another comment")[0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:5:1]
 [2m4[0m │ 
 [2m5[0m │ // Another comment
   · [38;2;246;87;248m──────────────────[0m
 [2m6[0m │ let y = 2;
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtokens-plugin(tokens): Keyword ("let")[0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:6:1]
 [2m5[0m │ // Another comment
 [2m6[0m │ let y = 2;
   · [38;2;246;87;248m───[0m
 [2m7[0m │ 
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtokens-plugin(tokens): Identifier ("y")[0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:6:5]
 [2m5[0m │ // Another comment
 [2m6[0m │ let y = 2;
   · [38;2;246;87;248m    ─[0m
 [2m7[0m │ 
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtokens-plugin(tokens): Punctuator ("=")[0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:6:7]
 [2m5[0m │ // Another comment
 [2m6[0m │ let y = 2;
   · [38;2;246;87;248m      ─[0m
 [2m7[0m │ 
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtokens-plugin(tokens): Numeric ("2")[0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:6:9]
 [2m5[0m │ // Another comment
 [2m6[0m │ let y = 2;
   · [38;2;246;87;248m        ─[0m
 [2m7[0m │ 
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtokens-plugin(tokens): Punctuator (";")[0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:6:10]
 [2m5[0m │ // Another comment
 [2m6[0m │ let y = 2;
   · [38;2;246;87;248m         ─[0m
 [2m7[0m │ 
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtokens-plugin(tokens): Line (" Trailing comment")[0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:8:1]
 [2m7[0m │ 
 [2m8[0m │ // Trailing comment
   · [38;2;246;87;248m───────────────────[0m
   ╰────

Found 0 warnings and 16 errors.
Finished in Xms on 1 file using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
