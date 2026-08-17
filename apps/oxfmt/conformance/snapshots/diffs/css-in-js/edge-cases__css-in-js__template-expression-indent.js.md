# edge-cases/css-in-js/template-expression-indent.js

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
@@ -1,10 +1,10 @@
 // prettier/prettier#19725: embedded `${expr}` no longer preserves source indentation
 _ = css`
   a {
     color: ${
-                    a +
-                    // comment
-                    b
-                  };
+      a +
+      // comment
+      b
+    };
   }
 `;

`````

### Actual (oxfmt)

`````js
// prettier/prettier#19725: embedded `${expr}` no longer preserves source indentation
_ = css`
  a {
    color: ${
      a +
      // comment
      b
    };
  }
`;

`````

### Expected (prettier)

`````js
// prettier/prettier#19725: embedded `${expr}` no longer preserves source indentation
_ = css`
  a {
    color: ${
                    a +
                    // comment
                    b
                  };
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
@@ -1,10 +1,10 @@
 // prettier/prettier#19725: embedded `${expr}` no longer preserves source indentation
 _ = css`
   a {
     color: ${
-                    a +
-                    // comment
-                    b
-                  };
+      a +
+      // comment
+      b
+    };
   }
 `;

`````

### Actual (oxfmt)

`````js
// prettier/prettier#19725: embedded `${expr}` no longer preserves source indentation
_ = css`
  a {
    color: ${
      a +
      // comment
      b
    };
  }
`;

`````

### Expected (prettier)

`````js
// prettier/prettier#19725: embedded `${expr}` no longer preserves source indentation
_ = css`
  a {
    color: ${
                    a +
                    // comment
                    b
                  };
  }
`;

`````
