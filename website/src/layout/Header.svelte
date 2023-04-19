<script>
  import { logoSquare } from "../assets/brand";
  import LinkButton from "../components/LinkButton.svelte";
</script>

<header class="flex px-3 bg-gray-800">
  <container
    class="container mx-auto flex justify-between items-center h-80px py-16px"
  >
    <div>
      <a href="/" class="flex items-center">
        <img src={logoSquare} class="w-56px h-56px" alt="OXC Logo" />
        <h1 class="text-xl text-white ml-2">Oxc</h1>
      </a>
    </div>
    <div>
      <span class="flex space-x-4">
        <LinkButton href="/playground">Playground</LinkButton>
        <LinkButton href="https://github.com/Boshen/oxc">GitHub</LinkButton>
      </span>
    </div>
  </container>
</header>
