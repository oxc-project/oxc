# Known divergences

Admission reasons and rules: see `crates/oxc_formatter_core/FORMATTER_POLICY.md` "Known divergences".

## ts-in-vue-generic-trailing-comma

- Why: uniform-rule (embedded script formats like its standalone file)
- Pin: `conformance/fixtures/edge-cases/js-in-vue/generic-trailing-comma.vue`
- Conformance: `externals/vue-vben-admin/effects/common-ui/src/components/api-component/api-component.vue`

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

## styled-extend-tag

- Why: cost
- Pin: `conformance/fixtures/edge-cases/css-in-js/styled-extend-tag.js`
- Conformance: `externals/prettier/js/multiparser-css/styled-components.js`

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
- Pin: `conformance/fixtures/edge-cases/gql-in-js/embedded-template-short-argument.js`

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
- Pin: `conformance/fixtures/edge-cases/gql-in-js/embedded-template-invalid-content.js`

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
- Pin: `conformance/fixtures/edge-cases/xxx-in-md/nested-fence-length.md`
- Conformance: `externals/prettier/js/multiparser-markdown/codeblock.js`

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
Prettier `main` counts the printed content since #19908; the pin (3.9.9) still counts the source.

## line-ranged-code-block

- Why: uniform-rule (same construct, same output: `` ```js{1,3} ``)
- Pin: `conformance/fixtures/edge-cases/xxx-in-md/line-ranged-code-block.md`

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
- Pin: `conformance/fixtures/edge-cases/xxx-in-md/lwc-code-block.md`
- Conformance: `externals/prettier/markdown/code/lwc/lwc.md`

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

A Markdown code block's language is looked up in Shiki's ids and aliases (`route()` in `src/core/embed/dispatcher.rs`);
`lwc` is none of them, so the block stays as written.
Prettier looks it up in linguist, where `lwc` is an alias of its Lightning Web Components language.
