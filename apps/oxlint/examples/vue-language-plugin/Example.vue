<script setup lang="ts">
import { ref } from "vue";
const count = ref(0);
function inc() {
  count.value++;
}
</script>

<template>
  <button class="primary" @click="inc">
    {{ count }}
  </button>
</template>
