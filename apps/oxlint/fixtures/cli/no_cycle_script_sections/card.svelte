<script module lang="ts">
  import { registry } from './registry';

  export const kind = registry;
</script>

<script lang="ts">
  import { theme } from './theme';

  const color = theme;
</script>

<div>{color}</div>
