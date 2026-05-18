function singleStatementBody(_x) {
  return _singleStatementBody.apply(this, arguments);
}
function _singleStatementBody() {
  _singleStatementBody = babelHelpers.wrapAsyncGenerator(function* (asyncIterable) {
    var _iteratorAbruptCompletion = false;
    var _didIteratorError = false;
    var _iteratorError;
    try {
      for (var _iterator = babelHelpers.asyncIterator(asyncIterable), _step; _iteratorAbruptCompletion = !(_step = yield babelHelpers.awaitAsyncGenerator(_iterator.next())).done; _iteratorAbruptCompletion = false) {
        const chunk = _step.value;
        yield babelHelpers.awaitAsyncGenerator(chunk());
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
  });
  return _singleStatementBody.apply(this, arguments);
}
