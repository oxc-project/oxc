// prettier/prettier#19725: embedded `${expr}` no longer preserves source indentation
_ = gql`

                  ${
                    a
                    // comment
                    + b}

`;
