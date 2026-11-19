# Input

`````js
html`<script>f((${a}) /* c */)</script>`;
html`<script>f((${a}))</script>`;
html`<script>f(/* c */ (${a}))</script>`;

`````

# Output

## `{"printWidth":80}`

`````js
html`<script>
  f((${a}) /* c */);
</script>`;
html`<script>
  f((${a}));
</script>`;
html`<script>
  f(/* c */ (${a}));
</script>`;

`````

## `{"printWidth":100,"htmlWhitespaceSensitivity":"ignore"}`

`````js
html`
  <script>
    f((${a}) /* c */);
  </script>
`;
html`
  <script>
    f((${a}));
  </script>
`;
html`
  <script>
    f(/* c */ (${a}));
  </script>
`;

`````
