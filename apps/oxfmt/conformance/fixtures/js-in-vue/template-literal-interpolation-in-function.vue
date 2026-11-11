<script setup>
const f = () => {
  s.value = `
${items ? Object.entries(items).map(([k, v]) => k + v).join("") : ""}
`
}
</script>
