let f;

class C extends (f = () => this, class {}) {}

function outer() {
  class C extends (f = () => this, class {}) {}
}
