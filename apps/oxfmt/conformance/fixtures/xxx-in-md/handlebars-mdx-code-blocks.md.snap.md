# Input

`````md
```hbs
<div   class="a">{{#if b}}  c  {{/if}}</div>
```

```handlebars
{{#each items  as |item|}}<p>{{item}}</p>{{/each}}
```

```mdx
import  {A} from './a'

#   Title

<A   b="c" />
```

`````

# Output

## `{"printWidth":80}`

`````md
```hbs
<div class="a">{{#if b}} c {{/if}}</div>
```

```handlebars
{{#each items as |item|}}<p>{{item}}</p>{{/each}}
```

```mdx
import { A } from "./a";

# Title

<A b="c" />
```

`````
