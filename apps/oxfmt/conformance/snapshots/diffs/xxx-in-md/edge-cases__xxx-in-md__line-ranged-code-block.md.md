# edge-cases/xxx-in-md/line-ranged-code-block.md

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -1,4 +1,6 @@
 ```js {4}
-const a = [1, 2];
+const a = [
+  1, 2,
+];
 console.log(a);
 ```

`````

### Actual (oxfmt)

`````md
```js {4}
const a = [
  1, 2,
];
console.log(a);
```

`````

### Expected (prettier)

`````md
```js {4}
const a = [1, 2];
console.log(a);
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
@@ -1,4 +1,6 @@
 ```js {4}
-const a = [1, 2];
+const a = [
+  1, 2,
+];
 console.log(a);
 ```

`````

### Actual (oxfmt)

`````md
```js {4}
const a = [
  1, 2,
];
console.log(a);
```

`````

### Expected (prettier)

`````md
```js {4}
const a = [1, 2];
console.log(a);
```

`````
