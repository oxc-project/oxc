# Input

`````html
<script>
  const query = gql`
    query {
      field # <\/script>
    }
  `;
</script>

`````

# Output

## `{"printWidth":80}`

`````html
<script>
  const query = gql`
    query {
      field # <\/script>
    }
  `;
</script>

`````

## `{"printWidth":100,"htmlWhitespaceSensitivity":"ignore","singleAttributePerLine":true}`

`````html
<script>
  const query = gql`
    query {
      field # <\/script>
    }
  `;
</script>

`````

## `{"printWidth":120,"htmlWhitespaceSensitivity":"strict","bracketSameLine":true}`

`````html
<script>
  const query = gql`
    query {
      field # <\/script>
    }
  `;
</script>

`````
