# edge-cases/gql-in-js/template-expression-indent.js

> embedded `${expr}` re-indents to the placeholder. See apps/oxfmt/DIVERGENCES.md#template-expression-indent

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -1,8 +1,8 @@
 // prettier/prettier#19725: embedded `${expr}` no longer preserves source indentation
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
// prettier/prettier#19725: embedded `${expr}` no longer preserves source indentation
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
// prettier/prettier#19725: embedded `${expr}` no longer preserves source indentation
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
@@ -1,8 +1,8 @@
 // prettier/prettier#19725: embedded `${expr}` no longer preserves source indentation
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
// prettier/prettier#19725: embedded `${expr}` no longer preserves source indentation
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
// prettier/prettier#19725: embedded `${expr}` no longer preserves source indentation
_ = gql`
  ${
                    a +
                    // comment
                    b
                  }
`;

`````
