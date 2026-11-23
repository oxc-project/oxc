# Input

`````vue
<script setup>
import { ref } from "vue";
import clsx from "clsx";

const count = ref(0);
const cls = clsx("p-4 flex");
</script>
<template>
  <div class="flex p-4">{{count}}</div>
  <div class="p-4 flex">{{count}}</div>
</template>

`````

# Output

## `{"vueIndentScriptAndStyle":true,"sortImports":{},"sortTailwindcss":{"functions":["clsx"]}}`

`````vue
<script setup>
  import clsx from "clsx";
  import { ref } from "vue";

  const count = ref(0);
  const cls = clsx("flex p-4");
</script>
<template>
  <div class="flex p-4">{{ count }}</div>
  <div class="flex p-4">{{ count }}</div>
</template>

`````
