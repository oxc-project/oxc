// `require(...)` prints like a `require` call's arguments: a lone string never breaks,
// a comment ending its line breaks the group, a same-line block comment stays inline
import A1 = require("./a/long/long/long/long/long/long/long/long/long/long/long/long/long/path/to/module");
import B1 = require(
  // c1
  "b"
);
import B2 = require(/* c2 */ "b");
import B3 = require(
  /* c3 */
  "b"
);
