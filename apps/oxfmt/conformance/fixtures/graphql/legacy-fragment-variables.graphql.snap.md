# Input

`````graphql
"Frag description" fragment F("var description" $x: Int) on T { f(arg: $x) }
`````

# Output

## `{}`

`````graphql
"Frag description"
fragment F(
  "var description"
  $x: Int
) on T {
  f(arg: $x)
}

`````
