# edge-cases/js-in-vue/removed-attributes-as-written.vue

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -1,5 +1,5 @@
-<script setup="{ row }"></script>
-<style vars="{ color }"></style>
+<script setup="  {row}  "></script>
+<style vars="  {color}  "></style>
 <template>
-  <C slot-scope="{ row }">x</C>
+  <C slot-scope="  {row}  ">x</C>
 </template>

`````

### Actual (oxfmt)

`````vue
<script setup="  {row}  "></script>
<style vars="  {color}  "></style>
<template>
  <C slot-scope="  {row}  ">x</C>
</template>

`````

### Expected (prettier)

`````vue
<script setup="{ row }"></script>
<style vars="{ color }"></style>
<template>
  <C slot-scope="{ row }">x</C>
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
@@ -1,5 +1,5 @@
-<script setup="{ row }"></script>
-<style vars="{ color }"></style>
+<script setup="  {row}  "></script>
+<style vars="  {color}  "></style>
 <template>
-  <C slot-scope="{ row }">x</C>
+  <C slot-scope="  {row}  ">x</C>
 </template>

`````

### Actual (oxfmt)

`````vue
<script setup="  {row}  "></script>
<style vars="  {color}  "></style>
<template>
  <C slot-scope="  {row}  ">x</C>
</template>

`````

### Expected (prettier)

`````vue
<script setup="{ row }"></script>
<style vars="{ color }"></style>
<template>
  <C slot-scope="{ row }">x</C>
</template>

`````
