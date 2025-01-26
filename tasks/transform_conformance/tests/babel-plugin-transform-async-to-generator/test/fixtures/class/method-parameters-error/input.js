class Cls {
  // ReferenceError: Cannot access 'b' before initialization
  async method(a = b, b = 0) {}
}
