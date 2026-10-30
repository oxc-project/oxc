# Known divergences

Admission reasons and rules: see `crates/oxc_formatter_core/FORMATTER_POLICY.md` "Known divergences".

## whitespace-only-pre

- Why: semantics (whitespace in `pre` / `textarea` is content)
- Pin: `tests/fixtures/html/whitespace-only-pre.html`

```html
<!-- input -->
<pre> </pre>
<textarea>  </textarea>

<!-- ours -->
<pre> </pre>
<textarea>  </textarea>

<!-- prettier -->
<pre></pre>
<textarea> </textarea>
```

A whitespace-only `<pre>` or `<textarea>` keeps its content.
Prettier drops a lone whitespace-only text in `pre` (as in every element) and collapses one in `textarea` to a single space,
which changes what `pre` renders and what `textarea` submits.

## doctype-case

- Why: uniform-rule (same construct, same output: the doctype of a `.html` file, which Prettier lowercases)
- Pin: `tests/fixtures/vue/doctype-case.vue`

```vue
<!-- input -->
<!DOCTYPE html>
<html><body><p>{{ a }}</p></body></html>

<!-- ours -->
<!doctype html>
<html>
  <body>
    <p>{{ a }}</p>
  </body>
</html>

<!-- prettier -->
<!DOCTYPE html>
<html>
  <body>
    <p>{{ a }}</p>
  </body>
</html>
```

`<!DOCTYPE html>` prints `<!doctype html>` in every HTML document but XHTML (`.xhtml`, `.xht`), where an XML parser needs `<!DOCTYPE`:
a Vue page, html-in-js and an HTML code block too, and `.hta` / `.inc` files in oxfmt.
Prettier lowercases it in a file named `.html` / `.htm` only, as it decides by file name.
The `html` in any doctype prints lowercase (`<!DOCTYPE html PUBLIC …>`), as in Prettier.

## doctype-html-in-one-pass

- Why: invariant
- Pin: `tests/fixtures/html/doctype-html-in-one-pass.html`

```html
<!-- input -->
<!DOCTYPE HTML>

<!-- ours, and Prettier's second pass -->
<!doctype html>

<!-- prettier -->
<!DOCTYPE html>
```

Prettier's output is not a fixpoint: it lowercases the `HTML` and the `<!DOCTYPE` in different passes.
`<!DOCTYPE HTML>` prints `<!doctype html>` in one.

## cdata-as-written

