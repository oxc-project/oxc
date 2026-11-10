# Input

`````vue
<script setup lang="ts">
const getOptions = <T = any,>(list: T[]) => list;
const identity = <T,>(x: T) => x;
</script>

`````

# Output

## `{"printWidth":80}`

`````vue
<script setup lang="ts">
const getOptions = <T = any>(list: T[]) => list;
const identity = <T>(x: T) => x;
</script>

`````

## `{"printWidth":100,"vueIndentScriptAndStyle":true,"singleQuote":true,"semi":false}`

`````vue
<script setup lang="ts">
  const getOptions = <T = any>(list: T[]) => list
  const identity = <T>(x: T) => x
</script>

`````
