# externals/plugin-astro/styles/with-indented-sass/input.astro

## Option 1

`````json
{"printWidth":80,"astro":{}}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -1,131 +1,133 @@
 <div class="hello">lorem</div>
 
 <style lang="sass">
+
+
   $bg: #1E1E1E
   $text: #9cdcfe
   $shadow: #0002
   $fonts: monaco, Consolas, 'Lucida Console', monospace
 
   body
     padding: 0
-    margin: 0
+      margin: 0
     background: $bg
     color: $text
     font-family: $fonts
     display: flex
     flex-flow: column
     height: 100vh
 
-  .header
-    background: lighten($bg, 2.5)
-    box-shadow: 0 1px 4px $shadow
+.header
+     background: lighten($bg, 2.5)
+  box-shadow: 0 1px 4px $shadow
     padding: 1rem 2rem
-    display: flex
-    align-content: center
-    align-items: center
-    justify-content: center
+  display: flex
+  align-content: center
+      align-items: center
+  justify-content: center
 
-  .header-report-bug
-    position: absolute
-    right: 2rem
+.header-report-bug
+      position: absolute
+  right: 2rem
 
-  .header-title
-    position: absolute
-    left: 2rem
-    font-size: 1.5rem
+.header-title
+  position: absolute
+  left: 2rem
+  font-size: 1.5rem
 
-  .content
-    flex-grow: 1
-    display: grid
-    grid-template-rows: 2rem auto
-    gap: 1rem
-    margin: 1rem
+.content
+  flex-grow: 1
+  display: grid
+      grid-template-rows: 2rem auto
+  gap: 1rem
+  margin: 1rem
 
-  .center-text
-    text-align: center
-    > code
-      font-size: 1.1rem
+.center-text
+  text-align: center
+       > code
+    font-size: 1.1rem
 
-  .editor-container
-    overflow: hidden
+.editor-container
+  overflow: hidden
 
-  a
-    color: $text
-    text-decoration: none
-    outline: none
-    transition: color 100ms ease
+a
+  color: $text
+  text-decoration: none
+  outline: none
+  transition: color 100ms ease
 
-    &:hover
-      color: darken($text, 40)
+     &:hover
+    color: darken($text, 40)
 
-    &:active
-      text-decoration: underline
+  &:active
+    text-decoration: underline
 
-  .loader-container
-    background: $bg
+.loader-container
+background: $bg
     position: absolute
-    top: 0
-    bottom: 0
-    right: 0
-    left: 0
-    transition: opacity 200ms ease
+  top: 0
+      bottom: 0
+  right: 0
+  left: 0
+  transition: opacity 200ms ease
 
-  .loader
-    position: fixed
-    border: 5px solid lighten($bg, 15)
-    border-top: 5px solid $text
-    border-radius: 50%
-    background: #0000
-    width: 25px
-    height: 25px
-    animation: spin 1s linear infinite
-    top: 50%
-    left: 50%
-    transform: translate(-50%, -50%)
+.loader
+  position: fixed
+     border: 5px solid lighten($bg, 15)
+  border-top: 5px solid $text
+  border-radius: 50%
+  background: #0000
+       width: 25px
+  height: 25px
+  animation: spin 1s linear infinite
+       top: 50%
+  left: 50%
+  transform: translate(-50%, -50%)
 
-  @keyframes spin
-    from
-      transform: rotate(0deg)
+@keyframes spin
+  from
+    transform: rotate(0deg)
 
-    to
-      transform: rotate(360deg)
+      to
+    transform: rotate(360deg)
 
-  @media screen and ( max-width: 700px )
-    .header
-      flex-direction: column
-      > *
-        margin: 0.3rem 0
-        .content
-          max-height: 80vh
+@media screen and ( max-width: 700px )
+  .header
+    flex-direction: column
+       > *
+      margin: 0.3rem 0
+      .content
+    max-height: 80vh
 
-  select
-    display: inline-block
-    box-sizing: border-box
-    font: inherit
-    line-height: inherit
-    -webkit-appearance: none
-    -moz-appearance: none
-    -ms-appearance: none
-    appearance: none
-    outline: none
-    border: none
+select
+  display: inline-block
+box-sizing: border-box
+  font: inherit
+     line-height: inherit
+  -webkit-appearance: none
+  -moz-appearance: none
+       -ms-appearance: none
+       appearance: none
+       outline: none
+  border: none
 
-    color: $text
+  color: $text
 
-    border-radius: 5px
+  border-radius: 5px
 
-    padding: 0.5em 2em 0.5em 0.5em
+  padding: 0.5em 2em 0.5em 0.5em
 
-    background-color: lighten($bg, 10)
-    background-repeat: no-repeat
-    background-image: linear-gradient(45deg, transparent 49.5%, currentColor 50%), linear-gradient(135deg, currentColor 49.5%, transparent 50%)
-    background-position: right 15px top 1em, right 10px top 1em
-    background-size: 5px 5px, 5px 5px
+     background-color: lighten($bg, 10)
+       background-repeat: no-repeat
+  background-image: linear-gradient(45deg, transparent 49.5%, currentColor 50%), linear-gradient(135deg, currentColor 49.5%, transparent 50%)
+        background-position: right 15px top 1em, right 10px top 1em
+  background-size: 5px 5px, 5px 5px
 
-    transition: background 200ms ease
+  transition: background 200ms ease
 
-    cursor: pointer
+  cursor: pointer
 
-    &:hover
-      background-color: lighten($bg, 20)
+  &:hover
+    background-color: lighten($bg, 20)
 </style>

`````

