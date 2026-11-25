function* g(o) {
  for (yield (1 in o); false;);
  for (x = yield (1 in o); false;);
  for (yield yield (1 in o); false;);
  for (yield (x = (1 in o)); false;);
  for ((yield (1 in o)) + 1; false;);
  yield (1 in o);

  for (yield* (1 in o); false;);
  for (x = yield* (1 in o); false;);
  for (yield* yield (1 in o); false;);
  for (yield* (x = (1 in o)); false;);
  for ((yield* (1 in o)) + 1; false;);
  yield* (1 in o);
}
