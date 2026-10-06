async function f(using) {
  for (await using;;) { break; }
  for (await using + 1;;) { break; }
  for (await using.foo;;) { break; }
  for (await using(1);;) { break; }
  for (await (using);;) { break; }
  for (await using
    ;;) { break; }
  for (await
    using;;) { break; }
}
