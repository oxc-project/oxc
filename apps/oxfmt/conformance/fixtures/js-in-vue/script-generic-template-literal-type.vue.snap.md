# Input

`````vue
<script setup lang="ts" generic="T extends `${string}-${number}`, U">
const value = 1
</script>

`````

# Output

## `{"printWidth":80}`

`````vue
<script setup lang="ts" generic="T extends `${string}-${number}`, U">
const value = 1;
</script>

`````

## `{"printWidth":100,"vueIndentScriptAndStyle":true,"singleQuote":true,"semi":false}`

`````vue
<script setup lang="ts" generic="T extends `${string}-${number}`, U">
  const value = 1
</script>

`````
