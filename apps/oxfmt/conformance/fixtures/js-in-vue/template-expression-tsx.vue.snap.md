# Input

`````vue
<script setup lang="tsx">
const a = <div />;
</script>

<template>
  <p>{{ (value  as  Item).name }}</p>
</template>

`````

# Output

## `{"printWidth":80}`

`````vue
<script setup lang="tsx">
const a = <div />;
</script>

<template>
  <p>{{ (value as Item).name }}</p>
</template>

`````

## `{"printWidth":100,"vueIndentScriptAndStyle":true,"singleQuote":true,"semi":false}`

`````vue
<script setup lang="tsx">
  const a = <div />
</script>

<template>
  <p>{{ (value as Item).name }}</p>
</template>

`````
