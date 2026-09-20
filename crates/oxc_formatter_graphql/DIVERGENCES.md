# Known divergences

Admission reasons and rules: see `crates/oxc_formatter_core/FORMATTER_POLICY.md` "Known divergences".

## comment-after-keyword

- Why: invariant
- Pin: `tests/fixtures/graphql/comments-inside-node-spans.graphql`

```graphql
# input
"d" type # c
A {
  f: String
}

# ours
"d"
type # c
A {
  f: String
}

# prettier
"d" # c
type A {
  f: String
}
```

Prettier's attachment pulls the comment backwards across the `type` keyword (user content) onto the description's line.

## own-line-comment-after-description

- Why: invariant
- Pin: `tests/fixtures/graphql/comments-inside-node-spans.graphql`

```graphql
# input
"d"
# c
type A {
  f: String
}

# ours
"d"
# c
type A {
  f: String
}

# prettier
"d"
type # c
A {
  f: String
}
```

Prettier's attachment pushes the comment forward across the `type` keyword (user content), own-line to same-line.

## fields-block-opener-comment

- Why: uniform-rule (same construct, same output: a selection set's `{ # c`)
- Pin: `tests/fixtures/graphql/comment-after-opening-delimiter.graphql`

```graphql
# input
type B implements I @d { # c
  f: Int
}

# ours
type B implements I @d {
  # c
  f: Int
}

# prettier
type B implements I
  @d { # c
  f: Int
}
```

A comment right after an opening `{` leads the first item, as in a selection set or `schema` block (Prettier agrees there: no node precedes the comment inside the parent).
In a `type` / `interface` / `input` / `enum` / `extend` block the comment follows the name, an interface or a directive,
so Prettier's attachment makes it that node's trailing comment and `lineSuffix` carries it to the line end past `{`;
the `breakParent` it adds pushes a preceding directive onto its own line.

## argument-list-opener-comment

- Why: invariant
- Pin: `tests/fixtures/graphql/comment-after-opening-delimiter.graphql`

```graphql
# input
query T { a( # c
  x: 1) }

# ours
query T {
  a(
    # c
    x: 1
  )
}

# prettier
query T {
  a(x: 1) # c
}
```

Prettier's attachment makes the comment a trailing of the field name and `lineSuffix` carries it past `(x: 1)` (user content crossed).
Same class: `query R( # c` -> `query R($a: Int) { # c`, `directive @d( # c` -> `directive @d(a: Int) on FIELD # c`.

## trailing-comment-before-continuation

- Why: invariant
- Pin: `tests/fixtures/graphql/comments-inside-node-spans.graphql`

```graphql
# input
type A # c
implements B {
  f: String
}

# ours
type A # c
implements B {
  f: String
}

# prettier
type A implements B { # c
  f: String
}
```

Prettier's attachment scatters the comment to the line end, across `implements B {`; same class: `f(x) # c` + break + `: T` is pulled inside the parens (`x # c` before the `)`), user content crossed in both.
