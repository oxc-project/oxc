# Input

`````vue
<script setup lang="ts">
import z from "z";
  import a from "a";
    import m from "m";

const count = ref(0);
</script>
<template>
  <div class="p-4 flex m-0">{{ count }}</div>
  <button class="m-2 p-1 flex">Click</button>
  <p>Templates are formatted as well...
    </p>
</template>

<style>
.and { css: too !important }
</style>

`````

# Output

## `{"vueIndentScriptAndStyle":true,"sortImports":{},"sortTailwindcss":{}}`

`````vue
<script setup lang="ts">
  import a from "a";
  import m from "m";
  import z from "z";

  const count = ref(0);
</script>
<template>
  <div class="m-0 flex p-4">{{ count }}</div>
  <button class="m-2 flex p-1">Click</button>
  <p>Templates are formatted as well...</p>
</template>

<style>
  .and {
    css: too !important;
  }
</style>

`````
