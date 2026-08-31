# Known divergences

Admission reasons and rules: see `crates/oxc_formatter_core/FORMATTER_POLICY.md` "Known divergences".

## comment-after-keyword

- Why: prettier-bug (attachment artifact)
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

Prettier pulls the comment backwards across the keyword onto the description's line.

## own-line-comment-after-description

- Why: prettier-bug (attachment artifact)
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

Prettier pushes the comment forward across the keyword.

## trailing-comment-before-continuation

- Why: prettier-bug (attachment artifact)
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

Prettier scatters the comment to the line end; same class: `f(x) # c` + break + `: T` is pulled inside the parens (`x # c` before the `)`).

## comment-after-opening-delimiter

- Why: prettier-bug (attachment artifact)
- Pin: `tests/fixtures/graphql/comment-after-opening-delimiter.graphql`

```graphql
# input
{ # c
  test # t
} # e

# ours
{ # c
  test # t
} # e

# prettier
{
  # c
  test # t
} # e
```

Prettier moves the comment own-line as the first child's leading; asymmetric, `test # t` / `} # e` stay inline in both.
