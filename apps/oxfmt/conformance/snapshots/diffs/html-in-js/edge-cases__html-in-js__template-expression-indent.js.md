# edge-cases/html-in-js/template-expression-indent.js

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -2,11 +2,11 @@
 // see apps/oxfmt/DIVERGENCES.md#template-expression-indent
 _ = html`
   <div>
     ${
-                        a + //
-                        b
-                      }
+      a + //
+      b
+    }
   </div>
 `;
 
 // prettier/prettier#19518: nested embeds were not idempotent
@@ -19,10 +19,10 @@
             entry.children
               ? html`
                   <ol>
                     ${entry.children.map(
-                    (child) => html`<li>${child.title}</li>`,
-                  )}
+                      (child) => html`<li>${child.title}</li>`,
+                    )}
                   </ol>
                 `
               : entry.title
           }
@@ -36,16 +36,16 @@
   return html`
     <div>
       <pre>
 ${JSON.stringify({
-                a: 1,
-                b: 2,
-              })}</pre>
+          a: 1,
+          b: 2,
+        })}</pre>
     </div>
   `;
 }
 
 const a = html`
   ${{
-            c: y,
-          }}
+    c: y,
+  }}
 `;

`````

### Actual (oxfmt)

`````js
// DIVERGES: embedded `${expr}` re-indents to the placeholder (Prettier preserves the source indentation);
// see apps/oxfmt/DIVERGENCES.md#template-expression-indent
_ = html`
  <div>
    ${
      a + //
      b
    }
  </div>
`;

// prettier/prettier#19518: nested embeds were not idempotent
const t = html`
  <ol>
    ${items.map(
      (entry) => html`
        <li>
          ${
            entry.children
              ? html`
                  <ol>
                    ${entry.children.map(
                      (child) => html`<li>${child.title}</li>`,
                    )}
                  </ol>
                `
              : entry.title
          }
        </li>
      `,
    )}
  </ol>
`;

export function foo() {
  return html`
    <div>
      <pre>
${JSON.stringify({
          a: 1,
          b: 2,
        })}</pre>
    </div>
  `;
}

const a = html`
  ${{
    c: y,
  }}
`;

`````

### Expected (prettier)

`````js
// DIVERGES: embedded `${expr}` re-indents to the placeholder (Prettier preserves the source indentation);
// see apps/oxfmt/DIVERGENCES.md#template-expression-indent
_ = html`
  <div>
    ${
                        a + //
                        b
                      }
  </div>
`;

// prettier/prettier#19518: nested embeds were not idempotent
const t = html`
  <ol>
    ${items.map(
      (entry) => html`
        <li>
          ${
            entry.children
              ? html`
                  <ol>
                    ${entry.children.map(
                    (child) => html`<li>${child.title}</li>`,
                  )}
                  </ol>
                `
              : entry.title
          }
        </li>
      `,
    )}
  </ol>
`;

export function foo() {
  return html`
    <div>
      <pre>
${JSON.stringify({
                a: 1,
                b: 2,
              })}</pre>
    </div>
  `;
}

const a = html`
  ${{
            c: y,
          }}
`;

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
@@ -2,11 +2,11 @@
 // see apps/oxfmt/DIVERGENCES.md#template-expression-indent
 _ = html`
   <div>
     ${
-                        a + //
-                        b
-                      }
+      a + //
+      b
+    }
   </div>
 `;
 
 // prettier/prettier#19518: nested embeds were not idempotent
@@ -19,12 +19,12 @@
             entry.children
               ? html`
                   <ol>
                     ${entry.children.map(
-                    (child) => html`
-                      <li>${child.title}</li>
-                    `,
-                  )}
+                      (child) => html`
+                        <li>${child.title}</li>
+                      `,
+                    )}
                   </ol>
                 `
               : entry.title
           }
@@ -38,16 +38,16 @@
   return html`
     <div>
       <pre>
 ${JSON.stringify({
-                a: 1,
-                b: 2,
-              })}</pre>
+          a: 1,
+          b: 2,
+        })}</pre>
     </div>
   `;
 }
 
 const a = html`
   ${{
-            c: y,
-          }}
+    c: y,
+  }}
 `;

`````

### Actual (oxfmt)

`````js
// DIVERGES: embedded `${expr}` re-indents to the placeholder (Prettier preserves the source indentation);
// see apps/oxfmt/DIVERGENCES.md#template-expression-indent
_ = html`
  <div>
    ${
      a + //
      b
    }
  </div>
`;

// prettier/prettier#19518: nested embeds were not idempotent
const t = html`
  <ol>
    ${items.map(
      (entry) => html`
        <li>
          ${
            entry.children
              ? html`
                  <ol>
                    ${entry.children.map(
                      (child) => html`
                        <li>${child.title}</li>
                      `,
                    )}
                  </ol>
                `
              : entry.title
          }
        </li>
      `,
    )}
  </ol>
`;

export function foo() {
  return html`
    <div>
      <pre>
${JSON.stringify({
          a: 1,
          b: 2,
        })}</pre>
    </div>
  `;
}

const a = html`
  ${{
    c: y,
  }}
`;

`````

### Expected (prettier)

`````js
// DIVERGES: embedded `${expr}` re-indents to the placeholder (Prettier preserves the source indentation);
// see apps/oxfmt/DIVERGENCES.md#template-expression-indent
_ = html`
  <div>
    ${
                        a + //
                        b
                      }
  </div>
`;

// prettier/prettier#19518: nested embeds were not idempotent
const t = html`
  <ol>
    ${items.map(
      (entry) => html`
        <li>
          ${
            entry.children
              ? html`
                  <ol>
                    ${entry.children.map(
                    (child) => html`
                      <li>${child.title}</li>
                    `,
                  )}
                  </ol>
                `
              : entry.title
          }
        </li>
      `,
    )}
  </ol>
`;

export function foo() {
  return html`
    <div>
      <pre>
${JSON.stringify({
                a: 1,
                b: 2,
              })}</pre>
    </div>
  `;
}

const a = html`
  ${{
            c: y,
          }}
`;

`````
