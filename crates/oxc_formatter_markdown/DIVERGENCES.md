# Known divergences

Admission reasons and rules: see `crates/oxc_formatter_core/FORMATTER_POLICY.md` "Known divergences".

## leading-thematic-break

- Why: semantics (prettier/prettier#19839)
- Pin: `tests/fixtures/markdown/thematic-break-first.md`

```markdown
<!-- input -->
---

text

---

<!-- ours -->
***

text

---

<!-- prettier -->
---

text

---
```

A document whose first block is a thematic break gets `***`, never `---`:
Prettier's own next run would read `---` ... `---` as front matter and swallow `text` into it.
Prettier `main` prints `***` since #19839; the pin (3.9.6) still prints `---`.

## url-escaping

- Why: semantics (prettier/prettier#19482, prettier/prettier#19849, prettier/prettier#19891)
- Pin: `tests/fixtures/markdown/link-url-special-chars.md`

```markdown
<!-- input -->
[a](https://x/()foo->bar)

<!-- ours -->
[a](<https://x/()foo-\>bar>)

<!-- prettier -->
[a](<https://x/()foo-%3Ebar>)
```

Destinations follow Prettier `main`: `<`, `>` inside the angle brackets are backslash-escaped (the pin percent-encodes them, which changes the URL),
a `\` that would start an escape is doubled, and `&` that would start a character reference becomes `\&`
(the pin drops the escape and the reference is decoded on the next parse).
Titles get the same `\&` treatment.

## indented-code-tab

- Why: uniform-rule (same construct, same output: space-indented code block)
- Pin: `tests/fixtures/markdown/indented-code-tab.md`

````markdown
<!-- input: two spaces, a tab -->
  	foo

<!-- ours -->
    foo

<!-- prettier -->
```
foo
```
````

An indented code block is printed indented, however its 4 columns were written.
Prettier recognizes indented code from the raw text with a regex (four spaces or a tab),
so an indent written as spaces plus a tab becomes a fenced block.

## ignored-block-trailing-quote-line

- Why: uniform-rule (same construct, same output: the `proseWrap: preserve` layout)
- Pin: `tests/fixtures/markdown/prose-wrap/ignored-block-trailing-quote-line.md`

```markdown
<!-- input -->
> <!-- prettier-ignore -->
> - a   long   item
>

> next

<!-- ours -->
> <!-- prettier-ignore -->
> - a   long   item

> next

<!-- prettier, proseWrap: always -->
> <!-- prettier-ignore -->
> - a   long   item
>

> next
```

A blockquote's trailing blank `>` line is dropped, whatever `proseWrap` is.
Prettier drops it under `preserve` and `never` but prints a bare `>` under `always` when the block before it is `prettier-ignore`d.

## list-indented-code-alignment

- Why: semantics (prettier/prettier#19644, prettier/prettier#19647, prettier/prettier#19990)
- Pin: `tests/fixtures/markdown/tab-width-4/list-indented-code.md`, `tests/fixtures/markdown/tab-width-4/indented-code-block.md`

```markdown
<!-- input, tabWidth 4 -->
- foo

      code

<!-- ours -->
- foo

      code

<!-- prettier -->
- foo

        code
```

An indented code block inside a list item is printed at the item's content column + 4, nothing more:
the code's content is exactly what follows that column, so any extra alignment becomes part of it on the next parse.
The pin aligns it by the task checkbox (continuation lines drift by 4 per run, #19644);
#19647 aligned it by the tab width instead, which shifted the content whenever `tabWidth` exceeded the marker width.
Prettier `main` prints it as we do since #19990 (`indented-code-block.md` is its fixture); the pin (3.9.6) still differs.

## list-marker-after-ignored-list

- Why: semantics
- Pin: `tests/fixtures/markdown/oxfmt-ignore.md`

```markdown
<!-- input -->
<!-- prettier-ignore -->
*   kept   as   written

- formatted

<!-- ours -->
<!-- prettier-ignore -->
*   kept   as   written

- formatted

<!-- prettier -->
<!-- prettier-ignore -->
*   kept   as   written

* formatted
```

Adjacent lists alternate markers so they stay two lists. A list kept verbatim by `prettier-ignore` shows its own marker, so the list after it takes the other one;
Prettier alternates from the marker it would have printed (`-`), prints `*` next to the verbatim `*` list, and the two merge into one list on the next parse.

## single-tilde-strikethrough

- Why: semantics (prettier/prettier#19739)
- Pin: `tests/fixtures/markdown/single-tilde.md`

```markdown
<!-- input -->
H~2~O

<!-- ours -->
H~2~O

<!-- prettier -->
H~~2~~O
```

A single-tilde span is not strikethrough (GitHub strikes `~~x~~` only) and is kept as written.
The pin (3.9.6) parses it as strikethrough and rewrites it to `~~`, turning subscripts into strikes;
Prettier `main` preserves it since #19739.

## liquid-flow-tags

- Why: semantics (prettier/prettier#19724, prettier/prettier#19838)
- Pin: `tests/fixtures/markdown/prose-wrap/liquid-flow.md`

```markdown
<!-- input -->
{% css a %}
{% js b %}

{{ foo

bar   baz }}

<!-- ours -->
{% css a %}
{% js b %}

{{ foo

bar   baz }}

<!-- prettier, proseWrap always -->
{% css a %} {% js b %}

{{ foo

bar baz }}
```

A liquid tag standing on its own line is a flow node: it is kept verbatim, blank lines inside included,
and two tags on adjacent lines stay on their lines.
The pin (3.9.6) reads them as inline tags in a paragraph, so `always` joins them and a tag spanning a blank line falls apart into paragraphs whose whitespace collapses;
Prettier `main` parses them as flow nodes since #19838.

## container-directive

- Why: semantics
- Pin: `tests/fixtures/markdown/prose-wrap/container-directive.md`

```markdown
<!-- input -->
:::warning
Some **text** that is
long.
:::

<!-- ours, proseWrap always -->
:::warning
Some **text** that is long.
:::

<!-- prettier, proseWrap always -->
:::warning Some **text** that is long. :::
```

A `:::` container directive (VitePress, Docusaurus, remark-directive, Pandoc) is a block:
its fence lines are printed as written and only its children are formatted.
The parser opens one on any name-like fence line (`:::name[label]{attrs}`, `::: tip Title`, `::: {.class}`; parser DIVERGENCES.md).

Prettier has no directive construct (#19662 would add micromark's grammar only), so under `always` / `never` the fences are words of the paragraph and wrapping merges them into the text, which breaks the container for every dialect that reads it.

## line-shapes

- Why: semantics
- Pin: `tests/fixtures/markdown/prose-wrap/line-shapes.md`

```markdown
<!-- input -->
> [!NOTE]
> `DOOM`

<!-- ours, proseWrap always -->
> [!NOTE]
> `DOOM`

<!-- prettier, proseWrap always -->
> [!NOTE] `DOOM`
```

A paragraph line starting with `<<<` (VitePress snippet import), a component tag (`<Badge />`, `<my-element>`),
a stray `:::`, or `[!` as the first line of a blockquote (GitHub / Obsidian alert marker) keeps its line boundaries and is never re-wrapped (AGENTS.md "Dialects").
Prettier joins and wraps them like any word: the alert above loses its marker line (GitHub needs `[!NOTE]` alone on it),
a wrapped `<<<` line loses its `[title]`, and a component tag wrapped to column 0 opens an HTML block in MDX / VitePress.
`blockquote/notext-end.md` is the conformance failure.
