// The `define` calls deviate from Prettier (DIVERGENCES.md#amd-call-comment-around-arguments).
define("lib/a", ["deps/b"],
/**
 * @param {B} b
 */
(b) => {
  return b;
});
define(["a"], // c
function (a) {
  return a;
});
define(
// c
["a"], function (a) {
  return a;
});
define(["a"], function (a) {
  return a;
} // c
);
// Prettier also breaks these: their first argument has a comment.
require(["a"], // c
function (a) {
  return a;
});
require("x" // c
);
// A comment that doesn't end its line keeps the AMD layout.
define("lib/a", ["deps/b"], /* c */ (b) => {
  return b;
});