### Actual (oxfmt)

`````astro
<div class="hello">lorem</div>

<style lang="sass">


  $bg: #1E1E1E
  $text: #9cdcfe
  $shadow: #0002
  $fonts: monaco, Consolas, 'Lucida Console', monospace

  body
    padding: 0
      margin: 0
    background: $bg
    color: $text
    font-family: $fonts
    display: flex
    flex-flow: column
    height: 100vh

.header
     background: lighten($bg, 2.5)
  box-shadow: 0 1px 4px $shadow
    padding: 1rem 2rem
  display: flex
  align-content: center
      align-items: center
  justify-content: center

.header-report-bug
      position: absolute
  right: 2rem

.header-title
  position: absolute
  left: 2rem
  font-size: 1.5rem

.content
  flex-grow: 1
  display: grid
      grid-template-rows: 2rem auto
  gap: 1rem
  margin: 1rem

.center-text
  text-align: center
       > code
    font-size: 1.1rem

.editor-container
  overflow: hidden

a
  color: $text
  text-decoration: none
  outline: none
  transition: color 100ms ease

     &:hover
    color: darken($text, 40)

  &:active
    text-decoration: underline

.loader-container
background: $bg
    position: absolute
  top: 0
      bottom: 0
  right: 0
  left: 0
  transition: opacity 200ms ease

.loader
  position: fixed
     border: 5px solid lighten($bg, 15)
  border-top: 5px solid $text
  border-radius: 50%
  background: #0000
       width: 25px
  height: 25px
  animation: spin 1s linear infinite
       top: 50%
  left: 50%
  transform: translate(-50%, -50%)

@keyframes spin
  from
    transform: rotate(0deg)

      to
    transform: rotate(360deg)

@media screen and ( max-width: 700px )
  .header
    flex-direction: column
       > *
      margin: 0.3rem 0
      .content
    max-height: 80vh

select
  display: inline-block
box-sizing: border-box
  font: inherit
     line-height: inherit
  -webkit-appearance: none
  -moz-appearance: none
       -ms-appearance: none
       appearance: none
       outline: none
  border: none

  color: $text

  border-radius: 5px

  padding: 0.5em 2em 0.5em 0.5em

     background-color: lighten($bg, 10)
       background-repeat: no-repeat
  background-image: linear-gradient(45deg, transparent 49.5%, currentColor 50%), linear-gradient(135deg, currentColor 49.5%, transparent 50%)
        background-position: right 15px top 1em, right 10px top 1em
  background-size: 5px 5px, 5px 5px

  transition: background 200ms ease

  cursor: pointer

  &:hover
    background-color: lighten($bg, 20)
</style>

`````

### Expected (prettier)

