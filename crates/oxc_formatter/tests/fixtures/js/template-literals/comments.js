const plain = `value ${
  // leading comment stays attached to identifier
  value
}`;

const call = `value ${
  // leading comment stays attached to call expression
  call()
}`;

const inlineBlock = `value ${/* leading block */ value}`;

const trailing = `value ${
  value // trailing comment stays before interpolation end
}`;

// Trailing comment stays inside `${}` even when the expression source contains `}`
const stringWithBrace = `${'{admin: 1}' /* comment */}`;
const objectLiteral = `${ {k: 1} /* comment */}`;
const nestedTemplate = `${ `x${y}z` /* comment */}`;
