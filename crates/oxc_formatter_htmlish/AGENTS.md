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
`<script>` / `<style>` bodies, `style` / `on*` values, in Vue (`src/print/vue.rs`) `{{ }}`, directive values and the blocks that are no markup,
and in Angular (`src/print/angular.rs`) `{{ }}`, `[x]`, `[(x)]`, `(x)`, `*x`, `{{ }}` in another attribute's value, block parameters and `@let`.
Vue's JS is a part in the file's script language, named by `JsInVueTemplate`;
Angular's is TS, named by `JsInAngularTemplate`, read as the parser splits it (`AngularExpression`):
each piece of code is dispatched on its own (the code, each pipe's argument, each statement of `(x)`, each expression of `*x` or a block's parameters),
Angular's own syntax around them (pipes, comment, `;`, microsyntax, block parameters) prints in `src/print/angular.rs`,
and the host tells JS what Angular reads of the code (`JsEmbeddedIn::template_expressions_only`).
The host translates either into JS's `JsRoot` and `JsEmbeddedIn`.
An attribute's value is laid out by its language, in the layout its marker names (JS's `JsRoot`, CSS's `CssEmbeddedIn`):
this crate only quotes it (`write_quoted`: the `"` and a group; a `"` in the value is `&quot;`, by `dispatch_attribute_value`), and knows nothing of the language's syntax,
but Angular's own (laid out around the pieces of JS in `src/print/angular.rs`).
A `<script>` body's output gets one HTML fact back, which no source body holds: a `</script` it prints, which would end the element, is `<\/script` (`escape_script_end`),
the same in a JS string or template and in JSON (an html-in-js template re-cooked from `<\/script>` prints one).

#### What Vue reads

The output compiles as the input does, a template that does not compile included: to the same values and side effects.
A handler's value is part of it: Vue reports a promise it rejects, and vue-tsc types it; a `;` decides whether the handler returns it (`$event => (…)` or `{…}`).
The oracle is `@vue/compiler-sfc`, which compiles every `.vue` file (`@vue/compiler-dom`'s Node build; oxc-htmlish-parser's `tasks/differential`, `probe.ts vue <source>`);
the browser build, which reads some code otherwise (`isFnExpression` is a regex there), is none: it compiles in-DOM templates, no `.vue` file.
vue-tsc (`@vue/language-core` 3.3.14) is one where it agrees with the compiler, and the compiler wins where it does not:
vue-tsc returns the value of a lone expression statement whose last token is no `;` (`foo(() => { a(); })` too, which the compiler reads as statements),
a `function` in parens included (which the compiler reads as the handler).

Vue reads a value by its AST, but for its text in these places (`@vue/compiler-core` 3.5.45), the only ones formatting can change what it reads by:

| Where       | What of the text                                       | Kept by                                                                                                                                                                                  |
| ----------- | ------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `{{ }}`     | the first `}}` ends it                                 | parens around an object (`oxc_formatter`'s AGENTS.md "js-in-xxx" names the known gap)                                                                                                    |
| `v-on`      | but for a function, a `;` anywhere makes it statements | the handler's `;` (`ProgramSemicolons::MarksStatements`), parens around a leading `function` / `{` (`reads_as_statements`); `oxc_formatter`'s AGENTS.md "js-in-xxx" names the known gaps |
| `v-for`     | split at `in` / `of` and the last commas               | the alias printed as the params Vue splits (`JsRoot::FunctionParams { parens_when_many }`), as written holding `in` / `of`                                                               |
| `:style`    | a leading `[`                                          | no rule: no formatting moves the first token                                                                                                                                             |
| every value | put in the render function as written                  | a line comment closed, a top-level sequence in no parens as written (`JsEmbeddedIn::inlines_code`)                                                                                       |
| every value | character references, decoded before the code is read  | as written when it holds one, a `{{ }}` one to a quote (apps/oxfmt/DIVERGENCES.md#character-reference-in-code-as-written)                                                                |

A Vue release reading more of the text adds a row here, and its rule.

- A part keeps what Vue reads by a print rule where one exists (a handler's `;`, parens, the value on its own lines before the quote), JS's through its markers ("js-in-xxx" in `oxc_formatter`'s AGENTS.md)
  - What Vue reads wins over the options: under `semi: false`, a `;` still joins a handler's statements, a guard for Vue as the ASI guard is for JS
- Where none does, or where the formatter cannot tell, it prints as written (`v-pre`, a top-level sequence in no parens, a line comment no line break closes)
- A divergence where Prettier's output compiles otherwise is `semantics`;
  one that shows only with a part's language formatted is an embedding divergence, `apps/oxfmt`'s (this crate's fixtures run no dispatcher)

#### What Angular reads

The output parses to the input's template AST, its expressions and i18n messages included.
The oracle is `@angular/compiler`'s `parseTemplate` (22.2.5), spans aside: oxc-htmlish-parser's `tasks/differential` pins it (`probe.ts angular <source>`).
The formatter cannot see `preserveWhitespaces` (a component's, the app's, or `ngPreserveWhitespaces`), so a whitespace change must render the same under both:

- `false` (the default) drops a whitespace-only text: whitespace added in a text renders, where only whitespace-only text was it does not
- `true` keeps every text, as HTML does, which this crate's whitespace sensitivity already covers

Angular reads the AST, but for its text in these places, each kept as Vue's are (a print rule, as written where none):

| Where                              | What of the text                                                         | Kept by                                                                                                                                                                   |
| ---------------------------------- | ------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `{{ }}`                            | the first `}}` outside a string ends it                                  | parens around an object (`oxc_formatter`'s AGENTS.md "js-in-xxx" names the known gap)                                                                                     |
| every expression                   | Angular's own grammar, not TS's                                          | only what JS reads the same (`JsEmbeddedIn::template_expressions_only`); a JS reserved word as written (apps/oxfmt/DIVERGENCES.md#js-in-angular-reserved-word-as-written) |
| every expression                   | character references, decoded before the code is read                    | as written when it holds one, a `{{ }}` one to a quote (apps/oxfmt/DIVERGENCES.md#character-reference-in-code-as-written)                                                 |
| `@let`                             | its `;` ends the value                                                   | the `;`, even under `semi: false`; a value ending in a comment as written (apps/oxfmt/DIVERGENCES.md#js-in-angular-let-comment-as-written)                                |
| `@placeholder` / `@loading`        | the last parameter runs to the `)`, whitespace included: an invalid time | the `)` right after it (DIVERGENCES.md#angular-time-parameter-before-paren)                                                                                               |
| around a block's markers           | whitespace                                                               | the markers glued where it renders and there is none (DIVERGENCES.md#angular-glued-block-markers)                                                                         |
| a block name                       | `@else⏎if` is an unknown block                                           | as written, on its own line (DIVERGENCES.md#angular-block-name-with-line-break)                                                                                           |
| a `//` comment in a start tag      | it runs to the line break, past the tag's `>`                            | a line break after it (DIVERGENCES.md#angular-start-tag-line-comment-end)                                                                                                 |
| i18n content, `i18n-x` values, ICU | the message ID hashes it as written                                      | as written (DIVERGENCES.md#angular-i18n-content-as-written)                                                                                                               |

An Angular release reading more of the text adds a row here, and its rule.

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

The example reads `.vue` as Vue, `.xhtml` / `.xht` as XHTML and `*.component.html` as Angular; fixtures do too, of the extensions `build.rs` lists (no `.xht`).
