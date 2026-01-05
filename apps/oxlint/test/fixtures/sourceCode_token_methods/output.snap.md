# Exit code
1

# stdout
```
  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtoken-plugin(token): Tokens for VariableDeclaration:
  [38;2;225;80;80;1m│[0m Keyword           loc=1:6-1:9     range=6-9 "var"
  [38;2;225;80;80;1m│[0m Identifier        loc=1:10-1:16   range=10-16 "answer"
  [38;2;225;80;80;1m│[0m Punctuator        loc=1:23-1:24   range=23-24 "="
  [38;2;225;80;80;1m│[0m Identifier        loc=1:31-1:32   range=31-32 "a"
  [38;2;225;80;80;1m│[0m Punctuator        loc=1:39-1:40   range=39-40 "*"
  [38;2;225;80;80;1m│[0m Identifier        loc=1:41-1:42   range=41-42 "b"
  [38;2;225;80;80;1m│[0m Punctuator        loc=1:42-1:43   range=42-43 ";"[0m
   ╭─[[38;2;92;157;255;1mfiles/eslint_test_case.js[0m:1:7]
 [2m1[0m │ /*A*/ var answer /*B*/ = /*C*/ a /*D*/ * b; /*E*/ //F
   · [38;2;246;87;248m      ─────────────────────────────────────[0m
 [2m2[0m │ call();
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtoken-plugin(token): Tokens for VariableDeclaration, including comments:
  [38;2;225;80;80;1m│[0m Keyword           loc=1:6-1:9     range=6-9 "var"
  [38;2;225;80;80;1m│[0m Identifier        loc=1:10-1:16   range=10-16 "answer"
  [38;2;225;80;80;1m│[0m Block             loc=1:17-1:22   range=17-22 "B"
  [38;2;225;80;80;1m│[0m Punctuator        loc=1:23-1:24   range=23-24 "="
  [38;2;225;80;80;1m│[0m Block             loc=1:25-1:30   range=25-30 "C"
  [38;2;225;80;80;1m│[0m Identifier        loc=1:31-1:32   range=31-32 "a"
  [38;2;225;80;80;1m│[0m Block             loc=1:33-1:38   range=33-38 "D"
  [38;2;225;80;80;1m│[0m Punctuator        loc=1:39-1:40   range=39-40 "*"
  [38;2;225;80;80;1m│[0m Identifier        loc=1:41-1:42   range=41-42 "b"
  [38;2;225;80;80;1m│[0m Punctuator        loc=1:42-1:43   range=42-43 ";"[0m
   ╭─[[38;2;92;157;255;1mfiles/eslint_test_case.js[0m:1:7]
 [2m1[0m │ /*A*/ var answer /*B*/ = /*C*/ a /*D*/ * b; /*E*/ //F
   · [38;2;246;87;248m      ─────────────────────────────────────[0m
 [2m2[0m │ call();
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtoken-plugin(token): Tokens for ExpressionStatement:
  [38;2;225;80;80;1m│[0m Identifier        loc=2:0-2:4     range=54-58 "call"
  [38;2;225;80;80;1m│[0m Punctuator        loc=2:4-2:5     range=58-59 "("
  [38;2;225;80;80;1m│[0m Punctuator        loc=2:5-2:6     range=59-60 ")"
  [38;2;225;80;80;1m│[0m Punctuator        loc=2:6-2:7     range=60-61 ";"[0m
   ╭─[[38;2;92;157;255;1mfiles/eslint_test_case.js[0m:2:1]
 [2m1[0m │ /*A*/ var answer /*B*/ = /*C*/ a /*D*/ * b; /*E*/ //F
 [2m2[0m │ call();
   · [38;2;246;87;248m───────[0m
 [2m3[0m │ /*Z*/
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtoken-plugin(token): Tokens for ExpressionStatement:
  [38;2;225;80;80;1m│[0m Boolean           loc=5:0-5:4     range=67-71 "true"
  [38;2;225;80;80;1m│[0m Punctuator        loc=5:4-5:5     range=71-72 ";"[0m
   ╭─[[38;2;92;157;255;1mfiles/index.tsx[0m:5:1]
 [2m4[0m │ // BooleanToken
 [2m5[0m │ true;
   · [38;2;246;87;248m─────[0m
 [2m6[0m │ false;
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtoken-plugin(token): Tokens for ExpressionStatement:
  [38;2;225;80;80;1m│[0m Boolean           loc=6:0-6:5     range=73-78 "false"
  [38;2;225;80;80;1m│[0m Punctuator        loc=6:5-6:6     range=78-79 ";"[0m
   ╭─[[38;2;92;157;255;1mfiles/index.tsx[0m:6:1]
 [2m5[0m │ true;
 [2m6[0m │ false;
   · [38;2;246;87;248m──────[0m
 [2m7[0m │ 
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtoken-plugin(token): Tokens for ExpressionStatement:
  [38;2;225;80;80;1m│[0m Null              loc=9:0-9:4     range=94-98 "null"
  [38;2;225;80;80;1m│[0m Punctuator        loc=9:4-9:5     range=98-99 ";"[0m
    ╭─[[38;2;92;157;255;1mfiles/index.tsx[0m:9:1]
 [2m 8[0m │ // NullToken
 [2m 9[0m │ null;
    · [38;2;246;87;248m─────[0m
 [2m10[0m │ let undefined!: void;
    ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtoken-plugin(token): Tokens for VariableDeclaration:
  [38;2;225;80;80;1m│[0m Keyword           loc=10:0-10:3   range=100-103 "let"
  [38;2;225;80;80;1m│[0m Identifier        loc=10:4-10:13  range=104-113 "undefined"
  [38;2;225;80;80;1m│[0m Punctuator        loc=10:13-10:14 range=113-114 "!"
  [38;2;225;80;80;1m│[0m Punctuator        loc=10:14-10:15 range=114-115 ":"
  [38;2;225;80;80;1m│[0m Keyword           loc=10:16-10:20 range=116-120 "void"
  [38;2;225;80;80;1m│[0m Punctuator        loc=10:20-10:21 range=120-121 ";"[0m
    ╭─[[38;2;92;157;255;1mfiles/index.tsx[0m:10:1]
 [2m 9[0m │ null;
 [2m10[0m │ let undefined!: void;
    · [38;2;246;87;248m─────────────────────[0m
 [2m11[0m │ 
    ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtoken-plugin(token): Tokens for ExpressionStatement:
  [38;2;225;80;80;1m│[0m Numeric           loc=13:0-13:3   range=139-142 "123"
  [38;2;225;80;80;1m│[0m Punctuator        loc=13:3-13:4   range=142-143 ";"[0m
    ╭─[[38;2;92;157;255;1mfiles/index.tsx[0m:13:1]
 [2m12[0m │ // NumericToken
 [2m13[0m │ 123;
    · [38;2;246;87;248m────[0m
 [2m14[0m │ 3.14;
    ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtoken-plugin(token): Tokens for ExpressionStatement:
  [38;2;225;80;80;1m│[0m Numeric           loc=14:0-14:4   range=144-148 "3.14"
  [38;2;225;80;80;1m│[0m Punctuator        loc=14:4-14:5   range=148-149 ";"[0m
    ╭─[[38;2;92;157;255;1mfiles/index.tsx[0m:14:1]
 [2m13[0m │ 123;
 [2m14[0m │ 3.14;
    · [38;2;246;87;248m─────[0m
 [2m15[0m │ 
    ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtoken-plugin(token): Tokens for ExpressionStatement:
  [38;2;225;80;80;1m│[0m Punctuator        loc=17:0-17:1   range=166-167 "("
  [38;2;225;80;80;1m│[0m String            loc=17:1-17:9   range=167-175 ""string""
  [38;2;225;80;80;1m│[0m Punctuator        loc=17:9-17:10  range=175-176 ")"
  [38;2;225;80;80;1m│[0m Punctuator        loc=17:10-17:11 range=176-177 ";"[0m
    ╭─[[38;2;92;157;255;1mfiles/index.tsx[0m:17:1]
 [2m16[0m │ // StringToken
 [2m17[0m │ ("string");
    · [38;2;246;87;248m───────────[0m
 [2m18[0m │ 
    ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtoken-plugin(token): Tokens for ExpressionStatement:
  [38;2;225;80;80;1m│[0m Identifier        loc=20:0-20:6   range=196-202 "tagged"
  [38;2;225;80;80;1m│[0m Template          loc=20:6-20:18  range=202-214 "`template ${"
  [38;2;225;80;80;1m│[0m String            loc=20:18-20:27 range=214-223 ""literal""
  [38;2;225;80;80;1m│[0m Template          loc=20:27-20:29 range=223-225 "}`"
  [38;2;225;80;80;1m│[0m Punctuator        loc=20:29-20:30 range=225-226 ";"[0m
    ╭─[[38;2;92;157;255;1mfiles/index.tsx[0m:20:1]
 [2m19[0m │ // TemplateToken
 [2m20[0m │ tagged`template ${"literal"}`;
    · [38;2;246;87;248m──────────────────────────────[0m
 [2m21[0m │ 
    ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtoken-plugin(token): Tokens for ExpressionStatement:
  [38;2;225;80;80;1m│[0m RegularExpression loc=23:0-23:10  range=254-264 "/pattern/g"
  [38;2;225;80;80;1m│[0m Punctuator        loc=23:10-23:11 range=264-265 ";"[0m
    ╭─[[38;2;92;157;255;1mfiles/index.tsx[0m:23:1]
 [2m22[0m │ // RegularExpressionToken
 [2m23[0m │ /pattern/g;
    · [38;2;246;87;248m───────────[0m
 [2m24[0m │ // prettier-ignore
    ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtoken-plugin(token): Tokens for ExpressionStatement:
  [38;2;225;80;80;1m│[0m Numeric           loc=26:0-26:1   range=317-318 "1"
  [38;2;225;80;80;1m│[0m Punctuator        loc=26:2-26:3   range=319-320 "/"
  [38;2;225;80;80;1m│[0m Identifier        loc=26:3-26:14  range=320-331 "not_a_regex"
  [38;2;225;80;80;1m│[0m Punctuator        loc=26:14-26:15 range=331-332 "/"
  [38;2;225;80;80;1m│[0m Identifier        loc=26:15-26:17 range=332-334 "gu"
  [38;2;225;80;80;1m│[0m Punctuator        loc=26:17-26:18 range=334-335 ";"[0m
    ╭─[[38;2;92;157;255;1mfiles/index.tsx[0m:26:1]
 [2m25[0m │ // Not a RegularExpressionToken
 [2m26[0m │ 1 /not_a_regex/gu;
    · [38;2;246;87;248m──────────────────[0m
 [2m27[0m │ 
    ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtoken-plugin(token): Tokens for VariableDeclaration:
  [38;2;225;80;80;1m│[0m Keyword           loc=29:0-29:3   range=356-359 "let"
  [38;2;225;80;80;1m│[0m Identifier        loc=29:4-29:14  range=360-370 "identifier"
  [38;2;225;80;80;1m│[0m Punctuator        loc=29:15-29:16 range=371-372 "="
  [38;2;225;80;80;1m│[0m String            loc=29:17-29:24 range=373-380 ""value""
  [38;2;225;80;80;1m│[0m Punctuator        loc=29:24-29:25 range=380-381 ";"[0m
    ╭─[[38;2;92;157;255;1mfiles/index.tsx[0m:29:1]
 [2m28[0m │ // IdentifierToken
 [2m29[0m │ let identifier = "value";
    · [38;2;246;87;248m─────────────────────────[0m
 [2m30[0m │ 
    ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtoken-plugin(token): Tokens for IfStatement:
  [38;2;225;80;80;1m│[0m Keyword           loc=31:0-31:2   range=383-385 "if"
  [38;2;225;80;80;1m│[0m Punctuator        loc=31:3-31:4   range=386-387 "("
  [38;2;225;80;80;1m│[0m Identifier        loc=31:4-31:14  range=387-397 "identifier"
  [38;2;225;80;80;1m│[0m Punctuator        loc=31:14-31:15 range=397-398 ")"
  [38;2;225;80;80;1m│[0m Punctuator        loc=31:16-31:17 range=399-400 "{"
  [38;2;225;80;80;1m│[0m Keyword           loc=32:2-32:5   range=403-406 "for"
  [38;2;225;80;80;1m│[0m Punctuator        loc=32:6-32:7   range=407-408 "("
  [38;2;225;80;80;1m│[0m Keyword           loc=32:7-32:10  range=408-411 "let"
  [38;2;225;80;80;1m│[0m Identifier        loc=32:11-32:12 range=412-413 "i"
  [38;2;225;80;80;1m│[0m Punctuator        loc=32:13-32:14 range=414-415 "="
  [38;2;225;80;80;1m│[0m Numeric           loc=32:15-32:16 range=416-417 "0"
  [38;2;225;80;80;1m│[0m Punctuator        loc=32:16-32:17 range=417-418 ";"
  [38;2;225;80;80;1m│[0m Identifier        loc=32:18-32:19 range=419-420 "i"
  [38;2;225;80;80;1m│[0m Punctuator        loc=32:20-32:21 range=421-422 "<"
  [38;2;225;80;80;1m│[0m Numeric           loc=32:22-32:24 range=423-425 "10"
  [38;2;225;80;80;1m│[0m Punctuator        loc=32:24-32:25 range=425-426 ";"
  [38;2;225;80;80;1m│[0m Identifier        loc=32:26-32:27 range=427-428 "i"
  [38;2;225;80;80;1m│[0m Punctuator        loc=32:27-32:29 range=428-430 "++"
  [38;2;225;80;80;1m│[0m Punctuator        loc=32:29-32:30 range=430-431 ")"
  [38;2;225;80;80;1m│[0m Punctuator        loc=32:31-32:32 range=432-433 "{"
  [38;2;225;80;80;1m│[0m Punctuator        loc=33:4-33:5   range=438-439 "("
  [38;2;225;80;80;1m│[0m Punctuator        loc=33:5-33:6   range=439-440 "("
  [38;2;225;80;80;1m│[0m Identifier        loc=33:6-33:9   range=440-443 "NaN"
  [38;2;225;80;80;1m│[0m Punctuator        loc=33:10-33:11 range=444-445 "+"
  [38;2;225;80;80;1m│[0m String            loc=33:12-33:14 range=446-448 """"
  [38;2;225;80;80;1m│[0m Punctuator        loc=33:14-33:15 range=448-449 ")"
  [38;2;225;80;80;1m│[0m Identifier        loc=33:16-33:18 range=450-452 "as"
  [38;2;225;80;80;1m│[0m Identifier        loc=33:19-33:22 range=453-456 "any"
  [38;2;225;80;80;1m│[0m Punctuator        loc=33:22-33:23 range=456-457 ")"
  [38;2;225;80;80;1m│[0m Punctuator        loc=33:24-33:26 range=458-460 "**"
  [38;2;225;80;80;1m│[0m Numeric           loc=33:27-33:28 range=461-462 "5"
  [38;2;225;80;80;1m│[0m Punctuator        loc=33:28-33:29 range=462-463 ";"
  [38;2;225;80;80;1m│[0m Punctuator        loc=34:2-34:3   range=466-467 "}"
  [38;2;225;80;80;1m│[0m Punctuator        loc=35:0-35:1   range=468-469 "}"[0m
    ╭─[[38;2;92;157;255;1mfiles/index.tsx[0m:31:1]
 [2m30[0m │     
 [2m31[0m │ [38;2;246;87;248m╭[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m if (identifier) {
 [2m32[0m │ [38;2;246;87;248m│[0m     for (let i = 0; i < 10; i++) {
 [2m33[0m │ [38;2;246;87;248m│[0m       ((NaN + "") as any) ** 5;
 [2m34[0m │ [38;2;246;87;248m│[0m     }
 [2m35[0m │ [38;2;246;87;248m╰[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m }
 [2m36[0m │     
    ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtoken-plugin(token): Tokens for ClassDeclaration:
  [38;2;225;80;80;1m│[0m Keyword           loc=37:0-37:5   range=471-476 "class"
  [38;2;225;80;80;1m│[0m Identifier        loc=37:6-37:13  range=477-484 "MyClass"
  [38;2;225;80;80;1m│[0m Keyword           loc=37:14-37:21 range=485-492 "extends"
  [38;2;225;80;80;1m│[0m Identifier        loc=37:22-37:27 range=493-498 "Error"
  [38;2;225;80;80;1m│[0m Punctuator        loc=37:28-37:29 range=499-500 "{"
  [38;2;225;80;80;1m│[0m PrivateIdentifier loc=39:2-39:10  range=531-539 "private"
  [38;2;225;80;80;1m│[0m Punctuator        loc=39:11-39:12 range=540-541 "="
  [38;2;225;80;80;1m│[0m String            loc=39:13-39:20 range=542-549 ""field""
  [38;2;225;80;80;1m│[0m Punctuator        loc=39:20-39:21 range=549-550 ";"
  [38;2;225;80;80;1m│[0m Identifier        loc=40:2-40:13  range=553-564 "constructor"
  [38;2;225;80;80;1m│[0m Punctuator        loc=40:13-40:14 range=564-565 "("
  [38;2;225;80;80;1m│[0m Punctuator        loc=40:14-40:15 range=565-566 ")"
  [38;2;225;80;80;1m│[0m Punctuator        loc=40:16-40:17 range=567-568 "{"
  [38;2;225;80;80;1m│[0m Keyword           loc=41:4-41:9   range=573-578 "super"
  [38;2;225;80;80;1m│[0m Punctuator        loc=41:9-41:10  range=578-579 "("
  [38;2;225;80;80;1m│[0m Punctuator        loc=41:10-41:11 range=579-580 ")"
  [38;2;225;80;80;1m│[0m Punctuator        loc=41:11-41:12 range=580-581 ";"
  [38;2;225;80;80;1m│[0m Keyword           loc=42:4-42:8   range=586-590 "this"
  [38;2;225;80;80;1m│[0m Punctuator        loc=42:8-42:9   range=590-591 "."
  [38;2;225;80;80;1m│[0m PrivateIdentifier loc=42:9-42:17  range=591-599 "private"
  [38;2;225;80;80;1m│[0m Punctuator        loc=42:17-42:18 range=599-600 ";"
  [38;2;225;80;80;1m│[0m PrivateIdentifier loc=43:4-43:12  range=605-613 "private"
  [38;2;225;80;80;1m│[0m Keyword           loc=43:13-43:15 range=614-616 "in"
  [38;2;225;80;80;1m│[0m Keyword           loc=43:16-43:20 range=617-621 "this"
  [38;2;225;80;80;1m│[0m Punctuator        loc=43:20-43:21 range=621-622 ";"
  [38;2;225;80;80;1m│[0m Punctuator        loc=44:2-44:3   range=625-626 "}"
  [38;2;225;80;80;1m│[0m Punctuator        loc=45:0-45:1   range=627-628 "}"[0m
    ╭─[[38;2;92;157;255;1mfiles/index.tsx[0m:37:1]
 [2m36[0m │     
 [2m37[0m │ [38;2;246;87;248m╭[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m class MyClass extends Error {
 [2m38[0m │ [38;2;246;87;248m│[0m     // PrivateIdentifierToken
 [2m39[0m │ [38;2;246;87;248m│[0m     #private = "field";
 [2m40[0m │ [38;2;246;87;248m│[0m     constructor() {
 [2m41[0m │ [38;2;246;87;248m│[0m       super();
 [2m42[0m │ [38;2;246;87;248m│[0m       this.#private;
 [2m43[0m │ [38;2;246;87;248m│[0m       #private in this;
 [2m44[0m │ [38;2;246;87;248m│[0m     }
 [2m45[0m │ [38;2;246;87;248m╰[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m }
 [2m46[0m │     
    ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtoken-plugin(token): Tokens for ClassDeclaration, including comments:
  [38;2;225;80;80;1m│[0m Keyword           loc=37:0-37:5   range=471-476 "class"
  [38;2;225;80;80;1m│[0m Identifier        loc=37:6-37:13  range=477-484 "MyClass"
  [38;2;225;80;80;1m│[0m Keyword           loc=37:14-37:21 range=485-492 "extends"
  [38;2;225;80;80;1m│[0m Identifier        loc=37:22-37:27 range=493-498 "Error"
  [38;2;225;80;80;1m│[0m Punctuator        loc=37:28-37:29 range=499-500 "{"
  [38;2;225;80;80;1m│[0m Line              loc=38:2-38:27  range=503-528 " PrivateIdentifierToken"
  [38;2;225;80;80;1m│[0m PrivateIdentifier loc=39:2-39:10  range=531-539 "private"
  [38;2;225;80;80;1m│[0m Punctuator        loc=39:11-39:12 range=540-541 "="
  [38;2;225;80;80;1m│[0m String            loc=39:13-39:20 range=542-549 ""field""
  [38;2;225;80;80;1m│[0m Punctuator        loc=39:20-39:21 range=549-550 ";"
  [38;2;225;80;80;1m│[0m Identifier        loc=40:2-40:13  range=553-564 "constructor"
  [38;2;225;80;80;1m│[0m Punctuator        loc=40:13-40:14 range=564-565 "("
  [38;2;225;80;80;1m│[0m Punctuator        loc=40:14-40:15 range=565-566 ")"
  [38;2;225;80;80;1m│[0m Punctuator        loc=40:16-40:17 range=567-568 "{"
  [38;2;225;80;80;1m│[0m Keyword           loc=41:4-41:9   range=573-578 "super"
  [38;2;225;80;80;1m│[0m Punctuator        loc=41:9-41:10  range=578-579 "("
  [38;2;225;80;80;1m│[0m Punctuator        loc=41:10-41:11 range=579-580 ")"
  [38;2;225;80;80;1m│[0m Punctuator        loc=41:11-41:12 range=580-581 ";"
  [38;2;225;80;80;1m│[0m Keyword           loc=42:4-42:8   range=586-590 "this"
  [38;2;225;80;80;1m│[0m Punctuator        loc=42:8-42:9   range=590-591 "."
  [38;2;225;80;80;1m│[0m PrivateIdentifier loc=42:9-42:17  range=591-599 "private"
  [38;2;225;80;80;1m│[0m Punctuator        loc=42:17-42:18 range=599-600 ";"
  [38;2;225;80;80;1m│[0m PrivateIdentifier loc=43:4-43:12  range=605-613 "private"
  [38;2;225;80;80;1m│[0m Keyword           loc=43:13-43:15 range=614-616 "in"
  [38;2;225;80;80;1m│[0m Keyword           loc=43:16-43:20 range=617-621 "this"
  [38;2;225;80;80;1m│[0m Punctuator        loc=43:20-43:21 range=621-622 ";"
  [38;2;225;80;80;1m│[0m Punctuator        loc=44:2-44:3   range=625-626 "}"
  [38;2;225;80;80;1m│[0m Punctuator        loc=45:0-45:1   range=627-628 "}"[0m
    ╭─[[38;2;92;157;255;1mfiles/index.tsx[0m:37:1]
 [2m36[0m │     
 [2m37[0m │ [38;2;246;87;248m╭[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m class MyClass extends Error {
 [2m38[0m │ [38;2;246;87;248m│[0m     // PrivateIdentifierToken
 [2m39[0m │ [38;2;246;87;248m│[0m     #private = "field";
 [2m40[0m │ [38;2;246;87;248m│[0m     constructor() {
 [2m41[0m │ [38;2;246;87;248m│[0m       super();
 [2m42[0m │ [38;2;246;87;248m│[0m       this.#private;
 [2m43[0m │ [38;2;246;87;248m│[0m       #private in this;
 [2m44[0m │ [38;2;246;87;248m│[0m     }
 [2m45[0m │ [38;2;246;87;248m╰[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m }
 [2m46[0m │     
    ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtoken-plugin(token): Tokens for ExpressionStatement:
  [38;2;225;80;80;1m│[0m Punctuator        loc=48:0-48:1   range=704-705 "("
  [38;2;225;80;80;1m│[0m Boolean           loc=48:1-48:6   range=705-710 "false"
  [38;2;225;80;80;1m│[0m Punctuator        loc=48:6-48:7   range=710-711 ","
  [38;2;225;80;80;1m│[0m Identifier        loc=48:8-48:16  range=712-720 "Infinity"
  [38;2;225;80;80;1m│[0m Punctuator        loc=48:16-48:17 range=720-721 ","
  [38;2;225;80;80;1m│[0m Identifier        loc=48:18-48:22 range=722-726 "eval"
  [38;2;225;80;80;1m│[0m Punctuator        loc=48:22-48:23 range=726-727 ")"
  [38;2;225;80;80;1m│[0m Punctuator        loc=48:23-48:25 range=727-729 "?."
  [38;2;225;80;80;1m│[0m Punctuator        loc=48:25-48:26 range=729-730 "("
  [38;2;225;80;80;1m│[0m String            loc=48:26-48:38 range=730-742 ""use strict""
  [38;2;225;80;80;1m│[0m Punctuator        loc=48:38-48:39 range=742-743 ")"
  [38;2;225;80;80;1m│[0m Identifier        loc=48:40-48:49 range=744-753 "satisfies"
  [38;2;225;80;80;1m│[0m Identifier        loc=48:50-48:57 range=754-761 "MyClass"
  [38;2;225;80;80;1m│[0m Punctuator        loc=48:57-48:58 range=761-762 ";"[0m
    ╭─[[38;2;92;157;255;1mfiles/index.tsx[0m:48:1]
 [2m47[0m │ // PunctuatorToken (operators and punctuation: =, +, -, (), {}, [], etc.)
 [2m48[0m │ (false, Infinity, eval)?.("use strict") satisfies MyClass;
    · [38;2;246;87;248m──────────────────────────────────────────────────────────[0m
 [2m49[0m │ [1, 2, 3];
    ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtoken-plugin(token): Tokens for ExpressionStatement:
  [38;2;225;80;80;1m│[0m Punctuator        loc=49:0-49:1   range=763-764 "["
  [38;2;225;80;80;1m│[0m Numeric           loc=49:1-49:2   range=764-765 "1"
  [38;2;225;80;80;1m│[0m Punctuator        loc=49:2-49:3   range=765-766 ","
  [38;2;225;80;80;1m│[0m Numeric           loc=49:4-49:5   range=767-768 "2"
  [38;2;225;80;80;1m│[0m Punctuator        loc=49:5-49:6   range=768-769 ","
  [38;2;225;80;80;1m│[0m Numeric           loc=49:7-49:8   range=770-771 "3"
  [38;2;225;80;80;1m│[0m Punctuator        loc=49:8-49:9   range=771-772 "]"
  [38;2;225;80;80;1m│[0m Punctuator        loc=49:9-49:10  range=772-773 ";"[0m
    ╭─[[38;2;92;157;255;1mfiles/index.tsx[0m:49:1]
 [2m48[0m │ (false, Infinity, eval)?.("use strict") satisfies MyClass;
 [2m49[0m │ [1, 2, 3];
    · [38;2;246;87;248m──────────[0m
 [2m50[0m │ {
    ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtoken-plugin(token): Tokens for BlockStatement:
  [38;2;225;80;80;1m│[0m Punctuator        loc=50:0-50:1   range=774-775 "{"
  [38;2;225;80;80;1m│[0m Identifier        loc=51:2-51:5   range=778-781 "key"
  [38;2;225;80;80;1m│[0m Punctuator        loc=51:5-51:6   range=781-782 ":"
  [38;2;225;80;80;1m│[0m Punctuator        loc=51:7-51:8   range=783-784 "("
  [38;2;225;80;80;1m│[0m String            loc=51:8-51:15  range=784-791 ""value""
  [38;2;225;80;80;1m│[0m Punctuator        loc=51:15-51:16 range=791-792 ")"
  [38;2;225;80;80;1m│[0m Punctuator        loc=51:16-51:17 range=792-793 ";"
  [38;2;225;80;80;1m│[0m Punctuator        loc=52:0-52:1   range=794-795 "}"[0m
    ╭─[[38;2;92;157;255;1mfiles/index.tsx[0m:50:1]
 [2m49[0m │     [1, 2, 3];
 [2m50[0m │ [38;2;246;87;248m╭[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m {
 [2m51[0m │ [38;2;246;87;248m│[0m     key: ("value");
 [2m52[0m │ [38;2;246;87;248m╰[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m }
 [2m53[0m │     
    ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtoken-plugin(token): Tokens for TSTypeAliasDeclaration:
  [38;2;225;80;80;1m│[0m Identifier        loc=54:0-54:4   range=797-801 "type"
  [38;2;225;80;80;1m│[0m Identifier        loc=54:5-54:6   range=802-803 "T"
  [38;2;225;80;80;1m│[0m Punctuator        loc=54:7-54:8   range=804-805 "="
  [38;2;225;80;80;1m│[0m Punctuator        loc=54:9-54:10  range=806-807 "{"
  [38;2;225;80;80;1m│[0m Punctuator        loc=54:11-54:12 range=808-809 "["
  [38;2;225;80;80;1m│[0m Identifier        loc=54:12-54:15 range=809-812 "key"
  [38;2;225;80;80;1m│[0m Punctuator        loc=54:15-54:16 range=812-813 ":"
  [38;2;225;80;80;1m│[0m Identifier        loc=54:17-54:23 range=814-820 "string"
  [38;2;225;80;80;1m│[0m Punctuator        loc=54:23-54:24 range=820-821 "]"
  [38;2;225;80;80;1m│[0m Punctuator        loc=54:24-54:25 range=821-822 ":"
  [38;2;225;80;80;1m│[0m Identifier        loc=54:26-54:32 range=823-829 "number"
  [38;2;225;80;80;1m│[0m Punctuator        loc=54:33-54:34 range=830-831 "}"
  [38;2;225;80;80;1m│[0m Punctuator        loc=54:34-54:35 range=831-832 ";"[0m
    ╭─[[38;2;92;157;255;1mfiles/index.tsx[0m:54:1]
 [2m53[0m │ 
 [2m54[0m │ type T = { [key: string]: number };
    · [38;2;246;87;248m───────────────────────────────────[0m
 [2m55[0m │ interface I extends T {
    ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtoken-plugin(token): Tokens for TSInterfaceDeclaration:
  [38;2;225;80;80;1m│[0m Keyword           loc=55:0-55:9   range=833-842 "interface"
  [38;2;225;80;80;1m│[0m Identifier        loc=55:10-55:11 range=843-844 "I"
  [38;2;225;80;80;1m│[0m Keyword           loc=55:12-55:19 range=845-852 "extends"
  [38;2;225;80;80;1m│[0m Identifier        loc=55:20-55:21 range=853-854 "T"
  [38;2;225;80;80;1m│[0m Punctuator        loc=55:22-55:23 range=855-856 "{"
  [38;2;225;80;80;1m│[0m Identifier        loc=56:2-56:3   range=859-860 "x"
  [38;2;225;80;80;1m│[0m Punctuator        loc=56:3-56:4   range=860-861 ":"
  [38;2;225;80;80;1m│[0m Identifier        loc=56:5-56:6   range=862-863 "T"
  [38;2;225;80;80;1m│[0m Punctuator        loc=56:6-56:7   range=863-864 ";"
  [38;2;225;80;80;1m│[0m Punctuator        loc=57:0-57:1   range=865-866 "}"[0m
    ╭─[[38;2;92;157;255;1mfiles/index.tsx[0m:55:1]
 [2m54[0m │     type T = { [key: string]: number };
 [2m55[0m │ [38;2;246;87;248m╭[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m interface I extends T {
 [2m56[0m │ [38;2;246;87;248m│[0m     x: T;
 [2m57[0m │ [38;2;246;87;248m╰[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m }
 [2m58[0m │     
    ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtoken-plugin(token): Tokens for FunctionDeclaration:
  [38;2;225;80;80;1m│[0m Keyword           loc=59:0-59:8   range=868-876 "function"
  [38;2;225;80;80;1m│[0m Identifier        loc=59:9-59:12  range=877-880 "JSX"
  [38;2;225;80;80;1m│[0m Punctuator        loc=59:12-59:13 range=880-881 "("
  [38;2;225;80;80;1m│[0m Punctuator        loc=59:13-59:14 range=881-882 ")"
  [38;2;225;80;80;1m│[0m Punctuator        loc=59:15-59:16 range=883-884 "{"
  [38;2;225;80;80;1m│[0m Keyword           loc=60:2-60:8   range=887-893 "return"
  [38;2;225;80;80;1m│[0m Punctuator        loc=60:9-60:10  range=894-895 "<"
  [38;2;225;80;80;1m│[0m JSXIdentifier     loc=60:10-60:13 range=895-898 "div"
  [38;2;225;80;80;1m│[0m Punctuator        loc=60:13-60:14 range=898-899 ">"
  [38;2;225;80;80;1m│[0m JSXText           loc=60:14-60:25 range=899-910 "Hello World"
  [38;2;225;80;80;1m│[0m Punctuator        loc=60:25-60:26 range=910-911 "<"
  [38;2;225;80;80;1m│[0m Punctuator        loc=60:26-60:27 range=911-912 "/"
  [38;2;225;80;80;1m│[0m JSXIdentifier     loc=60:27-60:30 range=912-915 "div"
  [38;2;225;80;80;1m│[0m Punctuator        loc=60:30-60:31 range=915-916 ">"
  [38;2;225;80;80;1m│[0m Punctuator        loc=60:31-60:32 range=916-917 ";"
  [38;2;225;80;80;1m│[0m Punctuator        loc=61:0-61:1   range=918-919 "}"[0m
    ╭─[[38;2;92;157;255;1mfiles/index.tsx[0m:59:1]
 [2m58[0m │     
 [2m59[0m │ [38;2;246;87;248m╭[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m function JSX() {
 [2m60[0m │ [38;2;246;87;248m│[0m     return <div>Hello World</div>;
 [2m61[0m │ [38;2;246;87;248m╰[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m }
    ╰────

Found 0 warnings and 23 errors.
Finished in Xms on 2 files with 1 rules using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
