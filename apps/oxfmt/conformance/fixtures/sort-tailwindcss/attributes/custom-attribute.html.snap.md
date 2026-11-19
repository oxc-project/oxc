# Input

`````html
<div class="p-4 flex" data-classes="text-white bg-red-500">Hello</div>
`````

# Output

## `{"sortTailwindcss":{}}`

`````html
<div class="flex p-4" data-classes="text-white bg-red-500">Hello</div>

`````

## `{"sortTailwindcss":{"attributes":["myClassProp"]}}`

`````html
<div class="flex p-4" data-classes="text-white bg-red-500">Hello</div>

`````

## `{"sortTailwindcss":{"attributes":["data-classes"]}}`

`````html
<div class="flex p-4" data-classes="bg-red-500 text-white">Hello</div>

`````
