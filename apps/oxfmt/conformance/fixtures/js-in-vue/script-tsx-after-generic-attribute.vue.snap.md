# Input

`````vue
<script setup generic="T extends Record<string, string>" lang="tsx">
const pick = <U = T,>(x: U) => x;
</script>

`````

# Output

## `{"printWidth":80}`

`````vue
<script setup generic="T extends Record<string, string>" lang="tsx">
const pick = <U = T,>(x: U) => x;
</script>

`````

## `{"printWidth":100,"vueIndentScriptAndStyle":true,"singleQuote":true,"semi":false}`

`````vue
<script setup generic="T extends Record<string, string>" lang="tsx">
  const pick = <U = T,>(x: U) => x
</script>

`````
