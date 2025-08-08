// Examples of incorrect code for unbound-method rule

class MyClass {
  private value = 42;

  getValue() {
    return this.value;
  }

  processValue() {
    return this.value * 2;
  }
}

const obj = new MyClass();

// Unbound method call - 'this' context lost
const getValue = obj.getValue;
const result = getValue(); // 'this' is undefined