# Input

`````html
<p>{{   a  b   }}</p>
@if (a   b) {
  x
}
@for (let   item of items;index as i) {
  x
}
@let = 1;

`````

# Output

## `{}`

`````html
<p>{{   a  b   }}</p>
@if (a   b) {
  x
}
@for (let   item of items; index as i) {
  x
}
@let = 1;

`````
