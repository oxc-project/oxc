# externals/prettier/markdown/thematicBreak/simple.md

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -1,1 +1,1 @@
----
+***

`````

### Actual (oxfmt)

`````md
***

`````

### Expected (prettier)

`````md
---

`````

## Option 2

`````json
{"printWidth":100,"proseWrap":"always"}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -1,1 +1,1 @@
----
+***

`````

### Actual (oxfmt)

`````md
***

`````

### Expected (prettier)

`````md
---

`````
