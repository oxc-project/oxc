# Input

`````vue
<script lang="tsx">
const identity = <T,>(x: T) => x;
const el = <div>hi</div>;
</script>

`````

# Output

## `{"printWidth":80}`

`````vue
<script lang="tsx">
const identity = <T,>(x: T) => x;
const el = <div>hi</div>;
</script>

`````

## `{"printWidth":100,"vueIndentScriptAndStyle":true,"singleQuote":true,"semi":false}`

`````vue
<script lang="tsx">
  const identity = <T,>(x: T) => x
  const el = <div>hi</div>
</script>

`````
