# externals/prettier/html/script/script.html

## Option 3

`````json
{"printWidth":120,"htmlWhitespaceSensitivity":"strict","bracketSameLine":true}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -7,9 +7,10 @@
 <script type="importmap">
   { "json": true }
 </script>
 <script type="systemjs-importmap">
-  { "json": true }</script
+  { "json": true }
+</script
 ><script type="invalid">
   {   "json":false  }
 </script>
 <script type="text/html">

`````

### Actual (oxfmt)

`````html
<script type="application/ld+json">
  { "json": true }
</script>
<script type="application/json">
  { "json": true }
</script>
<script type="importmap">
  { "json": true }
</script>
<script type="systemjs-importmap">
  { "json": true }
</script
><script type="invalid">
  {   "json":false  }
</script>
<script type="text/html">
  <div>
    <p>foo</p>
  </div>
</script>

<script async="" id="" src="/_next/static/development/pages/_app.js?ts=1565732195968"></script
><script></script>

<!-- #8147 -->
<script lang="vbscript">
  Function hello()
  End Function
</script>

<script lang="unknown"></script>

<script type="speculationrules">
  {
    "prerender": [{ "source": "list", "urls": ["https://a.test/foo"] }]
  }
</script>

`````

### Expected (prettier)

`````html
<script type="application/ld+json">
  { "json": true }
</script>
<script type="application/json">
  { "json": true }
</script>
<script type="importmap">
  { "json": true }
</script>
<script type="systemjs-importmap">
  { "json": true }</script
><script type="invalid">
  {   "json":false  }
</script>
<script type="text/html">
  <div>
    <p>foo</p>
  </div>
</script>

<script async="" id="" src="/_next/static/development/pages/_app.js?ts=1565732195968"></script
><script></script>

<!-- #8147 -->
<script lang="vbscript">
  Function hello()
  End Function
</script>

<script lang="unknown"></script>

<script type="speculationrules">
  {
    "prerender": [{ "source": "list", "urls": ["https://a.test/foo"] }]
  }
</script>

`````
