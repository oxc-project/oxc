# Input

`````svelte
<script>let items = $state([1,2,3]);</script>
{#each items as n}
<div class="p-4 flex bg-red-500 text-white">{n}</div>
{/each}

`````

# Output

## `{"svelte":{},"sortTailwindcss":{}}`

`````svelte
<script>
  let items = $state([1, 2, 3]);
</script>

{#each items as n}
  <div class="flex bg-red-500 p-4 text-white">{n}</div>
{/each}

`````
