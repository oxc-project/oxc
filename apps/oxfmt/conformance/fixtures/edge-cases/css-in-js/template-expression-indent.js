// prettier/prettier#19725: embedded `${expr}` no longer preserves source indentation
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
