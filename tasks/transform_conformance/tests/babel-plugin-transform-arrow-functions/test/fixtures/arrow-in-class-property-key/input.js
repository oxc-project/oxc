let f;

class C {
  [f = () => this] = 1;
}

function outer() {
  class C {
    [f = () => this] = 1;
  }
}
