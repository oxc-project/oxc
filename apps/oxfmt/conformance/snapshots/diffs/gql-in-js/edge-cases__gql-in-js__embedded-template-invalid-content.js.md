# edge-cases/gql-in-js/embedded-template-invalid-content.js

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -1,5 +1,3 @@
-foo(
-  gql`
+foo(gql`
     query {{{
-  `,
-);
+  `);

`````

### Actual (oxfmt)

`````js
foo(gql`
    query {{{
  `);

`````

### Expected (prettier)

`````js
foo(
  gql`
    query {{{
  `,
);

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
@@ -1,5 +1,3 @@
-foo(
-  gql`
+foo(gql`
     query {{{
-  `,
-);
+  `);

`````

### Actual (oxfmt)

`````js
foo(gql`
    query {{{
  `);

`````

### Expected (prettier)

`````js
foo(
  gql`
    query {{{
  `,
);

`````
