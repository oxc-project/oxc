<script>
  import { logoRound } from "../../assets";
  import LinkButton from "../components/LinkButton.svelte";

  const footerGroups = [
    {
      name: "Docs",
      children: [
        {
          label: "About",
          link: "/about",
        },
        {
          label: "Usage",
          link: "/usage",
        },
      ],
    },
    {
      name: "Community",
      children: [
        {
          label: "Twitter",
          link: "",
        },
      ],
    },
    {
      name: "More",
      children: [
        {
          label: "GitHub",
          link: "https://github.com/Boshen/oxc",
        },
        {
          label: "Issues",
          link: "https://github.com/Boshen/oxc/issues",
        },
      ],
    },
  ];
</script>

<footer class="flex px-3 bg-gray-800">
  <container class="container mx-auto py-4">
    <div class="flex">
      <ul class="flex-1">
        <img src={logoRound} class="w-56px h-56px" alt="OXC Logo" />
      </ul>
      {#each footerGroups as group}
        <ul class="flex-1">
          <h4>{group.name}</h4>
          {#each group.children as item}
            <li>
              <LinkButton href={item.link}>{item.label}</LinkButton>
            </li>
          {/each}
        </ul>
      {/each}
    </div>
    <div class="text-center pt-2">
      <p class="text-sm">MIT Licensed | Copyright © 2023</p>
    </div>
  </container>
</footer>

<style>
</style>
