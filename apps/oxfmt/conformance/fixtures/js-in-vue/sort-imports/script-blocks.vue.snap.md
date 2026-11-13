# Input

`````vue
<script lang="ts">
import z from "z";
  import a from "a";
    import m from "m";

</script>
<script lang="ts" setup>
import z from "z";
  import a from "a";
    import m from "m";

</script>
<template> <div>{{a+m+z}}</div> </template>

`````

# Output

## `{"vueIndentScriptAndStyle":true,"sortImports":{}}`

`````vue
<script lang="ts">
  import a from "a";
  import m from "m";
  import z from "z";
</script>
<script lang="ts" setup>
  import a from "a";
  import m from "m";
  import z from "z";
</script>
<template><div>{{ a + m + z }}</div></template>

`````
