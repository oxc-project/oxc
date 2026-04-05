<script context="module" lang="ts">
  export const prerender = true;
</script>

<script lang="ts">
  import type { User } from "./types";

  export let user: User = { name: "world" };
</script>

<style>
  h1 {
    color: red;
  }
</style>

<h1>Hello {user.name}</h1>
