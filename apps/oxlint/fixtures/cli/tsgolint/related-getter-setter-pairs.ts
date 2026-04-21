class Example {
  private _value: number = 0;
  get value(): string {
    return this._value.toString();
  }
  set value(val: number) {
    this._value = val;
  }
}