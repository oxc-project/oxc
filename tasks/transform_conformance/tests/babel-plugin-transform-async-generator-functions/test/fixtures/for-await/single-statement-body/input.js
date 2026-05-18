async function* singleStatementBody(asyncIterable) {
  for await (const chunk of asyncIterable) await chunk();
}