`````astro
<div class="hello">lorem</div>

<style lang="sass">
  $bg: #1E1E1E
  $text: #9cdcfe
  $shadow: #0002
  $fonts: monaco, Consolas, 'Lucida Console', monospace

  body
    padding: 0
    margin: 0
    background: $bg
    color: $text
    font-family: $fonts
    display: flex
    flex-flow: column
    height: 100vh

  .header
    background: lighten($bg, 2.5)
    box-shadow: 0 1px 4px $shadow
    padding: 1rem 2rem
    display: flex
    align-content: center
    align-items: center
    justify-content: center

  .header-report-bug
    position: absolute
    right: 2rem

  .header-title
    position: absolute
    left: 2rem
    font-size: 1.5rem

  .content
    flex-grow: 1
    display: grid
    grid-template-rows: 2rem auto
    gap: 1rem
    margin: 1rem

  .center-text
    text-align: center
    > code
      font-size: 1.1rem

  .editor-container
    overflow: hidden

  a
    color: $text
    text-decoration: none
    outline: none
    transition: color 100ms ease

    &:hover
      color: darken($text, 40)

    &:active
      text-decoration: underline

  .loader-container
    background: $bg
    position: absolute
    top: 0
    bottom: 0
    right: 0
    left: 0
    transition: opacity 200ms ease

  .loader
    position: fixed
    border: 5px solid lighten($bg, 15)
    border-top: 5px solid $text
    border-radius: 50%
    background: #0000
    width: 25px
    height: 25px
    animation: spin 1s linear infinite
    top: 50%
    left: 50%
    transform: translate(-50%, -50%)

  @keyframes spin
    from
      transform: rotate(0deg)

    to
      transform: rotate(360deg)

  @media screen and ( max-width: 700px )
    .header
      flex-direction: column
      > *
        margin: 0.3rem 0
        .content
          max-height: 80vh

  select
    display: inline-block
    box-sizing: border-box
    font: inherit
    line-height: inherit
    -webkit-appearance: none
    -moz-appearance: none
    -ms-appearance: none
    appearance: none
    outline: none
    border: none

    color: $text

    border-radius: 5px

    padding: 0.5em 2em 0.5em 0.5em

    background-color: lighten($bg, 10)
    background-repeat: no-repeat
    background-image: linear-gradient(45deg, transparent 49.5%, currentColor 50%), linear-gradient(135deg, currentColor 49.5%, transparent 50%)
    background-position: right 15px top 1em, right 10px top 1em
    background-size: 5px 5px, 5px 5px

    transition: background 200ms ease

    cursor: pointer

    &:hover
      background-color: lighten($bg, 20)
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
@@ -1,131 +1,133 @@
 <div class="hello">lorem</div>
 
 <style lang="sass">
+
+
   $bg: #1E1E1E
   $text: #9cdcfe
   $shadow: #0002
   $fonts: monaco, Consolas, 'Lucida Console', monospace
 
   body
     padding: 0
-    margin: 0
+      margin: 0
     background: $bg
     color: $text
     font-family: $fonts
     display: flex
     flex-flow: column
     height: 100vh
 
-  .header
-    background: lighten($bg, 2.5)
-    box-shadow: 0 1px 4px $shadow
+.header
+     background: lighten($bg, 2.5)
+  box-shadow: 0 1px 4px $shadow
     padding: 1rem 2rem
-    display: flex
-    align-content: center
-    align-items: center
-    justify-content: center
+  display: flex
+  align-content: center
+      align-items: center
+  justify-content: center
 
-  .header-report-bug
-    position: absolute
-    right: 2rem
+.header-report-bug
+      position: absolute
+  right: 2rem
 
-  .header-title
-    position: absolute
-    left: 2rem
-    font-size: 1.5rem
+.header-title
+  position: absolute
+  left: 2rem
+  font-size: 1.5rem
 
-  .content
-    flex-grow: 1
-    display: grid
-    grid-template-rows: 2rem auto
-    gap: 1rem
-    margin: 1rem
+.content
+  flex-grow: 1
+  display: grid
+      grid-template-rows: 2rem auto
+  gap: 1rem
+  margin: 1rem
 
-  .center-text
-    text-align: center
-    > code
-      font-size: 1.1rem
+.center-text
+  text-align: center
+       > code
+    font-size: 1.1rem
 
-  .editor-container
-    overflow: hidden
+.editor-container
+  overflow: hidden
 
-  a
-    color: $text
-    text-decoration: none
-    outline: none
-    transition: color 100ms ease
+a
+  color: $text
+  text-decoration: none
+  outline: none
+  transition: color 100ms ease
 
-    &:hover
-      color: darken($text, 40)
+     &:hover
+    color: darken($text, 40)
 
-    &:active
-      text-decoration: underline
+  &:active
+    text-decoration: underline
 
-  .loader-container
-    background: $bg
+.loader-container
+background: $bg
     position: absolute
-    top: 0
-    bottom: 0
-    right: 0
-    left: 0
-    transition: opacity 200ms ease
+  top: 0
+      bottom: 0
+  right: 0
+  left: 0
+  transition: opacity 200ms ease
 
-  .loader
-    position: fixed
-    border: 5px solid lighten($bg, 15)
-    border-top: 5px solid $text
-    border-radius: 50%
-    background: #0000
-    width: 25px
-    height: 25px
-    animation: spin 1s linear infinite
-    top: 50%
-    left: 50%
-    transform: translate(-50%, -50%)
+.loader
+  position: fixed
+     border: 5px solid lighten($bg, 15)
+  border-top: 5px solid $text
+  border-radius: 50%
+  background: #0000
+       width: 25px
+  height: 25px
+  animation: spin 1s linear infinite
+       top: 50%
+  left: 50%
+  transform: translate(-50%, -50%)
 
-  @keyframes spin
-    from
-      transform: rotate(0deg)
+@keyframes spin
+  from
+    transform: rotate(0deg)
 
-    to
-      transform: rotate(360deg)
+      to
+    transform: rotate(360deg)
 
-  @media screen and ( max-width: 700px )
-    .header
-      flex-direction: column
-      > *
-        margin: 0.3rem 0
-        .content
-          max-height: 80vh
+@media screen and ( max-width: 700px )
+  .header
+    flex-direction: column
+       > *
+      margin: 0.3rem 0
+      .content
+    max-height: 80vh
 
-  select
-    display: inline-block
-    box-sizing: border-box
-    font: inherit
-    line-height: inherit
-    -webkit-appearance: none
-    -moz-appearance: none
-    -ms-appearance: none
-    appearance: none
-    outline: none
-    border: none
+select
+  display: inline-block
+box-sizing: border-box
+  font: inherit
+     line-height: inherit
+  -webkit-appearance: none
+  -moz-appearance: none
+       -ms-appearance: none
+       appearance: none
+       outline: none
+  border: none
 
-    color: $text
+  color: $text
 
-    border-radius: 5px
+  border-radius: 5px
 
-    padding: 0.5em 2em 0.5em 0.5em
+  padding: 0.5em 2em 0.5em 0.5em
 
-    background-color: lighten($bg, 10)
-    background-repeat: no-repeat
-    background-image: linear-gradient(45deg, transparent 49.5%, currentColor 50%), linear-gradient(135deg, currentColor 49.5%, transparent 50%)
-    background-position: right 15px top 1em, right 10px top 1em
-    background-size: 5px 5px, 5px 5px
+     background-color: lighten($bg, 10)
+       background-repeat: no-repeat
+  background-image: linear-gradient(45deg, transparent 49.5%, currentColor 50%), linear-gradient(135deg, currentColor 49.5%, transparent 50%)
+        background-position: right 15px top 1em, right 10px top 1em
+  background-size: 5px 5px, 5px 5px
 
-    transition: background 200ms ease
+  transition: background 200ms ease
 
-    cursor: pointer
+  cursor: pointer
 
-    &:hover
-      background-color: lighten($bg, 20)
+  &:hover
+    background-color: lighten($bg, 20)
 </style>

`````

