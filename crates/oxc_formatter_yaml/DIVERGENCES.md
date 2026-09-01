# Known divergences

Admission reasons and rules: see `crates/oxc_formatter_core/FORMATTER_POLICY.md` "Known divergences".

## anchor-tag-props-order

- Why: prettier-bug (prettier/prettier#19524)
- Pin: `tests/fixtures/yaml/anchor-tag-order.yaml`
- Drop when: the pin catches up to prettier#19599 (fixed upstream, unreleased)

```yaml
# input
- &a2 !!str two

# ours
- &a2 !!str two

# prettier
- !!str &a2 two
```

Anchor/tag source order is preserved, never reordered.

## block-scalar-trailing-whitespace

- Why: semantics
- Pin: `tests/fixtures/yaml/block-scalar-trailing-spaces.yaml`, `tests/fixtures/yaml/prose-wrap/trailing-spaces.yaml`
- Drop when: the pin catches up to prettier#19764 (fixed upstream, unreleased)

```yaml
# input ("␣" marks a real space)
strip: |-
  value␣␣

# ours: both spaces stay
# prettier: drops them, changing the value
```

Trailing whitespace in a block scalar is part of the VALUE:
the last content line's spaces/tabs, and space-only lines more-indented than the block (content per YAML).
`block-folded-strip.yml` stays a conformance failure until the pin catches up.
When converging, keep the blank line after such a scalar: post-#19764 Prettier eats it; the unified blank-line rule ("blank-lines" below) wins.

## eof-blank-lines

- Why: uniform-rule
- Pin: `tests/fixtures/yaml/eof-blank-lines.yaml`

```yaml
# input
a: 1

# c
␣    <- two blank lines here

# ours: the file ends with exactly one newline
# prettier: keeps an EOF blank line
```

Like every other formatter crate, the file always ends with exactly one newline (`|+` keep-chomped verbatim tails excepted); Prettier YAML alone preserves EOF blank lines.

## keep-chomped-space-only-eof-line

- Why: semantics (prettier/prettier#19256 is the nearest issue)
- Pin: `tests/fixtures/yaml/keep-chomped-eos-spaces-only.yaml`, `tests/fixtures/yaml/keep-chomped-eos-trailing-spaces.yaml`

```yaml
# input (no final newline; the last line is two spaces)
key: |+
␣␣

# ours: value "\n" (the break-less space-only line adds nothing to the kept tail)
# prettier: prints one newline too many, value "\n" -> "\n\n"
```

A space-only EOF line at-or-below the block's indent holds no line break, so it adds nothing to the kept tail (psych/PyYAML agree).

## prettier-ignore-range

- Why: prettier-bug (prettier/prettier#13008)
- Pin: `tests/fixtures/yaml/suppression.yaml`

```yaml
# input
# oxfmt-ignore
kept:   {  as: is  }
reformatted:   {  a: 1  }

# ours
# oxfmt-ignore
kept:   {  as: is  }
reformatted: { a: 1 }

# prettier: suppresses every following node, both lines stay verbatim
```

A suppression comment freezes exactly ONE node, never everything after it.

## blank-lines

- Why: uniform-rule (prettier/prettier#15528)
- Pin: `tests/fixtures/yaml/blank-lines.yaml`, `tests/fixtures/yaml/nested-end-comment-blank.yaml`

```yaml
# input
- a
- b


# blank above

# ours
- a
- b

# blank above

# prettier
- a
- b
# blank above
```

One unified rule: a blank line right after a node is preserved (normalized to one) if the source had one, never invented, identical for every node kind and context.
Prettier's matrix (block collections only between documents; mappings only before end comments; unconditional insertion after block scalars) is not ported.
This also keeps `proseWrap: never` idempotent where Prettier is not (prettier#10776),
and covers the blank DOUBLED in front of stream-end comments when the last item carries a trailing comment (the prettier#9130 shape, resurfaced: one source blank comes out as two).

## folded-more-indented-reflow

- Why: semantics (prettier/prettier#16126)
- Pin: `tests/fixtures/yaml/prose-wrap/more-indented.yaml`

```yaml
# input (proseWrap: always)
folded: >
  First Line.
           This more-indented line exceeds the print width but must not be broken.

# ours: the more-indented line is kept intact
# prettier: wraps it at the print width, changing the parsed value
```

More-indented lines in a folded scalar are never re-flowed under `proseWrap: always`: their line breaks are literal per YAML folding, so wrapping breaks idempotency and the value.

## flow-flat-with-newline

- Why: uniform-rule
- Pin: `tests/fixtures/yaml/flow-multiline-pair.yaml`, `tests/fixtures/yaml/flow-comments.yaml`

```yaml
# input
- [? foo
    bar
  : baz]

# ours
- [
    ? foo
      bar
    : baz,
  ]

# prettier
- [? foo
      bar
    : baz]
```

A flow collection either fits on one line or breaks normally (trailing comma, bracket on its own line).
Prettier sometimes emits a newline inside flow brackets while keeping them flat (no trailing comma, `]`/`}` on the content line): multiline pairs (spec-example-7-20 / 9-4) and key trailing comments.

## flow-comment-position

- Why: prettier-bug (attachment artifact, the spec-example-6-1 shape)
- Pin: `tests/fixtures/yaml/flow-comments.yaml`

```yaml
# input
key: [ # kept inside the brackets
  a,
  b ]

# ours
key: [
    # kept inside the brackets
    a,
    b,
  ]

# prettier
key: # kept inside the brackets
  [a, b]
```

A comment stays at its syntactic position; Prettier hoists a comment after `[` onto the `key:` line.

## comment-over-indented

- Why: prettier-bug
- Pin: `tests/fixtures/yaml/comment-over-indented.yaml`

```yaml
# input
Properties:
  Type: application
    # over-indented comment
  Other: 1

# ours: unchanged
# prettier
Properties:
  Type:
    application
    # over-indented comment
  Other: 1
```

A comment indented deeper than the value it follows never rewrites that value's layout; comment indentation alone must not break the preceding pair onto two lines.

## block-scalar-header-comment-width

- Why: uniform-rule
- Pin: `tests/fixtures/yaml/block-scalar-header-comment-width.yaml`

```yaml
# input
run: | # this trailing comment is long enough to push the header line far beyond every print width
  set -euo pipefail

# ours: unchanged, the comment is a line_suffix and never counts toward fits
# prettier
run:
  | # this trailing comment is long enough to push the header line far beyond every print width
  set -euo pipefail
```

A same-line trailing comment never counts toward the `fits` measurement, the treatment Prettier itself gives JS/JSON line comments and YAML flow collections, but not the block scalar header.
The KEY does count: a long key overflowing on `key: |` alone breaks the pair exactly like Prettier.
