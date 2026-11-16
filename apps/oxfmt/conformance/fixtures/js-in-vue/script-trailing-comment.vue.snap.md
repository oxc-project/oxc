# Input

`````vue
<script setup>
if (
  a &&
  b < c // trailing comment in multiline condition
) {
}

const x = 1; // simple trailing comment
</script>
<template>
  <div>hello</div>
</template>

`````

# Output

## `{"printWidth":80}`

`````vue
<script setup>
if (
  a &&
  b < c // trailing comment in multiline condition
) {
}

const x = 1; // simple trailing comment
</script>
<template>
  <div>hello</div>
</template>

`````

## `{"printWidth":100,"vueIndentScriptAndStyle":true,"singleQuote":true,"semi":false}`

`````vue
<script setup>
  if (
    a &&
    b < c // trailing comment in multiline condition
  ) {
  }

  const x = 1 // simple trailing comment
</script>
<template>
  <div>hello</div>
</template>

`````
