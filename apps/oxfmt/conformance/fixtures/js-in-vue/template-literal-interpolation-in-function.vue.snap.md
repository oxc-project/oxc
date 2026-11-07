# Input

`````vue
<script setup>
const f = () => {
  s.value = `
${items ? Object.entries(items).map(([k, v]) => k + v).join("") : ""}
`
}
</script>

`````

# Output

## `{"printWidth":80}`

`````vue
<script setup>
const f = () => {
  s.value = `
${
  items
    ? Object.entries(items)
        .map(([k, v]) => k + v)
        .join("")
    : ""
}
`;
};
</script>

`````

## `{"printWidth":100,"vueIndentScriptAndStyle":true,"singleQuote":true,"semi":false}`

`````vue
<script setup>
  const f = () => {
    s.value = `
${
  items
    ? Object.entries(items)
        .map(([k, v]) => k + v)
        .join('')
    : ''
}
`
  }
</script>

`````
