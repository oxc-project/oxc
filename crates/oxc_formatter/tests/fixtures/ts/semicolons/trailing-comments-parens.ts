// Trailing comments and parentheses (the statement-level basics:
// trailing-comments.ts).
// A return/throw argument's parentheses survive in the output, so comments
// inside them stay there (moving them behind the `;` would cross the `)` and,
// when the group breaks, a line boundary too — breaking line directives);
// only comments after the closing paren move behind the semicolon
function multiLineReturn() {
  return (
    aLongLongLongLongLongCondition &&
    anotherLongLongLongLongCondition // eslint-disable-line some-rule
  );
}
function ownLineCommentReturn() {
  return (
    aLongLongLongLongLongCondition &&
    anotherLongLongLongLongCondition
    /* eslint-enable some-rule */
  );
}
function afterCloseParenReturn() {
  return (
    aLongLongLongLongLongCondition &&
    anotherLongLongLongLongCondition
  ) /* moves */;
}
// No source `;` at all (ASI): nothing to move the comment across while the
// parens survive (group broken, printWidth 80); when the argument fits flat
// (printWidth 100) the parens are dropped and the comment settles only on the
// second format — the same group-fit limitation as a flat JSX arrow body
function asiReturn() {
  return (
    aLongLongLongLongLongCondition &&
    anotherLongLongLongLongCondition /* stays */
  )
}

// In an expression statement the parentheses are dropped, so the semicolon
// directly follows the content: a comment inside them also moves behind it,
// even from a nested assignment's dropped parentheses.
// (Prettier 3.9 moves these only on its second pass; oxfmt prints that
// fixpoint directly, see prettier#19893)
parenthesized = (someValue /* moves */);
parenthesizedChain = (inner = someValue /* moves */);
parenthesizedNested = (inner = (someValue /* moves */));
parenthesizedTernary = (cond ? someValue : other /* moves */);
parenthesizedLine = (inner = someValue // moves
);
// ... but not where the parentheses survive in the output (`keeps_trailing_comment_inside_parens`)
const parenthesizedInit = (inner = someValue /* stays */);

// The chain also passes through arrow expression bodies (prettier#19930 family):
// the body's parentheses are dropped, so its trailing comments
// move behind the statement's semicolon
chainedArrow = (a) => ((b) => {
  c();
} /* moves */ // rides
)
// A dropped `)` counts as the terminator even without a source `;` (ASI):
// the next format would move the comment behind the added `;` anyway,
// so print that fixpoint directly
asiParenChain = (inner = someValue /* moves */)
asiParenLeaf = (someValue /* moves */)
// A deferred own-line comment measures its break past the dropped `)` too:
// own-line and blank lines survive the vanished `)` line
asiParenOwnLine = (someValue
// defers
)

afterAsiParen();
// ... uniformly for any dropped-paren body, object parens included
// (the object's parens are re-printed on the statement's own terms)
arrowObjectBody = () => ({ a } /* moves */)
arrowPlainBody = (a) => (b /* moves */)
// ... including conditional bodies, whose parens are formatter-owned and
// often absent (object leftmost, broken groups)
arrowConditionalBody = (a) => (a ? b : c /* moves */);
arrowConditionalObjectLeftmost = (a) => (({}) ? b : c /* moves */);
// An arrow body whose printer re-adds the parens and keeps the comment
// inside them stops the walk: sequence, assignment (JSX too, see the
// jsx fixture)
arrowSequenceBody = (a) => ((b, c) /* stays */);
arrowAssignmentBody = (a) => (b = c /* stays */);
// The same chain rule in a variable declaration, a class property value,
// and a return argument (and export default, see trailing-comments.ts);
// other sites join the dropped-paren terminator rule through the shared gate
type ParenAlias = (X /* moves */)
var declaratorChain = (a) => ((b) => {
  c();
} /* moves */)
class ValueChain {
  p = (a) => (b /* moves */)
}
function returnChain() {
  return (a) => ((b) => {
    c();
  } /* moves */);
}
