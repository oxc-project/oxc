# edge-cases/astro/style-lang-sass.astro

## Option 1

`````json
{"printWidth":80,"astro":{}}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -1,6 +1,6 @@
 <p />
 
 <style lang="sass">
-  .a
-    color: red
+.a
+      color: red
 </style>

`````

### Actual (oxfmt)

`````astro
<p />

<style lang="sass">
.a
      color: red
</style>

`````

### Expected (prettier)

`````astro
<p />

<style lang="sass">
  .a
    color: red
</style>

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
@@ -1,6 +1,6 @@
 <p />
 
 <style lang="sass">
-  .a
-    color: red
+.a
+      color: red
 </style>

`````

### Actual (oxfmt)

`````astro
<p />

<style lang="sass">
.a
      color: red
</style>

`````

### Expected (prettier)

`````astro
<p />

<style lang="sass">
  .a
    color: red
</style>

`````
