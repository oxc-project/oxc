# Exit code
1

# stdout
```
  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1municode-comments(unicode-comments): {
  [38;2;225;80;80;1m│[0m     "lines": "1,0-1,53",
  [38;2;225;80;80;1m│[0m     "value": " Unicode test with emojis and multi-byte characters"
  [38;2;225;80;80;1m│[0m }[0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:1:1]
 [2m1[0m │ // Unicode test with emojis and multi-byte characters
   · [38;2;246;87;248m─────────────────────────────────────────────────────[0m
 [2m2[0m │ const greeting = "Hello 🌍"; // Line comment with emoji
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1municode-comments(unicode-comments): {
  [38;2;225;80;80;1m│[0m     "lines": "2,29-2,55",
  [38;2;225;80;80;1m│[0m     "value": " Line comment with emoji"
  [38;2;225;80;80;1m│[0m }[0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:2:32]
 [2m1[0m │ // Unicode test with emojis and multi-byte characters
 [2m2[0m │ const greeting = "Hello 🌍"; // Line comment with emoji
   · [38;2;246;87;248m                             ──────────────────────────[0m
 [2m3[0m │ 
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1municode-comments(unicode-comments): {
  [38;2;225;80;80;1m│[0m     "lines": "4,0-8,3",
  [38;2;225;80;80;1m│[0m     "value": "*\n * Function with emoji in JSDoc\n * @param {string} name - User's name 👤\n * @returns {string} Greeting message\n "
  [38;2;225;80;80;1m│[0m }[0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:4:1]
 [2m3[0m │     
 [2m4[0m │ [38;2;246;87;248m╭[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m /**
 [2m5[0m │ [38;2;246;87;248m│[0m    * Function with emoji in JSDoc
 [2m6[0m │ [38;2;246;87;248m│[0m    * @param {string} name - User's name 👤
 [2m7[0m │ [38;2;246;87;248m│[0m    * @returns {string} Greeting message
 [2m8[0m │ [38;2;246;87;248m╰[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m  */
 [2m9[0m │     function greetUser(name) {
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1municode-comments(unicode-comments): {
  [38;2;225;80;80;1m│[0m     "lines": "10,2-10,39",
  [38;2;225;80;80;1m│[0m     "value": " Comment with multiple emojis 🚀⭐💫"
  [38;2;225;80;80;1m│[0m }[0m
    ╭─[[38;2;92;157;255;1mfiles/index.js[0m:10:3]
 [2m 9[0m │ function greetUser(name) {
 [2m10[0m │   // Comment with multiple emojis 🚀⭐💫
    · [38;2;246;87;248m  ──────────────────────────────────────[0m
 [2m11[0m │   const message = `Hello ${name}! 🌟`;
    ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1municode-comments(unicode-comments): {
  [38;2;225;80;80;1m│[0m     "lines": "12,2-12,42",
  [38;2;225;80;80;1m│[0m     "value": " Block comment with unicode: ñáéíóú "
  [38;2;225;80;80;1m│[0m }[0m
    ╭─[[38;2;92;157;255;1mfiles/index.js[0m:12:3]
 [2m11[0m │   const message = `Hello ${name}! 🌟`;
 [2m12[0m │   /* Block comment with unicode: ñáéíóú */
    · [38;2;246;87;248m  ────────────────────────────────────────[0m
 [2m13[0m │   return message;
    ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1municode-comments(unicode-comments): {
  [38;2;225;80;80;1m│[0m     "lines": "16,0-16,30",
  [38;2;225;80;80;1m│[0m     "value": " Multi-byte comment: 你好世界 "
  [38;2;225;80;80;1m│[0m }[0m
    ╭─[[38;2;92;157;255;1mfiles/index.js[0m:16:1]
 [2m15[0m │ 
 [2m16[0m │ /* Multi-byte comment: 你好世界 */
    · [38;2;246;87;248m──────────────────────────────────[0m
 [2m17[0m │ const 你好世界 = "Testing üöä"; // Line comment: ñáéíóú
    ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1municode-comments(unicode-comments): {
  [38;2;225;80;80;1m│[0m     "lines": "17,28-17,51",
  [38;2;225;80;80;1m│[0m     "value": " Line comment: ñáéíóú"
  [38;2;225;80;80;1m│[0m }[0m
    ╭─[[38;2;92;157;255;1mfiles/index.js[0m:17:40]
 [2m16[0m │ /* Multi-byte comment: 你好世界 */
 [2m17[0m │ const 你好世界 = "Testing üöä"; // Line comment: ñáéíóú
    · [38;2;246;87;248m                                ───────────────────────[0m
 [2m18[0m │ 
    ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1municode-comments(unicode-comments): {
  [38;2;225;80;80;1m│[0m     "lines": "19,0-22,3",
  [38;2;225;80;80;1m│[0m     "value": "*\n * JSDoc with emojis and unicode: 你好 👋\n * @param {number} count - Number of items 🔢\n "
  [38;2;225;80;80;1m│[0m }[0m
    ╭─[[38;2;92;157;255;1mfiles/index.js[0m:19:1]
 [2m18[0m │     
 [2m19[0m │ [38;2;246;87;248m╭[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m /**
 [2m20[0m │ [38;2;246;87;248m│[0m    * JSDoc with emojis and unicode: 你好 👋
 [2m21[0m │ [38;2;246;87;248m│[0m    * @param {number} count - Number of items 🔢
 [2m22[0m │ [38;2;246;87;248m╰[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m  */
 [2m23[0m │     function processItems(count) {
    ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1municode-comments(unicode-comments): {
  [38;2;225;80;80;1m│[0m     "lines": "24,2-24,52",
  [38;2;225;80;80;1m│[0m     "value": " Comment with mixed unicode: αβγδε русский עברית"
  [38;2;225;80;80;1m│[0m }[0m
    ╭─[[38;2;92;157;255;1mfiles/index.js[0m:24:3]
 [2m23[0m │ function processItems(count) {
 [2m24[0m │   // Comment with mixed unicode: αβγδε русский עברית
    · [38;2;246;87;248m  ──────────────────────────────────────────────────[0m
 [2m25[0m │   const result = count * 2; /* Block: ñáéíóú 🚀 */
    ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1municode-comments(unicode-comments): {
  [38;2;225;80;80;1m│[0m     "lines": "25,28-25,50",
  [38;2;225;80;80;1m│[0m     "value": " Block: ñáéíóú 🚀 "
  [38;2;225;80;80;1m│[0m }[0m
    ╭─[[38;2;92;157;255;1mfiles/index.js[0m:25:29]
 [2m24[0m │   // Comment with mixed unicode: αβγδε русский עברית
 [2m25[0m │   const result = count * 2; /* Block: ñáéíóú 🚀 */
    · [38;2;246;87;248m                            ──────────────────────[0m
 [2m26[0m │   return result;
    ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1municode-comments(unicode-comments): {
  [38;2;225;80;80;1m│[0m     "lines": "29,0-29,34",
  [38;2;225;80;80;1m│[0m     "value": " Final comment with emoji: 🎉✨🎊"
  [38;2;225;80;80;1m│[0m }[0m
    ╭─[[38;2;92;157;255;1mfiles/index.js[0m:29:1]
 [2m28[0m │ 
 [2m29[0m │ // Final comment with emoji: 🎉✨🎊
    · [38;2;246;87;248m───────────────────────────────────[0m
 [2m30[0m │ const finalVar = "Done ✅";
    ╰────

Found 0 warnings and 11 errors.
Finished in Xms on 1 file using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