### Actual (oxfmt)

`````astro
<div class="hello">lorem</div>

<style lang="sass">


  $bg: #1E1E1E
  $text: #9cdcfe
  $shadow: #0002
  $fonts: monaco, Consolas, 'Lucida Console', monospace

  body
    padding: 0
      margin: 0
    background: $bg
    color: $text
    font-family: $fonts
    display: flex
    flex-flow: column
    height: 100vh

.header
     background: lighten($bg, 2.5)
  box-shadow: 0 1px 4px $shadow
    padding: 1rem 2rem
  display: flex
  align-content: center
      align-items: center
  justify-content: center

.header-report-bug
      position: absolute
  right: 2rem

.header-title
  position: absolute
  left: 2rem
  font-size: 1.5rem

.content
  flex-grow: 1
  display: grid
      grid-template-rows: 2rem auto
  gap: 1rem
  margin: 1rem

.center-text
  text-align: center
       > code
    font-size: 1.1rem

.editor-container
  overflow: hidden

a
  color: $text
  text-decoration: none
  outline: none
  transition: color 100ms ease

     &:hover
    color: darken($text, 40)

  &:active
    text-decoration: underline

.loader-container
background: $bg
    position: absolute
  top: 0
      bottom: 0
  right: 0
  left: 0
  transition: opacity 200ms ease

.loader
  position: fixed
     border: 5px solid lighten($bg, 15)
  border-top: 5px solid $text
  border-radius: 50%
  background: #0000
       width: 25px
  height: 25px
  animation: spin 1s linear infinite
       top: 50%
  left: 50%
  transform: translate(-50%, -50%)

@keyframes spin
  from
    transform: rotate(0deg)

      to
    transform: rotate(360deg)

@media screen and ( max-width: 700px )
  .header
    flex-direction: column
       > *
      margin: 0.3rem 0
      .content
    max-height: 80vh

select
  display: inline-block
box-sizing: border-box
  font: inherit
     line-height: inherit
  -webkit-appearance: none
  -moz-appearance: none
       -ms-appearance: none
       appearance: none
       outline: none
  border: none

  color: $text

  border-radius: 5px

  padding: 0.5em 2em 0.5em 0.5em

     background-color: lighten($bg, 10)
       background-repeat: no-repeat
  background-image: linear-gradient(45deg, transparent 49.5%, currentColor 50%), linear-gradient(135deg, currentColor 49.5%, transparent 50%)
        background-position: right 15px top 1em, right 10px top 1em
  background-size: 5px 5px, 5px 5px

  transition: background 200ms ease

  cursor: pointer

  &:hover
    background-color: lighten($bg, 20)
</style>

`````

