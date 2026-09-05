# edge-cases/js-in-vue/generic-trailing-comma.vue

> `<T = any,>` comma removed like plain `.ts`. See apps/oxfmt/DIVERGENCES.md#ts-in-vue-generic-trailing-comma

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
 <!-- DIVERGES: the `<T = any,>` disambiguating comma is removed like plain `.ts`;
   see apps/oxfmt/DIVERGENCES.md "ts-in-vue-generic-trailing-comma" -->
 <script setup lang="ts">
-const getComponentRef = <T = any,>() => componentRef.value as T;
+const getComponentRef = <T = any>() => componentRef.value as T;
 </script>

`````

### Actual (oxfmt)

`````vue
<!-- DIVERGES: the `<T = any,>` disambiguating comma is removed like plain `.ts`;
  see apps/oxfmt/DIVERGENCES.md "ts-in-vue-generic-trailing-comma" -->
<script setup lang="ts">
const getComponentRef = <T = any>() => componentRef.value as T;
</script>

`````

### Expected (prettier)

`````vue
<!-- DIVERGES: the `<T = any,>` disambiguating comma is removed like plain `.ts`;
  see apps/oxfmt/DIVERGENCES.md "ts-in-vue-generic-trailing-comma" -->
<script setup lang="ts">
const getComponentRef = <T = any,>() => componentRef.value as T;
</script>

`````

## Option 2

`````json
{"printWidth":100,"vueIndentScriptAndStyle":true,"singleQuote":true}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -1,5 +1,5 @@
 <!-- DIVERGES: the `<T = any,>` disambiguating comma is removed like plain `.ts`;
   see apps/oxfmt/DIVERGENCES.md "ts-in-vue-generic-trailing-comma" -->
 <script setup lang="ts">
-  const getComponentRef = <T = any,>() => componentRef.value as T;
+  const getComponentRef = <T = any>() => componentRef.value as T;
 </script>

`````

### Actual (oxfmt)

`````vue
<!-- DIVERGES: the `<T = any,>` disambiguating comma is removed like plain `.ts`;
  see apps/oxfmt/DIVERGENCES.md "ts-in-vue-generic-trailing-comma" -->
<script setup lang="ts">
  const getComponentRef = <T = any>() => componentRef.value as T;
</script>

`````

### Expected (prettier)

`````vue
<!-- DIVERGES: the `<T = any,>` disambiguating comma is removed like plain `.ts`;
  see apps/oxfmt/DIVERGENCES.md "ts-in-vue-generic-trailing-comma" -->
<script setup lang="ts">
  const getComponentRef = <T = any,>() => componentRef.value as T;
</script>

`````
