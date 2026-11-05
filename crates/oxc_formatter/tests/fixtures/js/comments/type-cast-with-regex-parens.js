// `)` and `(` bytes inside a regex literal must not be mistaken for the
// cast's parentheses: the cast target is determined from the span structure
// and the trivia gap after the comment, never by scanning expression bytes.
var a = /** @type {D} */ (s.replace(/\)/g, ""));
var b = /** @type {D} */ (s.match(/[)]/));
var c = /** @type {D} */ (s.match(/\(/));
var d = x ? y : /** @type {D} */ (a).b.match(/\(/) ?? z;
