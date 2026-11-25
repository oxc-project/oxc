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
// A type cast prints its own parens
/** @type {A} */ ({}.a).b;
/** @type {A} */ (function () {}.a);
