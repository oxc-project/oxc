# Input

`````html
<div [y]="{ a: 1 } // c" (z)="a() // c"></div>

`````

# Output

## `{}`

`````html
<div
  [y]="
    { a: 1 } // c
  "
  (z)="
    a() // c
  "
></div>

`````
