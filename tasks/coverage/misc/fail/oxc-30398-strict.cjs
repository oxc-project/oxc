class C {
  m() {
    f()++;
  }
}
function g() {
  "use strict";
  for (f() in o);
}
