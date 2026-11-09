# Known divergences

Admission reasons and rules: see `crates/oxc_formatter_core/FORMATTER_POLICY.md` "Known divergences".

## ts-in-vue-generic-trailing-comma

- Why: uniform-rule (embedded script formats like its standalone file)
- Pin: `conformance/fixtures/js-in-vue/generic-trailing-comma.vue`
- Conformance:
  - `externals/vue-vben-admin/effects/common-ui/src/components/api-component/api-component.vue`

```vue
<!-- input -->
<script setup lang="ts">
const getComponentRef = <T = any,>() => componentRef.value as T;
</script>

<!-- ours -->
<script setup lang="ts">
const getComponentRef = <T = any>() => componentRef.value as T;
</script>

<!-- prettier -->
<script setup lang="ts">
const getComponentRef = <T = any,>() => componentRef.value as T;
</script>
```

A ts-in-vue script formats exactly like plain `.ts`: the disambiguating trailing comma in `<T,>` is only
required where JSX is possible (`.tsx`, `.mts`/`.cts`). Prettier keeps it in ts-in-xxx embeds but removes it
in ts-in-md and plain `.ts` — one rule over that internal inconsistency, the same one css-in-js and gql-in-js follow.

## js-in-vue-for-alias-as-params

- Why: invariant
- Pin: `conformance/fixtures/js-in-vue/for-alias-commas.vue`

```vue
<!-- input -->
<template>
  <li v-for="(item = 'a, b', i) in items">x</li>
  <li v-for="(item /* a, b */, i) in items">x</li>
</template>

<!-- ours -->
<template>
  <li v-for="(item = 'a, b', i) in items">x</li>
  <li v-for="(item /* a, b */, i) in items">x</li>
</template>

<!-- prettier -->
<template>
  <li v-for="(item = 'a,b', i) in items">x</li>
  <li v-for="(item /* a,b */, i) in items">x</li>
</template>
```

