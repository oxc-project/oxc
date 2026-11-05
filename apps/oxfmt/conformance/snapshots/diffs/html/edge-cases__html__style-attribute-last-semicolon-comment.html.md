# edge-cases/html/style-attribute-last-semicolon-comment.html

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -1,3 +1,3 @@
 <div style="color: red"></div>
-<div style="color: red; /* c */"></div>
-<div style="color: red; /* c */"></div>
+<div style="color: red /* c */"></div>
+<div style="color: red /* c */"></div>

`````

### Actual (oxfmt)

`````html
<div style="color: red"></div>
<div style="color: red /* c */"></div>
<div style="color: red /* c */"></div>

`````

### Expected (prettier)

`````html
<div style="color: red"></div>
<div style="color: red; /* c */"></div>
<div style="color: red; /* c */"></div>

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
@@ -1,3 +1,3 @@
 <div style="color: red"></div>
-<div style="color: red; /* c */"></div>
-<div style="color: red; /* c */"></div>
+<div style="color: red /* c */"></div>
+<div style="color: red /* c */"></div>

`````

### Actual (oxfmt)

`````html
<div style="color: red"></div>
<div style="color: red /* c */"></div>
<div style="color: red /* c */"></div>

`````

### Expected (prettier)

`````html
<div style="color: red"></div>
<div style="color: red; /* c */"></div>
<div style="color: red; /* c */"></div>

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
@@ -1,3 +1,3 @@
 <div style="color: red"></div>
-<div style="color: red; /* c */"></div>
-<div style="color: red; /* c */"></div>
+<div style="color: red /* c */"></div>
+<div style="color: red /* c */"></div>

`````

### Actual (oxfmt)

`````html
<div style="color: red"></div>
<div style="color: red /* c */"></div>
<div style="color: red /* c */"></div>

`````

### Expected (prettier)

`````html
<div style="color: red"></div>
<div style="color: red; /* c */"></div>
<div style="color: red; /* c */"></div>

`````
