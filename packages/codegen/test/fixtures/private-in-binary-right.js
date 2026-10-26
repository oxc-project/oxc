class C {
  #x;
  test(a, b) {
    return [
      #x in (a instanceof b),
      #x in (a in b),
      #x in (a < b),
      #x in (a <= b),
      #x in (a > b),
      #x in (a >= b),
      #x in (a == b),
      #x in (a != b),
      #x in (a === b),
      #x in (a !== b),
      #x in (a & b),
      #x in (a ^ b),
      #x in (a | b),
      #x in (a && b),
      #x in (a || b),
      #x in (a ?? b),
      #x in (a = b),
      #x in (a << b),
      #x in (a >> b),
      #x in (a >>> b),
      #x in (a + b),
      #x in (a - b),
      #x in (a * b),
      #x in (a / b),
      #x in (a % b),
      #x in (a ** b),
      #x in (#x in a),
      #x in (a ? a : b),
      #x in (a, b),
    ];
  }
}
