# Input

`````vue
<!-- DIVERGES: the `<T = any,>` disambiguating comma is removed like plain `.ts`;
  see apps/oxfmt/DIVERGENCES.md#ts-in-vue-generic-trailing-comma -->
<script setup lang="ts">
const getComponentRef = <T = any,>() => componentRef.value as T;
</script>

`````

# Output

## `{"printWidth":80}`

`````vue
<!-- DIVERGES: the `<T = any,>` disambiguating comma is removed like plain `.ts`;
  see apps/oxfmt/DIVERGENCES.md#ts-in-vue-generic-trailing-comma -->
<script setup lang="ts">
const getComponentRef = <T = any>() => componentRef.value as T;
</script>

`````

## `{"printWidth":100,"vueIndentScriptAndStyle":true,"singleQuote":true,"semi":false}`

`````vue
<!-- DIVERGES: the `<T = any,>` disambiguating comma is removed like plain `.ts`;
  see apps/oxfmt/DIVERGENCES.md#ts-in-vue-generic-trailing-comma -->
<script setup lang="ts">
  const getComponentRef = <T = any>() => componentRef.value as T
</script>

`````
