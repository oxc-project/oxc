# edge-cases/xxx-in-md/nested-fence-length.md

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
-```md
+````md
 ```js
 const a = 1;
 ```
-```
+````

`````

### Actual (oxfmt)

`````md
````md
```js
const a = 1;
```
````

`````

### Expected (prettier)

`````md
```md
```js
const a = 1;
```
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
@@ -1,5 +1,5 @@
-```md
+````md
 ```js
 const a = 1;
 ```
-```
+````

`````

### Actual (oxfmt)

`````md
````md
```js
const a = 1;
```
````

`````

### Expected (prettier)

`````md
```md
```js
const a = 1;
```
```

`````
