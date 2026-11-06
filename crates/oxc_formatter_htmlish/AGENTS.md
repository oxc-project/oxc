# Coding agent guides for `crates/oxc_formatter_htmlish`

Follow @../oxc_formatter_core/FORMATTER_POLICY.md , this file holds only the htmlish-specific rules.
Known divergences live in DIVERGENCES.md.

## Overview

Prettier compatible formatter for HTML and the template languages built on it (Vue, Angular, ...), on `oxc_formatter_core`.

### Parser

Parses with [`oxc-htmlish-parser`](https://github.com/oxc-project/oxc-htmlish-parser).
Parse behavior and its own divergences are that repo's `AGENTS.md` / `DIVERGENCES.md`.

### Dialects

What a dialect changes in the shared rules is a `DialectRules` field (`src/rules.rs`), resolved once per document;
a dialect adds a field there, not a `Dialect` check at the use site.

### Embedded languages

Content in another language goes to the session's dispatcher (`src/print/embed.rs`), and prints as written where it is not formatted:
`<script>` / `<style>` bodies, `style` / `on*` values, and in Vue (`src/print/vue.rs`) `{{ }}`, directive values and the blocks that are no markup.
Vue's JS is a part in the file's script language, named by `JsInVueTemplate`, which the host translates into JS's `JsRoot` and `JsEmbeddedIn`.
An attribute's value is laid out by its language, in the layout its marker names (JS's `JsRoot`, CSS's `CssEmbeddedIn`):
this crate only quotes it (`write_quoted`: the `"`, `&quot;` for a `"` in it, a group), and knows nothing of the language's syntax.

#### What Vue reads

The output compiles as the input does, a template that does not compile included: to the same values and side effects.
A handler's value is no part of it: Vue uses it only to report a promise it rejects, and a `;` the formatter adds or drops inside changes it (`$event => (…)` or `{…}`).
The oracle is `@vue/compiler-sfc`, which compiles every `.vue` file (`@vue/compiler-dom`'s Node build);
the browser build, which reads some code otherwise (`isFnExpression` is a regex there), is none: it compiles in-DOM templates, no `.vue` file.

Vue reads a value by its AST, but for its text in these places (`@vue/compiler-core` 3.5.45), the only ones formatting can change what it reads by:

| Where       | What of the text                                                      | Kept by                                                                                                                    |
| ----------- | --------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------- |
| `{{ }}`     | the first `}}` ends it                                                | parens around an object (`oxc_formatter`'s AGENTS.md "js-in-xxx" names the known gap)                                      |
| `v-on`      | but for a reference or a function, a `;` anywhere makes it statements | the handler's `;` (`ProgramSemicolons::MarksStatements`), parens around a leading `function` / `{` (`reads_as_statements`) |
| `v-for`     | split at `in` / `of` and the last commas                              | the alias printed as the params Vue splits (`JsRoot::FunctionParams { parens_when_many }`)                                 |
| `:style`    | a leading `[`                                                         | no rule: no formatting moves the first token                                                                               |
| every value | put in the render function as written                                 | a line comment closed, a top-level sequence in no parens as written (`JsEmbeddedIn::inlines_code`)                         |

A Vue release reading more of the text adds a row here, and its rule.

- A part keeps what Vue reads by a print rule where one exists (a handler's `;`, parens, the value on its own lines before the quote)
  - What Vue reads wins over the options: under `semi: false`, a `;` still joins a handler's statements, a guard for Vue as the ASI guard is for JS
- Where none does, or where the formatter cannot tell, it prints as written (`v-pre`, a top-level sequence in no parens, a line comment no line break closes)
- Which reading a part has is its `JsInVueTemplate`, translated by the host into JS's markers, whose contract is `oxc_formatter`'s AGENTS.md "js-in-xxx"
- Otherwise Prettier is matched, but for the divergences (`apps/oxfmt/DIVERGENCES.md`, this crate's DIVERGENCES.md); one where Prettier's output compiles otherwise is `semantics`
  - Which parts dispatch, and how one lays out in the markup, shows only with its language formatted:
    an embedding divergence, `apps/oxfmt`'s (this crate's fixtures run no dispatcher, every part prints as written)

### Preprocess

The facts the printer needs besides the AST (CSS display, whitespace sensitivity, kept children, borrowed tag markers)
are a table keyed by `NodeId` (`src/tree/`).
The AST is never rewritten.

### Tags

An end tag the source omits is never added (DIVERGENCES.md#unclosed-at-end, DIVERGENCES.md#omitted-end-tag).
A normalization is applied only where it is known safe for every reader, as in `oxc_formatter_css` (its AGENTS.md "Case"):
a void element prints `/>`, a doctype `<!doctype html>` in a standalone HTML document (a Vue file read as HTML too) or Angular template only,
whose reader is an HTML parser (DIVERGENCES.md#doctype-case),
and the tag and attribute names Prettier knows print lowercase in a `.html` file and a Vue file read as HTML only (DIVERGENCES.md#tag-name-case).

The borrow rules (`src/print/tag.rs`) are shaped like Prettier's `print/tag.js` against the policy, kept deliberately (why: its module doc).

## Verification

```sh
cargo run -p oxc_formatter_htmlish --example htmlish_formatter [filename]
DUMP_IR=1 cargo run -p oxc_formatter_htmlish --example htmlish_formatter [filename]

# The oracle. The parser follows the file name as the example's variant does (`*.component.html` is angular).
node apps/oxfmt/node_modules/prettier/bin/prettier.cjs [filename]
```

Fixtures and the example read `.vue` as Vue, `.xhtml` / `.xht` as XHTML and `*.component.html` as Angular.
