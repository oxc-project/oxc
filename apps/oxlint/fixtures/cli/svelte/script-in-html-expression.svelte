<script lang="ts">
    let {json}: {json: string} = $props();
</script>
<svelte:head>
    {@html `<script type="application/ld+json">${json}</script>`}
</svelte:head>
