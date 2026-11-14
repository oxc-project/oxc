# Input

`````html
<div other="other" value="value" type="type" title="title" src="src" role="role" name="name" id="id" href="href" for="for" data-z="z" class="class" aria-label="Label" data-a="a" alt="alt"></div>

`````

# Output

## `{"sortAttributes":{"groups":["^data-","^data-a","$DEFAULT"]}}`

`````html
<div
  data-a="a"
  data-z="z"
  alt="alt"
  aria-label="Label"
  class="class"
  for="for"
  href="href"
  id="id"
  name="name"
  other="other"
  role="role"
  src="src"
  title="title"
  type="type"
  value="value"
></div>

`````

## `{"sortAttributes":{"groups":["$DEFAULT","^data-"]}}`

`````html
<div
  alt="alt"
  aria-label="Label"
  class="class"
  for="for"
  href="href"
  id="id"
  name="name"
  other="other"
  role="role"
  src="src"
  title="title"
  type="type"
  value="value"
  data-a="a"
  data-z="z"
></div>

`````

## `{"sortAttributes":{"groups":["$CODE_GUIDE"]}}`

`````html
<div
  class="class"
  id="id"
  name="name"
  data-a="a"
  data-z="z"
  src="src"
  for="for"
  type="type"
  href="href"
  value="value"
  title="title"
  alt="alt"
  role="role"
  aria-label="Label"
  other="other"
></div>

`````
