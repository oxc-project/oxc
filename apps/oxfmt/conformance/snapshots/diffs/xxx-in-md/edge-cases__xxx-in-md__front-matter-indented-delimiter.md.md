# edge-cases/xxx-in-md/front-matter-indented-delimiter.md

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -1,6 +1,6 @@
 ---
+ ---
+title:   Home
 ---
-title: Home
----
 
 # Heading

`````

### Actual (oxfmt)

`````md
---
 ---
title:   Home
---

# Heading

`````

### Expected (prettier)

`````md
---
---
title: Home
---

# Heading

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
@@ -1,6 +1,6 @@
 ---
+ ---
+title:   Home
 ---
-title: Home
----
 
 # Heading

`````

### Actual (oxfmt)

`````md
---
 ---
title:   Home
---

# Heading

`````

### Expected (prettier)

`````md
---
---
title: Home
---

# Heading

`````
