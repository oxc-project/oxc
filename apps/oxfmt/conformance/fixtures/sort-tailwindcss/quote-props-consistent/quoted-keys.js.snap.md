# Input

`````js
const a = clsx({ "p": x, "a b": y, "1": z, " ": w });
const b = clsx({ p: x, c: y });
const c = clsx(`a${{ " p ": x }}b`);

`````

# Output

## `{"sortTailwindcss":{"functions":["clsx"]}}`

`````js
const a = clsx({ p: x, "a b": y, 1: z, " ": w });
const b = clsx({ p: x, c: y });
const c = clsx(`a${{ " p ": x }}b`);

`````

## `{"sortTailwindcss":{"functions":["clsx"]},"quoteProps":"consistent"}`

`````js
const a = clsx({ "p": x, "a b": y, "1": z, " ": w });
const b = clsx({ p: x, c: y });
const c = clsx(`a${{ " p ": x }}b`);

`````
