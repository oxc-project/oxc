// Unicode test with emojis and multi-byte characters
const greeting = "Hello 🌍"; // Line comment with emoji

/**
 * Function with emoji in JSDoc
 * @param {string} name - User's name 👤
 * @returns {string} Greeting message
 */
function greetUser(name) {
  // Comment with multiple emojis 🚀⭐💫
  const message = `Hello ${name}! 🌟`;
  /* Block comment with unicode: ñáéíóú */
  return message;
}

/* Multi-byte comment: 你好世界 */
const 你好世界 = "Testing üöä"; // Line comment: ñáéíóú

/**
 * JSDoc with emojis and unicode: 你好 👋
 * @param {number} count - Number of items 🔢
 */
function processItems(count) {
  // Comment with mixed unicode: αβγδε русский עברית
  const result = count * 2; /* Block: ñáéíóú 🚀 */
  return result;
}

// Final comment with emoji: 🎉✨🎊
const finalVar = "Done ✅";