### Expected (prettier)

`````astro
<div class="hello">lorem</div>

<style lang="sass">
  $bg: #1E1E1E
  $text: #9cdcfe
  $shadow: #0002
  $fonts: monaco, Consolas, 'Lucida Console', monospace

  body
    padding: 0
    margin: 0
    background: $bg
    color: $text
    font-family: $fonts
    display: flex
    flex-flow: column
    height: 100vh

  .header
    background: lighten($bg, 2.5)
    box-shadow: 0 1px 4px $shadow
    padding: 1rem 2rem
    display: flex
    align-content: center
    align-items: center
    justify-content: center

  .header-report-bug
    position: absolute
    right: 2rem

  .header-title
    position: absolute
    left: 2rem
    font-size: 1.5rem

  .content
    flex-grow: 1
    display: grid
    grid-template-rows: 2rem auto
    gap: 1rem
    margin: 1rem

  .center-text
    text-align: center
    > code
      font-size: 1.1rem

  .editor-container
    overflow: hidden

  a
    color: $text
    text-decoration: none
    outline: none
    transition: color 100ms ease

    &:hover
      color: darken($text, 40)

    &:active
      text-decoration: underline

  .loader-container
    background: $bg
    position: absolute
    top: 0
    bottom: 0
    right: 0
    left: 0
    transition: opacity 200ms ease

  .loader
    position: fixed
    border: 5px solid lighten($bg, 15)
    border-top: 5px solid $text
    border-radius: 50%
    background: #0000
    width: 25px
    height: 25px
    animation: spin 1s linear infinite
    top: 50%
    left: 50%
    transform: translate(-50%, -50%)

  @keyframes spin
    from
      transform: rotate(0deg)

    to
      transform: rotate(360deg)

  @media screen and ( max-width: 700px )
    .header
      flex-direction: column
      > *
        margin: 0.3rem 0
        .content
          max-height: 80vh

  select
    display: inline-block
    box-sizing: border-box
    font: inherit
    line-height: inherit
    -webkit-appearance: none
    -moz-appearance: none
    -ms-appearance: none
    appearance: none
    outline: none
    border: none

    color: $text

    border-radius: 5px

    padding: 0.5em 2em 0.5em 0.5em

    background-color: lighten($bg, 10)
    background-repeat: no-repeat
    background-image: linear-gradient(45deg, transparent 49.5%, currentColor 50%), linear-gradient(135deg, currentColor 49.5%, transparent 50%)
    background-position: right 15px top 1em, right 10px top 1em
    background-size: 5px 5px, 5px 5px

    transition: background 200ms ease

    cursor: pointer

    &:hover
      background-color: lighten($bg, 20)
</style>

`````
