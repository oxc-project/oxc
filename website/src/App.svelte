<script lang="ts">
  import "virtual:windi.css";

  import Header from "./layout/Header.svelte";
  import Footer from "./layout/Footer.svelte";
  import Home from "./pages/Home.svelte";
</script>

<main class="flex flex-col h-full overflow-hidden justify-between">
  <Header />
  <Home />
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
