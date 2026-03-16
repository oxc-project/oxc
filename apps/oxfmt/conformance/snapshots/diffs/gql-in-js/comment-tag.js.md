# comment-tag.js

> `/* GraphQL */` comment tag not yet supported

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -1,17 +1,18 @@
 const query = /* GraphQL */ `
-  {
-    user(id: 5) {
+      {
+    user(   id :   5  )  {
       firstName
 
       lastName
     }
   }
 `;
 
 /* GraphQL */ `
-  {
-    user(id: 5, type: "without variable assignment") {
+      {
+    user(   id :   5 , type:
+    "without variable assignment"  )  {
       firstName
 
       lastName
     }

`````

### Actual (oxfmt)

`````js
const query = /* GraphQL */ `
      {
    user(   id :   5  )  {
      firstName

      lastName
    }
  }
`;

/* GraphQL */ `
      {
    user(   id :   5 , type:
    "without variable assignment"  )  {
      firstName

      lastName
    }
  }
`;

`````

### Expected (prettier)

`````js
const query = /* GraphQL */ `
  {
    user(id: 5) {
      firstName

      lastName
    }
  }
`;

/* GraphQL */ `
  {
    user(id: 5, type: "without variable assignment") {
      firstName

      lastName
    }
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
@@ -1,17 +1,18 @@
 const query = /* GraphQL */ `
-  {
-    user(id: 5) {
+      {
+    user(   id :   5  )  {
       firstName
 
       lastName
     }
   }
 `;
 
 /* GraphQL */ `
-  {
-    user(id: 5, type: "without variable assignment") {
+      {
+    user(   id :   5 , type:
+    "without variable assignment"  )  {
       firstName
 
       lastName
     }

`````

### Actual (oxfmt)

`````js
const query = /* GraphQL */ `
      {
    user(   id :   5  )  {
      firstName

      lastName
    }
  }
`;

/* GraphQL */ `
      {
    user(   id :   5 , type:
    "without variable assignment"  )  {
      firstName

      lastName
    }
  }
`;

`````

### Expected (prettier)

`````js
const query = /* GraphQL */ `
  {
    user(id: 5) {
      firstName

      lastName
    }
  }
`;

/* GraphQL */ `
  {
    user(id: 5, type: "without variable assignment") {
      firstName

      lastName
    }
  }
`;

`````
