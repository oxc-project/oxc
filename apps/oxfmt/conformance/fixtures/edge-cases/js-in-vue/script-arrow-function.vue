<script setup>
watchEffect(() => {
  const x = 1;
});

const fn = computed(() => {
  return "hello";
});
</script>
<template>
  <div>{{ fn }}</div>
</template>
