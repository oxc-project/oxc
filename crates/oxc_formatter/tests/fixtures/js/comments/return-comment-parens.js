// A return/throw argument with a leading comment gets the statement's own parentheses;
// a sequence or assignment prints no pair of its own inside them
// (Prettier's `willReturnOrThrowStatementBreak`).
function f() {
  return ( // c1
    a, b
  );
  return (
    // c2
    a = b
  );
  throw (
    // c3
    a, b
  );
  throw ( // c4
    a = b
  );
  // Without the comment the pair is the argument's own
  return a = b;
  return (a, b);
  throw (a, b);
}
