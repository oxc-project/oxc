# edge-cases/xxx-in-md/lwc-code-block.md

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -1,3 +1,4 @@
 ```lwc
-<x-a attr={b}> c</x-a>
+<x-a attr={b}>
+    c</x-a>
 ```

`````

### Actual (oxfmt)

`````md
```lwc
<x-a attr={b}>
    c</x-a>
```

`````

### Expected (prettier)

`````md
```lwc
<x-a attr={b}> c</x-a>
```

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
@@ -1,3 +1,4 @@
 ```lwc
-<x-a attr={b}> c</x-a>
+<x-a attr={b}>
+    c</x-a>
 ```

`````

### Actual (oxfmt)

`````md
```lwc
<x-a attr={b}>
    c</x-a>
```

`````

### Expected (prettier)

`````md
```lwc
<x-a attr={b}> c</x-a>
```

`````
