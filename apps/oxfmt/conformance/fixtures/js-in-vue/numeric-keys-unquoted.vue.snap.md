# Input

`````vue
<template>
  <p :b="{ '1': 1, 'b': 2 }">{{ { '0': 'x' }[0] }}</p>
</template>

<script>
const b = { '1': 1 };
</script>

`````

# Output

## `{"printWidth":80}`

`````vue
<template>
  <p :b="{ 1: 1, b: 2 }">{{ { 0: "x" }[0] }}</p>
</template>

<script>
const b = { 1: 1 };
</script>

`````

## `{"printWidth":100,"vueIndentScriptAndStyle":true,"singleQuote":true,"semi":false}`

`````vue
<template>
  <p :b="{ 1: 1, b: 2 }">{{ { 0: 'x' }[0] }}</p>
</template>

<script>
  const b = { 1: 1 }
</script>

`````
