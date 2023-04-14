<script lang="ts">
  import Header from "./Header.svelte";
  import Footer from "./Footer.svelte";
</script>

<main class="flex flex-col h-full overflow-hidden justify-between">
  <Header />
  <slot />
  <Footer />
</main>

<style>
  main {
    font-family: -apple-system, BlinkMacSystemFont, Segoe UI, Roboto, Oxygen,
      Ubuntu, Cantarell, Fira Sans, Droid Sans, Helvetica Neue, sans-serif;
    min-height: 100vh;
    background-color: #103e5f;
    color: #babc8f;
  }
</style>
