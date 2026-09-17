# edge-cases/gql-in-js/template-expression-indent.js

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -1,9 +1,9 @@
 // DIVERGES: embedded `${expr}` re-indents to the placeholder (Prettier preserves the source indentation);
 // see apps/oxfmt/DIVERGENCES.md#template-expression-indent
 _ = gql`
   ${
-                    a +
-                    // comment
-                    b
-                  }
+    a +
+    // comment
+    b
+  }
 `;

`````

### Actual (oxfmt)

`````js
// DIVERGES: embedded `${expr}` re-indents to the placeholder (Prettier preserves the source indentation);
// see apps/oxfmt/DIVERGENCES.md#template-expression-indent
_ = gql`
  ${
    a +
    // comment
    b
  }
`;

`````

### Expected (prettier)

`````js
// DIVERGES: embedded `${expr}` re-indents to the placeholder (Prettier preserves the source indentation);
// see apps/oxfmt/DIVERGENCES.md#template-expression-indent
_ = gql`
  ${
                    a +
                    // comment
                    b
                  }
`;

`````

## Option 2

`````json
{"printWidth":100}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -1,9 +1,9 @@
 // DIVERGES: embedded `${expr}` re-indents to the placeholder (Prettier preserves the source indentation);
 // see apps/oxfmt/DIVERGENCES.md#template-expression-indent
 _ = gql`
   ${
-                    a +
-                    // comment
-                    b
-                  }
+    a +
+    // comment
+    b
+  }
 `;

`````

### Actual (oxfmt)

`````js
// DIVERGES: embedded `${expr}` re-indents to the placeholder (Prettier preserves the source indentation);
// see apps/oxfmt/DIVERGENCES.md#template-expression-indent
_ = gql`
  ${
    a +
    // comment
    b
  }
`;

`````

### Expected (prettier)

`````js
// DIVERGES: embedded `${expr}` re-indents to the placeholder (Prettier preserves the source indentation);
// see apps/oxfmt/DIVERGENCES.md#template-expression-indent
_ = gql`
  ${
                    a +
                    // comment
                    b
                  }
`;

`````
