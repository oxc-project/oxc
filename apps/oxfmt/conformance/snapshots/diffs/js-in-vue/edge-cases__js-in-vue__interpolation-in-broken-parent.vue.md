# edge-cases/js-in-vue/interpolation-in-broken-parent.vue

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -1,12 +1,8 @@
 <template>
   <ul>
-    {{
-      item
-    }}
+    {{ item }}
   </ul>
   <select>
-    {{
-      selected
-    }}
+    {{ selected }}
   </select>
 </template>

`````

### Actual (oxfmt)

`````vue
<template>
  <ul>
    {{ item }}
  </ul>
  <select>
    {{ selected }}
  </select>
</template>

`````

### Expected (prettier)

`````vue
<template>
  <ul>
    {{
      item
    }}
  </ul>
  <select>
    {{
      selected
    }}
  </select>
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
@@ -1,12 +1,8 @@
 <template>
   <ul>
-    {{
-      item
-    }}
+    {{ item }}
   </ul>
   <select>
-    {{
-      selected
-    }}
+    {{ selected }}
   </select>
 </template>

`````

### Actual (oxfmt)

`````vue
<template>
  <ul>
    {{ item }}
  </ul>
  <select>
    {{ selected }}
  </select>
</template>

`````

### Expected (prettier)

`````vue
<template>
  <ul>
    {{
      item
    }}
  </ul>
  <select>
    {{
      selected
    }}
  </select>
</template>

`````
