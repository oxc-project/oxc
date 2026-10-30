# externals/prettier/html/basics/broken-html.html

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -5,5 +5,5 @@
 linebreak
 `<`
 extra space(s)
 -->
-<div><span> < </span></div>
+<div><span> <

`````

### Actual (oxfmt)

`````html
<!-- 
#7241, 
reproduction:
two different element,
linebreak
`<`
extra space(s)
-->
<div><span> <

`````

### Expected (prettier)

`````html
<!-- 
#7241, 
reproduction:
two different element,
linebreak
`<`
extra space(s)
-->
<div><span> < </span></div>

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
@@ -5,5 +5,5 @@
 linebreak
 `<`
 extra space(s)
 -->
-<div><span><</span></div>
+<div><span><

`````

### Actual (oxfmt)

`````html
<!-- 
#7241, 
reproduction:
two different element,
linebreak
`<`
extra space(s)
-->
<div><span><

`````

### Expected (prettier)

`````html
<!-- 
#7241, 
reproduction:
two different element,
linebreak
`<`
extra space(s)
-->
<div><span><</span></div>

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
@@ -5,5 +5,5 @@
 linebreak
 `<`
 extra space(s)
 -->
-<div><span> < </span></div>
+<div><span> <

`````

### Actual (oxfmt)

`````html
<!-- 
#7241, 
reproduction:
two different element,
linebreak
`<`
extra space(s)
-->
<div><span> <

`````

### Expected (prettier)

`````html
<!-- 
#7241, 
reproduction:
two different element,
linebreak
`<`
extra space(s)
-->
<div><span> < </span></div>

`````
