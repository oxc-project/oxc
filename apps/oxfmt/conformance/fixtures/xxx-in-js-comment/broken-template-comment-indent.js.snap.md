# Input

`````js
// DIVERGES: broken `${}` holding comments indents to the placeholder;
// see apps/oxfmt/DIVERGENCES.md#broken-template-comment-indent
html`
${
      foo
  /* comment */
}
`;

`````

# Output

## `{"printWidth":80}`

`````js
// DIVERGES: broken `${}` holding comments indents to the placeholder;
// see apps/oxfmt/DIVERGENCES.md#broken-template-comment-indent
html`
  ${
    foo
    /* comment */
  }
`;

`````

## `{"printWidth":100}`

`````js
// DIVERGES: broken `${}` holding comments indents to the placeholder;
// see apps/oxfmt/DIVERGENCES.md#broken-template-comment-indent
html`
  ${
    foo
    /* comment */
  }
`;

`````
