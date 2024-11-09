function handleAsyncIterables2(_x) {
  return _handleAsyncIterables.apply(this, arguments);
}
function _handleAsyncIterables() {
  _handleAsyncIterables = babelHelpers.wrapAsyncGenerator(function* (asyncIterable) {
    if (true) {
      var _iteratorAbruptCompletion = false;
      var _didIteratorError = false;
      var _iteratorError;
      try {
        for (var _iterator = babelHelpers.asyncIterator(asyncIterable), _step; _iteratorAbruptCompletion = !(_step = yield babelHelpers.awaitAsyncGenerator(_iterator.next())).done; _iteratorAbruptCompletion = false) {
          const chunk = _step.value;
          {
            for (;;) {
              if (delimIndex === -1) {
                // incomplete message, wait for more chunks
                // continue outer;
              }
            }
          }
        }
      } catch (err) {
        _didIteratorError = true;
        _iteratorError = err;
      } finally {
        try {
          if (_iteratorAbruptCompletion && _iterator.return != null) {
            yield babelHelpers.awaitAsyncGenerator(_iterator.return());
          }
        } finally {
          if (_didIteratorError) {
            throw _iteratorError;
          }
        }
      }
    }
  });
  return _handleAsyncIterables.apply(this, arguments);
}