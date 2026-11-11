# Input

`````html
<button onclick="x=a &amp; b">a</button>
<button onclick="x=a&&b">b</button>

`````

# Output

## `{"printWidth":80}`

`````html
<button onclick="x=a &amp; b">a</button>
<button onclick="x = a && b">b</button>

`````

## `{"printWidth":100,"htmlWhitespaceSensitivity":"ignore","singleAttributePerLine":true}`

`````html
<button onclick="x=a &amp; b">a</button>
<button onclick="x = a && b">b</button>

`````

## `{"printWidth":120,"htmlWhitespaceSensitivity":"strict","bracketSameLine":true}`

`````html
<button onclick="x=a &amp; b">a</button>
<button onclick="x = a && b">b</button>

`````
