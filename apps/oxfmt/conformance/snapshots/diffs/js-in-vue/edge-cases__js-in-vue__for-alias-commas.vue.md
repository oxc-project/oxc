# edge-cases/js-in-vue/for-alias-commas.vue

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -1,4 +1,4 @@
 <template>
-  <li v-for="(item = 'a,b', i) in items">x</li>
-  <li v-for="(item /* a,b */, i) in items">x</li>
+  <li v-for="(item = 'a, b', i) in items">x</li>
+  <li v-for="(item /* a, b */, i) in items">x</li>
 </template>

`````

### Actual (oxfmt)

`````vue
<template>
  <li v-for="(item = 'a, b', i) in items">x</li>
  <li v-for="(item /* a, b */, i) in items">x</li>
</template>

`````

### Expected (prettier)

`````vue
<template>
  <li v-for="(item = 'a,b', i) in items">x</li>
  <li v-for="(item /* a,b */, i) in items">x</li>
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
@@ -1,4 +1,4 @@
 <template>
-  <li v-for="(item = 'a,b', i) in items">x</li>
-  <li v-for="(item /* a,b */, i) in items">x</li>
+  <li v-for="(item = 'a, b', i) in items">x</li>
+  <li v-for="(item /* a, b */, i) in items">x</li>
 </template>

`````

### Actual (oxfmt)

`````vue
<template>
  <li v-for="(item = 'a, b', i) in items">x</li>
  <li v-for="(item /* a, b */, i) in items">x</li>
</template>

`````

### Expected (prettier)

`````vue
<template>
  <li v-for="(item = 'a,b', i) in items">x</li>
  <li v-for="(item /* a,b */, i) in items">x</li>
</template>

`````
