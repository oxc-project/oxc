# html-template-literals.js

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -4,9 +4,9 @@
   </script>`;
 
 const nestedFun2 = /* HTML */ `${outerExpr(1)}
   <script>
-    const tpl = html\` <div>\${innerExpr(1)} ${outerExpr(2)}</div> \`;
+    const tpl = html\`\\n<div>\${innerExpr(1)} ${outerExpr(2)}</div>\\n\`;
   </script>`;
 
 setFoo(
   html`<div>one</div>
@@ -32,9 +32,13 @@
 );
 
 // Attribute quotes
 a = /* HTML */ `<div
-  double-quoted="${foo}"
-  single-quoted="${foo}"
-  unquoted=${foo}
-></div> `;
-a = /* HTML */ `<div style="${foo}" style="${foo}" style=${foo}></div> `;
+    double-quoted="${foo}"
+single-quoted='${foo}'
+        unquoted=${foo}>   </div>
+`;
+a = /* HTML */ `<div
+    style="${foo}"
+style='${foo}'
+        style=${foo}>   </div>
+`;

`````

### Actual (oxfmt)

`````js
const nestedFun = /* HTML */ `${outerExpr(1)}
  <script>
    const tpl = html\`<div>\${innerExpr(1)} ${outerExpr(2)}</div>\`;
  </script>`;

const nestedFun2 = /* HTML */ `${outerExpr(1)}
  <script>
    const tpl = html\`\\n<div>\${innerExpr(1)} ${outerExpr(2)}</div>\\n\`;
  </script>`;

setFoo(
  html`<div>one</div>
    <div>two</div>
    <div>three</div>`,
  secondArgument,
);

setFoo(
  html`<div>
      <div>nested</div>
    </div>
    <div>two</div>
    <div>three</div>`,
  secondArgument,
);

setFoo(
  html`<div>
    <div>nested</div>
  </div>`,
  secondArgument,
);

// Attribute quotes
a = /* HTML */ `<div
    double-quoted="${foo}"
single-quoted='${foo}'
        unquoted=${foo}>   </div>
`;
a = /* HTML */ `<div
    style="${foo}"
style='${foo}'
        style=${foo}>   </div>
`;

`````

### Expected (prettier)

`````js
const nestedFun = /* HTML */ `${outerExpr(1)}
  <script>
    const tpl = html\`<div>\${innerExpr(1)} ${outerExpr(2)}</div>\`;
  </script>`;

const nestedFun2 = /* HTML */ `${outerExpr(1)}
  <script>
    const tpl = html\` <div>\${innerExpr(1)} ${outerExpr(2)}</div> \`;
  </script>`;

setFoo(
  html`<div>one</div>
    <div>two</div>
    <div>three</div>`,
  secondArgument,
);

setFoo(
  html`<div>
      <div>nested</div>
    </div>
    <div>two</div>
    <div>three</div>`,
  secondArgument,
);

setFoo(
  html`<div>
    <div>nested</div>
  </div>`,
  secondArgument,
);

// Attribute quotes
a = /* HTML */ `<div
  double-quoted="${foo}"
  single-quoted="${foo}"
  unquoted=${foo}
></div> `;
a = /* HTML */ `<div style="${foo}" style="${foo}" style=${foo}></div> `;

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
@@ -1,21 +1,13 @@
-const nestedFun = /* HTML */ `
-  ${outerExpr(1)}
+const nestedFun = /* HTML */ `${outerExpr(1)}
   <script>
-    const tpl = html\`
-      <div>\${innerExpr(1)} ${outerExpr(2)}</div>
-    \`;
-  </script>
-`;
+    const tpl = html\`<div>\${innerExpr(1)} ${outerExpr(2)}</div>\`;
+  </script>`;
 
-const nestedFun2 = /* HTML */ `
-  ${outerExpr(1)}
+const nestedFun2 = /* HTML */ `${outerExpr(1)}
   <script>
-    const tpl = html\`
-      <div>\${innerExpr(1)} ${outerExpr(2)}</div>
-    \`;
-  </script>
-`;
+    const tpl = html\`\\n<div>\${innerExpr(1)} ${outerExpr(2)}</div>\\n\`;
+  </script>`;
 
 setFoo(
   html`
     <div>one</div>
@@ -45,10 +37,14 @@
   secondArgument,
 );
 
 // Attribute quotes
-a = /* HTML */ `
-  <div double-quoted="${foo}" single-quoted="${foo}" unquoted=${foo}></div>
+a = /* HTML */ `<div
+    double-quoted="${foo}"
+single-quoted='${foo}'
+        unquoted=${foo}>   </div>
 `;
-a = /* HTML */ `
-  <div style="${foo}" style="${foo}" style=${foo}></div>
+a = /* HTML */ `<div
+    style="${foo}"
+style='${foo}'
+        style=${foo}>   </div>
 `;

`````

### Actual (oxfmt)

`````js
const nestedFun = /* HTML */ `${outerExpr(1)}
  <script>
    const tpl = html\`<div>\${innerExpr(1)} ${outerExpr(2)}</div>\`;
  </script>`;

const nestedFun2 = /* HTML */ `${outerExpr(1)}
  <script>
    const tpl = html\`\\n<div>\${innerExpr(1)} ${outerExpr(2)}</div>\\n\`;
  </script>`;

setFoo(
  html`
    <div>one</div>
    <div>two</div>
    <div>three</div>
  `,
  secondArgument,
);

setFoo(
  html`
    <div>
      <div>nested</div>
    </div>
    <div>two</div>
    <div>three</div>
  `,
  secondArgument,
);

setFoo(
  html`
    <div>
      <div>nested</div>
    </div>
  `,
  secondArgument,
);

// Attribute quotes
a = /* HTML */ `<div
    double-quoted="${foo}"
single-quoted='${foo}'
        unquoted=${foo}>   </div>
`;
a = /* HTML */ `<div
    style="${foo}"
style='${foo}'
        style=${foo}>   </div>
`;

`````

### Expected (prettier)

`````js
const nestedFun = /* HTML */ `
  ${outerExpr(1)}
  <script>
    const tpl = html\`
      <div>\${innerExpr(1)} ${outerExpr(2)}</div>
    \`;
  </script>
`;

const nestedFun2 = /* HTML */ `
  ${outerExpr(1)}
  <script>
    const tpl = html\`
      <div>\${innerExpr(1)} ${outerExpr(2)}</div>
    \`;
  </script>
`;

setFoo(
  html`
    <div>one</div>
    <div>two</div>
    <div>three</div>
  `,
  secondArgument,
);

setFoo(
  html`
    <div>
      <div>nested</div>
    </div>
    <div>two</div>
    <div>three</div>
  `,
  secondArgument,
);

setFoo(
  html`
    <div>
      <div>nested</div>
    </div>
  `,
  secondArgument,
);

// Attribute quotes
a = /* HTML */ `
  <div double-quoted="${foo}" single-quoted="${foo}" unquoted=${foo}></div>
`;
a = /* HTML */ `
  <div style="${foo}" style="${foo}" style=${foo}></div>
`;

`````
