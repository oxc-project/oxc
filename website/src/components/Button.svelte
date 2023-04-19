<script lang="ts">
  export let href: string = "";
</script>

<a
  class="inline-flex items-center justify-center
    px-3 py-2
    border border-transparent rounded-md
    bg-primary
    hover:bg-primary-600
    shadow-sm
    text-base text-white font-medium
    transition duration-50
    ring-primary focus:ring-3 ring-opacity-50 text-base px-3 py-2.5 md:text-lg md:px-4 md:py-3 border-2"
  {href}
  target="_blank"><slot /></a
>
