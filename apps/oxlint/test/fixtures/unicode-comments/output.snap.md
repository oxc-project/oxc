# Exit code
1

# stdout
```
  x unicode-comments(unicode-comments): getAllComments: [
  |     {
  |         "type": "Line",
  |         "value": " Unicode test with emojis and multi-byte characters"
  |     },
  |     {
  |         "type": "Line",
  |         "value": " Line comment with emoji"
  |     },
  |     {
  |         "type": "Block",
  |         "value": "*\n * Function with emoji in JSDoc\n * @param {string} name - User's name 👤\n * @returns {string} Greeting message\n "
  |     },
  |     {
  |         "type": "Line",
  |         "value": " Comment with multiple emojis 🚀⭐💫"
  |     },
  |     {
  |         "type": "Block",
  |         "value": " Block comment with unicode: ñáéíóú "
  |     },
  |     {
  |         "type": "Block",
  |         "value": " Multi-byte comment: 你好世界 "
  |     },
  |     {
  |         "type": "Line",
  |         "value": " Line comment: ñáéíóú"
  |     },
  |     {
  |         "type": "Block",
  |         "value": "*\n * JSDoc with emojis and unicode: 你好 👋\n * @param {number} count - Number of items 🔢\n "
  |     },
  |     {
  |         "type": "Line",
  |         "value": " Comment with mixed unicode: αβγδε русский עברית"
  |     },
  |     {
  |         "type": "Block",
  |         "value": " Block: ñáéíóú 🚀 "
  |     },
  |     {
  |         "type": "Line",
  |         "value": " Final comment with emoji: 🎉✨🎊"
  |     }
  | ]
    ,-[files/unicode-comments.js:2:1]
  1 |     // Unicode test with emojis and multi-byte characters
  2 | ,-> const greeting = 'Hello 🌍'; // Line comment with emoji
  3 | |   
  4 | |   /**
  5 | |    * Function with emoji in JSDoc
  6 | |    * @param {string} name - User's name 👤
  7 | |    * @returns {string} Greeting message
  8 | |    */
  9 | |   function greetUser(name) {
 10 | |     // Comment with multiple emojis 🚀⭐💫
 11 | |     const message = `Hello ${name}! 🌟`;
 12 | |     /* Block comment with unicode: ñáéíóú */
 13 | |     return message;
 14 | |   }
 15 | |   
 16 | |   /* Multi-byte comment: 你好世界 */
 17 | |   const 你好世界 = 'Testing üöä'; // Line comment: ñáéíóú
 18 | |   
 19 | |   /**
 20 | |    * JSDoc with emojis and unicode: 你好 👋
 21 | |    * @param {number} count - Number of items 🔢
 22 | |    */
 23 | |   function processItems(count) {
 24 | |     // Comment with mixed unicode: αβγδε русский עברית
 25 | |     const result = count * 2; /* Block: ñáéíóú 🚀 */
 26 | |     return result;
 27 | |   }
 28 | |   
 29 | |   // Final comment with emoji: 🎉✨🎊
 30 | `-> const finalVar = 'Done ✅';
    `----

Found 0 warnings and 1 error.
Finished in Xms on 1 file using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
