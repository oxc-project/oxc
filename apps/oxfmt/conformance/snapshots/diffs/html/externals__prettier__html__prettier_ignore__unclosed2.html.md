# externals/prettier/html/prettier_ignore/unclosed2.html

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -1,5 +1,5 @@
 <!-- prettier-ignore -->
 <p>
 # Hello world!
 
-<div></div>
+<div>

`````

### Actual (oxfmt)

`````html
<!-- prettier-ignore -->
<p>
# Hello world!

<div>

`````

### Expected (prettier)

`````html
<!-- prettier-ignore -->
<p>
# Hello world!

<div></div>

`````

## Option 2

`````json
{"printWidth":100,"htmlWhitespaceSensitivity":"ignore","singleAttributePerLine":true}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -1,5 +1,5 @@
 <!-- prettier-ignore -->
 <p>
 # Hello world!
 
-<div></div>
+<div>

`````

### Actual (oxfmt)

`````html
<!-- prettier-ignore -->
<p>
# Hello world!

<div>

`````

### Expected (prettier)

`````html
<!-- prettier-ignore -->
<p>
# Hello world!

<div></div>

`````

## Option 3

`````json
{"printWidth":120,"htmlWhitespaceSensitivity":"strict","bracketSameLine":true}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -1,5 +1,5 @@
 <!-- prettier-ignore -->
 <p>
 # Hello world!
 
-<div> </div>
+<div>

`````

### Actual (oxfmt)

`````html
<!-- prettier-ignore -->
<p>
# Hello world!

<div>

`````

### Expected (prettier)

`````html
<!-- prettier-ignore -->
<p>
# Hello world!

<div> </div>

`````
