// Examples of incorrect code for related-getter-setter-pairs rule

class Example {
  private _value: number = 0;

  // Getter and setter with incompatible types
  get value(): string {
    return this._value.toString();
  }

  set value(val: number) {
    // Incompatible with getter
    this._value = val;
  }
}
