for (using x = (a in b); a in b; a in b) {
  a in b;
}
a in b;

async function f() {
  for (await using x = (a in b); a in b; a in b) {
    a in b;
  }
  a in b;
}
