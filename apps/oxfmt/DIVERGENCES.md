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
