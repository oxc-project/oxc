// Test declare fields with computed keys
const KEY1 = "dynamicKey1";
const KEY2 = "dynamicKey2";

class TestClass {
  // declare field with computed key should be removed
  declare [KEY1]: string;
  
  // regular field with computed key should be transformed
  [KEY2]: number = 42;
  
  // static declare field with computed key should be removed
  declare static [KEY1 + "Static"]: boolean;
  
  // regular static field with computed key should be transformed
  static [KEY2 + "Static"]: string = "static";
}
