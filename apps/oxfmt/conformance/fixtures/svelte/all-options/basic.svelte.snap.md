# Input

`````svelte
<script>
let value = $state("x");
</script>
<input bind:value={value} />
<style>
input { color: red; }
</style>

`````

# Output

## `{"svelte":{"sortOrder":"styles-options-markup-scripts","allowShorthand":false,"indentScriptAndStyle":false}}`

`````svelte
<style>
input {
  color: red;
}
</style>

<input bind:value={value} />

<script>
let value = $state("x");
</script>

`````
