// Examples of incorrect code for prefer-return-this-type rule

class Builder {
  private value: string = '';

  setValue(value: string): Builder { // Should return 'this'
    this.value = value;
    return this;
  }

  build(): string {
    return this.value;
  }
}