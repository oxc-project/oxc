# Exit code
1

# stdout
```
  x unicode-comments(unicode-comments): {
  |     "lines": "1,0-1,53",
  |     "value": " Unicode test with emojis and multi-byte characters"
  | }
   ,-[files/index.js:1:1]
 1 | // Unicode test with emojis and multi-byte characters
   : ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
 2 | const greeting = "Hello 🌍"; // Line comment with emoji
   `----

  x unicode-comments(unicode-comments): {
  |     "lines": "2,29-2,55",
  |     "value": " Line comment with emoji"
  | }
   ,-[files/index.js:2:32]
 1 | // Unicode test with emojis and multi-byte characters
 2 | const greeting = "Hello 🌍"; // Line comment with emoji
   :                              ^^^^^^^^^^^^^^^^^^^^^^^^^^
 3 | 
   `----

  x unicode-comments(unicode-comments): {
  |     "lines": "4,0-8,3",
  |     "value": "*\n * Function with emoji in JSDoc\n * @param {string} name - User's name 👤\n * @returns {string} Greeting message\n "
  | }
   ,-[files/index.js:4:1]
 3 |     
 4 | ,-> /**
 5 | |    * Function with emoji in JSDoc
 6 | |    * @param {string} name - User's name 👤
 7 | |    * @returns {string} Greeting message
 8 | `->  */
 9 |     function greetUser(name) {
   `----

  x unicode-comments(unicode-comments): {
  |     "lines": "10,2-10,39",
  |     "value": " Comment with multiple emojis 🚀⭐💫"
  | }
    ,-[files/index.js:10:3]
  9 | function greetUser(name) {
 10 |   // Comment with multiple emojis 🚀⭐💫
    :   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
 11 |   const message = `Hello ${name}! 🌟`;
    `----

  x unicode-comments(unicode-comments): {
  |     "lines": "12,2-12,42",
  |     "value": " Block comment with unicode: ñáéíóú "
  | }
    ,-[files/index.js:12:3]
 11 |   const message = `Hello ${name}! 🌟`;
 12 |   /* Block comment with unicode: ñáéíóú */
    :   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
 13 |   return message;
    `----

  x unicode-comments(unicode-comments): {
  |     "lines": "16,0-16,30",
  |     "value": " Multi-byte comment: 你好世界 "
  | }
    ,-[files/index.js:16:1]
 15 | 
 16 | /* Multi-byte comment: 你好世界 */
    : ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
 17 | const 你好世界 = "Testing üöä"; // Line comment: ñáéíóú
    `----

  x unicode-comments(unicode-comments): {
  |     "lines": "17,28-17,51",
  |     "value": " Line comment: ñáéíóú"
  | }
    ,-[files/index.js:17:40]
 16 | /* Multi-byte comment: 你好世界 */
 17 | const 你好世界 = "Testing üöä"; // Line comment: ñáéíóú
    :                                 ^^^^^^^^^^^^^^^^^^^^^^^
 18 | 
    `----

  x unicode-comments(unicode-comments): {
  |     "lines": "19,0-22,3",
  |     "value": "*\n * JSDoc with emojis and unicode: 你好 👋\n * @param {number} count - Number of items 🔢\n "
  | }
    ,-[files/index.js:19:1]
 18 |     
 19 | ,-> /**
 20 | |    * JSDoc with emojis and unicode: 你好 👋
 21 | |    * @param {number} count - Number of items 🔢
 22 | `->  */
 23 |     function processItems(count) {
    `----

  x unicode-comments(unicode-comments): {
  |     "lines": "24,2-24,52",
  |     "value": " Comment with mixed unicode: αβγδε русский עברית"
  | }
    ,-[files/index.js:24:3]
 23 | function processItems(count) {
 24 |   // Comment with mixed unicode: αβγδε русский עברית
    :   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
 25 |   const result = count * 2; /* Block: ñáéíóú 🚀 */
    `----

  x unicode-comments(unicode-comments): {
  |     "lines": "25,28-25,50",
  |     "value": " Block: ñáéíóú 🚀 "
  | }
    ,-[files/index.js:25:29]
 24 |   // Comment with mixed unicode: αβγδε русский עברית
 25 |   const result = count * 2; /* Block: ñáéíóú 🚀 */
    :                             ^^^^^^^^^^^^^^^^^^^^^^
 26 |   return result;
    `----

  x unicode-comments(unicode-comments): {
  |     "lines": "29,0-29,34",
  |     "value": " Final comment with emoji: 🎉✨🎊"
  | }
    ,-[files/index.js:29:1]
 28 | 
 29 | // Final comment with emoji: 🎉✨🎊
    : ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
 30 | const finalVar = "Done ✅";
    `----

Found 0 warnings and 11 errors.

Errors  Files
    11  files/index.js:1

Finished in Xms on 1 file with 1 rules using X threads.
```

# stderr
```
```
