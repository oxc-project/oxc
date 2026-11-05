# Known divergences

Admission reasons and rules: see `crates/oxc_formatter_core/FORMATTER_POLICY.md` "Known divergences".

## anchor-tag-props-order

- Why: uniform-rule (source order of user tokens is preserved; prettier/prettier#19524, prettier/prettier#19599)
- Pin: `tests/fixtures/yaml/anchor-tag-order.yaml`
- Conformance:
  - `externals/prettier/yaml/spec/anchors-and-tags.yml`
  - `externals/prettier/yaml/spec/node-anchor-and-tag-on-seperate-lines.yml`
  - `externals/prettier/yaml/spec/various-combinations-of-tags-and-anchors.yml`
  - `externals/prettier/yaml/yaml-test-suite/snippet: 9KAX.yaml*`
  - `externals/prettier/yaml/yaml-test-suite/snippet: BU8L.yaml*`
  - `externals/prettier/yaml/yaml-test-suite/snippet: F2C7.yaml*`

```yaml
# input
- &a2 !!str two

# ours
- &a2 !!str two

# prettier
- !!str &a2 two
```

Anchor/tag source order is preserved, never reordered, as mapping keys and sequence items are.

## block-scalar-trailing-whitespace

- Why: semantics (prettier/prettier#19764)
- Pin: `tests/fixtures/yaml/block-scalar-trailing-spaces.yaml`, `tests/fixtures/yaml/prose-wrap/trailing-spaces.yaml`
- Conformance:
  - `externals/prettier/yaml/block-folded/block-folded-strip.yml`
  - `externals/prettier/yaml/block-value/snippet: "foo: >\n  x\n   \n*`
  - `externals/prettier/yaml/block-value/snippet: "foo: >\n  x\n  \t\n*`
  - `externals/prettier/yaml/block-value/snippet: "foo: >-\n  x\n   \n*`
  - `externals/prettier/yaml/block-value/snippet: "foo: >-\n  x\n  \t\n*`
  - `externals/prettier/yaml/block-value/snippet: "foo: |\n  x\n   \n*`
  - `externals/prettier/yaml/block-value/snippet: "foo: |\n  x\n  \t\n*`
  - `externals/prettier/yaml/block-value/snippet: "foo: |-\n  x\n   \n*`
  - `externals/prettier/yaml/block-value/snippet: "foo: |-\n  x\n  \t\n*`
  - `externals/prettier/yaml/yaml-test-suite/snippet: L24T.yaml*` (its JSON keeps that line as content)
  - `externals/prettier/yaml/yaml-test-suite/snippet: L24T-2.yaml*` (its JSON keeps that line as content)
  - `externals/prettier/yaml/yaml-test-suite/snippet: Y79Y-2.yaml*` (its JSON keeps that line as content)

```yaml
# input ("␣" marks a real space)
strip: |-
  value␣␣

# ours: both spaces stay
# prettier: drops them, changing the value
```

Trailing whitespace in a block scalar is part of the VALUE:
the last content line's spaces/tabs, and space-only lines more-indented than the block (content per YAML).
When converging, keep the blank line after such a scalar: post-prettier/prettier#19764 Prettier eats it; the unified blank-line rule ("blank-lines" below) wins.

## eof-blank-lines

- Why: uniform-rule (one final newline)
- Pin: `tests/fixtures/yaml/eof-blank-lines.yaml`
- Conformance:
  - `externals/prettier/yaml/block-value/snippet: "foo: >+\n"`
  - `externals/prettier/yaml/block-value/snippet: "foo: |+\n"`
  - `externals/prettier/yaml/yaml-test-suite/snippet: 2G84-4.yaml*`

```yaml
# input
a: 1

# c
␣    <- two blank lines here

# ours: the file ends with exactly one newline
# prettier: keeps an EOF blank line
```

Like every other formatter crate, the file always ends with exactly one newline (`|+` keep-chomped verbatim tails excepted); Prettier YAML alone preserves EOF blank lines.
After a contentless keep-chomped scalar (its conformance failures), Prettier prints no final newline at all.

## keep-chomped-space-only-eof-line

- Why: semantics (prettier/prettier#19256)
- Pin: `tests/fixtures/yaml/keep-chomped-eos-spaces-only.yaml`, `tests/fixtures/yaml/keep-chomped-eos-trailing-spaces.yaml`
- Conformance:
  - `externals/prettier/yaml/yaml-test-suite/snippet: JEF9-3.yaml*` (the suite asserts no JSON value, only yaml@2's dump)

```yaml
# input (no final newline; the last line is two spaces)
key: |+
␣␣

# ours: value "" (the break-less space-only line adds nothing to the kept tail)
# prettier: prints one newline too many, value "\n"
```

A space-only EOF line at-or-below the block's indent holds no line break, so it adds nothing to the kept tail (psych/PyYAML agree).

## empty-document-end-marker

- Why: uniform-rule (blank lines are never invented; see "blank-lines")
- Pin: `tests/fixtures/yaml/empty-document-end-marker.yaml`
- Conformance:
  - `externals/prettier/yaml/yaml-test-suite/snippet: HWV9.yaml*`

```yaml
# input
...

# ours
...

# prettier: a blank line stands in for the empty document
␣
...
```

An explicit `...` after an empty document is printed as-is.
Prettier prints the empty document body as a blank line before the marker.

## tab-width-zero

- Why: semantics
- Pin: `tests/fixtures/yaml/tab-width-zero/nested.yaml`

```yaml
# input (tabWidth: 0)
a:
  b: c

# ours: the width is clamped to 1
a:
 b: c

# prettier: the nesting is gone, `b` becomes a top-level key
a:
b: c
```

Indentation is the block structure in YAML, so a zero indent width cannot print a nested collection.
`tabWidth: 0` is a valid option elsewhere (JS, CSS, GraphQL print without indentation and stay valid); YAML alone clamps it to `1`.
`useTabs` is the other layout option YAML cannot honor (tab indentation is forbidden); it needs no clamp because this printer only aligns with spaces.

## prettier-ignore-range

- Why: uniform-rule (suppression freezes exactly the next node; prettier/prettier#13008)
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

A suppression comment freezes exactly ONE node, never everything after it; the same contract as in JS, CSS and GraphQL.

## blank-lines

- Why: uniform-rule (blank-line preservation; prettier/prettier#15528)
- Pin: `tests/fixtures/yaml/blank-lines.yaml`, `tests/fixtures/yaml/nested-end-comment-blank.yaml`
- Conformance:
  - `externals/prettier/yaml/mapping/duplicated-keys/flow-mapping.yml` (a blank after a flow mapping, before `---`)

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
This also keeps `proseWrap: never` idempotent where Prettier is not (prettier/prettier#10776),
and covers the blank DOUBLED in front of stream-end comments when the last item carries a trailing comment (the prettier/prettier#9130 shape, resurfaced: one source blank comes out as two).
Same rule in `oxc_formatter` (statements, members) and `oxc_formatter_css` (declarations, rules).

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

- Why: uniform-rule (a group is flat or fully expanded)
- Pin: `tests/fixtures/yaml/flow-multiline-pair.yaml`, `tests/fixtures/yaml/flow-comments.yaml`
- Conformance:
  - `externals/prettier/yaml/spec/spec-example-7-20-single-pair-explicit-entry.yml`
  - `externals/prettier/yaml/spec/spec-example-9-4-explicit-documents.yml`
  - `externals/prettier/yaml/yaml-test-suite/snippet: CT4Q.yaml*` (multiline pairs)
  - `externals/prettier/yaml/yaml-test-suite/snippet: UT92.yaml*` (multiline pairs)
  - `externals/prettier/yaml/yaml-test-suite/snippet: NJ66.yaml*` (multiline pairs)
  - `externals/prettier/yaml/yaml-test-suite/snippet: 9SA2.yaml*` (multiline pairs)
  - `externals/prettier/yaml/flow-mapping/comments/key.yml` (key trailing comment)
  - `externals/prettier/yaml/yaml-test-suite/snippet: K3WX.yaml*` (key trailing comment)

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

A flow collection either fits on one line or breaks normally (trailing comma, bracket on its own line), the core `group` semantics every crate prints (JS arrays and objects, GraphQL argument lists).
Prettier sometimes emits a newline inside flow brackets while keeping them flat (no trailing comma, `]`/`}` on the content line): multiline pairs (spec-example-7-20 / 9-4) and key trailing comments.

## flow-comment-position

- Why: invariant
- Pin: `tests/fixtures/yaml/flow-comments.yaml`
- Conformance:
  - `externals/prettier/yaml/spec/spec-example-6-1-indentation-spaces.yml`
  - `externals/prettier/yaml/yaml-test-suite/snippet: 6HB6.yaml*` (the same example)

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

A comment stays at its syntactic position; Prettier's attachment hoists a comment after `[` onto the `key:` line, across the `[` delimiter (user content; the spec-example-6-1 shape).

## comment-over-indented

- Why: uniform-rule (comment presence never changes layout)
- Pin: `tests/fixtures/yaml/comment-over-indented.yaml`
- Conformance:
  - `externals/aws-cloudformation-templates/RainModules/load-balancer.yml`

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
Same rule as CSS `comment-preceded-map-indent`.

## block-scalar-header-comment-width

- Why: uniform-rule (line_suffix is never measured)
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
Same rule as CSS `trailing-line-comment-print-width`.
The KEY does count: a long key overflowing on `key: |` alone breaks the pair exactly like Prettier.

## document-marker-content

- Why: semantics
- Pin: `tests/fixtures/yaml/document-marker-content/document.yaml`, `tests/fixtures/yaml/document-marker-content/indented.yaml`, `tests/fixtures/yaml/document-marker-content/key.yaml`, `tests/fixtures/yaml/document-marker-content/scalar.yaml`

```yaml
# input
--- ---
? --- x
: m
# (proseWrap: always, printWidth: 10)
aaaa bbbb --- cccc
...
 ---

# ours: no line starts with the marker at column 0
--- ---
? --- x
: m
aaaa bbbb
  --- cccc
...
 ---

# prettier: the content becomes a marker
---
---
--- x: m
aaaa bbbb
--- cccc
...
---
```

Text starting with `---` or `...` never starts a line at column 0, where it is read as a document marker (`"---"` becomes an empty document, the rest fails to parse).
A root plain scalar on a line of its own stays verbatim from its line start: a lenient parser (js-yaml) reads an indented marker as one, so re-indenting it would change the value there.
The guarded sites are listed in AGENTS.md "Line starts".

## bom-content

- Why: semantics
- Pin: `tests/fixtures/yaml/bom-content.yaml`

```yaml
# input ("␢" marks U+FEFF)
? ␢a
: m

# ours: a `---` keeps the key inside a document
---
␢a: m

# prettier: the key leads the stream, read back as a BOM, the key becomes "a"
␢a: m
```

A U+FEFF never starts the text of a document without `---`, where it is read as a BOM: the document gets a `---` (AGENTS.md "Line starts").
