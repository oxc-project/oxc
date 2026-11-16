# Input

`````vue
<script setup lang="ts">
const a = clsx("p-4 flex");
</script>

<template>
  <p v-if="clsx('p-4 flex')">{{ clsx("p-4 flex") }}</p>
  <p :title="clsx('p-4 flex')" @click="f(clsx('p-4 flex'))" />
</template>

`````

# Output

## `{"sortTailwindcss":{"functions":["clsx"]}}`

`````vue
<script setup lang="ts">
const a = clsx("flex p-4");
</script>

<template>
  <p v-if="clsx('flex p-4')">{{ clsx("flex p-4") }}</p>
  <p :title="clsx('flex p-4')" @click="f(clsx('flex p-4'))" />
</template>

`````
