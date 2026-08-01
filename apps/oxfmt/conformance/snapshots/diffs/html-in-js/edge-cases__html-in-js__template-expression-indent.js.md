# edge-cases/html-in-js/template-expression-indent.js

> We match Prettier main (prettier/prettier#19725); 3.9.6 still preserves source indent non-idempotently

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -1,11 +1,11 @@
 // prettier/prettier#19725: embedded `${expr}` no longer preserves source indentation
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
@@ -18,10 +18,10 @@
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
@@ -35,16 +35,16 @@
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
// prettier/prettier#19725: embedded `${expr}` no longer preserves source indentation
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
// prettier/prettier#19725: embedded `${expr}` no longer preserves source indentation
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
@@ -1,11 +1,11 @@
 // prettier/prettier#19725: embedded `${expr}` no longer preserves source indentation
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
@@ -18,12 +18,12 @@
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
@@ -37,16 +37,16 @@
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
// prettier/prettier#19725: embedded `${expr}` no longer preserves source indentation
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
// prettier/prettier#19725: embedded `${expr}` no longer preserves source indentation
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
