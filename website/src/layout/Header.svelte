<script>
  import logo from "../assets/logo-square.png";
  import LinkButton from "../components/LinkButton.svelte";
</script>

<header class="flex px-3 bg-gray-800">
  <container
    class="container mx-auto flex justify-between items-center h-80px py-16px"
  >
    <div>
      <a href="/" class="flex items-center">
        <img src={logo} class="w-56px h-56px" alt="OXC Logo" />
        <h1 class="text-xl text-white ml-2">OXC</h1>
      </a>
    </div>
    <div>
      <span class="links">
        <LinkButton href="https://github.com/Boshen/oxc">GitHub</LinkButton>
      </span>
    </div>
  </container>
</header>
