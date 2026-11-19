# Input

`````svelte
<script>let   count=$state(0);</script>
<button onclick={()=>count++}>clicks: {count}</button>
<style>button{color:red;}</style>

`````

# Output

## `{"svelte":{}}`

`````svelte
<script>
  let count = $state(0);
</script>

<button onclick={() => count++}>clicks: {count}</button>

<style>
  button {
    color: red;
  }
</style>

`````
