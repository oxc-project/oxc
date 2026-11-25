# Known divergences

Admission reasons and rules: see `crates/oxc_formatter_core/FORMATTER_POLICY.md` "Known divergences".

## array-hole-comments

- Why: invariant
- Pin: `tests/fixtures/json/comments/array_hole_comments.json`

```json
// input
[1, /* keep */, 2]

// ours
[1, /* keep */, 2]

// prettier
[1 /* keep */, , 2]
```

A hole's `,` is content, it keeps the hole count.
A comment next to a hole stays on its source side of that `,`, the same as a comment next to a value (`[1, /* keep */ 2]`).
Prettier has no node for a hole to attach the comment to, so it moves the comment across one or more `,`.
Spacing follows the `,` like any separator: a comment before it is glued to it (`/* keep */,`, as in `1 /* keep */,`),
a comment after it is spaced (`, /* keep */`), the same as `oxc_formatter` (JS) prints array holes.
