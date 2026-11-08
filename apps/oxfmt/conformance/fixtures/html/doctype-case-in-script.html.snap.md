# Input

`````html
<script>
  const page = html`<!DOCTYPE html>
    <html>
      <head><title>x</title></head>
    </html>`;
</script>

`````

# Output

## `{"printWidth":80}`

`````html
<script>
  const page = html`<!DOCTYPE html>
    <html>
      <head>
        <title>x</title>
      </head>
    </html>`;
</script>

`````

## `{"printWidth":100,"htmlWhitespaceSensitivity":"ignore","singleAttributePerLine":true}`

`````html
<script>
  const page = html`
    <!DOCTYPE html>
    <html>
      <head>
        <title>x</title>
      </head>
    </html>
  `;
</script>

`````

## `{"printWidth":120,"htmlWhitespaceSensitivity":"strict","bracketSameLine":true}`

`````html
<script>
  const page = html`<!DOCTYPE html>
    <html>
      <head
        ><title>x</title></head
      >
    </html>`;
</script>

`````
