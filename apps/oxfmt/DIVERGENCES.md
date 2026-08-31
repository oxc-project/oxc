# Known divergences

Admission reasons and rules: see `crates/oxc_formatter_core/FORMATTER_POLICY.md` "Known divergences".

## template-expression-indent

- Why: prettier-bug (prettier/prettier#19725)
- Pin: `conformance/fixtures/edge-cases/css-in-js/template-expression-indent.js` (also `gql-in-js` / `html-in-js` siblings)
- Drop when: the pin catches up

```js
/* input */
_ = gql`
  ${
                    a +
                    // comment
                    b
                  }
`;

/* ours */
_ = gql`
  ${
    a +
    // comment
    b
  }
`;

/* prettier */
_ = gql`
  ${
                    a +
                    // comment
                    b
                  }
`;
```

A broken `${expr}` inside an embedded template re-indents to the placeholder's position;
Prettier 3.9.6 preserves the source indentation non-idempotently (fixed upstream by prettier#19725).

## broken-template-comment-indent

- Why: prettier-bug
- Pin: `conformance/fixtures/edge-cases/xxx-in-js-comment/broken-template-comment-indent.js` (also tracked by conformance `externals/prettier/js/multiparser-comments/comment-inside.js`)

```js
/* input */
html`
  ${
  foo
  /* comment */
}
`;

/* ours */
html`
  ${
    foo
    /* comment */
  }
`;

/* prettier */
html`
  ${
  foo
  /* comment */
}
`;
```

A `${}` whose embed formatting bails (comments force the broken form) still indents its expression to the
placeholder, same as `template-expression-indent`; Prettier prints it at ROOT indent, dropping the embed
indent entirely (an artifact of its embed bail-out path).

## ts-in-vue-generic-trailing-comma

- Why: uniform-rule
- Pin: `conformance/fixtures/edge-cases/js-in-vue/generic-trailing-comma.vue` (also tracked by conformance `externals/vue-vben-admin/.../api-component/api-component.vue`)

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
in ts-in-md and plain `.ts` — one rule over that internal inconsistency.

## styled-extend-tag

- Why: cost
- Pin: `conformance/fixtures/edge-cases/css-in-js/styled-extend-tag.js` (also tracked by conformance `externals/prettier/js/multiparser-css/styled-components.js`)

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
