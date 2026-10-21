# externals/prettier/markdown/list/parser-regression/issue-17778.md

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -1,4 +1,5 @@
 1. list item
 2. list item
 
-3. list item 1000000000. ordered list marker can't have more than 9 digits
+3. list item
+   1000000000. ordered list marker can't have more than 9 digits

`````

### Actual (oxfmt)

`````md
1. list item
2. list item

3. list item
   1000000000. ordered list marker can't have more than 9 digits

`````

### Expected (prettier)

`````md
1. list item
2. list item

3. list item 1000000000. ordered list marker can't have more than 9 digits

`````
