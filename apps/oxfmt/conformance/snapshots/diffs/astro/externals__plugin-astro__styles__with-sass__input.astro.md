# externals/plugin-astro/styles/with-sass/input.astro

## Option 1

`````json
{"printWidth":80,"astro":{}}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -3,8 +3,10 @@
 <style lang="sass">
   .hello
     color: red
 
+
+
   p
     background-color: white
 
   // hidden comment
@@ -14,8 +16,9 @@
       margin: 0
       padding: 0
       list-style: none
 
+
     /* visible comment */
 
     li
       display: inline-block
@@ -23,7 +26,10 @@
       a
         display: block
         padding: 6px 12px
         text-decoration: none
+
+
+
 </style>
 
 <style lang="sass" set:html={someCSS}></style>

`````

### Actual (oxfmt)

`````astro
<div class="hello">lorem</div>

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

<style lang="sass" set:html={someCSS}></style>

`````

### Expected (prettier)

`````astro
<div class="hello">lorem</div>

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

<style lang="sass" set:html={someCSS}></style>

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
@@ -3,8 +3,10 @@
 <style lang="sass">
   .hello
     color: red
 
+
+
   p
     background-color: white
 
   // hidden comment
@@ -14,8 +16,9 @@
       margin: 0
       padding: 0
       list-style: none
 
+
     /* visible comment */
 
     li
       display: inline-block
@@ -23,7 +26,10 @@
       a
         display: block
         padding: 6px 12px
         text-decoration: none
+
+
+
 </style>
 
 <style lang="sass" set:html={someCSS}></style>

`````

### Actual (oxfmt)

`````astro
<div class="hello">lorem</div>

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

<style lang="sass" set:html={someCSS}></style>

`````

### Expected (prettier)

`````astro
<div class="hello">lorem</div>

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

<style lang="sass" set:html={someCSS}></style>

`````
