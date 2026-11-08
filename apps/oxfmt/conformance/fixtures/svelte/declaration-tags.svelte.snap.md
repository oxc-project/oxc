# Input

`````svelte
{#each boxes as box}
{const area = box.width * box.height}
{const label = `${box.width} x ${box.height} = ${area}`}
<p>{label}</p>
{/each}

`````

# Output

## `{"svelte":{}}`

`````svelte
{#each boxes as box}
  {const area = box.width * box.height}
  {const label = `${box.width} x ${box.height} = ${area}`}
  <p>{label}</p>
{/each}

`````
