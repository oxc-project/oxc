<script lang="ts">
  export let href: string = "";
</script>

<a
  class="inline-flex items-center justify-center text-base font-medium transition duration-50"
  {href}
  target="_blank"><slot /></a
>

<style>
  a {
    color: #a4e7e4;
  }
  a:hover {
    color: #ea8b5d;
  }
</style>
