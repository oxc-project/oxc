class Example {
  private value = 1;
  private readonly ok = 2;

  constructor() {
    this.value = 2;
  }

  mutate() {
    return this.ok;
  }
}
