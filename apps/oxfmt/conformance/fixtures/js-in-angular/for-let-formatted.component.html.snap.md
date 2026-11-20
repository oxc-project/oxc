# Input

`````html
@for (item of items;track item.id;let e=$even,o=$odd) {
  {{ item }}
}

`````

# Output

## `{}`

`````html
@for (item of items; track item.id; let e = $even, o = $odd) {
  {{ item }}
}

`````
