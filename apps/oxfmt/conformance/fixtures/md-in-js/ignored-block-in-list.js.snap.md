# Input

`````js
const doc = markdown`
  - inner

    <!-- prettier-ignore -->
    | a | b |
    |--|--|
`;

`````

# Output

## `{"printWidth":80}`

`````js
const doc = markdown`
  - inner

    <!-- prettier-ignore -->
    | a | b |
    |--|--|
`;

`````

## `{"printWidth":100,"proseWrap":"always"}`

`````js
const doc = markdown`
  - inner

    <!-- prettier-ignore -->
    | a | b |
    |--|--|
`;

`````
