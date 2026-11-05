# edge-cases/html-in-js/pre-like-template-expression.js

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -4,12 +4,12 @@
     <body>
       <form>
         <textarea>
 ${Match.valueTags(form, {
-            EmptyForm: () => "",
-            InvalidForm: () => "",
-            CompletedForm: () => "",
-          })}</textarea>
+  EmptyForm: () => "",
+  InvalidForm: () => "",
+  CompletedForm: () => "",
+})}</textarea>
       </form>
     </body>
   </html>
 `;
@@ -17,17 +17,17 @@
 const indented = html`
   <form>
     <pre>
   ${foo({
-        x: 1,
-      })}</pre>
+    x: 1,
+  })}</pre>
   </form>
 `;
 
 const afterText = html`<pre>
 a ${foo({
-    x: 1,
-  })} b</pre>`;
+  x: 1,
+})} b</pre>`;
 
 const sameLine = html`<pre>
   ${a({
     x: 1,

`````

### Actual (oxfmt)

`````js
// prettier/prettier#20355
html`
  <html>
    <body>
      <form>
        <textarea>
${Match.valueTags(form, {
  EmptyForm: () => "",
  InvalidForm: () => "",
  CompletedForm: () => "",
})}</textarea>
      </form>
    </body>
  </html>
`;

const indented = html`
  <form>
    <pre>
  ${foo({
    x: 1,
  })}</pre>
  </form>
`;

const afterText = html`<pre>
a ${foo({
  x: 1,
})} b</pre>`;

const sameLine = html`<pre>
  ${a({
    x: 1,
  })} ${b({
    y: 2,
  })}</pre>`;

const afterElement = html`<div>
  <pre><b>x</b>${foo({
    x: 1,
  })}</pre>
</div>`;

`````

### Expected (prettier)

`````js
// prettier/prettier#20355
html`
  <html>
    <body>
      <form>
        <textarea>
${Match.valueTags(form, {
            EmptyForm: () => "",
            InvalidForm: () => "",
            CompletedForm: () => "",
          })}</textarea>
      </form>
    </body>
  </html>
`;

const indented = html`
  <form>
    <pre>
  ${foo({
        x: 1,
      })}</pre>
  </form>
`;

const afterText = html`<pre>
a ${foo({
    x: 1,
  })} b</pre>`;

const sameLine = html`<pre>
  ${a({
    x: 1,
  })} ${b({
    y: 2,
  })}</pre>`;

const afterElement = html`<div>
  <pre><b>x</b>${foo({
    x: 1,
  })}</pre>
</div>`;

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
@@ -4,12 +4,12 @@
     <body>
       <form>
         <textarea>
 ${Match.valueTags(form, {
-            EmptyForm: () => "",
-            InvalidForm: () => "",
-            CompletedForm: () => "",
-          })}</textarea>
+  EmptyForm: () => "",
+  InvalidForm: () => "",
+  CompletedForm: () => "",
+})}</textarea>
       </form>
     </body>
   </html>
 `;
@@ -17,27 +17,27 @@
 const indented = html`
   <form>
     <pre>
   ${foo({
-        x: 1,
-      })}</pre>
+    x: 1,
+  })}</pre>
   </form>
 `;
 
 const afterText = html`
   <pre>
 a ${foo({
-      x: 1,
-    })} b</pre>
+  x: 1,
+})} b</pre>
 `;
 
 const sameLine = html`
   <pre>
   ${a({
-      x: 1,
-    })} ${b({
-      y: 2,
-    })}</pre>
+    x: 1,
+  })} ${b({
+    y: 2,
+  })}</pre>
 `;
 
 const afterElement = html`
   <div>

`````

### Actual (oxfmt)

`````js
// prettier/prettier#20355
html`
  <html>
    <body>
      <form>
        <textarea>
${Match.valueTags(form, {
  EmptyForm: () => "",
  InvalidForm: () => "",
  CompletedForm: () => "",
})}</textarea>
      </form>
    </body>
  </html>
`;

const indented = html`
  <form>
    <pre>
  ${foo({
    x: 1,
  })}</pre>
  </form>
`;

const afterText = html`
  <pre>
a ${foo({
  x: 1,
})} b</pre>
`;

const sameLine = html`
  <pre>
  ${a({
    x: 1,
  })} ${b({
    y: 2,
  })}</pre>
`;

const afterElement = html`
  <div>
    <pre><b>x</b>${foo({
      x: 1,
    })}</pre>
  </div>
`;

`````

### Expected (prettier)

`````js
// prettier/prettier#20355
html`
  <html>
    <body>
      <form>
        <textarea>
${Match.valueTags(form, {
            EmptyForm: () => "",
            InvalidForm: () => "",
            CompletedForm: () => "",
          })}</textarea>
      </form>
    </body>
  </html>
`;

const indented = html`
  <form>
    <pre>
  ${foo({
        x: 1,
      })}</pre>
  </form>
`;

const afterText = html`
  <pre>
a ${foo({
      x: 1,
    })} b</pre>
`;

const sameLine = html`
  <pre>
  ${a({
      x: 1,
    })} ${b({
      y: 2,
    })}</pre>
`;

const afterElement = html`
  <div>
    <pre><b>x</b>${foo({
      x: 1,
    })}</pre>
  </div>
`;

`````
