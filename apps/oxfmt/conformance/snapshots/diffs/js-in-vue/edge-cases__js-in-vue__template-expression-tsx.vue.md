# edge-cases/js-in-vue/template-expression-tsx.vue

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -2,6 +2,6 @@
 const a = <div />;
 </script>
 
 <template>
-  <p>{{ (value  as  Item).name }}</p>
+  <p>{{ (value as Item).name }}</p>
 </template>

`````

### Actual (oxfmt)

`````vue
<script setup lang="tsx">
const a = <div />;
</script>

<template>
  <p>{{ (value as Item).name }}</p>
</template>

`````

### Expected (prettier)

`````vue
<script setup lang="tsx">
const a = <div />;
</script>

<template>
  <p>{{ (value  as  Item).name }}</p>
</template>

`````

## Option 2

`````json
{"printWidth":100,"vueIndentScriptAndStyle":true,"singleQuote":true,"semi":false}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -2,6 +2,6 @@
   const a = <div />
 </script>
 
 <template>
-  <p>{{ (value  as  Item).name }}</p>
+  <p>{{ (value as Item).name }}</p>
 </template>

`````

### Actual (oxfmt)

`````vue
<script setup lang="tsx">
  const a = <div />
</script>

<template>
  <p>{{ (value as Item).name }}</p>
</template>

`````

### Expected (prettier)

`````vue
<script setup lang="tsx">
  const a = <div />
</script>

<template>
  <p>{{ (value  as  Item).name }}</p>
</template>

`````
