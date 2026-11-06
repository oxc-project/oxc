# edge-cases/js-in-vue/top-level-sequence.vue

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -1,6 +1,6 @@
 <template>
-  <p :class="(a, b)" v-show="(a, b)">{{ (a, b) }}</p>
-  <p :class="(a, b)" v-show="/* c */ (a, b)">{{ ((a, b), c) }}</p>
+  <p :class="a,b" v-show="a,b">{{ a,b }}</p>
+  <p :class="(a),b" v-show="/* c */ (a),b">{{ (a,b),c }}</p>
   <p v-if="(a, b)" @click="(a, b)">{{ (a, b) }}</p>
   <p v-else-if="(a, b)" :class="/* c */ (a, b)">{{ (a, b) }}</p>
 </template>

`````

### Actual (oxfmt)

`````vue
<template>
  <p :class="a,b" v-show="a,b">{{ a,b }}</p>
  <p :class="(a),b" v-show="/* c */ (a),b">{{ (a,b),c }}</p>
  <p v-if="(a, b)" @click="(a, b)">{{ (a, b) }}</p>
  <p v-else-if="(a, b)" :class="/* c */ (a, b)">{{ (a, b) }}</p>
</template>

`````

### Expected (prettier)

`````vue
<template>
  <p :class="(a, b)" v-show="(a, b)">{{ (a, b) }}</p>
  <p :class="(a, b)" v-show="/* c */ (a, b)">{{ ((a, b), c) }}</p>
  <p v-if="(a, b)" @click="(a, b)">{{ (a, b) }}</p>
  <p v-else-if="(a, b)" :class="/* c */ (a, b)">{{ (a, b) }}</p>
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
@@ -1,6 +1,6 @@
 <template>
-  <p :class="(a, b)" v-show="(a, b)">{{ (a, b) }}</p>
-  <p :class="(a, b)" v-show="/* c */ (a, b)">{{ ((a, b), c) }}</p>
+  <p :class="a,b" v-show="a,b">{{ a,b }}</p>
+  <p :class="(a),b" v-show="/* c */ (a),b">{{ (a,b),c }}</p>
   <p v-if="(a, b)" @click="(a, b)">{{ (a, b) }}</p>
   <p v-else-if="(a, b)" :class="/* c */ (a, b)">{{ (a, b) }}</p>
 </template>

`````

### Actual (oxfmt)

`````vue
<template>
  <p :class="a,b" v-show="a,b">{{ a,b }}</p>
  <p :class="(a),b" v-show="/* c */ (a),b">{{ (a,b),c }}</p>
  <p v-if="(a, b)" @click="(a, b)">{{ (a, b) }}</p>
  <p v-else-if="(a, b)" :class="/* c */ (a, b)">{{ (a, b) }}</p>
</template>

`````

### Expected (prettier)

`````vue
<template>
  <p :class="(a, b)" v-show="(a, b)">{{ (a, b) }}</p>
  <p :class="(a, b)" v-show="/* c */ (a, b)">{{ ((a, b), c) }}</p>
  <p v-if="(a, b)" @click="(a, b)">{{ (a, b) }}</p>
  <p v-else-if="(a, b)" :class="/* c */ (a, b)">{{ (a, b) }}</p>
</template>

`````
