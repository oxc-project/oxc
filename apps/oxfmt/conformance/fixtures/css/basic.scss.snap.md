# Input

`````scss
$gap:8px;.card{margin:$gap;&:hover{color:blue}}
`````

# Output

## `{}`

`````scss
$gap: 8px;
.card {
  margin: $gap;
  &:hover {
    color: blue;
  }
}

`````
