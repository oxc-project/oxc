# Input

`````vue
<script lang="ts">
export const identity=<T>(x:T):T=>x;
</script>

`````

# Output

## `{"printWidth":80}`

`````vue
<script lang="ts">
export const identity = <T>(x: T): T => x;
</script>

`````

## `{"printWidth":100,"vueIndentScriptAndStyle":true,"singleQuote":true,"semi":false}`

`````vue
<script lang="ts">
  export const identity = <T>(x: T): T => x
</script>

`````
