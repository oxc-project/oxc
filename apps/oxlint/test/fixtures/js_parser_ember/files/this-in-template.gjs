// `this` used only inside a template must count as `this` usage for native rules:
// `{{this.name}}` is a Glimmer path with a `ThisHead`, which gets injected into the
// shadow source placeholder as `${this}`. `class-methods-use-this` must flag only
// `farewell`, not `greeting`.
//
// Note this is deliberately better than ESLint: ESLint's core rule only knows
// `ThisExpression`, so it cannot see `this` inside templates and flags both methods
// (see eslint.snap.md).
class Greeter {
  name = "world";

  greeting() {
    return <template><p>hello {{this.name}}</p></template>;
  }

  farewell() {
    return <template><p>goodbye</p></template>;
  }
}

export { Greeter };
