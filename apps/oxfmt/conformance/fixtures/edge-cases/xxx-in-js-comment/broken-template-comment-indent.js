// DIVERGES: broken `${}` holding comments indents to the placeholder;
// see apps/oxfmt/DIVERGENCES.md "broken-template-comment-indent"
html`
${
      foo
  /* comment */
}
`;
