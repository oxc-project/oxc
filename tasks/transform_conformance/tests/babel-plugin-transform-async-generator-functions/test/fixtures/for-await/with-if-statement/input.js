async function* handleAsyncIterables2(asyncIterable) {
  if (true)
    for await (const chunk of asyncIterable) {
      for (;;) {
          if (delimIndex === -1) {
              // incomplete message, wait for more chunks
              // continue outer;
          }
      }
    }
}