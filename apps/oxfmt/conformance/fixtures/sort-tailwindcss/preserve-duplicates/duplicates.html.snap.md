# Input

`````html
<div class="flex p-4 flex p-4">Hello</div>
`````

# Output

## `{"sortTailwindcss":{}}`

`````html
<div class="flex p-4">Hello</div>

`````

## `{"sortTailwindcss":{"preserveDuplicates":true}}`

`````html
<div class="flex flex p-4 p-4">Hello</div>

`````
