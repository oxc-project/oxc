# Input

`````vue
<script lang="ts">
import z from "z";
import a from "a";

export default { name: "App" };
</script>
<script setup lang="ts">
import m from "m";
import b from "b";

const count = ref(0);
</script>
<template>
  <div>{{ count }}</div>
</template>

`````

# Output

## `{"vueIndentScriptAndStyle":true,"sortImports":{},"sortTailwindcss":{}}`

`````vue
<script lang="ts">
  import a from "a";
  import z from "z";

  export default { name: "App" };
</script>
<script setup lang="ts">
  import b from "b";
  import m from "m";

  const count = ref(0);
</script>
<template>
  <div>{{ count }}</div>
</template>

`````
