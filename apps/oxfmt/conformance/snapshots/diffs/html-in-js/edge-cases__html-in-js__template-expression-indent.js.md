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
@@ -34,11 +34,11 @@
   return html`
     <div>
       <pre>
 ${JSON.stringify({
-          a: 1,
-          b: 2,
-        })}</pre>
+  a: 1,
+  b: 2,
+})}</pre>
     </div>
   `;
 }
 

`````

### Actual (oxfmt)

`````js
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
@@ -36,11 +36,11 @@
   return html`
     <div>
       <pre>
 ${JSON.stringify({
-          a: 1,
-          b: 2,
-        })}</pre>
+  a: 1,
+  b: 2,
+})}</pre>
     </div>
   `;
 }
 

`````

### Actual (oxfmt)

`````js
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
