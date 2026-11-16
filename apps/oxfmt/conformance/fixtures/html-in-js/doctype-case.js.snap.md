# Input

`````js
export const page = html`<!DOCTYPE html>
  <html>
    <head><title>x</title></head>
  </html>`;

`````

# Output

## `{"printWidth":80}`

`````js
export const page = html`<!DOCTYPE html>
  <html>
    <head>
      <title>x</title>
    </head>
  </html>`;

`````

## `{"printWidth":100,"htmlWhitespaceSensitivity":"ignore"}`

`````js
export const page = html`
  <!DOCTYPE html>
  <html>
    <head>
      <title>x</title>
    </head>
  </html>
`;

`````