`v-for`'s alias formats as the params it is, its strings and comments as written, as `v-slot`'s do.
Prettier splits it at its last commas as Vue does, and joins the trimmed parts with `,`, which drops the spaces around a comma in a string or a comment.
Vue splits there too, and compiles neither (`@vue/compiler-dom` 3.5.45: `Error parsing JavaScript expression`).
The comment Prettier prints is not the user's (`/* a,b */`), which breaks the lossless contract;
the string is uniform-rule (same construct, same output: a default value in `v-slot`'s params).

## js-in-vue-bitwise-or

- Why: uniform-rule (same construct, same output: a `&&` chain in the same root)
- Pin: `conformance/fixtures/js-in-vue/bitwise-or.vue`
- Conformance:
  - `externals/prettier/vue/interpolation/bitwise-or-operator.vue`
  - `externals/prettier/vue/ts-expression/filter.vue`
  - `externals/prettier/vue/vue/filter.vue`

```vue
<!-- input -->
<template>
  <p v-if="aaaaaaaaaaaaaaaaaaaa | bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb | cccccccccccccccccccccccc">{{ value | thisIsARealSuperLongFilterPipe("arg1", arg2) | anotherPipeLongJustForFun }}</p>
</template>

<!-- ours -->
<template>
  <p
    v-if="
      aaaaaaaaaaaaaaaaaaaa |
      bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb |
      cccccccccccccccccccccccc
    "
  >
    {{
      value |
      thisIsARealSuperLongFilterPipe("arg1", arg2) |
      anotherPipeLongJustForFun
    }}
  </p>
</template>

<!-- prettier -->
<template>
  <p
    v-if="
      aaaaaaaaaaaaaaaaaaaa |
        bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb |
        cccccccccccccccccccccccc
    "
  >
    {{
      value
        | thisIsARealSuperLongFilterPipe("arg1", arg2)
        | anotherPipeLongJustForFun
    }}
  </p>
</template>
```

A `|` in a template expression formats as the bitwise OR it is to Vue 3, as any operator does there:
each `|` ends its line, and its chain does not indent under the host's delimiters, as a `&&` chain does (and as `|` and `&&` print alike in a `<script>`).
Vue 3 removed Vue 2's filters (`@vue/compiler-dom` 3.5.45 compiles `{{ a | f }}` to `a | f`);
Prettier still reads `{{ }}`'s and a binding's `|` as filters, starting each line with `|`, and indents every `|` chain there.
A migration build (`@vue/compat`'s `COMPILER_FILTERS`) still reads them as filters, which compile the same either way: only the line breaks differ.

## js-in-vue-removed-attributes-as-written

- Why: semantics (Vue 3 passes `slot-scope` to the element as a string, which formatting changes)
- Pin: `conformance/fixtures/js-in-vue/removed-attributes-as-written.vue`
- Conformance:
  - `externals/prettier/vue/vue/attributes.vue`
  - `externals/prettier/vue/vue/slot-ts.vue`
  - `externals/prettier/vue/vue-3/script-setup.vue`
  - `externals/prettier/vue/vue-3/style-variables.vue`

```vue
<!-- input -->
<script setup="  {row}  "></script>
<style vars="  {color}  "></style>
<template>
  <C slot-scope="  {row}  ">x</C>
</template>

<!-- ours -->
<script setup="  {row}  "></script>
<style vars="  {color}  "></style>
<template>
  <C slot-scope="  {row}  ">x</C>
</template>

<!-- prettier -->
<script setup="{ row }"></script>
<style vars="{ color }"></style>
<template>
  <C slot-scope="{ row }">x</C>
</template>
```

`slot-scope`, Vue 2's slot props, and `<script setup="…">` / `<style vars="…">` of Vue 3's early RFCs, which Vue 3 removed, print as written, as any plain attribute does.
Vue 3 passes `slot-scope` to the element as a string (`@vue/compiler-dom` 3.5.45: `slot-scope="  {row}  "` is `"  {row}  "`), which Prettier's formatting changes;
it reads neither value of `setup` and `vars` (`@vue/compiler-sfc` 3.5.45 compiles `<script setup>` the same whatever the value), which Prettier formats as JS.

## js-in-vue-v-pre-as-written

- Why: semantics (Vue renders a `v-pre` element and its content as written)
- Pin: `conformance/fixtures/js-in-vue/v-pre-as-written.vue`

```vue
<!-- input -->
<template>
  <div v-pre :a="b+c">{{a+b}} <span @click="x+y">{{c}}</span></div>
</template>

<!-- ours -->
<template>
  <div v-pre :a="b+c">{{a+b}} <span @click="x+y">{{c}}</span></div>
</template>

<!-- prettier -->
<template>
  <div v-pre :a="b + c">
    {{ a + b }} <span @click="x + y">{{ c }}</span>
  </div>
</template>
```

In a `v-pre` element and its content, `{{ }}` is text and a directive a plain attribute, as Vue's compiler reads them:
they print as written, and lay out as text and attributes do.
Prettier formats them, which changes what renders (`{{ a + b }}`) and the attribute values (`:a` is `b + c`), as `@vue/compiler-dom` 3.5.45 compiles them.

## js-in-vue-handler-statements

- Why: semantics (Vue reads an event handler holding a `;` as statements, and one without as an expression; prettier/prettier#14270)
- Pin: `conformance/fixtures/js-in-vue/handler-statements-semi-false.vue`
- Conformance:
  - `externals/prettier/vue/event-binding/function-expression-ts.vue`
  - `externals/prettier/vue/vue/attributes.vue`

```vue
<!-- input (semi: false) -->
<template>
  <button @click="count++; if (a) { b(); c() }"></button>
</template>

<!-- ours -->
<template>
  <button
    @click="
      count++;
      if (a) {
        b()
        c()
      }
    "
  ></button>
</template>

<!-- prettier -->
<template>
  <button
    @click="
      count++
      if (a) {
        b()
        c()
      }
    "
  ></button>
</template>
```

Under `semi: false`, `;` still separates an event handler's top-level statements, all but after the last,
and ends a lone one that is no expression (`if (a) b();`); the statements in them follow `semi`.
Vue's compiler reads a handler as statements when its value holds a `;` (`exp.content.includes(';')`), wrapped in braces, and as an expression otherwise:
Prettier drops every `;`, and the statements no longer compile (`@vue/compiler-dom` 3.5.45: `Error parsing JavaScript expression`).
Under `semi: true` both print every `;` between and after the statements.

```vue
<!-- input (any semi) -->
<button @focus="if (a) b()"></button>

<!-- ours -->
<button @focus="if (a) b()"></button>

<!-- prettier -->
<button @focus="if (a) b();"></button>
```

Statements without a `;` print as written: Vue reads them as an expression, which does not compile, and the `;` Prettier adds makes them compile.
A `;` added or dropped in a handler's code otherwise (`a();` printing `a()`, a callback's body under `semi`) changes no more than its value,
which is no part of what Vue reads (`crates/oxc_formatter_htmlish/AGENTS.md` "What Vue reads").

```vue
<!-- input (any semi) -->
<button @focus="; /* c */"></button>

<!-- ours -->
<button @focus="; /* c */"></button>

<!-- prettier -->
<button @focus="/* c */"></button>
```

A handler of nothing but comments and `;` prints as written: without its `;`, Vue reads the comment as an expression, which does not compile.

```vue
<!-- input -->
<button @keyup="function () { return 1 }"></button>
<button @keyup="({ a() { b(); c() } }).a()"></button>

<!-- ours -->
<button
  @keyup="
    (function () {
      return 1;
    })
  "
></button>
<button
  @keyup="
    ({
      a() {
        b();
        c();
      },
    }).a()
  "
></button>

<!-- prettier -->
<button
  @keyup="
    function () {
      return 1;
    }
  "
></button>
<button
  @keyup="
    {
      a() {
        b();
        c();
      },
    }.a()
  "
></button>
```

A handler expression starts a statement, as an expression statement does: a leading `function`, `{`, `class` or `let[`, and an object pattern's assignment, print in parens.
Its bodies may print a `;` (`return 1;`), and Vue reads statements there:
a `function` begins a declaration (one without a name does not compile, a named one declares a function the click never calls),
and a `{` a block. Prettier drops the parens, and the handler no longer compiles.
An arrow function reads as itself either way.
A handler whose code holds a `;` already and starts so (`function () { a(); }`) prints as written:
Vue reads it as statements, which do not compile, and the parens would make it compile.

## tsx-in-vue-template-expressions

- Why: uniform-rule (same construct, same output: a template expression of a `lang="ts"` file, which Prettier parses as TS)
- Pin: `conformance/fixtures/js-in-vue/template-expression-tsx.vue`

```vue
<!-- input -->
<script setup lang="tsx">
const a = <div />;
</script>

<template>
  <p>{{ (value  as  Item).name }}</p>
</template>

<!-- ours -->
<script setup lang="tsx">
const a = <div />;
</script>

<template>
  <p>{{ (value as Item).name }}</p>
</template>

<!-- prettier -->
<script setup lang="tsx">
const a = <div />;
</script>

<template>
  <p>{{ (value  as  Item).name }}</p>
</template>
```

A Vue file whose `<script>` is TSX has its template expressions parsed as TSX, as Vue's compiler reads them (`lang === 'ts' || lang === 'tsx'`).
Prettier parses them as TS only for `lang="ts"`, and as JS for `lang="tsx"`, so it leaves TS syntax there as written.

## js-in-vue-tailwind-functions

- Why: uniform-rule (same construct, same output: a `tailwindFunctions` call in a `<script>`, and in `v-if` of a JS file)
- Pin: `conformance/fixtures/js-in-vue/sort-tailwindcss-functions/every-template-expression.vue`

```vue
<!-- input: { sortTailwindcss: { functions: ["clsx"] } } -->
<script setup lang="ts">
const a = clsx("p-4 flex");
</script>

<template>
  <p v-if="clsx('p-4 flex')">{{ clsx("p-4 flex") }}</p>
  <p :title="clsx('p-4 flex')" @click="f(clsx('p-4 flex'))" />
</template>

<!-- ours -->
<script setup lang="ts">
const a = clsx("flex p-4");
</script>

<template>
  <p v-if="clsx('flex p-4')">{{ clsx("flex p-4") }}</p>
  <p :title="clsx('flex p-4')" @click="f(clsx('flex p-4'))" />
</template>

<!-- prettier (prettier-plugin-tailwindcss) -->
<script setup lang="ts">
const a = clsx("flex p-4");
</script>

<template>
  <p v-if="clsx('p-4 flex')">{{ clsx("p-4 flex") }}</p>
  <p :title="clsx('p-4 flex')" @click="f(clsx('p-4 flex'))" />
</template>
```

A listed function's classes sort in every template expression, as in the file's `<script>`.
prettier-plugin-tailwindcss sorts them only in some, by which parser Prettier picks:
`v-if`, an `@click` expression and `v-for` of a JS file, none in a TS file.
What the attributes sort (`class`, `:class`, a listed `attributes`) is the same.

## js-in-vue-trailing-line-comment

- Why: semantics (Prettier's output adds the comment's words as attributes of the element)
- Pin: `conformance/fixtures/js-in-vue/trailing-line-comment.vue`
- Conformance:
  - `externals/prettier/vue/vue/attributes.vue`

```vue
<!-- input -->
<template>
  <li v-for="item   in items // c">x</li>
  <a :a="{ b } // c" :b="x  +  y // c" />
  <p>{{ a  +  b // c }}</p>
</template>

<!-- ours -->
<template>
  <li v-for="item   in items // c">x</li>
  <a :a="{ b } // c" :b="x  +  y // c" />
  <p>{{ a  +  b // c }}</p>
</template>

<!-- prettier -->
<template>
  <li
    v-for="item in items" // c
  >
    x
  </li>
  <a
    :a="{ b }" // c
    :b="
      x + y // c
    "
  />
  <p>
    {{
      a + b // c
    }}
  </p>
</template>
```

A Vue `{{ }}` or directive value whose code ends in a line comment keeps what Vue reads of it:
Vue puts the code in its output up to the value's end, so the comment comments out what follows there,
unless a line break closes it first (`@vue/compiler-dom` 3.5.45 fails to compile `:a="b // c"`, and compiles `:a="b // c⏎"`).
- Closed by a line break the value ends with, the value is formatted on its own lines, the comment closed before the quote;
  so is one other comments follow (`{ b } // c⏎/* d */`).
- Not closed, it prints as written. So does every `{{ }}` and `v-for`'s source, whose code Vue trims first,
  and `v-for`'s alias, whose comment would move past `in` and the source.

A comment followed by more code (`{ a, // c⏎ b }`) is formatted as anywhere.
Prettier prints a comment that ends a value hugging the quotes (an object, `v-for`, slot props) past the `"`,
which Vue reads as attributes `//` and `c` (`Illegal '/' in tags`);
it puts the others on their own lines, which makes a binding compile where it did not.

## js-in-vue-top-level-sequence-as-written

- Why: semantics (Vue reads a top-level `a, b` as more than one value, which the added parens make one)
- Pin: `conformance/fixtures/js-in-vue/top-level-sequence.vue`

```vue
<!-- input -->
<template>
  <p :class="a,b" v-show="a,b">{{ a,b }}</p>
  <p :class="(a),b">{{ (a,b),c }}</p>
</template>

<!-- ours -->
<template>
  <p :class="a,b" v-show="a,b">{{ a,b }}</p>
  <p :class="(a),b">{{ (a,b),c }}</p>
</template>

<!-- prettier -->
<template>
  <p :class="(a, b)" v-show="(a, b)">{{ (a, b) }}</p>
  <p :class="(a, b)">{{ ((a, b), c) }}</p>
</template>
```

A Vue `{{ }}` or directive value that is a sequence expression in no parens prints as written.
Vue puts the code in its output as written (`@vue/compiler-dom` 3.5.45), where `a, b` is not one value:
- `{{ }}`, `v-text`, `:class`, `:style` and `v-bind` pass it as two arguments (`toDisplayString(a, b)`), and use `a`;
- `v-show` and a custom directive take `b` as the directive's argument (`[vShow, a, b]`);
- `v-for`'s source, `v-memo`, a bound attribute and `v-html` fail (`renderList(a, b, fn)`, `{ x: a, b }`).

Prettier prints it in parens, which makes it the one value `b`, or makes the code compile where it did not.
In parens, it is one value, and formats as Prettier does (`(a, b)`):
the source's (`{{ (a,b) }}`), or Vue's own, around `v-if`, `v-else-if` and an event handler's expression (`$event => (…)`).

## js-in-vue-interpolation-in-broken-parent

- Why: uniform-rule (same construct, same output: `{{ }}` in a `<div>`; prettier/prettier#11406)
- Pin: `conformance/fixtures/js-in-vue/interpolation-in-broken-parent.vue`

```vue
<!-- input -->
<template>
  <ul>
    {{ item }}
  </ul>
  <select>{{ selected }}</select>
</template>

<!-- ours -->
<template>
  <ul>
    {{ item }}
  </ul>
  <select>
    {{ selected }}
  </select>
</template>

<!-- prettier -->
<template>
  <ul>
    {{
      item
    }}
  </ul>
  <select>
    {{
      selected
    }}
  </select>
</template>
```

`{{ }}` keeps its expression on its line when it fits, in an element that puts each child on its own line
(`ul`, `ol`, `select`, `display: table*`) as in any other.
Prettier lays out such children in the parent's group, which always breaks, so its interpolation always breaks too.

## js-in-vue-object-before-double-brace

- Why: semantics (Vue ends `{{ }}` at the first `}}`)
- Pin: `conformance/fixtures/js-in-vue/bracket-spacing-false/interpolation-closing-braces-template-literal.vue`, `conformance/fixtures/js-in-vue/bracket-spacing-false/interpolation-closing-braces.vue`

```vue
<!-- input -->
<template>
  <p>{{ `${ {a: 1} }` }}</p>
</template>

<!-- ours -->
<template>
  <p>{{ `${({ a: 1 })}` }}</p>
</template>

<!-- prettier -->
<template>
  <p>{{ `${{ a: 1 }}` }}</p>
</template>
```

```vue
<!-- input (bracketSpacing: false) -->
<template>
  <p>{{ { a: { b: 1 } } }}</p>
  <p>{{ { ...{ a: 1 } } }}</p>
  <p>{{ { ...!{ a: 1 } } }}</p>
</template>

<!-- ours -->
<template>
  <p>{{ {a: ({b: 1})} }}</p>
  <p>{{ {...({a: 1})} }}</p>
  <p>{{ {...!({a: 1})} }}</p>
</template>

<!-- prettier -->
<template>
  <p>{{ {a: ({b: 1})} }}</p>
  <p>{{ {...{a: 1}} }}</p>
  <p>{{ {...!{a: 1}} }}</p>
</template>
```

In `{{ }}`, an object that would print right before another `}` prints in parens:
a template literal's substitution's, whatever `bracketSpacing` says, and, under `bracketSpacing: false`, an object's (a spread's argument too).
Vue's tokenizer ends the interpolation at the first `}}`, so `{{ {...{a: 1}} }}` no longer compiles (`@vue/compiler-dom` 3.5.45: `Error parsing JavaScript expression`).
Prettier does it for a property's value only.

## styled-extend-tag

- Why: cost
- Pin: `conformance/fixtures/css-in-js/styled-extend-tag.js`
- Conformance:
  - `externals/prettier/js/multiparser-css/styled-components.js`

```js
/* input */
const TomatoButton = Button.extend`
	color  : tomato  ;
`;

/* ours */
const TomatoButton = Button.extend`
	color  : tomato  ;
`;

/* prettier */
const TomatoButton = Button.extend`
  color: tomato;
`;
```

`Xxx.extend` / `Xxx.extend.attr(...)` (styled-components v3, removed in v4) is not recognized as a css-in-js tag,
so its template stays verbatim; Prettier still formats it. Deprecated API, not worth extending the tag heuristic.

## embedded-template-short-argument

- Why: invariant
- Pin: `conformance/fixtures/gql-in-js/embedded-template-short-argument.js`

```js
/* input */
const schema = graphql(`query{users{x}}`);

/* ours */
const schema = graphql(`
  query {
    users {
      x
    }
  }
`);

/* prettier */
const schema =
  graphql(`
    query {
      users {
        x
      }
    }
  `);
```

To decide whether an assignment breaks after `=`, Prettier checks if a sole template argument is short by its source text.
An embedded template is rewritten, so that text says nothing about the output:
the first pass breaks after `=`, and the second pass hugs, which is not a fixpoint.
We treat an embedded template as never short, which gives Prettier's second-pass output (its fixpoint).

## embedded-template-invalid-content

- Why: uniform-rule (same construct, same output: the same template with valid content)
- Pin: `conformance/fixtures/gql-in-js/embedded-template-invalid-content.js`

```js
/* input */
foo(
  gql`
    query {{{
  `
);

/* ours */
foo(gql`
    query {{{
  `);

/* prettier */
foo(
  gql`
    query {{{
  `,
);
```

The layout of an embedded template as a sole argument or an arrow body is decided from the AST, whether its content formats or not.
Prettier falls back to the source shape when the content fails to format, while it hugs the same template with valid content.
The verbatim content cannot be re-indented, so its source indentation stays and may look misaligned once hugged (as with any verbatim template Prettier hugs).
Under `embeddedLanguageFormatting: off`, the source shape still decides, like Prettier.

## nested-fence-length

- Why: semantics (prettier/prettier#19908)
- Pin: `conformance/fixtures/xxx-in-md/nested-fence-length.md`
- Conformance:
  - `externals/prettier/js/multiparser-markdown/codeblock.js`

`````markdown
<!-- input -->
```md
~~~js
const  a = 1
~~~
```

<!-- ours -->
````md
```js
const a = 1;
```
````

<!-- prettier -->
```md
```js
const a = 1;
```
```
`````

A fenced code block's fence outnumbers the backtick (or tilde) runs of the content it prints, the embedded formatter's output included.
Markdown in a code block formats its own fences, so `~~~` becomes a backtick fence the outer one must outnumber;
Prettier keeps the outer fence at three and the inner fence closes it on the next parse.
The same holds for markdown-in-js, whose fences are `~`.
Prettier `main` counts the printed content since #19908; the pin (3.9.10) still counts the source.

## line-ranged-code-block

- Why: uniform-rule (same construct, same output: `` ```js{1,3} ``)
- Pin: `conformance/fixtures/xxx-in-md/line-ranged-code-block.md`

````markdown
<!-- input -->
```js {4}
const a = [
  1, 2,
];
console.log(a);
```

<!-- ours -->
```js {4}
const a = [
  1, 2,
];
console.log(a);
```

<!-- prettier -->
```js {4}
const a = [1, 2];
console.log(a);
```
````

A code block whose meta addresses lines by number (`{1,3-5}`, VitePress / Docusaurus line highlighting) stays as written:
formatting moves the lines the numbers point at.
Above, `{4}` highlights `console.log(a)`; Prettier's output has two lines, so it highlights nothing.
Prettier formats it when a space separates the language, and keeps `js{1,3}` as written because `js{1,3}` is no language it knows.

## lwc-code-block

- Why: uniform-rule (code fence names follow Shiki's vocabulary)
- Pin: `conformance/fixtures/xxx-in-md/lwc-code-block.md`
- Conformance:
  - `externals/prettier/markdown/code/lwc/lwc.md`

````markdown
<!-- input -->
```lwc
<x-a attr={b}>
    c</x-a>
```

<!-- ours -->
```lwc
<x-a attr={b}>
    c</x-a>
```

<!-- prettier -->
```lwc
<x-a attr={b}> c</x-a>
```
````

A Markdown code block's language is looked up in Shiki's ids and aliases (`route_embedded()` in `src/core/language.rs`);
`lwc` is none of them, so the block stays as written.
Prettier looks it up in linguist, where `lwc` is an alias of its Lightning Web Components language.

## whitespace-only-code-block

- Why: uniform-rule (embedded script formats like its standalone file)
- Pin: `conformance/fixtures/xxx-in-md/whitespace-only-code-block.md`

`````markdown
<!-- input -->
```json


```

<!-- ours -->
```json

```

<!-- prettier -->
```json


```
`````

A blank `.json` file formats to an empty one, and so does a code block of blank lines, an empty block like `jsonc` / `css` / `js` in Prettier too.
Prettier keeps `json` / `json5` blocks as written: their parser rejects a blank document inside Markdown.

## astro-style-lang-sass

- Why: cost
- Pin: `conformance/fixtures/astro/style-lang-sass.astro`
- Conformance:
  - `externals/plugin-astro/styles/with-sass/input.astro`
  - `externals/plugin-astro/styles/with-indented-sass/input.astro`
  - `externals/plugin-astro/styles/format-nested-sass-style-tag-content/input.astro`

```astro
<!-- input -->
<style lang="sass">
.a
      color: red
</style>

<!-- ours -->
<style lang="sass">
.a
      color: red
</style>

<!-- prettier -->
<style lang="sass">
  .a
    color: red
</style>
```

`<style lang="sass">` (indented syntax) stays verbatim, consistent with Vue / Svelte and also `.sass` file.
`prettier-plugin-astro` formats it with bundled `sass-formatter`, which we do not bundle (patched out).
Rare in Astro projects (GitHub code search, 2026-10: about 350 files in 72 repos, vs 29k for `lang="scss"`).
Not tested but `oxc-css-parser` already parses it, so we can add Sass support for `oxc_formatter_css` in the future.

## front-matter-indented-delimiter

- Why: semantics
- Pin: `conformance/fixtures/xxx-in-md/front-matter-indented-delimiter.md`

```markdown
<!-- input -->
---
 ---
title:   Home
---

<!-- ours -->
---
 ---
title:   Home
---

<!-- prettier -->
---
---
title: Home
---
```

A front matter block whose body starts with an indented delimiter (` ---`, ` ...`, ` +++`) stays verbatim in CSS and Markdown (`write_front_matter` in `oxc_formatter_core`).
The indented delimiter does not close the block, but the body is trimmed before formatting, so it prints at the start of a line.
On the next parse (Prettier's and ours), the block closes there and the rest of the body moves into the document.

## script-end-in-embedded-template

- Why: semantics (the first `</script` ends an HTML `<script>`, the rest of the code becomes markup)
- Pin: `conformance/fixtures/html/script-end-in-gql-template.html`

```html
<!-- input -->
<script>
  const query = gql`
    query {
      field # <\/script>
    }
  `;
</script>

<!-- ours -->
<script>
  const query = gql`
    query {
      field # <\/script>
    }
  `;
</script>

<!-- prettier -->
<script>
  const query = gql`
    query {
      field # </script>
    }
  `;
</script>
```

An HTML `<script>` body never prints `</script`: one the formatting produces (an embedded template part printed from its cooked value, html-in-js or gql-in-js) prints as `<\/script`.
Prettier keeps it in html-in-js, css-in-js and markdown-in-js, but a gql-in-js part prints the cooked `</script`, which closes the `<script>` there.

## style-attribute-last-semicolon-comment

- Why: uniform-rule (comment presence never changes layout)
- Pin: `conformance/fixtures/html/style-attribute-last-semicolon-comment.html`

```html
<!-- input -->
<div style="color: red;"></div>
<div style="color: red; /* c */"></div>
<div style="color: red /* c */"></div>

<!-- ours -->
<div style="color: red"></div>
<div style="color: red /* c */"></div>
<div style="color: red /* c */"></div>

<!-- prettier -->
<div style="color: red"></div>
<div style="color: red; /* c */"></div>
<div style="color: red; /* c */"></div>
```

A `style` attribute's last declaration prints its `;` only when the attribute breaks, whether a comment follows or not.
Same rule as `crates/oxc_formatter_css/DIVERGENCES.md#keyframe-selector-comment-list`;
the `;` is a terminator, so the comment may sit where it was (`crates/oxc_formatter_css/AGENTS.md` "Placement invariants").
Prettier prints the `;` when a comment follows: its last-declaration check counts the comment as a node.

## doctype-case-in-script

- Why: semantics (an html-in-js template is a JS string, and code reads its case; oxc#28793)
- Pin: `conformance/fixtures/html/doctype-case-in-script.html`

```html
<!-- input -->
<script>
  const page = html`<!DOCTYPE html>
    <html></html>`;
</script>

<!-- ours -->
<script>
  const page = html`<!DOCTYPE html>
    <html></html>`;
</script>

<!-- prettier -->
<script>
  const page = html`<!doctype html>
    <html></html>`;
</script>
```

An html-in-js doctype prints as written in every host (crates/oxc_formatter_htmlish/DIVERGENCES.md#doctype-case).
Prettier passes the host's file name to the embedded part, so it lowercases one in a `.html` / `.htm` host only.

## placeholder-case-in-child

- Why: semantics (Prettier prints a lowercased placeholder in place of the `${}` expression)
- Pin: `conformance/fixtures/html-in-js/placeholder-case-in-child.js`

```js
// input
const a = html`<style>a { ${b}: ${c} }</style>`;

// ours
const a = html`<style>
  a {
    ${b}: ${c};
  }
</style>`;

// prettier
const a = html`<style>
  a {
    prettier_html_placeholder_0_0_in_js: ${c};
  }
</style>`;
```

An html-in-js placeholder is found in the child's output ASCII case-insensitively (`HTML_PLACEHOLDER_PREFIX` in `oxc_formatter`):
CSS lowercases a property name, and the placeholder standing in it with it.
Prettier looks for the placeholder as it wrote it, so the lowercased one stays in the output and `${b}` is lost.

## pre-like-template-expression

- Why: uniform-rule (same construct, same output: an expression outside `pre`-like content, whose lines indent from the line its `${` starts on; prettier/prettier#20355)
- Pin: `conformance/fixtures/html-in-js/pre-like-template-expression.js`, `conformance/fixtures/html-in-js/template-expression-indent.js`
- Conformance:
  - `externals/prettier/js/embeded/indention/19518.js`

```js
// input
html`
  <form>
    <textarea>
${foo({
  x: 1,
})}</textarea>
  </form>
`;

// ours
html`
  <form>
    <textarea>
${foo({
  x: 1,
})}</textarea>
  </form>
`;

// prettier
html`
  <form>
    <textarea>
${foo({
        x: 1,
      })}</textarea>
  </form>
`;
```

A `${}` expression in `pre`-like content (`pre`, `textarea`, ...) indents its lines from the whitespace its line starts with,
as an expression elsewhere does from the line its `${` starts on.
A line of that content prints from the root, so the expression aligns there (`oxc_formatter_htmlish`'s `write_pre_like_text`);
one on the line of an element before it (`<pre><b>x</b>${`) indents from that line, as elsewhere.
Prettier 3.9.6 aligned an expression starting its line so; since 3.9.7 (prettier/prettier#19725, for idempotency) it indents it by the elements around the content instead,
which leaves `})` right of the `${` it closes.
Reformatting the expression changes no content: its source is code, not text in the element.

## dynamic-tag-name-end-tag

- Why: uniform-rule (same construct, same output: an element htm's `<//>` closes, which Prettier formats; prettier/prettier#17849)
- Pin: `conformance/fixtures/html-in-js/dynamic-tag-name.js`
- Conformance:
  - `externals/webawesome/button/button.ts`

```js
// input
html`<${For} each=${() => items}><li class="item">${() => content}</li><li class="item">more items</li></${For}>`;

// ours
html`<${For} each=${() => items}
  ><li class="item">${() => content}</li>
  <li class="item">more items</li></${For}
>`;

// prettier
html`<${For} each=${() => items}><li class="item">${() => content}</li><li class="item">more items</li></${For}>`;
```

An element whose tag name is a `${}` expression (lit's static `html`, Solid, htm) closes at an end tag named by one, as at htm's `<//>`, and is formatted the same.
Prettier gives each `${}` its own placeholder, so the end tag names another element than the start tag: its parser stops ("Unexpected closing tag") and the template stays as written.
The parser is told the placeholders, not the expressions (`oxc-htmlish-parser`'s `ParserOptions::template_placeholder`):
an end tag named by one closes the innermost element named by one, and such an element written `/>` closes there.
