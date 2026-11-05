# edge-cases/html/script-end-in-gql-template.html

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -1,7 +1,7 @@
 <script>
   const query = gql`
     query {
-      field # </script>
+      field # <\/script>
     }
   `;
 </script>

`````

### Actual (oxfmt)

`````html
<script>
  const query = gql`
    query {
      field # <\/script>
    }
  `;
</script>

`````

### Expected (prettier)

`````html
<script>
  const query = gql`
    query {
      field # </script>
    }
  `;
</script>

`````

## Option 2

`````json
{"printWidth":100,"htmlWhitespaceSensitivity":"ignore","singleAttributePerLine":true}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -1,7 +1,7 @@
 <script>
   const query = gql`
     query {
-      field # </script>
+      field # <\/script>
     }
   `;
 </script>

`````

### Actual (oxfmt)

`````html
<script>
  const query = gql`
    query {
      field # <\/script>
    }
  `;
</script>

`````

### Expected (prettier)

`````html
<script>
  const query = gql`
    query {
      field # </script>
    }
  `;
</script>

`````

## Option 3

`````json
{"printWidth":120,"htmlWhitespaceSensitivity":"strict","bracketSameLine":true}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -1,7 +1,7 @@
 <script>
   const query = gql`
     query {
-      field # </script>
+      field # <\/script>
     }
   `;
 </script>

`````

### Actual (oxfmt)

`````html
<script>
  const query = gql`
    query {
      field # <\/script>
    }
  `;
</script>

`````

### Expected (prettier)

`````html
<script>
  const query = gql`
    query {
      field # </script>
    }
  `;
</script>

`````
