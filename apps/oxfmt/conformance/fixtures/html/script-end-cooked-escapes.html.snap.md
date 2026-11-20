# Input

`````html
<script>
  const a = html`<div><script>a<\/script></div>`;
  const b = html`<div><script>b</\x73cript></div>`;
</script>

`````

# Output

## `{"printWidth":80}`

`````html
<script>
  const a = html`<div>
    <script>
      a;
    <\/script>
  </div>`;
  const b = html`<div>
    <script>
      b;
    <\/script>
  </div>`;
</script>

`````

## `{"printWidth":100,"htmlWhitespaceSensitivity":"ignore","singleAttributePerLine":true}`

`````html
<script>
  const a = html`
    <div>
      <script>
        a;
      <\/script>
    </div>
  `;
  const b = html`
    <div>
      <script>
        b;
      <\/script>
    </div>
  `;
</script>

`````

## `{"printWidth":120,"htmlWhitespaceSensitivity":"strict","bracketSameLine":true}`

`````html
<script>
  const a = html`<div
    ><script>
      a;
    <\/script></div
  >`;
  const b = html`<div
    ><script>
      b;
    <\/script></div
  >`;
</script>

`````
