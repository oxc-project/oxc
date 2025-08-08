// Examples of incorrect code for unbound-method rule

class Calculator {
  private value: number = 0;

  add(num: number): number {
    this.value += num;
    return this.value;
  }

  getValue(): number {
    return this.value;
  }
}

const calc = new Calculator();

// Unbound method - loses 'this' context
const addMethod = calc.add; 
addMethod(5); // Error: 'this' is undefined

// Array method callback loses context
const getValue = calc.getValue;
[1, 2, 3].map(getValue); // Error: each call loses 'this'

// Unbound method in setTimeout
setTimeout(calc.add, 1000, 10); // Error: 'this' context lost

// Class method destructuring
const { getValue: getVal } = calc;
getVal(); // Error: 'this' context lost