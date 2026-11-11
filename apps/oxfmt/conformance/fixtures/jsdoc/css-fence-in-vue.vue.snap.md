# Input

`````vue
<script>
/**
 * ```css
 * .body {color:red;
 * }
 * ```
 */
function foo() {}
</script>

`````

# Output

## `{"jsdoc":{}}`

`````vue
<script>
/**
 * ```css
 * .body {
 *   color: red;
 * }
 * ```
 */
function foo() {}
</script>

`````
