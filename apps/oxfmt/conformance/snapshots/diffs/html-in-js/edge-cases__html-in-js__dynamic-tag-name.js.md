# edge-cases/html-in-js/dynamic-tag-name.js

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -1,5 +1,8 @@
-html`<${For} each=${() => items}><li class="item">${() => content}</li><li class="item">more items</li></${For}>`;
+html`<${For} each=${() => items}
+  ><li class="item">${() => content}</li>
+  <li class="item">more items</li></${For}
+>`;
 html`<${Layout}
   ><${Header} title="a long title attribute" />
   <main class="content">main content</main><//
 >`;

`````

### Actual (oxfmt)

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

### Expected (prettier)

`````js
html`<${For} each=${() => items}><li class="item">${() => content}</li><li class="item">more items</li></${For}>`;
html`<${Layout}
  ><${Header} title="a long title attribute" />
  <main class="content">main content</main><//
>`;

`````

## Option 2

`````json
{"printWidth":100,"htmlWhitespaceSensitivity":"ignore"}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -1,6 +1,11 @@
-html`<${For} each=${() => items}><li class="item">${() => content}</li><li class="item">more items</li></${For}>`;
 html`
+  <${For} each=${() => items}>
+    <li class="item">${() => content}</li>
+    <li class="item">more items</li>
+  </${For}>
+`;
+html`
   <${Layout}>
     <${Header} title="a long title attribute" />
     <main class="content">main content</main>
   <//>

`````

### Actual (oxfmt)

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

### Expected (prettier)

`````js
html`<${For} each=${() => items}><li class="item">${() => content}</li><li class="item">more items</li></${For}>`;
html`
  <${Layout}>
    <${Header} title="a long title attribute" />
    <main class="content">main content</main>
  <//>
`;

`````
