# Input

`````svelte
<style>button{color:red;}</style>
<script>let count=$state(0);</script>
<button onclick={()=>count++}>{count}</button>

`````

# Output

## `{"svelte":{}}`

`````svelte
<script>
  let count = $state(0);
</script>

<button onclick={() => count++}>{count}</button>

<style>
  button {
    color: red;
  }
</style>

`````

## `{"svelte":{"sortOrder":"styles-options-markup-scripts"}}`

`````svelte
<style>
  button {
    color: red;
  }
</style>

<button onclick={() => count++}>{count}</button>

<script>
  let count = $state(0);
</script>

`````
