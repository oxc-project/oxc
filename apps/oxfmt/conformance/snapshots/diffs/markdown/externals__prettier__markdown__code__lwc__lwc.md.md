# externals/prettier/markdown/code/lwc/lwc.md

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -2,6 +2,7 @@
 <test-this attr="{should-be-quoted}"> Awesome</test-this>
 ```
 
 ```lwc
-<test-this attr={should-NOT-quoted}> Awesome</test-this>
+<test-this attr={should-NOT-quoted}>
+    Awesome</test-this>
 ```

`````

### Actual (oxfmt)

`````md
```html
<test-this attr="{should-be-quoted}"> Awesome</test-this>
```

```lwc
<test-this attr={should-NOT-quoted}>
    Awesome</test-this>
```

`````

### Expected (prettier)

`````md
```html
<test-this attr="{should-be-quoted}"> Awesome</test-this>
```

```lwc
<test-this attr={should-NOT-quoted}> Awesome</test-this>
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
@@ -2,6 +2,7 @@
 <test-this attr="{should-be-quoted}"> Awesome</test-this>
 ```
 
 ```lwc
-<test-this attr={should-NOT-quoted}> Awesome</test-this>
+<test-this attr={should-NOT-quoted}>
+    Awesome</test-this>
 ```

`````

### Actual (oxfmt)

`````md
```html
<test-this attr="{should-be-quoted}"> Awesome</test-this>
```

```lwc
<test-this attr={should-NOT-quoted}>
    Awesome</test-this>
```

`````

### Expected (prettier)

`````md
```html
<test-this attr="{should-be-quoted}"> Awesome</test-this>
```

```lwc
<test-this attr={should-NOT-quoted}> Awesome</test-this>
```

`````
