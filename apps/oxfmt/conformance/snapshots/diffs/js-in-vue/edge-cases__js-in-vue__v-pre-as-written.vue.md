# edge-cases/js-in-vue/v-pre-as-written.vue

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
 <template>
-  <div v-pre :a="b + c">
-    {{ a + b }} <span @click="x + y">{{ c }}</span>
-  </div>
+  <div v-pre :a="b+c">{{a+b}} <span @click="x+y">{{c}}</span></div>
 </template>

`````

### Actual (oxfmt)

`````vue
<template>
  <div v-pre :a="b+c">{{a+b}} <span @click="x+y">{{c}}</span></div>
</template>

`````

### Expected (prettier)

`````vue
<template>
  <div v-pre :a="b + c">
    {{ a + b }} <span @click="x + y">{{ c }}</span>
  </div>
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
@@ -1,5 +1,3 @@
 <template>
-  <div v-pre :a="b + c">
-    {{ a + b }} <span @click="x + y">{{ c }}</span>
-  </div>
+  <div v-pre :a="b+c">{{a+b}} <span @click="x+y">{{c}}</span></div>
 </template>

`````

### Actual (oxfmt)

`````vue
<template>
  <div v-pre :a="b+c">{{a+b}} <span @click="x+y">{{c}}</span></div>
</template>

`````

### Expected (prettier)

`````vue
<template>
  <div v-pre :a="b + c">
    {{ a + b }} <span @click="x + y">{{ c }}</span>
  </div>
</template>

`````
