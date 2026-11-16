# Input

`````vue
<script setup>
const a = `
  hello
  world
`;
</script>
<template>
  <div>{{ a }}</div>
</template>

`````

# Output

## `{"printWidth":80}`

`````vue
<script setup>
const a = `
  hello
  world
`;
</script>
<template>
  <div>{{ a }}</div>
</template>

`````

## `{"printWidth":100,"vueIndentScriptAndStyle":true,"singleQuote":true,"semi":false}`

`````vue
<script setup>
  const a = `
  hello
  world
`
</script>
<template>
  <div>{{ a }}</div>
</template>

`````
