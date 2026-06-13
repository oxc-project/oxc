<template>
  <div>test</div>
</template>

<script setup lang="ts">
import { ref } from "vue";

const test = ref();

// ok
test.value = "a";
</script>
