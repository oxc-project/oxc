# edge-cases/xxx-in-md/whitespace-only-code-block.md

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -1,9 +1,7 @@
 ```json
 
-
 ```
 
 ```json5
 
-
 ```

`````

### Actual (oxfmt)

`````md
```json

```

```json5

```

`````

### Expected (prettier)

`````md
```json


```

```json5


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
@@ -1,9 +1,7 @@
 ```json
 
-
 ```
 
 ```json5
 
-
 ```

`````

### Actual (oxfmt)

`````md
```json

```

```json5

```

`````

### Expected (prettier)

`````md
```json


```

```json5


```

`````
