# Input

`````js
html`<${For} each=${() => items}><li class="item">${() => content}</li><li class="item">more items</li></${For}>`;
html`<${Layout}><${Header} title="a long title attribute" /><main class="content">main content</main><//>`;

`````

# Output

## `{"printWidth":80}`

`````js
html`<${For} each=${() => items}
  ><li class="item">${() => content}</li>
  <li class="item">more items</li></${For}
>`;
html`<${Layout}
  ><${Header} title="a long title attribute" />
  <main class="content">main content</main><//
>`;

`````

## `{"printWidth":100,"htmlWhitespaceSensitivity":"ignore"}`

`````js
html`
  <${For} each=${() => items}>
    <li class="item">${() => content}</li>
    <li class="item">more items</li>
  </${For}>
`;
html`
  <${Layout}>
    <${Header} title="a long title attribute" />
    <main class="content">main content</main>
  <//>
`;

`````
