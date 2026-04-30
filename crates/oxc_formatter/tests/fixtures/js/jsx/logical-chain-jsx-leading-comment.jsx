// JSX `&&` chain whose final operand is a parenthesized JSX element with a leading
// inline `//` comment after `(`. The chain should break when the inline comment
// would push the collapsed line past `printWidth`.
const A = () => (
  <div>
    {someLongConditionA &&
      someLongConditionB && ( // an explanatory comment that pushes the line past printWidth
        <span>content</span>
      )}
  </div>
);

// Same but with a leading own-line block comment before the JSX element.
const B = () => (
  <div>
    {someLongConditionA &&
      someLongConditionB && (
        /* an explanatory block comment that forces the JSX onto its own lines */
        <span>content</span>
      )}
  </div>
);
