let f;

class C {
  [f = () => this]() {}
}

function outer() {
  class C {
    [f = () => this]() {}
  }
}
