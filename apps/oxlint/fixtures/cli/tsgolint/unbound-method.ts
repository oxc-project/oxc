class Calculator {
  private value: number = 0;
  add(num: number): number {
    this.value += num;
    return this.value;
  }
}
const calc = new Calculator();
const addMethod = calc.add;