# edge-cases/html-in-js/placeholder-case-in-child.js

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -1,7 +1,7 @@
 const a = html`<style>
   a {
-    prettier_html_placeholder_0_13_in_js: ${c};
+    ${b}: ${c};
   }
 </style>`;
 const d = html`<style>
     ${e} {

`````

### Actual (oxfmt)

`````js
const a = html`<style>
  a {
    ${b}: ${c};
  }
</style>`;
const d = html`<style>
    ${e} {
      color: red;
    }
  </style>
  <p>${f}</p>`;

`````

### Expected (prettier)

`````js
const a = html`<style>
  a {
    prettier_html_placeholder_0_13_in_js: ${c};
  }
</style>`;
const d = html`<style>
    ${e} {
      color: red;
    }
  </style>
  <p>${f}</p>`;

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
@@ -1,8 +1,8 @@
 const a = html`
   <style>
     a {
-      prettier_html_placeholder_0_2989_in_js: ${c};
+      ${b}: ${c};
     }
   </style>
 `;
 const d = html`

`````

### Actual (oxfmt)

`````js
const a = html`
  <style>
    a {
      ${b}: ${c};
    }
  </style>
`;
const d = html`
  <style>
    ${e} {
      color: red;
    }
  </style>
  <p>${f}</p>
`;

`````

### Expected (prettier)

`````js
const a = html`
  <style>
    a {
      prettier_html_placeholder_0_2989_in_js: ${c};
    }
  </style>
`;
const d = html`
  <style>
    ${e} {
      color: red;
    }
  </style>
  <p>${f}</p>
`;

`````
