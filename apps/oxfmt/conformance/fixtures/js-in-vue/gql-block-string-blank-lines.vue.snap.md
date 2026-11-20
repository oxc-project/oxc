# Input

`````vue
<script setup>
const q = graphql`
  """
  First paragraph.


  Second paragraph after two blanks.
  """
  type Query {
    hello: String
  }
`;
</script>

`````

# Output

## `{"printWidth":80}`

`````vue
<script setup>
const q = graphql`
  """
  First paragraph.


  Second paragraph after two blanks.
  """
  type Query {
    hello: String
  }
`;
</script>

`````

## `{"printWidth":100,"vueIndentScriptAndStyle":true,"singleQuote":true,"semi":false}`

`````vue
<script setup>
  const q = graphql`
    """
    First paragraph.


    Second paragraph after two blanks.
    """
    type Query {
      hello: String
    }
  `
</script>

`````
