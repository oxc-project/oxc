# externals/plugin-astro/styles/format-nested-sass-style-tag-content/input.astro

## Option 1

`````json
{"printWidth":80,"astro":{}}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -1,30 +1,36 @@
 <html lang={content.lang || "en"}>
   <head>
     <MainHead title={content.title} />
     <style lang="sass">
-      .hello
-        color: red
+  .hello
+    color: red
 
-      p
-        background-color: white
 
-      // hidden comment
 
-      nav
-        ul
-          margin: 0
-          padding: 0
-          list-style: none
+  p
+    background-color: white
 
-        /* visible comment */
+  // hidden comment
 
-        li
-          display: inline-block
+  nav
+    ul
+      margin: 0
+      padding: 0
+      list-style: none
 
-          a
-            display: block
-            padding: 6px 12px
-            text-decoration: none
-    </style>
+
+    /* visible comment */
+
+    li
+      display: inline-block
+
+      a
+        display: block
+        padding: 6px 12px
+        text-decoration: none
+
+
+
+</style>
   </head>
 </html>

`````

### Actual (oxfmt)

`````astro
<html lang={content.lang || "en"}>
  <head>
    <MainHead title={content.title} />
    <style lang="sass">
  .hello
    color: red



  p
    background-color: white

  // hidden comment

  nav
    ul
      margin: 0
      padding: 0
      list-style: none


    /* visible comment */

    li
      display: inline-block

      a
        display: block
        padding: 6px 12px
        text-decoration: none



</style>
  </head>
</html>

`````

### Expected (prettier)

`````astro
<html lang={content.lang || "en"}>
  <head>
    <MainHead title={content.title} />
    <style lang="sass">
      .hello
        color: red

      p
        background-color: white

      // hidden comment

      nav
        ul
          margin: 0
          padding: 0
          list-style: none

        /* visible comment */

        li
          display: inline-block

          a
            display: block
            padding: 6px 12px
            text-decoration: none
    </style>
  </head>
</html>

`````

## Option 2

`````json
{"printWidth":120,"singleQuote":true,"semi":false,"astroAllowShorthand":true,"astroCompressHTML":"html","astro":{"allowShorthand":true,"compressHTML":"html"}}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -1,30 +1,36 @@
 <html lang={content.lang || 'en'}>
   <head>
     <MainHead title={content.title} />
     <style lang="sass">
-      .hello
-        color: red
+  .hello
+    color: red
 
-      p
-        background-color: white
 
-      // hidden comment
 
-      nav
-        ul
-          margin: 0
-          padding: 0
-          list-style: none
+  p
+    background-color: white
 
-        /* visible comment */
+  // hidden comment
 
-        li
-          display: inline-block
+  nav
+    ul
+      margin: 0
+      padding: 0
+      list-style: none
 
-          a
-            display: block
-            padding: 6px 12px
-            text-decoration: none
-    </style>
+
+    /* visible comment */
+
+    li
+      display: inline-block
+
+      a
+        display: block
+        padding: 6px 12px
+        text-decoration: none
+
+
+
+</style>
   </head>
 </html>

`````

### Actual (oxfmt)

`````astro
<html lang={content.lang || 'en'}>
  <head>
    <MainHead title={content.title} />
    <style lang="sass">
  .hello
    color: red



  p
    background-color: white

  // hidden comment

  nav
    ul
      margin: 0
      padding: 0
      list-style: none


    /* visible comment */

    li
      display: inline-block

      a
        display: block
        padding: 6px 12px
        text-decoration: none



</style>
  </head>
</html>

`````

### Expected (prettier)

`````astro
<html lang={content.lang || 'en'}>
  <head>
    <MainHead title={content.title} />
    <style lang="sass">
      .hello
        color: red

      p
        background-color: white

      // hidden comment

      nav
        ul
          margin: 0
          padding: 0
          list-style: none

        /* visible comment */

        li
          display: inline-block

          a
            display: block
            padding: 6px 12px
            text-decoration: none
    </style>
  </head>
</html>

`````
