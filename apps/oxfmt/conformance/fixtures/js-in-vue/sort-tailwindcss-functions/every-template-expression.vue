<script setup lang="ts">
const a = clsx("p-4 flex");
</script>

<template>
  <p v-if="clsx('p-4 flex')">{{ clsx("p-4 flex") }}</p>
  <p :title="clsx('p-4 flex')" @click="f(clsx('p-4 flex'))" />
</template>
