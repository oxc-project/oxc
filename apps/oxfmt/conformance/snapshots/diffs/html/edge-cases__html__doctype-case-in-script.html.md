# edge-cases/html/doctype-case-in-script.html

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
 <script>
-  const page = html`<!doctype html>
+  const page = html`<!DOCTYPE html>
     <html>
       <head>
         <title>x</title>
       </head>

`````

### Actual (oxfmt)

`````html
<script>
  const page = html`<!DOCTYPE html>
    <html>
      <head>
        <title>x</title>
      </head>
    </html>`;
</script>

`````

### Expected (prettier)

`````html
<script>
  const page = html`<!doctype html>
    <html>
      <head>
        <title>x</title>
      </head>
    </html>`;
</script>

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
@@ -1,7 +1,7 @@
 <script>
   const page = html`
-    <!doctype html>
+    <!DOCTYPE html>
     <html>
       <head>
         <title>x</title>
       </head>

`````

### Actual (oxfmt)

`````html
<script>
  const page = html`
    <!DOCTYPE html>
    <html>
      <head>
        <title>x</title>
      </head>
    </html>
  `;
</script>

`````

### Expected (prettier)

`````html
<script>
  const page = html`
    <!doctype html>
    <html>
      <head>
        <title>x</title>
      </head>
    </html>
  `;
</script>

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
@@ -1,6 +1,6 @@
 <script>
-  const page = html`<!doctype html>
+  const page = html`<!DOCTYPE html>
     <html>
       <head
         ><title>x</title></head
       >

`````

### Actual (oxfmt)

`````html
<script>
  const page = html`<!DOCTYPE html>
    <html>
      <head
        ><title>x</title></head
      >
    </html>`;
</script>

`````

### Expected (prettier)

`````html
<script>
  const page = html`<!doctype html>
    <html>
      <head
        ><title>x</title></head
      >
    </html>`;
</script>

`````
