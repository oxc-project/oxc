// https://github.com/oxc-project/oxc/issues/25494
// When comments sit before the argument or between the argument and the closing paren,
// the unary provides the single pair of parentheses and the comments stay inside.
function foo() {
  return !(
    a && // A
    b // B
  );
}

// Known divergence: a trailing comment before the closing paren does not break
// the operand chain, so pre-broken operands collapse (`a && b // B`) — the same
// behavior both formatters have for return/throw arguments, call arguments,
// assignments, and arrow bodies. Only here Prettier preserves the source break
// (attachment binds the comment to the last operand when alone on its line);
// with the same code on one line, Prettier also keeps it flat (see below).
!(
  a &&
  b // B
);
!(
  a +
  b // B
);

// Same shapes on one line: flat, matching Prettier.
!(x && y // B
);
!(x + y // B
);

!(a // B
);

!(/* L */ a && b);

!(a && b /* B */);
