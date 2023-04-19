<script lang="ts">
  import Router from 'svelte-spa-router'
  import "virtual:windi.css";

  import Layout from "./layout/Layout.svelte";
  import { routes } from './routers'

</script>

<Layout>
  <Router {routes}/>
</Layout>
