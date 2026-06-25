# Exit code
1

# stdout
```
  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 1:
  | [tokens, comments, tokensAndComments, lastToken, lastComment]
   ,-[files/001.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 2:
  | [comments, tokens, tokensAndComments, lastToken, lastComment]
   ,-[files/002.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 3:
  | [tokensAndComments, tokens, comments, lastToken, lastComment]
   ,-[files/003.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 4:
  | [lastToken, tokens, comments, tokensAndComments, lastComment]
   ,-[files/004.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 5:
  | [lastComment, tokens, comments, tokensAndComments, lastToken]
   ,-[files/005.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 6:
  | [tokens, tokensAndComments, comments, lastToken, lastComment]
   ,-[files/006.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 7:
  | [comments, tokensAndComments, tokens, lastToken, lastComment]
   ,-[files/007.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 8:
  | [tokensAndComments, comments, tokens, lastToken, lastComment]
   ,-[files/008.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 9:
  | [lastToken, comments, tokens, tokensAndComments, lastComment]
   ,-[files/009.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 10:
  | [lastComment, comments, tokens, tokensAndComments, lastToken]
   ,-[files/010.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 11:
  | [tokens, lastToken, comments, tokensAndComments, lastComment]
   ,-[files/011.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 12:
  | [comments, lastToken, tokens, tokensAndComments, lastComment]
   ,-[files/012.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 13:
  | [tokensAndComments, lastToken, tokens, comments, lastComment]
   ,-[files/013.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 14:
  | [lastToken, tokensAndComments, tokens, comments, lastComment]
   ,-[files/014.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 15:
  | [lastComment, tokensAndComments, tokens, comments, lastToken]
   ,-[files/015.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 16:
  | [tokens, lastComment, comments, tokensAndComments, lastToken]
   ,-[files/016.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 17:
  | [comments, lastComment, tokens, tokensAndComments, lastToken]
   ,-[files/017.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 18:
  | [tokensAndComments, lastComment, tokens, comments, lastToken]
   ,-[files/018.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 19:
  | [lastToken, lastComment, tokens, comments, tokensAndComments]
   ,-[files/019.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 20:
  | [lastComment, lastToken, tokens, comments, tokensAndComments]
   ,-[files/020.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 21:
  | [tokens, comments, lastToken, tokensAndComments, lastComment]
   ,-[files/021.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 22:
  | [comments, tokens, lastToken, tokensAndComments, lastComment]
   ,-[files/022.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 23:
  | [tokensAndComments, tokens, lastToken, comments, lastComment]
   ,-[files/023.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 24:
  | [lastToken, tokens, tokensAndComments, comments, lastComment]
   ,-[files/024.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 25:
  | [lastComment, tokens, tokensAndComments, comments, lastToken]
   ,-[files/025.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 26:
  | [tokens, tokensAndComments, lastToken, comments, lastComment]
   ,-[files/026.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 27:
  | [comments, tokensAndComments, lastToken, tokens, lastComment]
   ,-[files/027.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 28:
  | [tokensAndComments, comments, lastToken, tokens, lastComment]
   ,-[files/028.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 29:
  | [lastToken, comments, tokensAndComments, tokens, lastComment]
   ,-[files/029.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 30:
  | [lastComment, comments, tokensAndComments, tokens, lastToken]
   ,-[files/030.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 31:
  | [tokens, lastToken, tokensAndComments, comments, lastComment]
   ,-[files/031.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 32:
  | [comments, lastToken, tokensAndComments, tokens, lastComment]
   ,-[files/032.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 33:
  | [tokensAndComments, lastToken, comments, tokens, lastComment]
   ,-[files/033.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 34:
  | [lastToken, tokensAndComments, comments, tokens, lastComment]
   ,-[files/034.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 35:
  | [lastComment, tokensAndComments, comments, tokens, lastToken]
   ,-[files/035.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 36:
  | [tokens, lastComment, tokensAndComments, comments, lastToken]
   ,-[files/036.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 37:
  | [comments, lastComment, tokensAndComments, tokens, lastToken]
   ,-[files/037.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 38:
  | [tokensAndComments, lastComment, comments, tokens, lastToken]
   ,-[files/038.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 39:
  | [lastToken, lastComment, comments, tokens, tokensAndComments]
   ,-[files/039.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 40:
  | [lastComment, lastToken, comments, tokens, tokensAndComments]
   ,-[files/040.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 41:
  | [tokens, comments, lastComment, tokensAndComments, lastToken]
   ,-[files/041.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 42:
  | [comments, tokens, lastComment, tokensAndComments, lastToken]
   ,-[files/042.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 43:
  | [tokensAndComments, tokens, lastComment, comments, lastToken]
   ,-[files/043.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 44:
  | [lastToken, tokens, lastComment, comments, tokensAndComments]
   ,-[files/044.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 45:
  | [lastComment, tokens, lastToken, comments, tokensAndComments]
   ,-[files/045.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 46:
  | [tokens, tokensAndComments, lastComment, comments, lastToken]
   ,-[files/046.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 47:
  | [comments, tokensAndComments, lastComment, tokens, lastToken]
   ,-[files/047.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 48:
  | [tokensAndComments, comments, lastComment, tokens, lastToken]
   ,-[files/048.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 49:
  | [lastToken, comments, lastComment, tokens, tokensAndComments]
   ,-[files/049.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 50:
  | [lastComment, comments, lastToken, tokens, tokensAndComments]
   ,-[files/050.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 51:
  | [tokens, lastToken, lastComment, comments, tokensAndComments]
   ,-[files/051.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 52:
  | [comments, lastToken, lastComment, tokens, tokensAndComments]
   ,-[files/052.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 53:
  | [tokensAndComments, lastToken, lastComment, tokens, comments]
   ,-[files/053.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 54:
  | [lastToken, tokensAndComments, lastComment, tokens, comments]
   ,-[files/054.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 55:
  | [lastComment, tokensAndComments, lastToken, tokens, comments]
   ,-[files/055.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 56:
  | [tokens, lastComment, lastToken, comments, tokensAndComments]
   ,-[files/056.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 57:
  | [comments, lastComment, lastToken, tokens, tokensAndComments]
   ,-[files/057.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 58:
  | [tokensAndComments, lastComment, lastToken, tokens, comments]
   ,-[files/058.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 59:
  | [lastToken, lastComment, tokensAndComments, tokens, comments]
   ,-[files/059.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 60:
  | [lastComment, lastToken, tokensAndComments, tokens, comments]
   ,-[files/060.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 61:
  | [tokens, comments, tokensAndComments, lastComment, lastToken]
   ,-[files/061.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 62:
  | [comments, tokens, tokensAndComments, lastComment, lastToken]
   ,-[files/062.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 63:
  | [tokensAndComments, tokens, comments, lastComment, lastToken]
   ,-[files/063.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 64:
  | [lastToken, tokens, comments, lastComment, tokensAndComments]
   ,-[files/064.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 65:
  | [lastComment, tokens, comments, lastToken, tokensAndComments]
   ,-[files/065.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 66:
  | [tokens, tokensAndComments, comments, lastComment, lastToken]
   ,-[files/066.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 67:
  | [comments, tokensAndComments, tokens, lastComment, lastToken]
   ,-[files/067.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 68:
  | [tokensAndComments, comments, tokens, lastComment, lastToken]
   ,-[files/068.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 69:
  | [lastToken, comments, tokens, lastComment, tokensAndComments]
   ,-[files/069.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 70:
  | [lastComment, comments, tokens, lastToken, tokensAndComments]
   ,-[files/070.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 71:
  | [tokens, lastToken, comments, lastComment, tokensAndComments]
   ,-[files/071.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 72:
  | [comments, lastToken, tokens, lastComment, tokensAndComments]
   ,-[files/072.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 73:
  | [tokensAndComments, lastToken, tokens, lastComment, comments]
   ,-[files/073.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 74:
  | [lastToken, tokensAndComments, tokens, lastComment, comments]
   ,-[files/074.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 75:
  | [lastComment, tokensAndComments, tokens, lastToken, comments]
   ,-[files/075.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 76:
  | [tokens, lastComment, comments, lastToken, tokensAndComments]
   ,-[files/076.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 77:
  | [comments, lastComment, tokens, lastToken, tokensAndComments]
   ,-[files/077.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 78:
  | [tokensAndComments, lastComment, tokens, lastToken, comments]
   ,-[files/078.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 79:
  | [lastToken, lastComment, tokens, tokensAndComments, comments]
   ,-[files/079.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 80:
  | [lastComment, lastToken, tokens, tokensAndComments, comments]
   ,-[files/080.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 81:
  | [tokens, comments, lastToken, lastComment, tokensAndComments]
   ,-[files/081.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 82:
  | [comments, tokens, lastToken, lastComment, tokensAndComments]
   ,-[files/082.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 83:
  | [tokensAndComments, tokens, lastToken, lastComment, comments]
   ,-[files/083.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 84:
  | [lastToken, tokens, tokensAndComments, lastComment, comments]
   ,-[files/084.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 85:
  | [lastComment, tokens, tokensAndComments, lastToken, comments]
   ,-[files/085.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 86:
  | [tokens, tokensAndComments, lastToken, lastComment, comments]
   ,-[files/086.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 87:
  | [comments, tokensAndComments, lastToken, lastComment, tokens]
   ,-[files/087.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 88:
  | [tokensAndComments, comments, lastToken, lastComment, tokens]
   ,-[files/088.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 89:
  | [lastToken, comments, tokensAndComments, lastComment, tokens]
   ,-[files/089.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 90:
  | [lastComment, comments, tokensAndComments, lastToken, tokens]
   ,-[files/090.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 91:
  | [tokens, lastToken, tokensAndComments, lastComment, comments]
   ,-[files/091.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 92:
  | [comments, lastToken, tokensAndComments, lastComment, tokens]
   ,-[files/092.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 93:
  | [tokensAndComments, lastToken, comments, lastComment, tokens]
   ,-[files/093.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 94:
  | [lastToken, tokensAndComments, comments, lastComment, tokens]
   ,-[files/094.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 95:
  | [lastComment, tokensAndComments, comments, lastToken, tokens]
   ,-[files/095.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 96:
  | [tokens, lastComment, tokensAndComments, lastToken, comments]
   ,-[files/096.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 97:
  | [comments, lastComment, tokensAndComments, lastToken, tokens]
   ,-[files/097.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 98:
  | [tokensAndComments, lastComment, comments, lastToken, tokens]
   ,-[files/098.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 99:
  | [lastToken, lastComment, comments, tokensAndComments, tokens]
   ,-[files/099.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 100:
  | [lastComment, lastToken, comments, tokensAndComments, tokens]
   ,-[files/100.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 101:
  | [tokens, comments, lastComment, lastToken, tokensAndComments]
   ,-[files/101.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 102:
  | [comments, tokens, lastComment, lastToken, tokensAndComments]
   ,-[files/102.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 103:
  | [tokensAndComments, tokens, lastComment, lastToken, comments]
   ,-[files/103.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 104:
  | [lastToken, tokens, lastComment, tokensAndComments, comments]
   ,-[files/104.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 105:
  | [lastComment, tokens, lastToken, tokensAndComments, comments]
   ,-[files/105.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 106:
  | [tokens, tokensAndComments, lastComment, lastToken, comments]
   ,-[files/106.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 107:
  | [comments, tokensAndComments, lastComment, lastToken, tokens]
   ,-[files/107.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 108:
  | [tokensAndComments, comments, lastComment, lastToken, tokens]
   ,-[files/108.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 109:
  | [lastToken, comments, lastComment, tokensAndComments, tokens]
   ,-[files/109.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 110:
  | [lastComment, comments, lastToken, tokensAndComments, tokens]
   ,-[files/110.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 111:
  | [tokens, lastToken, lastComment, tokensAndComments, comments]
   ,-[files/111.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 112:
  | [comments, lastToken, lastComment, tokensAndComments, tokens]
   ,-[files/112.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 113:
  | [tokensAndComments, lastToken, lastComment, comments, tokens]
   ,-[files/113.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 114:
  | [lastToken, tokensAndComments, lastComment, comments, tokens]
   ,-[files/114.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 115:
  | [lastComment, tokensAndComments, lastToken, comments, tokens]
   ,-[files/115.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 116:
  | [tokens, lastComment, lastToken, tokensAndComments, comments]
   ,-[files/116.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 117:
  | [comments, lastComment, lastToken, tokensAndComments, tokens]
   ,-[files/117.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 118:
  | [tokensAndComments, lastComment, lastToken, comments, tokens]
   ,-[files/118.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 119:
  | [lastToken, lastComment, tokensAndComments, comments, tokens]
   ,-[files/119.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

  x tokens-and-comments-order-plugin(tokens-and-comments-order): OK
  | Permutation 120:
  | [lastComment, lastToken, tokensAndComments, comments, tokens]
   ,-[files/120.js:1:1]
 1 | ,-> #!/usr/bin/env node
 2 | |   // Leading comment
 3 | |   let x = /* inline */ 1;
 4 | |   let y = 2;
 5 | `-> // Trailing comment
   `----

Found 0 warnings and 120 errors.
Finished in Xms on 120 files with 1 rules using X threads.
```

# stderr
```
```
