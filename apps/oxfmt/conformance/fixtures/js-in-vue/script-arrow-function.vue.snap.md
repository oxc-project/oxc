# Input

`````vue
<script setup>
watchEffect(() => {
  const x = 1;
});

const fn = computed(() => {
  return "hello";
});
</script>
<template>
  <div>{{ fn }}</div>
</template>

`````

# Output

## `{"printWidth":80}`

`````vue
<script setup>
watchEffect(() => {
  const x = 1;
});

const fn = computed(() => {
  return "hello";
});
</script>
<template>
  <div>{{ fn }}</div>
</template>

`````

## `{"printWidth":100,"vueIndentScriptAndStyle":true,"singleQuote":true,"semi":false}`

`````vue
<script setup>
  watchEffect(() => {
    const x = 1
  })

  const fn = computed(() => {
    return 'hello'
  })
</script>
<template>
  <div>{{ fn }}</div>
</template>

`````
