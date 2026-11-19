# Input

`````svelte
<script>let count = $state(0);</script>
<p>{count}</p>

`````

# Output

## `{}`

### Errors

`````
Cannot format `.svelte`: `svelte` plugin is not enabled in resolved config: fixtures/svelte/disabled/basic.svelte
`````

## `{"svelte":false}`

### Errors

`````
Cannot format `.svelte`: `svelte` plugin is not enabled in resolved config: fixtures/svelte/disabled/basic.svelte
`````
