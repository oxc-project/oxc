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
  |     "lines": "4,0-9,3",
  |     "value": "*\n * Function with emoji in JSDoc\n *\n * @param {string} name - User's name 👤\n * @returns {string} Greeting message\n "
  | }
    ,-[files/index.js:4:1]
  3 |     
  4 | ,-> /**
  5 | |    * Function with emoji in JSDoc
  6 | |    *
  7 | |    * @param {string} name - User's name 👤
  8 | |    * @returns {string} Greeting message
  9 | `->  */
 10 |     function greetUser(name) {
    `----

  x unicode-comments(unicode-comments): {
  |     "lines": "11,2-11,39",
  |     "value": " Comment with multiple emojis 🚀⭐💫"
  | }
    ,-[files/index.js:11:3]
 10 | function greetUser(name) {
 11 |   // Comment with multiple emojis 🚀⭐💫
    :   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
 12 |   const message = `Hello ${name}! 🌟`;
    `----

  x unicode-comments(unicode-comments): {
  |     "lines": "13,2-13,42",
  |     "value": " Block comment with unicode: ñáéíóú "
  | }
    ,-[files/index.js:13:3]
 12 |   const message = `Hello ${name}! 🌟`;
 13 |   /* Block comment with unicode: ñáéíóú */
    :   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
 14 |   return message;
    `----

  x unicode-comments(unicode-comments): {
  |     "lines": "17,0-17,30",
  |     "value": " Multi-byte comment: 你好世界 "
  | }
    ,-[files/index.js:17:1]
 16 | 
 17 | /* Multi-byte comment: 你好世界 */
    : ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
 18 | const 你好世界 = "Testing üöä"; // Line comment: ñáéíóú
    `----

  x unicode-comments(unicode-comments): {
  |     "lines": "18,28-18,51",
  |     "value": " Line comment: ñáéíóú"
  | }
    ,-[files/index.js:18:40]
 17 | /* Multi-byte comment: 你好世界 */
 18 | const 你好世界 = "Testing üöä"; // Line comment: ñáéíóú
    :                                 ^^^^^^^^^^^^^^^^^^^^^^^
 19 | 
    `----

  x unicode-comments(unicode-comments): {
  |     "lines": "20,0-24,3",
  |     "value": "*\n * JSDoc with emojis and unicode: 你好 👋\n *\n * @param {number} count - Number of items 🔢\n "
  | }
    ,-[files/index.js:20:1]
 19 |     
 20 | ,-> /**
 21 | |    * JSDoc with emojis and unicode: 你好 👋
 22 | |    *
 23 | |    * @param {number} count - Number of items 🔢
 24 | `->  */
 25 |     function processItems(count) {
    `----

  x unicode-comments(unicode-comments): {
  |     "lines": "26,2-26,52",
  |     "value": " Comment with mixed unicode: αβγδε русский עברית"
  | }
    ,-[files/index.js:26:3]
 25 | function processItems(count) {
 26 |   // Comment with mixed unicode: αβγδε русский עברית
    :   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
 27 |   const result = count * 2; /* Block: ñáéíóú 🚀 */
    `----

  x unicode-comments(unicode-comments): {
  |     "lines": "27,28-27,50",
  |     "value": " Block: ñáéíóú 🚀 "
  | }
    ,-[files/index.js:27:29]
 26 |   // Comment with mixed unicode: αβγδε русский עברית
 27 |   const result = count * 2; /* Block: ñáéíóú 🚀 */
    :                             ^^^^^^^^^^^^^^^^^^^^^^
 28 |   return result;
    `----

  x unicode-comments(unicode-comments): {
  |     "lines": "31,0-31,34",
  |     "value": " Final comment with emoji: 🎉✨🎊"
  | }
    ,-[files/index.js:31:1]
 30 | 
 31 | // Final comment with emoji: 🎉✨🎊
    : ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
 32 | const finalVar = "Done ✅";
    `----

Found 0 warnings and 11 errors.
Finished in Xms on 1 file with 1 rules using X threads.
```

# stderr
```
```
