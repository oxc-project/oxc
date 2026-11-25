# Input

`````ts
const a = tw.style("p-4  flex");
const b = cn.a("p-4  flex").b("m-2  block").c();
const c = cn.foo["a  b"]("x  y");
const d = cn(x.y("a  b"), cn.x("p-4  flex"));

`````

# Output

## `{"sortTailwindcss":{"functions":["cn","tw"]}}`

`````ts
const a = tw.style("flex p-4");
const b = cn.a("flex p-4").b("m-2 block").c();
const c = cn.foo["a  b"]("x y");
const d = cn(x.y("a  b"), cn.x("flex p-4"));

`````
