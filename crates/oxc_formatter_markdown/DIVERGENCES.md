# Known divergences

Admission reasons and rules: see `crates/oxc_formatter_core/FORMATTER_POLICY.md` "Known divergences".

## leading-thematic-break

- Why: semantics (prettier/prettier#19839)
- Pin: `tests/fixtures/markdown/leading-thematic-break.md`
- Conformance: `markdown/thematicBreak/simple.md`, `markdown/commonmark-test-suite/snippet: example-11.md`, `example-50.md` to `example-54.md`
- Oxfmt: `externals/prettier/markdown/thematicBreak/simple.md`

```markdown
<!-- input -->
---

text

***

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
the next run (Prettier's and ours) would read `---` ... `---` as front matter and swallow `text` into it,
which is what the input itself would be with a `---` at the end (`front-matter.md`).
Prettier `main` prints `***` since #19839; the pin (3.9.9) still prints `---`.

## url-escaping

- Why: semantics (prettier/prettier#19482, prettier/prettier#19849, prettier/prettier#19891)
- Pin: `tests/fixtures/markdown/url-escaping.md`
- Conformance: `markdown/link/encodedLink.md`
- Oxfmt: `externals/prettier/markdown/link/encodedLink.md`

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
- Conformance: `markdown/commonmark-test-suite/snippet: example-2.md`

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

- Why: uniform-rule (same construct, same output: a blockquote's trailing blank `>` line under `proseWrap: preserve`)
- Pin: `tests/fixtures/markdown/prose-wrap/ignored-block-trailing-quote-line.md`
- Oxfmt: `externals/prettier/markdown/blockquote/ignore-code.md`

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

## ordered-marker-before-indented-code

- Why: semantics
- Pin: `tests/fixtures/markdown/tab-width-4/ordered-marker-before-indented-code.md`

```markdown
<!-- input -->
>2.     foo

<!-- ours -->
> 2.     foo

<!-- prettier (the code becomes " foo") -->
> 2.      foo
```

The marker of an ordered list stays unpadded when an item starts with indented code:
the content column is the marker plus one space, so any padding lands inside the code
(the rule indented code in a list item follows, applied to the marker).
Prettier pads it when the code starts at a tab stop in the source, which `>2.     foo` does only because of the `>`.

## list-html-block-alignment

- Why: semantics
- Pin: `tests/fixtures/markdown/tab-width-4/list-html-block-alignment.md`

```markdown
<!-- input -->
- [ ] a

  <textarea>

<!-- ours -->
- [ ] a

  <textarea>

<!-- prettier (the html leaves the list) -->
- [ ] a

<textarea>
```

An HTML block after a task item's paragraph keeps the item's content column, as indented code does.
Prettier drops the alignment when the html starts at another column than the paragraph (the checkbox counts for its printer, not for the parser), which moves the block out of the item.

## html-block-trailing-newline

- Why: invariant
- Pin: `tests/fixtures/markdown/html-block-trailing-newline.md`

```markdown
<!-- input -->
- > <script>
  > a
- b

<!-- ours -->
- > <script>
  > a
- b

<!-- prettier (one more `> ` line per pass) -->
- > <script>
  > a
  >
- b
```

An HTML block closed by its container's end carries that line ending as content (micromark's `trailing_newline`); it is not printed.
Prettier prints it as a blank line of the blockquote, and inside a list item that line is not a fixpoint: every pass adds another.

## lazy-html-block

- Why: semantics
- Pin: `tests/fixtures/markdown/lazy-html-block.md`

```markdown
<!-- input -->
> a
<a>
> c

<!-- ours -->
> a
>
> <a>
> c

<!-- prettier (the html becomes paragraph text) -->
> a
> <a>
> c
```

A complete type 7 tag on a lazy line opens an HTML block inside the container (micromark's rule, oxc-markdown-parser DIVERGENCES.md),
though no type 7 block can interrupt a paragraph on a regular line.
Printed with the container's prefix the line is no longer lazy, so a blank line keeps the block apart;
Prettier prints it adjacent, and the next parse reads it as part of the paragraph.

In a tight list item the line stays lazy, and the block's other lines keep the item's content column:

```markdown
<!-- input, ours -->
- a
<a>
    b

<!-- prettier (the html loses the 2 columns before `b`) -->
- a
<a>
  b
```

## list-after-html-block

- Why: semantics (prettier/prettier#17690)
- Pin: `tests/fixtures/markdown/list-after-html-block.md`

```markdown
<!-- input -->
- a

  <div>

  - b

<!-- ours -->
- a

  <div>

  - b

<!-- prettier (the list item becomes html, one per pass) -->
- a

  <div>
  - b
```

A nested list after a type 6 / 7 HTML block keeps its blank line: only a blank line ends such a block.
Prettier joins any list in a list item to the previous block, and the next parse reads the first list line as part of the html.
Types 1 to 5 end at their end marker, so a list after them is printed adjacent as Prettier does.

## list-marker-after-ignored-list

- Why: semantics
- Pin: `tests/fixtures/markdown/list-marker-after-ignored-list.md`

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

## container-directive

- Why: semantics
- Pin: `tests/fixtures/markdown/prose-wrap/container-directive.md`
- Conformance: `markdown/paragraph/cjk.md`
- Oxfmt: `externals/prettier/markdown/paragraph/cjk.md`

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

An opener interrupts a paragraph (as in micromark's directive extension and markdown-it-container),
so like any block it is printed after a blank line, under `preserve` too; Prettier keeps it on the paragraph's next line.

A fence left open inside a directive closes at the directive's closer (as in micromark's directive extension and markdown-it-container),
so the closing fence is printed explicitly: a reader without directives (CommonMark, GitHub) reads the open fence on to the next closing run, and sees the output's tree change.

## line-shapes

- Why: semantics
- Pin: `tests/fixtures/markdown/prose-wrap/line-shapes.md`
- Conformance: `markdown/blockquote/notext-end.md`
- Oxfmt: `externals/prettier/markdown/blockquote/notext-end.md`

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

## definition-shaped-first-line

- Why: semantics
- Pin: `tests/fixtures/markdown/prose-wrap/definition-shaped-first-line.md`

```markdown
<!-- input -->
[core]: https://example.com/a/very/long/path/that/fills/most/xxxxxxxxxxxxxxxxxxxxxxxx then more

<!-- ours, proseWrap always -->
[core]: https://example.com/a/very/long/path/that/fills/most/xxxxxxxxxxxxxxxxxxxxxxxx then more

<!-- prettier, proseWrap always (a definition and a paragraph) -->
[core]:
https://example.com/a/very/long/path/that/fills/most/xxxxxxxxxxxxxxxxxxxxxxxx
then more
```

A paragraph's first line shaped like a definition (`[label]: dest text`) is a paragraph only because of what follows the destination on the line,
so it keeps its line boundaries and is never re-wrapped, under every `proseWrap`.
Prettier wraps it like any line: a break right after the destination makes it a definition on the next parse.

## wrapped-block-starts

- Why: semantics (prettier/prettier#13634, prettier/prettier#19112, prettier/prettier#19847)
- Pin: `tests/fixtures/markdown/prose-wrap/wrapped-block-starts.md`
- Oxfmt: `externals/prettier/markdown/list/parser-regression/issue-17778.md`

```markdown
<!-- input, proseWrap always -->
Some words that push the tag to the edge of the line so the html block <div>x</div> lands first.

<!-- ours -->
Some words that push the tag to the edge of the line so the html block <div>x</div>
lands first.

<!-- prettier -->
Some words that push the tag to the edge of the line so the html block
<div>x</div> lands first.
```

A line break is never inserted before a word that would open a block at a line start.
The parser's `lexical::line_start` decides, asked about the line the wrap would produce
(`- x` interrupts a paragraph, an empty `-` does not; `*** x` is text, `***` alone is a break).
Prettier tests a regex of list markers, `#`s and `>` only, so a wrapped `<div>`, `<!--`, `***`, `---`, fence,
`|` row or `[^x]:` opens a block on the next parse.

A kept line break (`proseWrap: preserve`) is dropped by the same rule when its next line only stays text by its indentation,
which a paragraph does not keep: `| x | y |` over an 8-space `|---|---|` prints as one line,
Prettier's `|---|---|` at column 0 is a table on the next parse (prettier/prettier#19847).
A delimiter row only counts under a header with as many cells (`lexical::table_delimiter_activates`, asked about the line above):
under another one it opens nothing and the kept break before it stays.
A break that cannot be dropped (a hard break, or a line shape on either side of it) keeps the line as text behind four spaces instead:
indented code cannot interrupt a paragraph (the pin's math span after a hard break).
Prettier drops the break before `- item`, `# heading` and `> quote` itself, and before `1000000000.` too,
which its regex takes for a list marker (a marker has at most 9 digits).

## stray-delimiters

- Why: semantics (prettier/prettier#6035, prettier/prettier#17303, prettier/prettier#17353)
- Pin: `tests/fixtures/markdown/prose-wrap/stray-delimiters.md`
- Conformance: `markdown/commonmark-test-suite/snippet: example-412.md`

```markdown
<!-- input -->
A stray ` backtick and then ``code``.

<!-- ours -->
A stray ` backtick and then ``code``.

<!-- prettier -->
A stray ` backtick and then `code`.
```

A code span keeps a longer fence when a literal backtick run of the shorter length precedes it;
Prettier picks the shortest fence from the content alone, so its output pairs with the stray run and moves the span.

Emphasis markers are normalized (`__` to `**`, `_` / `*` by their neighbors) only when the printed text
still forms the same nodes: the parser's own resolver (`oxc_markdown_parser::attention::pairs`) is run on
the runs of the text about to be printed, and on a mismatch the paragraph is printed with its markers
as written (`**a ***b*** c**` stays: `**a _**b**_ c**` reads differently, prettier/prettier#20048).
Prettier always normalizes.

A `*` / `_` run inside emphasis that could pair is escaped character by character
(`\*\*\*`; Prettier's single backslash leaves `**`, which still pairs: `example-412.md`).
Found by `tests/invariants.rs`, the fuzzed `parse(format(x)) ≅ parse(x)` check.

## verbatim-inline-continuation

- Why: semantics (prettier/prettier#19116)
- Pin: `tests/fixtures/markdown/verbatim-inline-continuation.md`

```markdown
<!-- input -->
- [ ] a `x
      y` b

- [ ] c <!-- one
      two -->

<!-- ours -->
- [ ] a `x
      y` b

- [ ] c <!-- one
      two -->

<!-- prettier -->
- [ ] a `x
    y` b

- [ ] c <!-- one
          two -->
```

A continuation line of a verbatim inline node (code span, inline HTML, math span, image alt, link title)
resumes at the container's content column, the only indentation the parser strips;
whatever whitespace follows is the node's content and is printed as is.
Prettier prints code span lines from column 0 and the others from its own alignment (a task item's checkbox counts there, not for the parser),
so the content of `` `x y` `` loses two spaces per pass and an HTML comment gains four.
With no whitespace past the container's column the same column-0 line is a lazy continuation and reads the same,
so there the difference is layout only (most real-world cases).

## wiki-link-code-span

- Why: semantics
- Pin: `tests/fixtures/markdown/prose-wrap/wiki-link-code-span.md`

```markdown
<!-- input, proseWrap always -->
See [[the `first
second` note]] for details.

<!-- ours -->
See [[the `first
second` note]] for details.

<!-- prettier -->
See [[the `first second` note]] for details.
```

In a paragraph printed as written for wiki link risk (Prettier's `riskyParagraphPositions`, `[[` ... `]]`),
a code span keeps its line break too; Prettier joins it, and the one-line `[[...]]` is a wiki link on the next parse.

## leading-dashes

- Why: semantics
- Pin: `tests/fixtures/markdown/leading-dashes-paragraph.md`

```markdown
<!-- input -->

---:::a
text
---[x]: /u

<!-- ours -->
\---:::a
text
---[x]: /u

<!-- prettier -->
---:::a
text
---[x]: /u
```

A document whose first block is a paragraph (or setext heading) starting with `---` / `+++` gets that
delimiter escaped when a later line could close it as front matter (a line starting with the delimiter,
or a thematic break, which prints `---`): the next parse (Prettier's and ours) would read the block as front matter.
Prettier prints it as written (the leading blank line that kept it out of front matter is dropped).

## cj-line-break

- Why: semantics (prettier/prettier#20143)
- Pin: `tests/fixtures/markdown/prose-wrap/cj-line-break.md`
- Conformance: `markdown/splitCjkText/symbolSpaceNewLine.md`
- Oxfmt: `externals/prettier/markdown/splitCjkText/symbolSpaceNewLine.md`

```markdown
<!-- input, proseWrap always -->
日本語の文章は
改行しても
そのままです。

見て http://x.y2.
。次

<!-- ours -->
日本語の文章は
改行しても
そのままです。

見て http://x.y2.
。次

<!-- prettier -->
日本語の文章は改行してもそのままです。

見て http://x.y2.。次
```

A line break next to Chinese / Japanese text is kept under every `proseWrap`,
except between a Korean and a CJ letter, where it is a space.
Browsers disagree on such a break (Firefox drops it, Chrome and Safari render a space),
so removing it or making it a space changes the rendered text in some of them.
Joined, an autolink literal also takes the CJ punctuation after it (`http://x.y2.。次`).
Prettier `main` keeps the break since #20143; the pin (3.9.9) removes it or makes it a space.

## wiki-link-risk-link-text

- Why: semantics
- Pin: `tests/fixtures/markdown/prose-wrap/wiki-link-risk-link-text.md`

```markdown
<!-- input -->
[[a], b]] [*y*-b](#c)

<!-- ours -->
[[a], b]] [_y_-b](#c)

<!-- prettier -->
[[a], b]] [_y_

-b](#c)
```

In a paragraph printed as written for wiki link risk (`[[` ... `]]`), Prettier splits an inline link's text
after an emphasis followed by `-` into two paragraphs, which breaks the link (d3's READMEs).
Ours keeps the line.

## setext-heading-wrap

- Why: uniform-rule (same construct, same output: a paragraph)
- Pin: `tests/fixtures/markdown/prose-wrap/setext-heading-wrap.md`
- Oxfmt: `externals/prettier/markdown/heading/setext/issue-6013-2.md`

```markdown
<!-- input, proseWrap always -->
Some heading words that run on and then [a link](https://example.com/a/rather/long/path/to/somewhere) end
===

<!-- ours -->
Some heading words that run on and then
[a link](https://example.com/a/rather/long/path/to/somewhere) end
===

<!-- prettier -->
Some heading words that run on and then [a link](https://example.com/a/rather/long/path/to/somewhere)
end
===
```

A setext heading's content wraps like a paragraph's, which Prettier breaks before the link.
Prettier makes setext headings breakable but prints their children without the paragraph's `flattenFill`:
each sentence is its own fill, so the whitespace before a link, emphasis or code span ends a fill and never breaks.

## nul-character

- Why: semantics
- Pin: `tests/fixtures/markdown/nul-character.md`

```markdown
<!-- input (␀ is U+0000) -->
`c␀d`

<!-- ours -->
`c␀d`

<!-- prettier -->
`c�d`
```

A NUL stays as written, in code spans and code blocks too.
CommonMark replaces it with U+FFFD for security when rendering;
Prettier prints the decoded value of code, so the replacement lands in the source, while plain text keeps its NUL (a raw slice).
Ours prints every node as written, the next parse replaces it again.

## fence-info-backtick

- Why: semantics
- Pin: `tests/fixtures/markdown/fence-info-backtick.md`

````markdown
<!-- input -->
~~~js `x`
a
~~~

<!-- ours -->
~~~js `x`
a
~~~

<!-- prettier -->
```js `x`
a
```
````

A fenced code block whose info string has a backtick keeps its `~~~` fence; any other fence is printed with backticks.
A backtick fence cannot have a backtick in its info string (CommonMark),
so Prettier's opener is a paragraph on the next parse and its closer opens a code block that runs to the end of the document.

## footnote-kept-line-break

- Why: semantics
- Pin: `tests/fixtures/markdown/prose-wrap/footnote-kept-line-break.md`

```markdown
<!-- input -->
[^1]: a b\
=

<!-- ours (every proseWrap) -->
[^1]:
    a b\
        =

<!-- prettier, preserve (a setext heading on the next parse) -->
[^1]:
    a b\
    =

<!-- prettier, never -->
[^1]: a b\
=
```

A footnote's single paragraph follows the marker only when it prints as one line, under every `proseWrap`:
`always` also breaks it for the width, the other modes only for a line break the paragraph keeps.
Prettier asks `preserve` whether the source is one line, and its block form above moves the lazy `=` into the footnote,
where it underlines the paragraph.
Its `never` keeps the marker line whatever the paragraph holds (admissible here); ours follows the same rule as the other modes.
