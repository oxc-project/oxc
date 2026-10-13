({}).a++;
(function () {}).a++;
(class {}).a--;
class A {
  #x;
  m() {
    (function () {}).#x;
    ({}).#x;
    (class {}).#x = 1;
  }
}
new ({}).a();
