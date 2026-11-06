# edge-cases/js-in-vue/trailing-line-comment.vue

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -1,43 +1,38 @@
 <template>
-  <li
-    v-for="item in items" // c
-    :key="item"
-  >
-    x
-  </li>
+  <li v-for="item   in items // c" :key="item">x</li>
   <a
-    :a="{ b }" // c
-    :b="
-      x + y // c
-    "
-    @click="
-      go() // c
-    "
+    :a="{ b } // c"
+    :b="x  +  y // c"
+    @click="go( ) // c"
     :d="{
       a: 1, // c
     }"
   />
   <C #default="{ item } // c" />
-  <p>
-    {{
-      a + b // c
-    }}
-  </p>
+  <p>{{ a  +  b // c }}</p>
   <a
-    :a="{ b }" // c
+    :a="
+      { b } // c
+    "
     :b="
       x + y // c
     "
   />
   <C
-    #default="{ item }" // c
+    #default="
+      { item } // c
+    "
   />
   <a
-    :a="{ b }" // c
-    /* d */
+    :a="
+      { b } // c
+      /* d */
+    "
   />
   <C
-    #default="{ item }" // c
-    /* d */
+    #default="
+      { item } // c
+      /* d */
+    "
   />
 </template>

`````

### Actual (oxfmt)

`````vue
<template>
  <li v-for="item   in items // c" :key="item">x</li>
  <a
    :a="{ b } // c"
    :b="x  +  y // c"
    @click="go( ) // c"
    :d="{
      a: 1, // c
    }"
  />
  <C #default="{ item } // c" />
  <p>{{ a  +  b // c }}</p>
  <a
    :a="
      { b } // c
    "
    :b="
      x + y // c
    "
  />
  <C
    #default="
      { item } // c
    "
  />
  <a
    :a="
      { b } // c
      /* d */
    "
  />
  <C
    #default="
      { item } // c
      /* d */
    "
  />
</template>

`````

### Expected (prettier)

`````vue
<template>
  <li
    v-for="item in items" // c
    :key="item"
  >
    x
  </li>
  <a
    :a="{ b }" // c
    :b="
      x + y // c
    "
    @click="
      go() // c
    "
    :d="{
      a: 1, // c
    }"
  />
  <C #default="{ item } // c" />
  <p>
    {{
      a + b // c
    }}
  </p>
  <a
    :a="{ b }" // c
    :b="
      x + y // c
    "
  />
  <C
    #default="{ item }" // c
  />
  <a
    :a="{ b }" // c
    /* d */
  />
  <C
    #default="{ item }" // c
    /* d */
  />
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
@@ -1,43 +1,38 @@
 <template>
-  <li
-    v-for="item in items" // c
-    :key="item"
-  >
-    x
-  </li>
+  <li v-for="item   in items // c" :key="item">x</li>
   <a
-    :a="{ b }" // c
-    :b="
-      x + y // c
-    "
-    @click="
-      go() // c
-    "
+    :a="{ b } // c"
+    :b="x  +  y // c"
+    @click="go( ) // c"
     :d="{
       a: 1, // c
     }"
   />
   <C #default="{ item } // c" />
-  <p>
-    {{
-      a + b // c
-    }}
-  </p>
+  <p>{{ a  +  b // c }}</p>
   <a
-    :a="{ b }" // c
+    :a="
+      { b } // c
+    "
     :b="
       x + y // c
     "
   />
   <C
-    #default="{ item }" // c
+    #default="
+      { item } // c
+    "
   />
   <a
-    :a="{ b }" // c
-    /* d */
+    :a="
+      { b } // c
+      /* d */
+    "
   />
   <C
-    #default="{ item }" // c
-    /* d */
+    #default="
+      { item } // c
+      /* d */
+    "
   />
 </template>

`````

### Actual (oxfmt)

`````vue
<template>
  <li v-for="item   in items // c" :key="item">x</li>
  <a
    :a="{ b } // c"
    :b="x  +  y // c"
    @click="go( ) // c"
    :d="{
      a: 1, // c
    }"
  />
  <C #default="{ item } // c" />
  <p>{{ a  +  b // c }}</p>
  <a
    :a="
      { b } // c
    "
    :b="
      x + y // c
    "
  />
  <C
    #default="
      { item } // c
    "
  />
  <a
    :a="
      { b } // c
      /* d */
    "
  />
  <C
    #default="
      { item } // c
      /* d */
    "
  />
</template>

`````

### Expected (prettier)

`````vue
<template>
  <li
    v-for="item in items" // c
    :key="item"
  >
    x
  </li>
  <a
    :a="{ b }" // c
    :b="
      x + y // c
    "
    @click="
      go() // c
    "
    :d="{
      a: 1, // c
    }"
  />
  <C #default="{ item } // c" />
  <p>
    {{
      a + b // c
    }}
  </p>
  <a
    :a="{ b }" // c
    :b="
      x + y // c
    "
  />
  <C
    #default="{ item }" // c
  />
  <a
    :a="{ b }" // c
    /* d */
  />
  <C
    #default="{ item }" // c
    /* d */
  />
</template>

`````
