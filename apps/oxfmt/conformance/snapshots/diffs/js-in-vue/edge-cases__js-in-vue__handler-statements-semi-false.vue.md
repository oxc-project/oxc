# edge-cases/js-in-vue/handler-statements-semi-false.vue

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -12,43 +12,43 @@
     @blur="
       a = 1;
       [b, c] = [c, b];
     "
-    @focus="/* c */"
+    @focus="; /* c */"
     @keyup="
-      function () {
+      (function () {
         return 1;
-      }
+      })
     "
     @dblclick="
-      function () {
+      (function () {
         a();
         b();
-      }.call(this)
+      }).call(this)
     "
     @mousedown="
-      {
+      ({
         a() {
           b();
           c();
         },
-      }.a()
+      }).a()
     "
     @mouseup="
-      class {
+      (class {
         x = 1;
-      }.name
+      }).name
     "
     @mouseenter="
-      { a } = {
+      ({ a } = {
         b() {
           c();
           d();
         },
-      }
+      })
     "
     @mouseleave="
-      let[0] = {
+      (let)[0] = {
         a() {
           b();
           c();
         },
@@ -59,14 +59,10 @@
       b();
       c();
     "
     @dragstart="a()"
-    @dragend="if (a) b();"
-    @drop="
-      function () {
-        a();
-      }
-    "
+    @dragend="if (a) b()"
+    @drop="function () { a(); }"
     @contextmenu="
       a(); /* c */
       b();
     "

`````

### Actual (oxfmt)

`````vue
<template>
  <button
    @click="
      count++;
      if (a) {
        b();
        c();
      }
    "
    @input="handler;"
    @change="if (x) y();"
    @blur="
      a = 1;
      [b, c] = [c, b];
    "
    @focus="; /* c */"
    @keyup="
      (function () {
        return 1;
      })
    "
    @dblclick="
      (function () {
        a();
        b();
      }).call(this)
    "
    @mousedown="
      ({
        a() {
          b();
          c();
        },
      }).a()
    "
    @mouseup="
      (class {
        x = 1;
      }).name
    "
    @mouseenter="
      ({ a } = {
        b() {
          c();
          d();
        },
      })
    "
    @mouseleave="
      (let)[0] = {
        a() {
          b();
          c();
        },
      }
    "
    @select="
      #!a
      b();
      c();
    "
    @dragstart="a()"
    @dragend="if (a) b()"
    @drop="function () { a(); }"
    @contextmenu="
      a(); /* c */
      b();
    "
  ></button>
</template>

`````

### Expected (prettier)

`````vue
<template>
  <button
    @click="
      count++;
      if (a) {
        b();
        c();
      }
    "
    @input="handler;"
    @change="if (x) y();"
    @blur="
      a = 1;
      [b, c] = [c, b];
    "
    @focus="/* c */"
    @keyup="
      function () {
        return 1;
      }
    "
    @dblclick="
      function () {
        a();
        b();
      }.call(this)
    "
    @mousedown="
      {
        a() {
          b();
          c();
        },
      }.a()
    "
    @mouseup="
      class {
        x = 1;
      }.name
    "
    @mouseenter="
      { a } = {
        b() {
          c();
          d();
        },
      }
    "
    @mouseleave="
      let[0] = {
        a() {
          b();
          c();
        },
      }
    "
    @select="
      #!a
      b();
      c();
    "
    @dragstart="a()"
    @dragend="if (a) b();"
    @drop="
      function () {
        a();
      }
    "
    @contextmenu="
      a(); /* c */
      b();
    "
  ></button>
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
@@ -1,74 +1,70 @@
 <template>
   <button
     @click="
-      count++
+      count++;
       if (a) {
         b()
         c()
       }
     "
     @input="handler;"
-    @change="if (x) y()"
+    @change="if (x) y();"
     @blur="
-      a = 1
-      ;[b, c] = [c, b]
+      a = 1;
+      [b, c] = [c, b]
     "
-    @focus="/* c */"
+    @focus="; /* c */"
     @keyup="
-      function () {
+      (function () {
         return 1
-      }
+      })
     "
     @dblclick="
-      function () {
+      (function () {
         a()
         b()
-      }.call(this)
+      }).call(this)
     "
     @mousedown="
-      {
+      ({
         a() {
           b()
           c()
         },
-      }.a()
+      }).a()
     "
     @mouseup="
-      class {
+      (class {
         x = 1
-      }.name
+      }).name
     "
     @mouseenter="
-      { a } = {
+      ({ a } = {
         b() {
           c()
           d()
         },
-      }
+      })
     "
     @mouseleave="
-      let[0] = {
+      (let)[0] = {
         a() {
           b()
           c()
         },
       }
     "
     @select="
       #!a
-      b()
+      b();
       c()
     "
     @dragstart="a()"
     @dragend="if (a) b()"
-    @drop="
-      function () {
-        a()
-      }
-    "
+    @drop="function () { a(); }"
     @contextmenu="
-      a() /* c */
+      a(); /* c */
       b()
     "
   ></button>
 </template>

`````

### Actual (oxfmt)

`````vue
<template>
  <button
    @click="
      count++;
      if (a) {
        b()
        c()
      }
    "
    @input="handler;"
    @change="if (x) y();"
    @blur="
      a = 1;
      [b, c] = [c, b]
    "
    @focus="; /* c */"
    @keyup="
      (function () {
        return 1
      })
    "
    @dblclick="
      (function () {
        a()
        b()
      }).call(this)
    "
    @mousedown="
      ({
        a() {
          b()
          c()
        },
      }).a()
    "
    @mouseup="
      (class {
        x = 1
      }).name
    "
    @mouseenter="
      ({ a } = {
        b() {
          c()
          d()
        },
      })
    "
    @mouseleave="
      (let)[0] = {
        a() {
          b()
          c()
        },
      }
    "
    @select="
      #!a
      b();
      c()
    "
    @dragstart="a()"
    @dragend="if (a) b()"
    @drop="function () { a(); }"
    @contextmenu="
      a(); /* c */
      b()
    "
  ></button>
</template>

`````

### Expected (prettier)

`````vue
<template>
  <button
    @click="
      count++
      if (a) {
        b()
        c()
      }
    "
    @input="handler;"
    @change="if (x) y()"
    @blur="
      a = 1
      ;[b, c] = [c, b]
    "
    @focus="/* c */"
    @keyup="
      function () {
        return 1
      }
    "
    @dblclick="
      function () {
        a()
        b()
      }.call(this)
    "
    @mousedown="
      {
        a() {
          b()
          c()
        },
      }.a()
    "
    @mouseup="
      class {
        x = 1
      }.name
    "
    @mouseenter="
      { a } = {
        b() {
          c()
          d()
        },
      }
    "
    @mouseleave="
      let[0] = {
        a() {
          b()
          c()
        },
      }
    "
    @select="
      #!a
      b()
      c()
    "
    @dragstart="a()"
    @dragend="if (a) b()"
    @drop="
      function () {
        a()
      }
    "
    @contextmenu="
      a() /* c */
      b()
    "
  ></button>
</template>

`````
