let _ref;
// Test declare fields with computed keys
const KEY1 = "dynamicKey1";
const KEY2 = "dynamicKey2";
_ref = KEY2 + "Static";

class TestClass {
  // regular field with computed key should be transformed
  constructor() {
    babelHelpers.defineProperty(this, KEY2, 42);
  }
  
  // regular static field with computed key should be transformed
}

babelHelpers.defineProperty(TestClass, _ref, "static");
