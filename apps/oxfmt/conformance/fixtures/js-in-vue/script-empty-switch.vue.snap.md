# Input

`````vue
<script>
switch (foo) {
}
</script>
<template>
  <div>hello</div>
</template>

`````

# Output

## `{"printWidth":80}`

`````vue
<script>
switch (foo) {
}
</script>
<template>
  <div>hello</div>
</template>

`````

## `{"printWidth":100,"vueIndentScriptAndStyle":true,"singleQuote":true,"semi":false}`

`````vue
<script>
  switch (foo) {
  }
</script>
<template>
  <div>hello</div>
</template>

`````
