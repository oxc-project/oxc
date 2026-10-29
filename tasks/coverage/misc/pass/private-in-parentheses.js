class PrivateInParentheses {
  #outer;
  #inner;

  check(value) {
    return #outer in (#inner in value);
  }
}
