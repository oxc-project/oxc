function f() {
  await using x = null;
  for (await using y = null; ; ) {}
  for (await using z of []) {}
}

function* g() {
  await using x = null;
  for (await using y = null; ; ) {}
  for (await using z of []) {}
}
