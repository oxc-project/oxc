# Input

`````vue
<!-- https://github.com/oxc-project/oxc/issues/20428 -->
<script lang="ts">
let foo = 'foo';

let bar = 'bar';
</script>

`````

# Output

## `{"vueIndentScriptAndStyle":true,"sortImports":{},"sortTailwindcss":{}}`

`````vue
<!-- https://github.com/oxc-project/oxc/issues/20428 -->
<script lang="ts">
  let foo = "foo";

  let bar = "bar";
</script>

`````
