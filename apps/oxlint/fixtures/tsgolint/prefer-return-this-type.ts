class Builder {
  private value: string = '';
  setValue(value: string): Builder {
    this.value = value;
    return this;
  }
}