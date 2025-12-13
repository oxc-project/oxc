<template>
  <div class="wrapper">
    <header class="header">
      <h1>{{ formatted by oxfmt expr }}</h1>
      <p v-if="formatted by oxfmt expr">{{ formatted by oxfmt expr }}</p>

      <!-- Slots -->
      <slot name="actions" />

      <!-- v-for with key + destructuring -->
      <ul>
        <li v-for="formatted by oxfmt in formatted by oxfmt expr" :key="formatted by oxfmt expr">
          {{ formatted by oxfmt expr }}. {{ formatted by oxfmt expr }}
        </li>
      </ul>
    </header>
  </div>
</template>

<script lang="ts" setup>
formatted by oxfmt
</script>
