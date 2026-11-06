# Input

`````yaml
a:   1
list:
  - "x"
  -   y
flow: {a: 1, b: [1,2]}

`````

# Output

## `{}`

`````yaml
a: 1
list:
  - "x"
  - y
flow: { a: 1, b: [1, 2] }

`````
