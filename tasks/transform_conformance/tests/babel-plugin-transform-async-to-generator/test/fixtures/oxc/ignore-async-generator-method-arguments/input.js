const object = {
  async *values() {
    return arguments;
  },
};

class Values {
  async *values() {
    return arguments;
  }
}
