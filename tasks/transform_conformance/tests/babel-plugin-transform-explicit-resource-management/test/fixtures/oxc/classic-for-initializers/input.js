function run() {
  outer: inner: for (using first = getFirst(), second = getSecond(); keepGoing(); advance()) {
    if (skip()) continue outer;
    if (stop()) break inner;
    use(first, second);
  }
}

async function runAsync() {
  for (await using first = getFirst(), second = getSecond(); keepGoing(); advance()) {
    use(first, second);
  }
}
