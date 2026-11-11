<template>
  <div :class="{a:1}" v-if="a&&b">{{  a+b  }}</div>
</template>

<script setup lang="ts" generic="T extends string,U">
const a = 1
</script>
<style   scoped>
.a{color:red}
</style>
<!-- display: inline -->
<i18n lang="json">
{"en":{"hello":"Hello"}}
</i18n>
<docs>
# kept
  as written
</docs>
<docs lang="md" src="./docs.md"></docs>
<template lang="pug">
div
  p {{ kept }}
</template>
<custom lang="unknown">   </custom>
