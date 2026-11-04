# edge-cases/gql-in-js/embedded-template-short-argument.js

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -1,8 +1,7 @@
-const schema =
-  graphql(`
-    query {
-      users {
-        x
-      }
+const schema = graphql(`
+  query {
+    users {
+      x
     }
-  `);
+  }
+`);

`````

### Actual (oxfmt)

`````js
const schema = graphql(`
  query {
    users {
      x
    }
  }
`);

`````

### Expected (prettier)

`````js
const schema =
  graphql(`
    query {
      users {
        x
      }
    }
  `);

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
@@ -1,8 +1,7 @@
-const schema =
-  graphql(`
-    query {
-      users {
-        x
-      }
+const schema = graphql(`
+  query {
+    users {
+      x
     }
-  `);
+  }
+`);

`````

### Actual (oxfmt)

`````js
const schema = graphql(`
  query {
    users {
      x
    }
  }
`);

`````

### Expected (prettier)

`````js
const schema =
  graphql(`
    query {
      users {
        x
      }
    }
  `);

`````
