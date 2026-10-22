# externals/prettier/js/embeded/indention/19518.js

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -24,11 +24,11 @@
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
@@ -26,11 +26,11 @@
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
