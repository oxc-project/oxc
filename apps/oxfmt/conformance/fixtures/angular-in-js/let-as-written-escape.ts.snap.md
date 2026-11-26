# Input

`````ts
// A `@let` value kept as written (a line comment ends it) lays out the same with or without a re-escaped `\`
@Component({
  template: `@let a = f('a\\b') // c
;`,
})
class A {}

@Component({
  template: `@let b = f('ab') // c
;`,
})
class B {}

`````

# Output

## `{"printWidth":80}`

`````ts
// A `@let` value kept as written (a line comment ends it) lays out the same with or without a re-escaped `\`
@Component({
  template: `@let a = f('a\\b') // c
;`,
})
class A {}

@Component({
  template: `@let b = f('ab') // c
;`,
})
class B {}

`````

## `{"printWidth":100,"htmlWhitespaceSensitivity":"ignore"}`

`````ts
// A `@let` value kept as written (a line comment ends it) lays out the same with or without a re-escaped `\`
@Component({
  template: `
    @let a = f('a\\b') // c
;
  `,
})
class A {}

@Component({
  template: `
    @let b = f('ab') // c
;
  `,
})
class B {}

`````
