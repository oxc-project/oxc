# Input

`````svelte
<script lang="ts">
import type { Z } from "zoo";
import { a } from "ant";
import type { A } from "ant";
let n: number = $state(0);
</script>
<p>{n}</p>

`````

# Output

## `{"svelte":{},"sortImports":{"order":"asc"}}`

`````svelte
<script lang="ts">
  import { a } from "ant";
  import type { A } from "ant";
  import type { Z } from "zoo";
  let n: number = $state(0);
</script>

<p>{n}</p>

`````
