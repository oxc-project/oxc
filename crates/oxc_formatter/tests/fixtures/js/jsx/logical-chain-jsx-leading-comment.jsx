// 1. Long `&&` chain whose final JSX operand has a `(`-trailing inline `//`
//    comment. The chain must break because the inline comment forces a newline.
//    The break is structural, not width-driven: the comment rides a `line_suffix`
//    and never counts toward `printWidth` (case 3 breaks the same way with a
//    tiny comment).
const A = () => (
  <div>
    {someLongConditionA &&
      someLongConditionB && ( // an explanatory comment that pushes the line past printWidth
        <span>content</span>
      )}
  </div>
);

// 2. Same chain but with an own-line block comment instead of an inline `//`.
//    A block comment does not force a newline, so the chain should stay flat
//    when it still fits within `printWidth`.
const B = () => (
  <div>
    {someLongConditionA &&
      someLongConditionB && (
        /* an explanatory block comment that does not force the chain to break */
        <span>content</span>
      )}
  </div>
);

// 3. Short conditions + inline `//` comment after `(`. The chain must still
//    break because the line comment forces a newline.
const C = () => (
  <div>
    {a && b && ( // tiny
        <span>x</span>
      )}
  </div>
);

// 4. Single operand (no chain) + inline `//` comment after `(`. There is no
//    chain to break, so nothing extra should happen.
const D = () => (
  <div>
    {a && ( // tiny
        <span>x</span>
      )}
  </div>
);
