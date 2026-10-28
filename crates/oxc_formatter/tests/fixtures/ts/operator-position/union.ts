// `operatorPosition` does NOT apply to unions: `|` leads the members in either
// mode (both option sections below must stay identical; see NOTE in union_type.rs).
type Long = AAAAAAAAAAAAAAAAAAAAAAAA | BBBBBBBBBBBBBBBBBBBBBBBB | CCCCCCCCCCCCCCCCCCCCCCCC;

// A leading own-line comment stays own-line, above the leading `|`.
type WithComment = SerializedProps |
  // own line comment
  { cause: unknown };

// A blank line before the hoisted comment is preserved (trailing-style rendering);
// binary chains and intersections collapse it instead, like Prettier does.
type WithBlank = AAAAAAAAAAAAAAAAAAAAAAAA |

  // own line after blank
  BBBBBBBBBBBBBBBBBBBBBBBB | CCCCCCCCCCCCCCCCCCCCCCCC;
