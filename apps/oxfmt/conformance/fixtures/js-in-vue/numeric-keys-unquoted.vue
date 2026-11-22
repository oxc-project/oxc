<template>
  <p :b="{ '1': 1, 'b': 2 }">{{ { '0': 'x' }[0] }}</p>
</template>

<script>
const b = { '1': 1 };
</script>
