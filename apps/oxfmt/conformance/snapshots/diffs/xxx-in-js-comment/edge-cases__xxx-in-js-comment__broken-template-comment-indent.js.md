# edge-cases/xxx-in-js-comment/broken-template-comment-indent.js

> broken `${}` holding comments indents to the placeholder. See apps/oxfmt/DIVERGENCES.md#broken-template-comment-indent

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
 // DIVERGES: broken `${}` holding comments indents to the placeholder;
 // see apps/oxfmt/DIVERGENCES.md "broken-template-comment-indent"
 html`
   ${
-  foo
-  /* comment */
-}
+    foo
+    /* comment */
+  }
 `;

`````

### Actual (oxfmt)

`````js
// DIVERGES: broken `${}` holding comments indents to the placeholder;
// see apps/oxfmt/DIVERGENCES.md "broken-template-comment-indent"
html`
  ${
    foo
    /* comment */
  }
`;

`````

### Expected (prettier)

`````js
// DIVERGES: broken `${}` holding comments indents to the placeholder;
// see apps/oxfmt/DIVERGENCES.md "broken-template-comment-indent"
html`
  ${
  foo
  /* comment */
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
 // DIVERGES: broken `${}` holding comments indents to the placeholder;
 // see apps/oxfmt/DIVERGENCES.md "broken-template-comment-indent"
 html`
   ${
-  foo
-  /* comment */
-}
+    foo
+    /* comment */
+  }
 `;

`````

### Actual (oxfmt)

`````js
// DIVERGES: broken `${}` holding comments indents to the placeholder;
// see apps/oxfmt/DIVERGENCES.md "broken-template-comment-indent"
html`
  ${
    foo
    /* comment */
  }
`;

`````

### Expected (prettier)

`````js
// DIVERGES: broken `${}` holding comments indents to the placeholder;
// see apps/oxfmt/DIVERGENCES.md "broken-template-comment-indent"
html`
  ${
  foo
  /* comment */
}
`;

`````
