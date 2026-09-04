const object = {
  async *values() {
    yield 1;
  },
};

class Values {
  async *values() {
    yield 2;
  }
}
