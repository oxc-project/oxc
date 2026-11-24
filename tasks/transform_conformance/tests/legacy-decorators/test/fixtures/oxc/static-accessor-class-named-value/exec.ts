class value {
  static accessor x = 1;
  accessor y = 1;
}

value.x = 2;
expect(value.x).toBe(2);

const instance = new value();
instance.y = 3;
expect(instance.y).toBe(3);
