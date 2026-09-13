<!-- DIVERGES: the `<T = any,>` disambiguating comma is removed like plain `.ts`;
  see apps/oxfmt/DIVERGENCES.md "ts-in-vue-generic-trailing-comma" -->
<script setup lang="ts">
const getComponentRef = <T = any,>() => componentRef.value as T;
</script>