- Why: uniform-rule (a section's content prints as written, as a comment's does: in HTML content it is a bogus comment to browsers; prettier/prettier#20256, prettier/prettier#16147)
- Pin: `tests/fixtures/html/cdata-as-written.html`

```html
<!-- input (printWidth: 20) -->
<p>text <![CDATA[aaaa bbbb cccc dddd eeee]]> more</p>

<!-- ours -->
<p>
  text
  <![CDATA[aaaa bbbb cccc dddd eeee]]>
  more
</p>

<!-- prettier -->
<p>
  text <![CDATA[aaaa
  bbbb cccc dddd
  eeee]]> more
</p>
```

A CDATA section is one unbreakable word, together with any text touching it; the text around it wraps as usual.
Prettier merges it into the surrounding text and re-wraps its words, changing what it holds:
in HTML the section is comment data, and in SVG / MathML its text, where a re-wrap only moves collapsible whitespace.
Prettier `main` keeps it as written (prettier/prettier#20256), unreleased at the pin.

## class-ascii-whitespace

- Why: semantics (class names are separated by ASCII whitespace only)
- Pin: `tests/fixtures/html/class-ascii-whitespace.html`

```html
<!-- input (the space in `a b` is U+00A0) -->
<div class="a b  c"></div>

<!-- ours -->
<div class="a b c"></div>

<!-- prettier: `a` and `b` are now two classes -->
<div class="a b c"></div>
```

A class list collapses and trims ASCII whitespace; a no-break space stays part of a class name.
Prettier collapses and trims with JavaScript's `\s`, which splits `a b` written with a literal U+00A0 (one class) into two classes.

## end-tag-indentation-as-printed

- Why: uniform-rule (same construct, same output: Prettier's output for the same layout indented with spaces; prettier/prettier#17048)
- Pin: `tests/fixtures/html/end-tag-indentation-as-printed.html`, `tests/fixtures/html/use-tabs/end-tag-indentation-as-printed.html`

```html
<!-- input (useTabs) -->
<div>
	<div>
		<!--
		comment
		-->
	</div>
</div>

<!-- ours -->
<div>
	<div>
		<!--
		comment
		-->
	</div>
</div>

<!-- prettier -->
<div>
	<div>
		<!--
		comment
		--></div>
</div>
```

When the last child is a comment or `pre` text ending in a line break plus the element's indentation, the end tag follows it on the same line.
That indentation is counted in the printer's indent unit: a tab per level with `useTabs`, `indentWidth` spaces otherwise.
Prettier counts `tabWidth` × depth characters, tabs or spaces alike.
With `useTabs`, a comment closing one level deeper than its element then counts as being at the element's indentation and the end tag joins it,
while the same layout written with spaces keeps the end tag on its own line.
Without `useTabs`, a tab-indented tail of that many characters joins likewise.

## srcset-zero-density

- Why: semantics (`0x` is a density the HTML srcset parser accepts; without it the candidate defaults to `1x`)
- Pin: `tests/fixtures/html/srcset-zero-density.html`

```html
<!-- input -->
<img srcset="a.png 0x, b.png 2x">

<!-- ours -->
<img srcset="a.png 0x, b.png 2x" />

<!-- prettier -->
<img srcset="a.png, b.png 2x" />
```

A `0x` (or `-0x`) descriptor is kept, printed as `0x`.
Authors must write a density above zero, but the HTML srcset parser reads `0x` as density 0 (infinite natural dimensions), not as the default `1x`.
Prettier's srcset parser treats a zero density as absent and drops it.

## script-display-none

- Why: semantics (whitespace added on both sides of a `script` renders between its inline neighbors)
- Pin: `tests/fixtures/html/script-display-none.html`
- Conformance: `html/tags/tags.html`

```html
<!-- input -->
<p><span>a</span><script src="a.js"></script><span>b</span></p>

<!-- ours -->
<p>
  <span>a</span
  ><script src="a.js"></script
  ><span>b</span>
</p>

<!-- prettier -->
<p>
  <span>a</span>
  <script src="a.js"></script>
  <span>b</span>
</p>
```

```html
<!-- input -->
<div>Price: <span>10</span> <meta itemprop="priceCurrency" content="USD">USD, and some more text that wraps the line</div>

<!-- ours -->
<div>
  Price: <span>10</span> <meta itemprop="priceCurrency" content="USD" /> USD,
  and some more text that wraps the line
</div>

<!-- prettier -->
<div>
  Price: <span>10</span> <meta itemprop="priceCurrency" content="USD" />USD, and
  some more text that wraps the line
</div>
```

An element with the UA stylesheet's `display: none` (`script`, `style`, `link`, `meta`, `title`, ...) renders nothing:
the whitespace on both its sides is one run between the content around it, judged as if the element were not there.
- Between inline content, a run without whitespace stays glued (`>` borrowed);
  one with whitespace may break at any of its gaps, as it renders one space either way.
- Next to a block or a parent's edge the run renders nothing and breaks freely,
  though a gap between two non-blocks stays on the line where it fits (`<p>x<meta /></p>`).

Prettier lays `script` out as a block and puts it on its own line, which turns `a` and `b` glued around it into `a b`:
it did so for glued scripts between blocks (`</script><script>` borrowing `>`, prettier/prettier#6393, prettier/prettier#6423),
which passing hidden elements prints the same way without changing what renders.
It keeps the whitespace around the other hidden elements as written,
so a glued one borrows `>` even where nothing renders, and a spaced run breaks only at the space.
A `script` / `style` body renders nothing, so its end tag takes its own line even where the next sibling borrows its `>`.
Prettier's other block overrides for boxless elements (`param`, `source`, `track`) are kept:
they sit inside `object` / `video` / `audio`, whose children render only as fallback content.

## svg-text-content-inline

- Why: semantics (whitespace added between a `<text>`'s children renders: `H<tspan>2</tspan>O` reads "H 2 O")
- Pin: `tests/fixtures/html/svg-text-content-inline.html`

```html
<!-- input -->
<svg><text>H<tspan baseline-shift="sub">2</tspan>O</text></svg>

<!-- ours -->
<svg>
  <text>H<tspan baseline-shift="sub">2</tspan>O</text>
</svg>

<!-- prettier -->
<svg>
  <text>
    H
    <tspan baseline-shift="sub">2</tspan>
    O
  </text>
</svg>
```

An SVG `<text>` lays out its content as a line of text, so its descendants (`tspan`, `textPath`, `a`, ...) are inline:
whitespace between them is kept or omitted as written, as between HTML inline elements.
`<text>` itself stays a block, whose leading and trailing whitespace collapses away.
Prettier lays out every SVG element but `<svg>` as a block (prettier/prettier#17713, prettier/prettier#10121).
With `htmlWhitespaceSensitivity: "ignore"` they stay blocks, as HTML inline elements do.

## dialog-out-of-flow

- Why: semantics (whitespace added on both sides of a `dialog` renders between its inline neighbors)
- Pin: `tests/fixtures/html/dialog-out-of-flow.html`
- Conformance: `html/tags/tags2.html`

```html
<!-- input -->
<main><b>a</b><dialog></dialog><b>b</b></main>

<!-- ours -->
<main><b>a</b><dialog></dialog><b>b</b></main>

<!-- prettier -->
<main>
  <b>a</b>
  <dialog></dialog>
  <b>b</b>
</main>
```

A `dialog` takes no room in its line: the UA stylesheet hides it while closed (`dialog:not([open])`)
and positions it absolutely while open, so whitespace around it is kept or omitted as written, as around a `script` (DIVERGENCES.md#script-display-none).
Its content has a box of its own, so it lays out as an inline-block's: line breaks inside its tags are free.
Prettier reads `dialog` as a block from the UA stylesheet's `display: block` and skips the `:not([open])` rule.
Between blocks, where it usually sits, the output is the same.

## nul-as-written

- Why: semantics (Prettier drops everything after a NUL between tags)
- Pin: `tests/fixtures/html/nul/nul-as-written.html`

```html
<!-- input (`␀` is U+0000) -->
<p   a>b</p>
<p>c␀d</p>
<p   >e</p>

<!-- ours -->
<p a>b</p>
<p>c␀d</p>
<p>e</p>

<!-- prettier -->
<p a>b</p>
<p>c</p>
```

A NUL prints as written, as in every oxc formatter (CSS, Markdown, YAML and JS strings keep it too).
HTML makes it a parse error that browsers ignore in text (or replace with U+FFFD elsewhere), so the output reads as the input does.

## pre-after-borrowed-closing-end

- Why: semantics (Prettier prints the borrowed `>` twice; the second is text; prettier/prettier#16184)
- Pin: `tests/fixtures/html/strict/pre-after-borrowed-closing-end.html`

```html
<!-- input (htmlWhitespaceSensitivity: strict) -->
<b>a</b><pre><b>x</b></pre>

<!-- ours -->
<b>a</b><pre><b>x</b></pre>

<!-- prettier -->
<b>a</b>><pre><b>x</b></pre>
```

A `pre` holding an element (printed as written) that borrows its previous sibling's `>` prints it once.
Prettier prints it again, adding a `>` to the rendered text.
Prettier `main` prints it once (prettier/prettier#20215), unreleased at the pin.

## ignored-closing-end-once

- Why: semantics (Prettier prints the borrowed `>` twice; the second is text)
- Pin: `tests/fixtures/html/ignored-closing-end-once.html`

```html
<!-- input -->
<time><!-- prettier-ignore -->
<mark>
<input />
</mark></time>

<!-- ours -->
<time
  ><!-- prettier-ignore -->
  <mark>
<input />
</mark
></time>

<!-- prettier: a `>` text after </mark> -->
<time
  ><!-- prettier-ignore -->
  <mark>
<input />
</mark>
></time>
```

A `prettier-ignore`d last child whose closing `>` its parent borrows prints its source without that `>`.
Prettier strips it only when the next sibling borrows it, so the parent prints it a second time.

## line-break-before-borrowed-end

- Why: invariant
- Pin: `tests/fixtures/html/line-break-before-borrowed-end.html`, `tests/fixtures/html/strict/line-break-before-borrowed-end.html`

```html
<!-- input -->
<div><progress><progress>x</progress><wbr />
amet,

</progress><abbr></abbr></div>

<!-- ours, and Prettier's second pass -->
<div>
  <progress><progress>x</progress><wbr /> amet,</progress><abbr></abbr>
</div>

<!-- prettier -->
<div>
  <progress>
    <progress>x</progress><wbr />
    amet,</progress
  ><abbr></abbr>
</div>
```

Prettier's output is not a fixpoint: its second pass joins the lines the first broke.
A line break after a last child, before an end tag whose `>` is borrowed, does not count as one
(for an only child on its own line, or a hard line before a node with line breaks around it).
No printer can keep it there: the end tag follows on the same line, after a space where whitespace matters and right after the child otherwise.
Prettier counts it from the source, then its next pass no longer sees it and joins the lines.
Where that layout stays for other reasons, Prettier's first pass is stable and differs from ours.

## text-display-width

- Why: uniform-rule (text width is the display width; `oxc_formatter_core`'s FORMATTER_POLICY.md "Text width"; prettier/prettier#20081)
- Pin: `tests/fixtures/html/text-display-width.html`

```html
<!-- input (print width 80; `{60 × U+00A0 U+200C}` stands for the characters) -->
<div>{60 × U+00A0 U+200C}</div>

<!-- ours -->
<div>{60 × U+00A0 U+200C}</div>

<!-- prettier -->
<div>
  {60 × U+00A0 U+200C}
</div>
```

Zero-width characters count as 0 toward the print width, so the text fits next to its tags.
Prettier counts the format characters (`U+200B..U+200F`, `U+FEFF`) as 1 and breaks.
An email template pads its preview text this way, NBSP alternating with zero-width characters.

## unclosed-at-end

- Why: semantics (an end tag added where a fragment ends changes the page it is joined into; prettier/prettier#17761, prettier/prettier#5665)
- Pin: `tests/fixtures/html/omitted-end-tag-at-eof.html`
- Conformance: `html/basics/broken-html.html`, `html/prettier_ignore/unclosed2.html`

```html
<!-- input (header.html, closed by footer.html) -->
<html>
<body>
<div class="wrap">
<header>Site</header>

<!-- ours -->
<html>
  <body>
    <div class="wrap">
      <header>Site</header>

<!-- prettier -->
<html>
  <body>
    <div class="wrap">
      <header>Site</header>
    </div>
  </body>
</html>
```

An element left open at the end gets no end tag: it is a fragment that something else closes,
html-in-js joined with other templates (Prettier turns `` html`<div>` `` into `` html`<div></div>` ``),
or a partial `.html` file included before its closing counterpart (a header for a footer).
Prettier closes it where the fragment ends, which changes the page it is joined into.

## omitted-end-tag

- Why: uniform-rule (a normalization is applied only where it is known safe for every reader: `oxc_formatter_css`'s AGENTS.md "Case")
- Pin: `tests/fixtures/html/omitted-end-tag.html`

```html
<!-- input -->
<ul>
  <li>one
  <li>two
</ul>

<!-- ours -->
<ul>
  <li>one
  <li>two
</ul>

<!-- prettier -->
<ul>
  <li>one</li>
  <li>two</li>
</ul>
```

An end tag the source omits where HTML closes the element (`</li>`, `</p>`) is not added.
To a browser reading the page the end tag changes nothing, but in a template the place HTML closes it may sit in a branch:

```html
{% if x %}<p>a{% else %}<p>b{% endif %}<div>
<!-- an end tag added before the second <p> lands in the else branch: a stray </p>, which makes an empty p -->
```

## pre-content-without-end-tag

- Why: semantics (whitespace in `pre`-like elements is content; prettier/prettier#16205)
- Pin: `tests/fixtures/html/pre-content-without-end-tag.html`

```html
<!-- input -->
<div><pre />
<b>x</b></div>
<pre><b>x</b> unclosed

<!-- ours -->
<div>
  <pre />
<b>x</b></div>
<pre><b>x</b> unclosed

<!-- prettier -->
<div>
  <pre />
  <b>x</b>
</div>
<pre>>
```

A `pre`, `listing`, `plaintext` or `xmp` holding markup without its end tag keeps its content as written, up to where HTML closes it.
`<pre />` reads as one: HTML ignores the `/` and the content up to an ancestor's end tag is inside it.
Prettier reads `<pre />` as self-closing and re-indents that content, adding whitespace to what `pre` renders;
a `pre` left open at the end loses its content to a stray `>`.

## tag-name-case

- Why: semantics (consumers other than an HTML parser tell names apart by case; prettier/prettier#16898)
- Pin: `tests/fixtures/html/tag-name-case.xhtml`

```html
<!-- input (an .xhtml file) -->
<DIV CLASS="a"><P>Text</P></DIV>

<!-- ours -->
<DIV CLASS="a"><P>Text</P></DIV>

<!-- prettier -->
<div class="a"><p>Text</p></div>
```

```js
// input
html`<A someThing="bla"></A>`;

// ours
html`<A someThing="bla"></A>`;

// prettier
html`<a someThing="bla"></a>`;
```

Tag and attribute names keep their case where a reader other than an HTML parser may read them: embedded HTML and XHTML.
In a `.html` file, whose reader is an HTML parser, the names Prettier knows print lowercase as it does (`src/tree/names.rs`).
Prettier lowercases them in embedded HTML and XHTML too, by the language alone:
- a framework's `html` tagged template is read by its own runtime, which tells `<A>` (a router component) from the element `<a>`;
- a code block labelled `html` often holds a template of another language (a Vue `<Button>`);
- an XML parser reads XHTML's names case-sensitively.

Vue and Angular keep every name as written, as Prettier does.

## prefixed-script-style

- Why: uniform-rule (same construct, same output: `<html:p>` and every other prefixed name, which Prettier lays out as an unknown element)
- Pin: `tests/fixtures/html/prefixed-script-style.html`
- Conformance: `html/basics/with-colon.html`

```html
<!-- input -->
<div><html:style>.a{color:red}</html:style></div>
<div>x<html:p>p</html:p>y</div>

<!-- ours -->
<div><html:style>.a{color:red}</html:style></div>
<div>x<html:p>p</html:p>y</div>

<!-- prettier -->
<div>
  <html:style
    >.a{color:red}</html:style
  >
</div>
<div>x<html:p>p</html:p>y</div>
```

In HTML, an element whose name has a `prefix:` is an unknown element, as browsers read it (the parser's DIVERGENCES.md, "Namespace prefixes"):
`<html:script>` and `<html:style>` too, whose content is text.
Prettier lays out every prefixed name so, but sees `script` / `style` through the prefix and puts them on their own lines.
A `prefix:` comes from XML namespaces; only an XML parser (XHTML, XUL, a standalone SVG) reads `html:script` as a script.
Angular's compiler reads a prefix as a namespace (`svg:rect`), and so does the Angular dialect.

## unquoted-value-with-slash

- Why: semantics (Prettier ends an unquoted value at `/`: `src=a.png/` becomes `src="a.png"`, another URL)
- Pin: `tests/fixtures/html/unquoted-value-with-slash.html`

```html
<!-- input -->
<img src=a.png/>

<!-- ours -->
<img src="a.png/" />

<!-- prettier -->
<img src="a.png" />
```

```html
<!-- input -->
<a href=https://www.mozilla.org/>favorite website</a>

<!-- ours -->
<a href="https://www.mozilla.org/">favorite website</a>

<!-- prettier: error, Opening tag "a" not terminated -->
```

In HTML, an unquoted attribute value runs to whitespace or `>`, as browsers read it (the parser's DIVERGENCES.md, "Unquoted attribute values").
Prettier's parser also ends it at `/`: a value ending in `/` loses it and the tag self-closes,
and a `/` inside the value (a URL) leaves the rest of the tag unreadable, a parse error
(in a Markdown code block, the block is left as written).

## htm-closing-tag-as-written

- Why: uniform-rule (same construct, same output: an htm end tag Prettier prints as written outside a merged text)
- Pin: `tests/fixtures/html/htm-closing-tag-as-written.html`

```html
<!-- input -->
<span>a<b>x<//>c</span>

<!-- ours -->
<span>a<b>x<//>c</span>

<!-- prettier -->
<span>a<b>x</b>c</span>
```

htm's `<//>`, an end tag closing the innermost element, prints as written.
Prettier prints it as written too, except for an element it merges into the text around it, whose end tag it rebuilds from the element's name.

## end-tag-with-space-is-comment

- Why: semantics (HTML reads `</ div>` as a comment, so the element stays open)
- Pin: `tests/fixtures/html/end-tag-with-space-is-comment.html`

```html
<!-- input -->
<div>a</ div>b

<!-- ours -->
<div>a</ div>b

<!-- prettier -->
<div>a</div>
b
```

In HTML, `</` without a name right after it is a bogus comment, as in browsers, and prints as written.
Vue's and Angular's compilers read `</ div>` as an end tag, and so do those dialects.
Prettier reads it as an end tag and prints `</div>`, which closes the `div` there and moves what follows out of it.

## vue-element-gap-kind

- Why: semantics (Vue's `whitespace: 'condense'` drops whitespace with a line break between two elements and keeps a same-line one as a space)
- Pin: `tests/fixtures/vue/vue-element-gap-kind.vue`

```vue
<!-- input -->
<template>
  <ul>
    <li>
      <router-link to="/using-store">/using-store</router-link> — <b>universal</b>
      <code>reactive()</code> stores.
    </li>
  </ul>
  <p><span>a long enough first element here</span> <span>and a second one that does not fit</span></p>
</template>

<!-- ours -->
<template>
  <ul>
    <li>
      <router-link to="/using-store">/using-store</router-link> —
      <b>universal</b>
      <code>reactive()</code> stores.
    </li>
  </ul>
  <p>
    <span>a long enough first element here</span> <span
      >and a second one that does not fit</span
    >
  </p>
</template>

<!-- prettier -->
<template>
  <ul>
    <li>
      <router-link to="/using-store">/using-store</router-link> —
      <b>universal</b> <code>reactive()</code> stores.
    </li>
  </ul>
  <p>
    <span>a long enough first element here</span>
    <span>and a second one that does not fit</span>
  </p>
</template>
```

In Vue, whitespace between two sibling elements keeps its kind where it renders (between inline elements):
a line break stays a line break and a same-line space stays a space that never breaks.
Vue's compiler drops whitespace containing a line break between elements and condenses the rest to a space,
so Prettier's output renders `universal reactive()` (a space the source did not render) and `hereand` (losing one).
Where the render layer ignores the gap (next to a block), and at text edges, the layout is Prettier's.

## angular-glued-block-markers

- Why: semantics (whitespace added around a block marker renders between the content it joins; prettier/prettier#16577, prettier/prettier#19020)
- Pin: `tests/fixtures/angular/angular-glued-block-markers.component.html`

```html
<!-- input -->
<p>a@if (x) {b}c</p>
<span>@if (test) {text}</span>

<!-- ours -->
<p>a@if (x) {b}c</p>
<span>@if (test) {text}</span>

<!-- prettier -->
<p>
  a
  @if (x) {
    b
  }
  c
</p>
<span>
  @if (test) {
    text
  }
</span>
```

An Angular control flow block makes no box: its markers (`@if (x) {`, `}`) render nothing,
so the whitespace on both sides of a marker is one run between the content around it.
Where that run is significant and empty, the markers stay glued to their neighbors;
otherwise a block is laid out on its own lines, as Prettier does.
Prettier always breaks around a block, which turns `abc` into `a b c` (and adds spaces inside the `span`).

## angular-block-name-with-line-break

- Why: semantics (Angular reads `@else⏎ if` as an unknown block, a compile error, and `@else if` as an else-if branch)
- Pin: `tests/fixtures/angular/angular-block-name-with-line-break.component.html`

```html
<!-- input -->
@if (a) {
  x
} @else
 if (b) {
  y
}

<!-- ours -->
@if (a) {
  x
}
@else
 if (b) {
  y
}

<!-- prettier -->
@if (a) {
  x
} @else if (b) {
  y
}
```

A block name holding a line break prints as written, on its own line, as an unknown block: Angular's compiler reports `Unrecognized block @else`.
Prettier collapses the name to `else if` and joins it to the previous block, turning the error into an else-if branch.
Whitespace without a line break (`@else   if`) is collapsed, as Angular reads it.
