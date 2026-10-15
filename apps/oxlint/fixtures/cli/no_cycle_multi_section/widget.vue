<script lang="ts">
import { helper } from './helper';

export default { name: 'Widget', helper };
</script>

<script setup lang="ts">
import { other } from './other';

const label = other;
</script>

<template>
  <div>{{ label }}</div>
</template>
