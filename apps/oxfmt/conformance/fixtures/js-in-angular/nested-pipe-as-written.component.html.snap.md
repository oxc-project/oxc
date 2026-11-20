# Input

`````html
<p>{{a?b:c|p}}</p>
<p>{{ f(a ? b : c|p) }}</p>
<p>{{ `${a|p}` }}</p>

`````

# Output

## `{}`

`````html
<p>{{a?b:c|p}}</p>
<p>{{ f(a ? b : c|p) }}</p>
<p>{{ `${a|p}` }}</p>

`````
