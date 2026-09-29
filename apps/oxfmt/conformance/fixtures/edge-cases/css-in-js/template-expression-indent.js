// DIVERGES: embedded `${expr}` re-indents to the placeholder (Prettier preserves the source indentation);
// see apps/oxfmt/DIVERGENCES.md#template-expression-indent
_ = css`
  a{
    color:
                  ${
                    a
                    // comment
                    + b}
    ;
  }
`;
