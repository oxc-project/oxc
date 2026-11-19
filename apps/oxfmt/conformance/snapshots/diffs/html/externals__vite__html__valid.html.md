# externals/vite/html/valid.html

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -1,1 +1,20 @@
-ERROR
\ No newline at end of file
+<!-- accepted HTML parsing errors, browsers accept these -->
+<div id="duplicated-attrs" style="color: green" style="color: red">
+  Accept duplicated attribute
+</div>
+
+<!-- accepted HTML constructs -->
+<datalist>
+  <option value="option-without-closing-tag">
+</datalist>
+
+<div id="no-quotes-on-attr">No quotes on Attr</div>
+<script type="module" src="/valid.js"></script>
+
+<svg>
+  <!-- attr with prefix -->
+  <use xlink:href="/sprite.svg#svg"></use>
+</svg>
+
+<!-- allow self closing on non-void elements -->
+<self-close-non-void />

`````

### Actual (oxfmt)

`````html
<!-- accepted HTML parsing errors, browsers accept these -->
<div id="duplicated-attrs" style="color: green" style="color: red">
  Accept duplicated attribute
</div>

<!-- accepted HTML constructs -->
<datalist>
  <option value="option-without-closing-tag">
</datalist>

<div id="no-quotes-on-attr">No quotes on Attr</div>
<script type="module" src="/valid.js"></script>

<svg>
  <!-- attr with prefix -->
  <use xlink:href="/sprite.svg#svg"></use>
</svg>

<!-- allow self closing on non-void elements -->
<self-close-non-void />

`````

### Expected (prettier)

`````html
ERROR
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
@@ -1,1 +1,27 @@
-ERROR
\ No newline at end of file
+<!-- accepted HTML parsing errors, browsers accept these -->
+<div
+  id="duplicated-attrs"
+  style="color: green"
+  style="color: red"
+>
+  Accept duplicated attribute
+</div>
+
+<!-- accepted HTML constructs -->
+<datalist>
+  <option value="option-without-closing-tag">
+</datalist>
+
+<div id="no-quotes-on-attr">No quotes on Attr</div>
+<script
+  type="module"
+  src="/valid.js"
+></script>
+
+<svg>
+  <!-- attr with prefix -->
+  <use xlink:href="/sprite.svg#svg"></use>
+</svg>
+
+<!-- allow self closing on non-void elements -->
+<self-close-non-void />

`````

### Actual (oxfmt)

`````html
<!-- accepted HTML parsing errors, browsers accept these -->
<div
  id="duplicated-attrs"
  style="color: green"
  style="color: red"
>
  Accept duplicated attribute
</div>

<!-- accepted HTML constructs -->
<datalist>
  <option value="option-without-closing-tag">
</datalist>

<div id="no-quotes-on-attr">No quotes on Attr</div>
<script
  type="module"
  src="/valid.js"
></script>

<svg>
  <!-- attr with prefix -->
  <use xlink:href="/sprite.svg#svg"></use>
</svg>

<!-- allow self closing on non-void elements -->
<self-close-non-void />

`````

### Expected (prettier)

`````html
ERROR
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
@@ -1,1 +1,16 @@
-ERROR
\ No newline at end of file
+<!-- accepted HTML parsing errors, browsers accept these -->
+<div id="duplicated-attrs" style="color: green" style="color: red">Accept duplicated attribute</div>
+
+<!-- accepted HTML constructs -->
+<datalist> <option value="option-without-closing-tag"> </datalist>
+
+<div id="no-quotes-on-attr">No quotes on Attr</div>
+<script type="module" src="/valid.js"></script>
+
+<svg>
+  <!-- attr with prefix -->
+  <use xlink:href="/sprite.svg#svg"></use>
+</svg>
+
+<!-- allow self closing on non-void elements -->
+<self-close-non-void />

`````

### Actual (oxfmt)

`````html
<!-- accepted HTML parsing errors, browsers accept these -->
<div id="duplicated-attrs" style="color: green" style="color: red">Accept duplicated attribute</div>

<!-- accepted HTML constructs -->
<datalist> <option value="option-without-closing-tag"> </datalist>

<div id="no-quotes-on-attr">No quotes on Attr</div>
<script type="module" src="/valid.js"></script>

<svg>
  <!-- attr with prefix -->
  <use xlink:href="/sprite.svg#svg"></use>
</svg>

<!-- allow self closing on non-void elements -->
<self-close-non-void />

`````

### Expected (prettier)

`````html
ERROR
`````
