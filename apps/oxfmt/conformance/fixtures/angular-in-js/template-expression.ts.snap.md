# Input

`````ts
@Component({
  template: `<ul><li>${item}</li></ul>`,
  styles: `div { color: ${color} }`,
})
class MyComponent {}

`````

# Output

## `{"printWidth":80}`

`````ts
@Component({
  template: `<ul>
    <li>${item}</li>
  </ul>`,
  styles: `
    div {
      color: ${color};
    }
  `,
})
class MyComponent {}

`````

## `{"printWidth":100,"htmlWhitespaceSensitivity":"ignore"}`

`````ts
@Component({
  template: `
    <ul>
      <li>${item}</li>
    </ul>
  `,
  styles: `
    div {
      color: ${color};
    }
  `,
})
class MyComponent {}

`````
